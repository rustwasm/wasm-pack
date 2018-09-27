//! Implementation of the `wasm-pack test` command.

use super::build::BuildMode;
use bindgen;
use build;
use command::utils::set_crate_path;
use console::style;
use emoji;
use error::Error;
use indicatif::HumanDuration;
use lockfile::Lockfile;
use manifest;
use progressbar::Step;
use slog::Logger;
use std::path::PathBuf;
use std::time::Instant;
use test::{self, webdriver};
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

type TestStep = fn(&mut Test, &Step, &Logger) -> Result<(), Error>;

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

        // let geckodriver = get_web_driver("geckodriver", test_opts.geckodriver, test_opts.firefox)?;
        // let chromedriver =
        //     get_web_driver("chromedriver", test_opts.chromedriver, test_opts.chrome)?;
        // let safaridriver =
        //     get_web_driver("safaridriver", test_opts.safaridriver, test_opts.safari)?;

        let any_browser = chrome || firefox || safari;

        if !node && !any_browser {
            return Error::crate_config(
                "Must specify at least one of `--node`, `--chrome`, `--firefox`, or `--safari`",
            )
            .map(|_| unreachable!());
        }

        if headless && !any_browser {
            return Error::crate_config(
                "The `--headless` flag only applies to browser tests. Node does not provide a UI, \
                 so it doesn't make sense to talk about a headless version of Node tests.",
            )
            .map(|_| unreachable!());
        }

        Ok(Test {
            crate_path,
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

    /// Execute this test command.
    pub fn run(mut self, log: &Logger) -> Result<(), Error> {
        let process_steps = self.get_process_steps();
        let mut step_counter = Step::new(process_steps.len());

        let started = Instant::now();
        for (_, process_step) in process_steps {
            process_step(&mut self, &step_counter, log)?;
            step_counter.inc();
        }
        let duration = HumanDuration(started.elapsed());
        info!(&log, "Done in {}.", &duration);

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
                step_check_crate_config,
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
                step_check_crate_config,
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

    fn step_check_rustc_version(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        info!(log, "Checking rustc version...");
        let _ = build::check_rustc_version(step)?;
        info!(log, "Rustc version is correct.");
        Ok(())
    }

    fn step_check_crate_config(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        info!(log, "Checking crate configuration...");
        manifest::check_crate_config(&self.crate_path, step)?;
        info!(log, "Crate is correctly configured.");
        Ok(())
    }

    fn step_add_wasm_target(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        info!(&log, "Adding wasm-target...");
        build::rustup_add_wasm_target(step)?;
        info!(&log, "Adding wasm-target was successful.");
        Ok(())
    }

    fn step_build_tests(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        info!(log, "Compiling tests to wasm...");

        let msg = format!("{}Compiling tests to WASM...", emoji::CYCLONE);
        PBAR.step(step, &msg);

        build::cargo_build_wasm_tests(&self.crate_path, !self.release)?;

        info!(log, "Finished compiling tests to wasm.");
        Ok(())
    }

    fn step_install_wasm_bindgen(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        info!(&log, "Identifying wasm-bindgen dependency...");
        let lockfile = Lockfile::new(&self.crate_path)?;
        let bindgen_version = lockfile.require_wasm_bindgen()?;

        // Unlike `wasm-bindgen` and `wasm-bindgen-cli`, `wasm-bindgen-test`
        // will work with any semver compatible `wasm-bindgen-cli`, so just make
        // sure that it is depended upon, so we can run tests on
        // `wasm32-unkown-unknown`. Don't enforce that it is the same version as
        // `wasm-bindgen`.
        if lockfile.wasm_bindgen_test_version().is_none() {
            let message = format!(
                "Ensure that you have \"{}\" as a dependency in your Cargo.toml file:\n\
                 [dev-dependencies]\n\
                 wasm-bindgen-test = \"0.2\"",
                style("wasm-bindgen-test").bold().dim(),
            );
            return Err(Error::CrateConfig { message });
        }

        let install_permitted = match self.mode {
            BuildMode::Normal => {
                info!(&log, "Ensuring wasm-bindgen-cli is installed...");
                true
            }
            BuildMode::Force => {
                info!(&log, "Ensuring wasm-bindgen-cli is installed...");
                true
            }
            BuildMode::Noinstall => {
                info!(&log, "Searching for existing wasm-bindgen-cli install...");
                false
            }
        };

        bindgen::install_wasm_bindgen(
            &self.crate_path,
            &bindgen_version,
            install_permitted,
            step,
            log,
        )?;

        self.test_runner_path = Some(bindgen::wasm_bindgen_test_runner_path(log, &self.crate_path)
            .expect("if installing wasm-bindgen succeeded, then we should have wasm-bindgen-test-runner too"));

        info!(&log, "Getting wasm-bindgen-cli was successful.");
        Ok(())
    }

    fn step_test_node(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        assert!(self.node);
        info!(log, "Running tests in node...");
        PBAR.step(step, "Running tests in node...");
        test::cargo_test_wasm(
            &self.crate_path,
            self.release,
            log,
            Some((
                "CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER",
                &self.test_runner_path.as_ref().unwrap(),
            )),
        )?;
        info!(log, "Finished running tests in node.");
        Ok(())
    }

    fn step_get_chromedriver(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        PBAR.step(step, "Getting chromedriver...");
        assert!(self.chrome && self.chromedriver.is_none());

        self.chromedriver = Some(webdriver::get_or_install_chromedriver(
            log,
            &self.crate_path,
            self.mode,
        )?);
        Ok(())
    }

    fn step_test_chrome(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        PBAR.step(step, "Running tests in Chrome...");

        let chromedriver = self.chromedriver.as_ref().unwrap().display().to_string();
        let chromedriver = chromedriver.as_str();
        info!(
            log,
            "Running tests in Chrome with chromedriver at {}", chromedriver
        );

        let test_runner = self
            .test_runner_path
            .as_ref()
            .unwrap()
            .display()
            .to_string();
        let test_runner = test_runner.as_str();
        info!(log, "Using wasm-bindgen test runner at {}", test_runner);

        let mut envs = vec![
            ("CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER", test_runner),
            ("CHROMEDRIVER", chromedriver),
        ];
        if !self.headless {
            envs.push(("NO_HEADLESS", "1"));
        }

        test::cargo_test_wasm(&self.crate_path, self.release, log, envs)
    }

    fn step_get_geckodriver(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        PBAR.step(step, "Getting geckodriver...");
        assert!(self.firefox && self.geckodriver.is_none());

        self.geckodriver = Some(webdriver::get_or_install_geckodriver(
            log,
            &self.crate_path,
            self.mode,
        )?);
        Ok(())
    }

    fn step_test_firefox(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        PBAR.step(step, "Running tests in Firefox...");

        let geckodriver = self.geckodriver.as_ref().unwrap().display().to_string();
        let geckodriver = geckodriver.as_str();
        info!(
            log,
            "Running tests in Firefox with geckodriver at {}", geckodriver
        );

        let test_runner = self
            .test_runner_path
            .as_ref()
            .unwrap()
            .display()
            .to_string();
        let test_runner = test_runner.as_str();
        info!(log, "Using wasm-bindgen test runner at {}", test_runner);

        let mut envs = vec![
            ("CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER", test_runner),
            ("GECKODRIVER", geckodriver),
        ];
        if !self.headless {
            envs.push(("NO_HEADLESS", "1"));
        }

        test::cargo_test_wasm(&self.crate_path, self.release, log, envs)
    }

    fn step_get_safaridriver(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        PBAR.step(step, "Getting safaridriver...");
        assert!(self.safari && self.safaridriver.is_none());

        self.safaridriver = Some(webdriver::get_safaridriver(log, &self.crate_path)?);
        Ok(())
    }

    fn step_test_safari(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        PBAR.step(step, "Running tests in Safari...");

        let safaridriver = self.safaridriver.as_ref().unwrap().display().to_string();
        let safaridriver = safaridriver.as_str();
        info!(
            log,
            "Running tests in Safari with safaridriver at {}", safaridriver
        );

        let test_runner = self
            .test_runner_path
            .as_ref()
            .unwrap()
            .display()
            .to_string();
        let test_runner = test_runner.as_str();
        info!(log, "Using wasm-bindgen test runner at {}", test_runner);

        let mut envs = vec![
            ("CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER", test_runner),
            ("SAFARIDRIVER", safaridriver),
        ];
        if !self.headless {
            envs.push(("NO_HEADLESS", "1"));
        }

        test::cargo_test_wasm(&self.crate_path, self.release, log, envs)
    }
}
