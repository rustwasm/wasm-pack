use assert_cmd::prelude::*;
use failure::Error;
use predicates::prelude::*;
use std::env;
use utils::fixture;
use wasm_pack::command::{build, test, Command};

fn assert_err<T>(result: Result<T, Error>, msg: &str) -> Error {
    let error = result.err().expect("should have failed");
    for e in error.iter_chain() {
        println!("err: {}", e);
    }
    assert!(error.iter_chain().any(|e| e.to_string().contains(msg)));
    error
}

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
    fixture.run(cmd).unwrap();
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
    fixture.run(cmd).unwrap();
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

    fixture.run(cmd).unwrap();
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
    assert_err(
        fixture.run(cmd),
        "Running Wasm tests with wasm-bindgen-test failed",
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
    use std::process::Command;

    let fixture = fixture::wbg_test_browser();
    let local_geckodriver = fixture.install_local_geckodriver();
    let local_wasm_bindgen = fixture.install_local_wasm_bindgen();

    let mut paths: Vec<_> = env::split_paths(&env::var("PATH").unwrap()).collect();
    paths.insert(0, local_geckodriver.parent().unwrap().to_path_buf());
    paths.insert(0, local_wasm_bindgen.parent().unwrap().to_path_buf());
    let path = env::join_paths(paths).unwrap();

    let _lock = fixture.lock();

    let mut me = env::current_exe().unwrap();
    me.pop();
    me.pop();
    me.push("wasm-pack");
    let output = Command::new(&me)
        .arg("test")
        .arg("--firefox")
        .arg("--headless")
        .arg("--mode")
        .arg("no-install")
        .env("PATH", &path)
        .arg(&fixture.path)
        .output()
        .unwrap();
    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    println!("status: {}", output.status);
    assert!(output.status.success());
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
    assert_err(fixture.run(cmd), "Must specify at least one of");
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
    assert_err(fixture.run(cmd), "only applies to browser tests");
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
    let error = assert_err(fixture.run(cmd), "Ensure that you have");

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

    let err_msg = error.to_string();
    let first = err_msg.find("wasm-bindgen-test");
    assert!(first.is_some());
    let second = err_msg.rfind("wasm-bindgen-test");
    assert!(second.is_some());
    assert_ne!(first, second, "should have found two occurrences");
}

#[test]
fn renamed_crate_name_works() {
    let fixture = fixture::Fixture::new();
    fixture
        .readme()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                authors = []

                [lib]
                crate-type = ["cdylib"]
                name = 'bar'

                [dependencies]
                wasm-bindgen = "=0.2.21"

                [dev-dependencies]
                wasm-bindgen-test = "=0.2.21"
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                extern crate wasm_bindgen;
                use wasm_bindgen::prelude::*;

                #[wasm_bindgen]
                pub fn one() -> u32 { 1 }
            "#,
        );
    fixture.install_local_wasm_bindgen();
    let cmd = Command::Test(test::TestOptions {
        path: Some(fixture.path.clone()),
        node: true,
        mode: build::BuildMode::Noinstall,
        ..Default::default()
    });
    fixture.run(cmd).unwrap();
}

#[test]
fn cdylib_not_required() {
    let fixture = fixture::Fixture::new();
    fixture
        .readme()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                authors = []

                [dependencies]
                wasm-bindgen = "=0.2.21"

                [dev-dependencies]
                wasm-bindgen-test = "=0.2.21"
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                pub fn foo() -> u32 { 1 }
            "#,
        )
        .file(
            "tests/foo.rs",
            r#"
                extern crate wasm_bindgen_test;
                use wasm_bindgen_test::*;

                #[wasm_bindgen_test]
                fn smoke() {
                    foo::foo();
                }
            "#,
        );
    fixture.install_local_wasm_bindgen();
    let cmd = Command::Test(test::TestOptions {
        path: Some(fixture.path.clone()),
        node: true,
        mode: build::BuildMode::Noinstall,
        ..Default::default()
    });
    fixture.run(cmd).unwrap();
}

#[test]
fn test_output_is_printed_once() {
    let fixture = fixture::Fixture::new();
    fixture
        .readme()
        .cargo_toml("wbg-test-node")
        .hello_world_src_lib()
        .file(
            "tests/node.rs",
            r#"
                extern crate wasm_bindgen;
                extern crate wasm_bindgen_test;
                use wasm_bindgen::prelude::*;
                use wasm_bindgen_test::*;

                #[wasm_bindgen]
                extern {
                    #[wasm_bindgen(js_namespace = console)]
                    fn log(s: &str);
                }

                #[wasm_bindgen_test]
                fn pass() {
                    log("YABBA DABBA DOO");
                    assert_eq!(1, 2);
                }
            "#,
        );

    fixture
        .wasm_pack()
        .arg("test")
        .arg("--node")
        .assert()
        .stderr(predicate::function(|err: &str| {
            err.matches("YABBA DABBA DOO").count() == 1
        }))
        .failure();
}
