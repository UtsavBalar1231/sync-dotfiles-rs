use sync_dotfiles_rs::*;
mod args;
use args::{get_env_args, SubCommandArgs};

fn main() -> Result<()> {
    let args = get_env_args();
    let dotconfig = dotconfig::DotConfig::parse_dotconfig(args.config_path.as_deref())
        .context("Failed to parse config file")?;

    if args.force_push {
        dotconfig
            .force_push_configs()
            .context("Failed to force push configs")?;

        println!("Successfully force pushed the configs");
    }

    if args.force_pull {
        dotconfig
            .force_pull_configs()
            .context("Failed to force pull configs")?;

        println!("Successfully force pulled the configs");
    }

    if args.update {
        let modified_dotconfig = dotconfig.sync_configs().context("Failed to sync configs")?;
        modified_dotconfig
            .save_configs()
            .context("Failed to save config file")?;

        println!("Successfully updated the config file");
    }

    if args.clean_hash {
        let modified_dotconfig = dotconfig
            .clean_hash_from_configs()
            .context("Failed to clean hashes from the configs")?;

        modified_dotconfig
            .save_configs()
            .context("Failed to save config file")?;

        println!("Successfully cleaned hashes from the config file");
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

            println!("Successfully added {name:?} to the config file");
        }
        Some(SubCommandArgs::CleanDirAll) => {
            let path = dotconfig
                .dotconfigs_path
                .fix_path()
                .ok_or_else(|| PathBuf::from(dotconfig.dotconfigs_path))
                .expect("Failed to fix path");

            println!("Cleaning all the configs inside {path:?}");

            // iterate over all the files and directories inside the dotconfigs folder
            walkdir::WalkDir::new(&path)
                .into_iter()
                .filter_map(|e| e.ok())
                .for_each(|e| {
                    // skip the path itself and the .git folder
                    if e.path() == path || e.path().to_string_lossy().contains(".git") {
                        return;
                    }

                    // remove the file or directory depending on the type
                    if e.file_type().is_dir() {
                        std::fs::remove_dir_all(e.path()).expect("Failed to remove directory");
                    } else {
                        std::fs::remove_file(e.path()).expect("Failed to remove file");
                    }
                });
            println!("Successfully cleaned all the configs inside {path:?}");
        }
        None => {}
    }

    if args.print {
        println!("{dotconfig}");
    }

    Ok(())
}
