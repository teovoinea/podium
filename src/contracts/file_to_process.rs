use crate::tantivy_api::*;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FileToProcess {
    pub path: std::path::PathBuf,
    pub hash: blake2b_simd::Hash,
    pub contents: Vec<u8>,
}

impl From<&Path> for FileToProcess {
    fn from(path: &Path) -> Self {
        FileToProcess {
            path: path.to_path_buf(),
            hash: get_file_hash(path).unwrap(),
            contents: Vec::new()
        }
    }
}


impl From<PathBuf> for FileToProcess {
    fn from(path: PathBuf) -> Self {
        let hash = get_file_hash(&path).unwrap();
        FileToProcess {
            path: path,
            hash: hash,
            contents: Vec::new()
        }
    }
}
