//! Functionality related to installing prebuilt binaries and/or running cargo install.

use self::krate::Krate;
use binary_install::{Cache, Download};
use child;
use emoji;
use failure::{self, ResultExt};
use log::debug;
use log::{info, warn};
use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use target;
use which::which;
use PBAR;

mod krate;
mod mode;
pub use self::mode::InstallMode;

/// Third-party tools
pub struct Tool {
    /// tool kind
    pub kind: Kind,
    /// tool version
    pub version: String,
}

/// Represents the set of CLI tools wasm-pack uses
#[derive(Clone, Copy)]
pub enum Kind {
    /// cargo-generate
    CargoGenerate,
    /// wasm-bindgen
    WasmBindgen,
    /// wasm-opt
    WasmOpt,
    /// wasm-dis
    WasmDis,
}

impl Tool {
    /// Create a tool
    pub fn new(kind: Kind, version: String) -> Self {
        Self { kind, version }
    }

    /// Execute the CLI
    pub fn run(
        &self,
        cache: &Cache,
        install_permitted: bool,
        execute: impl FnOnce(&Path) -> Result<(), failure::Error>,
    ) -> Result<(), failure::Error> {
        let exec = match self.install(cache, install_permitted)? {
            Status::Found(path) => path,
            Status::CannotInstall => {
                PBAR.info(&format!(
                    "Skipping {} as no downloading was requested",
                    self.kind
                ));
                return Ok(());
            }
            Status::PlatformNotSupported => {
                PBAR.info(&format!(
                    "Skipping {} because it is not supported on this platform",
                    self.kind
                ));
                return Ok(());
            }
        };

        let exec_path = exec.binary(&self.kind.to_string())?;
        PBAR.info(&format!("Executing `{}`...", self.kind));
        execute(&exec_path)?;

        Ok(())
    }

    /// Attempts to find CLIs in `PATH` locally, or failing that downloads a
    /// precompiled binary.
    ///
    /// Returns `Ok` if a binary was found or it was successfully downloaded.
    /// Returns `Err` if a binary wasn't found in `PATH` and this platform doesn't
    /// have precompiled binaries. Returns an error if we failed to download the
    /// binary.
    pub fn install(
        &self,
        cache: &Cache,
        install_permitted: bool,
    ) -> Result<Status, failure::Error> {
        let status =
            download_prebuilt_or_cargo_install(self.kind, cache, &self.version, install_permitted)?;
        let msg = format!("`{}` installed successfully", self.kind);
        PBAR.info(&msg);
        Ok(status)
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::CargoGenerate => "cargo-generate",
            Self::WasmBindgen => "wasm-bindgen",
            Self::WasmOpt => "wasm-opt",
            Self::WasmDis => "wasm-dis",
        };
        write!(f, "{}", s)
    }
}

impl Kind {
    fn is_from_binaryen(self) -> bool {
        use self::Kind::*;
        match self {
            WasmOpt | WasmDis => true,
            _ => false,
        }
    }
}

/// Possible outcomes of attempting to find/install a tool
pub enum Status {
    /// Couldn't install tool because downloads are forbidden by user
    CannotInstall,
    /// The current platform doesn't support precompiled binaries for this tool
    PlatformNotSupported,
    /// We found the tool at the specified path
    Found(Download),
}

/// Install a cargo CLI tool
///
/// Prefers an existing local install, if any exists. Then checks if there is a
/// global install on `$PATH` that fits the bill. Then attempts to download a
/// tarball from the GitHub releases page, if this target has prebuilt
/// binaries. Finally, falls back to `cargo install`.
fn download_prebuilt_or_cargo_install(
    tool: Kind,
    cache: &Cache,
    version: &str,
    install_permitted: bool,
) -> Result<Status, failure::Error> {
    // If the tool is installed globally and it has the right version, use
    // that. Assume that other tools are installed next to it.
    //
    // This situation can arise if the tool is already installed via
    // `cargo install`, for example.
    if let Ok(path) = which(tool.to_string()) {
        debug!("found global {} binary at: {}", tool, path.display());
        if check_version(tool, &path, version)? {
            let download = Download::at(path.parent().unwrap());
            return Ok(Status::Found(download));
        }
    }

    let msg = format!("{}Installing {}...", emoji::DOWN_ARROW, tool);
    PBAR.info(&msg);

    let dl = download_prebuilt(tool, &cache, version, install_permitted);
    match dl {
        Ok(dl) => return Ok(dl),
        Err(e) => {
            if tool.is_from_binaryen() {
                bail!("Could not install {} with {}", tool, version);
            }
            warn!(
                "could not download pre-built `{}`: {}. Falling back to `cargo install`.",
                tool, e
            );
        }
    }

    cargo_install(tool, &cache, version, install_permitted)
}

