pub use anyhow::{anyhow, Context, Result};
pub use ron::{
    extensions::Extensions,
    ser::{to_string_pretty, PrettyConfig},
    Options,
};
pub use serde::{Deserialize, Serialize};
pub use std::{fs, io::Read};
pub use std::{path::PathBuf, str::FromStr};

/// Crate to hold the config file data
pub mod config;
/// Crate to hold the dotconfig file data
pub mod dotconfig;

use home::home_dir;

/// Fix the path to be absolute and not relative
pub trait FixPath<T> {
    fn fix_path(&self) -> Option<PathBuf>;
}

/// Fix the path to be absolute and not relative for PathBuf type
impl FixPath<PathBuf> for PathBuf {
    fn fix_path(&self) -> Option<PathBuf> {
        if self.starts_with("~") {
            Some(
                self.strip_prefix("~")
                    .map(|p| home_dir().expect("Failed to get home directory").join(p))
                    .expect("Failed to strip prefix"),
            )
        } else {
            None
        }
    }
}

/// Fix the path to be absolute and not relative for String type
impl FixPath<&str> for &str {
    fn fix_path(&self) -> Option<PathBuf> {
        if self.starts_with('~') {
            Some(
                self.replace(
                    '~',
                    home_dir()
                        .expect("Failed to get home directory")
                        .to_str()
                        .unwrap(),
                )
                .into(),
            )
        } else {
            None
        }
    }
}
