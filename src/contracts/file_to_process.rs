use crate::tantivy_api::*;
use async_trait::async_trait;
use std::borrow::*;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Clone)]
pub struct FileToProcess {
    pub path: std::path::PathBuf,
    pub hash: blake2b_simd::Hash,
    pub contents: Vec<u8>,
}

// #[async_trait]
// trait AsyncFrom<'a, T> where T: AsRef<Path> {
//     async fn from(path: &'a T) -> FileToProcess;
// }

// #[async_trait]
// impl<'a, T> AsyncFrom<'a, T> for FileToProcess where T: AsRef<Path> + Clone + Send + Sync + 'a {
//     async fn from(path: &'a T) -> Self {
//         //let path: PathBuf = path.into().clone();
//         let hash = get_file_hash(path.as_ref()).unwrap();
//         let mut contents = fs::read(path).await.unwrap();

//         FileToProcess {
//             path: PathBuf::from(path.as_ref()),
//             hash: hash,
//             contents: contents,
//         }
//     }
// }

pub async fn newFileToProcess<T: AsRef<Path>>(path: T) -> FileToProcess {
    let contents = fs::read(&path).await.unwrap();
    let hash = calculate_hash(&contents);

    FileToProcess {
        path: PathBuf::from(path.as_ref()),
        hash: hash,
        contents: contents,
    }
}

// impl From<&Path> for FileToProcess {
//     fn from(path: &Path) -> Self {
//         let mut contents = Vec::new();
//         let mut file = File::open(path).unwrap();
//         file.read_to_end(&mut contents).unwrap();

//         FileToProcess {
//             path: path.to_path_buf(),
//             hash: get_file_hash(path).unwrap(),
//             contents: contents,
//         }
//     }
// }

// impl From<PathBuf> for FileToProcess {
//     fn from(path: PathBuf) -> Self {
//         let hash = get_file_hash(&path).unwrap();
//         let mut contents = Vec::new();
//         let mut file = File::open(&path).unwrap();
//         file.read_to_end(&mut contents).unwrap();

//         FileToProcess {
//             path: path,
//             hash: hash,
//             contents: contents,
//         }
//     }
// }
