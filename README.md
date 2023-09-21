# sync-dotfiles-rs: Easily sync dotfiles across machines

sync-dotfiles-rs is a Rust project that simplifies the management and
synchronization of configuration files, often referred to as "dotfiles,"
between a central repository and a user's home directory.
This project is designed to help users maintain consistent configurations
across multiple machines effortlessly.

The library consists of the following modules:

- `config`: Provides support to represent and manipulate the config file
  data using a structure.
- `dotconfig`: Provides support to store the list of the config files with
  their path in the config file.
- `hasher`: Contains various hashing functionality used to calculate file
  and directory hashes.
- `utils`: Contains various utility functions used for path manipulation
  and directory copying.

This library can be used to create, update, and synchronize configuration
files between a central repository
(e.g., a version control system like Git) and a user's home directory,
making it easier to manage and version-control
configuration settings across multiple machines.

> Example

```rust
use sync_dotfiles_rs::config::Config;

fn test() {
    // Create a new Config instance
    let config = Config::new(
    String::from("config.ron"),
    format!("{}/examples/config.ron", env!("CARGO_MANIFEST_DIR")),
        None,
        None,
    );

    // Check if the config path exists
    if config.path_exists() {
        println!("Config file exists: {}", config.path);
    } else {
        println!("Config file does not exist: {}", config.path);
    }
}
```

## Features

- Update Configs: Synchronize dotfiles based on the configuration file.
- Hash-Based Updates: Parse the config file and update configs based on their hash values.
- Add New Configs: Easily add new configurations to the config file.
- Parallel Processing: Speed up the update process with parallelization.
- Configuration Customization: Modify the config.ron file to suit your needs.

---

## Installation

To install sync-dotfiles-rs, follow these steps:

> [!WARNING]
> You must have Rust installed on your computer.
> You can install Rust by running the following command.
>
> ```bash
> curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
> ```

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
Easily sync dotfiles across machines

Usage: sync-dotfiles-rs [OPTIONS] <COMMAND>

Commands:
  force-push, -f      Force push configs from dotconfigs directory into your local system
  force-pull, -F      Force pull configs from your local system into the dotconfigs directory
  pull, -u            Update your dotconfigs directory with the latest configs
  push, -U            Update your local system configs with the configs from the dotconfigs directory
  clear-metadata, -x  Clear the metadata of config entries in the sync-dotfiles config
  new, -n             Prints a new sync-dotfiles configuration
  printconf, -P       Prints the currently used sync-dotfiles config file
  fix-config, -z      Fix your sync-dotfiles config file for any errors
  add, -a             Adds a new config entry to your exisiting sync-dotfiles config
  clean, -C           Clean all the config directories from your specified dotconfigs path
  edit, -e            Edit the sync-dotfiles config file
  help                Print this message or the help of the given subcommand(s)

Options:
  -c, --config-path <CONFIG_PATH>  Provide custom path to the config file (default: ${pwd}/config.ron)
  -h, --help                       Print help
  -V, --version                    Print version
