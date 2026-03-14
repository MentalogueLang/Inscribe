use std::collections::HashMap;

use inscribe_abi::AbiType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FfiTypeMap {
    entries: HashMap<String, AbiType>,
}

impl Default for FfiTypeMap {
    fn default() -> Self {
        Self::mantelogue_core()
    }
}

impl FfiTypeMap {
    pub fn mantelogue_core() -> Self {
        let mut entries = HashMap::new();
        entries.insert("int".to_string(), AbiType::Int);
        entries.insert("float".to_string(), AbiType::Float);
        entries.insert("bool".to_string(), AbiType::Bool);
        entries.insert("Error".to_string(), AbiType::Error);
        entries.insert("ptr".to_string(), AbiType::Pointer);
        entries.insert("unit".to_string(), AbiType::Unit);
        Self { entries }
    }

    pub fn insert(&mut self, name: impl Into<String>, ty: AbiType) -> Option<AbiType> {
        self.entries.insert(name.into(), ty)
    }

    pub fn resolve(&self, name: &str) -> Option<&AbiType> {
        self.entries.get(name)
    }

    pub fn supports_abi_type(&self, ty: &AbiType) -> bool {
        match ty {
            AbiType::Unit
            | AbiType::Int
            | AbiType::Float
            | AbiType::Bool
            | AbiType::Error
            | AbiType::Pointer => true,
            AbiType::Struct(layout) => layout
                .fields
                .iter()
                .all(|field| self.supports_abi_type(&field.ty)),
            AbiType::Result(ok, err) => self.supports_abi_type(ok) && self.supports_abi_type(err),
        }
    }
}
