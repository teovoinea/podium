use crate::contracts::file_to_process::FileToProcess;
use crate::file_watcher::*;
use crate::query_executor::*;
use crate::tantivy_api::*;

use app_dirs::*;
use config::*;
use crossbeam::channel::unbounded;
use crossbeam::channel::{Receiver, Sender};
use tantivy::directory::*;
use tantivy::{Index, ReloadPolicy, Term};
use walkdir::WalkDir;

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::thread;

const APP_INFO: AppInfo = AppInfo {
    name: "Podium",
    author: "Teodor Voinea",
};

/// Starts tantivy thread
/// Starts file watcher thread
/// Does initial file processing
/// Starts reader thread
/// Owns tantivy's index_writer so it's able to write/delete documents
/// TODO: This function does too much? Should break it up
pub fn start_tantivy(
    query_channel: (Sender<String>, Receiver<String>),
    result_tx: Sender<QueryResponse>,
) -> tantivy::Result<()> {
    let index_path = app_dir(AppDataType::UserData, &APP_INFO, "index").unwrap();
    info!("Using index file in: {:?}", index_path);

    let state_path = app_dir(AppDataType::UserData, &APP_INFO, "state").unwrap();
    let mut initial_processing_file = state_path.clone();
    initial_processing_file.push("initial_processing");

    let config_path = app_dir(AppDataType::UserConfig, &APP_INFO, "config").unwrap();
    let mut config_file = config_path.clone();
    config_file.push("config");
    config_file.set_extension("json");

    if !config_file.as_path().exists() {
        info!("Config file not found, copying default config");
        let default_config_path = Path::new("debug_default_config.json");
        fs::copy(default_config_path, &config_file).unwrap();
    }

    info!("Loading config file from: {:?}", config_file);
    let mut settings = Config::default();
    settings.merge(File::from(config_file)).unwrap();
    let settings_dict = settings.try_into::<HashMap<String, Vec<String>>>().unwrap();
    let directories = settings_dict.get("directories").unwrap();

    let schema = build_schema();

    let index = Index::open_or_create(MmapDirectory::open(&index_path).unwrap(), schema.clone())?;

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;

    let directories_clone = directories.clone();

    let (index_writer_tx, index_writer_rx) = unbounded();

    let watcher_index_writer = index_writer_tx.clone();

    let watcher_schema = schema.clone();

    let watcher_reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;

    let mut index_writer = index.writer(50_000_000)?;

    if !initial_processing_file.exists() {
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
                    let file_hash = if let Ok(f_h) = get_file_hash(entry_path) {
                        f_h
                    } else {
                        continue;
                    };

                    let file_to_process = FileToProcess {
                        path: entry_path.to_path_buf(),
                        hash: file_hash,
                        contents: Vec::new(),
                    };

                    // Check if this file has been processed before at a different location
                    if let Some(doc_to_update) =
                        update_existing_file(&file_to_process, &schema, &reader)
                    {
                        // If it has, add this current location to the document
                        let (_title, hash_field, _location, _body) = destructure_schema(&schema);
                        // Delete the old document
                        let hash_term =
                            Term::from_field_text(hash_field, file_hash.to_hex().as_str());
                        info!("Deleting the old document");
                        index_writer.delete_term(hash_term);
                        info!("Adding the new document");
                        index_writer.add_document(doc_to_update);
                    }
                    // We might not need to add anything if the file already exists
                    else if let Some(doc_to_add) =
                        process_file(&file_to_process, &schema, &reader)
                    {
                        index_writer.add_document(doc_to_add);
                    }
                }
            }
        }
        index_writer.commit()?;
        // After we finished doing the initial processing, add the file so that we know for next time
        fs::File::create(initial_processing_file).unwrap();
    } else {
        info!("Initial processing already done! Starting a reader");
    }

    let _watcher_thread = thread::Builder::new()
        .name("file_watcher_thread".to_string())
        .spawn(|| {
            start_watcher(
                directories_clone,
                watcher_index_writer,
                watcher_schema,
                watcher_reader,
            )
        });

    let reader_schema = schema.clone();

    let (_reader_tx, reader_rx) = query_channel;

    let _reader_thread = thread::Builder::new()
        .name("tantivy_reader".to_string())
        .spawn(move || start_reader(index, reader, reader_rx, &reader_schema, result_tx));

    for writer_action in index_writer_rx.iter() {
        match writer_action {
            WriterAction::Add(document_to_write) => {
                index_writer.add_document(document_to_write);
                // TODO: be smarter about when we commit
                index_writer.commit()?;
            }
            WriterAction::Delete(hash_term) => {
                index_writer.delete_term(hash_term);
                index_writer.commit()?;
            }
        }
    }

    Ok(())
}
