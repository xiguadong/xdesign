//! xdesign application library in two layers: `core` holds the flow-agnostic
//! infrastructure and `simple_flow` builds the designed four-stage flow on it.
//! Dependencies point one way: `simple_flow` may use `core`, never the reverse.

pub mod core;
pub mod simple_flow;
