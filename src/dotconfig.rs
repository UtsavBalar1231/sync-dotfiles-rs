use crate::config::Config;
use crate::*;
use std::io::Write;

#[derive(Serialize, Deserialize)]
pub struct DotConfig<'a> {
    pub dotconfigs_path: &'a str,
    pub configs: Vec<Config<'a>>,
}

static mut CONTENTS: String = String::new();

impl<'a> DotConfig<'a> {
    #[inline]
    /// Parses the dotconfig file and returns a DotConfig struct
    pub fn parse_dotconfig(filepath: Option<String>) -> Result<Self> {
        let mut file = match filepath {
            Some(path) => {
                File::open(path.fix_path()?).context("Failed to open config file from {path}")?
            }
            None => File::open("config.ron")
                .context("Failed to open config file from current directory")
                .or_else(|_| File::open(home_dir().unwrap().join("config.ron")))
                .context("Failed to open config file from home directory")?,
        };

        unsafe {
            file.read_to_string(&mut CONTENTS)?;
            let config = Options::default()
                .with_default_extension(Extensions::IMPLICIT_SOME)
                .from_str(&CONTENTS)
                .context("Failed to parse config file")?;

            Ok(config)
        }
    }

    /// Create a new DotConfig struct from the dotconfig directory
    #[inline]
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
    #[inline]
    pub fn save_configs(&self) -> Result<()> {
        let ron_pretty = PrettyConfig::new()
            .depth_limit(2)
            .extensions(Extensions::IMPLICIT_SOME);

        let config = to_string_pretty(self, ron_pretty).context("Failed to serialize config")?;

        let mut file = File::create("config.ron").context("Failed to create config file")?;
        file.write_all(config.as_bytes())
            .context("Failed to write to config file")?;

        Ok(())
    }

    /// Update all the configs mentioned in the config file
    /// TODO: Optimize this, Reduce the cloning
    #[inline]
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

                dir.sync_config(self.dotconfigs_path)
                    .expect("Failed to sync config");
            } else {
                println!("Skipping {:?} already up-to date.", dir.name);
                new_dotconfig
                    .configs
                    .push(Config::new(dir.name, dir.path, dir.hash.clone()));
            }
        });

        Ok(new_dotconfig)
    }

    /// Force update all the configs mentioned in the config file
    #[inline]
    pub fn force_sync_configs(&self) -> Result<()> {
        self.configs.iter().for_each(|dir| {
            println!("Force Updating {}.", dir.name);
            dir.sync_config(self.dotconfigs_path)
                .expect("Failed to sync config");
        });

        Ok(())
    }

    /// Remove hash from config file
    #[inline]
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
    #[inline]
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
