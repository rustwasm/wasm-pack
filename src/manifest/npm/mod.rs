mod commonjs;
mod es6;
pub mod repository;

pub use self::commonjs::CommonJSPackage;
pub use self::es6::ES6Package;

#[derive(Serialize)]
pub enum NpmPackage {
    CommonJSPackage(CommonJSPackage),
    ES6Package(ES6Package),
}
