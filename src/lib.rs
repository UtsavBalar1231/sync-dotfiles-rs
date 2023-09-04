//! This library provides tools for managing and synchronizing configuration
//! files, often referred to as "dotfiles,"
//! between a centralized configuration repository and a user's home directory.
//!
//! The library consists of the following modules:
//!
//! - `config`: Provides support to represent and manipulate the config file
//! data using a structure.
//! - `dotconfig`: Provides support to store the list of the config files with
//! their path in the config file.
//! - `hasher`: Contains various hashing functionality used to calculate file
//! and directory hashes.
//! - `utils`: Contains various utility functions used for path manipulation
//! and directory copying.
//!
//! This library can be used to create, update, and synchronize configuration
//! files between a central repository
//! (e.g., a version control system like Git) and a user's home directory,
//! making it easier to manage and version-control
//! configuration settings across multiple machines.
//!
//! # Example
//!
//! ```rust
//! use sync_dotfiles_rs::config::Config;
//!
//! fn test() {
//!     // Create a new Config instance
//!     let config = Config::new(
//!         String::from("vimrc"),
//!         String::from("~/vimrc"),
//!         None,
//!         None,
//!     );
//!
//!     // Check if the config path exists
//!     if config.path_exists() {
//!         println!("Config file exists: {}", config.path);
//!     } else {
//!         println!("Config file does not exist: {}", config.path);
//!     }
//! }
//! ```

/// Provides support to represent and manipulate the config file data using a
/// structure
pub mod config;

/// Provides support to store the list of the config files with their path
/// in the config file.
pub mod dotconfig;

/// Various utility functions for working with files, paths, and hashing.
pub mod utils;

/// Various hashing functions for calculating file and directory hashes.
pub mod hasher;
