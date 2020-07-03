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

use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::iter::FromIterator;

use anyhow::Result;
use once_cell::sync::Lazy;

use crate::contracts::file_to_process::FileToProcess;

/// The schema of the information that an Indexer extracts from a file
#[derive(Debug)]
pub struct DocumentSchema {
    pub name: String,
    pub body: String,
}

/// Each Indexer needs to be able to say if a file extension is supported and extract information from a supported file
pub trait Indexer: Send + Sync {
    /// If the Indexer supports a file extension
    /// Eg: PdfIndexer supports .pdf extensions
    fn supports_extension(&self, extension: &OsStr) -> bool;

    /// The logic behind the Indexer to extract information from a file
    fn index_file(&self, file_to_process: &FileToProcess) -> Result<DocumentSchema>;

    fn supported_extensions(&self) -> Vec<OsString>;
}

/// Container for all Indexers
pub struct Analyzer {
    pub supported_extensions: HashSet<OsString>,
}

impl Default for Analyzer {
    #[cfg(not(target_os = "windows"))]
    fn default() -> Analyzer {
        let indexers: Vec<Box<dyn Indexer>> = vec![
            Box::new(TextIndexer),
            Box::new(ExifIndexer),
            Box::new(PdfIndexer),
            Box::new(MobileNetV2Indexer),
            Box::new(PptxIndexer),
            Box::new(CsvIndexer),
            Box::new(SpreadsheetIndexer),
        ];

        let supported_extensions = HashSet::from_iter(
            indexers
                .iter()
                .map(|indexer| indexer.supported_extensions())
                .flatten(),
        );

        Analyzer {
            supported_extensions: supported_extensions,
        }
    }

    #[cfg(target_os = "windows")]
    fn default() -> Analyzer {
        let indexers: Vec<Box<dyn Indexer>> = vec![
            Box::new(TextIndexer),
            Box::new(ExifIndexer),
            Box::new(PdfIndexer),
            Box::new(PptxIndexer),
            Box::new(CsvIndexer),
            Box::new(SpreadsheetIndexer),
        ];

        let supported_extensions = HashSet::from_iter(
            indexers
                .iter()
                .map(|indexer| indexer.supported_extensions())
                .flatten(),
        );

        Analyzer {
            supported_extensions: supported_extensions,
        }
    }
}

#[cfg(not(target_os = "windows"))]
static INDEXERS: Lazy<Vec<Box<dyn Indexer>>> = Lazy::new(|| {
    let indexers: Vec<Box<dyn Indexer>> = vec![
        Box::new(TextIndexer),
        Box::new(ExifIndexer),
        Box::new(PdfIndexer),
        Box::new(MobileNetV2Indexer),
        Box::new(PptxIndexer),
        Box::new(CsvIndexer),
        Box::new(SpreadsheetIndexer),
    ];
    indexers
});

#[cfg(target_os = "windows")]
static INDEXERS: Lazy<Vec<Box<dyn Indexer>>> = Lazy::new(|| {
    let indexers: Vec<Box<dyn Indexer>> = vec![
        Box::new(TextIndexer),
        Box::new(ExifIndexer),
        Box::new(PdfIndexer),
        Box::new(MobileNetV2Indexer),
        Box::new(PptxIndexer),
        Box::new(CsvIndexer),
        Box::new(SpreadsheetIndexer),
    ];
    indexers
});

pub async fn analyze(extension: OsString, file_to_process: FileToProcess) -> Vec<DocumentSchema> {
    let processing_task = tokio::task::spawn_blocking(move || {
        INDEXERS
            .iter()
            .filter(|indexer| indexer.supports_extension(extension.as_os_str()))
            .filter_map(|indexer| indexer.index_file(&file_to_process).ok())
            .collect()
    });

    processing_task.await.unwrap()
}
