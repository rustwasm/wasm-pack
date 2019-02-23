//! Reading and writing Cargo.toml and package.json manifests.

#![allow(clippy::new_ret_no_self, clippy::needless_pass_by_value)]

mod npm;

use std::fs;
use std::path::Path;

use self::npm::{
    repository::Repository, CommonJSPackage, ESModulesPackage, NoModulesPackage, NpmPackage,
};
use cargo_metadata::Metadata;
use command::build::BuildProfile;
use emoji;
use failure::{Error, ResultExt};
use progressbar::Step;
use serde::{self, Deserialize};
use serde_json;
use std::collections::BTreeSet;
use strsim::levenshtein;
use toml;
use PBAR;

const WASM_PACK_METADATA_KEY: &str = "package.metadata.wasm-pack";

/// Store for metadata learned about a crate
pub struct CrateData {
    data: Metadata,
    current_idx: usize,
    manifest: CargoManifest,
}

#[doc(hidden)]
#[derive(Deserialize)]
pub struct CargoManifest {
    package: CargoPackage,
}

#[derive(Deserialize)]
struct CargoPackage {
    name: String,
    description: Option<String>,
    license: Option<String>,
    #[serde(rename = "license-file")]
    license_file: Option<String>,
    repository: Option<String>,
    homepage: Option<String>,

    #[serde(default)]
    metadata: CargoMetadata,
}

#[derive(Default, Deserialize)]
struct CargoMetadata {
    #[serde(default, rename = "wasm-pack")]
    wasm_pack: CargoWasmPack,
}

#[derive(Default, Deserialize)]
struct CargoWasmPack {
    #[serde(default)]
    profile: CargoWasmPackProfiles,
}

#[derive(Deserialize)]
struct CargoWasmPackProfiles {
    #[serde(
        default = "CargoWasmPackProfile::default_dev",
        deserialize_with = "CargoWasmPackProfile::deserialize_dev"
    )]
    dev: CargoWasmPackProfile,

    #[serde(
        default = "CargoWasmPackProfile::default_release",
        deserialize_with = "CargoWasmPackProfile::deserialize_release"
    )]
    release: CargoWasmPackProfile,

    #[serde(
        default = "CargoWasmPackProfile::default_profiling",
        deserialize_with = "CargoWasmPackProfile::deserialize_profiling"
    )]
    profiling: CargoWasmPackProfile,
}

impl Default for CargoWasmPackProfiles {
    fn default() -> CargoWasmPackProfiles {
        CargoWasmPackProfiles {
            dev: CargoWasmPackProfile::default_dev(),
            release: CargoWasmPackProfile::default_release(),
            profiling: CargoWasmPackProfile::default_profiling(),
        }
    }
}

/// This is where configuration goes for wasm-bindgen, wasm-opt, wasm-snip, or
/// anything else that wasm-pack runs.
#[derive(Default, Deserialize)]
pub struct CargoWasmPackProfile {
    #[serde(default, rename = "wasm-bindgen")]
    wasm_bindgen: CargoWasmPackProfileWasmBindgen,
}

#[derive(Default, Deserialize)]
struct CargoWasmPackProfileWasmBindgen {
    #[serde(default, rename = "debug-js-glue")]
    debug_js_glue: Option<bool>,

    #[serde(default, rename = "demangle-name-section")]
    demangle_name_section: Option<bool>,

    #[serde(default, rename = "dwarf-debug-info")]
    dwarf_debug_info: Option<bool>,
}

impl CargoWasmPackProfile {
    fn default_dev() -> Self {
        CargoWasmPackProfile {
            wasm_bindgen: CargoWasmPackProfileWasmBindgen {
                debug_js_glue: Some(true),
                demangle_name_section: Some(true),
                dwarf_debug_info: Some(false),
            },
        }
    }

    fn default_release() -> Self {
        CargoWasmPackProfile {
            wasm_bindgen: CargoWasmPackProfileWasmBindgen {
                debug_js_glue: Some(false),
                demangle_name_section: Some(true),
                dwarf_debug_info: Some(false),
            },
        }
    }

    fn default_profiling() -> Self {
        CargoWasmPackProfile {
            wasm_bindgen: CargoWasmPackProfileWasmBindgen {
                debug_js_glue: Some(false),
                demangle_name_section: Some(true),
                dwarf_debug_info: Some(false),
            },
        }
    }

