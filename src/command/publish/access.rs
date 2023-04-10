use anyhow::Result;
use std::ffi::{OsStr, OsString};

/// Represents access level for the to-be publish package. Passed to `wasm-pack publish` as a flag, e.g. `--access=public`.
#[derive(Debug)]
pub enum Access {
    /// Access is granted to all. All unscoped packages *must* be public.
    Public,
    /// Access is restricted, granted via npm permissions. Must be a scoped package.
    Restricted,
}

impl Access {
    /// Returns the mode's name
    pub fn name(&self) -> &'static str {
        match self {
            Access::Public => "--access=public",
            Access::Restricted => "--access=restricted",
        }
    }
}

impl TryFrom<&OsStr> for Access {
    type Error = OsString;

    fn try_from(s: &OsStr) -> Result<Self, OsString> {
        if s == "public" {
            Ok(Access::Public)
        } else if s == "restricted" {
            Ok(Access::Restricted)
        } else if s == "private" {
            Ok(Access::Restricted)
        } else {
            let mut err = OsString::from(s);
            err.push(" is not a supported access level. See https://docs.npmjs.com/cli/access for more information on npm package access levels.");
            Err(err)
        }
    }
}
