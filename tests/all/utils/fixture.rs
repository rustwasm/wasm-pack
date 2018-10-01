use std::env;
use std::fs;
use std::io;
use std::mem::ManuallyDrop;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Once, ONCE_INIT};
use std::thread;
use wasm_pack;

use tempfile::TempDir;

fn hard_link_or_copy<P1: AsRef<Path>, P2: AsRef<Path>>(from: P1, to: P2) -> io::Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();
    fs::hard_link(from, to).or_else(|_| fs::copy(from, to).map(|_| ()))
}

/// A test fixture in a temporary directory.
pub struct Fixture {
    // NB: we wrap the fixture's tempdir in a `ManuallyDrop` so that if a test
    // fails, its directory isn't deleted, and we have a chance to manually
    // inspect its state and figure out what is going on.
    pub dir: ManuallyDrop<TempDir>,
    pub path: PathBuf,
}

impl Fixture {
    /// Create a new test fixture in a temporary directory.
    pub fn new() -> Fixture {
        // Make sure that all fixtures end up sharing a target dir, and we don't
        // recompile wasm-bindgen and friends many times over.
        static SET_TARGET_DIR: Once = ONCE_INIT;
        let target_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target");
        SET_TARGET_DIR.call_once(|| {
            env::set_var("CARGO_TARGET_DIR", &target_dir);
        });

        let root = target_dir.join("t");
        fs::create_dir_all(&root).unwrap();
        let dir = TempDir::new_in(&root).unwrap();
        let path = dir.path().join("wasm-pack");
        eprintln!("Created fixture at {}", path.display());
        Fixture {
            dir: ManuallyDrop::new(dir),
            path,
        }
    }

    /// Create a file within this fixture.
    ///
    /// `path` should be a relative path to the file (relative within this
    /// fixture's path).
    ///
    /// The `contents` are written to the file.
    pub fn file<P: AsRef<Path>, C: AsRef<[u8]>>(&self, path: P, contents: C) -> &Self {
        assert!(path.as_ref().is_relative());
        let path = self.path.join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
        self
    }

    /// Add a generic `README.md` file to the fixture.
    pub fn readme(&self) -> &Self {
        self.file(
            "README.md",
            r#"
                # Fixture!
                > an example rust -> wasm project
            "#,
        )
    }

    /// Add a `Cargo.toml` with a correctly configured `wasm-bindgen`
    /// dependency, `wasm-bindgen-test` dev-dependency, and `crate-type =
    /// ["cdylib"]`.
    ///
    /// `name` is the crate's name.
    pub fn cargo_toml(&self, name: &str) -> &Self {
        self.file(
            "Cargo.toml",
            &format!(
                r#"
                    [package]
                    authors = ["The wasm-pack developers"]
                    description = "so awesome rust+wasm package"
                    license = "WTFPL"
                    name = "{}"
                    repository = "https://github.com/rustwasm/wasm-pack.git"
                    version = "0.1.0"

                    [lib]
                    crate-type = ["cdylib"]

                    [dependencies]
                    wasm-bindgen = "=0.2.21"

                    [dev-dependencies]
                    wasm-bindgen-test = "=0.2.21"
                "#,
                name
            ),
        )
    }

    /// Add a `src/lib.rs` file that contains a "hello world" program.
    pub fn hello_world_src_lib(&self) -> &Self {
        self.file(
            "src/lib.rs",
            r#"
                extern crate wasm_bindgen;
                use wasm_bindgen::prelude::*;

                // Import the `window.alert` function from the Web.
                #[wasm_bindgen]
                extern {
                    fn alert(s: &str);
                }

                // Export a `greet` function from Rust to JavaScript, that alerts a
                // hello message.
                #[wasm_bindgen]
                pub fn greet(name: &str) {
                    alert(&format!("Hello, {}!", name));
                }
            "#,
        )
    }

