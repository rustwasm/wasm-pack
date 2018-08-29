use std::path::{Path, PathBuf};

use copy_dir::copy_dir;
use tempfile;

pub struct Fixture {
    pub dir: tempfile::TempDir,
    pub path: PathBuf,
}

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

    {
        // Copying too many things in parallel totally kills my machine(??!!?!),
        // so make sure we are only doing one `copy_dir` at a time...
        use std::sync::Mutex;
        lazy_static! {
            static ref ONE_AT_A_TIME: Mutex<()> = Mutex::new(());
        }
        let _locked = ONE_AT_A_TIME.lock();

        copy_dir(fixture, &path)
            .expect("should copy fixture directory into temporary directory OK");
    }

    Fixture { dir, path }
}
