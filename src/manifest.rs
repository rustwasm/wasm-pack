use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use console::style;
use emoji;
use failure::{Error, ResultExt};
use parity_wasm;
use parity_wasm::elements::*;
use serde_json;
use toml;
use PBAR;

#[derive(Deserialize)]
struct CargoManifest {
    package: CargoPackage,
}

#[derive(Deserialize)]
struct CargoPackage {
    name: String,
    authors: Vec<String>,
    description: Option<String>,
    version: String,
    license: Option<String>,
    repository: Option<String>,
}

#[derive(Serialize)]
struct NpmPackage {
    name: String,
    collaborators: Vec<String>,
    description: Option<String>,
    version: String,
    license: Option<String>,
    repository: Option<Repository>,
    files: Vec<String>,
    main: String,
    dependencies: Option<BTreeMap<String, String>>,
}

#[derive(Serialize)]
struct Repository {
    #[serde(rename = "type")]
    ty: String,
    url: String,
}

fn read_cargo_toml(path: &Path) -> Result<CargoManifest, Error> {
    let mut cargo_file = File::open(path.join("Cargo.toml"))?;
    let mut cargo_contents = String::new();
    cargo_file.read_to_string(&mut cargo_contents)?;

    Ok(toml::from_str(&cargo_contents)?)
}

/// Locates the `__wasm_pack_unstable` module section inside the wasm file
/// specified, parsing it and returning dependencies found.
///
/// Crates compiled with `wasm-bindgen` can declare dependencies on NPM packages
/// in their code and this is communicated to us, `wasm-pack`, via a custom
/// section in the final binary.
fn read_npm_dependencies(wasm: &Path) -> Result<BTreeMap<String, String>, Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Schema<'a> {
        V1 {
            version: &'a str,
            modules: Vec<(String, String)>,
        },
        Unknown {
            version: &'a str,
        },
    }

    let mut module = parity_wasm::deserialize_file(wasm)
        .with_context(|_| format!("failed to parse `{}` as wasm", wasm.display()))?;
    let wasm_pack_module;
    let deps = {
        let result = module
            .sections()
            .iter()
            .enumerate()
            .filter_map(|(i, s)| match *s {
                Section::Custom(ref cs) => Some((i, cs)),
                _ => None,
            })
            .find(|&(_i, section)| section.name() == "__wasm_pack_unstable");
        let data = match result {
            Some((i, section)) => {
                wasm_pack_module = i;
                section.payload()
            }
            None => return Ok(BTreeMap::new()),
        };
        let schema = serde_json::from_slice(data).with_context(|_| {
            "the wasm file emitted by `wasm-bindgen` contains a \
             `__wasm_pack_unstable` section which should describe \
             js dependencies, but it's in a format that this \
             `wasm-pack` tool does not understand; does `wasm-pack` \
             need to be updated?"
        })?;
        let modules = match schema {
            Schema::V1 {
                version,
                ref modules,
            } if version == "0.0.1" =>
            {
                modules.clone()
            }
            Schema::Unknown { version } | Schema::V1 { version, .. } => bail!(
                "the wasm file emitted by `wasm-bindgen` contains a \
                 `__wasm_pack_unstable` section which should describe \
                 js dependencies, but it's schema version is `{}` \
                 while this `wasm-pack` tool only understands the \
                 schema version 0.0.1; does `wasm-pack` need to be updated?",
                version
            ),
        };

        modules.into_iter().collect()
    };

    // Delete the `__wasm_pack_unstable` custom section and rewrite the wasm
    // file that we're emitting.
    module.sections_mut().remove(wasm_pack_module);
    parity_wasm::serialize_to_file(wasm, module)
        .with_context(|_| format!("failed to write wasm to `{}`", wasm.display()))?;

    Ok(deps)
}

impl CargoManifest {
    fn into_npm(mut self, pkg: &Path, scope: Option<String>) -> Result<NpmPackage, Error> {
        let filename = self.package.name.replace("-", "_");
        let wasm_file = format!("{}_bg.wasm", filename);
        let js_file = format!("{}.js", filename);
        let dependencies = read_npm_dependencies(&pkg.join(&wasm_file))?;
        if let Some(s) = scope {
            self.package.name = format!("@{}/{}", s, self.package.name);
        }
        Ok(NpmPackage {
            name: self.package.name,
            collaborators: self.package.authors,
            description: self.package.description,
            version: self.package.version,
            license: self.package.license,
            repository: self.package.repository.map(|repo_url| Repository {
                ty: "git".to_string(),
                url: repo_url,
            }),
            files: vec![wasm_file],
            main: js_file,
            dependencies: if dependencies.len() == 0 {
                None
            } else {
                Some(dependencies)
            },
        })
    }
}

/// Generate a package.json file inside in `./pkg`.
pub fn write_package_json(path: &str, scope: Option<String>) -> Result<(), Error> {
    let path = Path::new(path);
    let step = format!(
        "{} {}Writing a package.json...",
        style("[4/7]").bold().dim(),
        emoji::MEMO
    );

    let warn_fmt = |field| {
        format!(
            "Field {} is missing from Cargo.toml. It is not necessary, but recommended",
            field
        )
    };

    let pb = PBAR.message(&step);
    let pkg_file_path = path.join("pkg/package.json");
    let mut pkg_file = File::create(pkg_file_path)?;
    let crate_data = read_cargo_toml(path)?;
    let npm_data = crate_data.into_npm(&path.join("pkg"), scope)?;

    if npm_data.description.is_none() {
        PBAR.warn(&warn_fmt("description"));
    }
    if npm_data.repository.is_none() {
        PBAR.warn(&warn_fmt("repository"));
    }
    if npm_data.license.is_none() {
        PBAR.warn(&warn_fmt("license"));
    }

    let npm_json = serde_json::to_string_pretty(&npm_data)?;
    pkg_file.write_all(npm_json.as_bytes())?;
    pb.finish();
    Ok(())
}

pub fn get_crate_name(path: &str) -> Result<String, Error> {
    Ok(read_cargo_toml(Path::new(path))?.package.name)
}
