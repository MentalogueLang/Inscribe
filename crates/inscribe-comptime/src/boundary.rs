use std::fmt;

use inscribe_mir::{Constant, ConstantValue};
use inscribe_typeck::Type;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComptimeError {
    pub message: String,
}

impl ComptimeError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ComptimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ComptimeError {}

pub type ComptimeResult<T> = Result<T, ComptimeError>;

#[derive(Debug, Clone, PartialEq)]
pub struct StructValue {
    pub path: Vec<String>,
    pub fields: Vec<(String, ComptimeValue)>,
}

impl StructValue {
    pub fn new(path: Vec<String>, fields: Vec<(String, ComptimeValue)>) -> Self {
        Self { path, fields }
    }

    pub fn type_name(&self) -> String {
        self.path.join(".")
    }

    pub fn field(&self, name: &str) -> Option<&ComptimeValue> {
        self.fields
            .iter()
            .find_map(|(field, value)| (field == name).then_some(value))
    }

    pub fn field_mut(&mut self, name: &str) -> Option<&mut ComptimeValue> {
        self.fields
            .iter_mut()
            .find_map(|(field, value)| (field == name).then_some(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangeValue {
    pub next: i64,
    pub end: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComptimeValue {
    Unit,
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Function(String),
    Struct(StructValue),
    ResultOk(Box<ComptimeValue>),
    ResultErr(Box<ComptimeValue>),
    Range(RangeValue),
}

impl ComptimeValue {
    pub fn display(&self) -> String {
        match self {
            Self::Unit => "()".to_string(),
            Self::Integer(value) => value.to_string(),
            Self::Float(value) => value.to_string(),
            Self::String(value) => format!("{value:?}"),
            Self::Bool(value) => value.to_string(),
            Self::Function(name) => name.clone(),
            Self::Struct(value) => value.type_name(),
            Self::ResultOk(value) => format!("Ok({})", value.display()),
            Self::ResultErr(value) => format!("Err({})", value.display()),
            Self::Range(value) => format!("{}..{}", value.next, value.end),
        }
    }

    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Unit => "unit",
            Self::Integer(_) => "int",
            Self::Float(_) => "float",
            Self::String(_) => "string",
            Self::Bool(_) => "bool",
            Self::Function(_) => "function",
            Self::Struct(_) => "struct",
            Self::ResultOk(_) | Self::ResultErr(_) => "result",
            Self::Range(_) => "range",
        }
    }

    pub fn expect_bool(&self) -> ComptimeResult<bool> {
        match self {
            Self::Bool(value) => Ok(*value),
            other => Err(ComptimeError::new(format!(
                "expected bool, found {}",
                other.kind_name()
            ))),
        }
    }
}

pub fn constant_to_value(constant: &Constant) -> ComptimeValue {
    match (&constant.ty, &constant.value) {
        (_, ConstantValue::Unit) => ComptimeValue::Unit,
        (_, ConstantValue::Integer(value)) => {
            ComptimeValue::Integer(value.parse().unwrap_or_default())
        }
        (_, ConstantValue::Float(value)) => ComptimeValue::Float(value.parse().unwrap_or_default()),
        (Type::Int, ConstantValue::String(value)) => {
            ComptimeValue::Integer(value.parse().unwrap_or_default())
        }
        (Type::Float, ConstantValue::String(value)) => {
            ComptimeValue::Float(value.parse().unwrap_or_default())
        }
        (Type::Bool, ConstantValue::String(value)) => ComptimeValue::Bool(value == "true"),
        (_, ConstantValue::String(value)) => ComptimeValue::String(value.clone()),
        (_, ConstantValue::Bool(value)) => ComptimeValue::Bool(*value),
        (_, ConstantValue::Function(value)) => ComptimeValue::Function(value.clone()),
    }
}

pub fn value_to_constant(value: &ComptimeValue, ty: Type) -> ComptimeResult<Constant> {
    let value = match value {
        ComptimeValue::Unit => ConstantValue::Unit,
        ComptimeValue::Integer(value) => ConstantValue::Integer(value.to_string()),
        ComptimeValue::Float(value) => ConstantValue::Float(value.to_string()),
        ComptimeValue::String(value) => ConstantValue::String(value.clone()),
        ComptimeValue::Bool(value) => ConstantValue::Bool(*value),
        ComptimeValue::Function(value) => ConstantValue::Function(value.clone()),
        ComptimeValue::Struct(_)
        | ComptimeValue::ResultOk(_)
        | ComptimeValue::ResultErr(_)
        | ComptimeValue::Range(_) => {
            return Err(ComptimeError::new(format!(
                "cannot lower {} into a MIR constant",
                value.kind_name()
            )));
        }
    };

    Ok(Constant { ty, value })
}
