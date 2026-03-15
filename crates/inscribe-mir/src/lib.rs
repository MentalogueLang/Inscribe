use inscribe_hir as _;
use inscribe_typeck as _;

pub mod borrow_check;
pub mod const_eval;
pub mod determinism;
pub mod lower;
pub mod nodes;
pub mod optimize;

pub use borrow_check::{check_mutable_assignments, BorrowIssue};
pub use const_eval::{evaluate_constant_rvalue, fold_block_constants, fold_function_constants};
pub use determinism::{find_nondeterministic_calls, DeterminismIssue};
pub use lower::lower_program;
pub use nodes::{
    BasicBlockData, BasicBlockId, Constant, ConstantValue, LocalDecl, LocalId, MatchTarget,
    MirFunction, MirProgram, Operand, Place, ProjectionElem, Rvalue, Statement, StatementKind,
    TerminatorKind,
};
pub use optimize::{optimize_function, optimize_program};

// TODO: Add a pipeline facade that lowers source all the way to MIR once the session owns orchestration.

#[cfg(test)]
mod tests {
    use inscribe_hir::lower_module;
    use inscribe_lexer::lex;
    use inscribe_parser::parse_module;
    use inscribe_resolve::resolve_module;
    use inscribe_typeck::check_module;

    use crate::{
        check_mutable_assignments, find_nondeterministic_calls, fold_function_constants,
        lower_program, optimize_function, Constant, ConstantValue, Operand, Rvalue, StatementKind,
        TerminatorKind,
    };

    #[test]
    fn lowers_control_flow_into_cfg_blocks() {
        let source = r#"
fn main() -> int {
    let sum = 0
    let limit = 3

    while limit > 0 {
        sum = sum + 1
        limit = limit - 1
    }

    sum
}
"#;

        let tokens = lex(source).expect("lexing should succeed");
        let module = parse_module(tokens).expect("parsing should succeed");
        let resolved = resolve_module(&module).expect("resolution should succeed");
        let typed = check_module(&module, &resolved).expect("type checking should succeed");
        let hir = lower_module(&module, &resolved, &typed);
        let mir = lower_program(&hir);

        let function = &mir.functions[0];
        assert!(function.blocks.len() >= 4);
        assert!(function
            .blocks
            .iter()
            .any(|block| matches!(block.terminator, TerminatorKind::Branch { .. })));
        assert!(matches!(
            function.blocks.last().map(|block| &block.terminator),
            Some(TerminatorKind::Return) | Some(TerminatorKind::Unreachable)
        ));
        assert!(check_mutable_assignments(function).is_empty());
    }

    #[test]
    fn flags_calls_with_nondeterministic_capabilities() {
        let source = r#"
fn host_magic(value: int)

fn helper() -> int {
    host_magic(4)
    4
}

fn main() -> int {
    helper()
}
"#;

        let tokens = lex(source).expect("lexing should succeed");
        let module = parse_module(tokens).expect("parsing should succeed");
        let resolved = resolve_module(&module).expect("resolution should succeed");
        let typed = check_module(&module, &resolved).expect("type checking should succeed");
        let hir = lower_module(&module, &resolved, &typed);
        let mir = lower_program(&hir);

        let issues = find_nondeterministic_calls(&mir);
        assert_eq!(issues.len(), 2);
        assert!(issues.iter().any(|issue| issue.function == "helper" && issue.callee == "host_magic"));
        assert!(issues.iter().any(|issue| issue.function == "main" && issue.callee == "helper"));
    }

    #[test]
    fn keeps_deterministic_runtime_capabilities_allowed() {
        let source = r#"
fn print_int(value: int)

fn main() -> int {
    print_int(4)
    0
}
"#;

        let tokens = lex(source).expect("lexing should succeed");
        let module = parse_module(tokens).expect("parsing should succeed");
        let resolved = resolve_module(&module).expect("resolution should succeed");
        let typed = check_module(&module, &resolved).expect("type checking should succeed");
        let hir = lower_module(&module, &resolved, &typed);
        let mir = lower_program(&hir);

        let issues = find_nondeterministic_calls(&mir);
        assert!(issues.is_empty());
    }

