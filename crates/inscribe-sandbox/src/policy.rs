use crate::capability::Capability;

#[derive(Debug, Clone)]
pub struct SandboxPolicy {
    pub allow_stdout: bool,
    pub allow_stdin: bool,
    pub allow_network: bool,
    pub deterministic_only: bool,
}

impl SandboxPolicy {
    pub fn allows(&self, capability: Capability) -> bool {
        match capability {
            Capability::Stdout => self.allow_stdout,
            Capability::Stdin => self.allow_stdin && !self.deterministic_only,
            Capability::Network => self.allow_network && !self.deterministic_only,
        }
    }
}

impl Default for SandboxPolicy {
    fn default() -> Self {
        Self {
            allow_stdout: false,
            allow_stdin: false,
            allow_network: false,
            deterministic_only: false,
        }
    }
}
