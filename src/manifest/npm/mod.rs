mod commonjs;
mod esmodules;
mod nomodules;
pub mod repository;

pub use self::commonjs::CommonJSPackage;
pub use self::esmodules::ESModulesPackage;
pub use self::nomodules::NoModulesPackage;

#[derive(Serialize)]
#[serde(untagged)]
pub enum NpmPackage {
    CommonJSPackage(CommonJSPackage),
    ESModulesPackage(ESModulesPackage),
    NoModulesPackage(NoModulesPackage),
}
