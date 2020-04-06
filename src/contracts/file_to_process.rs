use crate::tantivy_api::*;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FileToProcess {
    pub path: std::path::PathBuf,
    pub hash: blake2b_simd::Hash,
    pub contents: Vec<u8>,
}

impl From<&Path> for FileToProcess {
    fn from(path: &Path) -> Self {
        let mut contents = Vec::new();
        let mut file = File::open(path).unwrap();
        file.read_to_end(&mut contents).unwrap();

        FileToProcess {
            path: path.to_path_buf(),
            hash: get_file_hash(path).unwrap(),
            contents: contents,
        }
    }
}

impl From<PathBuf> for FileToProcess {
    fn from(path: PathBuf) -> Self {
        let hash = get_file_hash(&path).unwrap();
        let mut contents = Vec::new();
        let mut file = File::open(&path).unwrap();
        file.read_to_end(&mut contents).unwrap();

        FileToProcess {
            path: path,
            hash: hash,
            contents: contents,
        }
    }
}
