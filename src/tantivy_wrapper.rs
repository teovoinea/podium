use tantivy::collector::{Count, TopDocs};
use tantivy::query::TermQuery;
use tantivy::schema::*;
use tantivy::DocAddress;
use tantivy::{IndexReader, IndexWriter};

use crate::contracts::file_to_process::FileToProcess;
use crate::indexers::*;

use async_trait::async_trait;
use blake2b_simd::blake2b;

use std::path::{Path, PathBuf};

use crate::path_facet_convert::*;

pub struct TantivyWrapper {
    pub index_reader: IndexReader,
    pub index_writer: IndexWriter,
    pub schema: Schema,
}

impl TantivyWrapper {
    pub fn new(index_reader: IndexReader, index_writer: IndexWriter, schema: Schema) -> Self {
        TantivyWrapper {
            index_reader,
            index_writer,
            schema,
        }
    }

    pub fn update_doc_by_hash(
        &self,
        entry_path: &Path,
        hash: &blake2b_simd::Hash,
    ) -> Option<Document> {
        let searcher = self.index_reader.searcher();
        let location_facet = &entry_path.to_facet_value();
        let (_title, hash_field, location, _body) = destructure_schema(&self.schema);
        if let Some(doc_address) = self.get_doc_by_hash(hash_field, hash.to_hex().as_str()) {
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

    fn delete_doc_by_location(
        &self,
        location_field: Field,
        location_facet: &Facet,
    ) -> Option<Document> {
        if let Some(old_address) = self.get_doc_by_location(location_field, location_facet) {
            let searcher = self.index_reader.searcher();
            let location_term = Term::from_facet(location_field, location_facet);
            let old_document = Some(searcher.doc(old_address).unwrap());
            self.index_writer.delete_term(location_term);
            info!("Deleting document by location: {:?}", old_document);
            old_document
        } else {
            None
        }
    }

    pub fn delete_doc_by_hash(
        mut self,
        hash_field: Field,
        hash: &str,
    ) -> tantivy::Result<Option<Document>> {
        if let Some(old_address) = self.get_doc_by_hash(hash_field, hash) {
            let searcher = self.index_reader.searcher();
            let hash_term = Term::from_field_text(hash_field, hash);
            let old_document = Some(searcher.doc(old_address).unwrap());
            self.index_writer.delete_term(hash_term);
            self.index_writer.commit()?;
            info!("Deleting document by hash: {:?}", old_document);
            Ok(old_document)
        } else {
            Ok(None)
        }
    }

    pub fn get_doc_by_hash(&self, hash_field: Field, hash: &str) -> Option<DocAddress> {
        let searcher = self.index_reader.searcher();
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

    pub fn get_doc_by_location(
        &self,
        location_field: Field,
        location_facet: &Facet,
    ) -> Option<DocAddress> {
        let searcher = self.index_reader.searcher();
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

    /// Removes this path from its associated document
    /// If this path is the last remaining path associated to this document, will dete the document
    pub fn remove(&self, path_buf: &PathBuf) {
        // Remove the old document
        let location_facet = Facet::from_text(path_buf.as_path().to_str().unwrap());
        let (_title, _hash_field, location, _body) = destructure_schema(&self.schema);
        if let Some(old_doc) = self.delete_doc_by_location(location, &location_facet) {
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
                    if let Value::Facet(location_value_facet) = location_value {
                        info!(
                            "Checking if {0:?} is equal to {1:?}",
                            location_value_facet, &location_facet
                        );
                        if location_value_facet == &location_facet {
                            old_location_index = Some(index)
                        }
                    }
                }
                info!("Index to remove: {:?}", old_location_index);
                match old_location_index {
                    Some(index) => {
                        locations.remove(index);
                    }
                    None => {
                        panic!("Tried to remove location {0:?} from document {1:?} but the location was not found", path_buf, old_doc);
                    }
                }

                let mut new_doc = Document::default();
                new_doc_for_update(&mut new_doc, &old_doc, locations, &self.schema);

                info!("The new doc after modifications {:?}", new_doc);
                self.index_writer.add_document(new_doc.clone());
            }
        }
    }
}

#[async_trait]
pub trait FileProcessor {
    async fn process_file(&self, file_to_process: FileToProcess) -> Option<Document>;
}

#[async_trait]
impl FileProcessor for TantivyWrapper {
    async fn process_file(&self, file_to_process: FileToProcess) -> Option<Document> {
        let entry_path = file_to_process.path.clone();
        let path = entry_path.as_path();
        let file_hash = file_to_process.hash;
        if entry_path.extension() == None {
            info!("Skipping, no file extension: {:?}", entry_path);
            return None;
        }

        let location_facet = &entry_path.to_facet_value();

        info!("Processing: {:?}", entry_path);
        info!("Hash of file is: {:?}", file_hash);

        // Check if the file has already been indexed
        if let Some(doc) = self.update_doc_by_hash(path, &file_to_process.hash) {
            // TODO: Since this file has been seen before, we should simply add the location of this current field to process to the document
            return Some(doc);
        }

        // We're indexing the file for the first time
        let results = analyze(
            entry_path.extension().unwrap().to_os_string(),
            file_to_process,
        )
        .await;
        if !results.is_empty() {
            info!("This is a new file, we need to process it");
            let title = &results[0].name;
            let body = results.iter().fold(String::new(), |mut acc, x| {
                acc.push_str(&x.body);
                acc.push_str(" ");
                acc
            });
            info!("{:?} {:?}", title, body);

            let (title_field, hash_field, location_field, body_field) =
                destructure_schema(&self.schema);
            let mut new_doc = Document::default();

            new_doc.add_text(title_field, &title);
            new_doc.add_facet(location_field, location_facet);
            new_doc.add_text(hash_field, file_hash.to_hex().as_str());
            new_doc.add_text(body_field, &body);
            self.index_writer.add_document(new_doc.clone());
            // self.index_writer.commit().unwrap();
            return Some(new_doc);
        } else {
            info!("Couldn't find any results for file at: {:?}", entry_path);
        }

        None
    }
}

pub fn destructure_schema(schema: &Schema) -> (Field, Field, Field, Field) {
    (
        schema.get_field("title").unwrap(),
        schema.get_field("hash").unwrap(),
        schema.get_field("location").unwrap(),
        schema.get_field("body").unwrap(),
    )
}

/// Takes a default new doc, adds the values from old doc, but uses a different set of locations
/// Used when removing 1 location from a list of locations
fn new_doc_for_update(
    new_doc: &mut Document,
    old_doc: &Document,
    locations: Vec<&Value>,
    schema: &Schema,
) {
    let (title, hash_field, location, body) = destructure_schema(&schema);

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
    new_doc.add_text(
        hash_field,
        old_doc.get_first(hash_field).unwrap().text().unwrap(),
    );

    info!("Setting body for new doc");
    for body_value in old_doc.get_all(body) {
        new_doc.add_text(body, body_value.text().unwrap());
    }
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

pub fn calculate_hash(input: &[u8]) -> blake2b_simd::Hash {
    let file_hash = blake2b(input);
    info!("Hash of file is: {:?}", file_hash);
    file_hash
}
