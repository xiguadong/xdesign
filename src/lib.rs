//! xdesign application library: `core` holds flow-agnostic infrastructure,
//! `simple_flow` and `wish_verify` build the designed flows on it, `storage`
//! owns local operational persistence, and `web` is the embedded operator web
//! UI. Flow and storage modules may use core; core never depends on a flow or
//! on SQLite storage, and the web operator UI depends on flows and storage
//! without being a flow.

pub mod core;
pub mod simple_flow;
pub mod storage;
pub mod wish_verify;

/// The operator web UI is authored under `web/` so its server module, page
/// assets, and developer notes stay together in the shipped source tree. It is
/// compiled as a library module through this path declaration and its page
/// assets are embedded by `web/server/build.rs`; no part of it is a separate
/// process or runtime dependency.
#[path = "../web/server/mod.rs"]
pub mod web;
