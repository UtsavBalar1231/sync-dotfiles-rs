use crate::config::Config;
use crate::*;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug)]
pub struct DotConfig {
    pub dotconfigs_path: String,
    pub configs: Vec<Config>,
}

impl DotConfig {
    fn from(path: &String) -> Self {
        DotConfig {
            dotconfigs_path: path.to_string(),
            configs: Vec::new(),
        }
    }

    /// Save the config files to disk
    pub fn save_configs(&self) -> Result<()> {
        let ron_pretty = PrettyConfig::new()
            .depth_limit(2)
            .extensions(Extensions::IMPLICIT_SOME);

        let config = to_string_pretty(self, ron_pretty).context("Failed to serialize config")?;

        let mut file = File::create("config.ron")?;
        file.write_all(config.as_bytes())?;

        Ok(())
    }

    /// Update all the configs mentioned in the config file
    /// TODO: Optimize this, Reduce the cloning
    pub fn sync_configs(&self) -> Result<Self> {
        let mut new_dotconfig = DotConfig::from(&self.dotconfigs_path);

        for dir in &self.configs {
            if dir.check_update_metadata_required().is_ok() {
                println!("Updating {}.", dir.name);
                let new_hash = dir.metadata_digest()?;

                new_dotconfig
                    .configs
                    .push(Config::new(&dir.name, &dir.path, Some(new_hash)));

                dir.sync_config(&self.dotconfigs_path)?;
            } else {
                println!("Skipping {:?} already up-to date.", dir.name);
                new_dotconfig
                    .configs
                    .push(Config::new(&dir.name, &dir.path, dir.hash.clone()));
            }
        }

        Ok(new_dotconfig)
    }

    /// Force update all the configs mentioned in the config file
    pub fn force_sync_configs(&self) -> Result<()> {
        for dir in &self.configs {
            dir.sync_config(&self.dotconfigs_path)?;
        }

        Ok(())
    }
}