/// Check if the tool dependency is locally satisfied.
pub fn check_version(
    tool: Kind,
    path: &PathBuf,
    expected_version: &str,
) -> Result<bool, failure::Error> {
    let expected_version = if expected_version == "latest" {
        let krate = Krate::new(tool)?;
        krate.max_version
    } else {
        expected_version.to_string()
    };

    let v = get_cli_version(tool, path)?;
    info!(
        "Checking installed `{}` version == expected version: {} == {}",
        tool, v, &expected_version
    );
    Ok(v == expected_version)
}

/// Fetches the version of a CLI tool
pub fn get_cli_version(tool: Kind, path: &Path) -> Result<String, failure::Error> {
    let mut cmd = Command::new(path);
    cmd.arg("--version");
    let stdout = child::run_capture_stdout(cmd, &tool.to_string())?;
    let version = stdout.trim().split_whitespace().nth(1);
    match version {
        Some(v) => Ok(v.to_string()),
        None => bail!("Something went wrong! We couldn't determine your version of the wasm-bindgen CLI. We were supposed to set that up for you, so it's likely not your fault! You should file an issue: https://github.com/rustwasm/wasm-pack/issues/new?template=bug_report.md.")
    }
}

/// Downloads a precompiled copy of the tool, if available.
pub fn download_prebuilt(
    tool: Kind,
    cache: &Cache,
    version: &str,
    install_permitted: bool,
) -> Result<Status, failure::Error> {
    use self::Kind::*;
    let url = match prebuilt_url(tool, version) {
        Ok(url) => url,
        Err(e) => bail!(
            "no prebuilt {} binaries are available for this platform: {}",
            tool,
            e,
        ),
    };
    match tool {
        WasmBindgen => {
            let binaries = &["wasm-bindgen", "wasm-bindgen-test-runner"];
            match cache.download(install_permitted, "wasm-bindgen", binaries, &url)? {
                Some(download) => Ok(Status::Found(download)),
                None => bail!("wasm-bindgen v{} is not installed!", version),
            }
        }
        CargoGenerate => {
            let binaries = &["cargo-generate"];
            match cache.download(install_permitted, "cargo-generate", binaries, &url)? {
                Some(download) => Ok(Status::Found(download)),
                None => bail!("cargo-generate v{} is not installed!", version),
            }
        }
        WasmOpt => {
            let binaries = &["wasm-opt"];
            match cache.download(install_permitted, "wasm-opt", binaries, &url)? {
                Some(download) => Ok(Status::Found(download)),
                // TODO(ag_dubs): why is this different? i forget...
                None => Ok(Status::CannotInstall),
            }
        }
        WasmDis => {
            let binaries = &["wasm-dis"];
            match cache.download(install_permitted, "wasm-dis", binaries, &url)? {
                Some(dl) => Ok(Status::Found(dl)),
                None => bail!("{} version({}) is not installed!", tool, version),
            }
        }
    }
}

