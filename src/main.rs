use std::env::args;
use sync_dotfiles_rs::dotconfig::DotConfig;
use sync_dotfiles_rs::*;

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
    let args = args().skip(1).collect::<Vec<_>>();
    let dotconfig = parse_dotconfig().context("Failed to parse config file")?;

    if args.is_empty() {
        println!("No arguments provided, doing nothing.");
        return Ok(());
    }

    match args[0].as_str() {
        "update" | "u" => {
            let modified_dotconfig = dotconfig.sync_configs()?;
            modified_dotconfig
                .save_configs()
                .context("Failed to save config file")?;
        }
        "force" | "f" => {
            dotconfig.force_sync_configs()?;
        }
        _ => {
            println!("Invalid argument provided, doing nothing.");
        }
    }

    Ok(())
}
