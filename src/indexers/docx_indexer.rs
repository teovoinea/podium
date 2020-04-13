use super::Indexer;
use super::DocumentSchema;
use std::path::Path;
use std::ffi::{OsStr, OsString};
use std::fs;

use docx::prelude::*;

pub struct DocxIndexer;

impl Indexer for DocxIndexer {
    fn supports_extension(&self, extension: &OsStr) -> bool {
        extension == OsStr::new("docx")
    }

    fn supported_extensions(self) -> Vec<OsString> {
        vec![OsString::from("docx")]
    }

    // Parsing Cats.docx panics the `docx` library...
    // We're just going to leave this out for now
    fn index_file(&self, path: &Path) -> DocumentSchema {
        let mut docx = Docx::from_file(path).unwrap();
        dbg!(docx);

        DocumentSchema {
            name: String::new(),
            body: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indexing_docx_file() {
        let test_file_path = Path::new("./test_files/Cats.docx");
        let indexed_document = DocxIndexer.index_file(test_file_path);

        assert_eq!(indexed_document.name, "file.txt");
        assert_eq!(indexed_document.body, "this is a file with some contents in it");
    }
}
