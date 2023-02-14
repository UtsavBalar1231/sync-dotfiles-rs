use crate::*;
use walkdir::WalkDir;
use xxhash_rust::xxh3::Xxh3;

#[derive(Serialize, Deserialize)]
pub struct Config<'a> {
    pub name: &'a str,
    pub path: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

impl Default for Config<'_> {
    fn default() -> Self {
        Config {
            name: "/* Name of the config */",
            path: "/* Path to the config */",
            hash: None,
        }
    }
}

impl<'a> Config<'a> {
    #[inline]
    pub fn new(name: &'a str, path: &'a str, hash: Option<String>) -> Self {
        Self { name, path, hash }
    }

    /// Hashes the metadata of a file/dir and returns the hash as a string
    #[inline]
    pub fn metadata_digest(&self) -> Result<String> {
        let path = PathBuf::from_str(&self.path)?.fix_path()?;
        let mut hasher = Xxh3::new();

        if path.is_dir() {
            WalkDir::new(path).into_iter().for_each(|entry| {
                let entry = entry.expect("Failed to read entry");
                let child_path = entry.path();
                if child_path.is_file() {
                    let mut file = File::open(child_path).expect("Failed to open file");
                    let mut contents = Vec::new();
                    file.read_to_end(&mut contents)
                        .expect("Failed to read file");
                    hasher.update(&contents);
                }
            });
        }

        Ok(hasher.digest().to_string())
    }

    /// Check if the path has changed since the last time it was hashed
    #[inline]
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
    #[inline]
    pub fn sync_config(&self, path: &str) -> Result<()> {
        let dotconfigs_path = path.to_string().fix_path()?;
        let selfpath = self.path.to_string().fix_path()?;
        let config_path = dotconfigs_path.join(&selfpath);

        // If dotconfigs_path doesn't exist, create it
        if !dotconfigs_path.exists() {
            std::fs::create_dir_all(&dotconfigs_path)?;
        }

        // If the config path doesn't exist, create it
        if !config_path.exists() {
            std::fs::create_dir_all(&config_path)?;
        }

        // copy config dir contents to dotconfigs_path dir
        WalkDir::new(selfpath).into_iter().for_each(|entry| {
            let entry = entry.expect("Failed to read entry");
            // ignore git directory
            if entry.path().to_str().unwrap().contains(".git") {
                return;
            }

            let path = entry.path();
            let new_path = dotconfigs_path.join(
                PathBuf::from(&self.name).join(
                    path.strip_prefix(self.path.to_string().fix_path().unwrap())
                        .unwrap(),
                ),
            );
            if path.is_dir() {
                std::fs::create_dir_all(new_path).expect("Failed to create dir");
            } else {
                std::fs::copy(path, new_path).expect("Failed to copy file");
            }
        });

        Ok(())
    }
}

impl std::fmt::Display for Config<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ ")?;
        write!(f, "name: {}, ", self.name)?;
        write!(f, "path: {}, ", self.path)?;
        write!(
            f,
            "hash: {} ",
            self.hash.as_ref().unwrap_or(&"None".to_string())
        )?;
        write!(f, "}}")
    }
}
