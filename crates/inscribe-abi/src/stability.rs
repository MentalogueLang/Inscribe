use crate::versioning::AbiVersion;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stability {
    Experimental,
    Stable,
    Deprecated { since: AbiVersion, note: String },
    Internal,
}

impl Stability {
    pub const fn is_public(&self) -> bool {
        !matches!(self, Self::Internal)
    }

    pub const fn is_stable(&self) -> bool {
        matches!(self, Self::Stable | Self::Deprecated { .. })
    }

    pub const fn allows_external_linking(&self) -> bool {
        matches!(self, Self::Stable | Self::Deprecated { .. })
    }

    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Experimental => "experimental",
            Self::Stable => "stable",
            Self::Deprecated { .. } => "deprecated",
            Self::Internal => "internal",
        }
    }
}
