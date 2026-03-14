use crate::calling_conv::{AbiTarget, CallingConvention};
use crate::stability::Stability;
use crate::versioning::{AbiVersion, CURRENT_ABI_VERSION};

pub const ABI_MAGIC: [u8; 8] = *b"INSCRIBE";
pub const ABI_HEADER_SIZE: usize = 24;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbiHeader {
    pub magic: [u8; 8],
    pub version: AbiVersion,
    pub target: AbiTarget,
    pub calling_convention: CallingConvention,
    pub stability: Stability,
    pub flags: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbiCompatibilityError {
    pub message: String,
}

impl AbiCompatibilityError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for AbiCompatibilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AbiCompatibilityError {}

impl AbiHeader {
    pub fn current(target: AbiTarget, stability: Stability) -> Self {
        Self {
            magic: ABI_MAGIC,
            version: CURRENT_ABI_VERSION,
            target,
            calling_convention: CallingConvention::for_target(target),
            stability,
            flags: 0,
        }
    }

    pub fn is_compatible_with_current(&self) -> bool {
        self.magic == ABI_MAGIC && CURRENT_ABI_VERSION.is_compatible_with(self.version)
    }

    pub fn ensure_link_compatible(
        &self,
        target: AbiTarget,
        require_stable: bool,
    ) -> Result<(), AbiCompatibilityError> {
        if self.magic != ABI_MAGIC {
            return Err(AbiCompatibilityError::new(
                "ABI header magic does not match Inscribe",
            ));
        }

        if !CURRENT_ABI_VERSION.is_compatible_with(self.version) {
            return Err(AbiCompatibilityError::new(format!(
                "ABI version {} is not compatible with current {}",
                self.version, CURRENT_ABI_VERSION
            )));
        }

        if self.target != target {
            return Err(AbiCompatibilityError::new(format!(
                "ABI target mismatch: header is {:?}, requested {:?}",
                self.target, target
            )));
        }

        let target_cc = CallingConvention::for_target(target);
        if self.calling_convention != target_cc && self.calling_convention != CallingConvention::C {
            return Err(AbiCompatibilityError::new(format!(
                "calling convention {:?} is not link-compatible with target {:?}",
                self.calling_convention, target
            )));
        }

        if require_stable && !self.stability.allows_external_linking() {
            return Err(AbiCompatibilityError::new(format!(
                "ABI stability `{}` is not allowed for external linking",
                self.stability.display_name()
            )));
        }

        Ok(())
    }

    pub fn to_bytes(&self) -> [u8; ABI_HEADER_SIZE] {
        let mut bytes = [0u8; ABI_HEADER_SIZE];
        bytes[..8].copy_from_slice(&self.magic);
        bytes[8..10].copy_from_slice(&self.version.major.to_le_bytes());
        bytes[10..12].copy_from_slice(&self.version.minor.to_le_bytes());
        bytes[12..14].copy_from_slice(&self.version.patch.to_le_bytes());
        bytes[14] = encode_target(self.target);
        bytes[15] = encode_calling_convention(self.calling_convention);
        bytes[16] = encode_stability(&self.stability);
        bytes[20..24].copy_from_slice(&self.flags.to_le_bytes());
        bytes
    }

    pub fn from_bytes(bytes: [u8; ABI_HEADER_SIZE]) -> Option<Self> {
        let target = decode_target(bytes[14])?;
        let calling_convention = decode_calling_convention(bytes[15])?;
        let stability = decode_stability(bytes[16])?;
        Some(Self {
            magic: bytes[..8].try_into().ok()?,
            version: AbiVersion::new(
                u16::from_le_bytes([bytes[8], bytes[9]]),
                u16::from_le_bytes([bytes[10], bytes[11]]),
                u16::from_le_bytes([bytes[12], bytes[13]]),
            ),
            target,
            calling_convention,
            stability,
            flags: u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]),
        })
    }
}

fn encode_target(target: AbiTarget) -> u8 {
    match target {
        AbiTarget::LinuxX86_64 => 1,
        AbiTarget::WindowsX86_64 => 2,
    }
}

fn decode_target(byte: u8) -> Option<AbiTarget> {
    match byte {
        1 => Some(AbiTarget::LinuxX86_64),
        2 => Some(AbiTarget::WindowsX86_64),
        _ => None,
    }
}

fn encode_calling_convention(calling_convention: CallingConvention) -> u8 {
    match calling_convention {
        CallingConvention::Mantelogue => 1,
        CallingConvention::C => 2,
        CallingConvention::SystemV64 => 3,
        CallingConvention::Win64 => 4,
    }
}

fn decode_calling_convention(byte: u8) -> Option<CallingConvention> {
    match byte {
        1 => Some(CallingConvention::Mantelogue),
        2 => Some(CallingConvention::C),
        3 => Some(CallingConvention::SystemV64),
        4 => Some(CallingConvention::Win64),
        _ => None,
    }
}

fn encode_stability(stability: &Stability) -> u8 {
    match stability {
        Stability::Experimental => 1,
        Stability::Stable => 2,
        Stability::Deprecated { .. } => 3,
        Stability::Internal => 4,
    }
}

fn decode_stability(byte: u8) -> Option<Stability> {
    match byte {
        1 => Some(Stability::Experimental),
        2 => Some(Stability::Stable),
        3 => Some(Stability::Deprecated {
            since: CURRENT_ABI_VERSION,
            note: String::new(),
        }),
        4 => Some(Stability::Internal),
        _ => None,
    }
}
