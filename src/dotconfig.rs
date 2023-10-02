use crate::{
    config::ConfType,
    config::Config,
    fix_path, hasher,
    utils::{get_ron_formatter, FixPath},
};

use anyhow::{Context, Result};
use lazy_static::lazy_static;
use rayon::prelude::*;
use ron::{extensions::Extensions, ser::to_string_pretty, Options};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::{fmt, fs, io::Write, path::PathBuf, process, sync::Mutex};

/// Struct to store configuration data, including the path to the dotconfig
/// directory and a list of configuration files.
///
/// The `DotConfig` struct is used to manage configuration settings for
/// syncing dotfiles.
/// It includes the path to the dotconfig directory and a list of individual
/// `Config` structs, each representing a configuration file.
#[derive(Serialize, Deserialize)]
pub struct DotConfig {
    /// Enum representing the path to the dotconfig directory.
    pub dotconfigs_path: DotconfigPath,
    /// A vector of `Config` structs, each representing an individual
    /// configuration file.
    pub configs: Vec<Config>,
}

/// Enum representing the path to the dotconfig directory.
///
/// This enum is used to specify the path to the dotconfig directory.
/// It can be either a GitHub repository or a local directory.
///
/// # Examples
///
/// ```rust
/// use sync_dotfiles_rs::dotconfig::DotconfigPath;
///
/// let dotconfig_github = DotconfigPath::Github("https://github.com/user/repo".to_string());
/// let dotconfig_local = DotconfigPath::Local(String::from("~/dotfiles"));
/// ```
#[derive(Serialize, Deserialize)]
pub enum DotconfigPath {
    Github(String),
    Local(String),
}

lazy_static! {
    /// Mutex-protected global configuration file path.
    ///
    /// This static variable stores the path to the configuration file and
    /// allows it to be accessed and modified safely from multiple threads.
    static ref CONFIG_PATH: Mutex<PathBuf> = Mutex::new(get_default_config_path());
}

/// Function to determine the default configuration file path.
///
/// This function determines the default configuration file path based on
/// the operating system.
/// If the config file is not found in the ${HOME}/.config/sync-dotfiles
/// directory, it will try to find the config file in the current directory.
/// Otherwise, it will return an empty path.
fn get_default_config_path() -> PathBuf {
    let home_dir = PathBuf::from(env!("HOME"));
    // Try to find the config file in the ${HOME}/.sync-dotfiles.ron
    let path = home_dir.join(".sync-dotfiles.ron");
    if fs::File::open(&path).is_ok() {
        println!(
            "Found config file: {}/.sync-dotfiles.ron",
            home_dir.display()
        );
        return path;
    }
    // Try to find the config file in the ${HOME}/.config/sync-dotfiles directory
    let path = home_dir.join(".config/sync-dotfiles/config.ron");
    if fs::File::open(&path).is_ok() {
        println!("Found config file at {}", path.display());
        return path;
    }

    // If the config file is not found in the $HOME/.config/sync-dotfiles directory
    // Try to find the config file in the current directory
    let local_config_path = PathBuf::from("config.ron");
    if fs::File::open(&local_config_path).is_ok() {
        println!("Found config file in current directory");
        return local_config_path;
    }

    // Return an empty path if no config file is found
    PathBuf::new()
}

