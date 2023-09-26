use crate::{
    fix_path, hasher,
    utils::{self, FixPath},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

/// Config struct for storing config metadata and syncing configs.
///
/// The `Config` struct represents a configuration file or directory.
/// It includes information such as the name, path, hash
/// (used to check if the config has changed since the last sync),
/// and the type of configuration (file or directory).
///
/// # Example
///
/// ```rust
/// use sync_dotfiles_rs::config::{Config, ConfType};
///
/// let config = Config::new(
///     String::from("example-config"),
///     String::from("/path/to/example-config"),
///     None,
///     Some(ConfType::File),
/// );
/// ```
#[derive(Serialize, Deserialize)]
pub struct Config {
    /// Name of the config (e.g., "vimrc")
    pub name: String,
    /// Path to the config (e.g., "${HOME}/.vimrc")
    pub path: String,
    /// Hash of the config
    /// (used to check if the config has changed since the last sync)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// Config type (file or directory)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conf_type: Option<ConfType>,
}

/// Enum representing the type of a configuration, which can be either a
/// file or a directory.
///
/// The `ConfType` enum is used to specify whether a configuration is a
/// file or a directory.
///
/// # Variants
///
/// - `File`: Indicates that the configuration is a file.
/// - `Dir`: Indicates that the configuration is a directory.
///
/// ## Equality Comparison
///
/// The `ConfType` enum implements the `PartialEq` and `Eq` traits,
/// allowing you to compare instances for equality.
///
/// # Examples
///
/// ```rust
/// use sync_dotfiles_rs::config::ConfType;
///
/// let file_type = ConfType::File;
/// let dir_type = ConfType::Dir;
///
/// assert_eq!(file_type, ConfType::File);
/// assert_eq!(dir_type, ConfType::Dir);
/// ```
#[derive(Serialize, Deserialize, Debug)]
pub enum ConfType {
    /// Configuration is a file.
    File,
    /// Configuration is a directory.
    Dir,
}

/// Equality Comparison for `ConfType`.
///
/// The `PartialEq` trait allows you to compare instances of the `ConfType`
/// enum for equality.
///
/// # Examples
///
/// ```rust
/// use sync_dotfiles_rs::config::ConfType;
///
/// let file_type = ConfType::File;
/// let another_file_type = ConfType::File;
/// let dir_type = ConfType::Dir;
///
/// assert_eq!(file_type, another_file_type);
/// assert_ne!(file_type, dir_type);
/// ```
///
/// ## Implementation Notes
///
/// - Two `ConfType` variants are considered equal if they are of the
/// same variant (`File` or `Dir`).
impl PartialEq for ConfType {
    fn eq(&self, other: &Self) -> bool {
        match self {
            ConfType::File => matches!(other, ConfType::File),
            ConfType::Dir => matches!(other, ConfType::Dir),
        }
    }
}

/// Equality Comparison for `ConfType`.
///
/// The `Eq` trait allows you to compare instances of the
/// `ConfType` enum for equality.
///
/// # Examples
///
/// ```rust
/// use sync_dotfiles_rs::config::ConfType;
///
/// let file_type = ConfType::File;
/// let another_file_type = ConfType::File;
/// let dir_type = ConfType::Dir;
///
/// assert_eq!(file_type, another_file_type);
/// assert_ne!(file_type, dir_type);
/// ```
///
/// ## Implementation Notes
///
/// - Two `ConfType` variants are considered equal if they are of the same
/// variant (`File` or `Dir`).
impl Eq for ConfType {}

impl ConfType {
    /// Check if the config is a file.
    fn is_file(&self) -> bool {
        matches!(self, ConfType::File)
    }

    /// Check if the config is a directory.
    fn is_dir(&self) -> bool {
        matches!(self, ConfType::Dir)
    }
}

/// Default implementation for `Config`.
///
/// The `Config` struct implements the `Default` trait, allowing you to
/// create a new `Config` instance with default values.
///
/// # Examples
///
/// ```rust
/// use sync_dotfiles_rs::config::Config;
///
/// let config = Config::default();
///
/// assert_eq!(config.name, String::from("placeholder"));
/// assert_eq!(config.path, String::from("~/placeholder"));
/// assert_eq!(config.hash, None);
/// assert_eq!(config.conf_type, None);
/// ```
impl Default for Config {
    fn default() -> Self {
        Config {
            name: String::from("placeholder"),
            path: String::from("~/placeholder"),
            hash: None,
            conf_type: None,
        }
    }
}

impl Config {
    /// Create a new `Config` instance with the specified attributes.
    ///
    /// This method allows you to create a new `Config` instance with a
    /// given name, path, hash, and configuration type.
    ///
    /// # Arguments
    ///
    /// * `name` - A string representing the name of the configuration.
    /// * `path` - A string representing the path to the configuration.
    /// * `hash` - An optional string representing the hash of the
    /// configuration (used for change detection).
    /// * `conf_type` - An optional `ConfType` enum indicating the type of the
    /// configuration (file or directory).
    ///
    /// # Returns
    ///
    /// A new `Config` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sync_dotfiles_rs::config::{Config, ConfType};
    ///
    /// let config = Config::new(
    ///     String::from("vimrc"),
    ///     String::from("~/.vimrc"),
    ///     Some(String::from("abcd1234")),
    ///     Some(ConfType::File),
    /// );
    /// ```
    ///
    /// In this example, a new `Config` instance is created with a name
    /// "vimrc," a path "~/.vimrc, "a hash "abcd1234," and a configuration
    /// type "File."
    ///
    /// ```rust
    /// use sync_dotfiles_rs::config::Config;
    ///
    /// let config = Config::new(
    ///     String::from("example"),
    ///     String::from("~/example.conf"),
    ///     None,
    ///     None,
    /// );
    /// ```
    ///
    /// In this example, a new `Config` instance is created with a name
    /// "example" and a path "~/example.conf" without specifying a hash or
    /// configuration type.
    pub fn new(
        name: String,
        path: String,
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

    /// Check if the config path exists.
    ///
    /// This method checks whether the file or directory specified by the
    /// `path` field of the `Config` instance exists.
    ///
    /// # Returns
    ///
    /// `true` if the path exists, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sync_dotfiles_rs::config::Config;
    ///
    /// let non_existant_config = Config::new(
    ///     String::from("example"),
    ///     String::from("~/example.conf"),
    ///     None,
    ///     None,
    /// );
    ///
    /// let existant_config = Config::new(
    ///     String::from("config.ron"),
    ///     format!("{}/examples/config.ron", env!("CARGO_MANIFEST_DIR")),
    ///     None,
    ///     None,
    /// );
    ///
    /// assert!(!non_existant_config.path_exists());
    /// assert!(existant_config.path_exists());
    /// ```
    pub fn path_exists(&self) -> bool {
        fix_path!(self.path, PathBuf::from(&self.path)).exists()
    }

    /// Calculate the hash of the metadata for a file or directory.
    ///
    /// This method computes the hash of the metadata
    /// (e.g., file content or directory structure) for the configuration file
    /// or directory specified by the `path` field of the `Config` instance.
    ///
    /// # Returns
    ///
    /// A `Result` containing the hash as a string if successful,
    /// or an error if the operation fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sync_dotfiles_rs::config::Config;
    ///
    /// let config = Config::new(
    ///     String::from("config.ron"),
    ///     format!("{}/examples/config.ron", env!("CARGO_MANIFEST_DIR")),
    ///     None,
    ///     None,
    /// );
    ///
    /// match config.metadata_digest() {
    ///     Ok(hash) => println!("Metadata digest: {}", hash),
    ///     Err(err) => eprintln!("Error calculating metadata digest: {:?}", err),
    /// }
    /// ```
    pub fn metadata_digest(&self) -> Result<String> {
        let path = fix_path!(self.path, PathBuf::from(&self.path));

        // check if the path exists and return empty string if it doesn't
        if !self.path_exists() {
            return Ok(String::new());
        }

        if path.is_file() {
            return Ok(hasher::get_file_hash(&path, &mut Sha1::new())?);
        }
        if path.is_dir() {
            return Ok(hasher::get_complete_dir_hash(&path, &mut Sha1::new())?);
        }

        Err(anyhow::anyhow!("Invalid config type: {:#?}", self.path))
    }

    /// Check if the configuration needs metadata update.
    ///
    /// This method checks whether the configuration needs an update of its
    /// metadata, such as hash and configuration type. It is required because
    /// the hash and configuration type are not stored in the dotconfig file
    /// and are used to determine if the configuration has changed since the
    /// last sync.
    ///
    /// # Returns
    ///
    /// `true` if metadata update is required, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sync_dotfiles_rs::config::{Config, ConfType};
    ///
    /// let mut config = Config::new(
    ///     String::from("config.ron"),
    ///     format!("{}/examples/config.ron", env!("CARGO_MANIFEST_DIR")),
    ///     None,
    ///     Some(ConfType::File),
    /// );
    ///
    /// assert!(config.check_update_metadata_required());
    /// ```
    pub fn check_update_metadata_required(&self) -> bool {
        match self.hash.as_ref() {
            Some(hash) => {
                let digest = self
                    .metadata_digest()
                    .expect("Failed to get metadata digest");

                // If hash hash doesn't match, then we require metadata update
                if hash.ne(&digest) {
                    true
                } else {
                    // If config tye is not preset, then we require metadata update
                    self.conf_type.is_none()
                }
            }
            // If hash is not set, then we require metadata update
            None => true,
        }
    }

    /// Update the hash of the configuration's metadata.
    ///
    /// This method calculates the new hash of the configuration's metadata
    /// and updates the `hash` field of the `Config` instance.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the operation.
    pub fn update_config_hash(&mut self) -> Result<()> {
        // calculate the new hash of the config
        let new_hash = self
            .metadata_digest()
            .expect("Failed to get metadata digest");

        self.hash = Some(new_hash);
        Ok(())
    }

    /// Update the configuration type of the `Config`.
    ///
    /// This method checks whether the configuration specified by the
    /// `Config` instance is a file or a directory.
    /// If the `conf_type` field is already set, it skips the check.
    /// Otherwise, it determines the type based on the path.
    ///
    /// If the path does not exist, it prints an error message and
    /// returns without modifying the `Config`.
    ///
    /// # Errors
    ///
    /// This method may return an error if it encounters issues accessing the
    /// file system or determining the config type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sync_dotfiles_rs::config::{Config, ConfType};
    ///
    /// let mut config = Config::new(
    ///     String::from("config.ron"),
    ///     format!("{}/examples/config.ron", env!("CARGO_MANIFEST_DIR")),
    ///     None,
    ///     None,
    /// );
    ///
    /// // Update the configuration type.
    /// config.update_config_type().expect("Failed to update config type");
    ///
    /// assert_eq!(config.conf_type, Some(ConfType::File));
    pub fn update_config_type(&mut self) -> Result<()> {
        let path = fix_path!(self.path, PathBuf::from(&self.path));

        if !path.exists() {
            println!("Config does not exist: {:#?}", self.path);
            return Ok(());
        }

        // If the config type is not set, then update it
        if self.conf_type.is_none() {
            if path.is_file() {
                self.conf_type = Some(ConfType::File);
            } else if path.is_dir() {
                self.conf_type = Some(ConfType::Dir);
            } else {
                println!("Invalid config type: {:#?}", self.path);
                return Err(anyhow::anyhow!("Invalid config type"));
            }
        }

        Ok(())
    }

    /// Update the metadata of the `Config`.
    ///
    /// This method updates the hash of the configuration and its type by
    /// calling `update_config_hash` and `update_config_type`.
    ///
    /// # Errors
    ///
    /// This method may return errors if any of the sub-methods
    /// (`update_config_hash` or `update_config_type`) encounter issues.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sync_dotfiles_rs::config::Config;
    ///
    /// let mut config = Config::new(
    ///     String::from("config.ron"),
    ///     format!("{}/examples/config.ron", env!("CARGO_MANIFEST_DIR")),
    ///     None,
    ///     None,
    /// );
    ///
    /// // Update the metadata of the configuration.
    /// config.update_metadata().expect("Failed to update metadata");
    /// ```
    ///
    /// ## Implementation Notes
    ///
    /// - This method updates the `hash` and `conf_type` fields in the `Config`
    /// instance.
    /// - It relies on the `update_config_hash` and `update_config_type` methods.
    pub fn update_metadata(&mut self) -> Result<()> {
        self.update_config_hash()?;
        self.update_config_type()?;

        Ok(())
    }

    /// Sync the configuration from the dotconfig directory to the home
    /// directory or expected configuration directory.
    ///
    /// This method copies the configuration files or directory from the
    /// dotconfig directory to the specified destination.
    ///
    /// If the `conf_type` field is set to `ConfType::File`, it copies the
    /// file directly. If set to `ConfType::Dir`, it copies the entire
    /// directory and its contents.
    ///
    /// # Arguments
    ///
    /// - `path`: A string specifying the destination path where the
    /// configuration should be synced.
    ///
    /// # Errors
    ///
    /// This method may return errors if it encounters issues during the
    /// file copying process.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sync_dotfiles_rs::config::Config;
    ///
    /// let config = Config::new(
    ///     String::from("config.ron"),
    ///     format!("{}/examples/config.ron", env!("CARGO_MANIFEST_DIR")),
    ///     None,
    ///     None,
    /// );
    ///
    /// // Sync the configuration to the specified path.
    /// config.pull_config(&format!("{}/examples", env!("CARGO_MANIFEST_DIR")))
    ///         .expect("Failed to pull config");
    /// ```
    ///
    /// ## Implementation Notes
    ///
    /// - This method determines whether to copy a file or a directory based
    /// on the `conf_type` field.
    /// - It relies on the `copy_config_directory` method for directory
    /// copying.
    pub fn pull_config(&self, path: &String) -> Result<()> {
        let dotconfigs_path = fix_path!(path, path.into());

        let selfpath = fix_path!(self.path, PathBuf::from(&self.path));

        let config_path = dotconfigs_path.join(selfpath);

        // If dotconfigs_path doesn't exist, create it
        if !dotconfigs_path.exists() {
            println!(
                "Creating dotconfigs directory: {:#?}",
                dotconfigs_path.display()
            );
            fs::create_dir_all(&dotconfigs_path)?;
        }

        // If the config path doesn't exist, skip it
        if !config_path.exists() {
            println!("Path does not exists! skipping: {:#?}", config_path);
            return Ok(());
        }

        // if the config path is just a file, then directly copy it
        if let Some(conf_type) = &self.conf_type {
            if conf_type.is_file() {
                fs::copy(
                    &config_path,
                    dotconfigs_path.join(config_path.file_name().unwrap()),
                )?;
                return Ok(());
            } else if conf_type.is_dir() {
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
                                path.strip_prefix(fix_path!(self.path, PathBuf::from(&self.path)))
                                    .unwrap(),
                            ),
                        );

                        if path.is_dir() {
                            if let Err(e) = fs::create_dir_all(&new_path) {
                                match e.kind() {
                                    io::ErrorKind::AlreadyExists => {}
                                    _ => {
                                        println!("Failed to create directory: {:#?}", new_path);
                                    }
                                }
                            }
                        } else {
                            fs::copy(path, new_path).expect("Failed to copy file");
                        }
                    });
            }
        }

        Ok(())
    }

    /// Copies the contents of a configuration directory from the dotconfig
    /// directory to the home directory.
    ///
    /// This function is used by the push_config function to perform the
    /// actual copy operation.
    ///
    /// # Arguments
    ///
    /// * `to_config_path`: The path to the configuration directory in the
    /// home directory.
    /// * `from_dotconfigs_path`: The path to the dotconfig directory.
    ///
    /// # Returns
    ///
    /// Returns a Result indicating success or an error if the copy operation
    /// fails.
    fn copy_config_directory(to_config_path: &PathBuf, from_dotconfigs_path: &Path) -> Result<()> {
        if !to_config_path.exists() {
            fs::create_dir_all(to_config_path).expect("Failed to create directory");
        } else {
            // Delete all the files in the to_config_path directory
            // Use match for Ignoring the NotFound error as it is not a problem
            if let Err(e) = fs::remove_dir_all(to_config_path) {
                match e.kind() {
                    io::ErrorKind::NotFound => {}
                    _ => {
                        return Err(anyhow::anyhow!(
                            "Failed to delete directory: {:#?}",
                            to_config_path
                        ));
                    }
                }
            }

            // Create the to_config_path directory again
            fs::create_dir_all(to_config_path).expect("Failed to create directory");
        }

        // copy config from from_dotconfigs_path directory to to_config_path directory
        WalkDir::new(to_config_path).into_iter().for_each(|entry| {
            if entry.is_err() {
                println!("Failed to read directory: {:#?}", entry);
                return;
            }

            let entry = entry.ok().unwrap();

            utils::copy_dir(from_dotconfigs_path, entry.path()).expect("Failed to copy directory");
        });

        Ok(())
    }

    /// Push the configuration from the home directory or expected
    /// configuration directory to the dotconfig directory.
    ///
    /// This method copies the configuration files or directory from the source
    /// to the dotconfig directory.
    ///
    /// If the `conf_type` field is set to `ConfType::File`, it copies the file
    /// directly. If set to `ConfType::Dir`, it copies the entire directory and
    /// its contents.
    ///
    /// # Arguments
    ///
    /// - `path`: A string specifying the destination path in the dotconfig
    /// directory where the configuration should be pushed.
    ///
    /// # Errors
    ///
    /// This method may return errors if it encounters issues during the file
    /// copying process or if the specified paths do not exist.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sync_dotfiles_rs::config::{Config, ConfType};
    /// use std::io::{Read, Write};
    ///
    /// let path = std::path::PathBuf::from(format!(
    ///     "{}/examples/config.ron", env!("CARGO_MANIFEST_DIR")
    /// ));
    ///
    /// let mut file = std::fs::File::open(&path).expect("Failed to open file");
    ///
    /// let mut content = String::new();
    ///
    /// file.read_to_string(&mut content).expect("Failed to read file");
    ///
    /// let config = Config::new(
    ///     String::from("config.ron"),
    ///     format!("{}/examples/config.ron", env!("CARGO_MANIFEST_DIR")),
    ///     None,
    ///     Some(ConfType::File),
    /// );
    ///
    /// assert!(config.path_exists());
    ///
    /// // Push the configuration to the dotconfig directory.
    /// config.push_config(&path)
    ///             .expect("Failed to push config");
    ///
    /// let mut file =
    ///     std::fs::File::create(path).expect("Failed to create config file");
    ///
    /// file.write_all(content.as_bytes()).expect("Failed to write file");
    /// ```
    ///
    /// ## Implementation Notes
    ///
    /// - This method determines whether to copy a file or a directory based on
    /// the `conf_type` field.
    /// - It relies on the `copy_config_directory` method for directory
    /// copying.
    pub fn push_config(&self, path: &PathBuf) -> Result<()> {
        let from_dotconfigs_path = fix_path!(path, path.into());
        let to_config_path = fix_path!(self.path, PathBuf::from(&self.path));

        // If dotconfigs_path doesn't exist, then return
        if !from_dotconfigs_path.exists() {
            return Err(anyhow::anyhow!(
                "{:#?} does not exist!",
                from_dotconfigs_path
            ));
        }

        // If the to_config_path is a file, then just copy it
        if let Some(conf_type) = &self.conf_type {
            if conf_type.is_file() {
                fs::copy(from_dotconfigs_path, &to_config_path)?;
            } else if conf_type.is_dir() {
                Self::copy_config_directory(&to_config_path, &from_dotconfigs_path)?;
            } else {
                return Err(anyhow::anyhow!("Invalid config type!"));
            }
        } else {
            // check if the to_config_path is a file
            if to_config_path.is_file() {
                fs::copy(from_dotconfigs_path, &to_config_path)?;
            } else if to_config_path.is_dir() {
                Self::copy_config_directory(&to_config_path, &from_dotconfigs_path)?;
            } else {
                return Err(anyhow::anyhow!("Invalid config path!"));
            }
        }

        Ok(())
    }
}

/// Implements the Display trait for the Config struct.
///
/// This allows a Config instance to be formatted as a string when using the
/// format! macro
/// or the println! macro, providing a human-readable representation of the
/// Config instance.
///
/// # Example
///
/// ```rust
/// use sync_dotfiles_rs::config::{Config, ConfType};
///
/// let config = Config::new(
///     String::from("config.ron"),
///     format!("{}/examples/config.ron", env!("CARGO_MANIFEST_DIR")),
///     Some(String::from("abcd1234")),
///     Some(ConfType::File),
/// );
///
/// println!("Config details: {}", config);
/// ```
///
/// Output:
/// ```text
/// Config details: { name: config, path: <path>/config.ron, conf_type: Some(ConfType::File) }
/// ```
impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
