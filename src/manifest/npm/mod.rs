mod commonjs;
mod esmodules;
mod nomodules;
pub mod repository;

pub use self::commonjs::CommonJSPackage;
pub use self::esmodules::ESModulesPackage;
pub use self::nomodules::NoModulesPackage;

use error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Serialize)]
#[serde(untagged)]
pub enum NpmPackage {
    CommonJSPackage(CommonJSPackage),
    ESModulesPackage(ESModulesPackage),
    NoModulesPackage(NoModulesPackage),
}

impl Default for NpmPackage {
    fn default() -> NpmPackage {
        ESModulesPackage {}
    }
}

impl FromStr for NpmPackage {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "nodejs" => Ok(NpmPackage::CommonJSPackage),
            "nomodules" => Ok(NpmPackage::NoModulesPackage),
            _ => Err(Error::Unsupported {
                message: format!("{} is not a supported target module type.", s),
            }),
        }
    }
}

impl fmt::Display for NpmPackage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            CommonJSPackage => "--nodejs",
            NoModulesPackage => "--no-modules",
        };
        write!(f, "{}", printable)
    }
}
