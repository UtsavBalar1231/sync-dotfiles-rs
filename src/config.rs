use crate::*;
use merkle_hash::MerkleTree;
use rayon::prelude::*;
use walkdir::WalkDir;

/// Config struct for storing config metadata and syncing configs
///
/// # Example
/// ```
/// use dotconfigs::config::Config;
///
/// let config = Config::new("<Name of the config>", "<Path to the config>", None);
/// ```
/// Provides methods to sync configs from the dotconfig directory to the home directory
/// and vice versa. Also provides methods to check if the config has changed since the last
/// time it was synced.

#[derive(Serialize, Deserialize)]
pub struct Config<'a> {
    /// Name of the config (e.g. vimrc)
    pub name: &'a str,
    /// Path to the config (e.g. ~/.vimrc)
    pub path: &'a str,
    /// Hash of the config (used to check if the config has changed since the last time it was synced)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// Config type (file or directory)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conf_type: Option<ConfType>,
}

/// Struct for storing the config type, i.e. whether the config is a file or a directory
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum ConfType {
    /// Config is a file
    File,
    /// Config is a directory
    Dir,
}

/// ConfType can be compared using `==` for equality
impl PartialEq for ConfType {
    fn eq(&self, other: &Self) -> bool {
        match self {
            ConfType::File => match other {
                ConfType::File => true,
                ConfType::Dir => false,
            },
            ConfType::Dir => match other {
                ConfType::File => false,
                ConfType::Dir => true,
            },
        }
    }
}

/// ConfType can be compared using `==` for equality
impl Eq for ConfType {}

impl ConfType {
    /// Check if the config is a file
    fn is_file(&self) -> bool {
        if self.eq(&ConfType::File) {
            return true;
        }
        false
    }

    /// Check if the config is a directory
    fn is_dir(&self) -> bool {
        if self.eq(&ConfType::Dir) {
            return true;
        }
        false
    }
}

/// Default implementation for Config
impl Default for Config<'_> {
    fn default() -> Self {
        Config {
            name: "/* Name of the config */",
            path: "/* Path to the config */",
            hash: None,
            conf_type: None,
        }
    }
}

impl<'a> Config<'a> {
    /// Create a new Config using the name, path, hash and config type
    #[inline(always)]
    pub fn new(
        name: &'a str,
        path: &'a str,
        hash: Option<String>,
        conf_type: Option<ConfType>,
    ) -> Self {
        Self {
            name,
            path,
            hash,
            conf_type,
        }
    }

    /// Check if config path exists
    #[inline(always)]
    pub fn path_exists(&self) -> bool {
        let path = self
            .path
            .fix_path()
            .unwrap_or_else(|| PathBuf::from_str(self.path).unwrap());

        path.exists()
    }

    /// Hashes the metadata of a file or directory and returns the hash as a string
    #[inline(always)]
    pub fn metadata_digest(&self) -> Result<String> {
        let path = self
            .path
            .fix_path()
            .unwrap_or_else(|| PathBuf::from_str(self.path).unwrap());

        // check if the path exists and return empty string if it doesn't
        if !self.path_exists() {
            return Ok(String::new());
        }

        // safely unwrap the hash or return empty string
        let hasher = MerkleTree::builder(path.to_string_lossy())
            .hash_names(true)
            .build()
            .expect("Failed to build merkle tree");

        // print hash as hex
        let hash = hasher
            .root
            .item
            .hash
            .par_iter()
            .map(|b| format!("{b:x}"))
            .collect::<String>();

        Ok(hash)
    }

    /// Check if the path has changed since the last time it was hashed
    /// This is required because the hash is not stored in the dotconfig file,
    /// Also required because the config type is not stored in the dotconfig file
    /// and is only used to determine if the config is a file or a directory during syncing
    #[inline(always)]
    pub fn check_update_metadata_required(&self) -> Result<()> {
        let digest = self.metadata_digest();
        if digest.is_err() {
            return Err(anyhow::anyhow!("Failed to get metadata for {}", self.path));
        }

        match self.hash.as_ref() {
            Some(hash) => {
                if &digest.unwrap() == hash {
                    return Err(anyhow::anyhow!("No update required"));
                }
            }
            None => return Ok(()),
        }

        if let Some(conf) = self.conf_type {
            if conf.is_file() {
                let path = self
                    .path
                    .fix_path()
                    .unwrap_or_else(|| PathBuf::from_str(self.path).unwrap());

                if path.is_file() {
                    return Err(anyhow::anyhow!("No update required"));
                }
            } else if conf.is_dir() {
                let path = self
                    .path
                    .fix_path()
                    .unwrap_or_else(|| PathBuf::from_str(self.path).unwrap());

                if path.is_dir() {
                    return Err(anyhow::anyhow!("No update required"));
                }
            }
        }

        Ok(())
    }

    /// Update hash of the config to the current hash
    /// This is required because the hash is not stored in the dotconfig file
    /// and is only used to determine if the config has changed since the last time it was synced
    #[inline(always)]
    pub fn update_config_hash(&mut self) -> Result<()> {
        // calculate the new hash of the config
        let new_hash = self
            .metadata_digest()
            .expect("Failed to get metadata digest");

        self.hash = Some(new_hash);
        Ok(())
    }

