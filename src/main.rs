use sync_dotfiles_rs::*;
mod args;
use args::{get_env_args, SubCommandArgs};

fn main() -> Result<()> {
    let args = get_env_args();
    let dotconfig = dotconfig::DotConfig::parse_dotconfig(args.config_path)
        .context("Failed to parse config file")?;

    if args.force {
        dotconfig
            .force_sync_configs()
            .context("Failed to force sync configs")?;
    }

    if args.update {
        let modified_dotconfig = dotconfig.sync_configs().context("Failed to sync configs")?;
        modified_dotconfig
            .save_configs()
            .context("Failed to save config file")?;
    }

    if args.clean_hash {
        let modified_dotconfig = dotconfig
            .clean_hash_from_configs()
            .context("Failed to clean hashes from the configs")?;
        modified_dotconfig
            .save_configs()
            .context("Failed to save config file")?;
    }

    if args.new {
        println!(
            r#"
The default config file is as follows:

#![enable(implicit_some)]"#
        );
        let config = Options::default()
            .with_default_extension(Extensions::IMPLICIT_SOME)
            .to_string_pretty(&dotconfig::DotConfig::default(), PrettyConfig::default())?;
        println!("{config}");
    }

    match &args.subcommand {
        Some(SubCommandArgs::Add { name, path }) => {
            let modified_dotconfig = dotconfig
                .add_config(name, path)
                .context("Failed to insert config")?
                .sync_configs()
                .context("Failed to sync the newly inserted config")?;
            modified_dotconfig
                .save_configs()
                .context("Failed to save config file")?;
        }
        Some(SubCommandArgs::CleanDirAll) => {
            std::fs::remove_dir_all(dotconfig.dotconfigs_path)
                .context("Failed to remove all the config directories")?;
            println!(
                "Cleaned all the configs inside {}",
                dotconfig.dotconfigs_path
            );
        }
        None => {}
    }

    if args.print {
        println!("{dotconfig}");
    }

    Ok(())
}
