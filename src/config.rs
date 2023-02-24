use crate::*;
use walkdir::WalkDir;
use xxhash_rust::xxh3::Xxh3;

#[derive(Serialize, Deserialize)]
pub struct Config<'a> {
    pub name: &'a str,
    pub path: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<u64>,
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
    #[inline(always)]
    pub fn new(name: &'a str, path: &'a str, hash: Option<u64>) -> Self {
        Self { name, path, hash }
    }

    /// Hashes the metadata of a file/dir and returns the hash as a string
    #[inline(always)]
    pub fn metadata_digest(&self) -> Result<u64> {
        let path = PathBuf::from_str(self.path)?
            .fix_path()
            .ok_or_else(|| PathBuf::from_str(self.path).unwrap())
            .expect("Failed to fix path");

        let mut hasher = Xxh3::new();

        if path.is_dir() {
            WalkDir::new(path).into_iter().for_each(|entry| {
                let entry = entry.expect("Failed to read entry");
                let child_path = entry.path();
                if child_path.is_file() {
                    let mut file = fs::File::open(child_path).expect("Failed to open file");
                    let mut contents = Vec::new();
                    file.read_to_end(&mut contents)
                        .expect("Failed to read file");
                    hasher.update(&contents);
                }
            });
        }

        Ok(hasher.digest())
    }

    /// Check if the path has changed since the last time it was hashed
    #[inline(always)]
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
    #[inline(always)]
    pub fn pull_config(&self, path: &str) -> Result<()> {
        let dotconfigs_path = path
            .fix_path()
            .ok_or_else(|| PathBuf::from_str(path).unwrap())
            .expect("Failed to fix path");

        let selfpath = self
            .path
            .fix_path()
            .ok_or_else(|| PathBuf::from_str(self.path).unwrap())
            .expect("Failed to fix path");

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
        WalkDir::new(selfpath)
            .into_iter()
            .filter_map(|e| e.ok())
            .for_each(|entry| {
                // ignore git directory
                if entry.path().to_str().unwrap().contains(".git") {
                    return;
                }

                let path = entry.path();
                let new_path = dotconfigs_path.join(
                    PathBuf::from(&self.name).join(
                        path.strip_prefix(
                            self.path
                                .fix_path()
                                .ok_or_else(|| PathBuf::from_str(self.path).unwrap())
                                .expect("Failed to fix path"),
                        )
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

    /// Sync configs from the dotconfig directory to the home directory
    #[inline(always)]
    pub fn push_config(&self, path: &str) -> Result<()> {
        let dotconfigs_path = path
            .fix_path()
            .ok_or_else(|| PathBuf::from_str(path).unwrap())
            .expect("Failed to fix path");

        let config_path = self
            .path
            .fix_path()
            .ok_or_else(|| PathBuf::from_str(self.path).unwrap())
            .expect("Failed to fix path");

        // If dotconfigs_path doesn't exist, then
        if !dotconfigs_path.exists() {
            panic!("Dotconfigs path doesn't exist!");
        }

        // If the config path doesn't exist, create it
        if !config_path.exists() {
            std::fs::create_dir_all(&config_path)?;
        }

        // copy config from dotconfigs_path dir to config_path dir
        WalkDir::new(&config_path)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .for_each(|entry| {
                // ignore git directory
                if entry.path().to_str().unwrap().contains(".git") {
                    return;
                }

                let path = &dotconfigs_path.join(
                    // Convert: /home/user/dotconfigs-repo/someconfig/config to /home/user/.config/someconfig/config
                    PathBuf::from(
                        &config_path
                            .strip_prefix(home_dir().unwrap().join(".config"))
                            .unwrap(),
                    )
                    .join(
                        entry
                            .path()
                            .strip_prefix(&config_path)
                            .expect("Failed to strip prefix"),
                    ),
                );

                println!("Copying {:?} to {}", path, entry.path().display());

                // TODO: add check if we actually need to copy the file (check the hash of the current file and the hash of the file in the dotconfigs repo)
                copy_dir(path, &entry.path().to_path_buf()).expect("Failed to copy config back");
            });

        Ok(())
    }
}

impl std::fmt::Display for Config<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ ")?;
        write!(f, "name: {}, ", self.name)?;
        write!(f, "path: {}, ", self.path)?;
        write!(f, "hash: {} ", self.hash.as_ref().unwrap_or(&0))?;
        write!(f, "}}")
    }
}

fn copy_dir<T>(from: T, to: T) -> Result<()>
where
    T: AsRef<std::path::Path>,
{
    if to.as_ref().exists() {
        // TODO: Replace this to check if the previous to config has the same hash as the new one
        fs::remove_dir_all(&to)?;
    }
    fs::create_dir_all(&to)?;

    fs::read_dir(from)?
        .filter_map(|e| e.ok())
        .for_each(|entry| {
            let filetype = entry.file_type().expect("Failed to read file type");
            if filetype.is_dir() {
                copy_dir(entry.path(), to.as_ref().join(entry.file_name()))
                    .expect("Failed to copy directory");
            } else if filetype.is_file() {
                if let Err(e) = fs::copy(entry.path(), to.as_ref().join(entry.file_name())) {
                    match e.kind() {
                        std::io::ErrorKind::AlreadyExists => {
                            println!("File already exists, skipping: {}", entry.path().display())
                        }
                        _ => panic!("Error copying file: {e}"),
                    }
                }
            } else {
                println!("Skipping symlinks file: {}", entry.path().display());
            }
        });
    Ok(())
}
