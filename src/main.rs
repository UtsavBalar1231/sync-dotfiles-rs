use crate::*;
mod args;
use args::{get_env_args, SubCommandArgs};
use std::process::exit;

fn main() -> Result<()> {
    let args = get_env_args();

    let mut dotconfig = dotconfig::DotConfig::parse_dotconfig(args.config_path.as_deref())
        .context("Failed to parse config file")?;

    if args.force_push {
        dotconfig
            .force_push_configs()
            .context("Failed to force push configs")?;

        println!("Successfully force pushed the configs");

        exit(0);
    }

    if args.force_pull {
        dotconfig
            .clean_dotconfigs_dir()
            .context("Failed to clean all the configs inside the dotconfig directory")?;
        dotconfig
            .force_pull_configs()
            .context("Failed to force pull configs")?;

        println!("Successfully force pulled the configs");

        exit(0);
    }

    if args.update {
        dotconfig.sync_configs().context("Failed to sync configs")?;
        dotconfig
            .save_configs()
            .context("Failed to save config file")?;

        println!("Successfully updated the config file");

        exit(0);
    }

    if args.clear_metadata {
        dotconfig
            .clean_metadata_from_configs()
            .context("Failed to clear the metadata from the config file")?;

        dotconfig
            .save_configs()
            .context("Failed to save config file")?;

        println!("Successfully cleared the metadata from the config file");

        exit(0);
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

        exit(0);
    }

    match &args.subcommand {
        Some(SubCommandArgs::Add { name, path }) => {
            let path = path.fix_path().unwrap_or_else(|| PathBuf::from(path));
            dotconfig
                .add_config(name, path.as_path())
                .context("Failed to insert config")?;

            dotconfig
                .sync_configs()
                .context("Failed to sync the newly inserted config")?;

            dotconfig
                .save_configs()
                .context("Failed to save config file")?;

            println!("Successfully added {name:?} to the config file");

            exit(0);
        }

        Some(SubCommandArgs::CleanDirAll) => {
            dotconfig
                .clean_dotconfigs_dir()
                .context("Failed to clean all the configs inside the dotconfig directory")?;

            println!(
                "Successfully cleaned all the configs inside {:?}",
                dotconfig.dotconfigs_path
            );

            exit(0);
        }

        None => {}
    }

    if args.print {
        println!("{dotconfig}");

        exit(0);
    }

    Ok(())
}
