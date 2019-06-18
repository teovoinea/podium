use super::Indexer;
use super::DocumentSchema;
use std::path::Path;
use std::ffi::OsStr;
use std::fs;


pub struct TextIndexer;

impl Indexer for TextIndexer {
    fn supports_extension(&self, extension: &OsStr) -> bool {
        extension == OsStr::new("txt")
    }

    fn index_file(&self, path: &Path) -> DocumentSchema {
        DocumentSchema {
            name: path.file_name().unwrap().to_os_string().into_string().unwrap(),
            body: fs::read_to_string(path).unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indexing_text_file() {
        let test_file_path = Path::new("./test_files/file.txt");
        let indexed_document = TextIndexer.index_file(test_file_path);

        assert_eq!(indexed_document.name, "file.txt");
        assert_eq!(indexed_document.body, "this is a file with some contents in it");
    }

    #[test]
    fn test_supports_text_extension() {
        assert_eq!(true, TextIndexer.supports_extension(OsStr::new("txt")));
        assert_eq!(false, TextIndexer.supports_extension(OsStr::new("png")));
    }
}