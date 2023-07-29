# sync-dotfiles-rs: Easily sync dotfiles across machines

## Features

- Update your dotconfigs all at once based on the configuration file.
- Parse the config file and update the configs based on the hash of the config.
- Add new configs to the config file and update the configs.
- Parallelize the update process to speed up the process.
- Easy to configure by yourself, simply modify the `config.ron` file.

---

## Usage

```text
Usage: sync-dotfiles-rs [OPTIONS] [COMMAND]

Commands:
  add    Adds a new config entry to your exisiting config file
  clean  Clean all the config directories from the dotconfigs path specified in the config file
  help   Print this message or the help of the given subcommand(s)

Options:
  -F, --fpush                Force push the configs listed in config to the local configs directory
  -f, --fpull                Force pull the local configs inside the mentioned dotconfigs directory
  -u, --update               Update the config file with new files
  -x, --clear                Clear the metadata of config entries in the config file
  -n, --new                  Prints the new config file
  -p, --print                Print the contents of the config file
  -c, --cpath <CONFIG_PATH>  The path of the config file (default: current_dir/config.ron)
  -h, --help                 Print help
  -V, --version              Print version
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
The hash of the config can initially be set to `None` and you can update it
later using: `sync-dotfiles-rs -u`

### Clearing the config

You can clear the sync-dotfiles-rs config by using the command:

```bash
sync-dotfiles-rs -x
```

### Printing the config

You can print your config file by using the command:

```bash
sync-dotfiles-rs -p
```

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

## Configuration

### Configs structure

The default configuration inside the `config.ron` is `struct DotConfig` which
contains `dotconfigs_path` and a **Vector** of `struct Config`. \
The `dotconfigs_path` is used to store the location of your configs and \
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
