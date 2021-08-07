#[test]
#[cfg(any(
    all(target_os = "linux", target_arch = "x86_64"),
    all(
        target_os = "macos",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ),
    all(windows, target_arch = "x86_64"),
))]
fn can_download_prebuilt_wasm_bindgen() {
    let dir = tempfile::TempDir::new().unwrap();
    let cache = binary_install::Cache::at(dir.path());
    if let wasm_pack::install::Status::Found(dl) = wasm_pack::install::download_prebuilt(
        &wasm_pack::install::Tool::WasmBindgen,
        &cache,
        "0.2.74",
        true,
    )
    .unwrap()
    {
        assert!(dl.binary("wasm-bindgen").unwrap().is_file());
        assert!(dl.binary("wasm-bindgen-test-runner").unwrap().is_file())
    } else {
        assert!(false, "Download failed")
    }
}

#[test]
#[cfg(any(
    all(target_os = "linux", target_arch = "x86_64"),
    all(
        target_os = "macos",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ),
    all(windows, target_arch = "x86_64"),
))]
fn downloading_prebuilt_wasm_bindgen_handles_http_errors() {
    let dir = tempfile::TempDir::new().unwrap();
    let bad_version = "0.2.74-some-trailing-version-stuff-that-does-not-exist";
    let cache = binary_install::Cache::at(dir.path());
    let result = wasm_pack::install::download_prebuilt(
        &wasm_pack::install::Tool::WasmBindgen,
        &cache,
        bad_version,
        true,
    );
    assert!(result.is_err());
    let error = result.err().unwrap();

    assert!(error.iter_chain().any(|e| e.to_string().contains("404")));
    assert!(error
        .iter_chain()
        .any(|e| e.to_string().contains(bad_version)));
}

#[test]
#[cfg(any(
    all(target_os = "linux", target_arch = "x86_64"),
    all(
        target_os = "macos",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ),
    all(windows, target_arch = "x86_64"),
))]
fn can_download_prebuilt_cargo_generate() {
    let dir = tempfile::TempDir::new().unwrap();
    let cache = binary_install::Cache::at(dir.path());
    if let wasm_pack::install::Status::Found(dl) = wasm_pack::install::download_prebuilt(
        &wasm_pack::install::Tool::CargoGenerate,
        &cache,
        "latest",
        true,
    )
    .unwrap()
    {
        assert!(dl.binary("cargo-generate").unwrap().is_file());
    } else {
        assert!(false, "Download Failed");
    }
}
