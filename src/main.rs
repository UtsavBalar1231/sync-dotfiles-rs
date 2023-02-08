use sync_dotfiles_rs::dotconfig::*;

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

fn main() -> Result<()> {
    let dotconfig = parse_dotconfig().context("Failed to parse config file")?;

    let modified_dotconfig = dotconfig.update_dotconfig()?;
    modified_dotconfig
        .save_config()
        .context("Failed to save config file")?;

    Ok(())
}
