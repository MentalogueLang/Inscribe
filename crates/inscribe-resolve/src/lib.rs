use inscribe_ast as _;
use inscribe_session as _;

pub mod cycle_detect;
pub mod import;
pub mod module_tree;
pub mod resolver;
pub mod scope;

pub use resolver::{
    resolve_module, Builtins, FunctionInfo, FunctionKey, ParamInfo, ResolveError, ResolvedProgram,
    StructInfo, Symbol, SymbolId, SymbolKind, TypeName,
};

// TODO: Add a loader-facing API that can resolve multiple source files into a single module graph.
