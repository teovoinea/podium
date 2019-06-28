use super::Indexer;
use super::DocumentSchema;
use std::path::Path;
use std::ffi::OsStr;
use std::fs;

use calamine::{Reader, open_workbook, Xlsx, DataType};


pub struct SpreadsheetIndexer;

impl Indexer for SpreadsheetIndexer {
    fn supports_extension(&self, extension: &OsStr) -> bool {
        // Only xslx for now
        extension == OsStr::new("xlsx")
    }

    fn index_file(&self, path: &Path) -> DocumentSchema {
        let mut workbook: Xlsx<_> = open_workbook(path).expect("Cannot open file");
        let sheet_names = workbook.sheet_names().to_vec();

        let strings = sheet_names.iter()
                    .filter_map(|sheet_name| workbook.worksheet_range(sheet_name))
                    .filter_map(Result::ok)
                    .map(|range| {
                        range.used_cells()
                            .filter(|(_, _, cell)| cell.is_string())
                            .filter_map(|(_, _, cell)| cell.get_string())
                            .map(|val| val.to_string())
                            .collect::<Vec<String>>()
                    })
                    .flatten()
                    .fold(String::new(), |mut acc, x| { acc.push_str(&x); acc.push_str(" "); acc });

        DocumentSchema {
            name: String::new(),
            body: strings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indexing_spreadsheet_file() {
        let test_file_path = Path::new("./test_files/Cats.xlsx");
        let indexed_document = SpreadsheetIndexer.index_file(test_file_path);

        assert_eq!(indexed_document.name, "");
        assert_eq!(indexed_document.body, "this sheet is about cats cats have paws they\'re pretty cool Horses are also an animal Horses don\'t have paws Weird isn\'t it? ");
    }

    #[test]
    fn test_supports_spreadsheet_extension() {
        assert_eq!(true, SpreadsheetIndexer.supports_extension(OsStr::new("xlsx")));
        assert_eq!(false, SpreadsheetIndexer.supports_extension(OsStr::new("xls"))); // not yet..
    }
}