//! Functionality related to running `wasm-bindgen`.

use binary_install::Download;
use child;
use command::build::{BuildProfile, Target};
use failure::{self, ResultExt};
use install;
use manifest::CrateData;
use semver;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Run the `wasm-bindgen` CLI to generate bindings for the current crate's
/// `.wasm`.
pub fn wasm_bindgen_build(
    data: &CrateData,
    bindgen: &Download,
    out_dir: &Path,
    out_name: &Option<String>,
    disable_dts: bool,
    target: Target,
    profile: BuildProfile,
) -> Result<(), failure::Error> {

    match target {
        Target::All => {

            // Get our base name for our was output
            let mut out_base_name_for_bundler = data.crate_name();
            if let Some(value) = out_name {
                out_base_name_for_bundler = value.to_string();
            }  

            // Bundler
            // NOTE: We only generate our type definitions here if we want them
            let mut out_name_for_bundler = Some(format!("{}_esm", out_base_name_for_bundler));
            run_wasm_bindgen(
                data,
                bindgen,
                out_dir,
                &out_name_for_bundler,
                disable_dts,
                Target::Bundler,
                profile
                )?;

            // Rename our typescript definition files
            if !disable_dts {
                // Get our input filenames
                let typescript_definition_input = format!(
                    "{}/{}_esm.d.ts", 
                    out_dir.to_str().unwrap(), 
                    &out_base_name_for_bundler
                    );
                let typescript_bg_definition_input = format!(
                    "{}/{}_esm_bg.d.ts", 
                    out_dir.to_str().unwrap(), 
                    &out_base_name_for_bundler                    
                    );

                // Get our output file names
                let typescript_definition_output = format!(
                    "{}/{}.d.ts", 
                    out_dir.to_str().unwrap(), 
                    &out_base_name_for_bundler
                    );
                let typescript_bg_definition_output = format!(
                    "{}/{}_bg.d.ts", 
                    out_dir.to_str().unwrap(), 
                    out_base_name_for_bundler
                    );


                // Rename the type definition paths
                if Path::new(&typescript_definition_input).exists() {
                    fs::rename(&typescript_definition_input, &typescript_definition_output)?;
                }
                if Path::new(&typescript_bg_definition_input).exists() {
                    fs::rename(&typescript_bg_definition_input, &typescript_bg_definition_output)?;
                }
            }


            // Web
            let mut out_name_for_web = Some(format!("{}_web", data.crate_name()));
            if let Some(value) = out_name {
                out_name_for_web = Some(format!("{}_web", value));
            }            
            run_wasm_bindgen(
                data,
                bindgen,
                out_dir,
                &out_name_for_web,
                true,
                Target::Web,
                profile
                )?;

            // Nodejs
            let mut out_name_for_nodejs = Some(format!("{}_cjs", data.crate_name()));
            if let Some(value) = out_name {
                out_name_for_nodejs = Some(format!("{}_cjs", value));
            }            
            run_wasm_bindgen(
                data,
                bindgen,
                out_dir,
                &out_name_for_nodejs,
                true,
                Target::Nodejs,
                profile
                )?;

            // NoModules
            let mut out_name_for_nomodules = Some(format!("{}_browser", data.crate_name()));
            if let Some(value) = out_name {
                out_name_for_nomodules = Some(format!("{}_browser", value));
            }            
            run_wasm_bindgen(
                data,
                bindgen,
                out_dir,
                &out_name_for_nomodules,
                true,
                Target::NoModules,
                profile
                )?;
        },
        _ => {
            run_wasm_bindgen(
                    data,
                    bindgen,
                    out_dir,
                    out_name,
                    disable_dts,
                    target,
                    profile
                )?;
        }
    }
    Ok(())
}

fn run_wasm_bindgen(
    data: &CrateData,
    bindgen: &Download,
    out_dir: &Path,
    out_name: &Option<String>,
    disable_dts: bool,
    target: Target,
    profile: BuildProfile,
    ) -> Result<(), failure::Error> {

    let release_or_debug = match profile {
        BuildProfile::Release | BuildProfile::Profiling => "release",
        BuildProfile::Dev => "debug",
    };

    let out_dir = out_dir.to_str().unwrap();

    let wasm_path = data
        .target_directory()
        .join("wasm32-unknown-unknown")
        .join(release_or_debug)
        .join(data.crate_name())
        .with_extension("wasm");

    let dts_arg = if disable_dts {
        "--no-typescript"
    } else {
        "--typescript"
    };
    let bindgen_path = bindgen.binary("wasm-bindgen")?;

    let mut cmd = Command::new(&bindgen_path);
    cmd.arg(&wasm_path)
        .arg("--out-dir")
        .arg(out_dir)
        .arg(dts_arg);

    let target_arg = build_target_arg(target, &bindgen_path)?;
    if supports_dash_dash_target(&bindgen_path)? {
        cmd.arg("--target").arg(target_arg);
    } else {
        cmd.arg(target_arg);
    }

    if let Some(value) = out_name {
        cmd.arg("--out-name").arg(value);
    }

    let profile = data.configured_profile(profile);
    if profile.wasm_bindgen_debug_js_glue() {
        cmd.arg("--debug");
    }
    if !profile.wasm_bindgen_demangle_name_section() {
        cmd.arg("--no-demangle");
    }
    if profile.wasm_bindgen_dwarf_debug_info() {
        cmd.arg("--keep-debug");
    }

    child::run(cmd, "wasm-bindgen").context("Running the wasm-bindgen CLI")?;
    Ok(())
}

/// Check if the `wasm-bindgen` dependency is locally satisfied for the web target
fn supports_web_target(cli_path: &PathBuf) -> Result<bool, failure::Error> {
    let cli_version = semver::Version::parse(&install::get_cli_version(
        &install::Tool::WasmBindgen,
        cli_path,
    )?)?;
    let expected_version = semver::Version::parse("0.2.39")?;
    Ok(cli_version >= expected_version)
}

/// Check if the `wasm-bindgen` dependency is locally satisfied for the --target flag
fn supports_dash_dash_target(cli_path: &PathBuf) -> Result<bool, failure::Error> {
    let cli_version = semver::Version::parse(&install::get_cli_version(
        &install::Tool::WasmBindgen,
        cli_path,
    )?)?;
    let expected_version = semver::Version::parse("0.2.40")?;
    Ok(cli_version >= expected_version)
}

fn build_target_arg(target: Target, cli_path: &PathBuf) -> Result<String, failure::Error> {
    if !supports_dash_dash_target(cli_path)? {
        Ok(build_target_arg_legacy(target, cli_path)?)
    } else {
        Ok(target.to_string())
    }
}

fn build_target_arg_legacy(target: Target, cli_path: &PathBuf) -> Result<String, failure::Error> {
    log::info!("Your version of wasm-bindgen is out of date. You should consider updating your Cargo.toml to a version >= 0.2.40.");
    let target_arg = match target {
        Target::Nodejs => "--nodejs",
        Target::NoModules => "--no-modules",
        Target::Web => {
            if supports_web_target(&cli_path)? {
                "--web"
            } else {
                bail!("Your current version of wasm-bindgen does not support the 'web' target. Please update your project to wasm-bindgen version >= 0.2.39.")
            }
        }
        Target::Bundler => "--browser",
        _ => bail!(format!("The passed target, {}, is not supported", target))
    };
    Ok(target_arg.to_string())
}
