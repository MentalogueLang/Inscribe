use inscribe_hir as _;
use inscribe_typeck as _;

pub mod borrow_check;
pub mod const_eval;
pub mod determinism;
pub mod lower;
pub mod nodes;

// TODO: Implement the library root module for inscribe-mir.
