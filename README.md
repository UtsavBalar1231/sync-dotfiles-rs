## Sync Dotfiles based on your environment configuration
---

### Features
- Update your dotconfigs all at once based on the configuration file.
- Check inside each config folder and update only if the config folder was actually modified.
- Easy to configure by yourself, simply modify the `config.ron` file as per your choice.
---

### Usage
The default configuration inside the config.ron is a structure DotConfig which contains dotconfigs_path and a Vector of Config structure.
The dotconfigs_path is used to store the location of your configs and the Config structure is a list that contains a list of all the configs (it can be a directory or a single config file).

```bash
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
___

### Configs structure

```rust
/// Dotconfig structure that holds a dotconfigs_path handle and a handle to a list of configs
DotConfig {
    dotconfigs_path: String,
    configs: Vec<Config>,
}

/// Config structure that holds the name, path and hash of the config folder/file
Config {
    name: String,
    path: String,
    hash: Option<String>,
    conf_type: Option<ConfType> // Dir or File
}
```
---

Default configuration inside config.ron looks like a tuple of dotconfigs_path and configs variables
```bash
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
---

You can insert a new config in the configs list by simply modifying the configs list manually or by using the command `sync-dotconfigs add -n <name> -p <path>`.
The hash of the config can initially be set to None and you can update it later using 'sync-dotconfigs -u'.
```bash
dotconfigs_path: "/home/<username>/my-dotfiles/configs/"
configs: [
    (name: "nvim", path: "~/.config/nvim", hash: None),
],
```