    /// Install a local wasm-bindgen for this fixture.
    ///
    /// Takes care not to re-install for every fixture, but only the one time
    /// for the whole test suite.
    pub fn install_local_wasm_bindgen(&self) -> &Self {
        static INSTALL_WASM_BINDGEN: Once = ONCE_INIT;

        let tests = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
        let shared_wasm_bindgen = wasm_pack::binaries::local_bin_path(&tests, "wasm-bindgen");
        let shared_wasm_bindgen_test_runner =
            wasm_pack::binaries::local_bin_path(&tests, "wasm-bindgen-test-runner");

        INSTALL_WASM_BINDGEN.call_once(|| {
            if shared_wasm_bindgen.is_file() {
                assert!(shared_wasm_bindgen_test_runner.is_file());
                return;
            }

            const WASM_BINDGEN_VERSION: &str = "0.2.21";
            wasm_pack::bindgen::download_prebuilt_wasm_bindgen(&tests, WASM_BINDGEN_VERSION)
                .or_else(|_| {
                    wasm_pack::bindgen::cargo_install_wasm_bindgen(&tests, WASM_BINDGEN_VERSION)
                })
                .unwrap();
        });

        assert!(shared_wasm_bindgen.is_file());
        assert!(shared_wasm_bindgen_test_runner.is_file());

        wasm_pack::binaries::ensure_local_bin_dir(&self.path).unwrap();

        hard_link_or_copy(
            &shared_wasm_bindgen,
            wasm_pack::binaries::local_bin_path(&self.path, "wasm-bindgen"),
        )
        .expect("could not copy `wasm-bindgen` to fixture directory");

        hard_link_or_copy(
            &shared_wasm_bindgen_test_runner,
            wasm_pack::binaries::local_bin_path(&self.path, "wasm-bindgen-test-runner"),
        )
        .expect("could not copy `wasm-bindgen-test` to fixture directory");

        self
    }

    /// Download `geckodriver` and return its path.
    ///
    /// Takes care to ensure that only one `geckodriver` is downloaded for the whole
    /// test suite.
    pub fn install_local_geckodriver(&self) -> &Self {
        static FETCH_GECKODRIVER: Once = ONCE_INIT;

        let tests = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");

        wasm_pack::binaries::ensure_local_bin_dir(&tests)
            .expect("could not create fixture's `bin` directory");

        let geckodriver = wasm_pack::binaries::local_bin_path(&tests, "geckodriver");

        FETCH_GECKODRIVER.call_once(|| {
            if geckodriver.is_file() {
                return;
            }

            wasm_pack::test::webdriver::install_geckodriver(&tests).unwrap();
            assert!(geckodriver.is_file());
        });

        wasm_pack::binaries::ensure_local_bin_dir(&self.path)
            .expect("could not create fixture's `bin` directory");

        hard_link_or_copy(
            &geckodriver,
            wasm_pack::binaries::local_bin_path(&self.path, "geckodriver"),
        )
        .expect("could not copy `geckodriver` to fixture directory");

        self
    }

    /// Download `chromedriver` and return its path.
    ///
    /// Takes care to ensure that only one `chromedriver` is downloaded for the whole
    /// test suite.
    pub fn install_local_chromedriver(&self) -> &Self {
        static FETCH_CHROMEDRIVER: Once = ONCE_INIT;

        let tests = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");

        wasm_pack::binaries::ensure_local_bin_dir(&tests)
            .expect("could not create fixture's `bin` directory");

        let chromedriver = wasm_pack::binaries::local_bin_path(&tests, "chromedriver");

        FETCH_CHROMEDRIVER.call_once(|| {
            if chromedriver.is_file() {
                return;
            }

            wasm_pack::test::webdriver::install_chromedriver(&tests).unwrap();
            assert!(chromedriver.is_file());
        });

        wasm_pack::binaries::ensure_local_bin_dir(&self.path)
            .expect("could not create fixture's `bin` directory");

        hard_link_or_copy(
            &chromedriver,
            wasm_pack::binaries::local_bin_path(&self.path, "chromedriver"),
        )
        .expect("could not copy `chromedriver` to fixture directory");

        self
    }

    /// The `step_install_wasm_bindgen` and `step_run_wasm_bindgen` steps only
    /// occur after the `step_build_wasm` step. In order to read the lockfile
    /// in the test fixture's temporary directory, we should first build the
    /// crate, targeting `wasm32-unknown-unknown`.
    pub fn cargo_check(&self) -> &Self {
        Command::new("cargo")
            .current_dir(&self.path)
            .arg("check")
            .arg("--target")
            .arg("wasm32-unknown-unknown")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap();
        self
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if !thread::panicking() {
            unsafe { ManuallyDrop::drop(&mut self.dir) }
        }
    }
}

