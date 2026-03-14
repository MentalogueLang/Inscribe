#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AbiTarget {
    LinuxX86_64,
    WindowsX86_64,
}

impl AbiTarget {
    pub const fn pointer_width(self) -> u8 {
        let _ = self;
        64
    }

    pub const fn pointer_size(self) -> u32 {
        let _ = self;
        8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CallingConvention {
    Mantelogue,
    C,
    SystemV64,
    Win64,
}

impl CallingConvention {
    pub const fn for_target(target: AbiTarget) -> Self {
        match target {
            AbiTarget::LinuxX86_64 => Self::SystemV64,
            AbiTarget::WindowsX86_64 => Self::Win64,
        }
    }

    pub const fn preserves_rbx(self) -> bool {
        let _ = self;
        true
    }

    pub const fn stack_shadow_space(self) -> u32 {
        match self {
            Self::Win64 => 32,
            Self::Mantelogue | Self::C | Self::SystemV64 => 0,
        }
    }
}
