use crate::custom_tantivy::utils::calculate_hash;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{info_span, instrument};

#[derive(Debug, Clone)]
pub struct FileToProcess {
    pub path: std::path::PathBuf,
    pub hash: blake2b_simd::Hash,
    pub contents: Vec<u8>,
}

impl FileToProcess {
    pub fn path(&self) -> String {
        self.path.to_string_lossy().to_string()
    }
}

#[instrument]
pub async fn new_file_to_process<T: AsRef<Path> + Debug>(path: T) -> FileToProcess {
    let contents = fs::read(&path).await.unwrap();

    let span = info_span!("calculating hash");
    let _enter = span.enter();
    let hash = calculate_hash(&contents);
    drop(_enter);

    FileToProcess {
        path: PathBuf::from(path.as_ref()),
        hash: hash,
        contents: contents,
    }
}
