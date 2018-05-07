pub use super::cargo::{CargoManifest, CargoPackage};

#[derive(Deserialize, Serialize)]
pub struct NpmPackage {
    pub name: String,
    pub collaborators: Vec<String>,
    pub description: Option<String>,
    pub version: String,
    pub license: Option<String>,
    pub repository: Option<Repository>,
    pub files: Vec<String>,
    pub main: String,
}

#[derive(Deserialize, Serialize)]
pub struct Repository {
    #[serde(rename = "type")]
    pub ty: String,
    pub url: String,
}

#[derive(Serialize)]
struct PackageFiles {
    main: String,
    files: Vec<String>,
}

impl NpmPackage {
    /// Create an NpmPackage from a borrowed `CargoManifest` object.
    pub fn new(cargo: &CargoManifest, scope: Option<String>) -> NpmPackage {
        let name = NpmPackage::scoped_name(&cargo.package.name, scope);
        let repository = NpmPackage::get_repo(cargo);
        let PackageFiles { files, main } = NpmPackage::get_filenames(cargo);
        NpmPackage {
            name,
            collaborators: cargo.package.authors.clone(),
            description: cargo.package.description.clone(),
            version: cargo.package.version.clone(),
            license: cargo.package.license.clone(),
            repository,
            files,
            main,
        }
    }

    fn scoped_name(name: &str, scope: Option<String>) -> String {
        match scope {
            Some(s) => format!("@{}/{}", s, name),
            None => name.to_string(),
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn repository_url(&self) -> Option<String> {
        match &self.repository {
            Some(repo) => Some(repo.url.clone()),
            None => None,
        }
    }

    pub fn repository_type(&self) -> Option<String> {
        match &self.repository {
            Some(repo) => Some(repo.ty.clone()),
            None => None,
        }
    }

    pub fn main(&self) -> String {
        self.main.clone()
    }

    pub fn files(&self) -> Vec<String> {
        self.files.clone()
    }

    fn get_repo(cargo: &CargoManifest) -> Option<Repository> {
        cargo.package.repository.clone().map(|repo_url| Repository {
            ty: "git".to_string(),
            url: repo_url,
        })
    }

    fn get_filenames(cargo: &CargoManifest) -> PackageFiles {
        let filename = cargo.package.name.clone().replace("-", "_");
        let js_file = format!("{}.js", filename);
        let wasm_file = format!("{}_bg.wasm", filename);
        PackageFiles {
            main: js_file,
            files: vec![wasm_file],
        }
    }
}
