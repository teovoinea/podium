use common::anyhow::{Context, Error, Result};
use common::tracing::span;
use contracts::file_to_process::FileToProcess;
use contracts::indexer::{DocumentSchema, Indexer};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::Path;

use docx::*;

pub struct DocxIndexer;

impl Indexer for DocxIndexer {
    fn supports_extension(&self, extension: &OsStr) -> bool {
        extension == OsStr::new("docx")
    }

    fn supported_extensions(&self) -> Vec<OsString> {
        vec![OsString::from("docx")]
    }

    // Parsing Cats.docx panics the `docx` library...
    // We're just going to leave this out for now
    fn index_file(&self, file_to_process: &FileToProcess) -> Result<DocumentSchema> {
        // let mut docx = Docx::from_file(file_to_process.path).unwrap();
        // dbg!(docx);

        Ok(DocumentSchema {
            name: String::new(),
            body: String::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::tokio;
    use contracts::file_to_process::new_file_to_process;

    // #[tokio::test]
    // async fn test_indexing_docx_file() {
    //     let test_file_path = Path::new("../../../test_files/Cats.docx");

    //     let indexed_document = DocxIndexer
    //         .index_file(&new_file_to_process(test_file_path).await)
    //         .unwrap();

    //     assert_eq!(indexed_document.name, "file.txt");
    //     assert_eq!(
    //         indexed_document.body,
    //         "this is a file with some contents in it"
    //     );
    // }
}
