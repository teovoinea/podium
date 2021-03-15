use common::tantivy::collector::TopDocs;
use common::tantivy::query::QueryParser;
use common::tantivy::schema::*;
use common::tantivy::Index;
use common::tantivy::IndexReader;
use common::tracing::info;
use serde::{Deserialize, Serialize};

use crate::custom_tantivy::{path_facet_convert::TantivyConvert, utils::destructure_schema};

use std::path::*;

pub type QueryResponse = Vec<Response>;

/// Each tantivy document is stored in this format to be communicated to the ui
#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    /// File title
    pub title: String,
    /// Where the file can be found
    pub location: Vec<PathBuf>,
    /// The content that was indexed from the file
    pub body: String,
}
pub struct Searcher {
    index: Index,
    index_reader: IndexReader,
    schema: Schema,
}

impl Searcher {
    pub fn new(index: Index, index_reader: IndexReader, schema: Schema) -> Self {
        Searcher {
            index,
            index_reader,
            schema,
        }
    }

    pub fn search(&self, query_string: String) -> QueryResponse {
        let searcher = self.index_reader.searcher();

        let (title, _, location, body) = destructure_schema(&self.schema);

        let query_parser = QueryParser::for_index(&self.index, vec![title, body]);
        info!("Searching for a file with {:?}...", query_string);
        let query = query_parser.parse_query(&query_string).unwrap();
        info!("Parsed query");

        let top_docs = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();
        info!("Executed search");

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

        result
    }
}
