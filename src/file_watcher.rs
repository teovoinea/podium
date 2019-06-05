use std::time::Duration;
use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent};
use tantivy::schema::*;
use crate::tantivy_api::*;
use std::sync::mpsc::*;
use tantivy::IndexReader;
use tantivy::schema::Value;
use std::sync::mpsc::channel;
use walkdir::{DirEntry};

pub fn start_watcher(directories: Vec<String>, index_writer: Sender<WriterAction>, schema: Schema, index_reader: IndexReader) {
    info!("Starting file watcher thread on: {:?}", directories);
    let (watcher_tx, watcher_rx) = channel();
    let mut watcher = watcher(watcher_tx, Duration::from_secs(10)).unwrap();

    // Start watching all directories in the config file
    for directory in directories {
        watcher.watch(directory, RecursiveMode::Recursive).unwrap();
    }

    loop {
        match watcher_rx.recv() {
            Ok(event) => {
                info!("Received watcher event: {:?}", event);
                match event {
                    DebouncedEvent::Create(path_buf) => {
                        if path_buf.is_dir() {
                            // Traverse through all the files in the directory 
                        }
                        else {
                            let file_hash = get_file_hash(path_buf.as_path());
                            // Check if this file has been processed before at a different location
                            if let Some(doc_to_update) = update_existing_file(path_buf.as_path(), &schema, &index_reader, &file_hash) {
                                // If it has, add this current location to the document
                                // let location_facet = Facet::from_text(path_buf.as_path().to_str().unwrap());
                                let (_title, hash_field, _location, _body) = destructure_schema(&schema);
                                // Delete the old document
                                info!("Deleting the old document");
                                delete_doc_by_hash(&index_reader, &index_writer, hash_field, file_hash.to_hex().as_str());
                                info!("Adding the new document");
                                index_writer.send(
                                    WriterAction::Add(doc_to_update)
                                ).unwrap();
                            }
                            // We might not need to add anything if the file already exists
                            else if let Some(doc_to_add) = process_file(path_buf.as_path(), &schema, &index_reader) {
                                index_writer.send(
                                    WriterAction::Add(doc_to_add)
                                ).unwrap();
                            }
                        }
                    },
                    DebouncedEvent::Write(path_buf) => {
                        // Remove the old document, reprocess and add the new content
                        if path_buf.is_dir() {
                            // Traverse through all the files in the directory 
                        }
                        else {
                            // Remove the old document
                            let location_facet = Facet::from_text(path_buf.as_path().canonicalize().unwrap().to_str().unwrap());
                            let (title, hash_field, location, body) = destructure_schema(&schema);
                            if let Some(old_doc) = delete_doc(&index_reader, &index_writer, location, &location_facet) {
                                info!("Deleted old document succesfully");
                                let mut locations = old_doc.get_all(location);
                                info!("Current locations for the doc are: {:?}", locations);
                                if locations.len() > 1 {
                                    info!("Removing old document but there are multiple locations");
                                    info!("Removing {0:?} from {1:?}", path_buf, locations);
                                    // there were multiple copies of this file elsewhere
                                    // only remove this location, keep the rest
                                    let mut old_location_index = None;
                                    for (index, &location_value) in locations.iter().enumerate() {
                                        match location_value {
                                            Value::Facet(location_value_facet) => {
                                                info!("Checking if {0:?} is equal to {1:?}", location_value_facet, &location_facet);
                                                if location_value_facet == &location_facet {
                                                    old_location_index = Some(index)
                                                }
                                            }
                                            _ => { /* ignore */ }
                                        }
                                    }
                                    info!("Index to remove: {:?}", old_location_index);
                                    match old_location_index {
                                        Some(index) => { locations.remove(index); },
                                        None => { panic!("Tried to remove location {0:?} from document {1:?} but the location was not found", path_buf, old_doc); }
                                    }
                                    
                                    let mut new_doc = Document::default();
                                    info!("Setting title for new doc");

                                    for title_value in old_doc.get_all(title) {
                                        new_doc.add_text(title, title_value.text().unwrap());
                                    }

                                    info!("Setting locations for new doc");
                                    for location_value in locations {
                                        new_doc.add(FieldValue::new(location, location_value.clone()));
                                    }

                                    info!("Setting hash for new doc");

                                    // There should only be 1 hash value
                                    new_doc.add_text(hash_field, old_doc.get_first(hash_field).unwrap().text().unwrap());

                                    info!("Setting body for new doc");
                                    for body_value in old_doc.get_all(body) {
                                        new_doc.add_text(body, body_value.text().unwrap());
                                    }

                                    info!("The new doc after modifications {:?}", new_doc);
                                    index_writer.send(
                                        WriterAction::Add(new_doc)
                                    ).unwrap();
                                }
                            }

                            let file_hash = get_file_hash(path_buf.as_path());
                            // Check if this file has been processed before at a different location
                            if let Some(doc_to_update) = update_existing_file(path_buf.as_path(), &schema, &index_reader, &file_hash) {
                                // If it has, add this current location to the document
                                // let location_facet = Facet::from_text(path_buf.as_path().to_str().unwrap());
                                let (_title, hash_field, _location, _body) = destructure_schema(&schema);
                                // Delete the old document
                                info!("Deleting the old document");
                                delete_doc_by_hash(&index_reader, &index_writer, hash_field, file_hash.to_hex().as_str());
                                info!("Adding the new document: {:?}", doc_to_update);
                                index_writer.send(
                                    WriterAction::Add(doc_to_update)
                                ).unwrap();
                            }
                            // We might not need to add anything if the file already exists
                            else if let Some(doc_to_add) = process_file(path_buf.as_path(), &schema, &index_reader) {
                                index_writer.send(
                                    WriterAction::Add(doc_to_add)
                                ).unwrap();
                            }
                        }
                    },
                    DebouncedEvent::NoticeRemove(path_buf) => {
                        if path_buf.is_dir() {
                            // Traverse through all the files in the directory 
                        }
                        else {
                            // Remove the old document
                            let location_facet = Facet::from_text(path_buf.as_path().to_str().unwrap());
                            let (_title, _hash_field, location, _body) = destructure_schema(&schema);
                            delete_doc(&index_reader, &index_writer, location, &location_facet);
                        }
                    },
                    DebouncedEvent::Rename(src_path_buf, dst_path_buf) => {
                        // Figure out if you can just update the facet without reprocessing the whole document?
                    },
                    _ => {
                        // Ignore the rest for now? Not sure...
                    }
                }
            },
            Err(e) => error!("watch error: {:?}", e),
        }
    }
}

pub fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with("."))
         .unwrap_or(false)
}