pub use anyhow::{anyhow, Context, Result};
pub use ron::{
    extensions::Extensions,
    ser::{to_string_pretty, PrettyConfig},
    Options,
};
pub use serde::{Deserialize, Serialize};
pub use std::path::PathBuf;

/// Provides support to represent and manipulate the config file data using a structure.
pub mod config;
/// Provides support to store the list of the config files with their path in the config file.
pub mod dotconfig;
/// Various utility functions.
pub mod utils;
