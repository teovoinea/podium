mod exif_indexer;
#[cfg(not(target_os = "windows"))]
mod mobile_net_v2_indexer;
mod pdf_indexer;
mod text_indexer;
// mod docx_indexer;
mod csv_indexer;
mod pptx_indexer;
mod spreadsheet_indexer;

pub use self::exif_indexer::ExifIndexer;
#[cfg(not(target_os = "windows"))]
pub use self::mobile_net_v2_indexer::MobileNetV2Indexer;
pub use self::pdf_indexer::PdfIndexer;
pub use self::text_indexer::TextIndexer;
// pub use self::docx_indexer::DocxIndexer;
pub use self::csv_indexer::CsvIndexer;
pub use self::pptx_indexer::PptxIndexer;
pub use self::spreadsheet_indexer::SpreadsheetIndexer;

use std::ffi::OsStr;
use std::path::Path;

use anyhow::Result;

/// The schema of the information that an Indexer extracts from a file
#[derive(Debug)]
pub struct DocumentSchema {
    pub name: String,
    pub body: String,
}

/// Each Indexer needs to be able to say if a file extension is supported and extract information from a supported file
pub trait Indexer {
    /// If the Indexer supports a file extension
    /// Eg: PdfIndexer supports .pdf extensions
    fn supports_extension(&self, extension: &OsStr) -> bool;

    /// The logic behind the Indexer to extract information from a file
    fn index_file(&self, path: &Path) -> Result<DocumentSchema>;
}

/// Container for all Indexers
pub struct Analyzer {
    pub indexers: Vec<Box<dyn Indexer>>,
}

impl Analyzer {
    /// Applies the indexing function of Indexers that support the given extension
    pub fn analyze<P>(&self, extension: &OsStr, path: P) -> Vec<DocumentSchema> where P: AsRef<Path> {
        self.indexers
            .iter()
            .filter(|indexer| indexer.supports_extension(extension))
            .filter_map(|indexer| indexer.index_file(path.as_ref()).ok())
            .collect()
    }
}

impl Default for Analyzer {
    #[cfg(not(target_os = "windows"))]
    fn default() -> Analyzer {
        Analyzer {
            indexers: vec![
                Box::new(TextIndexer),
                Box::new(ExifIndexer),
                Box::new(PdfIndexer),
                Box::new(MobileNetV2Indexer),
                Box::new(PptxIndexer),
                Box::new(CsvIndexer),
                Box::new(SpreadsheetIndexer),
            ],
        }
    }

    #[cfg(target_os = "windows")]
    fn default() -> Analyzer {
        Analyzer {
            indexers: vec![
                Box::new(TextIndexer),
                Box::new(ExifIndexer),
                Box::new(PdfIndexer),
                Box::new(PptxIndexer),
                Box::new(CsvIndexer),
                Box::new(SpreadsheetIndexer),
            ],
        }
    }
}
