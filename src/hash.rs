/*
 * Stripped down version of the `hristogochev/merkle_hash` crate
 * Extracted only the Blake3 hashing functionality
 * All credits go to the original author
 */

use anyhow::{bail, Context, Result};
use blake3::Hasher;
use rayon::prelude::*;
use std::{
    cmp::Ordering,
    collections::{btree_set::IntoIter, btree_set::Iter, BTreeSet},
    iter::FusedIterator,
    path::PathBuf,
};

/// An utility struct that contains an absolute path and a relative path
#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub struct MerklePath {
    pub relative: PathBuf,
    pub absolute: PathBuf,
}

impl MerklePath {
    pub fn new(relative_path: PathBuf, absolute_path: PathBuf) -> Self {
        Self {
            relative: relative_path,
            absolute: absolute_path,
        }
    }
}

impl PartialOrd<Self> for MerklePath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.relative.partial_cmp(&other.relative)
    }
}

impl Ord for MerklePath {
    fn cmp(&self, other: &Self) -> Ordering {
        self.relative.cmp(&other.relative)
    }
}

/// Holds the path, hash and children paths of a file or directory
#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub struct MerkleItem {
    pub path: MerklePath,
    pub hash: Vec<u8>,
    pub children_paths: BTreeSet<MerklePath>,
}

impl MerkleItem {
    pub fn new(path: MerklePath, hash: Vec<u8>, children_paths: BTreeSet<MerklePath>) -> Self {
        Self {
            path,
            hash,
            children_paths,
        }
    }
}

impl PartialOrd<Self> for MerkleItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.path.partial_cmp(&other.path)
    }
}

impl Ord for MerkleItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.path.cmp(&other.path)
    }
}

/// Owned node iterator
#[derive(Default)]
pub struct MerkleNodeIntoIter {
    value: Option<MerkleItem>,
    children: Option<IntoIter<MerkleNode>>,
    parent: Option<Box<MerkleNodeIntoIter>>,
}

impl MerkleNodeIntoIter {
    pub fn new(
        value: MerkleItem,
        children: IntoIter<MerkleNode>,
        parent: Option<Box<MerkleNodeIntoIter>>,
    ) -> Self {
        Self {
            value: Some(value),
            children: Some(children),
            parent,
        }
    }
}

impl Iterator for MerkleNodeIntoIter {
    type Item = MerkleItem;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(value) = self.value.take() {
            return Some(value);
        }
        if let Some(first_child) = self.children.as_mut().and_then(|children| children.next()) {
            return if first_child.children.is_empty() {
                Some(first_child.item)
            } else {
                *self = MerkleNodeIntoIter::new(
                    first_child.item,
                    first_child.children.into_iter(),
                    Some(Box::new(std::mem::take(self))),
                );
                self.next()
            };
        } else if let Some(parent) = self.parent.take() {
            *self = *parent;
            return self.next();
        }
        None
    }
}

impl IntoIterator for MerkleNode {
    type Item = MerkleItem;

    type IntoIter = MerkleNodeIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        MerkleNodeIntoIter::new(self.item, self.children.into_iter(), None)
    }
}

/// Node iterator
#[derive(Default)]
pub struct MerkleNodeIter<'a> {
    value: Option<&'a MerkleItem>,
    children: Option<Iter<'a, MerkleNode>>,
    parent: Option<Box<MerkleNodeIter<'a>>>,
}

impl<'a> MerkleNodeIter<'a> {
    pub fn new(
        value: &'a MerkleItem,
        children: Iter<'a, MerkleNode>,
        parent: Option<Box<MerkleNodeIter<'a>>>,
    ) -> Self {
        Self {
            value: Some(value),
            children: Some(children),
            parent,
        }
    }
}

impl<'a> Iterator for MerkleNodeIter<'a> {
    type Item = &'a MerkleItem;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(value) = self.value.take() {
            return Some(value);
        }

        if let Some(first_child) = self.children.as_mut().and_then(|children| children.next()) {
            return if first_child.children.is_empty() {
                Some(&first_child.item)
            } else {
                *self = MerkleNodeIter::new(
                    &first_child.item,
                    first_child.children.iter(),
                    Some(Box::new(std::mem::take(self))),
                );
                self.next()
            };
        } else if let Some(parent) = self.parent.take() {
            *self = *parent;
            return self.next();
        }
        None
    }
}

impl MerkleNode {
    /// Returns an iterator over each file and directory descendant of the current node
    pub fn iter(&self) -> MerkleNodeIter<'_> {
        MerkleNodeIter::new(&self.item, self.children.iter(), None)
    }
}

impl<'a> IntoIterator for &'a MerkleNode {
    type Item = &'a MerkleItem;

    type IntoIter = MerkleNodeIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> FusedIterator for MerkleNodeIter<'a> {}

/// Represents a single node on the merkle tree
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct MerkleNode {
    pub item: MerkleItem,
    pub children: BTreeSet<MerkleNode>,
}

impl PartialOrd<Self> for MerkleNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.item.partial_cmp(&other.item)
    }
}

impl Ord for MerkleNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.item.cmp(&other.item)
    }
}

