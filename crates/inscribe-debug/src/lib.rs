use inscribe_mir::MirProgram;

pub mod dwarf;
pub mod source_map;
pub mod var_tracking;

pub use dwarf::{
    emit_function_dwarf, emit_function_dwarf_with_sources, emit_program_dwarf,
    emit_program_dwarf_with_sources, DwarfFunction, DwarfLineRow, DwarfUnit, DwarfVariable,
};
pub use source_map::{SourceFile, SourceFileId, SourceLocation, SourceMap, SourceRange};
pub use var_tracking::{
    track_variables, ProgramPoint, VariableReport, VariableSummary, VariableUse, VariableUseKind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDebugInfo {
    pub name: String,
    pub variables: VariableReport,
    pub dwarf: DwarfFunction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramDebugInfo {
    pub dwarf: DwarfUnit,
    pub functions: Vec<FunctionDebugInfo>,
}

pub fn build_program_debug_info(program: &MirProgram) -> ProgramDebugInfo {
    ProgramDebugInfo {
        dwarf: emit_program_dwarf(program),
        functions: program
            .functions
            .iter()
            .map(|function| FunctionDebugInfo {
                name: qualified_function_name(function),
                variables: track_variables(function),
                dwarf: emit_function_dwarf(function),
            })
            .collect(),
    }
}

fn qualified_function_name(function: &inscribe_mir::MirFunction) -> String {
    function
        .receiver
        .as_ref()
        .map(|receiver| format!("{receiver}.{}", function.name))
        .unwrap_or_else(|| function.name.clone())
}

#[cfg(test)]
mod tests {
    use inscribe_ast::span::{Position, Span};
    use inscribe_mir::{BasicBlockData, BasicBlockId, LocalDecl, LocalId, MirFunction, MirProgram, TerminatorKind};
    use inscribe_resolve::FunctionKey;
    use inscribe_typeck::{FunctionSignature, Type};

    use crate::build_program_debug_info;

    #[test]
    fn builds_program_debug_summary() {
        let span = Span::new(Position::new(0, 1, 1), Position::new(8, 1, 9));
        let program = MirProgram {
            functions: vec![MirFunction {
                receiver: None,
                name: "main".to_string(),
                signature: FunctionSignature {
                    key: FunctionKey {
                        receiver: None,
                        name: "main".to_string(),
                    },
                    params: Vec::new(),
                    return_type: Box::new(Type::Unit),
                },
                is_declaration: false,
                locals: vec![LocalDecl {
                    id: LocalId(0),
                    name: "_return".to_string(),
                    ty: Type::Unit,
                    mutable: true,
                    temp: true,
                    span,
                }],
                blocks: vec![BasicBlockData {
                    id: BasicBlockId(0),
                    statements: Vec::new(),
                    terminator: TerminatorKind::Return,
                }],
                entry: BasicBlockId(0),
                return_local: LocalId(0),
                span,
            }],
            span,
        };

        let info = build_program_debug_info(&program);

        assert_eq!(info.functions.len(), 1);
        assert_eq!(info.functions[0].name, "main");
        assert_eq!(info.dwarf.functions[0].name, "main");
    }
}
