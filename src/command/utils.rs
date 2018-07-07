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
        for entry in entries {
            if entry.is_ok() {
                let p = entry.unwrap().path();
                if is_pkg_directory(&p) {
                    return Some(p);
                }
            }
        }
        None
    })
}

fn is_pkg_directory(path: &Path) -> bool {
    path.exists() && path.is_dir() && path.ends_with("pkg")
}
