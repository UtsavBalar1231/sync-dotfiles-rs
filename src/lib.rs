pub use anyhow::{Context, Result};
pub use ron::{
    extensions::Extensions,
    ser::{to_string_pretty, PrettyConfig},
    Options,
};
pub use serde::{Deserialize, Serialize};
pub use std::{fs::File, io::Read};
pub use std::{path::PathBuf, str::FromStr};

pub mod config;
pub mod dotconfig;

use crypto_hash::{hex_digest, Algorithm};
use home::home_dir;

/// Hashes the contents of a file and returns the hash as a string
fn hash_file(bytes: &[u8]) -> String {
    hex_digest(Algorithm::SHA256, bytes)
}

/// Fix the path to be absolute and not relative
pub trait FixPath<T> {
    fn fix_path(&self) -> Result<PathBuf>;
}

impl FixPath<PathBuf> for PathBuf {
    fn fix_path(&self) -> Result<PathBuf> {
        if self.is_relative() {
            return Ok(self
                .strip_prefix("~/")
                .map(|p| home_dir().unwrap().join(p))?);
        } else {
            Ok(self.clone())
        }
    }
}

impl FixPath<String> for String {
    fn fix_path(&self) -> Result<PathBuf> {
        if self.starts_with('~') {
            self.replace('~', home_dir().unwrap().to_str().unwrap())
                .fix_path()
        } else {
            Ok(PathBuf::from_str(self).expect("Failed to parse path"))
        }
    }
}