impl DotConfig {
    /// Parses the dotconfig file and returns a `DotConfig` structure.
    ///
    /// The dotconfig file is the configuration file that contains the list of
    /// all the configuration files to be synced.
    /// It is a RON file (`config.ron`), which is a human-readable version of
    /// the Rust data serialization format.
    ///
    /// The config file location can be specified by the user using the
    /// `--config-path` or `-c` flag.
    ///
    /// If the config file location is not specified by the user,
    /// the config file is searched in the `${HOME}/.config/sync-dotfiles`
    /// directory.
    /// If the config file is not found in the `${HOME/.config/sync-dotfiles`
    /// directory,
    /// the config file is searched in the current directory.
    ///
    /// # Arguments
    ///
    /// * `filepath` - An optional reference to a String representing the path
    /// to the config file specified by the user.
    ///
    /// # Returns
    ///
    /// A Result containing a `DotConfig` struct if the parsing is successful,
    /// or an error if parsing fails.
    pub fn parse_dotconfig(filepath: &Option<String>) -> Result<Self> {
        // If the user has specified a config file path
        if let Some(path) = filepath {
            *CONFIG_PATH.lock().unwrap() = fix_path!(path);
        }

        let file = fs::File::open(CONFIG_PATH.lock().unwrap().as_path())
            .context("Failed to open config file from the current directory")?;

        let config = Options::default()
            .with_default_extension(Extensions::IMPLICIT_SOME)
            .from_reader(file)
            .context("Failed to parse config file")?;

        Ok(config)
    }

    /// Fix the config file path if it is a relative path.
    /// Also fix the wrong username in the config file path if it is present.
    ///
    /// This function adjusts the configuration file paths to make them
    /// valid and usable.
    /// It ensures that relative paths are converted to absolute paths
    /// and handles potential issues related to usernames in file paths.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error if any path adjustments fail.
    pub fn fixup_config(&mut self) -> Result<()> {
        self.configs.iter_mut().for_each(|config| {
            config.path = fix_path!(&config.path).to_string_lossy().to_string();
        });

        Ok(())
    }

    /// Save the current configuration to a local file.
    ///
    /// This method serializes the `DotConfig` structure into a human-readable
    /// RON (Rust Object Notation) format and writes it to the configuration
    /// file specified in the `CONFIG_PATH` mutex.
    ///
    /// The configuration file contains information about the dotconfig
    /// directory and the list of configuration files to sync.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error if any file operations fail.
    pub fn save_configs(&self) -> Result<()> {
        let ron_pretty = get_ron_formatter();

        let config = to_string_pretty(self, ron_pretty).context("Failed to serialize config")?;

        let config_path = CONFIG_PATH.lock().unwrap();
        println!("Saving config file to {:#?}", config_path.display());

        let mut file =
            fs::File::create(config_path.as_path()).context("Failed to create config file")?;

        file.write_all(config.as_bytes())
            .context("Failed to write to config file")?;

        Ok(())
    }

    /// Pull all configured files based on their metadata.
    ///
    /// This method iterates through the list of configured files and checks
    /// whether each file needs to be updated based on its metadata.
    ///
    /// If a configuration file requires an update, it updates the metadata
    /// in the config file and replaces the file with the latest version from
    /// the source specified in the `DotConfig` structure.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error if any synchronization operations fail.
    pub fn pull_updated_configs(&mut self) -> Result<()> {
        // iterate through all the configs
        self.configs.iter_mut().for_each(|dir| {
            // check if the config dir exists
            if !dir.path_exists() {
                // if the config dir does not exist, exit safely
                println!("Skipping {:#?} does not exist.", dir.name);
                return;
            }

            // check if the config needs to be updated
            if dir.check_update_metadata_required() {
                println!("Updating {:#?}.", dir.name);

                // update the metadata in the config file
                dir.update_metadata().expect("Failed to update config hash");

                if let DotconfigPath::Local(local_dotconfigs_path) = &self.dotconfigs_path {
                    // Replace the config file with the latest version
                    dir.pull_config(local_dotconfigs_path)
                        .expect("Failed to pull config");
                } else {
                    println!("Skipping dotconfigs_path does not exist.");
                }
            } else {
                // if the config does not need to be updated, skip the config
                println!("Skipping {:#?} already up-to date.", dir.name);
            }
        });

        Ok(())
    }

