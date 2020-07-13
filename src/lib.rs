#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

pub mod contracts;
pub mod custom_tantivy;
pub mod error_adapter;
pub mod indexers;
pub mod routes;
pub mod searcher;
pub mod tantivy_process;

mod file_watcher;
