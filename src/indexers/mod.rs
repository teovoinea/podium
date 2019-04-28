mod text_indexer;

use std::path::Path;
use std::ffi::OsStr;
pub use self::text_indexer::TextIndexer;

#[derive(Debug)]
pub struct DocumentSchema {
    pub name: String,
    pub body: String,
}

pub trait Indexer {
    fn supports_extension(extension: &OsStr) -> bool;

    fn index_file(path: &Path) -> DocumentSchema;
}