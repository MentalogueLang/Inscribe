use std::fmt;

use inscribe_ast::span::Span;

// TODO: Split these into richer diagnostics with labels and notes once the session crate owns reporting.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeError {
    pub message: String,
    pub span: Span,
}

impl TypeError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at line {}, column {}",
            self.message, self.span.start.line, self.span.start.column
        )
    }
}

impl std::error::Error for TypeError {}