    /// Push Updatable configs back to their local destination in the system
    ///
    /// The `push_updated_configs` method is responsible for pushing all the
    /// updated configuration files to their specified destination paths.
    ///
    /// It iterates through the list of configured files and, for each
    /// configuration, checks if it needs to be updated based on its metadata.
    /// If an update is required, it pushes the updated configuration to its
    /// specified destination.
    ///
    /// This operation is essential when you want to synchronize your local
    /// configurations with a remote repository or directory specified in the
    /// `DotConfig` structure's `dotconfigs_path` field.
    ///
    /// # Errors
    ///
    /// This method may return an error if any file operations or the pushing
    /// process fail. Errors could include issues with file I/O, authentication,
    /// or network connectivity.
    ///
    /// # Note
    ///
    /// Before using this method, ensure that you have configured the
    /// `dotconfigs_path` field in your `DotConfig` instance to specify where
    /// you want to push the updated configurations. The method will use this
    /// destination to push the changes.
    ///
    /// Additionally, this method does not perform metadata checks and
    /// forcefully pushes all updated configurations, overwriting existing files
    /// if necessary.
    ///
    /// This method is often used in conjunction with `pull_updated_configs` to
    /// maintain a consistent state between local and remote configurations.
    ///
    /// If the `dotconfigs_path` field specifies a local directory, the method
    /// copies the updated configurations from your local system to the
    /// specified directory. If it specifies a remote repository URL, the
    /// method may use Git or another version control system to push changes to
    /// the remote repository.
    ///
    /// For security reasons, be cautious when using this method in automated
    /// scripts, as it may overwrite existing files in the destination
    /// directory.
    pub fn push_updated_configs(&mut self) -> Result<()> {
        self.configs.par_iter().for_each(|dir| {
            if let DotconfigPath::Local(local_dotconfigs_path) = &self.dotconfigs_path {
                let dotconfigs_config_path = {
                    let mut path = fix_path!(local_dotconfigs_path).join(&dir.name);

                    if !path.exists() {
                        path = fix_path!(local_dotconfigs_path);
                        path.push(PathBuf::from(&dir.path).file_name().unwrap());
                    }

                    path
                };

                let local_config_hash = dir
                    .metadata_digest()
                    .expect("Failed to get metadata digest");

                let mut dotconfigs_hash: Option<String> = None;
                if dotconfigs_config_path.is_file() {
                    dotconfigs_hash =
                        hasher::get_file_hash(&dotconfigs_config_path, &mut Sha1::new())
                            .unwrap()
                            .into();
                } else if dotconfigs_config_path.is_dir() {
                    dotconfigs_hash =
                        hasher::get_complete_dir_hash(&dotconfigs_config_path, &mut Sha1::new())
                            .unwrap()
                            .into();
                }

                if dotconfigs_hash.is_none() {
                    println!("Skipping {:#?} does not exist.", dotconfigs_config_path);
                    return;
                }

                if dotconfigs_hash.unwrap().ne(&local_config_hash) {
                    println!("Updating {:#?}.", dir.name);

                    dir.push_config(&dotconfigs_config_path)
                        .expect("Failed to push the config");
                } else {
                    println!("Skipping {:#?} already up-to date.", dir.name);
                }
            }
        });

        Ok(())
    }

    /// Forcefully pull the latest versions of all configured files from the
    /// source.
    ///
    /// This method iterates through the list of configured files and pulls
    /// the latest versions of each file from the source specified in the
    /// `DotConfig` structure.
    ///
    /// It does not perform metadata checks and forcefully updates all
    /// configured files.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error if any file operations
    /// fail during the pull operation.
    pub fn force_pull_configs(&self) -> Result<()> {
        self.configs.par_iter().for_each(|dir| {
            if let DotconfigPath::Local(local_dotconfigs_path) = &self.dotconfigs_path {
                println!("Force pulling {:#?}.", dir.name);

                dir.pull_config(local_dotconfigs_path)
                    .expect("Failed to force pull the config");
            } else {
                println!("Skipping dotconfigs_path does not exist.");
            }
        });

        Ok(())
    }

