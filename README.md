# sync-dotfiles-rs: Easily sync dotfiles across machines

## Features

- Update your dotconfigs all at once based on the configuration file.
- Parse the config file and update the configs based on the hash of the config.
- Add new configs to the config file and update the configs.
- Parallelize the update process to speed up the process.
- Easy to configure by yourself, simply modify the `config.ron` file.

---

## Installation

Taking into consideration that you have **Rust** installed on your system.

### Clone the repository

```bash
git clone https://github.com/UtsavBalar1231/sync-dotfiles-rs
```

### Build using cargo

```bash
cd sync-dotfiles-rs

cargo build --release
```

### Add the binary to your path

```bash
sudo cp target/release/sync-dotfiles-rs /usr/local/bin
```

---

## Usage

```text
Easily sync dotfiles from a source directory to a destination directory as per your configuration

Usage: sync-dotfiles-rs [OPTIONS] <COMMAND>

Commands:
  force-push, -f      Force push configs from dotconfigs directory into your local system
  force-pull, -F      Force pull configs from your local system into the dotconfigs directory
  update, -u          Update your dotconfigs directory with the latest configs
  clear-metadata, -x  Clear the metadata of config entries in the sync-dotfiles config
  new, -n             Prints a new sync-dotfiles configuration
  printconf, -P       Prints the currently used sync-dotfiles config file
  fix-config, -z      Fix your sync-dotfiles config file for any errors
  add, -a             Adds a new config entry to your exisiting sync-dotfiles config
  clean, -C           Clean all the config directories from your specified dotconfigs path
  help                Print this message or the help of the given subcommand(s)

Options:
  -c, --config-path <CONFIG_PATH>  Provide custom path to the config file (default: ${pwd}/config.ron)
  -h, --help                       Print help
  -V, --version                    Print version
```

### Adding a new config

You can insert a new config in the configs list by simply modifying the configs
list manually or by using the command:

```bash
sync-dotfiles-rs add -n <name> -p <path>`.
```

`-n` or `--name` is the name of the config and `-p` or `--path` is the path to
the config.

### Updating the config

You can update the config by using the command:

```bash
sync-dotfiles-rs -u
```

> Note:
> The hash of the config can initially be set to `None` and you can update it
> later using: `sync-dotfiles-rs -u`

### Clearing the config

You can clear the sync-dotfiles-rs config by using the command:

```bash
sync-dotfiles-rs -x
```

### Printing the config

You can print your config file by using the command:

```bash
sync-dotfiles-rs -P
```

### Using a custom config file path

There can be times when you want to use a custom config file path and not the
default one.\
You can use a custom config file path by using the command:

```bash
sync-dotfiles-rs -c <path_to_config_file>
```

You can also use other commands with the custom config file path by using the
command:

```bash
sync-dotfiles-rs -c <path_to_config_file> <command>
```

Example:

```bash
sync-dotfiles-rs -c /home/utsav/dotfiles/configs/config.ron -u
```

> :warning: **Note:**
> You should first use the `-c <path_to_config_file>` flag and then the command
> and not the other way around.
> You can use the custom config file path with all the commands except `new`.

### Force pushing the configs

Force pushing the configs will push the configs to the local configs directory.

**i.e.** it will overwrite the local configs with the configs in the
`dotconfigs_path`.

You can force push the configs by using the command:

```bash
sync-dotfiles-rs -F
```

### Force pulling the configs

Force pulling the configs will pull the configs from the local configs directory.

**i.e.** it will overwrite the configs in the `dotconfigs_path` with the local
configs.

You can force pull the configs by using the command:

```bash
sync-dotfiles-rs -f
```

---

## Configuration

### Configs structure

The default configuration inside the `config.ron` is `struct DotConfig` which
contains `dotconfigs_path` and a **Vector** of `struct Config`.\
The `dotconfigs_path` is used to store the location of your configs and\
`struct Config` is a Vector (list) of all the configs (it can be a directory
or a single config file).

```text
/// Dotconfig structure that holds a dotconfigs_path handle and a handle to
/// a list of configs
DotConfig {
    dotconfigs_path: String,
    configs: Vec<Config>,
}

/// Config structure that holds the name of the config, path to the config,
/// hash of the config and the type of the config (Dir or File)
Config {
    name: String,
    path: String,
    hash: Option<String>,
    conf_type: Option<ConfType> // Dir or File
}
```

**Default configuration inside `config.ron` looks like a tuple of
`dotconfigs_path` and `struct Config` variables**

```text
#![enable(implicit_some)]
(
    dotconfigs_path: "/* Path to your dotconfigs folder or repository */",
    configs: [
        (
            name: "/* Name of the config */",
            path: "/* Path to the config */",
        ),
    ],
)
```

### Example config file

```text
dotconfigs_path: "/home/<username>/my-dotfiles/configs/"
configs: [
    (name: "nvim", path: "~/.config/nvim", hash: None),
],
```

`dotconfigs_path` is the path to the dotconfigs directory and `configs` is a
list of configs that you want to sync.
