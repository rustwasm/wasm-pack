pub mod repository;

use self::repository::Repository;
use super::{CargoManifest, NpmData};

#[derive(Serialize)]
pub struct NpmPackage {
    pub name: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub collaborators: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<Repository>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub types: Option<String>,
    #[serde(rename = "sideEffects")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side_effects: Option<String>,
}

impl NpmPackage {
    pub(super) fn base(
        data: NpmData,
        pkg: &cargo_metadata::Package,
        manifest: &CargoManifest,
    ) -> Self {
        Self {
            name: data.name,
            collaborators: pkg.authors.clone(),
            description: manifest.package.description.clone(),
            version: pkg.version.clone(),
            license: manifest.package.license.clone(),
            repository: manifest
                .package
                .repository
                .clone()
                .map(|repo_url| Repository {
                    ty: "git".to_string(),
                    url: repo_url,
                }),
            files: vec![],
            types: data.dts_file,
            ..Self::default()
        }
    }
}

impl Default for NpmPackage {
    fn default() -> Self {
        Self {
            name: "".to_owned(),
            collaborators: vec![],
            description: None,
            version: "".to_owned(),
            license: None,
            repository: None,
            files: vec![],
            browser: None,
            main: None,
            module: None,
            types: None,
            side_effects: None,
        }
    }
}
