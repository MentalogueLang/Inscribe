use inscribe_ast as _;
use inscribe_session as _;

pub mod cycle_detect;
pub mod import;
pub mod loader;
pub mod module_tree;
pub mod resolver;
pub mod scope;

pub use loader::{
    load_module_graph, load_module_graph_with_options, LoadedModuleGraph, ModuleLoadOptions,
    SourceModule,
};
pub use resolver::{
    resolve_module, Builtins, EnumInfo, FunctionInfo, FunctionKey, ParamInfo, ResolveError,
    ResolvedProgram, StructInfo, Symbol, SymbolId, SymbolKind, TypeName,
};

pub fn resolve_module_graph(
    graph: &LoadedModuleGraph,
) -> Result<ResolvedProgram, Vec<ResolveError>> {
    let mut resolved = resolve_module(&graph.merged)?;
    resolved.module_tree = graph.tree.clone();
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{load_module_graph, resolve_module_graph};

    #[test]
    fn loads_local_and_stdlib_imports_into_a_module_tree() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("resolve crate should live in workspace");
        let entry = root.join("tests/compile_pass/import_stdlib.mtl");

        let graph = load_module_graph(&entry).expect("imports should load");
        let resolved = resolve_module_graph(&graph).expect("loaded graph should resolve");

        assert!(graph.modules.len() >= 3);
        assert_eq!(
            resolved.module_tree.entry,
            entry.canonicalize().expect("entry path")
        );
        assert!(resolved
            .module_tree
            .modules
            .iter()
            .any(|module| module.name == "math"));
    }
}
