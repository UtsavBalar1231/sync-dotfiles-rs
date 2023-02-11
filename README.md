## Sync Dotfiles based on your environment configuration
---

### Features
- Update your dotconfigs all at once based on the configuration.
- Check inside each config folders and update only if config folder is modified.
- Easy to configure it yourself by simply modifying `config.ron` file.
---

### Example
The default configuration inside config.ron is a structure DotConfig containing a repo to store your configs and a structure Config which is a list of all the configs (it can be a directory or a single config file).

Force Update initially to get the folders synced once.
This will force create and copy all the files from your config folders in your environment (excluding .git files)
```bash
./sync-dotfiles (-f | --force)
```
---

Remove hashes from the config file for a case when your configuration gets invalid somehow
```bash
./sync-dotfiles (-C | --clean-hash)
```
---

To Update your configs once they were synced for the first time
```bash
./sync-dotfiles (-u | --update)
```
---

To Get a dummy config file
```bash
./sync-dotfiles (-n | --new)
```
---

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
    hash: Option<String>
}
```
---

Default configuration inside config.ron looks like a tuple of dotconfigs_path and configs variables
```bash
#![enable(implicit_some)]
(
    dotconfigs_path: ""
    configs: [],
)
```
---

You can insert new config in configs list by simply modifying configs list
The hash of the config initially can be None and you can update it later.
```bash
dotconfigs_path: "/home/<username>/my-dotfiles/configs/"
configs: [
    (name: "nvim", path: "~/.config/nvim", hash: None),
],
```
