use blake2b_simd::blake2b;
use common::tokio::fs;
use common::tracing::*;
use common::tracing::{info_span, instrument};
use common::tracing_futures;
use std::fmt::Debug;
use std::path::{Path, PathBuf};

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

fn calculate_hash(input: &[u8]) -> blake2b_simd::Hash {
    let file_hash = blake2b(input);
    info!("Hash of file is: {:?}", file_hash);
    file_hash
}
