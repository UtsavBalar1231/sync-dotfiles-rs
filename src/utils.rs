use crate::*;
use home::home_dir;

/// Fix the path to make sure it is absolute and not relative
/// For example, ~/Downloads will be converted to /home/username/Downloads
pub trait FixPath<T> {
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

/// Copy the directory recursively
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
        fs::remove_dir_all(&to)?;
    }
    fs::create_dir_all(&to)?;

    fs::read_dir(from)?
        .filter_map(|e| e.ok())
        .for_each(|entry| {
            let filetype = entry.file_type().expect("Failed to read file type");
            if filetype.is_dir() {
                copy_dir(entry.path(), to.as_ref().join(entry.file_name()))
                    .expect("Failed to copy directory");
            } else if filetype.is_file() {
                if let Err(e) = fs::copy(entry.path(), to.as_ref().join(entry.file_name())) {
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
