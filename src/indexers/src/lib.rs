// mod exif_indexer;
// mod mobile_net_v2_indexer;
// mod pdf_indexer;
// mod text_indexer;
// // mod docx_indexer;
// mod csv_indexer;
// mod pptx_indexer;
// mod spreadsheet_indexer;

use exif_indexer::exif_indexer::ExifIndexer;
use mobile_net_v2_indexer::mobile_net_v2_indexer::MobileNetV2Indexer;
//use pdf_indexer::PdfIndexer;
use text_indexer::text_indexer::TextIndexer;
// pub use self::docx_indexer::DocxIndexer;
use csv_indexer::csv_indexer::CsvIndexer;
use pptx_indexer::pptx_indexer::PptxIndexer;
use spreadsheet_indexer::spreadsheet_indexer::SpreadsheetIndexer;

use std::collections::HashSet;
use std::ffi::OsString;
use std::iter::FromIterator;

use common::tokio;
use common::tracing::instrument;
use common::tracing_futures;
use once_cell::sync::Lazy;

use contracts::file_to_process::FileToProcess;
use contracts::indexer::{DocumentSchema, Indexer};

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
            // Box::new(PdfIndexer),
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
            // Box::new(PdfIndexer),
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

static INDEXERS: Lazy<Vec<Box<dyn Indexer>>> = Lazy::new(|| {
    let indexers: Vec<Box<dyn Indexer>> = vec![
        Box::new(TextIndexer),
        Box::new(ExifIndexer),
        // Box::new(PdfIndexer),
        Box::new(MobileNetV2Indexer),
        Box::new(PptxIndexer),
        Box::new(CsvIndexer),
        Box::new(SpreadsheetIndexer),
    ];
    indexers
});

#[instrument(skip(file_to_process))]
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
