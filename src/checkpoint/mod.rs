//! Project-level checkpoint library: the strict data model, validation, atomic
//! storage, built-in seeds, scaffold generation, and the `xdesign checkpoint`
//! command envelope. The library is Git-tracked authoring source; frozen runs
//! keep their own copy of the rules/criteria they bound.

pub mod cli;
pub mod library;

pub use library::*;
