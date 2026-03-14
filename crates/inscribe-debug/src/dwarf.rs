use inscribe_ast::span::Span;
use inscribe_mir::{MirFunction, MirProgram};

use crate::source_map::{SourceFileId, SourceMap, SourceRange};
use crate::var_tracking::{track_variables, ProgramPoint};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DwarfLineRow {
    pub address: u64,
    pub span: Span,
    pub source: Option<SourceRange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DwarfVariable {
    pub name: String,
    pub ty: String,
    pub mutable: bool,
    pub temp: bool,
    pub live_from: Option<ProgramPoint>,
    pub live_until: Option<ProgramPoint>,
    pub span: Span,
    pub source: Option<SourceRange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DwarfFunction {
    pub name: String,
    pub low_pc: u64,
    pub high_pc: u64,
    pub span: Span,
    pub source: Option<SourceRange>,
    pub lines: Vec<DwarfLineRow>,
    pub variables: Vec<DwarfVariable>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DwarfUnit {
    pub name: String,
    pub functions: Vec<DwarfFunction>,
}

pub fn emit_program_dwarf(program: &MirProgram) -> DwarfUnit {
    emit_program_dwarf_with_sources(program, None, None)
}

pub fn emit_program_dwarf_with_sources(
    program: &MirProgram,
    source_map: Option<&SourceMap>,
    file: Option<SourceFileId>,
) -> DwarfUnit {
    DwarfUnit {
        name: "inscribe".to_string(),
        functions: program
            .functions
            .iter()
            .map(|function| emit_function_dwarf_with_sources(function, source_map, file))
            .collect(),
    }
}

pub fn emit_function_dwarf(function: &MirFunction) -> DwarfFunction {
    emit_function_dwarf_with_sources(function, None, None)
}

pub fn emit_function_dwarf_with_sources(
    function: &MirFunction,
    source_map: Option<&SourceMap>,
    file: Option<SourceFileId>,
) -> DwarfFunction {
    let source = source_map.and_then(|map| file.and_then(|file| map.resolve_span(file, function.span)));
    let lines = build_line_rows(function, source_map, file);
    let report = track_variables(function);
    let variables = report
        .summaries
        .into_iter()
        .map(|summary| {
            let span = function
                .locals
                .get(summary.local.0)
                .map(|local| local.span)
                .unwrap_or(summary.last_span);
            DwarfVariable {
                name: summary.name,
                ty: summary.ty,
                mutable: summary.mutable,
                temp: summary.temp,
                live_from: summary.live_from,
                live_until: summary.live_until,
                span,
                source: source_map.and_then(|map| file.and_then(|file| map.resolve_span(file, span))),
            }
        })
        .collect::<Vec<_>>();

    DwarfFunction {
        name: qualified_function_name(function),
        low_pc: 0,
        high_pc: lines.len() as u64,
        span: function.span,
        source,
        lines,
        variables,
    }
}

fn build_line_rows(
    function: &MirFunction,
    source_map: Option<&SourceMap>,
    file: Option<SourceFileId>,
) -> Vec<DwarfLineRow> {
    let mut rows = Vec::new();
    let mut address = 0_u64;

    for block in &function.blocks {
        for statement in &block.statements {
            rows.push(DwarfLineRow {
                address,
                span: statement.span,
                source: source_map.and_then(|map| file.and_then(|file| map.resolve_span(file, statement.span))),
            });
            address += 1;
        }

        rows.push(DwarfLineRow {
            address,
            span: function.span,
            source: source_map.and_then(|map| file.and_then(|file| map.resolve_span(file, function.span))),
        });
        address += 1;
    }

    rows
}

fn qualified_function_name(function: &MirFunction) -> String {
    function
        .receiver
        .as_ref()
        .map(|receiver| format!("{receiver}.{}", function.name))
        .unwrap_or_else(|| function.name.clone())
}

#[cfg(test)]
mod tests {
    use inscribe_ast::span::{Position, Span};
    use inscribe_mir::{
        BasicBlockData, BasicBlockId, LocalDecl, LocalId, MirFunction, MirProgram, Statement,
        StatementKind, TerminatorKind,
    };
    use inscribe_resolve::FunctionKey;
    use inscribe_typeck::{FunctionSignature, Type};

    use crate::dwarf::emit_program_dwarf_with_sources;
    use crate::source_map::SourceMap;

    fn sample_program() -> (MirProgram, SourceMap, crate::source_map::SourceFileId) {
        let span = Span::new(Position::new(0, 1, 1), Position::new(14, 2, 4));
        let function = MirFunction {
            receiver: None,
            name: "main".to_string(),
            signature: FunctionSignature {
                key: FunctionKey {
                    receiver: None,
                    name: "main".to_string(),
                },
                params: Vec::new(),
                return_type: Box::new(Type::Int),
            },
            is_declaration: false,
            locals: vec![
                LocalDecl {
                    id: LocalId(0),
                    name: "_return".to_string(),
                    ty: Type::Int,
                    mutable: true,
                    temp: true,
                    span,
                },
                LocalDecl {
                    id: LocalId(1),
                    name: "value".to_string(),
                    ty: Type::Int,
                    mutable: true,
                    temp: false,
                    span,
                },
            ],
            blocks: vec![BasicBlockData {
                id: BasicBlockId(0),
                statements: vec![
                    Statement {
                        kind: StatementKind::StorageLive(LocalId(1)),
                        span,
                    },
                    Statement {
                        kind: StatementKind::StorageDead(LocalId(1)),
                        span,
                    },
                ],
                terminator: TerminatorKind::Return,
            }],
            entry: BasicBlockId(0),
            return_local: LocalId(0),
            span,
        };
        let program = MirProgram {
            functions: vec![function],
            span,
        };
        let mut source_map = SourceMap::new();
        let file = source_map.add_file("main.ins", "fn main() {\n  1\n}\n");
        (program, source_map, file)
    }

    #[test]
    fn emits_function_and_variable_records() {
        let (program, source_map, file) = sample_program();
        let unit = emit_program_dwarf_with_sources(&program, Some(&source_map), Some(file));

        assert_eq!(unit.name, "inscribe");
        assert_eq!(unit.functions.len(), 1);
        assert_eq!(unit.functions[0].name, "main");
        assert_eq!(unit.functions[0].variables.len(), 2);
        assert!(unit.functions[0].source.is_some());
        assert!(unit.functions[0].lines.len() >= 2);
    }
}
