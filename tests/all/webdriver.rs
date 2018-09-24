use utils::fixture;
use wasm_pack::test::webdriver;

#[test]
#[cfg(
    any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "x86")
    )
)]
fn can_install_chromedriver() {
    let fixture = fixture::js_hello_world();
    assert!(webdriver::install_chromedriver(&fixture.path).is_ok());
}

#[test]
#[cfg(
    any(
        all(target_os = "linux", target_arch = "x86"),
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "x86"),
        all(target_os = "windows", target_arch = "x86_64")
    )
)]
fn can_install_geckodriver() {
    let fixture = fixture::js_hello_world();
    assert!(webdriver::install_geckodriver(&fixture.path).is_ok());
}
