use super::DocumentSchema;
use super::Indexer;
use crate::contracts::file_to_process::FileToProcess;
use crate::error_adapter::log_and_return_error_string;
use anyhow::{Context, Result};
use std::ffi::{OsStr, OsString};
use std::path::Path;

use calamine::{open_workbook, Reader, Xlsx};

pub struct SpreadsheetIndexer;

impl Indexer for SpreadsheetIndexer {
    fn supports_extension(&self, extension: &OsStr) -> bool {
        // Only xslx for now
        extension == OsStr::new("xlsx")
    }

    fn supported_extensions(&self) -> Vec<OsString> {
        vec![OsString::from("xlsx")]
    }

    fn index_file(&self, file_to_process: &FileToProcess) -> Result<DocumentSchema> {
        let mut workbook: Xlsx<_> =
            open_workbook(&file_to_process.path).expect(&log_and_return_error_string(format!(
                "spreadsheet_indexer: Failed to open workbook at path: {:?}",
                file_to_process.path
            )));
        let sheet_names = workbook.sheet_names().to_vec();

        let strings = sheet_names
            .iter()
            .filter_map(|sheet_name| workbook.worksheet_range(sheet_name))
            .filter_map(Result::ok)
            .map(|range| {
                range
                    .used_cells()
                    .filter(|(_, _, cell)| cell.is_string())
                    .filter_map(|(_, _, cell)| cell.get_string())
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<String>>()
            })
            .flatten()
            .fold(String::new(), |mut acc, x| {
                acc.push_str(&x);
                acc.push_str(" ");
                acc
            });

        Ok(DocumentSchema {
            name: String::new(),
            body: strings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::file_to_process::newFileToProcess;

    #[tokio::test(core_threads = 1)]
    async fn test_indexing_spreadsheet_file() {
        let test_file_path = Path::new("./test_files/Cats.xlsx");
        let indexed_document = SpreadsheetIndexer
            .index_file(&newFileToProcess(test_file_path).await)
            .unwrap();

        assert_eq!(indexed_document.name, "");
        assert_eq!(indexed_document.body, "this sheet is about cats cats have paws they\'re pretty cool Horses are also an animal Horses don\'t have paws Weird isn\'t it? ");
    }

    #[test]
    fn test_supports_spreadsheet_extension() {
        assert_eq!(
            true,
            SpreadsheetIndexer.supports_extension(OsStr::new("xlsx"))
        );
        assert_eq!(
            false,
            SpreadsheetIndexer.supports_extension(OsStr::new("xls"))
        ); // not yet..
    }
}
