pub mod capability;
pub mod policy;
pub mod runtime;
pub mod wasm_backend;

use inscribe_comptime::{ComptimeError, ComptimeValue};
use inscribe_mir::MirProgram;

pub use capability::Capability;
pub use policy::SandboxPolicy;
pub use wasm_backend::WasmBackend;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxError {
    pub message: String,
}

impl SandboxError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl From<ComptimeError> for SandboxError {
    fn from(error: ComptimeError) -> Self {
        Self::new(error.message)
    }
}

pub fn run_main(program: &MirProgram, policy: SandboxPolicy) -> Result<ComptimeValue, SandboxError> {
    WasmBackend::new(policy).run_main(program)
}
