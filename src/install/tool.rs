/// Represents the set of CLI tools wasm-pack uses
pub enum Tool {
    /// cargo-generate CLI tool
    CargoGenerate,
    /// wasm-bindgen CLI tools
    WasmBindgen,
    /// wasm-opt CLI tool
    WasmOpt,
}

impl Tool {
    /// Returns the binary's name
    pub fn name(&self) -> &'static str {
        match self {
            Tool::CargoGenerate => "cargo-generate",
            Tool::WasmBindgen => "wasm-bindgen",
            Tool::WasmOpt => "wasm-opt",
        }
    }
}
