//! Information about the target wasm-pack is currently being compiled for.
//!
//! That is, whether we are building wasm-pack for windows vs linux, and x86 vs
//! x86-64, etc.

#![allow(missing_docs)]

pub const WINDOWS: bool = cfg!(target_os = "windows");
