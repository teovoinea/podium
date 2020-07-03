use super::DocumentSchema;
use super::Indexer;
use crate::contracts::file_to_process::FileToProcess;
use crate::error_adapter::log_and_return_error_string;
use anyhow::{Context, Result};
use std::ffi::{OsStr, OsString};
use std::io::Cursor;

pub struct CsvIndexer;

impl Indexer for CsvIndexer {
    fn supports_extension(&self, extension: &OsStr) -> bool {
        extension == OsStr::new("csv")
    }

    fn supported_extensions(&self) -> Vec<OsString> {
        vec![OsString::from("csv")]
    }

    fn index_file(&self, file_to_process: &FileToProcess) -> Result<DocumentSchema> {
        let mut reader = csv::Reader::from_reader(Cursor::new(&file_to_process.contents));

        let headers = reader
            .headers()
            .with_context(|| {
                log_and_return_error_string(format!(
                    "csv_indexer: Failed to get headers from csv at path: {:?}",
                    file_to_process.path
                ))
            })?
            .iter()
            .fold(String::new(), |mut acc, x| {
                acc.push_str(&x);
                acc.push_str(" ");
                acc
            });

        Ok(DocumentSchema {
            name: String::new(),
            body: headers,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::file_to_process::new_file_to_process;
    use std::path::Path;

    #[tokio::test(core_threads = 1)]
    async fn test_indexing_csv_file() {
        let test_file_path = Path::new("./test_files/data.csv");
        let indexed_document = CsvIndexer
            .index_file(&new_file_to_process(test_file_path).await)
            .unwrap();

        assert_eq!(indexed_document.name, "");
        assert_eq!(
            indexed_document.body,
            "first_name last_name street city state postal_code "
        );
    }

    #[test]
    fn test_supports_csv_extension() {
        assert_eq!(true, CsvIndexer.supports_extension(OsStr::new("csv")));
        assert_eq!(false, CsvIndexer.supports_extension(OsStr::new("xslx")));
    }
}
