use common::anyhow;
use common::anyhow::Result;
use common::tracing::{span, Level};
use contracts::file_to_process::FileToProcess;
use contracts::indexer::{DocumentSchema, Indexer};
use std::ffi::{OsStr, OsString};

use msoffice_pptx::document::PPTXDocument;
use msoffice_pptx::pml::ShapeGroup;
use msoffice_shared::drawingml::TextRun;

pub struct PptxIndexer;

impl Indexer for PptxIndexer {
    fn supports_extension(&self, extension: &OsStr) -> bool {
        extension == OsStr::new("pptx")
    }

    fn supported_extensions(&self) -> Vec<OsString> {
        vec![OsString::from("pptx")]
    }

    fn index_file(&self, file_to_process: &FileToProcess) -> Result<DocumentSchema> {
        let path = file_to_process.path.to_str().unwrap();
        span!(Level::INFO, "pptx_indexer: indexing powerpoint file", path).in_scope(|| {
            let mut total_text = String::new();
            let document = span!(Level::INFO, "pptx_indexer: Load from disk").in_scope(|| {
                match PPTXDocument::from_file(file_to_process.path.as_path()) {
                    Ok(doc) => Ok(doc),
                    Err(e) => Err(anyhow::anyhow!(format!(
                        "pptx_indexer: Failed to open PPTX Document from file at path: {:?} with additional error info {:?}",
                        file_to_process.path,
                        e
                    )))
                }
            })?;

            span!(Level::INFO, "pptx_indexer: Process file").in_scope(|| {
                for slide in document.slide_map.values() {
                    let shape_group = &(*(*slide.common_slide_data).shape_tree).shape_array;
                    for s_g in shape_group {
                        if let Some(res_text) = extract_text(s_g) {
                            total_text.push_str(&res_text);
                        }
                    }
                }
            });

            Ok(DocumentSchema {
                name: String::new(),
                body: total_text,
            })
        })
    }
}

fn extract_text(shape_group: &ShapeGroup) -> Option<String> {
    let mut total_text = String::new();
    match shape_group {
        ShapeGroup::Shape(shape) => {
            if let Some(text_body) = &shape.text_body {
                for paragraph in &text_body.paragraph_array {
                    for text_run in &paragraph.text_run_list {
                        if let TextRun::RegularTextRun(regular_text_run) = text_run {
                            total_text.push_str(&regular_text_run.text);
                            total_text.push_str(" ");
                        }
                    }
                }
            }
        }
        ShapeGroup::GroupShape(group_shape) => {
            let res_text = group_shape
                .shape_array
                .iter()
                .map(|s_g| extract_text(s_g))
                .filter_map(|r_t| r_t)
                .fold(String::new(), |mut acc, x| {
                    acc.push_str(&x);
                    acc.push_str(" ");
                    acc
                });

            total_text.push_str(&res_text);
        }
        _ => {}
    }
    if total_text != String::new() {
        Some(total_text)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::tokio;
    use contracts::file_to_process::new_file_to_process;

    use std::path::Path;

    #[tokio::test(core_threads = 1)]
    async fn test_indexing_pptx_file() {
        let test_file_path = Path::new("./test_files/Cats.pptx");
        let indexed_document = PptxIndexer
            .index_file(&new_file_to_process(test_file_path).await)
            .unwrap();

        assert_eq!(indexed_document.name, "");
        assert!(indexed_document.body.contains("Cats"));
        assert!(indexed_document.body.contains("quick"));
        assert!(indexed_document.body.contains("story"));
        assert!(indexed_document.body.contains("Paws"));
        assert!(indexed_document.body.contains("cool"));
    }

    #[test]
    fn test_supports_pptgx_extension() {
        assert_eq!(true, PptxIndexer.supports_extension(OsStr::new("pptx")));
        assert_eq!(false, PptxIndexer.supports_extension(OsStr::new("ppt")));
    }
}
