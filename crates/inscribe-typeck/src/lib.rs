use inscribe_ast as _;
use inscribe_resolve as _;
use inscribe_session as _;

pub mod infer;
pub mod check;
pub mod unify;
pub mod ownership;
pub mod errors;

// TODO: Implement the library root module for inscribe-typeck.
