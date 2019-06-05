use crate::tantivy_api::*;

use tantivy::query::QueryParser;
use tantivy::Index;
use tantivy::IndexReader;
use tantivy::collector::TopDocs;
use tantivy::schema::*;

use std::sync::mpsc::*;


pub fn start_reader(index: Index, reader: IndexReader, queries: Receiver<String>, schema: &Schema) {
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