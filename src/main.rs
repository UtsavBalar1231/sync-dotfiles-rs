use anyhow::{Context, Result};
use ron::{
    extensions::Extensions,
    ser::{to_string_pretty, PrettyConfig},
    Options,
};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, io::Write, time::SystemTime};

#[derive(Serialize, Deserialize, Debug)]
struct DotConfig {
    config: Vec<ConfigDir>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ConfigDir {
    name: String,
    path: String,
    last_modified: Option<SystemTime>,
}

impl DotConfig {
    fn new() -> Self {
        Self { config: Vec::new() }
    }
}

impl ConfigDir {
    #[allow(dead_code)]
    fn new(name: String, path: String) -> Self {
        Self {
            name,
            path,
            last_modified: None,
        }
    }
}

fn parse_dotconfig() -> Result<DotConfig> {
    let mut file = File::open("config.ron")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let ron = Options::default().with_default_extension(Extensions::IMPLICIT_SOME);
    let config: DotConfig = ron
        .from_str(&contents)
        .context("Failed to parse config file")?;

    Ok(config)
}

pub fn fix_path(path: &str) -> String {
    if !path.starts_with('~') {
        return String::from(path);
    }
    path.replace(
        '~',
        home::home_dir()
            .expect("Failed to get home directory")
            .as_path()
            .to_str()
            .unwrap(),
    )
}

fn check_metadata(path: &str) -> Result<SystemTime> {
    // TODO: Optimize this to not have to create a new string
    let path = fix_path(path);
    let file = File::open(path).context("Failed to open file")?;

    let modified = file.metadata()?.modified()?;
    Ok(modified)
}

fn save_config(config: &DotConfig) -> Result<()> {
    let ron_pretty = PrettyConfig::new()
        .depth_limit(2)
        .extensions(Extensions::IMPLICIT_SOME);

    let config = to_string_pretty(config, ron_pretty)?;

    let mut file = File::create("config.ron")?;
    file.write_all(config.as_bytes())?;

    Ok(())
}

fn create_modified_dotconfig(config: &DotConfig) -> Result<DotConfig> {
    let mut new_config = DotConfig::new();

    for dir in &config.config {
        let modified = check_metadata(&dir.path)?;
        new_config.config.push(ConfigDir {
            name: dir.name.clone(),
            path: dir.path.clone(),
            last_modified: Some(modified),
        });
    }

    Ok(new_config)
}

fn main() -> Result<()> {
    let dotconfig = parse_dotconfig().context("Failed to parse config file")?;
    // println!("{:#?}", dotconfig);

    let modified_dotconfig = create_modified_dotconfig(&dotconfig)?;
    save_config(&modified_dotconfig).context("Failed to save config file")?;

    Ok(())
}
