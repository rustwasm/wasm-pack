#[test]
#[cfg(any(
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "macos", any(target_arch = "x86_64", target_arch = "aarch64")),
    all(target_os = "windows", target_arch = "x86"),
    all(target_os = "windows", target_arch = "x86_64")
))]
fn can_install_chromedriver() {
    let fixture = crate::utils::fixture::js_hello_world();
    let cache = binary_install::Cache::at(&fixture.path);
    assert!(wasm_pack::test::webdriver::install_chromedriver(&cache, true).is_ok());
}

#[test]
#[cfg(any(
    all(target_os = "linux", target_arch = "x86"),
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "macos", any(target_arch = "x86_64", target_arch = "aarch64")),
    all(target_os = "windows", target_arch = "x86"),
    all(target_os = "windows", target_arch = "x86_64")
))]
fn can_install_geckodriver() {
    let fixture = crate::utils::fixture::js_hello_world();
    let cache = binary_install::Cache::at(&fixture.path);
    assert!(wasm_pack::test::webdriver::install_geckodriver(&cache, true).is_ok());
}
