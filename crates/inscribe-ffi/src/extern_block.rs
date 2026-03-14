use inscribe_abi::{AbiHeader, AbiTarget, AbiType, CallingConvention, Stability, StructLayout};

use crate::type_map::FfiTypeMap;
use crate::FfiError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternParam {
    pub name: String,
    pub ty: AbiType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternFunction {
    pub name: String,
    pub params: Vec<ExternParam>,
    pub return_type: AbiType,
    pub calling_convention: CallingConvention,
    pub stability: Stability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternBlock {
    pub name: String,
    pub target: AbiTarget,
    pub header: AbiHeader,
    pub functions: Vec<ExternFunction>,
    pub exported_types: Vec<StructLayout>,
}

impl ExternBlock {
    pub fn stable_c(
        name: impl Into<String>,
        target: AbiTarget,
        functions: Vec<ExternFunction>,
    ) -> Self {
        let mut header = AbiHeader::current(target, Stability::Stable);
        header.calling_convention = CallingConvention::C;
        Self {
            name: name.into(),
            target,
            header,
            functions,
            exported_types: Vec::new(),
        }
    }

    pub fn validate(&self) -> Result<(), FfiError> {
        self.header
            .ensure_link_compatible(self.target, true)
            .map_err(|error| FfiError::new(error.message))?;

        for function in &self.functions {
            let target_cc = CallingConvention::for_target(self.target);
            if function.calling_convention != CallingConvention::C
                && function.calling_convention != target_cc
            {
                return Err(FfiError::new(format!(
                    "function `{}` uses incompatible calling convention {:?}",
                    function.name, function.calling_convention
                )));
            }

            if !function.stability.allows_external_linking() {
                return Err(FfiError::new(format!(
                    "function `{}` is not stable enough for FFI export",
                    function.name
                )));
            }
        }

        Ok(())
    }

    pub fn validate_against(&self, type_map: &FfiTypeMap) -> Result<(), FfiError> {
        self.validate()?;

        for layout in &self.exported_types {
            for field in &layout.fields {
                if !type_map.supports_abi_type(&field.ty) {
                    return Err(FfiError::new(format!(
                        "exported type `{}` contains unsupported field `{}`",
                        layout.name, field.name
                    )));
                }
            }
        }

        for function in &self.functions {
            if !type_map.supports_abi_type(&function.return_type) {
                return Err(FfiError::new(format!(
                    "function `{}` returns an unsupported ABI type",
                    function.name
                )));
            }

            for param in &function.params {
                if !type_map.supports_abi_type(&param.ty) {
                    return Err(FfiError::new(format!(
                        "parameter `{}` in function `{}` uses an unsupported ABI type",
                        param.name, function.name
                    )));
                }
            }
        }

        Ok(())
    }
}