    /// Forcefully push all the configured files to their specified destinations.
    ///
    /// This method iterates through the list of configured files and
    /// forcefully pushes each file to its specified destination path as
    /// defined in the `DotConfig` structure.
    ///
    /// It does not perform metadata checks and forcefully updates all
    /// configured files, overwriting existing files if necessary.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error if any file operations fail
    /// during the push operation.
    pub fn force_push_configs(&self) -> Result<()> {
        self.configs.par_iter().for_each(|dir| {
            if let DotconfigPath::Local(local_dotconfigs_path) = &self.dotconfigs_path {
                let dotconfigs_config_path = {
                    let mut path = fix_path!(local_dotconfigs_path).join(&dir.name);

                    if !path.exists() {
                        path = fix_path!(local_dotconfigs_path);
                        path.push(PathBuf::from(&dir.path).file_name().unwrap());
                    }

                    path
                };

                println!("Force pushing {:#?}.", dir.name);

                dir.push_config(&dotconfigs_config_path)
                    .expect("Failed to force push the config");
            } else {
                println!("Skipping dotconfigs path does not exist.");
            }
        });

        Ok(())
    }

    /// Remove metadata from all configured files within the `DotConfig` structure.
    ///
    /// This method iterates through the list of configured files and removes
    /// the metadata associated with each file. Specifically, it clears the
    /// hash and configuration type information.
    ///
    /// This operation is useful when the user wants to update the
    /// configuration files with the latest versions without updating
    /// their hashes or types.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error if clearing the metadata fails
    /// for any configured file.
    pub fn clean_metadata_from_configs(&mut self) -> Result<()> {
        self.configs.iter_mut().for_each(|dir| {
            dir.hash = None;
            dir.conf_type = None;
        });

        println!("Metadata removed from the config file.");
        Ok(())
    }

    /// Clean all files and directories in the dotconfig directory except the
    /// .git folder.
    ///
    /// This method recursively iterates over all files and directories within
    /// the dotconfig directory (specified in `dotconfigs_path`). It deletes
    /// all files and directories except for the `.git` folder,
    /// which is typically used for version control.
    ///
    /// This operation is useful when the user wants to perform maintenance or
    /// replace existing configurations in the dotconfig directory.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error if any file or directory
    /// removal fails.
    pub fn clean_dotconfigs_dir(&self) -> Result<()> {
        let mut path: Option<PathBuf> = None;
        if let DotconfigPath::Local(local_dotconfigs_path) = &self.dotconfigs_path {
            path = Some(fix_path!(local_dotconfigs_path));
        }
        println!("Cleaning all the configs inside {path:#?}");

        // iterate over all the files and directories inside the dotconfigs folder
        walkdir::WalkDir::new(path.as_ref().unwrap())
            .into_iter()
            .filter_map(|e| e.ok())
            .for_each(|e| {
                // skip the path itself and the .git folder
                if e.path().eq(path.as_ref().unwrap())
                    || e.path().to_string_lossy().contains(".git")
                {
                    return;
                }

                // remove the file or directory depending on the type
                if e.file_type().is_dir() {
                    std::fs::remove_dir_all(e.path()).expect("Failed to remove directory");
                } else {
                    std::fs::remove_file(e.path()).expect("Failed to remove file");
                }
            });

        Ok(())
    }

    /// Add a new configuration to the `DotConfig` structure.
    ///
    /// This method adds a new configuration to the `DotConfig` structure.
    /// It creates a new `Config` struct with the specified name and path and
    /// appends it to the list of configurations. It also checks if a
    /// configuration with the same name already exists to prevent duplicates.
    ///
    /// # Arguments
    ///
    /// * `name` - A reference to a String representing the name of the
    /// new configuration.
    /// * `path` - A reference to a Path representing the path of the
    /// new configuration.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error if the addition fails due to
    /// a duplicate name or other issues.
    pub fn add_config(&mut self, name: &String, path: PathBuf) -> Result<()> {
        self.configs
            .par_iter()
            .any(|dir| &dir.name == name)
            .then(|| {
                println!("Config with name {name:#?} already exists.");
                std::process::exit(1);
            });

        let mut conf_type = None;
        if path.is_dir() {
            conf_type = Some(ConfType::Dir);
        } else if path.is_file() {
            conf_type = Some(ConfType::File);
        }

        self.configs.push(Config::new(
            name.to_string(),
            path.to_string_lossy().to_string(),
            None,
            conf_type,
        ));

        Ok(())
    }

