use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sync-dotconfigs")]
#[command(author = "Utsav Balar")]
#[command(version, about, long_about)]
pub struct Args {
    /// Force push the configs listed in config to the local configs directory
    #[clap(short = 'F', long = "fpush")]
    pub force_push: bool,

    /// Force pull the local configs inside the mentioned dotconfigs directory
    #[clap(short = 'f', long = "fpull")]
    pub force_pull: bool,

    /// Update the config file with new files
    #[clap(short, long)]
    pub update: bool,

    /// Clear the metadata of config entries in the config file
    #[clap(short = 'x', long = "clear")]
    pub clear_metadata: bool,

    /// Prints the new config file
    #[clap(short, long)]
    pub new: bool,

    /// Subcommand to add or clean config entries
    #[clap(subcommand)]
    pub subcommand: Option<SubCommandArgs>,

    /// Print the contents of the config file
    #[clap(short, long)]
    pub print: bool,

    /// The path of the config file (default: current_dir/config.ron)
    #[clap(short, long = "cpath")]
    pub config_path: Option<String>,
}

#[derive(Subcommand)]
pub enum SubCommandArgs {
    /// Adds a new config entry to your exisiting config file
    Add {
        /// The name of the config entry
        #[arg(short, long)]
        name: String,

        /// The path of the config entry
        #[arg(short, long)]
        path: String,
    },

    /// Clean all the config directories from the dotconfigs path specified in the config file
    #[clap(name = "clean")]
    CleanDirAll,
}

pub fn get_env_args() -> Args {
    Args::parse()
}
