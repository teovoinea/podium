use super::DocumentSchema;
use super::Indexer;
use std::ffi::OsStr;
use std::path::Path;

pub struct CsvIndexer;

impl Indexer for CsvIndexer {
    fn supports_extension(&self, extension: &OsStr) -> bool {
        extension == OsStr::new("csv")
    }

    fn index_file(&self, path: &Path) -> DocumentSchema {
        let mut reader = csv::Reader::from_path(path).unwrap();
        if reader.has_headers() {
            let headers = reader
                .headers()
                .unwrap()
                .iter()
                .fold(String::new(), |mut acc, x| {
                    acc.push_str(&x);
                    acc.push_str(" ");
                    acc
                });
            DocumentSchema {
                name: String::new(),
                body: headers,
            }
        } else {
            DocumentSchema {
                name: String::new(),
                body: String::new(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indexing_csv_file() {
        let test_file_path = Path::new("./test_files/data.csv");
        let indexed_document = CsvIndexer.index_file(test_file_path);

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