    /// Create a new `DotConfig` instance with default template.
    ///
    /// This method constructs a new `DotConfig` structure with default
    /// settings. It initializes the `dotconfigs_path` with the default
    /// dotfiles directory path and includes a single default
    /// configuration in the `configs` vector.
    ///
    /// # Returns
    ///
    /// A `DotConfig` struct with default settings.
    /// Create a new dotconfig file with default template
    pub fn new() -> Self {
        DotConfig::default()
    }

    /// Edit the `sync-dotfiles` configuration file.
    ///
    /// This method opens the `sync-dotfiles` configuration file in the
    /// editor specified by the `EDITOR` environment variable.
    /// If the `EDITOR` environment variable is not set, it will open the
    /// `sync-dotfiles` configuration file in the vim editor.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error if the editor fails to open.
    pub fn edit_config_file(&self) -> Result<()> {
        let editor: String = std::env::var("EDITOR").unwrap_or("vim".into());

        process::Command::new(editor)
            .arg(CONFIG_PATH.lock().unwrap().as_path())
            .status()
            .context("Failed to open the editor")?;

        Ok(())
    }
}

/// Display implementation for DotConfig.
///
/// This implementation allows you to print a human-readable representation
/// of a `DotConfig` instance.
///
/// # Example
///
/// ```rust
/// use sync_dotfiles_rs::dotconfig::DotConfig;
///
/// let dotconfig = DotConfig::new();
/// println!("{}", dotconfig);
/// ```
impl fmt::Display for DotConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "DotConfig {{")?;
        writeln!(f, "    dotconfigs_path: {},", self.dotconfigs_path)?;
        writeln!(f, "    configs: [")?;
        self.configs.iter().for_each(|config| {
            writeln!(f, "        {config},").expect("Failed to display config");
        });
        writeln!(f, "    ],")?;
        writeln!(f, "}}")
    }
}

/// Display implementation for DotconfigPath.
///
/// This implementation allows you to print a human-readable representation
/// of a `DotconfigPath` instance.
///
/// # Example
///
/// ```rust
/// use sync_dotfiles_rs::dotconfig::DotconfigPath;
///
/// let dotconfig_path = DotconfigPath::Local("path/to/dotconfigs".to_string());
/// println!("{}", dotconfig_path);
/// ```
impl fmt::Display for DotconfigPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DotconfigPath::Local(local_dotconfigs_path) => {
                write!(f, "{local_dotconfigs_path}")
            }
            DotconfigPath::Github(remote_dotconfigs_path) => {
                write!(f, "{remote_dotconfigs_path}")
            }
        }
    }
}

/// Debug implementation for DotconfigPath.
///
/// This implementation allows you to print a human-readable representation
/// of a `DotconfigPath` instance.
///
/// # Example
///
/// ```rust
/// use sync_dotfiles_rs::dotconfig::DotconfigPath;
///
/// let dotconfig_path = DotconfigPath::Github(
///         "https://github.com/user/repo".to_string());
/// println!("{:?}", dotconfig_path);
/// ```
impl fmt::Debug for DotconfigPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DotconfigPath::Local(local_dotconfigs_path) => {
                write!(f, "{local_dotconfigs_path}")
            }
            DotconfigPath::Github(remote_dotconfigs_path) => {
                write!(f, "{remote_dotconfigs_path}")
            }
        }
    }
}

/// Default implementation for DotConfig.
///
/// This implementation creates a new `DotConfig` instance with default
/// settings.
impl Default for DotConfig {
    fn default() -> Self {
        DotConfig {
            dotconfigs_path: DotconfigPath::Local(String::from("~/dotfiles")),
            configs: vec![Config::default()],
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_exisiting_defconfig() {
        let existing_dotconfig =
            DotConfig::parse_dotconfig(&Some(String::from("./examples/config.ron")));

        debug_assert!(
            existing_dotconfig.is_ok(),
            "Failed to parse the existing dotconfig file"
        );
    }
}
