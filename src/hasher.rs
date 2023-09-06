use digest::DynDigest;
use std::{
    fmt,
    fmt::Write,
    fs, io,
    io::Read,
    marker,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    string, thread,
};

/// HashBox is a Box<[u8]> type that implements hexadecimal formatting and
/// conversion to a String.
///
/// `HashBox` is a wrapper type for a boxed byte array (`Box<[u8]>`) that
/// represents a hash.
/// It implements the `std::fmt::LowerHex` trait for hexadecimal formatting
/// and the `std::string::ToString` trait
/// for converting the hash to a hexadecimal string.
struct HashBox(Box<[u8]>);

/// Implement std::fmt::LowerHex for Box<[u8]> type
impl fmt::LowerHex for HashBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0
            .iter()
            .for_each(|byte| write!(f, "{:02x}", byte).expect("Failed to write to string"));
        Ok(())
    }
}

/// Implement std::string::ToString for Box<[u8]> type
impl string::ToString for HashBox {
    fn to_string(&self) -> String {
        let mut hex_string = String::with_capacity(self.0.len() * 2);
        self.0.iter().for_each(|byte| {
            write!(hex_string, "{:02x}", byte).expect("Failed to write to string");
        });
        hex_string
    }
}

/// Returns a list of files in a directory.
///
/// This function uses the `walkdir` crate to recursively walk the specified
/// directory and filter out files (not directories).
///
/// # Arguments
///
/// * `p`: A path to the directory to list files from.
///
/// # Returns
///
/// Returns a vector of `PathBuf` representing the paths to files in the
/// directory.
///
/// # Example
///
/// ```rust
/// use sync_dotfiles_rs::hasher::list_dir_files;
///
/// let files = list_dir_files("/path/to/directory");
/// for file in files {
///     println!("Found file: {:?}", file);
/// }
/// ```
pub fn list_dir_files<P>(p: P) -> Vec<PathBuf>
where
    P: AsRef<Path>,
{
    walkdir::WalkDir::new(p)
        .into_iter()
        .filter_map(|file| file.ok())
        .filter(|normal_file| normal_file.metadata().unwrap().is_file())
        .map(|x| x.into_path())
        .collect::<Vec<PathBuf>>()
}

/// Returns the hash of a single file.
///
/// This function reads the specified file into a buffer and hashes it using
/// the provided hasher.
///
/// # Arguments
///
/// * `path`: The path to the file to be hashed.
/// * `hash`: A mutable reference to the hasher.
///
/// # Returns
///
/// Returns a `Result` containing the computed hash as a `String` if
/// successful, or an error if there was an issue reading or hashing the file.
///
/// # Example
///
/// ```rust
/// use sync_dotfiles_rs::hasher::get_file_hash;
/// use sha1::{Sha1, Digest};
///
/// let mut hasher = Sha1::new();
/// match get_file_hash("/path/to/file.txt", &mut hasher) {
///     Ok(hash) => println!("File hash: {}", hash),
///     Err(err) => eprintln!("Error calculating file hash: {:?}", err),
/// }
/// ```
pub fn get_file_hash<Hasher, P>(path: P, hash: &mut Hasher) -> Result<String, io::Error>
where
    Hasher: DynDigest + Clone,
    P: AsRef<Path>,
{
    let mut file = fs::File::open(path)?;
    let mut buf = [0u8; 4096];

    loop {
        let i = file.read(&mut buf)?;
        hash.update(&buf[..i]);

        if i == 0 {
            let final_hash = HashBox(hash.finalize_reset()).to_string();
            return Ok(final_hash);
        }
    }
}

/// Returns the combined hash of a list of files.
///
/// This function parallelizes the hash calculation of multiple files
/// using Rayon.
///
/// # Arguments
///
/// * `files`: A slice of file paths to be hashed.
/// * `hash`: A mutable reference to the hasher.
///
/// # Returns
///
/// Returns a `Result` containing the combined hash of all files as a `String`
/// if successful, or an error if there was an issue reading or hashing the
/// files.
///
/// # Example
///
/// ```rust
/// use sync_dotfiles_rs::hasher::get_files_hash;
/// use sha1::{Sha1, Digest};
///
/// let mut hasher = Sha1::new();
/// let files = vec!["/path/to/file1.txt", "/path/to/file2.txt"];
///
/// match get_files_hash(&files, &mut hasher) {
///     Ok(hash) => println!("Combined files hash: {}", hash),
///     Err(err) => eprintln!("Error calculating combined files hash: {:?}", err),
/// }
/// ```
pub fn get_files_hash<Hasher, P>(files: &[P], hash: &mut Hasher) -> Result<String, io::Error>
where
    P: AsRef<Path> + marker::Sync,
    Hasher: DynDigest + marker::Send + Clone,
{
    if files.is_empty() {
        return Ok(String::new());
    }

    let threads = thread::available_parallelism()
        .unwrap_or(NonZeroUsize::MIN)
        .get();

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .unwrap();

    let mut jobs: Vec<_> = Vec::with_capacity(files.len());

    files.iter().for_each(|file| {
        jobs.push(pool.install(|| -> Result<(), io::Error> {
            let filehash = get_file_hash(file, hash)?;
            hash.update(filehash.as_bytes());
            Ok(())
        }))
    });

    let final_hash = HashBox(hash.finalize_reset()).to_string();

    Ok(final_hash)
}

/// Returns the combined hash of all files in the specified directories.
///
/// This function calculates the hash of all files within the provided
/// directories, combining them into a single hash.
///
/// # Arguments
///
/// * `dirs`: A slice of directory paths containing files to be hashed.
/// * `hash`: A mutable reference to the hasher.
///
/// # Returns
///
/// Returns a `Result` containing the combined hash of all files in the
/// directories as a `String` if successful, or an error if there was an issue
/// reading or hashing the files.
///
/// # Example
///
/// ```rust
/// use sync_dotfiles_rs::hasher::get_complete_dir_hash;
/// use std::path::PathBuf;
/// use sha1::{Sha1, Digest};
///
/// let mut hasher = Sha1::new();
/// let dir_path = PathBuf::from("/path/to/directory");
///
/// match get_complete_dir_hash(&dir_path, &mut hasher) {
///     Ok(hash) => println!("Combined directory files hash: {}", hash),
///     Err(err) => eprintln!("Error calculating combined directory files hash: {:?}", err),
/// }
/// ```
pub fn get_complete_dir_hash<Hasher, P>(dir_path: P, hash: &mut Hasher) -> Result<String, io::Error>
where
    Hasher: DynDigest + Clone + marker::Send,
    P: AsRef<Path> + marker::Sync,
{
    let dirs = list_dir_files(dir_path);
    let mut paths: Vec<PathBuf> = vec![];

    dirs.iter()
        .for_each(|dir| paths.append(&mut list_dir_files(dir)));

    get_files_hash(&paths, hash)
}