pub fn bad_cargo_toml() -> Fixture {
    let fixture = Fixture::new();
    fixture.readme().hello_world_src_lib().file(
        "Cargo.toml",
        r#"
            [package]
            name = "bad-cargo-toml"
            version = "0.1.0"
            authors = ["The wasm-pack developers"]

            [lib]
            crate-type = ["foo"]

            [dependencies]
            # Note: no wasm-bindgen dependency!
        "#,
    );
    fixture
}

pub fn js_hello_world() -> Fixture {
    let fixture = Fixture::new();
    fixture
        .readme()
        .cargo_toml("js-hello-world")
        .hello_world_src_lib();
    fixture
}

pub fn no_cdylib() -> Fixture {
    let fixture = Fixture::new();
    fixture.readme().hello_world_src_lib().file(
        "Cargo.toml",
        r#"
            [package]
            authors = ["The wasm-pack developers"]
            description = "so awesome rust+wasm package"
            license = "WTFPL"
            name = "{}"
            repository = "https://github.com/rustwasm/wasm-pack.git"
            version = "0.1.0"

            # [lib]
            # crate-type = ["cdylib"]

            [dependencies]
            wasm-bindgen = "=0.2.21"

            [dev-dependencies]
            wasm-bindgen-test = "=0.2.21"
        "#,
    );
    fixture
}

pub fn not_a_crate() -> Fixture {
    let fixture = Fixture::new();
    fixture.file("README.md", "This is not a Rust crate!");
    fixture
}

pub fn serde_feature() -> Fixture {
    let fixture = Fixture::new();
    fixture.readme().hello_world_src_lib().file(
        "Cargo.toml",
        r#"
            [package]
            name = "serde-serialize"
            version = "0.1.0"
            authors = ["The wasm-pack developers"]

            [lib]
            crate-type = ["cdylib"]

            [dependencies.wasm-bindgen]
            version = "^0.2"
            features = ["serde-serialize"]
        "#,
    );
    fixture
}

pub fn wbg_test_diff_versions() -> Fixture {
    let fixture = Fixture::new();
    fixture
        .readme()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "wbg-test-diff-versions"
                version = "0.1.0"
                authors = ["The wasm-pack developers"]

                [lib]
                crate-type = ["cdylib", "rlib"]

                [dependencies]
                # We depend on wasm-bindgen 0.2.21
                wasm-bindgen = "=0.2.21"

                [dev-dependencies]
                # And we depend on wasm-bindgen-test 0.2.19. This should still
                # work, and we should end up with `wasm-bindgen` at 0.2.21 and
                # wasm-bindgen-test at 0.2.19, and everything should still work.
                wasm-bindgen-test = "0.2.19"
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
        )
        .file(
            "tests/node.rs",
            r#"
                extern crate wbg_test_diff_versions;
                extern crate wasm_bindgen_test;
                use wasm_bindgen_test::*;

                #[wasm_bindgen_test]
                fn pass() {
                    assert_eq!(wbg_test_diff_versions::one(), 1);
                }
            "#,
        );
    fixture
}

pub fn wbg_test_browser() -> Fixture {
    let fixture = Fixture::new();
    fixture
        .readme()
        .cargo_toml("wbg-test-browser")
        .hello_world_src_lib()
        .file(
            "tests/browser.rs",
            r#"
                extern crate wasm_bindgen_test;
                use wasm_bindgen_test::*;

                wasm_bindgen_test_configure!(run_in_browser);

                #[wasm_bindgen_test]
                fn pass() {
                    assert_eq!(1, 1);
                }
            "#,
        );
    fixture
}

pub fn wbg_test_fail() -> Fixture {
    let fixture = Fixture::new();
    fixture
        .readme()
        .cargo_toml("wbg-test-fail")
        .hello_world_src_lib()
        .file(
            "tests/node.rs",
            r#"
                extern crate wasm_bindgen_test;
                use wasm_bindgen_test::*;

                #[wasm_bindgen_test]
                fn pass() {
                    assert_eq!(1, 2);
                }
            "#,
        );
    fixture
}

pub fn wbg_test_node() -> Fixture {
    let fixture = Fixture::new();
    fixture
        .readme()
        .cargo_toml("wbg-test-node")
        .hello_world_src_lib()
        .file(
            "tests/node.rs",
            r#"
                extern crate wasm_bindgen_test;
                use wasm_bindgen_test::*;

                #[wasm_bindgen_test]
                fn pass() {
                    assert_eq!(1, 1);
                }
            "#,
        );
    fixture
}
