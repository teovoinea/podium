use contracts::file_to_process::{new_file_to_process, FileToProcess};
use custom_tantivy::wrapper::*;

use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use tracing::info;
use walkdir::{DirEntry, WalkDir};

use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;

/// Starts the file watcher thread
/// Reacts to document changes (create/update/delete)
/// Does appropriate housekeeping for documents (eg: removing old documents after update)
pub async fn start_watcher(directories: &Vec<PathBuf>, tantivy_wrapper: &mut TantivyWrapper) {
    info!("Starting file watcher thread on: {:?}", directories);
    let (watcher_tx, watcher_rx) = channel();
    let mut watcher = watcher(watcher_tx, Duration::from_secs(10)).unwrap();

    // Start watching all directories in the config file
    for directory in directories {
        watcher.watch(directory, RecursiveMode::Recursive).unwrap();
    }

    loop {
        let watcher_event = watcher_rx.recv();
        match watcher_event {
            Ok(event) => {
                info!("Received watcher event: {:?}", event);
                match event {
                    DebouncedEvent::Create(path_buf) => {
                        create_event(path_buf, &tantivy_wrapper).await;
                    }
                    DebouncedEvent::Write(path_buf) => {
                        write_event(path_buf, &tantivy_wrapper).await;
                    }
                    DebouncedEvent::NoticeRemove(path_buf) => {
                        remove_event(&path_buf, &tantivy_wrapper);
                    }
                    DebouncedEvent::Rename(src_path_buf, dst_path_buf) => {
                        rename_event(&src_path_buf, &dst_path_buf, &tantivy_wrapper);
                        // TODO: Figure out if you can just update the facet without reprocessing the whole document?
                    }
                    _ => {
                        // Ignore the rest for now? Not sure...
                    }
                }
            }
            Err(e) => error!("watch error: {:?}", e),
        }
        tantivy_wrapper.index_writer.commit().unwrap();
    }
}

/// Handles a create event from watch_dir
/// If a folder is created, recursively process all files in the folder
/// Otherwise process the single new file which was created
async fn create_event(path_buf: PathBuf, tantivy_wrapper: &TantivyWrapper) {
    if path_buf.is_dir() {
        // Traverse through all the files in the directory
        let walker = WalkDir::new(path_buf).into_iter();
        for entry in walker.filter_entry(|e| !is_hidden(e)) {
            let entry = entry.unwrap();
            create(
                new_file_to_process(entry.into_path()).await,
                tantivy_wrapper,
            )
            .await;
        }
    } else {
        create(new_file_to_process(path_buf).await, tantivy_wrapper).await;
    }
}

/// Processes a newly created file
/// If the hash has been seen before, skip processing and simply add the new location to the tantivy document
/// Otherwise process the file and create the new document
async fn create(file_to_process: FileToProcess, tantivy_wrapper: &TantivyWrapper) {
    tantivy_wrapper.process_file(file_to_process).await;
}

/// Handles a write event from watch_dir
/// If a folder is written, recursively process all files in the folder
/// Otherwise process the single file which was written
async fn write_event(path_buf: PathBuf, tantivy_wrapper: &TantivyWrapper) {
    // Remove the old document, reprocess and add the new content
    if path_buf.is_dir() {
        // Traverse through all the files in the directory
        let walker = WalkDir::new(path_buf).into_iter();
        for entry in walker.filter_entry(|e| !is_hidden(e)) {
            let entry = entry.unwrap();
            write(
                new_file_to_process(entry.into_path()).await,
                tantivy_wrapper,
            )
            .await;
        }
    } else {
        write(new_file_to_process(path_buf).await, tantivy_wrapper).await;
    }
}

/// Processes a newly written file
/// Removes the old document related to the file
/// Reprocesses the file
async fn write(file_to_process: FileToProcess, tantivy_wrapper: &TantivyWrapper) {
    let path_buf = file_to_process.path.clone();

    // Remove the old document
    remove(&path_buf, tantivy_wrapper);

    tantivy_wrapper.process_file(file_to_process).await;
}

/// Handles a remove event from watch_dir
/// If a folder is removed, recursively remove all files in the folder
/// Otherwise remove the single file
fn remove_event(path_buf: &PathBuf, tantivy_wrapper: &TantivyWrapper) {
    if path_buf.is_dir() {
        // Traverse through all the files in the directory
        let walker = WalkDir::new(path_buf).into_iter();
        for entry in walker.filter_entry(|e| !is_hidden(e)) {
            let entry = entry.unwrap();
            remove(&entry.into_path(), tantivy_wrapper);
        }
    } else {
        remove(path_buf, tantivy_wrapper);
    }
}

/// Removes the document which contains this given location
/// If the document contains multiple locations (same file hash in different locations)
/// Only remove this location from the list of locations
fn remove(path_buf: &PathBuf, tantivy_wrapper: &TantivyWrapper) {
    tantivy_wrapper.remove(path_buf);
}

/// TODO: Implement
fn rename_event(
    _src_path_buf: &PathBuf,
    _dst_path_buf: &PathBuf,
    _tantivy_wrapper: &TantivyWrapper,
) {
    unimplemented!();
}

/// Checks if a file or directory is hidden
pub fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}
