//! xdesign application library in two layers: `core` holds the flow-agnostic
//! infrastructure and the flow modules build the designed flows on it:
//! `simple_flow` the designed four-stage flow, `wish_verify` the designed
//! two-stage flow. Dependencies point one way: a flow module may use `core`,
//! never the reverse.

pub mod core;
pub mod simple_flow;
pub mod wish_verify;
