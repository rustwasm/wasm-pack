//! Key-value store in `*.stamps` file.

use anyhow::{anyhow, Context, Result};
use std::{env, fs, path::PathBuf};

/// Get a value corresponding to the key from the JSON value.
///
/// You should use return value of function `read_stamps_file_to_json()` as `json` argument.
pub fn get_stamp_value(key: impl AsRef<str>, json: &serde_json::Value) -> Result<String> {
    json.get(key.as_ref())
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .ok_or_else(|| anyhow!("cannot get stamp value for key '{}'", key.as_ref()))
}

/// Save the key-value pair to the store.
pub fn save_stamp_value(key: impl Into<String>, value: impl AsRef<str>) -> Result<()> {
    let mut json = read_stamps_file_to_json().unwrap_or_else(|_| serde_json::Map::new().into());

    {
        let stamps = json
            .as_object_mut()
            .ok_or_else(|| anyhow!("stamps file doesn't contain JSON object"))?;
        stamps.insert(key.into(), value.as_ref().into());
    }

    write_to_stamps_file(json)
}

/// Get the path of the `*.stamps` file that is used as the store.
pub fn get_stamps_file_path() -> Result<PathBuf> {
    let path = env::current_exe()
        .map(|path| path.with_extension("stamps"))
        .context("cannot get stamps file path")?;
    Ok(path)
}

/// Read `*.stamps` file and convert its content to the JSON value.
pub fn read_stamps_file_to_json() -> Result<serde_json::Value> {
    let stamps_file_path = get_stamps_file_path()?;
    let stamps_file_content =
        fs::read_to_string(stamps_file_path).context("cannot find or read stamps file")?;
    let json: serde_json::Value = serde_json::from_str(&stamps_file_content)
        .context("stamps file doesn't contain valid JSON")?;
    Ok(json)
}

fn write_to_stamps_file(json: serde_json::Value) -> Result<()> {
    let stamps_file_path = get_stamps_file_path()?;
    let pretty_json = serde_json::to_string_pretty(&json).context("JSON serialization failed")?;
    fs::write(stamps_file_path, pretty_json).context("cannot write to stamps file")?;
    Ok(())
}
