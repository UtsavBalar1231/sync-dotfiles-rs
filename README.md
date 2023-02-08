## Sync Dotfiles based on your environment configuration
---

### Features
1. Check inside config folders if any file is modified.
2. Easy to configure using `config.ron`.
---

### Example
The default configuration inside config.ron is a structure DotConfig containing a list of structure Config
```rust
DotConfig {
    configs: Vec<Config>,
}

Config {
    name: String,
    path: String,
    hash: Option<String>
}
```
---

Default configuration in config.ron
```bash
#![enable(implicit_some)]
(
    configs: [],
)
```
---

You can insert new config in configs list by simply modifying configs list
The hash of the config initially can be None and you can update it later.
```bash
configs: [
    (name: "nvim", path: "~/.config/nvim", hash: None),
],
```
