#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

pub mod contracts;
pub mod error_adapter;
pub mod indexers;
pub mod query_executor;
pub mod routes;
pub mod tantivy_process;

mod file_watcher;

mod tantivy_api;
