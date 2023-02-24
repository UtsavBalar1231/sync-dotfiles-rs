use crate::config::Config;
use crate::*;
use std::io::Write;

#[derive(Serialize, Deserialize)]
pub struct DotConfig<'a> {
    pub dotconfigs_path: &'a str,
    pub configs: Vec<Config<'a>>,
}

static mut CONTENTS: String = String::new();
static mut CONFIG_PATH: String = String::new();

impl<'a> DotConfig<'a> {
    #[inline(always)]
    /// Parses the dotconfig file and returns a DotConfig struct
    pub fn parse_dotconfig(filepath: Option<&'a str>) -> Result<Self> {
        let mut file = Err(anyhow!(""));
        unsafe {
            // If the user has specified a config file path
            // Try to find the config file in the specified path
            if let Some(path) = filepath {
                // Fix the path if it is a tilde path
                if path.fix_path().is_some() {
                    // Set the config path to the path of the config file after fixing the path as provided by the user
                    CONFIG_PATH = path.fix_path().unwrap().to_string_lossy().to_string();
                } else {
                    // Set the config path to the path of the config file
                    CONFIG_PATH = PathBuf::from_str(path)
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                }
            } else {
                // Try to find the config file in the $HOME/.config/sync-dotfiles directory first
                let path = home_dir().unwrap().join(".config/sync-dotfiles/config.ron");
                file = fs::File::open(&path)
                    .context("Failed to open config file from current directory");

                // If the config file is found in the $HOME/.config/sync-dotfiles directory
                // Set the config path to the path of the config file
                if file.is_ok() {
                    CONFIG_PATH = path.to_string_lossy().to_string();
                // If the config file is not found in the $HOME/.config/sync-dotfiles directory
                // Try to find the config file in the current directory
                } else {
                    CONFIG_PATH = PathBuf::from_str("config.ron")
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                }
            }

            if CONFIG_PATH != "config.ron" {
                file = fs::File::open(&CONFIG_PATH)
                    .context("Failed to open config file from current directory");
            }

            if file.is_err() {
                return Err(anyhow!("Failed to open config file"));
            }

            file?.read_to_string(&mut CONTENTS)?;

            let config = Options::default()
                .with_default_extension(Extensions::IMPLICIT_SOME)
                .from_str(&CONTENTS)
                .context("Failed to parse config file")?;

            Ok(config)
        }
    }

    /// Create a new DotConfig struct from the dotconfig directory
    #[inline(always)]
    fn from<'b>(path: &'b str) -> Self
    where
        'b: 'a,
    {
        DotConfig {
            dotconfigs_path: path,
            configs: Vec::new(),
        }
    }

    /// Save the config files to disk
    #[inline(always)]
    pub fn save_configs(&self) -> Result<()> {
        let ron_pretty = PrettyConfig::new()
            .depth_limit(2)
            .extensions(Extensions::IMPLICIT_SOME);

        let config = to_string_pretty(self, ron_pretty).context("Failed to serialize config")?;

        unsafe {
            println!("Saving config file to {CONFIG_PATH}");

            let mut file =
                fs::File::create(&CONFIG_PATH).context("Failed to create config file")?;
            file.write_all(config.as_bytes())
                .context("Failed to write to config file")?;
        }

        Ok(())
    }

    /// Update all the configs mentioned in the config file
    #[inline(always)]
    pub fn sync_configs(&self) -> Result<Self> {
        let mut new_dotconfig = DotConfig::from(self.dotconfigs_path);

        self.configs.iter().for_each(|dir| {
            if dir.check_update_metadata_required().is_ok() {
                println!("Updating {}.", dir.name);
                let new_hash = dir
                    .metadata_digest()
                    .expect("Failed to get metadata digest");

                new_dotconfig
                    .configs
                    .push(Config::new(dir.name, dir.path, Some(new_hash)));

                dir.pull_config(self.dotconfigs_path)
                    .expect("Failed to pull config");
            } else {
                println!("Skipping {:?} already up-to date.", dir.name);
                new_dotconfig
                    .configs
                    .push(Config::new(dir.name, dir.path, dir.hash.clone()));
            }
        });

        Ok(new_dotconfig)
    }

    /// Force pull all the configs mentioned in the config file from the dotconfig directory
    #[inline(always)]
    pub fn force_pull_configs(&self) -> Result<()> {
        self.configs.iter().for_each(|dir| {
            println!("Force pulling {}.", dir.name);
            dir.pull_config(self.dotconfigs_path)
                .expect("Failed to force pull the config");
        });

        Ok(())
    }

    /// Force push all the configs mentioned in the config file from the dotconfig directory,
    /// To the dotconfig directory
    #[inline(always)]
    pub fn force_push_configs(&self) -> Result<()> {
        self.configs.iter().for_each(|dir| {
            println!("Force pushing {}.", dir.name);
            dir.push_config(self.dotconfigs_path)
                .expect("Failed to force push the config");
        });

        Ok(())
    }

    /// Remove hash from config file
    #[inline(always)]
    pub fn clean_hash_from_configs(&self) -> Result<DotConfig> {
        let mut new_dotconfig = DotConfig::from(self.dotconfigs_path);

        self.configs.iter().for_each(|dir| {
            new_dotconfig
                .configs
                .push(Config::new(dir.name, dir.path, None));
        });

        println!("Hashes removed from config file.");
        Ok(new_dotconfig)
    }

    /// Add a new config inside the dot config file
    #[inline(always)]
    pub fn add_config(&self, name: &'a str, path: &'a str) -> Result<Self> {
        let mut new_dotconfig = DotConfig::from(self.dotconfigs_path);

        self.configs.iter().for_each(|dir| {
            new_dotconfig
                .configs
                .push(Config::new(dir.name, dir.path, dir.hash.clone()));
        });

        new_dotconfig.configs.push(Config::new(name, path, None));

        Ok(new_dotconfig)
    }
}

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

impl Default for DotConfig<'_> {
    fn default() -> Self {
        DotConfig {
            dotconfigs_path: "/* Path to your dotconfigs folder or repository */",
            configs: vec![Config::default()],
        }
    }
}
