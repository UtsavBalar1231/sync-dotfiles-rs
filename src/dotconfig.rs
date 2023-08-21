use crate::*;
use crate::{config::ConfType, config::Config};
use rayon::prelude::*;
use std::io::Write;

/// Struct to store the contents of the config file (`config.ron`)
///
/// The contents of the config file is stored in a vector of Config structs.
/// The path of the dotconfig directory is stored in a str slice.
///
/// This struct implements the Serialize and Deserialize traits from serde
/// which are used to serialize and deserialize the struct to and from a string.
///
#[derive(Serialize, Deserialize)]
pub struct DotConfig<'a> {
    /// The path of the dotconfig directory will panic if the path is not a valid utf-8 string or empty
    pub dotconfigs_path: &'a str,
    /// The vector of Config structs which holds the contents of the config file
    pub configs: Vec<Config<'a>>,
}

/// To store the contents of the config file for use in other functions.
///
/// The contents is stored in a static variable so that it can be used by parse_dotconfig function.
/// And the contents can be used by other functions without having to deal with rust borrowing rules
///
static mut CONTENTS: String = String::new();

/// To store the path of the config file for use in other functions.
///
/// The path can be changed by the user using the --config-path flag.
/// Initially, the path is set to the path of the config file in the $HOME/.config/sync-dotfiles directory.
/// If the config file is not found in the $HOME/.config/sync-dotfiles directory,
/// the path is set to the path of the config file in the current directory.
///
static mut CONFIG_PATH: String = String::new();

impl<'a> DotConfig<'a> {
    #[inline(always)]
    /// Parses the dotconfig file and returns a DotConfig structure.
    ///
    /// The dotconfig file is the config file which contains the list of all the config files to be synced.
    /// It is a RON file (`config.ron`) which is a human readable version of the RUST data serialization format.
    ///
    /// The config file location can be specified by the user using the --cpath or -c flag.
    /// If the config file location is not specified by the user,
    /// the config file is searched in the $HOME/.config/sync-dotfiles directory.
    /// Else if the config file is not found in the $HOME/.config/sync-dotfiles directory,
    /// the config file is searched in the current directory.
    pub fn parse_dotconfig(filepath: &Option<String>) -> Result<Self> {
        unsafe {
            // If the user has specified a config file path
            // Try to find the config file in the specified path
            if let Some(path) = filepath {
                // Set the config path to the path of the config file after fixing
                // the path as provided by the user
                CONFIG_PATH = path
                    .fix_path()
                    .unwrap_or_else(|| PathBuf::from(path))
                    .into_os_string()
                    .into_string()
                    .unwrap();
            } else {
                // Try to find the config file in the $HOME/.config/sync-dotfiles directory first
                let path = home::home_dir()
                    .unwrap()
                    .join(".config/sync-dotfiles/config.ron");

                // If the config file is found in the $HOME/.config/sync-dotfiles directory
                // Set the config path to the path of the config file
                if fs::File::open(&path).is_ok() {
                    println!("Found config file in $HOME/.config/sync-dotfiles directory");
                    CONFIG_PATH = path.into_os_string().into_string().unwrap();
                // If the config file is not found in the $HOME/.config/sync-dotfiles directory
                // Try to find the config file in the current directory
                } else {
                    let local_config_path = PathBuf::from_str("config.ron")
                        .unwrap()
                        .into_os_string()
                        .into_string()
                        .unwrap();

                    if fs::File::open(&local_config_path).is_ok() {
                        println!("Found config file in current directory");
                        CONFIG_PATH = local_config_path;
                    } else {
                        return Err(anyhow!("Failed to open config file, No config found!"));
                    }
                }
            }

            let file = fs::File::open(&CONFIG_PATH)
                .context("Failed to open config file from current directory");

            file?.read_to_string(&mut CONTENTS)?;

            let config = Options::default()
                .with_default_extension(Extensions::IMPLICIT_SOME)
                .from_str(&CONTENTS)
                .context("Failed to parse config file")?;

            Ok(config)
        }
    }

    /// Fix the config file path if it is a relative path.
    /// Also fix the wrong username in the config file path if it is present.
    /// This will be useful when the config file is shared between multiple users.
    #[inline(always)]
    pub fn fixup_config(&mut self) -> Result<()> {
        self.configs.iter_mut().for_each(|config| {
            config.path = config
                .path
                .fix_path()
                .unwrap_or_else(|| PathBuf::from(&config.path))
                .to_string_lossy()
                .to_string();
        });

        Ok(())
    }

    /// Save the config files to local disk at the path either specified by the user or the default path.
    ///
    /// The default path is the path of the config file in the $HOME/.config/sync-dotfiles directory
    /// If the config file is not found in the $HOME/.config/sync-dotfiles directory,
    /// the default path is the path of the config file in the current directory.
    ///
    #[inline(always)]
    pub fn save_configs(&self) -> Result<()> {
        let ron_pretty = PrettyConfig::new()
            .depth_limit(2)
            .extensions(Extensions::IMPLICIT_SOME);

        let config = to_string_pretty(self, ron_pretty).context("Failed to serialize config")?;

        unsafe {
            println!("Saving config file to {CONFIG_PATH:#?}");

            let mut file =
                fs::File::create(&CONFIG_PATH).context("Failed to create config file")?;

            file.write_all(config.as_bytes())
                .context("Failed to write to config file")?;
        }

        Ok(())
    }

