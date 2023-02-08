pub use anyhow::{Context, Result};
use crypto_hash::{hex_digest, Algorithm};
pub use ron::{
    extensions::Extensions,
    ser::{to_string_pretty, PrettyConfig},
    Options,
};
use serde::{Deserialize, Serialize};
pub use std::{fs::File, io::Read};
use std::{io::Write, path::PathBuf, str::FromStr};
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DotConfig {
    pub configs: Vec<Config>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Config {
    pub name: String,
    pub path: String,
    pub hash: Option<String>,
}

/// Hashes the contents of a file and returns the hash as a string
fn hash_file(bytes: &[u8]) -> String {
    hex_digest(Algorithm::SHA256, bytes)
}

/// Fix the path to be absolute and not relative
pub fn fix_path(path: &str) -> String {
    if !path.starts_with('~') {
        return String::from(path);
    }
    path.replace(
        '~',
        home::home_dir()
            .expect("Failed to get home directory")
            .as_path()
            .to_str()
            .unwrap(),
    )
}

/// Check if the path has changed since the last time it was hashed
fn check_update_metadata_required(config: &Config) -> Option<()> {
    if let Ok(metadata_digest) = metadata_digest(&config.path) {
        if Some(metadata_digest) == config.hash {
            return Some(());
        }
    }
    None
}

/// Hashes the metadata of a file/dir and returns the hash as a string
fn metadata_digest(path: &str) -> Result<String> {
    // TODO: Optimize this to not have to create a new string
    let path = fix_path(path);
    let path = PathBuf::from_str(&path).unwrap();
    let mut hashes = Vec::new();

    if path.is_dir() {
        for entry in WalkDir::new(path) {
            let entry = entry?;
            let child_path = entry.path();
            if child_path.is_file() {
                let mut file = File::open(child_path)?;
                let mut contents = Vec::new();
                file.read_to_end(&mut contents)?;
                hashes.push(hash_file(&contents));
            }
        }
    }

    Ok(hash_file(hashes.join("").as_bytes()))
}

impl DotConfig {
    /// Save the config file to disk
    pub fn save_config(&self) -> Result<()> {
        let ron_pretty = PrettyConfig::new()
            .depth_limit(2)
            .extensions(Extensions::IMPLICIT_SOME);

        let config = to_string_pretty(self, ron_pretty).context("Failed to serialize config")?;

        let mut file = File::create("config.ron")?;
        file.write_all(config.as_bytes())?;

        Ok(())
    }

    /// Update the digest of all configs in the config file
    pub fn update_dotconfig(&self) -> Result<Self> {
        let mut new_dotconfig = DotConfig::default();
        println!("default dotconfig: {:#?}", new_dotconfig);

        for dir in &self.configs {
            if check_update_metadata_required(dir).is_none() {
                println!("Updating {}.", dir.name);
                let new_hash = metadata_digest(&dir.path)?;

                new_dotconfig.configs.push(Config::new(
                    dir.name.clone(),
                    dir.path.clone(),
                    Some(new_hash),
                ));
            } else {
                println!("Skipping {:?} already up-to date.", dir.name);
                new_dotconfig.configs.push(dir.clone());
            }
        }

        Ok(new_dotconfig)
    }
}

impl Config {
    fn new(name: String, path: String, hash: Option<String>) -> Self {
        Self { name, path, hash }
    }
}