```

### Creating a new config

For the first time, you can create a new config by using the command:

```bash
sync-dotfiles-rs new
```

or

```bash
sync-dotfiles-rs -n
```

This will create a new config file in the home directory.

Confirm the config file by using the command:

```bash
sync-dotfiles-rs -P
```

> You will see the following output:
>
> ```text
> Found config file in /home/vicharak/.config/sync-dotfiles directory
> DotConfig {
>     dotconfigs_path: ~/dotfiles/configs/,
>     configs: [
>            ...
>     ],
> ```

### Adding a new config

For adding a new config entry on the sync-dotfiles config file, you have two
options:

1. You can edit the config file by using the command:

```bash
sync-dotfiles-rs edit
```

```bash
sync-dotfiles-rs -e
```

2. You can use the `add` command to add a new config entry to the config file.

```bash
sync-dotfiles-rs add -n <name> -p <path>`.
```

> [!NOTE]
> `-n` or `--name` is the name of the config and `-p` or `--path` is the path to
> the config.

### Updating your dotconfigs directory with local system configs

You can update the config files by

```bash
sync-dotfiles-rs pull
```

or

```bash
sync-dotfiles-rs -u
```

> [!NOTE]
> The hash of the config can initially be set to `None` and you can update it
> later using: `sync-dotfiles-rs -u`

### Updating your local system configs with the configs from the dotconfigs directory

You can update your local system configs with the configs from the dotconfigs
directory by using the command:

```bash
sync-dotfiles-rs push
```

or

```bash
sync-dotfiles-rs -U
```

### Clearing the metadata of config entries in the sync-dotfiles config

You can clean the hash and config type data from your sync-dotfiles config file
by using the command:

```bash
sync-dotfiles-rs clear-metadata
```

or

```bash
sync-dotfiles-rs -x
```

### Printing the currently active config file of sync-dotfiles

You can print your currently used sync-dotfiles config by using the command:

```bash
sync-dotfiles-rs printconf
```

or

```bash
sync-dotfiles-rs -P
```

### Using a custom config file path

**sync-dotfiles-rs** supports using a custom config file path. This is useful
when you want to use a custom config file path and not the default one.

You can use a custom config file path by using the command:

```bash
sync-dotfiles-rs --config-path <path_to_config_file>
```

```bash
sync-dotfiles-rs -c <path_to_config_file>
```

> [!NOTE]
> You can also use other commands with the custom config file path by using the
> command:
>
> ```bash
> sync-dotfiles-rs -c <path_to_config_file> <command>
> ```
>
> Example:
>
> ```bash
> sync-dotfiles-rs -c /home/utsav/dotfiles/configs/config.ron -u
> ```

> [!WARNING]
> You should first use the `-c <path_to_config_file>` flag and then the command
> and not the other way around.
> You can use the custom config file path with all the commands except `new`.

### Force pushing the configs

Forcefully push all the configs to their specified destinations.

**i.e.** It will forcefully overwrite the local configs
(configs in your home directory) with the configs in the `dotconfigs_path` as
specified in the sync-dotfiles config file.

You can force push the configs by using the command:

```bash
sync-dotfiles-rs force-push
```

or

```bash
sync-dotfiles-rs -F
```

### Force pulling the configs

Forcefully pull the latest versions of all the configs to their specified
destinations from the `dotconfigs_path`.

**i.e.** It will forcefully overwrite the configs in the `dotconfigs_path` with
the latest versions of the configs from your home directory.

You can force pull the configs by using the command:

```bash
sync-dotfiles-rs force-pull
```

or

```bash
sync-dotfiles-rs -f
```

### Fixing your sync-dotfiles config file

You can fix your sync-dotfiles config file for problems such as missing
configurations and wrong path entries by using the command:

```bash
sync-dotfiles-rs fix-config
```

or

```bash
sync-dotfiles-rs -z
```

## Cleaning the sync-dotfiles repository/directory

You can clean the sync-dotfiles repository or directory by using the command:

```bash
sync-dotfiles-rs clean
```

or

```bash
sync-dotfiles-rs -C
```

---


## Configuration

### Configs Structure

The default configuration inside the `config.ron` file is defined by the
`struct DotConfig`.

This structure contains two main components:

#### DotConfig Structure

The `DotConfig` structure holds two fields:

- `dotconfigs_path: String`:
    This field represents the path to the directory where your dotfiles and
    configurations are stored.

- `configs: Vec<Config>`:
    This is a vector (list) of `Config` structures, which can represent either
    individual configuration files or directories.

#### Config Structure

The `Config` structure is used to describe an individual configuration entry.

It contains the following fields:

- `name: String`: The name of the configuration entry.
- `path: String`: The path to the configuration file or directory.
- `hash: Option<String>`: An optional field to store the hash of the
configuration. This hash can be used for tracking changes in the configuration.
- `conf_type: Option<ConfType>`: An optional field indicating the type of the
configuration entry, which can be either a directory or a file.

**Default Configuration Inside `config.ron`**

Here's an example of the default configuration structure within the
`config.ron` file:

```ron
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

For reference, here's an example of how a config.ron file can be structured:

```text
dotconfigs_path: "/home/<username>/my-dotfiles/configs/"
configs: [
    (name: "nvim", path: "~/.config/nvim", hash: None),
],
```

In this example:

- `dotconfigs_path` specifies the path to the dotconfigs directory or
repository.

- `configs` is a list of individual configuration entries.
In this case, there's a single entry named "nvim," representing the Neovim
configuration, with its path and an optional hash field.

This configuration file allows you to define the locations and details of your
dotfiles and configuration files, making it easier to manage and synchronize
them across multiple machines.

---

## License

[MIT](./LICENSE)
