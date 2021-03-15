use common::anyhow::{Context, Error, Result};
use common::error_adapter::log_and_return_error_string;
use common::tracing::{span, Level};
use contracts::file_to_process::FileToProcess;
use contracts::indexer::{DocumentSchema, Indexer};
use std::ffi::{OsStr, OsString};

use pdf_extract::*;
use regex::Regex;

pub struct PdfIndexer;

impl Indexer for PdfIndexer {
    fn supports_extension(&self, extension: &OsStr) -> bool {
        extension == OsStr::new("pdf")
    }

    fn supported_extensions(&self) -> Vec<OsString> {
        vec![OsString::from("pdf")]
    }

    fn index_file(&self, file_to_process: &FileToProcess) -> Result<DocumentSchema> {
        let path = file_to_process.path.to_str().unwrap();
        span!(Level::INFO, "pdf_indexer: indexing pdf file", path).in_scope(|| {
            let res = span!(Level::INFO, "pdf_indexer: Loading from disk and processing")
                .in_scope(|| {
                    // TODO: the resulting string from this is poorly extracted
                    // better than nothing but it should be fixed
                    extract_text(&file_to_process.path).with_context(|| {
                        log_and_return_error_string(format!(
                            "pdf_indexer: Failed to extract text from pdf at path: {:?}",
                            file_to_process.path
                        ))
                    })
                })?;

            let clean = span!(Level::INFO, "pdf_indexer: Processing file").in_scope(
                || -> Result<String, Error> {
                    // THIS IS A BAD HACK
                    let re = Regex::new(r"\b ").with_context(|| {
                        log_and_return_error_string(format!("pdf_indexer: Failed to create regex"))
                    })?;

                    Ok(re.replace_all(&res, "").to_string())
                },
            )?;

            Ok(DocumentSchema {
                name: String::new(),
                body: clean,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contracts::file_to_process::new_file_to_process;

    use std::path::Path;

    // #[tokio::test(core_threads = 1)]
    // async fn test_indexing_pdf_file() {
    //     let test_file_path = Path::new("./test_files/Cats.pdf");
    //     let indexed_document = PdfIndexer
    //         .index_file(&new_file_to_process(test_file_path).await)
    //         .unwrap();

    //     assert_eq!(indexed_document.name, "");
    //     assert_eq!(indexed_document.body, "\n\nCats \n\nThis  is  an  example  document about cats.  \n\n \n\nCats  have  paws.  ");
    // }

    #[test]
    fn test_supports_pdf_extension() {
        assert_eq!(true, PdfIndexer.supports_extension(OsStr::new("pdf")));
        assert_eq!(false, PdfIndexer.supports_extension(OsStr::new("docx")))
    }
}
