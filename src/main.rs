extern crate app_dirs;
extern crate blake2b_simd;
extern crate config;
#[macro_use] extern crate log;
extern crate notify;
extern crate notify_rust;
extern crate simple_logger;
extern crate sysbar;
#[macro_use] extern crate tantivy;

use blake2b_simd::blake2b;
use sysbar::Sysbar;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::ReloadPolicy;
use tantivy::directory::*;
use tantivy::IndexReader;
use tantivy::IndexWriter;
use tantivy::{Index, Result, Term};
use tantivy::collector::{Count, TopDocs};
use tantivy::DocAddress;
use tantivy::query::TermQuery;
use tantivy::schema::Value;
use app_dirs::*;
use walkdir::{DirEntry, WalkDir};
use config::*;
use notify_rust::Notification;
use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent};

use std::sync::mpsc::channel;
use std::time::Duration;
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use std::thread;
use std::thread::*;
use std::io;
use std::io::prelude::*;
use std::sync::mpsc::*;

mod indexers;
use indexers::TextIndexer;
use indexers::Indexer;
use indexers::DocumentSchema;

const APP_INFO: AppInfo = AppInfo{name: "Podium", author: "Teodor Voinea"};


fn main() {
    simple_logger::init().unwrap();
    let (query_tx, query_rx) = channel();
    let tantivy_query_tx = query_tx.clone();
    let tantivy_thread = thread::Builder::new().name("tantivy".to_string()).spawn(move || {
        start_tantivy((tantivy_query_tx, query_rx))
    });

    let mut bar = sysbar::Sysbar::new("P");
    bar.add_item(
        "Search",
        Box::new(move || {
            println!("Searching!");
            query_tx.send("digimon".to_string()).unwrap();
        }),
    );

    bar.add_quit_item("Quit");

    bar.display();

    trace!("Taskbar has quit, cleaning up remaining threads...");

    tantivy_thread.unwrap().join();
}


fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with("."))
         .unwrap_or(false)
}

fn build_schema() -> Schema {
    let mut schema_builder = Schema::builder();

    schema_builder.add_text_field("title", TEXT | STORED);

    schema_builder.add_text_field("hash", STRING | STORED);

    schema_builder.add_facet_field("location");

    schema_builder.add_text_field("body", TEXT | STORED);

    schema_builder.build()
}

enum WriterAction {
    Add(Document),
    Delete(Term)
}

