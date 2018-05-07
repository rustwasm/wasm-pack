mod cargo;
mod npm;

pub use self::cargo::{CargoManifest, CargoPackage};
pub use self::npm::{NpmPackage, Repository};
