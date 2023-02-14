pub use anyhow::{Context, Result};
pub use ron::{
    extensions::Extensions,
    ser::{to_string_pretty, PrettyConfig},
    Options,
};
pub use serde::{Deserialize, Serialize};
pub use std::{fs::File, io::Read};
pub use std::{path::PathBuf, str::FromStr};

/// Crate to hold the config file data
pub mod config;
/// Crate to hold the dotconfig file data
pub mod dotconfig;

use home::home_dir;

/// Fix the path to be absolute and not relative
pub trait FixPath<T> {
    fn fix_path(&self) -> Result<PathBuf>;
}

/// Fix the path to be absolute and not relative for PathBuf type
impl FixPath<PathBuf> for PathBuf {
    fn fix_path(&self) -> Result<PathBuf> {
        if self.starts_with("~") {
            return Ok(self
                .strip_prefix("~")
                .map(|p| home_dir().expect("Failed to get home directory").join(p))?);
        } else {
            Ok(self.clone())
        }
    }
}

/// Fix the path to be absolute and not relative for String type
impl FixPath<String> for String {
    fn fix_path(&self) -> Result<PathBuf> {
        if self.starts_with('~') {
            Ok(PathBuf::from(
                self.replace(
                    '~',
                    home_dir()
                        .expect("Failed to get home directory!")
                        .to_str()
                        .unwrap(),
                ),
            ))
        } else {
            Ok(PathBuf::from_str(self).expect("Failed to parse path"))
        }
    }
}
