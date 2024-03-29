use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sync-dotconfigs")]
#[command(author = "Utsav Balar")]
#[command(version, about, long_about)]
pub struct SyncDotfilesArgs {
    /// Provide custom path to the config file (default: ${pwd}/config.ron)
    #[clap(short, long)]
    pub config_path: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Force push configs from dotconfigs directory into your local system
    #[clap(short_flag = 'f')]
    ForcePush,

    /// Force pull configs from your local system into the dotconfigs directory
    #[clap(short_flag = 'F')]
    ForcePull,

    /// Update your dotconfigs directory with the latest configs
    #[clap(short_flag = 'u')]
    Pull,

    /// Update your local system configs with the configs from the dotconfigs directory
    #[clap(short_flag = 'U')]
    Push,

    /// Clear the metadata of config entries in the sync-dotfiles config
    #[clap(short_flag = 'x')]
    ClearMetadata,

    /// Prints a new sync-dotfiles configuration
    #[clap(name = "new", short_flag = 'n')]
    PrintNew,

    /// Prints the currently used sync-dotfiles config file
    #[clap(name = "printconf", short_flag = 'P')]
    PrintConfig,

    /// Fix your sync-dotfiles config file for any errors
    #[clap(short_flag = 'z')]
    FixConfig,

    /// Adds a new config entry to your exisiting sync-dotfiles config
    #[clap(short_flag = 'a')]
    #[command(arg_required_else_help = true)]
    Add(AddArgs),

    /// Clean all the config directories from your specified dotconfigs path
    #[clap(short_flag = 'C')]
    Clean,

    /// Edit the sync-dotfiles config file
    #[clap(short_flag = 'e')]
    Edit,
}

#[derive(Args)]
pub struct AddArgs {
    /// The name of the config entry
    #[arg(short = 'n', long)]
    pub name: String,
    /// The path to the config entry
    #[arg(short = 'p', long)]
    pub path: String,
}

pub fn get_env_args() -> SyncDotfilesArgs {
    SyncDotfilesArgs::parse()
}
