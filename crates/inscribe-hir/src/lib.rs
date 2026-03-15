use inscribe_ast as _;
use inscribe_session as _;
use inscribe_typeck as _;

pub mod lower;
pub mod nodes;
pub mod pretty;

use inscribe_lexer::lex;
use inscribe_parser::parse_module;
use inscribe_session::SessionError;
use inscribe_typeck::TypeError;

pub use lower::lower_module;
pub use nodes::{
    HirBinding, HirBlock, HirExpr, HirExprKind, HirField, HirFor, HirFunction, HirImport, HirItem,
    HirMatchArm, HirParam, HirProgram, HirStmt, HirStruct, HirWhile,
};
pub use pretty::render;

pub fn lower_source(source: &str) -> Result<HirProgram, SessionError> {
    let tokens = lex(source).map_err(|error| SessionError::new("lex", error.to_string()))?;
    let module =
        parse_module(tokens).map_err(|error| SessionError::new("parse", error.to_string()))?;
    let resolved = inscribe_resolve::resolve_module(&module)
        .map_err(|errors| join_errors("resolve", errors.into_iter().map(|error| error.to_string())))?;
    let typed = inscribe_typeck::check_module(&module, &resolved)
        .map_err(|errors| join_type_errors(errors))?;
    Ok(lower_module(&module, &resolved, &typed))
}

fn join_errors<I>(stage: &'static str, errors: I) -> SessionError
where
    I: IntoIterator<Item = String>,
{
    SessionError::new(stage, errors.into_iter().collect::<Vec<_>>().join("\n"))
}

fn join_type_errors(errors: Vec<TypeError>) -> SessionError {
    join_errors("typeck", errors.into_iter().map(|error| error.to_string()))
}

#[cfg(test)]
mod tests {
    use inscribe_lexer::lex;
    use inscribe_parser::parse_module;
    use inscribe_resolve::resolve_module;
    use inscribe_typeck::check_module;

    use crate::{lower_module, lower_source, render};

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
    fn lowers_source_directly_into_hir() {
        let source = r#"
fn main() -> int {
    1
}
"#;

        let hir = lower_source(source).expect("source should lower through the facade");
        let printed = render(&hir);

        assert!(printed.contains("fn main -> int"));
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
