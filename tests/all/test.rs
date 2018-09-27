use std::env;
use std::fs;
use tempfile;
use utils::fixture;
use wasm_pack::binaries;
use wasm_pack::command::{self, build, test, Command};
use wasm_pack::logger;

#[test]
fn it_can_run_node_tests() {
    let fixture = fixture::wbg_test_node();
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
fn it_can_run_tests_with_different_wbg_test_and_wbg_versions() {
    let fixture = fixture::wbg_test_diff_versions();
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
    let fixture = fixture::wbg_test_browser();
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
    let fixture = fixture::wbg_test_fail();
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
    let fixture = fixture::wbg_test_browser();
    fixture.install_local_wasm_bindgen();
    fixture.install_local_geckodriver();

    let geckodriver_dir = tempfile::TempDir::new().unwrap();
    let local_geckodriver = binaries::local_bin_path(&fixture.path, "geckodriver");
    fs::copy(
        &local_geckodriver,
        geckodriver_dir
            .path()
            .join(local_geckodriver.file_name().unwrap()),
    )
    .unwrap();
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
    let fixture = fixture::wbg_test_node();
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
    let fixture = fixture::wbg_test_node();
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

#[test]
fn complains_about_missing_wasm_bindgen_test_dependency() {
    let fixture = fixture::Fixture::new();
    fixture
        .readme()
        .file(
            "Cargo.toml",
            r#"
                [package]
                authors = ["The wasm-pack developers"]
                description = "so awesome rust+wasm package"
                license = "WTFPL"
                name = "missing-wbg-test"
                repository = "https://github.com/rustwasm/wasm-pack.git"
                version = "0.1.0"

                [lib]
                crate-type = ["cdylib"]

                [dependencies]
                wasm-bindgen = "=0.2.21"

                [dev-dependencies]
                # no wasm-bindgen-test dep here!
            "#,
        )
        .hello_world_src_lib()
        .install_local_wasm_bindgen();

    let cmd = Command::Test(test::TestOptions {
        path: Some(fixture.path.clone()),
        node: true,
        mode: build::BuildMode::Noinstall,
        ..Default::default()
    });
    let logger = logger::new(&cmd, 3).unwrap();

    let result = command::run_wasm_pack(cmd, &logger);
    assert!(
        result.is_err(),
        "running tests without wasm-bindgen-test won't work"
    );

    // Test that the error message has two occurrences of "wasm-bindgen-test" in
    // it. I am surprised to learn there is no `str` method to count
    // occurrences, so we find the first and the last and assert that they
    // aren't the same occurrence.
    //
    // This should protect against regresstions where we said:
    //
    //     Ensure that you have "wasm-bindgen" as a dependency in your Cargo.toml file:
    //     [dev-dependencies]
    //     wasm-bindgen-test = "0.2"
    //
    // instead of
    //
    //     Ensure that you have "wasm-bindgen-test" as a dependency in your Cargo.toml file:
    //     [dev-dependencies]
    //     wasm-bindgen-test = "0.2"
    //
    // Note that the whole reason we are doing this string manipulation instead
    // of just doing `assert_eq!` is because the first occurrence of the
    // dependency name is bolded with terminal escape codes and if I try to pipe
    // the output to a text file, then the escape codes go away, so I can't
    // figure out which exact escape codes are even used here.

    let err_msg = result.unwrap_err().to_string();
    let first = err_msg.find("wasm-bindgen-test");
    assert!(first.is_some());
    let second = err_msg.rfind("wasm-bindgen-test");
    assert!(second.is_some());
    assert_ne!(first, second, "should have found two occurrences");
}
