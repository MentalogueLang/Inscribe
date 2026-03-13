use inscribe_ast as _;
use inscribe_session as _;

pub mod resolver;
pub mod module_tree;
pub mod import;
pub mod scope;
pub mod cycle_detect;

// TODO: Implement the library root module for inscribe-resolve.