    fn deserialize_dev<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut profile = <Option<Self>>::deserialize(deserializer)?.unwrap_or_default();
        profile.update_with_defaults(&Self::default_dev());
        Ok(profile)
    }

    fn deserialize_release<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut profile = <Option<Self>>::deserialize(deserializer)?.unwrap_or_default();
        profile.update_with_defaults(&Self::default_release());
        Ok(profile)
    }

    fn deserialize_profiling<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut profile = <Option<Self>>::deserialize(deserializer)?.unwrap_or_default();
        profile.update_with_defaults(&Self::default_profiling());
        Ok(profile)
    }

    fn update_with_defaults(&mut self, defaults: &Self) {
        macro_rules! d {
            ( $( $path:ident ).* ) => {
                self. $( $path ).* .get_or_insert(defaults. $( $path ).* .unwrap());
            }
        }
        d!(wasm_bindgen.debug_js_glue);
        d!(wasm_bindgen.demangle_name_section);
        d!(wasm_bindgen.dwarf_debug_info);
    }

    /// Get this profile's configured `[wasm-bindgen.debug-js-glue]` value.
    pub fn wasm_bindgen_debug_js_glue(&self) -> bool {
        self.wasm_bindgen.debug_js_glue.unwrap()
    }

    /// Get this profile's configured `[wasm-bindgen.demangle-name-section]` value.
    pub fn wasm_bindgen_demangle_name_section(&self) -> bool {
        self.wasm_bindgen.demangle_name_section.unwrap()
    }

    /// Get this profile's configured `[wasm-bindgen.dwarf-debug-info]` value.
    pub fn wasm_bindgen_dwarf_debug_info(&self) -> bool {
        self.wasm_bindgen.dwarf_debug_info.unwrap()
    }
}

struct NpmData {
    name: String,
    files: Vec<String>,
    dts_file: Option<String>,
    main: String,
    homepage: Option<String>, // https://docs.npmjs.com/files/package.json#homepage
}

#[doc(hidden)]
pub struct ManifestAndUnsedKeys {
    pub manifest: CargoManifest,
    pub unused_keys: BTreeSet<String>,
}

impl CrateData {
    /// Reads all metadata for the crate whose manifest is inside the directory
    /// specified by `path`.
    pub fn new(crate_path: &Path) -> Result<CrateData, Error> {
        let manifest_path = crate_path.join("Cargo.toml");
        if !manifest_path.is_file() {
            bail!(
                "crate directory is missing a `Cargo.toml` file; is `{}` the \
                 wrong directory?",
                crate_path.display()
            )
        }

        let data =
            cargo_metadata::metadata(Some(&manifest_path)).map_err(error_chain_to_failure)?;

        let manifest_and_keys = CrateData::parse_crate_data(&manifest_path)?;
        CrateData::warn_for_unused_keys(&manifest_and_keys);

        let manifest = manifest_and_keys.manifest;
        let current_idx = data
            .packages
            .iter()
            .position(|pkg| pkg.name == manifest.package.name)
            .ok_or_else(|| format_err!("failed to find package in metadata"))?;

        return Ok(CrateData {
            data,
            manifest,
            current_idx,
        });

        fn error_chain_to_failure(err: cargo_metadata::Error) -> Error {
            let errors = err.iter().collect::<Vec<_>>();
            let mut err: Error = match errors.last() {
                Some(e) => format_err!("{}", e),
                None => return format_err!("{}", err),
            };
            for e in errors[..errors.len() - 1].iter().rev() {
                err = err.context(e.to_string()).into();
            }
            err
        }
    }

    /// Read the `manifest_path` file and deserializes it using the toml Deserializer.
    /// Returns a Result containing `ManifestAndUnsedKeys` which contains `CargoManifest`
    /// and a `BTreeSet<String>` containing the unused keys from the parsed file.
    ///
    /// # Errors
    /// Will return Err if the file (manifest_path) couldn't be read or
    /// if deserialize to `CargoManifest` fails.
    pub fn parse_crate_data(manifest_path: &Path) -> Result<ManifestAndUnsedKeys, Error> {
        let manifest = fs::read_to_string(&manifest_path)
            .with_context(|_| format!("failed to read: {}", manifest_path.display()))?;
        let manifest = &mut toml::Deserializer::new(&manifest);

        let mut unused_keys = BTreeSet::new();
        let levenshtein_threshold = 1;

        let manifest: CargoManifest = serde_ignored::deserialize(manifest, |path| {
            let path_string = path.to_string();

            if path_string.starts_with("package.metadata")
                && (path_string.contains("wasm-pack")
                    || levenshtein(WASM_PACK_METADATA_KEY, &path_string) <= levenshtein_threshold)
            {
                unused_keys.insert(path_string);
            }
        })
        .with_context(|_| format!("failed to parse manifest: {}", manifest_path.display()))?;

        Ok(ManifestAndUnsedKeys {
            manifest,
            unused_keys,
        })
    }

