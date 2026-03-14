pub mod abi_header;
pub mod calling_conv;
pub mod layout;
pub mod stability;
pub mod versioning;

pub use abi_header::{AbiCompatibilityError, AbiHeader, ABI_HEADER_SIZE, ABI_MAGIC};
pub use calling_conv::{AbiTarget, CallingConvention};
pub use layout::{AbiType, FieldLayout, Layout, StructField, StructLayout, StructMemoryLayout};
pub use stability::Stability;
pub use versioning::{AbiVersion, CURRENT_ABI_VERSION};

pub fn current_header(target: AbiTarget, stability: Stability) -> AbiHeader {
    AbiHeader::current(target, stability)
}

#[cfg(test)]
mod tests {
    use super::{
        current_header, AbiHeader, AbiTarget, AbiType, CallingConvention, Layout, Stability,
        StructField, StructLayout, CURRENT_ABI_VERSION,
    };

    #[test]
    fn abi_header_round_trips() {
        let header = current_header(AbiTarget::WindowsX86_64, Stability::Stable);
        let bytes = header.to_bytes();
        let decoded = AbiHeader::from_bytes(bytes).expect("header should decode");

        assert_eq!(decoded, header);
        assert!(decoded.is_compatible_with_current());
    }

    #[test]
    fn abi_target_uses_expected_calling_convention() {
        assert_eq!(
            current_header(AbiTarget::LinuxX86_64, Stability::Experimental).calling_convention,
            CallingConvention::SystemV64
        );
        assert_eq!(
            current_header(AbiTarget::WindowsX86_64, Stability::Stable).calling_convention,
            CallingConvention::Win64
        );
    }

    #[test]
    fn struct_layout_tracks_offsets() {
        let layout = StructLayout::new(
            "Pair",
            vec![
                StructField {
                    name: "flag".to_string(),
                    ty: AbiType::Bool,
                },
                StructField {
                    name: "value".to_string(),
                    ty: AbiType::Int,
                },
            ],
        )
        .memory_layout();

        assert_eq!(layout.layout, Layout::new(16, 8));
        assert_eq!(layout.field("flag").expect("flag field").offset, 0);
        assert_eq!(layout.field("value").expect("value field").offset, 8);
    }

    #[test]
    fn result_layout_reserves_tag_and_payload() {
        let layout = AbiType::Result(Box::new(AbiType::Bool), Box::new(AbiType::Int)).layout();

        assert_eq!(layout, Layout::new(16, 8));
    }

    #[test]
    fn current_version_is_self_compatible() {
        assert!(CURRENT_ABI_VERSION.is_compatible_with(CURRENT_ABI_VERSION));
    }

    #[test]
    fn stable_header_is_link_compatible() {
        let header = current_header(AbiTarget::LinuxX86_64, Stability::Stable);
        header
            .ensure_link_compatible(AbiTarget::LinuxX86_64, true)
            .expect("stable header should be link-compatible");
    }

    #[test]
    fn unstable_header_is_rejected_for_stable_linking() {
        let header = current_header(AbiTarget::LinuxX86_64, Stability::Experimental);
        let error = header
            .ensure_link_compatible(AbiTarget::LinuxX86_64, true)
            .expect_err("experimental headers should be rejected");

        assert!(error.message.contains("not allowed for external linking"));
    }
}
