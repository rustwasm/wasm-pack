//! Functionality related to running `wasm-bindgen`.

use crate::child;
use crate::command::build::{BuildProfile, Target};
use crate::install::{self, Tool};
use crate::manifest::CrateData;
use anyhow::{bail, Context, Result};
use semver;
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::Command;

/// Run the `wasm-bindgen` CLI to generate bindings for the current crate's
/// `.wasm`.
pub fn wasm_bindgen_build(
    data: &CrateData,
    install_status: &install::Status,
    out_dir: &Path,
    out_name: &Option<String>,
    disable_dts: bool,
    weak_refs: bool,
    reference_types: bool,
    target: Target,
    profile: BuildProfile,
    extra_options: &[OsString],
) -> Result<()> {
    let release_or_debug = match profile {
        BuildProfile::Release | BuildProfile::Profiling => "release",
        BuildProfile::Dev => "debug",
    };

    let has_target_dir_overwrite = extra_options.iter().any(|i| i == "--target-dir");
    let target_directory = if has_target_dir_overwrite {
        let i = extra_options
            .binary_search_by(|i| i.as_os_str().cmp(OsStr::new("--target-dir")))
            .unwrap();
        extra_options
            .get(i + 1)
            .map(Path::new)
            .unwrap_or(data.target_directory())
    } else {
        data.target_directory()
    };

    let mut wasm_path = target_directory.join("wasm32-unknown-unknown");
    wasm_path.push(release_or_debug);
    wasm_path.push(data.crate_name());
    wasm_path.set_extension("wasm");

    let dts_arg = if disable_dts {
        "--no-typescript"
    } else {
        "--typescript"
    };
    let bindgen_path = install::get_tool_path(install_status, Tool::WasmBindgen)?
        .binary(Tool::WasmBindgen.name())?;

    let target_arg = build_target_arg(target, &bindgen_path)?;
    //let out_name_args = out_name.map(|value| [OsStr::new("--out-name"), value.as_ref()]).into_iter().flatten();
    let profile = data.configured_profile(profile);

    let mut cmd = Command::new(&bindgen_path);
    cmd.args(
        std::iter::empty::<&OsStr>()
            .chain([
                wasm_path.as_os_str(),
                "--out-dir".as_ref(),
                out_dir.as_os_str(),
                dts_arg.as_ref(),
            ])
            .chain(weak_refs.then_some("--weak-refs".as_ref()))
            .chain(reference_types.then_some("--reference-types".as_ref()))
            .chain(supports_dash_dash_target(&bindgen_path)?.then_some("--target".as_ref()))
            .chain([target_arg.as_ref()])
            .chain(
                out_name
                    .as_ref()
                    .map(|value| ["--out-name".as_ref(), value.as_ref()])
                    .into_iter()
                    .flatten(),
            )
            .chain(
                profile
                    .wasm_bindgen_debug_js_glue()
                    .then_some("--debug".as_ref()),
            )
            .chain(
                (!profile.wasm_bindgen_demangle_name_section()).then_some("--no-demangle".as_ref()),
            )
            .chain(
                profile
                    .wasm_bindgen_dwarf_debug_info()
                    .then_some("--keep-debug".as_ref()),
            ),
    );

    child::run(cmd, "wasm-bindgen").context("Running the wasm-bindgen CLI")?;
    Ok(())
}

/// Check if the `wasm-bindgen` dependency is locally satisfied for the web target
fn supports_web_target(cli_path: &Path) -> Result<bool> {
    let cli_version = semver::Version::parse(&install::get_cli_version(
        &install::Tool::WasmBindgen,
        cli_path,
    )?)?;
    let expected_version = semver::Version::parse("0.2.39")?;
    Ok(cli_version >= expected_version)
}

/// Check if the `wasm-bindgen` dependency is locally satisfied for the --target flag
fn supports_dash_dash_target(cli_path: &Path) -> Result<bool> {
    let cli_version = semver::Version::parse(&install::get_cli_version(
        &install::Tool::WasmBindgen,
        cli_path,
    )?)?;
    let expected_version = semver::Version::parse("0.2.40")?;
    Ok(cli_version >= expected_version)
}

fn build_target_arg(target: Target, cli_path: &Path) -> Result<String> {
    if !supports_dash_dash_target(cli_path)? {
        Ok(build_target_arg_legacy(target, cli_path)?)
    } else {
        Ok(target.to_string())
    }
}

fn build_target_arg_legacy(target: Target, cli_path: &Path) -> Result<String> {
    log::info!("Your version of wasm-bindgen is out of date. You should consider updating your Cargo.toml to a version >= 0.2.40.");
    let target_arg = match target {
        Target::Nodejs => "--nodejs",
        Target::NoModules => "--no-modules",
        Target::Web => {
            if supports_web_target(cli_path)? {
                "--web"
            } else {
                bail!("Your current version of wasm-bindgen does not support the 'web' target. Please update your project to wasm-bindgen version >= 0.2.39.")
            }
        }
        Target::Bundler => "--browser",
        Target::Deno => "--deno",
    };
    Ok(target_arg.to_string())
}
