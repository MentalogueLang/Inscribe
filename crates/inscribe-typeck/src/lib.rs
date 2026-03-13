use inscribe_ast as _;
use inscribe_resolve as _;
use inscribe_session as _;

pub mod check;
pub mod errors;
pub mod infer;
pub mod ownership;
pub mod unify;

// TODO: Implement the library root module for inscribe-typeck.
