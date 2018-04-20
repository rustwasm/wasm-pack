use error::Error;
use std::process::Command;

pub fn npm_pack(path: &str) -> Result<(), Error> {
    let pkg_file_path = format!("{}/pkg", path);
    let output = Command::new("npm")
        .current_dir(pkg_file_path)
        .arg("pack")
        .output()?;
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        Error::cli("Packaging up your code failed", s)
    } else {
        Ok(())
    }
}

pub fn npm_publish(path: &str) -> Result<(), Error> {
    let pkg_file_path = format!("{}/pkg", path);
    let output = Command::new("npm")
        .current_dir(pkg_file_path)
        .arg("publish")
        .output()?;
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        Error::cli("Publishing to npm failed", s)
    } else {
        Ok(())
    }
}

pub fn npm_adduser(
    registry: Option<String>,
    scope: Option<String>,
    always_auth: bool,
    auth_type: Option<String>,
) -> Result<(), Error> {
    let status = Command::new("npm")
        .arg("adduser")
        .arg(if let Some(registry) = registry {
            format!("--registry {}", registry)
        } else {
            String::from("")
        })
        .arg(if let Some(scope) = scope {
            format!("--scope {}", scope)
        } else {
            String::from("")
        })
        .arg(if always_auth == true {
            String::from("--always_auth")
        } else {
            String::from("")
        })
        .arg(if let Some(auth_type) = auth_type {
            format!("--auth_type {}", auth_type)
        } else {
            String::from("")
        })
        .status()?;

    if !status.success() {
        bail!("Adding registry user account failed");
    } else {
        Ok(())
    }
}
