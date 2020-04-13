use crate::indexers::*;

use crossbeam::channel::{unbounded, Receiver, Sender};
use tantivy::collector::{Count, TopDocs};
use tantivy::query::TermQuery;
use tantivy::schema::*;
use tantivy::DocAddress;
use tantivy::IndexReader;

use crate::contracts::file_to_process::FileToProcess;
use crate::error_adapter::log_and_return_error_string;
use anyhow::{Context, Result};

use blake2b_simd::blake2b;

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Actions being sent to the IndexWriter thread
pub enum WriterAction {
    Add(Document),
    Delete(Term),
}

pub fn destructure_schema(schema: &Schema) -> (Field, Field, Field, Field) {
    (
        schema.get_field("title").unwrap(),
        schema.get_field("hash").unwrap(),
        schema.get_field("location").unwrap(),
        schema.get_field("body").unwrap(),
    )
}

/// Gets the `DocAddress` of a file based on the hash
pub fn get_doc_by_hash(
    index_reader: &IndexReader,
    hash_field: Field,
    hash: &str,
) -> Option<DocAddress> {
    let searcher = index_reader.searcher();
    let query = TermQuery::new(
        Term::from_field_text(hash_field, hash),
        IndexRecordOption::Basic,
    );
    let (top_docs, count) = searcher
        .search(&query, &(TopDocs::with_limit(1), Count))
        .unwrap();
    if count == 1 {
        let (_score, address) = top_docs[0];
        Some(address)
    } else {
        if count > 1 {
            for (_score, doc_address) in top_docs {
                let retrieved_doc = searcher.doc(doc_address).unwrap();
                error!("{:?}", retrieved_doc);
            }
            panic!("More than 1 document with the same hash!!!");
        }
        None
    }
}

// /Gets the `DocAddress` of a file based on the file location
pub fn get_doc_by_location(
    index_reader: &IndexReader,
    location_field: Field,
    location_facet: &Facet,
) -> Option<DocAddress> {
    let searcher = index_reader.searcher();
    let query = TermQuery::new(
        Term::from_facet(location_field, location_facet),
        IndexRecordOption::Basic,
    );
    let (top_docs, _count) = searcher
        .search(&query, &(TopDocs::with_limit(1), Count))
        .unwrap();
    if top_docs.len() == 1 {
        let (_score, address) = top_docs[0];
        Some(address)
    } else {
        if top_docs.len() > 1 {
            for (_score, doc_address) in top_docs {
                let retrieved_doc = searcher.doc(doc_address).unwrap();
                error!("{:?}", retrieved_doc);
            }
            panic!("More than 1 document with the same location!!!");
        }
        None
    }
}

/// Find `DocAddress` by file location, delete the document and return its contents
pub fn delete_doc_by_location(
    index_reader: &IndexReader,
    index_writer: &Sender<WriterAction>,
    location_field: Field,
    location_facet: &Facet,
) -> Option<Document> {
    if let Some(old_address) = get_doc_by_location(index_reader, location_field, location_facet) {
        let searcher = index_reader.searcher();
        let location_term = Term::from_facet(location_field, location_facet);
        let old_document = Some(searcher.doc(old_address).unwrap());
        index_writer
            .send(WriterAction::Delete(location_term))
            .unwrap();
        info!("Deleting document by location: {:?}", old_document);
        old_document
    } else {
        None
    }
}

/// Find `DocAddress` by file hash, delete the document and return its contents
pub fn delete_doc_by_hash(
    index_reader: &IndexReader,
    index_writer: &Sender<WriterAction>,
    hash_field: Field,
    hash: &str,
) -> Option<Document> {
    if let Some(old_address) = get_doc_by_hash(index_reader, hash_field, hash) {
        let searcher = index_reader.searcher();
        let location_term = Term::from_field_text(hash_field, hash);
        let old_document = Some(searcher.doc(old_address).unwrap());
        index_writer
            .send(WriterAction::Delete(location_term))
            .unwrap();
        info!("Deleting document by hash: {:?}", old_document);
        old_document
    } else {
        None
    }
}

/// Calculates hash of file from Path
/// Can fail if it can't read all the bytes from the file
pub fn get_file_hash(entry_path: &Path) -> Result<blake2b_simd::Hash> {
    let file_hash;
    let mut file = fs::File::open(&entry_path).unwrap();
    let mut file_buffer = Vec::new();
    file.read_to_end(&mut file_buffer).with_context(|| {
        log_and_return_error_string(format!(
            "tantivy_api: Failed to read buffer to end for file at path: {:?}",
            entry_path
        ))
    })?;
    file_hash = blake2b(file_buffer.as_slice());
    info!("Hash of file is: {:?}", file_hash);
    Ok(file_hash)
}

