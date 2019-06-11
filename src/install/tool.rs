use std::fmt;

/// Represents the set of CLI tools wasm-pack uses
pub enum Tool {
    /// cargo-generate CLI tool
    CargoGenerate,
    /// wasm-bindgen CLI tools
    WasmBindgen,
}

impl fmt::Display for Tool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Tool::CargoGenerate => write!(f, "cargo-generate"),
            Tool::WasmBindgen => write!(f, "wasm-bindgen"),
        }
    }
}