    /// Update the config type of the config
    /// This is required because the config type is not stored in the dotconfig file
    /// and is only used to determine if the config is a file or a directory
    #[inline(always)]
    pub fn update_config_type(&mut self) -> Result<()> {
        let path = self
            .path
            .fix_path()
            .unwrap_or_else(|| PathBuf::from_str(self.path).unwrap());

        if !path.exists() {
            println!("Config does not exist: {:#?}", self.path);
            return Ok(());
        }

        if path.is_file() {
            self.conf_type = Some(ConfType::File);
        } else if path.is_dir() {
            self.conf_type = Some(ConfType::Dir);
        } else {
            println!("Invalid config type: {:#?}", self.path);
            return Err(anyhow::anyhow!("Invalid config type"));
        }

        Ok(())
    }

    /// Update metadata of the config
    #[inline(always)]
    pub fn update_metadata(&mut self) -> Result<()> {
        self.update_config_hash()?;
        self.update_config_type()?;

        Ok(())
    }

    /// Sync configs from the dotconfig directory to the home directory
    #[inline(always)]
    pub fn pull_config(&self, path: &str) -> Result<()> {
        let dotconfigs_path = path
            .fix_path()
            .unwrap_or_else(|| PathBuf::from_str(path).unwrap());

        let selfpath = self
            .path
            .fix_path()
            .unwrap_or_else(|| PathBuf::from_str(self.path).unwrap());

        let config_path = dotconfigs_path.join(selfpath);

        // If dotconfigs_path doesn't exist, create it
        if !dotconfigs_path.exists() {
            println!(
                "Creating dotconfigs directory: {:#?}",
                dotconfigs_path.display()
            );
            std::fs::create_dir_all(&dotconfigs_path)?;
        }

        // If the config path doesn't exist, skip it
        if !config_path.exists() {
            println!("Path does not exists! skipping: {:#?}", config_path);
            return Ok(());
        }

        // if the config path is just a file, then directly copy it
        if let Some(conf) = self.conf_type {
            if conf.is_file() {
                println!("Copying file: {:#?}", config_path);
                std::fs::copy(&config_path, dotconfigs_path.join(self.name))?;
                return Ok(());
            } else if conf.is_dir() {
                // if the config path is a directory, then copy the directory contents
                WalkDir::new(config_path)
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
                                        .unwrap_or_else(|| PathBuf::from_str(self.path).unwrap()),
                                )
                                .unwrap(),
                            ),
                        );

                        if path.is_dir() {
                            if let Err(e) = std::fs::create_dir_all(&new_path) {
                                match e.kind() {
                                    std::io::ErrorKind::AlreadyExists => {}
                                    _ => {
                                        println!("Failed to create directory: {:#?}", new_path);
                                    }
                                }
                            }
                        } else {
                            std::fs::copy(path, new_path).expect("Failed to copy file");
                        }
                    });
            }
        }

        Ok(())
    }

    /// Sync configs from the dotconfig directory to the home directory
    #[inline(always)]
    pub fn push_config(&self, path: &str) -> Result<()> {
        let dotconfigs_path = path
            .fix_path()
            .unwrap_or_else(|| PathBuf::from_str(path).unwrap());

        let config_path = self
            .path
            .fix_path()
            .unwrap_or_else(|| PathBuf::from_str(self.path).unwrap());

        // If dotconfigs_path doesn't exist, then
        if !dotconfigs_path.exists() {
            println!("{:#?} does not exist!", dotconfigs_path);
            return Err(anyhow::anyhow!(
                "Dotconfigs path doesn't exist! Please clone the dotconfigs repo first!"
            ));
        }

        // If the config_path is a file, then just copy it
        if let Some(conf) = self.conf_type {
            if conf.is_file() {
                std::fs::copy(dotconfigs_path.join(self.name), &config_path)?;
                return Ok(());
            } else if conf.is_dir() {
                // If the config path doesn't exist, create it
                if !config_path.exists() {
                    println!(
                        "Directory not found! creating: {:#?}",
                        config_path.to_str().unwrap()
                    );
                    std::fs::create_dir_all(&config_path)?;
                } else {
                    // Delete all the files in the config_path directory
                    if let Err(e) = std::fs::remove_dir_all(&config_path) {
                        match e.kind() {
                            std::io::ErrorKind::NotFound => {}
                            _ => {
                                println!("Failed to delete directory: {:#?}", config_path);
                            }
                        }
                    }
                }

                // copy config from dotconfigs_path directory to config_path directory
                WalkDir::new(&config_path)
                    .into_iter()
                    .filter_map(|entry| entry.ok())
                    .for_each(|entry| {
                        // ignore git directory
                        if entry.path().to_str().unwrap().contains(".git") {
                            return;
                        }

                        // Convert: /home/user/dotconfigs-repo/config/* to config_path/*
                        let path = &dotconfigs_path.join(config_path.iter().last().unwrap());

                        copy_dir(path, &entry.path().to_path_buf())
                            .expect("Failed to copy config back");
                    });
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for Config<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ ")?;
        write!(f, "name: {}, ", self.name)?;
        write!(f, "path: {}, ", self.path)?;

        if let Some(conf_type) = &self.conf_type {
            write!(f, "conf_type: {conf_type:?} ")?;
        } else {
            write!(f, "conf_type: None ")?;
        }
        write!(f, "}}")
    }
}

fn copy_dir<T>(from: T, to: T) -> Result<()>
where
    T: AsRef<std::path::Path>,
{
    if to.as_ref().exists() {
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
                            println!(
                                "File already exists, skipping: {:#?}",
                                entry.path().display()
                            )
                        }
                        _ => panic!("Error copying file: {e}"),
                    }
                }
            } else {
                println!("Skipping symlinks file: {:#?}", entry.path().display());
            }
        });
    Ok(())
}
