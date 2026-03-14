use inscribe_abi::AbiType;

use crate::extern_block::ExternFunction;
use crate::FfiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnershipPolicy {
    Copy,
    Borrowed,
    OwnedByCaller,
    OwnedByCallee,
    Handle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnershipRule {
    pub ty: AbiType,
    pub policy: OwnershipPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OwnershipBridge {
    pub params: Vec<OwnershipRule>,
    pub return_rule: Option<OwnershipRule>,
}

impl OwnershipBridge {
    pub fn validate_for(&self, function: &ExternFunction) -> Result<(), FfiError> {
        if self.params.len() != function.params.len() {
            return Err(FfiError::new(format!(
                "ownership bridge for `{}` expects {} params, found {}",
                function.name,
                function.params.len(),
                self.params.len()
            )));
        }

        for (param, rule) in function.params.iter().zip(self.params.iter()) {
            if param.ty != rule.ty {
                return Err(FfiError::new(format!(
                    "ownership rule for parameter `{}` in `{}` does not match its ABI type",
                    param.name, function.name
                )));
            }

            validate_policy(&param.ty, rule.policy, &function.name, Some(&param.name))?;
        }

        if let Some(rule) = &self.return_rule {
            if function.return_type != rule.ty {
                return Err(FfiError::new(format!(
                    "ownership rule for return type of `{}` does not match its ABI type",
                    function.name
                )));
            }

            validate_policy(&function.return_type, rule.policy, &function.name, None)?;
        }

        Ok(())
    }
}

fn validate_policy(
    ty: &AbiType,
    policy: OwnershipPolicy,
    function: &str,
    param: Option<&str>,
) -> Result<(), FfiError> {
    match (ty, policy) {
        (AbiType::Pointer, OwnershipPolicy::Copy) => Err(FfiError::new(format!(
            "pointer values in `{function}` must use borrowed, owned, or handle ownership{}",
            param
                .map(|name| format!(" for `{name}`"))
                .unwrap_or_default()
        ))),
        (AbiType::Struct(_), OwnershipPolicy::Handle) => Err(FfiError::new(format!(
            "struct values in `{function}` cannot be passed as opaque handles{}",
            param
                .map(|name| format!(" for `{name}`"))
                .unwrap_or_default()
        ))),
        _ => Ok(()),
    }
}
