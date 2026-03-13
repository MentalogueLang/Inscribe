use inscribe_ast as _;
use inscribe_typeck as _;

pub mod lower;
pub mod nodes;
pub mod pretty;

pub use lower::lower_module;
pub use nodes::{
    HirBinding, HirBlock, HirExpr, HirExprKind, HirField, HirFor, HirFunction, HirImport, HirItem,
    HirMatchArm, HirParam, HirProgram, HirStmt, HirStruct, HirWhile,
};
pub use pretty::render;

// TODO: Add a pipeline facade that can emit HIR directly from source once the session owns orchestration.

#[cfg(test)]
mod tests {
    use inscribe_lexer::lex;
    use inscribe_parser::parse_module;
    use inscribe_resolve::resolve_module;
    use inscribe_typeck::check_module;

    use crate::{lower_module, render};

    #[test]
    fn lowers_a_typed_program_into_hir() {
        let source = r#"
struct User {
    name: string
}

fn User.greet(self) -> string {
    self.name
}

fn main() -> int {
    let user = User { name: "Antonio" }
    let label = user.greet()
    label
    1
}
"#;

        let tokens = lex(source).expect("lexing should succeed");
        let module = parse_module(tokens).expect("parsing should succeed");
        let resolved = resolve_module(&module).expect("resolution should succeed");
        let typed = check_module(&module, &resolved).expect("type checking should succeed");
        let hir = lower_module(&module, &resolved, &typed);
        let printed = render(&hir);

        assert!(printed.contains("fn User.greet -> string"));
        assert!(printed.contains("let label: string"));
        assert!(printed.contains("User {...}: User"));
    }

    #[test]
    fn reports_type_errors_before_lowering() {
        let source = r#"
fn main() -> int {
    let value: int = "oops"
    value
}
"#;

        let tokens = lex(source).expect("lexing should succeed");
        let module = parse_module(tokens).expect("parsing should succeed");
        let resolved = resolve_module(&module).expect("resolution should succeed");
        let errors = check_module(&module, &resolved).expect_err("type checking should fail");

        assert!(errors
            .iter()
            .any(|error| error.message.contains("type mismatch")));
    }
}
