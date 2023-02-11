use crate::*;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub name: String,
    pub path: String,
    pub hash: Option<String>,
}

impl Config {
    pub fn new(name: &String, path: &String, hash: Option<String>) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_string(),
            hash,
        }
    }

    /// Hashes the metadata of a file/dir and returns the hash as a string
    pub fn metadata_digest(&self) -> Result<String> {
        let path = PathBuf::from_str(&self.path)?.fix_path()?;
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

    /// Check if the path has changed since the last time it was hashed
    pub fn check_update_metadata_required(&self) -> Result<()> {
        if self.metadata_digest().is_err() {
            return Err(anyhow::anyhow!("Failed to get metadata for {}", self.path));
        }

        if let Some(hash) = self.hash.as_ref() {
            if &self.metadata_digest().context("Metadeta digest failed!")? == hash {
                return Err(anyhow::anyhow!("No update required"));
            }
        }

        Ok(())
    }

    /// Sync configs from the dotconfig directory to the home directory
    pub fn sync_config(&self, path: &String) -> Result<()> {
        let dotconfigs_path = path.fix_path()?;
        let config_path = dotconfigs_path.join(self.path.fix_path()?);

        println!(
            "Copying {config_path:?} to {}{}",
            dotconfigs_path.display(),
            self.name,
        );

        // If dotconfigs_path doesn't exist, create it
        if !dotconfigs_path.exists() {
            std::fs::create_dir_all(&dotconfigs_path)?;
        }

        // If the config path doesn't exist, create it
        if !config_path.exists() {
            std::fs::create_dir_all(config_path)?;
        }

        // copy config dir contents to dotconfigs_path dir
        for entry in WalkDir::new(&self.path.fix_path()?) {
            let entry = entry?;
            // ignore git directory
            if entry.path().to_str().unwrap().contains(".git") {
                continue;
            }

            let path = entry.path();
            let new_path = dotconfigs_path
                .join(PathBuf::from(&self.name).join(path.strip_prefix(&self.path.fix_path()?)?));
            if path.is_dir() {
                std::fs::create_dir_all(new_path).context("Failed to create dir")?;
            } else {
                std::fs::copy(path, new_path).expect("Failed to copy file");
            }
        }

        Ok(())
    }
}
