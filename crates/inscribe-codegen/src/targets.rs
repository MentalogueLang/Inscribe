#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    X86_64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatingSystem {
    Linux,
    Windows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutableFormat {
    Elf64,
    Pe64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target {
    pub arch: Architecture,
    pub os: OperatingSystem,
}

impl Target {
    pub const fn linux_x86_64() -> Self {
        Self {
            arch: Architecture::X86_64,
            os: OperatingSystem::Linux,
        }
    }

    pub const fn windows_x86_64() -> Self {
        Self {
            arch: Architecture::X86_64,
            os: OperatingSystem::Windows,
        }
    }

    pub const fn executable_format(self) -> ExecutableFormat {
        match self.os {
            OperatingSystem::Linux => ExecutableFormat::Elf64,
            OperatingSystem::Windows => ExecutableFormat::Pe64,
        }
    }

    pub const fn entry_symbol(self) -> &'static str {
        match self.os {
            OperatingSystem::Linux => "_start",
            OperatingSystem::Windows => "mainCRTStartup",
        }
    }

    pub const fn executable_extension(self) -> &'static str {
        match self.os {
            OperatingSystem::Linux => "elf",
            OperatingSystem::Windows => "exe",
        }
    }
}
