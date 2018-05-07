#[derive(Deserialize)]
pub struct CargoManifest {
    pub package: CargoPackage,
}

impl CargoManifest {
    pub fn get_crate_name(&self) -> String {
        self.package.name.clone()
    }
}

#[derive(Deserialize)]
pub struct CargoPackage {
    pub name: String,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub version: String,
    pub license: Option<String>,
    pub repository: Option<String>,
}
