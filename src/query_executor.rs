use crate::tantivy_api::*;

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::Index;
use tantivy::IndexReader;

use std::path::*;
use std::sync::mpsc::*;

pub type QueryResponse = Vec<Response>;

#[derive(Debug)]
pub struct Response {
    pub title: String,
    pub location: Vec<PathBuf>,
    pub body: String,
}

// Starts the query executor thread
// It receives queries as strings and prints them out to console
pub fn start_reader(
    index: Index,
    reader: IndexReader,
    queries: Receiver<String>,
    schema: &Schema,
    results: Sender<QueryResponse>,
) {
    info!("Starting query executor thread");
    for query_string in queries.iter() {
        // Searchers are cheap and should be regenerated for each query
        let searcher = reader.searcher();

        let (title, _, location, body) = destructure_schema(schema);

        let query_parser = QueryParser::for_index(&index, vec![title, body, location]);
        info!("Searching for a file with {:?}...", query_string);
        let query = query_parser.parse_query(&query_string).unwrap();

        let top_docs = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();

        let result = top_docs
            .into_iter()
            .map(|(_score, doc_address)| searcher.doc(doc_address).unwrap())
            .map(|retrieved_doc| {
                let title = retrieved_doc
                    .get_all(title)
                    .iter()
                    .map(|val| val.text())
                    .fold(String::new(), |mut acc, x| {
                        acc.push_str(x.unwrap());
                        acc.push_str(" ");
                        acc
                    });
                let location = retrieved_doc
                    .get_all(location)
                    .iter()
                    .filter_map(|val| match &val {
                        Value::Facet(loc_str) => Some(Path::from_facet_value(loc_str)),
                        _ => None,
                    })
                    .collect();
                let body = retrieved_doc
                    .get_all(body)
                    .iter()
                    .map(|val| val.text())
                    .fold(String::new(), |mut acc, x| {
                        acc.push_str(x.unwrap());
                        acc.push_str(" ");
                        acc
                    });
                Response {
                    title,
                    location,
                    body,
                }
            })
            .collect();

        if results.send(result).is_err() {
            error!("Failed to send search results to UI");
        }
    }
}
