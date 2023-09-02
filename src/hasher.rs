use digest::DynDigest;
use std::{
    fmt,
    fmt::Write,
    io,
    io::Read,
    num::NonZeroUsize,
    path::{Path, PathBuf},
};

/// HashBox is a Box<[u8]> type
/// It implements the std::fmt::LowerHex trait and std::string::ToString
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
impl std::string::ToString for HashBox {
    fn to_string(&self) -> String {
        let mut hex_string = String::with_capacity(self.0.len() * 2);
        self.0.iter().for_each(|byte| {
            write!(hex_string, "{:02x}", byte).expect("Failed to write to string");
        });
        hex_string
    }
}

/// Returns all the files in a directory
/// Use Walker to walk the directory and check if it is a file
/// or a directory.
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

/// Returns hash of a single file
/// Read the file into a buffer and hash it using the provided hasher
pub fn get_file_hash<Hasher, P>(path: P, hash: &mut Hasher) -> Result<String, io::Error>
where
    Hasher: DynDigest + Clone,
    P: AsRef<Path>,
{
    let mut file = std::fs::File::open(path)?;
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

/// Returns hash of a list of files
/// Use rayon to parallelize the hash calculation
/// Iterate over the list of files and call get_file_hash
/// to calculate the hash
pub fn get_files_hash<Hasher, P>(files: &[P], hash: &mut Hasher) -> Result<String, io::Error>
where
    P: AsRef<Path> + std::marker::Sync,
    Hasher: DynDigest + std::marker::Send + Clone,
{
    if files.is_empty() {
        return Ok(String::new());
    }

    let threads = std::thread::available_parallelism()
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

/// Returns the hash of all the files in the directories
pub fn get_complete_dir_hash<Hasher, P>(dirs: &[P], hash: &mut Hasher) -> Result<String, io::Error>
where
    Hasher: DynDigest + Clone + std::marker::Send,
    P: AsRef<Path> + std::marker::Sync,
{
    let mut paths: Vec<PathBuf> = vec![];

    dirs.iter()
        .for_each(|dir| paths.append(&mut list_dir_files(dir)));

    get_files_hash(&paths, hash)
}
