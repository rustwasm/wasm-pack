mod commonjs;
mod esmodules;
pub mod repository;

pub use self::commonjs::CommonJSPackage;
pub use self::esmodules::ESModulesPackage;

#[derive(Serialize)]
#[serde(untagged)]
pub enum NpmPackage {
    CommonJSPackage(CommonJSPackage),
    ESModulesPackage(ESModulesPackage),
}