/// Returns the URL of a precompiled version of wasm-bindgen, if we have one
/// available for our host platform.
fn prebuilt_url(tool: Kind, version: &str) -> Result<String, failure::Error> {
    let target = if target::LINUX && target::x86_64 {
        match tool {
            Kind::WasmOpt | Kind::WasmDis => "x86-linux",
            _ => "x86_64-unknown-linux-musl",
        }
    } else if target::LINUX && target::x86 {
        match tool {
            Kind::WasmOpt => "x86-linux",
            _ => bail!("Unrecognized target!"),
        }
    } else if target::MACOS && target::x86_64 {
        "x86_64-apple-darwin"
    } else if target::WINDOWS && target::x86_64 {
        match tool {
            Kind::WasmOpt | Kind::WasmDis => "x86-windows",
            _ => "x86_64-pc-windows-msvc",
        }
    } else if target::WINDOWS && target::x86 {
        match tool {
            Kind::WasmOpt | Kind::WasmDis => "x86-windows",
            _ => bail!("Unrecognized target!"),
        }
    } else {
        bail!("Unrecognized target!")
    };

    match tool {
        Kind::WasmBindgen => {
            Ok(format!(
                "https://github.com/rustwasm/wasm-bindgen/releases/download/{0}/wasm-bindgen-{0}-{1}.tar.gz",
                version,
                target
            ))
        },
        Kind::CargoGenerate => {
            Ok(format!(
                "https://github.com/ashleygwilliams/cargo-generate/releases/download/v{0}/cargo-generate-v{0}-{1}.tar.gz",
                Krate::new(Kind::CargoGenerate)?.max_version,
                target
            ))
        },
        Kind::WasmOpt | Kind::WasmDis => {
            Ok(format!(
        "https://github.com/WebAssembly/binaryen/releases/download/{vers}/binaryen-{vers}-{target}.tar.gz",
        vers = version,
        target = target,
            ))
        }
    }
}

/// Use `cargo install` to install the tool locally into the given
/// crate.
pub fn cargo_install(
    tool: Kind,
    cache: &Cache,
    version: &str,
    install_permitted: bool,
) -> Result<Status, failure::Error> {
    debug!(
        "Attempting to use a `cargo install`ed version of `{}={}`",
        tool, version,
    );

    let dirname = format!("{}-cargo-install-{}", tool, version);
    let destination = cache.join(dirname.as_ref());
    if destination.exists() {
        debug!(
            "`cargo install`ed `{}={}` already exists at {}",
            tool,
            version,
            destination.display()
        );
        let download = Download::at(&destination);
        return Ok(Status::Found(download));
    }

    if !install_permitted {
        return Ok(Status::CannotInstall);
    }

    // Run `cargo install` to a temporary location to handle ctrl-c gracefully
    // and ensure we don't accidentally use stale files in the future
    let tmp = cache.join(format!(".{}", dirname).as_ref());
    drop(fs::remove_dir_all(&tmp));
    debug!("cargo installing {} to tempdir: {}", tool, tmp.display(),);

    let context = format!("failed to create temp dir for `cargo install {}`", tool);
    fs::create_dir_all(&tmp).context(context)?;

    let crate_name = match tool {
        Kind::WasmBindgen => "wasm-bindgen-cli".to_string(),
        _ => tool.to_string(),
    };
    let mut cmd = Command::new("cargo");
    cmd.arg("install")
        .arg("--force")
        .arg(crate_name)
        .arg("--version")
        .arg(version)
        .arg("--root")
        .arg(&tmp);

    let context = format!("Installing {} with cargo", tool);
    child::run(cmd, "cargo install").context(context)?;

    // `cargo install` will put the installed binaries in `$root/bin/*`, but we
    // just want them in `$root/*` directly (which matches how the tarballs are
    // laid out, and where the rest of our code expects them to be). So we do a
    // little renaming here.
    let binaries: Result<Vec<&str>, failure::Error> = match tool {
        Kind::WasmBindgen => Ok(vec!["wasm-bindgen", "wasm-bindgen-test-runner"]),
        Kind::CargoGenerate => Ok(vec!["cargo-genrate"]),
        Kind::WasmOpt | Kind::WasmDis => bail!("Cannot install with cargo."),
    };

    for b in binaries?.iter().cloned() {
        let from = tmp
            .join("bin")
            .join(b)
            .with_extension(env::consts::EXE_EXTENSION);
        let to = tmp.join(from.file_name().unwrap());
        fs::rename(&from, &to).with_context(|_| {
            format!(
                "failed to move {} to {} for `cargo install`ed `{}`",
                from.display(),
                to.display(),
                b
            )
        })?;
    }

    // Finally, move the `tmp` directory into our binary cache.
    fs::rename(&tmp, &destination)?;

    let download = Download::at(&destination);
    Ok(Status::Found(download))
}
