use inscribe_hir as _;
use inscribe_typeck as _;

pub mod borrow_check;
pub mod const_eval;
pub mod determinism;
pub mod lower;
pub mod nodes;

pub use borrow_check::{check_mutable_assignments, BorrowIssue};
pub use const_eval::{evaluate_constant_rvalue, fold_block_constants};
pub use determinism::{find_nondeterministic_calls, DeterminismIssue};
pub use lower::lower_program;
pub use nodes::{
    BasicBlockData, BasicBlockId, Constant, ConstantValue, LocalDecl, LocalId, MatchTarget,
    MirFunction, MirProgram, Operand, Place, ProjectionElem, Rvalue, Statement, StatementKind,
    TerminatorKind,
};

// TODO: Add a pipeline facade that lowers source all the way to MIR once the session owns orchestration.

#[cfg(test)]
mod tests {
    use inscribe_hir::lower_module;
    use inscribe_lexer::lex;
    use inscribe_parser::parse_module;
    use inscribe_resolve::resolve_module;
    use inscribe_typeck::check_module;

    use crate::{
        check_mutable_assignments, find_nondeterministic_calls, lower_program, TerminatorKind,
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
    fn flags_nondeterministic_calls_by_name() {
        let source = r#"
fn random_value() -> int {
    4
}

fn main() -> int {
    random_value()
}
"#;

        let tokens = lex(source).expect("lexing should succeed");
        let module = parse_module(tokens).expect("parsing should succeed");
        let resolved = resolve_module(&module).expect("resolution should succeed");
        let typed = check_module(&module, &resolved).expect("type checking should succeed");
        let hir = lower_module(&module, &resolved, &typed);
        let mir = lower_program(&hir);

        let issues = find_nondeterministic_calls(&mir);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].callee.contains("random_value"));
    }
}
