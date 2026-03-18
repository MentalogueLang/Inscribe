use inscribe_abi::{AbiTarget, MlibExport, MlibExportKind, MlibFile};
use inscribe_mir::{MirFunction, MirProgram};
use inscribe_typeck::{FunctionSignature, Type};

use crate::targets::{OperatingSystem, Target};
use crate::CodegenError;

pub fn emit_mlib(program: &MirProgram, target: Target) -> Result<Vec<u8>, CodegenError> {
    let exports = program
        .functions
        .iter()
        .filter(|function| !function.is_declaration)
        .map(|function| MlibExport {
            name: qualified_function_name(function),
            kind: MlibExportKind::Function,
            address: 0,
            signature: Some(encode_signature(&function.signature).into_bytes()),
        })
        .collect::<Vec<_>>();

    let abi_target = match target.os {
        OperatingSystem::Linux => AbiTarget::LinuxX86_64,
        OperatingSystem::Windows => AbiTarget::WindowsX86_64,
    };

    let file = MlibFile::new(abi_target, exports, Vec::new(), Vec::new());
    Ok(file.to_bytes())
}

fn qualified_function_name(function: &MirFunction) -> String {
    function
        .receiver
        .as_ref()
        .map(|receiver| format!("{receiver}.{}", function.name))
        .unwrap_or_else(|| function.name.clone())
}

fn encode_signature(signature: &FunctionSignature) -> String {
    let params = signature
        .params
        .iter()
        .map(type_to_text)
        .collect::<Vec<_>>()
        .join(", ");
    format!("fn({params}) -> {}", type_to_text(&signature.return_type))
}

fn type_to_text(ty: &Type) -> String {
    match ty {
        Type::Unknown => "_".to_string(),
        Type::Unit => "()".to_string(),
        Type::Int => "int".to_string(),
        Type::Byte => "byte".to_string(),
        Type::Float => "float".to_string(),
        Type::String => "string".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Error => "Error".to_string(),
        Type::Struct(name) | Type::Enum(name) => name.clone(),
        Type::Array(element, length) => format!("[{}; {}]", type_to_text(element), length),
        Type::Result(ok, err) => format!("Result<{}, {}>", type_to_text(ok), type_to_text(err)),
        Type::Range(inner) => format!("Range<{}>", type_to_text(inner)),
        Type::Function(signature) => encode_signature(signature),
    }
}
