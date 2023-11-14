use anyhow::{bail, Error, Result};
use std::fmt;
use std::str::FromStr;

/// Represents access level for the to-be publish package. Passed to `wasm-pack publish` as a flag, e.g. `--access=public`.
#[derive(Clone, Debug)]
pub enum Access {
    /// Access is granted to all. All unscoped packages *must* be public.
    Public,
    /// Access is restricted, granted via npm permissions. Must be a scoped package.
    Restricted,
}

impl FromStr for Access {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
      "public" => Ok(Access::Public),
      "restricted" => Ok(Access::Restricted),
      "private" => Ok(Access::Restricted),
      _ => bail!("{} is not a supported access level. See https://docs.npmjs.com/cli/access for more information on npm package access levels.", s),
    }
    }
}

impl fmt::Display for Access {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            Access::Public => "--access=public",
            Access::Restricted => "--access=restricted",
        };
        write!(f, "{}", printable)
    }
}
