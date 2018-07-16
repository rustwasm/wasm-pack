use std::path::{Path, PathBuf};

use copy_dir::copy_dir;
use tempfile;

/// Copy the given fixture into a unique temporary directory. This allows the
/// test to mutate the copied fixture without messing up other tests that are
/// also trying to read from or write to that fixture. The given path should be
/// relative from the root of the repository, eg
/// "tests/fixtures/im-from-brooklyn-the-place-where-stars-are-born".
pub fn fixture<P>(fixture: P) -> Fixture
where
    P: AsRef<Path>,
{
    let fixture = fixture
        .as_ref()
        .canonicalize()
        .expect("should canonicalize fixture path OK");
    let dir = tempfile::tempdir().expect("should create temporary directory OK");
    let path = dir.path().join("wasm-pack");
    println!(
        "wasm-pack: copying test fixture '{}' to temporary directory '{}'",
        fixture.display(),
        path.display()
    );
    copy_dir(fixture, &path).expect("should copy fixture directory into temporary directory OK");
    Fixture { dir, path }
}

pub struct Fixture {
    pub dir: tempfile::TempDir,
    pub path: PathBuf,
}

pub mod manifest {
    use std::fs::File;
    use std::io::prelude::*;
    use std::path::Path;

    use failure::Error;
    use serde_json;

    #[derive(Deserialize)]
    pub struct NpmPackage {
        pub name: String,
        pub description: String,
        pub version: String,
        pub license: String,
        pub repository: Repository,
        pub files: Vec<String>,
        pub main: String,
        pub types: Option<String>,
    }

    #[derive(Deserialize)]
    pub struct Repository {
        #[serde(rename = "type")]
        pub ty: String,
        pub url: String,
    }

    pub fn read_package_json(path: &Path) -> Result<NpmPackage, Error> {
        let manifest_path = path.join("pkg").join("package.json");
        let mut pkg_file = File::open(manifest_path)?;
        let mut pkg_contents = String::new();
        pkg_file.read_to_string(&mut pkg_contents)?;

        Ok(serde_json::from_str(&pkg_contents)?)
    }
}

pub mod readme {
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;

    use failure::Error;

    pub fn read_file(path: &Path) -> Result<String, Error> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        Ok(contents)
    }
}
