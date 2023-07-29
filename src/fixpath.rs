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
