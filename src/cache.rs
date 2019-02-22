//! Getting and configuring wasm-pack's binary cache.

use binary_install::Cache;
use std::env;
use std::path::Path;

/// Get wasm-pack's binary cache.
pub fn get_wasm_pack_cache() -> Result<Cache, failure::Error> {
    if let Ok(path) = env::var("WASM_PACK_CACHE") {
        Ok(Cache::at(Path::new(&path)))
    } else {
        Cache::new("wasm-pack")
    }
}
