use anyhow::{Context, Result};
pub use ron::{
    extensions::Extensions,
    ser::{to_string_pretty, PrettyConfig},
    Options,
};
use std::{path::PathBuf, process};
use sync_dotfiles_rs::{
    dotconfig::DotConfig,
    utils::{self, FixPath},
};
mod args;
use args::{get_env_args, Commands::*};

fn main() -> Result<()> {
    let args = get_env_args();
    let mut dotconfig;

    dotconfig = DotConfig::parse_dotconfig(&args.config_path)
        .context("Failed to parse custom config file")?;

    match args.command {
        Add(args::AddArgs { name, path }) => {
            let path = path.fix_path().unwrap_or(PathBuf::from(path));
            dotconfig
                .add_config(&name, path)
                .context("Failed to insert config")?;

            dotconfig
                .pull_updated_configs()
                .context("Failed to sync the newly inserted config")?;

            dotconfig
                .save_configs()
                .context("Failed to save config file")?;

            println!("Successfully added {name:?} to the config file");

            process::exit(0);
        }

        Clean => {
            dotconfig
                .clean_dotconfigs_dir()
                .context("Failed to clean all the configs inside the dotconfig directory")?;

            println!(
                "Successfully cleaned all the configs inside {:?}",
                dotconfig.dotconfigs_path
            );

            process::exit(0);
        }

        ClearMetadata => {
            dotconfig
                .clean_metadata_from_configs()
                .context("Failed to clear the metadata from the config file")?;

            dotconfig
                .save_configs()
                .context("Failed to save config file")?;

            println!("Successfully cleared the metadata from the config file");

            process::exit(0);
        }

        FixConfig => {
            dotconfig
                .fixup_config()
                .context("Failed to fixup the config file")?;

            dotconfig
                .save_configs()
                .context("Failed to save config file")?;

            println!("Successfully fixed up the config file");

            process::exit(0);
        }

        ForcePull => {
            dotconfig
                .clean_dotconfigs_dir()
                .context("Failed to clean all the configs inside the dotconfig directory")?;

            dotconfig
                .force_pull_configs()
                .context("Failed to force pull configs")?;

            println!("Successfully force pulled the configs");

            process::exit(0);
        }

        ForcePush => {
            dotconfig
                .force_push_configs()
                .context("Failed to force push configs")?;

            println!("Successfully force pushed the configs");

            process::exit(0);
        }

        PrintConfig => {
            println!("{dotconfig}");

            process::exit(0);
        }

        PrintNew => {
            let config = Options::default()
                .with_default_extension(Extensions::IMPLICIT_SOME)
                .to_string_pretty(&DotConfig::new(), utils::get_ron_formatter())
                .context("Failed to print the new config")?;

            println!("{config}");

            process::exit(0);
        }

        Pull => {
            dotconfig
                .pull_updated_configs()
                .context("Failed to pull updated configs")?;

            dotconfig
                .save_configs()
                .context("Failed to save config file")?;

            println!("Successfully updated the config file");

            process::exit(0);
        }

        Push => {
            dotconfig
                .push_updated_configs()
                .context("Failed to push configs")?;

            println!("Successfully pushed the updated configs");

            process::exit(0);
        }

        Edit => {
            dotconfig
                .edit_config_file()
                .context("Failed to edit config file")?;

            process::exit(0);
        }
    }
}
