//! Implementation of the `wasm-pack test` command.

use super::build::BuildMode;
use bindgen;
use build;
use command::utils::set_crate_path;
use console::style;
use emoji;
use failure::Error;
use indicatif::HumanDuration;
use lockfile::Lockfile;
use log::info;
use manifest;
use progressbar::Step;
use std::path::PathBuf;
use std::time::Instant;
use test::{self, webdriver};
use wasm_pack_binary_install::Cache;
use PBAR;

#[derive(Debug, Default, StructOpt)]
/// Everything required to configure the `wasm-pack test` command.
pub struct TestOptions {
    #[structopt(parse(from_os_str))]
    /// The path to the Rust crate.
    pub path: Option<PathBuf>,

    #[structopt(long = "node")]
    /// Run the tests in Node.js.
    pub node: bool,

    #[structopt(long = "firefox")]
    /// Run the tests in Firefox. This machine must have a Firefox installation.
    /// If the `geckodriver` WebDriver client is not on the `$PATH`, and not
    /// specified with `--geckodriver`, then `wasm-pack` will download a local
    /// copy.
    pub firefox: bool,

    #[structopt(long = "geckodriver", parse(from_os_str))]
    /// The path to the `geckodriver` WebDriver client for testing in
    /// Firefox. Implies `--firefox`.
    pub geckodriver: Option<PathBuf>,

    #[structopt(long = "chrome")]
    /// Run the tests in Chrome. This machine must have a Chrome installation.
    /// If the `chromedriver` WebDriver client is not on the `$PATH`, and not
    /// specified with `--chromedriver`, then `wasm-pack` will download a local
    /// copy.
    pub chrome: bool,

    #[structopt(long = "chromedriver", parse(from_os_str))]
    /// The path to the `chromedriver` WebDriver client for testing in
    /// Chrome. Implies `--chrome`.
    pub chromedriver: Option<PathBuf>,

    #[structopt(long = "safari")]
    /// Run the tests in Safari. This machine must have a Safari installation,
    /// and the `safaridriver` WebDriver client must either be on the `$PATH` or
    /// specified explicitly with the `--safaridriver` flag. `wasm-pack` cannot
    /// download the `safaridriver` WebDriver client for you.
    pub safari: bool,

    #[structopt(long = "safaridriver", parse(from_os_str))]
    /// The path to the `safaridriver` WebDriver client for testing in
    /// Safari. Implies `--safari`.
    pub safaridriver: Option<PathBuf>,

    #[structopt(long = "headless")]
    /// When running browser tests, run the browser in headless mode without any
    /// UI or windows.
    pub headless: bool,

    #[structopt(long = "mode", short = "m", default_value = "normal")]
    /// Sets steps to be run. [possible values: no-install, normal]
    pub mode: BuildMode,

    #[structopt(long = "release", short = "r")]
    /// Build with the release profile.
    pub release: bool,
}

/// A configured `wasm-pack test` command.
pub struct Test {
    crate_path: PathBuf,
    crate_data: manifest::CrateData,
    cache: Cache,
    node: bool,
    mode: BuildMode,
    firefox: bool,
    geckodriver: Option<PathBuf>,
    chrome: bool,
    chromedriver: Option<PathBuf>,
    safari: bool,
    safaridriver: Option<PathBuf>,
    headless: bool,
    release: bool,
    test_runner_path: Option<PathBuf>,
}

type TestStep = fn(&mut Test, &Step) -> Result<(), Error>;

impl Test {
    /// Construct a test command from the given options.
    pub fn try_from_opts(test_opts: TestOptions) -> Result<Self, Error> {
        let TestOptions {
            path,
            node,
            mode,
            headless,
            release,
            chrome,
            chromedriver,
            firefox,
            geckodriver,
            safari,
            safaridriver,
        } = test_opts;

        let crate_path = set_crate_path(path)?;
        let crate_data = manifest::CrateData::new(&crate_path)?;
        let any_browser = chrome || firefox || safari;

        if !node && !any_browser {
            bail!("Must specify at least one of `--node`, `--chrome`, `--firefox`, or `--safari`")
        }

        if headless && !any_browser {
            bail!(
                "The `--headless` flag only applies to browser tests. Node does not provide a UI, \
                 so it doesn't make sense to talk about a headless version of Node tests."
            )
        }

        Ok(Test {
            cache: Cache::new()?,
            crate_path,
            crate_data,
            node,
            mode,
            chrome,
            chromedriver,
            firefox,
            geckodriver,
            safari,
            safaridriver,
            headless,
            release,
            test_runner_path: None,
        })
    }

    /// Configures the cache that this test command uses
    pub fn set_cache(&mut self, cache: Cache) {
        self.cache = cache;
    }

