use common::anyhow::Result;

use std::ffi::{OsStr, OsString};

use crate::file_to_process::FileToProcess;

/// The schema of the information that an Indexer extracts from a file
#[derive(Debug)]
pub struct DocumentSchema {
    pub name: String,
    pub body: String,
}

/// Each Indexer needs to be able to say if a file extension is supported and extract information from a supported file
pub trait Indexer: Send + Sync {
    /// If the Indexer supports a file extension
    /// Eg: PdfIndexer supports .pdf extensions
    fn supports_extension(&self, extension: &OsStr) -> bool;

    /// The logic behind the Indexer to extract information from a file
    fn index_file(&self, file_to_process: &FileToProcess) -> Result<DocumentSchema>;

    fn supported_extensions(&self) -> Vec<OsString>;
}
