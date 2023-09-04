use anyhow::{anyhow, Result};
use home::home_dir;
use ron::{extensions::Extensions, ser::PrettyConfig};
use std::path::PathBuf;

/// A trait for fixing paths to ensure they are absolute and not relative
/// For example, ~/Downloads will be converted to /home/username/Downloads
///
/// Also, /home/username1 will be converted to /home/username2, where username1
/// can be the username of the some other user and username2 is the username of
/// the current user.
pub trait FixPath<T> {
    /// Fix the path to be absolute and not relative.
    fn fix_path(&self) -> Option<PathBuf>;
}

/// Fix the path to be absolute and not relative for PathBuf type
impl FixPath<PathBuf> for PathBuf {
    /// Fix the path to be absolute and not relative for PathBuf type
    fn fix_path(&self) -> Option<PathBuf> {
        let home_dir = home_dir().expect("Failed to get home directory");

        // Check if the path starts with ~/ and replace it with the home directory
        if self.starts_with("~") {
            return Some(
                self.strip_prefix("~")
                    .map(|p| home_dir.join(p))
                    .expect("Failed to strip prefix"),
            );
        } else if self.starts_with("/home/") {
            // Remove the /home/username/ part from the path
            return Some(
                self.strip_prefix("/home/")
                    .map(|p| p.strip_prefix(p.components().next().unwrap()).unwrap())
                    .expect("Failed to strip prefix")
                    .into(),
            );
        }

        None
    }
}

/// Fix the path to be absolute and not relative for string type
impl FixPath<String> for String {
    /// Fix the path to be absolute and not relative for string slice type
    fn fix_path(&self) -> Option<PathBuf> {
        if self.is_empty() {
            return Some(std::path::PathBuf::new());
        }

        let home_dir = home_dir().expect("Failed to get home directory");

        // Check if the path starts with ~/ and replace it with the home directory
        if self.starts_with('~') {
            return Some(self.replace('~', home_dir.to_str().unwrap()).into());
        } else if self.starts_with("/home/") {
            // Remove the /home/username/ part from the path
            let mut path = self.strip_prefix("/home/").unwrap().to_string();
            // Find the next '/' after the first '/' and remove the part before it
            path.drain(..path.find('/').unwrap() + 1);

            return Some(home_dir.join(path));
        }
        None
    }
}

/// Fix the path to be absolute and not relative for string slice type
impl FixPath<&str> for &str {
    /// Fix the path to be absolute and not relative for string slice type
    fn fix_path(&self) -> Option<PathBuf> {
        if self.is_empty() {
            return Some(std::path::PathBuf::new());
        }

        let home_dir = home_dir().expect("Failed to get home directory");

        // Check if the path starts with ~/ and replace it with the home directory
        if self.starts_with('~') {
            return Some(self.replace('~', home_dir.to_str().unwrap()).into());
        } else if self.starts_with("/home/") {
            // Remove the /home/username/ part from the path
            let mut path = self.strip_prefix("/home/").unwrap().to_string();
            // Find the next '/' after the first '/' and remove the part before it
            path.drain(..path.find('/').unwrap() + 1);

            return Some(home_dir.join(path));
        }
        None
    }
}

/// Recursively copy a directory and its contents to another location.
///
/// This function copies a directory and its contents to another location.
/// It is a recursive operation and can handle both directories and files.
/// If the destination directory exists, it will be removed and recreated to
/// ensure a clean copy.
///
/// # Arguments
///
/// * `from`: The source directory or file path to be copied.
/// * `to`: The destination directory where the source will be copied to.
///
/// # Returns
///
/// Returns a `Result` indicating success or an error if the copy operation
/// fails.
///
/// # Example
///
/// ```rust
/// use sync_dotfiles_rs::utils::copy_dir;
///
/// match copy_dir("/path/to/source", "/path/to/destination") {
///     Ok(()) => println!("Copy successful"),
///     Err(err) => eprintln!("Error copying directory: {:?}", err),
/// }
/// ```
pub fn copy_dir<T>(from: T, to: T) -> Result<()>
where
    T: AsRef<std::path::Path>,
{
    let from = from.as_ref();

    if !from.exists() {
        return Err(anyhow!(format!(
            "Path does not exist or access denied!: {:#?}",
            from
        )));
    }

    if to.as_ref().exists() {
        std::fs::remove_dir_all(&to)?;
    }
    std::fs::create_dir_all(&to)?;

    std::fs::read_dir(from)?
        .filter_map(|e| e.ok())
        .for_each(|entry| {
            let filetype = entry.file_type().expect("Failed to read file type");
            if filetype.is_dir() {
                copy_dir(entry.path(), to.as_ref().join(entry.file_name()))
                    .expect("Failed to copy directory");
            } else if filetype.is_file() {
                if let Err(e) = std::fs::copy(entry.path(), to.as_ref().join(entry.file_name())) {
                    match e.kind() {
                        std::io::ErrorKind::AlreadyExists => {
                            println!(
                                "File already exists, skipping: {:#?}",
                                entry.path().display()
                            )
                        }
                        _ => panic!("Error copying file: {e}"),
                    }
                }
            } else {
                println!("Skipping symlinks file: {:#?}", entry.path().display());
            }
        });
    Ok(())
}

/// Get a pretty printer configuration for RON (Rusty Object Notation)
/// serialization.
///
/// This function returns a configuration for pretty-printing RON data with a
/// depth limit and specific extensions.
///
/// # Returns
///
/// Returns a `PrettyConfig` that can be used with the RON serialization.
///
/// # Example
///
/// ```rust
/// use sync_dotfiles_rs::dotconfig::DotConfig;
/// use sync_dotfiles_rs::utils::get_ron_formatter;
/// use ron::ser::to_string_pretty;
///
/// let data = DotConfig::new();
/// let pretty_config = get_ron_formatter();
/// let ron_string = to_string_pretty(&data, pretty_config).expect("Failed to serialize data");
/// println!("Pretty RON:\n{}", ron_string);
/// ```
pub fn get_ron_formatter() -> PrettyConfig {
    PrettyConfig::new()
        .depth_limit(2)
        .extensions(Extensions::IMPLICIT_SOME)
}
