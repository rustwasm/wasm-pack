//! Reading Cargo.lock lock file.

#![allow(clippy::new_ret_no_self)]

use std::path::PathBuf;
use std::{borrow::Cow, fs};

use crate::manifest::CrateData;
use anyhow::{anyhow, bail, Context, Result};
use console::style;
use serde::{
    de::{DeserializeSeed, Error, IgnoredAny, Visitor},
    Deserializer,
};
use toml;

/// This struct represents a single package entry in `Cargo.lock`
pub struct Package<'a> {
    name: &'a str,
    version: Option<String>,
}

impl<'a> Package<'a> {
    /// Read the `Cargo.lock` file for the crate at the given path and get the versions of the named dependencies.
    pub fn get<const N: usize>(
        crate_data: &CrateData,
        dep_names: [&'a str; N],
    ) -> Result<[Self; N]> {
        let lock_path = get_lockfile_path(crate_data)?;
        let lockfile = fs::read_to_string(&lock_path)
            .with_context(|| anyhow!("failed to read: {}", lock_path.display()))?;
        toml::Deserializer::new(&lockfile)
            .deserialize_struct("Lockfile", &["package"], LockfileVisitor { dep_names })
            .with_context(|| anyhow!("failed to parse: {}", lock_path.display()))
    }

    /// Get the version of this package used in the `Cargo.lock`.
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    /// Like `version`, except it returns an error instead of `None`. `suggested_version` is only used when showing an example of adding the dependency to `Cargo.toml`.
    pub fn require_version_or_suggest(
        &self,
        section: &str,
        suggested_version: &str,
    ) -> Result<&str> {
        self.version().ok_or_else(|| {
            anyhow!(
                "Ensure that you have \"{}\" as a dependency in your Cargo.toml file:\n\
                 [{}]\n\
                 {} = \"{}\"",
                style(self.name).bold().dim(),
                section,
                self.name,
                suggested_version,
            )
        })
    }
}

struct LockfileVisitor<'a, const N: usize> {
    dep_names: [&'a str; N],
}

impl<'de, 'a, const N: usize> Visitor<'de> for LockfileVisitor<'a, N> {
    type Value = [Package<'a>; N];

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct Lockfile")
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Package,
            Other(IgnoredAny),
        }

        while let Some(key) = map.next_key()? {
            match key {
                Field::Package => {
                    return map.next_value_seed(PackagesSeed {
                        dep_names: self.dep_names,
                    })
                }
                Field::Other(_) => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Err(A::Error::missing_field("package"))
    }
}

struct PackagesSeed<'a, const N: usize> {
    dep_names: [&'a str; N],
}

impl<'de, 'a, const N: usize> DeserializeSeed<'de> for PackagesSeed<'a, N> {
    type Value = [Package<'a>; N];

    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, 'a, const N: usize> Visitor<'de> for PackagesSeed<'a, N> {
    type Value = [Package<'a>; N];

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a sequence")
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut packages = self.dep_names.map(|name| Package {
            name,
            version: None,
        });
        while let Some(PackageValue { name, version }) = seq.next_element()? {
            #[cfg(test)]
            assert_eq!(
                (true, true),
                (
                    matches!(name, Cow::Borrowed(_)),
                    matches!(version, Cow::Borrowed(_))
                )
            );
            for package in &mut packages {
                if package.name == name {
                    package.version = Some(version.into_owned());
                    if packages.iter().all(|i| i.version.is_some()) {
                        return Ok(packages);
                    } else {
                        break;
                    }
                }
            }
        }
        Ok(packages)
    }
}

#[derive(Deserialize)]
struct PackageValue<'a> {
    #[serde(borrow)]
    name: Cow<'a, str>,
    #[serde(borrow)]
    version: Cow<'a, str>,
}

/// Given the path to the crate that we are building, return a `PathBuf`
/// containing the location of the lock file, by finding the workspace root.
fn get_lockfile_path(crate_data: &CrateData) -> Result<PathBuf> {
    // Check that a lock file can be found in the directory. Return an error
    // if it cannot, otherwise return the path buffer.
    let lockfile_path = crate_data.workspace_root().join("Cargo.lock");
    if !lockfile_path.is_file() {
        bail!("Could not find lockfile at {:?}", lockfile_path)
    } else {
        Ok(lockfile_path)
    }
}
