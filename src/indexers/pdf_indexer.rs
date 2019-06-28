use super::Indexer;
use super::DocumentSchema;
use std::path::Path;
use std::ffi::OsStr;

use pdf_extract::*;
use regex::Regex;

pub struct PdfIndexer;

impl Indexer for PdfIndexer {
    fn supports_extension(&self, extension: &OsStr) -> bool {
        extension == OsStr::new("pdf")
    }

    fn index_file(&self, path: &Path) -> DocumentSchema {
        // TODO: the resulting string from this is poorly extracted
        // better than nothing but it should be fixed
        let res = extract_text(path).unwrap();

        // THIS IS A BAD HACK
        let re = Regex::new(r"\b ").unwrap();
        let clean = re.replace_all(&res, "").to_string();

        DocumentSchema {
            name: String::new(),
            body: clean
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indexing_pdf_file() {
        let test_file_path = Path::new("./test_files/Cats.pdf");
        let indexed_document = PdfIndexer.index_file(test_file_path);
        
        assert_eq!(indexed_document.name, "");
        assert_eq!(indexed_document.body, "\n\nCats \n\nThis  is  an  example  document about cats.  \n\n \n\nCats  have  paws.  ");
    }

    #[test]
    fn test_supports_pdf_extension() {
        assert_eq!(true, PdfIndexer.supports_extension(OsStr::new("pdf")));
        assert_eq!(false, PdfIndexer.supports_extension(OsStr::new("docx")))
    }
}