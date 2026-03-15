use inscribe_ast::span::Span;

use crate::errors::TypeError;
use crate::infer::{FunctionSignature, Type};

// TODO: Replace this with proper type variables once generic inference exists.

pub fn unify(expected: &Type, actual: &Type, span: Span) -> Result<Type, TypeError> {
    match (expected, actual) {
        (Type::Unknown, other) | (other, Type::Unknown) => Ok(other.clone()),
        (Type::Unit, Type::Unit)
        | (Type::Int, Type::Int)
        | (Type::Byte, Type::Byte)
        | (Type::Float, Type::Float)
        | (Type::String, Type::String)
        | (Type::Bool, Type::Bool)
        | (Type::Error, Type::Error) => Ok(expected.clone()),
        (Type::Byte, Type::Int) | (Type::Int, Type::Byte) => Ok(Type::Byte),
        (Type::Struct(left), Type::Struct(right)) if left == right => Ok(expected.clone()),
        (Type::Enum(left), Type::Enum(right)) if left == right => Ok(expected.clone()),
        (Type::Array(left, left_len), Type::Array(right, right_len)) if left_len == right_len => {
            Ok(Type::Array(Box::new(unify(left, right, span)?), *left_len))
        }
        (Type::Range(left), Type::Range(right)) => {
            Ok(Type::Range(Box::new(unify(left, right, span)?)))
        }
        (Type::Result(left_ok, left_err), Type::Result(right_ok, right_err)) => Ok(Type::Result(
            Box::new(unify(left_ok, right_ok, span)?),
            Box::new(unify(left_err, right_err, span)?),
        )),
        (Type::Function(left), Type::Function(right)) => {
            unify_signatures(left, right, span).map(Type::Function)
        }
        _ => Err(TypeError::new(
            format!(
                "type mismatch: expected `{}`, found `{}`",
                expected.display_name(),
                actual.display_name()
            ),
            span,
        )),
    }
}

fn unify_signatures(
    expected: &FunctionSignature,
    actual: &FunctionSignature,
    span: Span,
) -> Result<FunctionSignature, TypeError> {
    if expected.params.len() != actual.params.len() {
        return Err(TypeError::new(
            format!(
                "function arity mismatch: expected {} arguments, found {}",
                expected.params.len(),
                actual.params.len()
            ),
            span,
        ));
    }

    let params = expected
        .params
        .iter()
        .zip(&actual.params)
        .map(|(left, right)| unify(left, right, span))
        .collect::<Result<Vec<_>, _>>()?;
    let return_type = Box::new(unify(&expected.return_type, &actual.return_type, span)?);

    Ok(FunctionSignature {
        key: expected.key.clone(),
        params,
        return_type,
    })
}
