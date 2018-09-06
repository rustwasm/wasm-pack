use std::env;
use std::fs;
use tempfile;
use utils::fixture::fixture;
use wasm_pack::binaries;
use wasm_pack::command::{self, build, test, Command};
use wasm_pack::logger;

#[test]
fn it_can_run_node_tests() {
    let fixture = fixture("tests/fixtures/wbg-test-node");
    fixture.install_local_wasm_bindgen();
    let cmd = Command::Test(test::TestOptions {
        path: Some(fixture.path.clone()),
        node: true,
        mode: build::BuildMode::Noinstall,
        ..Default::default()
    });
    let logger = logger::new(&cmd, 3).unwrap();
    command::run_wasm_pack(cmd, &logger).expect("should run test command OK");
}

#[test]
#[cfg(any(
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "x86_64"),
    all(target_os = "windows", target_arch = "x86"),
    all(target_os = "windows", target_arch = "x86_64")
))]
fn it_can_run_browser_tests() {
    let fixture = fixture("tests/fixtures/wbg-test-browser");
    fixture.install_local_wasm_bindgen();

    let firefox = cfg!(any(
        all(target_os = "linux", target_arch = "x86"),
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "x86"),
        all(target_os = "windows", target_arch = "x86_64")
    ));
    if firefox {
        fixture.install_local_geckodriver();
    }

    let chrome = cfg!(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "x86")
    ));
    if chrome {
        fixture.install_local_chromedriver();
    }

    let safari = cfg!(target_os = "macos");

    if !firefox && !chrome && !safari {
        return;
    }

    let cmd = Command::Test(test::TestOptions {
        path: Some(fixture.path.clone()),
        firefox,
        chrome,
        safari,
        headless: true,
        mode: build::BuildMode::Noinstall,
        ..Default::default()
    });

    let logger = logger::new(&cmd, 3).unwrap();
    command::run_wasm_pack(cmd, &logger).expect("should run test command OK");
}

#[test]
fn it_can_run_failing_tests() {
    let fixture = fixture("tests/fixtures/wbg-test-fail");
    fixture.install_local_wasm_bindgen();
    let cmd = Command::Test(test::TestOptions {
        path: Some(fixture.path.clone()),
        node: true,
        mode: build::BuildMode::Noinstall,
        ..Default::default()
    });
    let logger = logger::new(&cmd, 3).unwrap();
    assert!(
        command::run_wasm_pack(cmd, &logger).is_err(),
        "failing tests should return Err"
    );
}

#[test]
#[cfg(any(
    all(target_os = "linux", target_arch = "x86"),
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "x86_64"),
    all(target_os = "windows", target_arch = "x86"),
    all(target_os = "windows", target_arch = "x86_64")
))]
fn it_can_find_a_webdriver_on_path() {
    let fixture = fixture("tests/fixtures/wbg-test-browser");
    fixture.install_local_wasm_bindgen();
    fixture.install_local_geckodriver();

    let geckodriver_dir = tempfile::TempDir::new().unwrap();
    let local_geckodriver = binaries::local_bin_path(&fixture.path, "geckodriver");
    fs::copy(
        &local_geckodriver,
        geckodriver_dir
            .path()
            .join(local_geckodriver.file_name().unwrap()),
    ).unwrap();
    fs::remove_file(&local_geckodriver).unwrap();

    let mut paths: Vec<_> = env::split_paths(&env::var("PATH").unwrap()).collect();
    paths.insert(0, geckodriver_dir.path().into());
    env::set_var("PATH", env::join_paths(paths).unwrap());

    let cmd = Command::Test(test::TestOptions {
        path: Some(fixture.path.clone()),
        firefox: true,
        headless: true,
        mode: build::BuildMode::Noinstall,
        ..Default::default()
    });
    let logger = logger::new(&cmd, 3).unwrap();
    command::run_wasm_pack(cmd, &logger).expect("should run test command OK");
}

#[test]
fn it_requires_node_or_a_browser() {
    let fixture = fixture("tests/fixtures/wbg-test-node");
    fixture.install_local_wasm_bindgen();

    let cmd = Command::Test(test::TestOptions {
        path: Some(fixture.path.clone()),
        mode: build::BuildMode::Noinstall,
        // Note: not setting node or any browser to true here.
        ..Default::default()
    });
    let logger = logger::new(&cmd, 3).unwrap();
    assert!(
        command::run_wasm_pack(cmd, &logger).is_err(),
        "need to enable node or browser testing"
    );
}

#[test]
fn the_headless_flag_requires_a_browser() {
    let fixture = fixture("tests/fixtures/wbg-test-node");
    fixture.install_local_wasm_bindgen();

    let cmd = Command::Test(test::TestOptions {
        path: Some(fixture.path.clone()),
        node: true,
        mode: build::BuildMode::Noinstall,
        headless: true,
        ..Default::default()
    });
    let logger = logger::new(&cmd, 3).unwrap();
    assert!(
        command::run_wasm_pack(cmd, &logger).is_err(),
        "running headless tests in node doesn't make sense"
    );
}
