use inscribe_ast::span::Span;
use inscribe_mir::{
    BasicBlockId, LocalId, MirFunction, Operand, Place, Rvalue, StatementKind, TerminatorKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableUseKind {
    StorageLive,
    StorageDead,
    Read,
    Write,
    Drop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramPoint {
    Statement {
        block: BasicBlockId,
        index: usize,
    },
    Terminator {
        block: BasicBlockId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableUse {
    pub local: LocalId,
    pub name: String,
    pub kind: VariableUseKind,
    pub point: ProgramPoint,
    pub span: Span,
    pub projection: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableSummary {
    pub local: LocalId,
    pub name: String,
    pub ty: String,
    pub mutable: bool,
    pub temp: bool,
    pub reads: usize,
    pub writes: usize,
    pub live_from: Option<ProgramPoint>,
    pub live_until: Option<ProgramPoint>,
    pub last_span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableReport {
    pub uses: Vec<VariableUse>,
    pub summaries: Vec<VariableSummary>,
}

impl VariableReport {
    pub fn user_variables(&self) -> impl Iterator<Item = &VariableSummary> {
        self.summaries.iter().filter(|summary| !summary.temp)
    }

    pub fn summary(&self, local: LocalId) -> Option<&VariableSummary> {
        self.summaries.iter().find(|summary| summary.local == local)
    }
}

pub fn track_variables(function: &MirFunction) -> VariableReport {
    let mut summaries = function
        .locals
        .iter()
        .map(|local| VariableSummary {
            local: local.id,
            name: local.name.clone(),
            ty: local.ty.display_name(),
            mutable: local.mutable,
            temp: local.temp,
            reads: 0,
            writes: 0,
            live_from: None,
            live_until: None,
            last_span: local.span,
        })
        .collect::<Vec<_>>();
    let mut uses = Vec::new();

    for block in &function.blocks {
        for (index, statement) in block.statements.iter().enumerate() {
            let point = ProgramPoint::Statement {
                block: block.id,
                index,
            };
            match &statement.kind {
                StatementKind::StorageLive(local) => {
                    record_use(
                        &mut uses,
                        &mut summaries,
                        *local,
                        VariableUseKind::StorageLive,
                        point,
                        statement.span,
                        Vec::new(),
                    );
                }
                StatementKind::StorageDead(local) => {
                    record_use(
                        &mut uses,
                        &mut summaries,
                        *local,
                        VariableUseKind::StorageDead,
                        point,
                        statement.span,
                        Vec::new(),
                    );
                }
                StatementKind::Drop(local) => {
                    record_use(
                        &mut uses,
                        &mut summaries,
                        *local,
                        VariableUseKind::Drop,
                        point,
                        statement.span,
                        Vec::new(),
                    );
                }
                StatementKind::Assign(place, value) => {
                    record_place_use(
                        &mut uses,
                        &mut summaries,
                        place,
                        VariableUseKind::Write,
                        point,
                        statement.span,
                    );
                    collect_rvalue_uses(&mut uses, &mut summaries, value, point, statement.span);
                }
                StatementKind::Nop => {}
            }
        }

        let point = ProgramPoint::Terminator { block: block.id };
        match &block.terminator {
            TerminatorKind::Goto { .. } | TerminatorKind::Return | TerminatorKind::Unreachable => {}
            TerminatorKind::Branch { condition, .. } => {
                collect_operand_use(&mut uses, &mut summaries, condition, point, function.span);
            }
            TerminatorKind::Match { discriminant, .. } => {
                collect_operand_use(
                    &mut uses,
                    &mut summaries,
                    discriminant,
                    point,
                    function.span,
                );
            }
            TerminatorKind::Call {
                callee,
                args,
                destination,
                ..
            } => {
                collect_operand_use(&mut uses, &mut summaries, callee, point, function.span);
                for arg in args {
                    collect_operand_use(&mut uses, &mut summaries, arg, point, function.span);
                }
                if let Some(destination) = destination {
                    record_place_use(
                        &mut uses,
                        &mut summaries,
                        destination,
                        VariableUseKind::Write,
                        point,
                        function.span,
                    );
                }
            }
            TerminatorKind::IterNext {
                iterator, binding, ..
            } => {
                record_place_use(
                    &mut uses,
                    &mut summaries,
                    iterator,
                    VariableUseKind::Read,
                    point,
                    function.span,
                );
                record_use(
                    &mut uses,
                    &mut summaries,
                    *binding,
                    VariableUseKind::Write,
                    point,
                    function.span,
                    Vec::new(),
                );
            }
            TerminatorKind::Try {
                operand,
                ok_local,
                err_local,
                ..
            } => {
                collect_operand_use(&mut uses, &mut summaries, operand, point, function.span);
                record_use(
                    &mut uses,
                    &mut summaries,
                    *ok_local,
                    VariableUseKind::Write,
                    point,
                    function.span,
                    Vec::new(),
                );
                record_use(
                    &mut uses,
                    &mut summaries,
                    *err_local,
                    VariableUseKind::Write,
                    point,
                    function.span,
                    Vec::new(),
                );
            }
        }
    }

    VariableReport { uses, summaries }
}

fn collect_rvalue_uses(
    uses: &mut Vec<VariableUse>,
    summaries: &mut [VariableSummary],
    value: &Rvalue,
    point: ProgramPoint,
    span: Span,
) {
    match value {
        Rvalue::Use(operand) | Rvalue::ResultOk(operand) | Rvalue::ResultErr(operand) => {
            collect_operand_use(uses, summaries, operand, point, span);
        }
        Rvalue::UnaryOp { operand, .. } => collect_operand_use(uses, summaries, operand, point, span),
        Rvalue::BinaryOp { left, right, .. } => {
            collect_operand_use(uses, summaries, left, point, span);
            collect_operand_use(uses, summaries, right, point, span);
        }
        Rvalue::AggregateStruct { fields, .. } => {
            for (_, operand) in fields {
                collect_operand_use(uses, summaries, operand, point, span);
            }
        }
        Rvalue::AggregateArray { elements } => {
            for operand in elements {
                collect_operand_use(uses, summaries, operand, point, span);
            }
        }
        Rvalue::RepeatArray { value, .. } => {
            collect_operand_use(uses, summaries, value, point, span);
        }
    }
}

fn collect_operand_use(
    uses: &mut Vec<VariableUse>,
    summaries: &mut [VariableSummary],
    operand: &Operand,
    point: ProgramPoint,
    span: Span,
) {
    match operand {
        Operand::Copy(place) | Operand::Move(place) => {
            record_place_use(uses, summaries, place, VariableUseKind::Read, point, span);
        }
        Operand::Constant(_) => {}
    }
}

fn record_place_use(
    uses: &mut Vec<VariableUse>,
    summaries: &mut [VariableSummary],
    place: &Place,
    kind: VariableUseKind,
    point: ProgramPoint,
    span: Span,
) {
    record_use(
        uses,
        summaries,
        place.local,
        kind,
        point,
        span,
        place.projection.iter().map(projection_name).collect(),
    );
}

fn projection_name(projection: &inscribe_mir::ProjectionElem) -> String {
    match projection {
        inscribe_mir::ProjectionElem::Field(name) => name.clone(),
        inscribe_mir::ProjectionElem::Index(_) => "[index]".to_string(),
    }
}

fn record_use(
    uses: &mut Vec<VariableUse>,
    summaries: &mut [VariableSummary],
    local: LocalId,
    kind: VariableUseKind,
    point: ProgramPoint,
    span: Span,
    projection: Vec<String>,
) {
    let Some(summary) = summaries.get_mut(local.0) else {
        return;
    };

    match kind {
        VariableUseKind::StorageLive => {
            summary.live_from = summary.live_from.or(Some(point));
        }
        VariableUseKind::StorageDead | VariableUseKind::Drop => {
            summary.live_until = Some(point);
        }
        VariableUseKind::Read => summary.reads += 1,
        VariableUseKind::Write => summary.writes += 1,
    }
    summary.last_span = span;

    uses.push(VariableUse {
        local,
        name: summary.name.clone(),
        kind,
        point,
        span,
        projection,
    });
}

#[cfg(test)]
mod tests {
    use inscribe_ast::span::{Position, Span};
    use inscribe_mir::{
        BasicBlockData, BasicBlockId, Constant, ConstantValue, LocalDecl, LocalId, MirFunction,
        Operand, Place, Rvalue, Statement, StatementKind, TerminatorKind,
    };
    use inscribe_resolve::FunctionKey;
    use inscribe_typeck::{FunctionSignature, Type};

    use crate::var_tracking::{track_variables, ProgramPoint, VariableUseKind};

    fn sample_function() -> MirFunction {
        let span = Span::new(Position::new(0, 1, 1), Position::new(12, 1, 13));
        MirFunction {
            receiver: None,
            name: "main".to_string(),
            signature: FunctionSignature {
                key: FunctionKey {
                    receiver: None,
                    name: "main".to_string(),
                },
                params: vec![Type::Int],
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
                    mutable: false,
                    temp: false,
                    span,
                },
                LocalDecl {
                    id: LocalId(2),
                    name: "sum".to_string(),
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
                        kind: StatementKind::StorageLive(LocalId(2)),
                        span,
                    },
                    Statement {
                        kind: StatementKind::Assign(
                            Place::new(LocalId(2)),
                            Rvalue::Use(Operand::Copy(Place::new(LocalId(1)))),
                        ),
                        span,
                    },
                    Statement {
                        kind: StatementKind::Assign(
                            Place::new(LocalId(0)),
                            Rvalue::BinaryOp {
                                op: "Add".to_string(),
                                left: Operand::Copy(Place::new(LocalId(2))),
                                right: Operand::Constant(Constant {
                                    ty: Type::Int,
                                    value: ConstantValue::Integer("1".to_string()),
                                }),
                            },
                        ),
                        span,
                    },
                    Statement {
                        kind: StatementKind::StorageDead(LocalId(2)),
                        span,
                    },
                ],
                terminator: TerminatorKind::Return,
            }],
            entry: BasicBlockId(0),
            return_local: LocalId(0),
            span,
        }
    }

    #[test]
    fn tracks_reads_writes_and_liveness() {
        let report = track_variables(&sample_function());
        let sum = report.summary(LocalId(2)).expect("sum local should exist");

        assert_eq!(sum.reads, 1);
        assert_eq!(sum.writes, 1);
        assert_eq!(
            sum.live_from,
            Some(ProgramPoint::Statement {
                block: BasicBlockId(0),
                index: 0
            })
        );
        assert_eq!(
            sum.live_until,
            Some(ProgramPoint::Statement {
                block: BasicBlockId(0),
                index: 3
            })
        );
        assert!(report
            .uses
            .iter()
            .any(|usage| usage.kind == VariableUseKind::Read && usage.local == LocalId(2)));
    }
}