    /// Update all the configs mentioned in the config file.
    ///
    /// Start by iteratating through all the configs and check if the config needs to be updated.
    /// If the config needs to be updated, update the config hash in the config file and
    /// replace the config file with the latest version.
    /// Else if the config does not need to be updated, skip the config
    ///
    #[inline(always)]
    pub fn sync_configs(&mut self) -> Result<()> {
        // iterate through all the configs
        self.configs.iter_mut().for_each(|dir| {
            // check if the config dir exists
            if !dir.path_exists() {
                // if the config dir does not exist, exit safely
                println!("Skipping {:#?} does not exist.", dir.name);
                return;
            }

            // check if the config needs to be updated
            if dir.check_update_metadata_required().is_ok() {
                println!("Updating {:#?}.", dir.name);

                // update the metadata in the config file
                dir.update_metadata().expect("Failed to update config hash");

                // replace the config file with the latest version
                dir.pull_config(self.dotconfigs_path)
                    .expect("Failed to pull config");
            } else {
                // if the config does not need to be updated, skip the config
                println!("Skipping {:#?} already up-to date.", dir.name);
            }
        });

        Ok(())
    }

    /// Force pull all the configs mentioned in the config file from the path specified by the user
    /// Into the dotconfig (`config.ron`) file
    #[inline(always)]
    pub fn force_pull_configs(&self) -> Result<()> {
        self.configs.par_iter().for_each(|dir| {
            println!("Force pulling {:#?}.", dir.name);
            dir.pull_config(self.dotconfigs_path)
                .expect("Failed to force pull the config");
        });

        Ok(())
    }

    /// Force push all the configs mentioned in the config file from the dotconfig directory,
    /// To the user specified path for each config
    ///
    /// ```
    ///
    /// # Example:
    /// # cat config.ron
    ///
    /// #(implicit_some)
    /// (
    ///     dotconfigs_path: "/home/user/.dotconfig",
    ///     configs: [
    ///         (
    ///             name: "nvim",
    ///             path: "/home/user/.config/nvim",
    ///         )
    ///     ]
    /// )
    /// ```
    /// During the force push, the config file will be pushed to the path specified by the user
    /// i.e. /home/user/.config/nvim
    ///
    #[inline(always)]
    pub fn force_push_configs(&self) -> Result<()> {
        self.configs.par_iter().for_each(|dir| {
            println!("Force pushing {:#?}.", dir.name);
            dir.push_config(self.dotconfigs_path)
                .expect("Failed to force push the config");
        });

        Ok(())
    }

    /// Remove metadata from the config file and return a new dotconfig.
    ///
    /// This is useful when the user wants to update the config file with the latest version of the config files
    /// without updating the hashes.
    ///
    #[inline(always)]
    pub fn clean_metadata_from_configs(&mut self) -> Result<()> {
        self.configs.iter_mut().for_each(|dir| {
            dir.hash = None;
            dir.conf_type = None;
        });

        println!("Metadata removed from the config file.");
        Ok(())
    }

    /// Clean all the configs from dotconfig directory except the .git folder.
    ///
    /// This is useful when the user wants to remove all the configs from the dotconfig directory for maintenance
    /// or to remove all the configs from the dotconfig directory and add new configs.
    ///
    #[inline(always)]
    pub fn clean_dotconfigs_dir(&self) -> Result<()> {
        let path = self
            .dotconfigs_path
            .fix_path()
            .ok_or_else(|| PathBuf::from(self.dotconfigs_path))
            .expect("Failed to fix path");

        println!("Cleaning all the configs inside {path:#?}");

        // iterate over all the files and directories inside the dotconfigs folder
        walkdir::WalkDir::new(&path)
            .into_iter()
            .filter_map(|e| e.ok())
            .for_each(|e| {
                // skip the path itself and the .git folder
                if e.path() == path || e.path().to_string_lossy().contains(".git") {
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

    /// Adds a new config inside the config file and returns a new dotconfig.
    ///
    /// This is useful when the user wants to add a new config to the config file.
    /// Additionally checks if the config with the same name already exists.
    ///
    #[inline(always)]
    pub fn add_config(&mut self, name: &'a str, path: &'a std::path::Path) -> Result<()> {
        self.configs
            .par_iter()
            .any(|dir| dir.name == name)
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
            name,
            path.to_string_lossy().to_string(),
            None,
            conf_type,
        ));

        Ok(())
    }
}

/// Display implementation for DotConfig
/// This is useful when the user wants to print the DotConfig struct
impl std::fmt::Display for DotConfig<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

/// Default implementation for DotConfig
impl Default for DotConfig<'_> {
    fn default() -> Self {
        DotConfig {
            dotconfigs_path: "/* Path to your dotconfigs folder or repository */",
            configs: vec![Config::default()],
        }
    }
}
