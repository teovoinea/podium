use crate::contracts::file_to_process::new_file_to_process;
use crate::file_watcher::*;
use crate::indexers::Analyzer;
use crate::searcher::Searcher;
use crate::tantivy_wrapper::*;

use tantivy::directory::*;
use tantivy::{Index, ReloadPolicy};
use walkdir::WalkDir;

use std::collections::HashMap;
use std::path::PathBuf;

/// Starts tantivy thread
/// Starts file watcher thread
/// Does initial file processing
/// Starts reader thread
/// Owns tantivy's index_writer so it's able to write/delete documents
/// TODO: This function does too much? Should break it up
pub async fn start_tantivy(
    settings: HashMap<String, Vec<String>>,
    tantivy_wrapper: &mut TantivyWrapper,
) -> tantivy::Result<()> {
    let directories = settings.get("directories").unwrap();
    let directories_clone = directories.clone();

    let initial_processing_done: bool = settings.get("initial_processing").unwrap()[0]
        .parse()
        .unwrap();

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
    // After we finished doing the initial processing, add the file so that we know for next time
    // TODO: Create initial processing file
    // fs::File::create(initial_processing_file).unwrap();
    } else {
        info!("Initial processing already done! Starting a reader");
    }

    start_watcher(directories_clone, tantivy_wrapper).await;

    Ok(())
}

pub fn tantivy_init(
    settings: &HashMap<String, Vec<String>>,
) -> tantivy::Result<(Searcher, TantivyWrapper)> {
    let index_path = PathBuf::from(settings.get("index_path").unwrap()[0].clone());

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
