use std::path::Path;

pub fn set_crate_path(path: Option<String>) -> String {
    let crate_path = match path {
        Some(p) => p,
        None => ".".to_string(),
    };

    crate_path
}

pub fn find_pkg_directory(guess_path: &str) -> Option<Box<&Path>> {
    let path = Path::new(guess_path);
    if is_pkg_directory(path) {
        return Some(Box::new(path));
    }

    path.parent().and_then(|v| {
        if is_pkg_directory(v) {
            Some(Box::new(v))
        } else {
            None
        }
    })
}

fn is_pkg_directory(path: &Path) -> bool {
    path.exists() && path.is_dir() && path.ends_with("pkg")
}
