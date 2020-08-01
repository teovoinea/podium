use crate::contracts::file_to_process::new_file_to_process;
use crate::custom_tantivy::{utils::build_schema, wrapper::*};
use crate::file_watcher::*;
use crate::indexers::Analyzer;
use crate::searcher::Searcher;

use tantivy::directory::*;
use tantivy::{Index, ReloadPolicy};
use walkdir::WalkDir;

use std::fs;
use std::path::PathBuf;

pub struct TantivyConfig {
    pub scan_directories: Vec<PathBuf>,
    pub initial_processing_file: PathBuf,
    pub index_path: PathBuf,
}

/// Starts watching directories
/// Does initial processing
/// Consumes watcher events to continue processing files
pub async fn start_tantivy(
    settings: &TantivyConfig,
    tantivy_wrapper: &mut TantivyWrapper,
) -> tantivy::Result<()> {
    let directories = &settings.scan_directories;
    let directories_clone = directories.clone();

    let initial_processing_done = settings.initial_processing_file.exists();

    let analyzer = Analyzer::default();

    if !initial_processing_done {
        info!("Initial processing was not previously done, doing now");
        for directory in directories {
            let walker = WalkDir::new(directory).into_iter();
            for entry in walker.filter_entry(|e| !is_hidden(e)) {
                match entry {
                    Err(_) => {
                        error!("Failed to read entry from dir walker: {:?}", entry);
                        continue;
                    }
                    _ => {}
                }
                let entry = entry.unwrap();
                if !entry.file_type().is_dir() {
                    let entry_path = entry.path();

                    match entry_path.extension() {
                        None => continue,
                        Some(extension) => {
                            if !analyzer.supported_extensions.contains(extension) {
                                continue;
                            }
                        }
                    }

                    let file_to_process = new_file_to_process(entry_path).await;

                    tantivy_wrapper.process_file(file_to_process).await;
                    tantivy_wrapper.index_writer.commit()?;
                }
            }
        }
        fs::File::create(&settings.initial_processing_file).unwrap();
    } else {
        info!("Initial processing already done! Starting a reader");
    }

    start_watcher(&directories_clone, tantivy_wrapper).await;

    Ok(())
}

pub fn tantivy_init(settings: &TantivyConfig) -> tantivy::Result<(Searcher, TantivyWrapper)> {
    let index_path = &settings.index_path;

    let schema = build_schema();

    let index = Index::open_or_create(MmapDirectory::open(&index_path).unwrap(), schema.clone())?;

    let index_reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;

    let index_writer = index.writer(50_000_000)?;

    let searcher = Searcher::new(index, index_reader.clone(), schema.clone());

    let tantivy_wrapper = TantivyWrapper::new(index_reader, index_writer, schema);

    Ok((searcher, tantivy_wrapper))
}
