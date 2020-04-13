use super::DocumentSchema;
use super::Indexer;
use crate::contracts::file_to_process::FileToProcess;
use crate::error_adapter::log_and_return_error_string;
use anyhow::{Context, Result};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::Path;
use std::str;

pub struct TextIndexer;

impl Indexer for TextIndexer {
    fn supports_extension(&self, extension: &OsStr) -> bool {
        extension == OsStr::new("txt")
    }

    fn supported_extensions(&self) -> Vec<OsString> {
        vec![OsString::from("txt")]
    }

    fn index_file(&self, file_to_process: &FileToProcess) -> Result<DocumentSchema> {
        let name = file_to_process
            .path
            .file_name()
            .unwrap()
            .to_os_string()
            .into_string()
            .expect(&log_and_return_error_string(format!(
                "text_indexer: Failed to get file name for file at path: {:?}",
                file_to_process.path
            )));

        let body = str::from_utf8(&file_to_process.contents).with_context(|| {
            log_and_return_error_string(format!(
                "text_indexer: Failed to read file to string at path: {:?}",
                file_to_process.path
            ))
        })?;

        Ok(DocumentSchema {
            name: name,
            body: body.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indexing_text_file() {
        let test_file_path = Path::new("./test_files/file.txt");
        let indexed_document = TextIndexer
            .index_file(&FileToProcess::from(test_file_path))
            .unwrap();

        assert_eq!(indexed_document.name, "file.txt");
        assert_eq!(
            indexed_document.body,
            "this is a file with some contents in it"
        );
    }

    #[test]
    fn test_supports_text_extension() {
        assert_eq!(true, TextIndexer.supports_extension(OsStr::new("txt")));
        assert_eq!(false, TextIndexer.supports_extension(OsStr::new("png")));
    }
}
