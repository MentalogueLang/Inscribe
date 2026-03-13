use inscribe_hir as _;
use inscribe_typeck as _;

pub mod lower;
pub mod nodes;
pub mod borrow_check;
pub mod const_eval;
pub mod determinism;

// TODO: Implement the library root module for inscribe-mir.
