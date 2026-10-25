//! xdesign application library: `core` holds flow-agnostic infrastructure,
//! `simple_flow` and `wish_verify` build the designed flows on it, and `storage`
//! owns local operational persistence. Flow and storage modules may use core;
//! core never depends on a flow or on SQLite storage.

pub mod core;
pub mod simple_flow;
pub mod storage;
pub mod wish_verify;