    /// Iterating through all the passed `unused_keys` and output
    /// a warning for each unknown key.
    pub fn warn_for_unused_keys(manifest_and_keys: &ManifestAndUnsedKeys) {
        manifest_and_keys.unused_keys.iter().for_each(|path| {
            PBAR.warn(&format!(
                "\"{}\" is an unknown key and will be ignored. Please check your Cargo.toml.",
                path
            ));
        });
    }

    /// Get the configured profile.
    pub fn configured_profile(&self, profile: BuildProfile) -> &CargoWasmPackProfile {
        match profile {
            BuildProfile::Dev => &self.manifest.package.metadata.wasm_pack.profile.dev,
            BuildProfile::Profiling => &self.manifest.package.metadata.wasm_pack.profile.profiling,
            BuildProfile::Release => &self.manifest.package.metadata.wasm_pack.profile.release,
        }
    }

    /// Check that the crate the given path is properly configured.
    pub fn check_crate_config(&self, step: &Step) -> Result<(), Error> {
        let msg = format!("{}Checking crate configuration...", emoji::WRENCH);
        PBAR.step(&step, &msg);
        self.check_crate_type()?;
        Ok(())
    }

    fn check_crate_type(&self) -> Result<(), Error> {
        let pkg = &self.data.packages[self.current_idx];
        let any_cdylib = pkg
            .targets
            .iter()
            .filter(|target| target.kind.iter().any(|k| k == "cdylib"))
            .any(|target| target.crate_types.iter().any(|s| s == "cdylib"));
        if any_cdylib {
            return Ok(());
        }
        bail!(
            "crate-type must be cdylib to compile to wasm32-unknown-unknown. Add the following to your \
             Cargo.toml file:\n\n\
             [lib]\n\
             crate-type = [\"cdylib\", \"rlib\"]"
        )
    }

    /// Get the crate name for the crate at the given path.
    pub fn crate_name(&self) -> String {
        let pkg = &self.data.packages[self.current_idx];
        match pkg
            .targets
            .iter()
            .find(|t| t.kind.iter().any(|k| k == "cdylib"))
        {
            Some(lib) => lib.name.replace("-", "_"),
            None => pkg.name.replace("-", "_"),
        }
    }

    /// Get the license for the crate at the given path.
    pub fn crate_license(&self) -> &Option<String> {
        &self.manifest.package.license
    }

    /// Get the license file path for the crate at the given path.
    pub fn crate_license_file(&self) -> &Option<String> {
        &self.manifest.package.license_file
    }

    /// Returns the path to this project's target directory where artifacts are
    /// located after a cargo build.
    pub fn target_directory(&self) -> &Path {
        Path::new(&self.data.target_directory)
    }

    /// Returns the path to this project's root cargo workspace directory
    pub fn workspace_root(&self) -> &Path {
        Path::new(&self.data.workspace_root)
    }

    /// Generate a package.json file inside in `./pkg`.
    pub fn write_package_json(
        &self,
        out_dir: &Path,
        scope: &Option<String>,
        disable_dts: bool,
        target: &str,
        step: &Step,
    ) -> Result<(), Error> {
        let msg = format!("{}Writing a package.json...", emoji::MEMO);

        PBAR.step(step, &msg);
        let pkg_file_path = out_dir.join("package.json");
        let npm_data = if target == "nodejs" {
            self.to_commonjs(scope, disable_dts, out_dir)
        } else if target == "no-modules" {
            self.to_nomodules(scope, disable_dts, out_dir)
        } else {
            self.to_esmodules(scope, disable_dts, out_dir)
        };

        let npm_json = serde_json::to_string_pretty(&npm_data)?;
        fs::write(&pkg_file_path, npm_json)
            .with_context(|_| format!("failed to write: {}", pkg_file_path.display()))?;
        Ok(())
    }

