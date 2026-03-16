use std::sync::Arc;

use inscribe_comptime::{ComptimeValue, Interpreter};
use inscribe_mir::MirProgram;

use crate::policy::SandboxPolicy;
use crate::runtime::SandboxRuntime;
use crate::SandboxError;

#[derive(Debug, Clone)]
pub struct WasmBackend {
    policy: SandboxPolicy,
}

impl WasmBackend {
    pub fn new(policy: SandboxPolicy) -> Self {
        Self { policy }
    }

    pub fn run_main(&self, program: &MirProgram) -> Result<ComptimeValue, SandboxError> {
        let runtime = Arc::new(SandboxRuntime::new(self.policy.clone()));
        let interpreter = Interpreter::with_runtime(program, runtime);
        interpreter.run_main().map_err(SandboxError::from)
    }

    pub fn run_function(
        &self,
        program: &MirProgram,
        name: &str,
        args: &[ComptimeValue],
    ) -> Result<ComptimeValue, SandboxError> {
        let runtime = Arc::new(SandboxRuntime::new(self.policy.clone()));
        let interpreter = Interpreter::with_runtime(program, runtime);
        interpreter
            .run_function(name, args)
            .map_err(SandboxError::from)
    }
}
