use std::env::args;
use sync_dotfiles_rs::dotconfig::DotConfig;
use sync_dotfiles_rs::*;

const ABOUT: &str = "Sync dotfiles easily from a source directory to a destination directory as per your config file";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const USAGE: &str = r#"
Usage:
sync-dotfiles-rs [options]
    sync-dotfiles-rs (-v | --version)
    sync-dotfiles-rs (-V | --verbose)
    sync-dotfiles-rs (-h | --help)
"#;

fn help() {
    println!("{ABOUT}");
    println!("{USAGE}");
    println!(
        r#"options:
    sync-dotfiles-rs (-f | --force)
    sync-dotfiles-rs (-u | --update)
    sync-dotfiles-rs (-C | --clean-hash)
    sync-dotfiles-rs (-n | --new)
"#
    );
}

fn main() -> Result<()> {
    let args = args().skip(1).collect::<Vec<_>>();
    let dotconfig = DotConfig::parse_dotconfig().context("Failed to parse config file")?;

    if args.is_empty() {
        println!("No arguments provided, doing nothing.");
        return Ok(());
    }

    match args[0].as_str() {
        "-h" | "--help" => help(),
        "-v" | "--version" => println!("sync-dotfiles-rs v{VERSION}"),
        "-V" | "--verbose" => todo!("Enable verbose logging"),
        "-f" | "--force" => {
            dotconfig.force_sync_configs()?;
        }
        "-u" | "--update" => {
            let modified_dotconfig = dotconfig.sync_configs()?;
            modified_dotconfig
                .save_configs()
                .context("Failed to save config file")?;
        }
        "-C" | "--clean-hash" => {
            let modified_dotconfig = dotconfig.clean_hash_from_configs()?;
            modified_dotconfig
                .save_configs()
                .context("Failed to save config file")?;
        }
        "-n" | "--new" => {
            println!(
                r#"The default config file is as follows:

#![enable(implicit_some)]"#
            );
            let config = Options::default()
                .with_default_extension(Extensions::IMPLICIT_SOME)
                .to_string_pretty(&DotConfig::default(), PrettyConfig::default())?;
            println!("{config}");
        }
        _ => {
            println!("Invalid argument provided, doing nothing.");
        }
    }

    Ok(())
}
