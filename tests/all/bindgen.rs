use tempfile;
use wasm_pack::bindgen;

#[test]
#[cfg(any(
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "x86_64"),
    all(windows, target_arch = "x86_64"),
))]
fn can_download_prebuilt_wasm_bindgen() {
    use std::env;
    let dir = tempfile::TempDir::new().unwrap();
    bindgen::download_prebuilt_wasm_bindgen(dir.path(), "0.2.21").unwrap();
    assert!(
        dir.path()
            .join("bin")
            .join("wasm-bindgen")
            .with_extension(env::consts::EXE_EXTENSION)
            .is_file()
    );
    assert!(
        dir.path()
            .join("bin")
            .join("wasm-bindgen-test-runner")
            .with_extension(env::consts::EXE_EXTENSION)
            .is_file()
    );
}

#[test]
#[cfg(any(
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "x86_64"),
    all(windows, target_arch = "x86_64"),
))]
fn downloading_prebuilt_wasm_bindgen_handles_http_errors() {
    let dir = tempfile::TempDir::new().unwrap();
    let bad_version = "0.2.21-some-trailing-version-stuff-that-does-not-exist";
    let result = bindgen::download_prebuilt_wasm_bindgen(dir.path(), bad_version);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("404"));
    assert!(error_msg.contains(bad_version));
}
