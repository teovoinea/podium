use crate::tantivy_api::*;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Clone)]
pub struct FileToProcess {
    pub path: std::path::PathBuf,
    pub hash: blake2b_simd::Hash,
    pub contents: Vec<u8>,
}

pub async fn newFileToProcess<T: AsRef<Path>>(path: T) -> FileToProcess {
    let contents = fs::read(&path).await.unwrap();
    let hash = calculate_hash(&contents);

    FileToProcess {
        path: PathBuf::from(path.as_ref()),
        hash: hash,
        contents: contents,
    }
}
