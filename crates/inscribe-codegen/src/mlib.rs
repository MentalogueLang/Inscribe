use inscribe_abi::{AbiTarget, MlibExport, MlibExportKind, MlibFile};
use inscribe_mir::{MirFunction, MirProgram};

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
            signature: None,
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
