use std::path::{Path, PathBuf};

pub fn set_crate_path(path: Option<String>) -> String {
    let crate_path = match path {
        Some(p) => p,
        None => ".".to_string(),
    };

    crate_path
}

pub fn find_pkg_directory(guess_path: &str) -> Option<PathBuf> {
    let path = PathBuf::from(guess_path);
    if is_pkg_directory(&path) {
        return Some(path);
    }

    path.read_dir().ok().and_then(|entries| {
        entries
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap().path())
            .find(|x| is_pkg_directory(&x))
    })
}

fn is_pkg_directory(path: &Path) -> bool {
    path.exists() && path.is_dir() && path.ends_with("pkg")
}
