use std::env;
use std::fs;
use std::mem::ManuallyDrop;
use std::path::{Path, PathBuf};
use std::sync::{Once, ONCE_INIT};
use std::thread;
use wasm_pack;

use copy_dir::copy_dir;
use tempfile;

pub struct Fixture {
    pub dir: ManuallyDrop<tempfile::TempDir>,
    pub path: PathBuf,
}

/// Copy the given fixture into a unique temporary directory. This allows the
/// test to mutate the copied fixture without messing up other tests that are
/// also trying to read from or write to that fixture. The given path should be
/// relative from the root of the repository, eg
/// "tests/fixtures/im-from-brooklyn-the-place-where-stars-are-born".
pub fn fixture<P>(fixture: P) -> Fixture
where
    P: AsRef<Path>,
{
    // Make sure that all fixtures end up sharing a target dir, and we don't
    // recompile wasm-bindgen and friends many times over.
    static SET_TARGET_DIR: Once = ONCE_INIT;
    SET_TARGET_DIR.call_once(|| {
        env::set_var(
            "CARGO_TARGET_DIR",
            Path::new(env!("CARGO_MANIFEST_DIR")).join("target"),
        );
    });

    let fixture = fixture
        .as_ref()
        .canonicalize()
        .expect("should canonicalize fixture path OK");
    let dir = ManuallyDrop::new(tempfile::tempdir().expect("should create temporary directory OK"));
    let path = dir.path().join("wasm-pack");
    println!(
        "wasm-pack: copying test fixture '{}' to temporary directory '{}'",
        fixture.display(),
        path.display()
    );

    {
        // Copying too many things in parallel totally kills my machine(??!!?!),
        // so make sure we are only doing one `copy_dir` at a time...
        use std::sync::Mutex;
        lazy_static! {
            static ref ONE_AT_A_TIME: Mutex<()> = Mutex::new(());
        }
        let _locked = ONE_AT_A_TIME.lock();

        copy_dir(fixture, &path)
            .expect("should copy fixture directory into temporary directory OK");
    }

    Fixture { dir, path }
}

impl Fixture {
    /// Install a local wasm-bindgen for this fixture.
    ///
    /// Takes care not to re-install for every fixture, but only the one time
    /// for the whole test suite.
    pub fn install_local_wasm_bindgen(&self) {
        static INSTALL_WASM_BINDGEN: Once = ONCE_INIT;

        let tests = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
        let bin = tests.join("bin");

        INSTALL_WASM_BINDGEN.call_once(|| {
            if bin.join("wasm-bindgen").is_file() {
                return;
            }

            const WASM_BINDGEN_VERSION: &str = "0.2.17";
            wasm_pack::bindgen::download_prebuilt_wasm_bindgen(&tests, WASM_BINDGEN_VERSION)
                .or_else(|_| {
                    wasm_pack::bindgen::cargo_install_wasm_bindgen(&tests, WASM_BINDGEN_VERSION)
                }).unwrap();
        });

        copy_dir(bin, self.path.join("bin")).expect("could not copy `bin` directory into temp dir");
    }

    /// Download `geckodriver` and return its path.
    ///
    /// Takes car to ensure that only one `geckodriver` is downloaded for the whole
    /// test suite.
    pub fn install_local_geckodriver(&self) {
        static FETCH_GECKODRIVER: Once = ONCE_INIT;

        let tests = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");

        let bin = tests.join("bin");
        fs::create_dir_all(&bin).expect("could not create `tests/bin` directory");

        let geckodriver = bin.join(if cfg!(target_os = "windows") {
            "geckodriver.exe"
        } else {
            "geckodriver"
        });

        FETCH_GECKODRIVER.call_once(|| {
            if geckodriver.is_file() {
                return;
            }

            wasm_pack::webdriver::install_geckodriver(&tests).unwrap();
            assert!(geckodriver.is_file());
        });

        let self_bin = self.path.join("bin");
        fs::create_dir_all(&self_bin).expect("could not create fixture's `bin` directory");

        fs::copy(
            &geckodriver,
            self_bin.join(geckodriver.file_name().unwrap()),
        ).expect("could not copy `geckodriver` to fixture directory");
    }

    /// Download `chromedriver` and return its path.
    ///
    /// Takes car to ensure that only one `chromedriver` is downloaded for the whole
    /// test suite.
    pub fn install_local_chromedriver(&self) {
        static FETCH_CHROMEDRIVER: Once = ONCE_INIT;

        let tests = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");

        let bin = tests.join("bin");
        fs::create_dir_all(&bin).expect("could not create `tests/bin` directory");

        let chromedriver = bin.join(if cfg!(target_os = "windows") {
            "chromedriver.exe"
        } else {
            "chromedriver"
        });

        FETCH_CHROMEDRIVER.call_once(|| {
            if chromedriver.is_file() {
                return;
            }

            wasm_pack::webdriver::install_chromedriver(&tests).unwrap();
            assert!(chromedriver.is_file());
        });

        let self_bin = self.path.join("bin");
        fs::create_dir_all(&self_bin).expect("could not create fixture's `bin` directory");

        fs::copy(
            &chromedriver,
            self_bin.join(chromedriver.file_name().unwrap()),
        ).expect("could not copy `chromedriver` to fixture directory");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if !thread::panicking() {
            unsafe { ManuallyDrop::drop(&mut self.dir) }
        }
    }
}
