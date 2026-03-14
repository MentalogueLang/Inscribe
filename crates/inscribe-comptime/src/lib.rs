use inscribe_mir::MirProgram;
use inscribe_typeck::Type;

pub mod boundary;
pub mod comptime_alloc;
pub mod interpreter;
pub mod reflect;

pub use boundary::{
    constant_to_value, value_to_constant, ComptimeError, ComptimeResult, ComptimeValue, RangeValue,
    StructValue,
};
pub use comptime_alloc::{ComptimeAllocId, ComptimeAllocator};
pub use interpreter::{Interpreter, InterpreterConfig};
pub use reflect::{qualified_function_name, FunctionReflection, LocalReflection, MirReflection};

pub fn evaluate_function(
    program: &MirProgram,
    name: &str,
    args: &[ComptimeValue],
) -> ComptimeResult<ComptimeValue> {
    Interpreter::new(program).run_function(name, args)
}

pub fn evaluate_main(program: &MirProgram) -> ComptimeResult<ComptimeValue> {
    Interpreter::new(program).run_main()
}

pub fn reflect_program(program: &MirProgram) -> MirReflection<'_> {
    MirReflection::new(program)
}

pub fn value_as_constant(
    value: &ComptimeValue,
    ty: Type,
) -> ComptimeResult<inscribe_mir::Constant> {
    value_to_constant(value, ty)
}

#[cfg(test)]
mod tests {
    use inscribe_hir::lower_module;
    use inscribe_lexer::lex;
    use inscribe_mir::lower_program;
    use inscribe_parser::parse_module;
    use inscribe_resolve::resolve_module;
    use inscribe_typeck::check_module;

    use crate::{
        evaluate_main, reflect_program, ComptimeAllocator, ComptimeValue, Interpreter, StructValue,
    };

    fn lower_source(source: &str) -> inscribe_mir::MirProgram {
        let tokens = lex(source).expect("lexing should succeed");
        let module = parse_module(tokens).expect("parsing should succeed");
        let resolved = resolve_module(&module).expect("resolution should succeed");
        let typed = check_module(&module, &resolved).expect("type checking should succeed");
        let hir = lower_module(&module, &resolved, &typed);
        lower_program(&hir)
    }

    #[test]
    fn executes_calls_branches_and_loops() {
        let mir = lower_source(
            r#"
fn bump(value: int) -> int {
    value + 1
}

fn main() -> int {
    let sum = 0

    for i in 0..4 {
        sum = sum + bump(i)
    }

    if sum > 0 {
        sum
    } else {
        0
    }
}
"#,
        );

        assert_eq!(evaluate_main(&mir), Ok(ComptimeValue::Integer(10)));
    }

    #[test]
    fn executes_result_try_flow() {
        let mir = lower_source(
            r#"
fn wrap(value: int) -> Result<int, Error> {
    Ok(value)
}

fn main() -> Result<int, Error> {
    let value = wrap(7)?
    Ok(value + 1)
}
"#,
        );

        assert_eq!(
            Interpreter::new(&mir).run_main(),
            Ok(ComptimeValue::ResultOk(Box::new(ComptimeValue::Integer(8))))
        );
    }

    #[test]
    fn reflects_function_metadata() {
        let mir = lower_source(
            r#"
struct User {
    id: int
}

fn User.greet(self) -> int {
    1
}

fn main() -> int {
    0
}
"#,
        );

        let reflection = reflect_program(&mir);
        let info = reflection
            .describe_function("User.greet")
            .expect("method should exist");

        assert_eq!(info.qualified_name, "User.greet");
        assert_eq!(info.param_count, 1);
        assert_eq!(info.return_type, "int");
        assert!(reflection
            .callable_names()
            .iter()
            .any(|name| name == "User.greet"));
    }

    #[test]
    fn allocator_stores_values_by_handle() {
        let mut allocator = ComptimeAllocator::new();
        let id = allocator.alloc(ComptimeValue::Struct(StructValue::new(
            vec!["Point".to_string()],
            vec![("x".to_string(), ComptimeValue::Integer(4))],
        )));

        assert_eq!(allocator.len(), 1);
        assert_eq!(
            allocator.get(id),
            Some(&ComptimeValue::Struct(StructValue::new(
                vec!["Point".to_string()],
                vec![("x".to_string(), ComptimeValue::Integer(4))]
            )))
        );
    }
}
