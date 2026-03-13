use inscribe_ast as _;
use inscribe_resolve as _;
use inscribe_session as _;

pub mod check;
pub mod errors;
pub mod infer;
pub mod ownership;
pub mod unify;

pub use check::check_module;
pub use errors::TypeError;
pub use infer::{expr_key, BindingInfo, BindingKind, FunctionSignature, Type, TypeCheckResult};

// TODO: Add a session-facing facade that can run resolve + typeck as a single compiler phase.
