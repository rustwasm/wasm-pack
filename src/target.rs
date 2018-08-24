//! Information about the target.

#![allow(missing_docs)]

pub const LINUX: bool = cfg!(target_os = "linux");
pub const MACOS: bool = cfg!(target_os = "macos");
pub const WINDOWS: bool = cfg!(target_os = "windows");

#[allow(non_upper_case_globals)]
pub const x86_64: bool = cfg!(target_arch = "x86_64");
#[allow(non_upper_case_globals)]
pub const x86: bool = cfg!(target_arch = "x86");
