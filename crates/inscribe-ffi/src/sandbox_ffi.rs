use crate::extern_block::ExternBlock;
use crate::FfiError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxPolicy {
    pub allow_filesystem: bool,
    pub allow_network: bool,
    pub allow_process: bool,
    pub deterministic_only: bool,
}

impl Default for SandboxPolicy {
    fn default() -> Self {
        Self {
            allow_filesystem: false,
            allow_network: false,
            allow_process: false,
            deterministic_only: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxedExternBlock {
    pub block: ExternBlock,
    pub policy: SandboxPolicy,
}

impl SandboxedExternBlock {
    pub fn validate(&self) -> Result<(), FfiError> {
        self.block.validate()?;

        if self.policy.deterministic_only
            && (self.policy.allow_network || self.policy.allow_process)
        {
            return Err(FfiError::new(
                "deterministic sandboxes cannot enable network or process access",
            ));
        }

        Ok(())
    }
}