    #[test]
    fn folds_constants_across_cfg_edges() {
        let source = r#"
fn main() -> int {
    let value = 4

    if true {
        let doubled = value + value
        doubled
    } else {
        0
    }
}
"#;

        let tokens = lex(source).expect("lexing should succeed");
        let module = parse_module(tokens).expect("parsing should succeed");
        let resolved = resolve_module(&module).expect("resolution should succeed");
        let typed = check_module(&module, &resolved).expect("type checking should succeed");
        let hir = lower_module(&module, &resolved, &typed);
        let mut mir = lower_program(&hir);

        let function = &mut mir.functions[0];
        fold_function_constants(function);

        assert!(function.blocks.iter().any(|block| {
            block.statements.iter().any(|statement| {
                matches!(
                    &statement.kind,
                    StatementKind::Assign(_, Rvalue::Use(Operand::Constant(constant)))
                        if matches!(constant.value, ConstantValue::Integer(ref value) if value == "8")
                )
            })
        }));
        assert!(matches!(
            function.blocks[1].terminator,
            TerminatorKind::Goto { .. }
        ));
    }

    #[test]
    fn optimizer_removes_unreachable_blocks() {
        let source = r#"
fn main() -> int {
    if true {
        1
    } else {
        2
    }
}
"#;

        let tokens = lex(source).expect("lexing should succeed");
        let module = parse_module(tokens).expect("parsing should succeed");
        let resolved = resolve_module(&module).expect("resolution should succeed");
        let typed = check_module(&module, &resolved).expect("type checking should succeed");
        let hir = lower_module(&module, &resolved, &typed);
        let mut mir = lower_program(&hir);

        let function = &mut mir.functions[0];
        let before = function.blocks.len();
        optimize_function(function);

        assert!(function.blocks.len() < before);
        assert!(function
            .blocks
            .iter()
            .all(|block| !matches!(block.terminator, TerminatorKind::Branch { .. })));
    }

    #[test]
    fn optimizer_simplifies_identity_arithmetic() {
        let source = r#"
fn passthrough(value: int) -> int {
    let multiplied = value * 1
    multiplied + 0
}
"#;

        let tokens = lex(source).expect("lexing should succeed");
        let module = parse_module(tokens).expect("parsing should succeed");
        let resolved = resolve_module(&module).expect("resolution should succeed");
        let typed = check_module(&module, &resolved).expect("type checking should succeed");
        let hir = lower_module(&module, &resolved, &typed);
        let mut mir = lower_program(&hir);

        let function = mir
            .functions
            .iter_mut()
            .find(|function| function.name == "passthrough")
            .expect("passthrough function should exist");
        optimize_function(function);

        assert!(function.blocks.iter().all(|block| {
            block
                .statements
                .iter()
                .all(|statement| !is_identity_binary(statement))
        }));
    }

    #[test]
    fn optimizer_prunes_unreachable_functions() {
        let source = r#"
fn helper() -> int {
    9
}

fn unused() -> int {
    3
}

fn main() -> int {
    helper()
}
"#;

        let tokens = lex(source).expect("lexing should succeed");
        let module = parse_module(tokens).expect("parsing should succeed");
        let resolved = resolve_module(&module).expect("resolution should succeed");
        let typed = check_module(&module, &resolved).expect("type checking should succeed");
        let hir = lower_module(&module, &resolved, &typed);
        let mut mir = lower_program(&hir);

        crate::optimize_program(&mut mir);

        assert_eq!(mir.functions.len(), 2);
        assert!(mir
            .functions
            .iter()
            .any(|function| function.name == "helper"));
        assert!(mir
            .functions
            .iter()
            .all(|function| function.name != "unused"));
    }

    fn is_identity_binary(statement: &crate::Statement) -> bool {
        let StatementKind::Assign(_, Rvalue::BinaryOp { op, left, right }) = &statement.kind else {
            return false;
        };

        (op == "Multiply" && is_integer_operand(right, "1"))
            || (op == "Add" && (is_integer_operand(left, "0") || is_integer_operand(right, "0")))
    }

    fn is_integer_operand(operand: &Operand, expected: &str) -> bool {
        matches!(
            operand,
            Operand::Constant(Constant {
                value: ConstantValue::Integer(value),
                ..
            }) if value == expected
        )
    }
}