impl MerkleNode {
    /// Creates a new root node
    pub fn root(root: &str, hash_names: bool) -> Result<Self> {
        // Creates a new empty relative path, as this is the root
        let relative_path = PathBuf::from("");

        // Gets an owned copy of the absolute path
        let absolute_path = PathBuf::from(root);

        // Creates a new merkle path based on them both
        let path = MerklePath::new(relative_path, absolute_path);

        // Indexes the newly created node and returns the result
        Self::index(root, path, hash_names)
    }

    /// Indexes a new node, finding its relative and absolute paths, its file/directory hash
    /// and the same for all of its descendants
    fn index(root: &str, path: MerklePath, hash_names: bool) -> Result<MerkleNode> {
        // Indexes its direct descendants for their hashes and paths
        let children = if path.absolute.is_dir() {
            std::fs::read_dir(&path.absolute)?
                .par_bridge()
                .map(|entry| {
                    let absolute_path = match entry {
                        Ok(absolute_path) => absolute_path.path(),
                        Err(path) => bail!("Path is not valid UTF8 path: {}", path),
                    };
                    let relative_path = absolute_path.strip_prefix(root)?.to_path_buf();
                    let path = MerklePath::new(relative_path, absolute_path);
                    let node = Self::index(root, path, hash_names)?;
                    Ok(node)
                })
                .collect::<Result<BTreeSet<MerkleNode>>>()?
        } else {
            BTreeSet::new()
        };

        // Finds the node's contents hash
        let contents_hash: Vec<u8> = if path.absolute.is_dir() {
            let hashes: Vec<_> = children
                .iter()
                .map(|child| child.item.hash.as_slice())
                .collect();

            match compute_merkle_hash(&hashes) {
                Some(hash) => hash,
                None => compute_hash(b""),
            }
        } else {
            let file_bytes = std::fs::read(&path.absolute)?;
            compute_hash(&file_bytes)
        };

        // Check if names should be included in the hashing results and get the output hash
        let hash: Vec<u8> = if hash_names {
            // Gets the node path's name
            let name = path
                .absolute
                .file_name()
                .with_context(|| format!("File name missing for: {path:?}"))?;

            // Create a hashing stack
            compute_hash_from_slices(name.to_string_lossy().as_bytes(), &contents_hash)
        } else {
            contents_hash
        };

        // Get the direct descendant paths
        let children_paths = children
            .par_iter()
            .map(|child| child.item.path.clone())
            .collect();

        // Returns the newly created node with its data
        let item = MerkleItem::new(path, hash, children_paths);
        let node = MerkleNode { item, children };

        Ok(node)
    }
}

/// Represents an indexed directory tree
pub struct MerkleTree {
    pub main_node: MerkleNode,
}

impl MerkleTree {
    /// Creates a new merkle tree builder
    ///
    /// - Default hash_names is **false**
    pub fn builder(root_absolute_path: impl AsRef<str>) -> MerkleTreeBuilder {
        let absolute_root_path = root_absolute_path.as_ref().to_owned();
        MerkleTreeBuilder {
            absolute_root_path,
            hash_names: false,
        }
    }
    /// Returns an iterator over each file and directory in the tree
    pub fn iter(&self) -> MerkleNodeIter {
        self.main_node.iter()
    }
}

impl<'a> IntoIterator for &'a MerkleTree {
    type Item = &'a MerkleItem;

    type IntoIter = MerkleNodeIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for MerkleTree {
    type Item = MerkleItem;

    type IntoIter = MerkleNodeIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.main_node.into_iter()
    }
}

/// Utility builder pattern
pub struct MerkleTreeBuilder {
    /// Absolute root path of the tree
    pub(crate) absolute_root_path: String,
    /// Whether to include names in the hashes of files and directories, default is false
    pub(crate) hash_names: bool,
}

impl MerkleTreeBuilder {
    /// Sets whether to include the names of the files and directories in the hashing process, default is **false**
    pub fn hash_names(mut self, hash_names: bool) -> Self {
        self.hash_names = hash_names;
        self
    }

    /// Builds the hash tree by indexing all of its descendants
    pub fn build(self) -> Result<MerkleTree> {
        let main_node = MerkleNode::root(&self.absolute_root_path, self.hash_names)?;
        Ok(MerkleTree { main_node })
    }
}

pub fn compute_merkle_hash(hashes: &[&[u8]]) -> Option<Vec<u8>> {
    let len = hashes.len();

    if len < 1 {
        return None;
    }

    if len == 1 {
        return hashes.first().copied().map(|first| first.to_vec());
    }

    let output: Vec<_> = hashes
        .par_chunks(2)
        .flat_map(|hash_chunks| {
            let first = hash_chunks.first()?;
            let second = match hash_chunks.get(1) {
                Some(second) => second,
                None => first,
            };
            let hash = compute_hash_from_slices(first, second);
            Some(hash)
        })
        .collect();

    let output: Vec<_> = output
        .iter()
        .map(|reference| reference.as_slice())
        .collect();

    compute_merkle_hash(&output)
}

/// Computes a single hash from 2 slices of bytes
pub fn compute_hash_from_slices(first_slice: &[u8], second_slice: &[u8]) -> Vec<u8> {
    let mut hasher = Hasher::new();
    hasher.update(first_slice);
    hasher.update(second_slice);
    hasher.finalize().as_bytes().to_vec()
}

/// Computes a hash from a slice of bytes
pub fn compute_hash(bytes: &[u8]) -> Vec<u8> {
    blake3::hash(bytes).as_bytes().to_vec()
}
