use crate::diagnostics::Diagnostic;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionError {
    pub stage: &'static str,
    pub message: String,
}

impl SessionError {
    pub fn new(stage: &'static str, message: impl Into<String>) -> Self {
        Self {
            stage,
            message: message.into(),
        }
    }

    pub fn diagnostic(&self) -> Diagnostic {
        Diagnostic::new(self.stage, &self.message)
    }
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} error: {}", self.stage, self.message)
    }
}

impl std::error::Error for SessionError {}

#[derive(Debug, Clone, Default)]
pub struct Session;
