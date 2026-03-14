pub mod extern_block;
pub mod ownership_bridge;
pub mod sandbox_ffi;
pub mod type_map;

pub use extern_block::{ExternBlock, ExternFunction, ExternParam};
pub use ownership_bridge::{OwnershipBridge, OwnershipPolicy, OwnershipRule};
pub use sandbox_ffi::{SandboxPolicy, SandboxedExternBlock};
pub use type_map::FfiTypeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FfiError {
    pub message: String,
}

impl FfiError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for FfiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for FfiError {}

#[cfg(test)]
mod tests {
    use super::{
        ExternBlock, ExternFunction, ExternParam, FfiTypeMap, OwnershipBridge, OwnershipPolicy,
        OwnershipRule, SandboxPolicy, SandboxedExternBlock,
    };
    use inscribe_abi::{
        AbiTarget, AbiType, CallingConvention, Stability, StructField, StructLayout,
    };

    #[test]
    fn validates_stable_extern_block() {
        let block = ExternBlock::stable_c(
            "math",
            AbiTarget::WindowsX86_64,
            vec![ExternFunction {
                name: "add".to_string(),
                params: vec![
                    ExternParam {
                        name: "left".to_string(),
                        ty: AbiType::Int,
                    },
                    ExternParam {
                        name: "right".to_string(),
                        ty: AbiType::Int,
                    },
                ],
                return_type: AbiType::Int,
                calling_convention: CallingConvention::C,
                stability: Stability::Stable,
            }],
        );

        block
            .validate_against(&FfiTypeMap::default())
            .expect("stable C ABI should validate");
    }

    #[test]
    fn ownership_bridge_rejects_copying_pointers() {
        let function = ExternFunction {
            name: "consume_ptr".to_string(),
            params: vec![ExternParam {
                name: "ptr".to_string(),
                ty: AbiType::Pointer,
            }],
            return_type: AbiType::Unit,
            calling_convention: CallingConvention::C,
            stability: Stability::Stable,
        };
        let bridge = OwnershipBridge {
            params: vec![OwnershipRule {
                ty: AbiType::Pointer,
                policy: OwnershipPolicy::Copy,
            }],
            return_rule: None,
        };

        let error = bridge
            .validate_for(&function)
            .expect_err("copying pointers across FFI should be rejected");
        assert!(error.message.contains("pointer values"));
    }

    #[test]
    fn sandbox_policy_rejects_nondeterministic_capabilities() {
        let block = ExternBlock::stable_c("sandbox", AbiTarget::LinuxX86_64, Vec::new());
        let sandboxed = SandboxedExternBlock {
            block,
            policy: SandboxPolicy {
                allow_network: true,
                ..SandboxPolicy::default()
            },
        };

        let error = sandboxed
            .validate()
            .expect_err("network access should violate deterministic sandboxing");
        assert!(error.message.contains("deterministic"));
    }

    #[test]
    fn type_map_accepts_nested_struct_layouts() {
        let mut types = FfiTypeMap::default();
        types.insert(
            "Pair",
            AbiType::Struct(StructLayout::new(
                "Pair",
                vec![
                    StructField {
                        name: "left".to_string(),
                        ty: AbiType::Int,
                    },
                    StructField {
                        name: "right".to_string(),
                        ty: AbiType::Int,
                    },
                ],
            )),
        );

        assert!(types.supports_abi_type(
            types
                .resolve("Pair")
                .expect("custom type should be present")
        ));
    }
}
