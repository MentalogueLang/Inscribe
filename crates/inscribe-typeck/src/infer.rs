use std::collections::HashMap;

use inscribe_ast::span::Span;
use inscribe_resolve::FunctionKey;

// TODO: Replace the span-keyed tables with stable node ids once the AST is interned.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Unknown,
    Unit,
    Int,
    Float,
    String,
    Bool,
    Error,
    Struct(String),
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
            Self::Float => "float".to_string(),
            Self::String => "string".to_string(),
            Self::Bool => "bool".to_string(),
            Self::Error => "Error".to_string(),
            Self::Struct(name) => name.clone(),
            Self::Result(ok, err) => {
                format!("Result<{}, {}>", ok.display_name(), err.display_name())
            }
            Self::Range(inner) => format!("Range<{}>", inner.display_name()),
            Self::Function(signature) => signature.display_name(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingKind {
    Let,
    Const,
    Param,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingInfo {
    pub ty: Type,
    pub kind: BindingKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeCheckResult {
    pub expr_types: HashMap<usize, Type>,
    pub function_signatures: HashMap<FunctionKey, FunctionSignature>,
    pub item_types: HashMap<String, Type>,
}

pub fn expr_key(span: Span) -> usize {
    span.start.offset
}