fn start_watcher(directories: Vec<String>, index_writer: Sender<WriterAction>, schema: Schema, index_reader: IndexReader) {
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
                                let location_facet = Facet::from_text(path_buf.as_path().to_str().unwrap());
                                let (_title, hash_field, location, _body) = destructure_schema(&schema);
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
                                let location_facet = Facet::from_text(path_buf.as_path().to_str().unwrap());
                                let (_title, hash_field, location, _body) = destructure_schema(&schema);
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

fn get_doc_by_hash(index_reader: &IndexReader, hash_field: Field, hash: &str) -> Option<DocAddress> {
    let searcher = index_reader.searcher();
    let query = TermQuery::new(
            Term::from_field_text(hash_field, hash),
            IndexRecordOption::Basic,
    );
    let (top_docs, count) = searcher.search(&query, &(TopDocs::with_limit(1), Count)).unwrap();
    if count == 1 {
        let (_score, address) = top_docs[0];
        Some(address)
    }
    else {
        if count > 1 {
            for (_score, doc_address) in top_docs {
                let retrieved_doc = searcher.doc(doc_address).unwrap();
                info!("{:?}", retrieved_doc);
            }
            panic!("More than 1 document with the same hash!!!");
        }
        None
    }
}

fn get_doc_by_location(index_reader: &IndexReader, location_field: Field, location_facet: &Facet) -> Option<DocAddress> {
    let searcher = index_reader.searcher();
    let query = TermQuery::new(
        Term::from_facet(location_field, location_facet),
            IndexRecordOption::Basic,
    );
    let (top_docs, count) = searcher.search(&query, &(TopDocs::with_limit(1), Count)).unwrap();
    if top_docs.len() == 1 {
        let (_score, address) = top_docs[0];
        Some(address)
    }
    else {
        if top_docs.len() > 1 {
            for (_score, doc_address) in top_docs {
                let retrieved_doc = searcher.doc(doc_address).unwrap();
                info!("{:?}", retrieved_doc);
            }
            panic!("More than 1 document with the same location!!!");
        }
        None
    }
}

fn delete_doc(index_reader: &IndexReader, index_writer: &Sender<WriterAction>, location_field: Field, location_facet: &Facet) -> Option<Document> {
    if let Some(old_address) = get_doc_by_location(index_reader, location_field, location_facet) {
        let searcher = index_reader.searcher();
        let location_term = Term::from_facet(location_field, location_facet);
        let old_document = Some(searcher.doc(old_address).unwrap());
        index_writer.send(WriterAction::Delete(location_term)).unwrap();
        info!("Deleting document by location: {:?}", old_document);
        old_document
    }
    else {
        None
    }
}

fn delete_doc_by_hash(index_reader: &IndexReader, index_writer: &Sender<WriterAction>, hash_field: Field, hash: &str) -> Option<Document> {
    if let Some(old_address) = get_doc_by_hash(index_reader, hash_field, hash) {
        let searcher = index_reader.searcher();
        let location_term = Term::from_field_text(hash_field, hash);
        let old_document = Some(searcher.doc(old_address).unwrap());
        index_writer.send(WriterAction::Delete(location_term)).unwrap();
        info!("Deleting document by hash: {:?}", old_document);
        old_document
    }
    else {
        None
    }
}

fn get_file_hash(entry_path: &Path) -> blake2b_simd::Hash {
    let file_hash;
    {
        let mut file = fs::File::open(&entry_path).unwrap();
        let mut file_buffer = Vec::new();
        file.read_to_end(&mut file_buffer);
        file_hash = blake2b(file_buffer.as_slice());
    }
    trace!("Hash of file is: {:?}", file_hash);
    file_hash
}

fn update_existing_file(entry_path: &Path, schema: &Schema, index_reader: &IndexReader, hash: &blake2b_simd::Hash) -> Option<Document> {
    let searcher = index_reader.searcher();
    let canonical_path = entry_path.canonicalize().unwrap();
    let location_facet = canonical_path.to_str().unwrap();
    let (title, hash_field, location, body) = destructure_schema(schema);
    if let Some(doc_address) = get_doc_by_hash(index_reader, hash_field, hash.to_hex().as_str()) {
        info!("We've seen this file before! {:?}", canonical_path);
        let mut retrieved_doc = searcher.doc(doc_address).unwrap();
        info!("Is this current file's location already in the document? {:?}", !retrieved_doc.get_all(location).contains(&&Value::from(Facet::from_text(location_facet))));
        if !retrieved_doc.get_all(location).contains(&&Value::from(Facet::from_text(location_facet))) {
            // If this location of the file isn't already stored in the document, add it
            retrieved_doc.add_facet(location, location_facet);
            info!("The new document with the added location is: {:?}", retrieved_doc);
            return Some(retrieved_doc)
        }
        // Otherwise, we can ignore
        return None
    }
    None
}

fn process_file(entry_path: &Path, schema: &Schema, index_reader: &IndexReader) -> Option<Document> {
    let searcher = index_reader.searcher();
    let canonical_path = entry_path.canonicalize().unwrap();
    let location_facet = canonical_path.to_str().unwrap();
    if TextIndexer::supports_extension(entry_path.extension().unwrap()) {
        let file_hash = get_file_hash(entry_path);
        trace!("Hash of file is: {:?}", file_hash);

        let (title, hash, location, body) = destructure_schema(schema);

        // Check if the file has already been indexed

        if let Some(doc_address) = get_doc_by_hash(index_reader, hash, file_hash.to_hex().as_str()) {
            info!("We've seen this file before! {:?}", canonical_path);
            let mut retrieved_doc = searcher.doc(doc_address).unwrap();
            info!("Is this current file's location already in the document? {:?}", !retrieved_doc.get_all(location).contains(&&Value::from(Facet::from_text(location_facet))));
            if !retrieved_doc.get_all(location).contains(&&Value::from(Facet::from_text(location_facet))) {
                // If this location of the file isn't already stored in the document, add it
                retrieved_doc.add_facet(location, location_facet);
                info!("The new document with the added location is: {:?}", retrieved_doc);
                return Some(retrieved_doc)
            }
            // Otherwise, we can ignore
            return None
        }
        else {
            info!("This is a new file, we need to process it");
            let mut new_doc = Document::default();
            let indexed_content = TextIndexer::index_file(entry_path);
            trace!("{:?}", indexed_content);
            
            new_doc.add_text(title, &indexed_content.name);
            new_doc.add_facet(location, location_facet);
            new_doc.add_text(hash, file_hash.to_hex().as_str());
            new_doc.add_text(body, &indexed_content.body);
            return Some(new_doc)
        }
    }
    None
}

fn destructure_schema(schema: &Schema) -> (Field, Field, Field, Field) {
    (schema.get_field("title").unwrap(), schema.get_field("hash").unwrap(),
    schema.get_field("location").unwrap(), schema.get_field("body").unwrap())
}

fn start_tantivy(query_channel: (Sender<String>, Receiver<String>)) -> tantivy::Result<()> {
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

        Notification::new().summary("Welcome!")
                        .body("Since this is your first time running podium, it will take a few minutes to index your files.")
                        .show()
                        .unwrap();

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

    let (index_writer_tx, index_writer_rx) = channel();

    let watcher_index_writer = index_writer_tx.clone();
    
    let watcher_schema = schema.clone();

    let watcher_reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;

    let watcher_thread = thread::Builder::new()
                            .name("file_watcher_thread".to_string())
                            .spawn(|| start_watcher(directories_clone, watcher_index_writer, watcher_schema, watcher_reader));

    let mut index_writer = index.writer(50_000_000)?;

    let title = schema.get_field("title").unwrap();
    let hash = schema.get_field("hash").unwrap();
    let location = schema.get_field("location").unwrap();
    let body = schema.get_field("body").unwrap();

    if !initial_processing_file.exists() {
        info!("Initial processing was not previously done, doing now");
        for directory in directories {
            let walker = WalkDir::new(directory).into_iter();
            for entry in walker.filter_entry(|e| !is_hidden(e)) {
                let entry = entry.unwrap();
                if !entry.file_type().is_dir() {
                    let entry_path = entry.path();
                    let file_hash = get_file_hash(entry_path);
                    // Check if this file has been processed before at a different location
                    if let Some(doc_to_update) = update_existing_file(entry_path, &schema, &reader, &file_hash) {
                        // If it has, add this current location to the document
                        let (_title, hash_field, _location, _body) = destructure_schema(&schema);
                        // Delete the old document
                        let hash_term = Term::from_field_text(hash_field, file_hash.to_hex().as_str());
                        info!("Deleting the old document");
                        index_writer.delete_term(hash_term);
                        info!("Adding the new document");
                        index_writer.add_document(doc_to_update);
                    }
                    // We might not need to add anything if the file already exists
                    else if let Some(doc_to_add) = process_file(entry_path, &schema, &reader) {
                        index_writer.add_document(doc_to_add);
                    }
                }
            }
        }
        index_writer.commit()?;
        // After we finished doing the initial processing, add the file so that we know for next time
        fs::File::create(initial_processing_file).unwrap();
    }
    else {
        println!("Initial processing already done! Starting a reader");
    }    

    let reader_schema = schema.clone();

    let (reader_tx, reader_rx) = query_channel;

    let reader_thread = thread::Builder::new()
                            .name("tantivy_reader".to_string())
                            .spawn(move || start_reader(index, reader, reader_rx, &reader_schema));

    for writer_action in index_writer_rx.iter() {
        match writer_action {
            WriterAction::Add(document_to_write) => {
                index_writer.add_document(document_to_write);
                // TODO: be smarter about when we commit
                index_writer.commit()?;
            },
            WriterAction::Delete(hash_term) => {
                index_writer.delete_term(hash_term);
                index_writer.commit()?;
            }
        } 
    }

    Ok(())
}

fn start_reader(index: Index, reader: IndexReader, queries: Receiver<String>, schema: &Schema) {
    info!("Starting query executor thread");
    for query_string in queries.iter() {
        // Searchers are cheap and should be regenerated for each query
        let searcher = reader.searcher();

        let (title, _, location, body) = destructure_schema(schema);

        let query_parser = QueryParser::for_index(&index, vec![title, body, location]);
        info!("Searching for a file with {:?}...", query_string);
        let query = query_parser.parse_query(&query_string).unwrap();

        let top_docs = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();

        for (_score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address).unwrap();
            info!("{}", schema.to_json(&retrieved_doc));
        }
    }
}