/// If a document with the same file hash already exists, we can avoid processing it again
/// In that case, if this file is found in a new location, add that location to the facet list
/// eg: if we have 2 files A and B with the same content
/// A is indexed and exists at /path/to/A
/// We will see B has the same hash as A
/// Instead of reprocessing B, we add /path/to/B to the list of locations
pub fn update_existing_file(
    file_to_process: &FileToProcess,
    schema: &Schema,
    index_reader: &IndexReader,
) -> Option<Document> {
    let entry_path = &file_to_process.path;
    let hash = file_to_process.hash;
    let searcher = index_reader.searcher();
    let location_facet = &entry_path.to_facet_value();
    let (_title, hash_field, location, _body) = destructure_schema(schema);
    if let Some(doc_address) = get_doc_by_hash(index_reader, hash_field, hash.to_hex().as_str()) {
        info!("We've seen this file before! {:?}", location_facet);
        let mut retrieved_doc = searcher.doc(doc_address).unwrap();
        info!(
            "Is this current file's location already in the document? {:?}",
            !retrieved_doc
                .get_all(location)
                .contains(&&Value::from(Facet::from_text(location_facet)))
        );
        if !retrieved_doc
            .get_all(location)
            .contains(&&Value::from(Facet::from_text(location_facet)))
        {
            // If this location of the file isn't already stored in the document, add it
            retrieved_doc.add_facet(location, location_facet);
            info!(
                "The new document with the added location is: {:?}",
                retrieved_doc
            );
            return Some(retrieved_doc);
        }
        // Otherwise, we can ignore
        return None;
    }
    None
}

/// Processes a file by running all available indexers on it
/// Updates an existing entry if necessary
pub fn process_file(
    file_to_process: &FileToProcess,
    schema: &Schema,
    index_reader: &IndexReader,
) -> Option<Document> {
    let entry_path = &file_to_process.path;
    let file_hash = file_to_process.hash;
    if entry_path.extension() == None {
        info!("Skipping, no file extension: {:?}", entry_path);
        return None;
    }

    let location_facet = &entry_path.to_facet_value();

    info!("Processing: {:?}", entry_path);
    info!("Hash of file is: {:?}", file_hash);

    // Check if the file has already been indexed
    if let Some(doc) = update_existing_file(file_to_process, &schema, &index_reader) {
        return Some(doc);
    }

    let analyzer = Analyzer::default();

    // We're indexing the file for the first time
    let results = analyzer.analyze(entry_path.extension().unwrap(), file_to_process);
    if !results.is_empty() {
        info!("This is a new file, we need to process it");
        let title = &results[0].name;
        let body = results.iter().fold(String::new(), |mut acc, x| {
            acc.push_str(&x.body);
            acc.push_str(" ");
            acc
        });
        info!("{:?} {:?}", title, body);

        let (title_field, hash_field, location_field, body_field) = destructure_schema(schema);
        let mut new_doc = Document::default();

        new_doc.add_text(title_field, &title);
        new_doc.add_facet(location_field, location_facet);
        new_doc.add_text(hash_field, file_hash.to_hex().as_str());
        new_doc.add_text(body_field, &body);
        return Some(new_doc);
    } else {
        info!("Couldn't find any results for file at: {:?}", entry_path);
    }

    None
}

/// Builds the tantivy schema
pub fn build_schema() -> Schema {
    let mut schema_builder = Schema::builder();

    schema_builder.add_text_field("title", TEXT | STORED);

    schema_builder.add_text_field("hash", STRING | STORED);

    schema_builder.add_facet_field("location");

    schema_builder.add_text_field("body", TEXT | STORED);

    schema_builder.build()
}

/// Converts to/from Facet/PathBuf
pub trait TantivyConvert {
    fn to_facet_value(&self) -> String;
    fn from_facet_value(facet_val: &Facet) -> PathBuf;
}

impl TantivyConvert for Path {
    #[cfg(target_os = "windows")]
    fn to_facet_value(&self) -> String {
        self.canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
            .replace("\\", "/")
    }

    #[cfg(not(target_os = "windows"))]
    fn to_facet_value(&self) -> String {
        String::from(self.canonicalize().unwrap().to_str().unwrap())
    }

    #[cfg(target_os = "windows")]
    fn from_facet_value(facet_val: &Facet) -> PathBuf {
        Path::new(
            &facet_val
                .encoded_str()
                .replace(char::from(0), "/")
                .replacen("/?/", "", 1),
        )
        .to_path_buf()
    }

    #[cfg(not(target_os = "windows"))]
    fn from_facet_value(facet_val: &Facet) -> PathBuf {
        let mut location = String::from("/");
        location.push_str(&facet_val.encoded_str().replace(char::from(0), "/"));
        Path::new(&location).to_path_buf()
    }
}

mod test {
    #[test]
    fn test_path_facet_conversion() {
        use super::*;
        use std::env;
        use std::fs::File;

        let mut current_dir = env::current_dir().unwrap();
        current_dir.push("Cargo.toml");
        println!("{:?}", current_dir);

        let current_dir_facet_string = current_dir.to_facet_value();
        println!("{:?}", current_dir_facet_string);

        let facet = Facet::from_text(&current_dir_facet_string);
        println!("{:?}", facet);

        let dir_from_facet = Path::from_facet_value(&facet);
        println!("{:?}", dir_from_facet);

        File::open(dir_from_facet).unwrap();
    }
}
