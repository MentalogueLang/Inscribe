use inscribe_ast::span::Span;

use crate::errors::TypeError;
use crate::infer::{BindingInfo, BindingKind};

// TODO: Grow this into a real ownership/borrowing analysis once references exist in the language.

pub fn ensure_assignable(binding: &BindingInfo, name: &str, span: Span) -> Result<(), TypeError> {
    match binding.kind {
        BindingKind::Let => Ok(()),
        BindingKind::Const | BindingKind::Param => Err(TypeError::new(
            format!("cannot assign to immutable binding `{name}`"),
            span,
        )),
    }
}
