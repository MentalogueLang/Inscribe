use inscribe_abi::{AbiTarget, MlibExport, MlibExportKind, MlibFile};
use inscribe_ast::nodes::Visibility;
use inscribe_hir::nodes::HirEnum;
use inscribe_hir::{HirField, HirItem, HirProgram, HirStruct, HirSymbolId};
use inscribe_typeck::{FunctionSignature, Type};

use crate::targets::{OperatingSystem, Target};
use crate::CodegenError;

pub fn emit_mlib(program: &HirProgram, target: Target) -> Result<Vec<u8>, CodegenError> {
    let mut exports = Vec::new();

    for item in &program.items {
        match item {
            HirItem::Function(function)
                if !function.is_declaration && function.visibility == Visibility::Public =>
            {
                exports.push(MlibExport {
                    name: qualified_function_name(program, function.receiver, &program.symbol_name(function.symbol)),
                    kind: MlibExportKind::Function,
                    address: 0,
                    signature: Some(encode_signature(&function.signature).into_bytes()),
                });
            }
            HirItem::Struct(struct_decl) => exports.push(MlibExport {
                name: program.symbol_name(struct_decl.symbol).to_string(),
                kind: MlibExportKind::Type,
                address: 0,
                signature: Some(encode_struct(program, struct_decl).into_bytes()),
            }),
            HirItem::Enum(enum_decl) => exports.push(MlibExport {
                name: program.symbol_name(enum_decl.symbol).to_string(),
                kind: MlibExportKind::Type,
                address: 0,
                signature: Some(encode_enum(program, enum_decl).into_bytes()),
            }),
            _ => {}
        }
    }

    let abi_target = match target.os {
        OperatingSystem::Linux => AbiTarget::LinuxX86_64,
        OperatingSystem::Windows => AbiTarget::WindowsX86_64,
    };

    let file = MlibFile::new(abi_target, exports, Vec::new(), Vec::new());
    Ok(file.to_bytes())
}

fn qualified_function_name(program: &HirProgram, receiver: Option<HirSymbolId>, name: &str) -> String {
    receiver
        .map(|receiver| format!("{}.{}", program.symbol_name(receiver), name))
        .unwrap_or_else(|| name.to_string())
}

fn encode_struct(program: &HirProgram, decl: &HirStruct) -> String {
    let fields = decl
        .fields
        .iter()
        .map(|field| encode_field(program, field))
        .collect::<Vec<_>>()
        .join(", ");
    format!("struct {{ {fields} }}")
}

fn encode_field(program: &HirProgram, field: &HirField) -> String {
    format!("{}: {}", program.symbol_name(field.symbol), type_to_text(&field.ty))
}

fn encode_enum(program: &HirProgram, decl: &HirEnum) -> String {
    let variants = decl
        .variants
        .iter()
        .map(|variant| {
            format!(
                "{} = {}",
                program.symbol_name(variant.symbol),
                variant.discriminant
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("enum {{ {variants} }}")
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
