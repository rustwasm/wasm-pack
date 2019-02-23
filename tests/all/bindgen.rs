use binary_install::Cache;
use tempfile;
use wasm_pack::bindgen;

#[test]
#[cfg(any(
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "x86_64"),
    all(windows, target_arch = "x86_64"),
))]
fn can_download_prebuilt_wasm_bindgen() {
    let dir = tempfile::TempDir::new().unwrap();
    let cache = Cache::at(dir.path());
    let dl = bindgen::download_prebuilt_wasm_bindgen(&cache, "0.2.37", true).unwrap();
    assert!(dl.binary("wasm-bindgen").unwrap().is_file());
    assert!(dl.binary("wasm-bindgen-test-runner").unwrap().is_file())
}

#[test]
#[cfg(any(
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "x86_64"),
    all(windows, target_arch = "x86_64"),
))]
fn downloading_prebuilt_wasm_bindgen_handles_http_errors() {
    let dir = tempfile::TempDir::new().unwrap();
    let bad_version = "0.2.37-some-trailing-version-stuff-that-does-not-exist";
    let cache = Cache::at(dir.path());
    let result = bindgen::download_prebuilt_wasm_bindgen(&cache, bad_version, true);
    assert!(result.is_err());
    let error = result.err().unwrap();

    assert!(error.iter_chain().any(|e| e.to_string().contains("404")));
    assert!(error
        .iter_chain()
        .any(|e| e.to_string().contains(bad_version)));
}
