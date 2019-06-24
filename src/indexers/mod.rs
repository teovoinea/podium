mod text_indexer;
mod exif_indexer;
mod pdf_indexer;
mod mobile_net_v2_indexer;
// mod docx_indexer;
mod pptx_indexer;
mod csv_indexer;
mod spreadsheet_indexer;

use std::path::Path;
use std::ffi::OsStr;
pub use self::text_indexer::TextIndexer;
pub use self::exif_indexer::ExifIndexer;
pub use self::pdf_indexer::PdfIndexer;
pub use self::mobile_net_v2_indexer::MobileNetV2Indexer;
// pub use self::docx_indexer::DocxIndexer;
pub use self::pptx_indexer::PptxIndexer;
pub use self::csv_indexer::CsvIndexer;
pub use self::spreadsheet_indexer::SpreadsheetIndexer;

#[derive(Debug)]
pub struct DocumentSchema {
    pub name: String,
    pub body: String,
}

pub trait Indexer {
    fn supports_extension(&self, extension: &OsStr) -> bool;

    fn index_file(&self, path: &Path) -> DocumentSchema;
}

pub struct Analyzer {
    pub indexers: Vec<Box<dyn Indexer>>,
}

impl Analyzer {
    pub fn analyze(&self, extension: &OsStr, path: &Path) -> Vec<DocumentSchema> {
        self.indexers.iter()
            .filter(|indexer| indexer.supports_extension(extension))
            .map(|indexer| indexer.index_file(path))
            .collect()
    }
}