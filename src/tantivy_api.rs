use crate::indexers::TextIndexer;
use crate::indexers::Indexer;

use tantivy::schema::*;
use tantivy::IndexReader;
use tantivy::DocAddress;
use tantivy::query::TermQuery;
use tantivy::collector::{Count, TopDocs};

use blake2b_simd::blake2b;

use std::sync::mpsc::*;
use std::fs;
use std::path::Path;
use std::io::Read;

pub enum WriterAction {
    Add(Document),
    Delete(Term)
}

pub fn destructure_schema(schema: &Schema) -> (Field, Field, Field, Field) {
    (schema.get_field("title").unwrap(), schema.get_field("hash").unwrap(),
    schema.get_field("location").unwrap(), schema.get_field("body").unwrap())
}

// Gets the `DocAddress` of a file based on the hash
pub fn get_doc_by_hash(index_reader: &IndexReader, hash_field: Field, hash: &str) -> Option<DocAddress> {
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

// Gets the `DocAddress` of a file based on the file location
pub fn get_doc_by_location(index_reader: &IndexReader, location_field: Field, location_facet: &Facet) -> Option<DocAddress> {
    let searcher = index_reader.searcher();
    let query = TermQuery::new(
        Term::from_facet(location_field, location_facet),
            IndexRecordOption::Basic,
    );
    let (top_docs, _count) = searcher.search(&query, &(TopDocs::with_limit(1), Count)).unwrap();
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

// Find `DocAddress` by file location, delete the document and return its contents
pub fn delete_doc_by_location(index_reader: &IndexReader, index_writer: &Sender<WriterAction>, location_field: Field, location_facet: &Facet) -> Option<Document> {
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

// Find `DocAddress` by file hash, delete the document and return its contents
pub fn delete_doc_by_hash(index_reader: &IndexReader, index_writer: &Sender<WriterAction>, hash_field: Field, hash: &str) -> Option<Document> {
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

// Calculates hash of file from Path
pub fn get_file_hash(entry_path: &Path) -> blake2b_simd::Hash {
    let file_hash;
    {
        let mut file = fs::File::open(&entry_path).unwrap();
        let mut file_buffer = Vec::new();
        // TODO: Handle error
        file.read_to_end(&mut file_buffer);
        file_hash = blake2b(file_buffer.as_slice());
    }
    trace!("Hash of file is: {:?}", file_hash);
    file_hash
}

// If a document with the same file hash already exists, we can avoid processing it again
// In that case, if this file is found in a new location, add that location to the facet list
// eg: if we have 2 files A and B with the same content
// A is indexed and exists at /path/to/A
// We will see B has the same hash as A
// Instead of reprocessing B, we add /path/to/B to the list of locations
pub fn update_existing_file(entry_path: &Path, schema: &Schema, index_reader: &IndexReader, hash: &blake2b_simd::Hash) -> Option<Document> {
    let searcher = index_reader.searcher();
    let canonical_path = entry_path.canonicalize().unwrap();
    let location_facet = canonical_path.to_str().unwrap();
    let (_title, hash_field, location, _body) = destructure_schema(schema);
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

// Processes a file by running all available indexers on it
pub fn process_file(entry_path: &Path, schema: &Schema, index_reader: &IndexReader) -> Option<Document> {
    let canonical_path = entry_path.canonicalize().unwrap();
    let location_facet = canonical_path.to_str().unwrap();
    let file_hash = get_file_hash(entry_path);
    trace!("Hash of file is: {:?}", file_hash);

    // Check if the file has already been indexed
    if let Some(doc) = update_existing_file(entry_path, &schema, &index_reader, &file_hash) {
        return Some(doc)
    }

    // We're indexing this file for the first time
    if TextIndexer::supports_extension(entry_path.extension().unwrap()) {
        let (title, hash, location, body) = destructure_schema(schema);
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
    None
}

// Builds the tantivy schema
pub fn build_schema() -> Schema {
    let mut schema_builder = Schema::builder();

    schema_builder.add_text_field("title", TEXT | STORED);

    schema_builder.add_text_field("hash", STRING | STORED);

    schema_builder.add_facet_field("location");

    schema_builder.add_text_field("body", TEXT | STORED);

    schema_builder.build()
}