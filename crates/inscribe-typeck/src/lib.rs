use inscribe_ast as _;
use inscribe_resolve as _;
use inscribe_session as _;

pub mod check;
pub mod errors;
pub mod infer;
pub mod ownership;
pub mod unify;

pub use check::check_module;
pub use errors::TypeError;
pub use infer::{expr_key, BindingInfo, BindingKind, FunctionSignature, Type, TypeCheckResult};
use inscribe_ast::nodes::Module;
use inscribe_resolve::{resolve_module, ResolvedProgram};

pub fn analyze_module(
    module: &Module,
) -> Result<(ResolvedProgram, TypeCheckResult), Vec<TypeError>> {
    let resolved = resolve_module(module).map_err(|errors| {
        errors
            .into_iter()
            .map(|error| TypeError::new(error.message, error.span))
            .collect::<Vec<_>>()
    })?;
    let typed = check_module(module, &resolved)?;
    Ok((resolved, typed))
}

#[cfg(test)]
mod tests {
    use inscribe_lexer::lex;
    use inscribe_parser::parse_module;

    use crate::analyze_module;

    #[test]
    fn typechecks_result_constructor_calls() {
        let source = r#"
fn wrap(value: int) -> Result<int, Error> {
    Ok(value)
}

fn rethrow(error: Error) -> Result<int, Error> {
    Err(error)
}
"#;

        let tokens = lex(source).expect("lexing should succeed");
        let module = parse_module(tokens).expect("parsing should succeed");

        analyze_module(&module).expect("constructors should typecheck");
    }
}
