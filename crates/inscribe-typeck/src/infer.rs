use std::collections::HashMap;

use inscribe_ast::span::Span;
use inscribe_resolve::FunctionKey;
use serde::{Deserialize, Serialize};

// TODO: Replace the span-keyed tables with stable node ids once the AST is interned.

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Unknown,
    Unit,
    Int,
    Byte,
    Float,
    String,
    Bool,
    Error,
    Struct(String),
    Enum(String),
    Array(Box<Type>, usize),
    Result(Box<Type>, Box<Type>),
    Range(Box<Type>),
    Function(FunctionSignature),
}

impl Type {
    pub fn display_name(&self) -> String {
        match self {
            Self::Unknown => "_".to_string(),
            Self::Unit => "()".to_string(),
            Self::Int => "int".to_string(),
            Self::Byte => "byte".to_string(),
            Self::Float => "float".to_string(),
            Self::String => "string".to_string(),
            Self::Bool => "bool".to_string(),
            Self::Error => "Error".to_string(),
            Self::Struct(name) => name.clone(),
            Self::Enum(name) => name.clone(),
            Self::Array(element, length) => format!("[{}; {}]", element.display_name(), length),
            Self::Result(ok, err) => {
                format!("Result<{}, {}>", ok.display_name(), err.display_name())
            }
            Self::Range(inner) => format!("Range<{}>", inner.display_name()),
            Self::Function(signature) => signature.display_name(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionSignature {
    pub key: FunctionKey,
    pub params: Vec<Type>,
    pub return_type: Box<Type>,
}

impl FunctionSignature {
    pub fn display_name(&self) -> String {
        let params = self
            .params
            .iter()
            .map(Type::display_name)
            .collect::<Vec<_>>()
            .join(", ");
        format!("fn({params}) -> {}", self.return_type.display_name())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum BindingKind {
    Let,
    Const,
    Param,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct BindingInfo {
    pub ty: Type,
    pub kind: BindingKind,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeCheckResult {
    pub expr_types: HashMap<ExprKey, Type>,
    pub function_signatures: HashMap<FunctionKey, FunctionSignature>,
    pub item_types: HashMap<String, Type>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExprKey {
    pub start: usize,
    pub end: usize,
}

pub fn expr_key(span: Span) -> ExprKey {
    ExprKey {
        start: span.start.offset,
        end: span.end.offset,
    }
}