    fn npm_data(
        &self,
        scope: &Option<String>,
        include_commonjs_shim: bool,
        disable_dts: bool,
        out_dir: &Path,
    ) -> NpmData {
        let crate_name = self.crate_name();
        let wasm_file = format!("{}_bg.wasm", crate_name);
        let js_file = format!("{}.js", crate_name);
        let mut files = vec![wasm_file];

        files.push(js_file.clone());
        if include_commonjs_shim {
            let js_bg_file = format!("{}_bg.js", crate_name);
            files.push(js_bg_file.to_string());
        }

        let pkg = &self.data.packages[self.current_idx];
        let npm_name = match scope {
            Some(s) => format!("@{}/{}", s, pkg.name),
            None => pkg.name.clone(),
        };

        let dts_file = if !disable_dts {
            let file = format!("{}.d.ts", crate_name);
            files.push(file.to_string());
            Some(file)
        } else {
            None
        };

        let readme_file = out_dir.join("README.md");
        if readme_file.is_file() {
            files.push("README.md".to_string());
        }

        if let Ok(entries) = fs::read_dir(out_dir) {
            let file_names = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.metadata().map(|m| m.is_file()).unwrap_or(false))
                .filter_map(|e| e.file_name().into_string().ok())
                .filter(|f| f.starts_with("LICENSE"));
            for file_name in file_names {
                files.push(file_name);
            }
        }

        NpmData {
            name: npm_name,
            dts_file,
            files,
            main: js_file,
            homepage: self.manifest.package.homepage.clone(),
        }
    }

    fn license(&self) -> Option<String> {
        self.manifest.package.license.clone().or_else(|| {
            self.manifest.package.license_file.clone().map(|file| {
                // When license is written in file: https://docs.npmjs.com/files/package.json#license
                format!("SEE LICENSE IN {}", file)
            })
        })
    }

    fn to_commonjs(&self, scope: &Option<String>, disable_dts: bool, out_dir: &Path) -> NpmPackage {
        let data = self.npm_data(scope, true, disable_dts, out_dir);
        let pkg = &self.data.packages[self.current_idx];

        self.check_optional_fields();

        NpmPackage::CommonJSPackage(CommonJSPackage {
            name: data.name,
            collaborators: pkg.authors.clone(),
            description: self.manifest.package.description.clone(),
            version: pkg.version.clone(),
            license: self.license(),
            repository: self
                .manifest
                .package
                .repository
                .clone()
                .map(|repo_url| Repository {
                    ty: "git".to_string(),
                    url: repo_url,
                }),
            files: data.files,
            main: data.main,
            homepage: data.homepage,
            types: data.dts_file,
        })
    }

    fn to_esmodules(
        &self,
        scope: &Option<String>,
        disable_dts: bool,
        out_dir: &Path,
    ) -> NpmPackage {
        let data = self.npm_data(scope, false, disable_dts, out_dir);
        let pkg = &self.data.packages[self.current_idx];

        self.check_optional_fields();

        NpmPackage::ESModulesPackage(ESModulesPackage {
            name: data.name,
            collaborators: pkg.authors.clone(),
            description: self.manifest.package.description.clone(),
            version: pkg.version.clone(),
            license: self.license(),
            repository: self
                .manifest
                .package
                .repository
                .clone()
                .map(|repo_url| Repository {
                    ty: "git".to_string(),
                    url: repo_url,
                }),
            files: data.files,
            module: data.main,
            homepage: data.homepage,
            types: data.dts_file,
            side_effects: "false".to_string(),
        })
    }

    fn to_nomodules(
        &self,
        scope: &Option<String>,
        disable_dts: bool,
        out_dir: &Path,
    ) -> NpmPackage {
        let data = self.npm_data(scope, false, disable_dts, out_dir);
        let pkg = &self.data.packages[self.current_idx];

        self.check_optional_fields();

        NpmPackage::NoModulesPackage(NoModulesPackage {
            name: data.name,
            collaborators: pkg.authors.clone(),
            description: self.manifest.package.description.clone(),
            version: pkg.version.clone(),
            license: self.license(),
            repository: self
                .manifest
                .package
                .repository
                .clone()
                .map(|repo_url| Repository {
                    ty: "git".to_string(),
                    url: repo_url,
                }),
            files: data.files,
            browser: data.main,
            homepage: data.homepage,
            types: data.dts_file,
        })
    }

    fn check_optional_fields(&self) {
        let mut messages = vec![];
        if self.manifest.package.description.is_none() {
            messages.push("description");
        }
        if self.manifest.package.repository.is_none() {
            messages.push("repository");
        }
        if self.manifest.package.license.is_none() {
            messages.push("license");
        }

        match messages.len() {
            1 => PBAR.info(&format!("Optional field missing from Cargo.toml: '{}'. This is not necessary, but recommended", messages[0])),
            2 => PBAR.info(&format!("Optional fields missing from Cargo.toml: '{}', '{}'. These are not necessary, but recommended", messages[0], messages[1])),
            3 => PBAR.info(&format!("Optional fields missing from Cargo.toml: '{}', '{}', and '{}'. These are not necessary, but recommended", messages[0], messages[1], messages[2])),
            _ => ()
        };
    }
}
