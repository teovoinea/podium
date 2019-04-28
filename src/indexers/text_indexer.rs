use super::Indexer;
use super::DocumentSchema;
use std::path::Path;
use std::ffi::OsStr;
use std::fs;


pub struct TextIndexer {}

impl Indexer for TextIndexer {
    fn supports_extension(extension: &OsStr) -> bool {
        extension == OsStr::new("txt")
    }

    fn index_file(path: &Path) -> DocumentSchema {
        DocumentSchema {
            name: path.file_name().unwrap().to_os_string().into_string().unwrap(),
            body: fs::read_to_string(path).unwrap(),
        }
    }
}