use super::DocumentSchema;
use super::Indexer;
use crate::contracts::file_to_process::FileToProcess;
use crate::error_adapter::log_and_return_error_string;
use anyhow::{Context, Result};
use std::ffi::{OsStr, OsString};
use std::str;
use tracing::{span, Level};

pub struct TextIndexer;

impl Indexer for TextIndexer {
    fn supports_extension(&self, extension: &OsStr) -> bool {
        extension == OsStr::new("txt")
    }

    fn supported_extensions(&self) -> Vec<OsString> {
        vec![OsString::from("txt")]
    }

    fn index_file(&self, file_to_process: &FileToProcess) -> Result<DocumentSchema> {
        span!(Level::INFO, "text_indexer: indexing text file").in_scope(|| {
            let name = file_to_process
                .path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();

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
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::file_to_process::new_file_to_process;

    use std::path::Path;

    #[tokio::test(core_threads = 1)]
    async fn test_indexing_text_file() {
        let test_file_path = Path::new("./test_files/file.txt");
        let indexed_document = TextIndexer
            .index_file(&new_file_to_process(test_file_path).await)
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