    /// Execute this test command.
    pub fn run(mut self) -> Result<(), Error> {
        let process_steps = self.get_process_steps();
        let mut step_counter = Step::new(process_steps.len());

        let started = Instant::now();
        for (_, process_step) in process_steps {
            process_step(&mut self, &step_counter)?;
            step_counter.inc();
        }
        let duration = HumanDuration(started.elapsed());
        info!("Done in {}.", &duration);

        Ok(())
    }

    fn get_process_steps(&self) -> Vec<(&'static str, TestStep)> {
        macro_rules! steps {
            ($($name:ident $(if $e:expr)* ),+) => {
                {
                    let mut steps: Vec<(&'static str, TestStep)> = Vec::new();
                    $(
                        $(if $e)* {
                            steps.push((stringify!($name), Test::$name));
                        }
                    )*
                    steps
                }
            };
            ($($name:ident $(if $e:expr)* ,)*) => (steps![$($name $(if $e)* ),*])
        }
        match self.mode {
            BuildMode::Normal => steps![
                step_check_rustc_version,
                step_add_wasm_target,
                step_build_tests,
                step_install_wasm_bindgen,
                step_test_node if self.node,
                step_get_chromedriver if self.chrome && self.chromedriver.is_none(),
                step_test_chrome if self.chrome,
                step_get_geckodriver if self.firefox && self.geckodriver.is_none(),
                step_test_firefox if self.firefox,
                step_get_safaridriver if self.safari && self.safaridriver.is_none(),
                step_test_safari if self.safari,
            ],
            BuildMode::Force => steps![
                step_add_wasm_target,
                step_build_tests,
                step_install_wasm_bindgen,
                step_test_node if self.node,
                step_get_chromedriver if self.chrome && self.chromedriver.is_none(),
                step_test_chrome if self.chrome,
                step_get_geckodriver if self.firefox && self.geckodriver.is_none(),
                step_test_firefox if self.firefox,
                step_get_safaridriver if self.safari && self.safaridriver.is_none(),
                step_test_safari if self.safari,
            ],
            BuildMode::Noinstall => steps![
                step_build_tests,
                step_install_wasm_bindgen,
                step_test_node if self.node,
                step_get_chromedriver if self.chrome && self.chromedriver.is_none(),
                step_test_chrome if self.chrome,
                step_get_geckodriver if self.firefox && self.geckodriver.is_none(),
                step_test_firefox if self.firefox,
                step_get_safaridriver if self.safari && self.safaridriver.is_none(),
                step_test_safari if self.safari,
            ],
        }
    }

    fn step_check_rustc_version(&mut self, step: &Step) -> Result<(), Error> {
        info!("Checking rustc version...");
        let _ = build::check_rustc_version(step)?;
        info!("Rustc version is correct.");
        Ok(())
    }

    fn step_add_wasm_target(&mut self, step: &Step) -> Result<(), Error> {
        info!("Adding wasm-target...");
        build::rustup_add_wasm_target(step)?;
        info!("Adding wasm-target was successful.");
        Ok(())
    }

    fn step_build_tests(&mut self, step: &Step) -> Result<(), Error> {
        info!("Compiling tests to wasm...");

        let msg = format!("{}Compiling tests to WASM...", emoji::CYCLONE);
        PBAR.step(step, &msg);

        build::cargo_build_wasm_tests(&self.crate_path, !self.release)?;

        info!("Finished compiling tests to wasm.");
        Ok(())
    }

    fn step_install_wasm_bindgen(&mut self, step: &Step) -> Result<(), Error> {
        info!("Identifying wasm-bindgen dependency...");
        let lockfile = Lockfile::new(&self.crate_data)?;
        let bindgen_version = lockfile.require_wasm_bindgen()?;

        // Unlike `wasm-bindgen` and `wasm-bindgen-cli`, `wasm-bindgen-test`
        // will work with any semver compatible `wasm-bindgen-cli`, so just make
        // sure that it is depended upon, so we can run tests on
        // `wasm32-unkown-unknown`. Don't enforce that it is the same version as
        // `wasm-bindgen`.
        if lockfile.wasm_bindgen_test_version().is_none() {
            bail!(
                "Ensure that you have \"{}\" as a dependency in your Cargo.toml file:\n\
                 [dev-dependencies]\n\
                 wasm-bindgen-test = \"0.2\"",
                style("wasm-bindgen-test").bold().dim(),
            )
        }

        let install_permitted = match self.mode {
            BuildMode::Normal => {
                info!("Ensuring wasm-bindgen-cli is installed...");
                true
            }
            BuildMode::Force => {
                info!("Ensuring wasm-bindgen-cli is installed...");
                true
            }
            BuildMode::Noinstall => {
                info!("Searching for existing wasm-bindgen-cli install...");
                false
            }
        };

        let dl =
            bindgen::install_wasm_bindgen(&self.cache, &bindgen_version, install_permitted, step)?;

        self.test_runner_path = Some(dl.binary("wasm-bindgen-test-runner"));

        info!("Getting wasm-bindgen-cli was successful.");
        Ok(())
    }

    fn step_test_node(&mut self, step: &Step) -> Result<(), Error> {
        assert!(self.node);
        info!("Running tests in node...");
        PBAR.step(step, "Running tests in node...");
        test::cargo_test_wasm(
            &self.crate_path,
            self.release,
            Some((
                "CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER",
                &self.test_runner_path.as_ref().unwrap(),
            )),
        )?;
        info!("Finished running tests in node.");
        Ok(())
    }

    fn step_get_chromedriver(&mut self, step: &Step) -> Result<(), Error> {
        PBAR.step(step, "Getting chromedriver...");
        assert!(self.chrome && self.chromedriver.is_none());

        self.chromedriver = Some(webdriver::get_or_install_chromedriver(
            &self.cache,
            self.mode,
        )?);
        Ok(())
    }

    fn step_test_chrome(&mut self, step: &Step) -> Result<(), Error> {
        PBAR.step(step, "Running tests in Chrome...");

        let chromedriver = self.chromedriver.as_ref().unwrap().display().to_string();
        let chromedriver = chromedriver.as_str();
        info!(
            "Running tests in Chrome with chromedriver at {}",
            chromedriver
        );

        let test_runner = self
            .test_runner_path
            .as_ref()
            .unwrap()
            .display()
            .to_string();
        let test_runner = test_runner.as_str();
        info!("Using wasm-bindgen test runner at {}", test_runner);

        let mut envs = vec![
            ("CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER", test_runner),
            ("CHROMEDRIVER", chromedriver),
        ];
        if !self.headless {
            envs.push(("NO_HEADLESS", "1"));
        }

        test::cargo_test_wasm(&self.crate_path, self.release, envs)?;
        Ok(())
    }

    fn step_get_geckodriver(&mut self, step: &Step) -> Result<(), Error> {
        PBAR.step(step, "Getting geckodriver...");
        assert!(self.firefox && self.geckodriver.is_none());

        self.geckodriver = Some(webdriver::get_or_install_geckodriver(
            &self.cache,
            self.mode,
        )?);
        Ok(())
    }

    fn step_test_firefox(&mut self, step: &Step) -> Result<(), Error> {
        PBAR.step(step, "Running tests in Firefox...");

        let geckodriver = self.geckodriver.as_ref().unwrap().display().to_string();
        let geckodriver = geckodriver.as_str();
        info!(
            "Running tests in Firefox with geckodriver at {}",
            geckodriver
        );

        let test_runner = self
            .test_runner_path
            .as_ref()
            .unwrap()
            .display()
            .to_string();
        let test_runner = test_runner.as_str();
        info!("Using wasm-bindgen test runner at {}", test_runner);

        let mut envs = vec![
            ("CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER", test_runner),
            ("GECKODRIVER", geckodriver),
        ];
        if !self.headless {
            envs.push(("NO_HEADLESS", "1"));
        }

        test::cargo_test_wasm(&self.crate_path, self.release, envs)?;
        Ok(())
    }

    fn step_get_safaridriver(&mut self, step: &Step) -> Result<(), Error> {
        PBAR.step(step, "Getting safaridriver...");
        assert!(self.safari && self.safaridriver.is_none());

        self.safaridriver = Some(webdriver::get_safaridriver()?);
        Ok(())
    }

    fn step_test_safari(&mut self, step: &Step) -> Result<(), Error> {
        PBAR.step(step, "Running tests in Safari...");

        let safaridriver = self.safaridriver.as_ref().unwrap().display().to_string();
        let safaridriver = safaridriver.as_str();
        info!(
            "Running tests in Safari with safaridriver at {}",
            safaridriver
        );

        let test_runner = self
            .test_runner_path
            .as_ref()
            .unwrap()
            .display()
            .to_string();
        let test_runner = test_runner.as_str();
        info!("Using wasm-bindgen test runner at {}", test_runner);

        let mut envs = vec![
            ("CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER", test_runner),
            ("SAFARIDRIVER", safaridriver),
        ];
        if !self.headless {
            envs.push(("NO_HEADLESS", "1"));
        }

        test::cargo_test_wasm(&self.crate_path, self.release, envs)?;
        Ok(())
    }
}
