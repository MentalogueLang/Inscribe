#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub stage: &'static str,
    pub message: String,
}

impl Diagnostic {
    pub fn new(stage: &'static str, message: impl Into<String>) -> Self {
        Self {
            stage,
            message: message.into(),
        }
    }
}
