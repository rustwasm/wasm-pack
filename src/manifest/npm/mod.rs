mod commonjs;
mod esmodules;
pub mod repository;

pub use self::commonjs::CommonJSPackage;
pub use self::esmodules::ESModulesPackage;

#[derive(Serialize)]
pub enum NpmPackage {
    CommonJSPackage(CommonJSPackage),
    ESModulesPackage(ESModulesPackage),
}
