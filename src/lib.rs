pub use anyhow::{anyhow, Context, Result};
pub use ron::{
    extensions::Extensions,
    ser::{to_string_pretty, PrettyConfig},
    Options,
};
pub use serde::{Deserialize, Serialize};
pub use std::{fs, io::Read};
pub use std::{path::PathBuf, str::FromStr};

/// Provides support to represent and manipulate the config file data using a structure.
pub mod config;
/// Provides support to store the list of the config files with their path in the config file.
pub mod dotconfig;

use home::home_dir;

/// Fix the path to make sure it is absolute and not relative
pub trait FixPath<T> {
    fn fix_path(&self) -> Option<PathBuf>;
}

impl FixPath<PathBuf> for PathBuf {
    /// Fix the path to be absolute and not relative for PathBuf type
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

impl FixPath<String> for String {
    /// Fix the path to be absolute and not relative for string slice type
    fn fix_path(&self) -> Option<PathBuf> {
        // replace $(pwd) with the current working directory
        if self.starts_with("$(pwd)") {
            return Some(
                self.replace(
                    "$(pwd)",
                    std::env::current_dir()
                        .expect("Failed to get current directory")
                        .to_str()
                        .unwrap(),
                )
                .into(),
            );
        } else if self.starts_with('~') {
            return Some(
                self.replace(
                    '~',
                    home_dir()
                        .expect("Failed to get home directory")
                        .to_str()
                        .unwrap(),
                )
                .into(),
            );
        }
        None
    }
}

impl FixPath<&str> for &str {
    /// Fix the path to be absolute and not relative for string slice type
    fn fix_path(&self) -> Option<PathBuf> {
        // replace $(pwd) with the current working directory
        if self.starts_with("$(pwd)") {
            return Some(
                self.replace(
                    "$(pwd)",
                    std::env::current_dir()
                        .expect("Failed to get current directory")
                        .to_str()
                        .unwrap(),
                )
                .into(),
            );
        } else if self.starts_with('~') {
            return Some(
                self.replace(
                    '~',
                    home_dir()
                        .expect("Failed to get home directory")
                        .to_str()
                        .unwrap(),
                )
                .into(),
            );
        }
        None
    }
}
