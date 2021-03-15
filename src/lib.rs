#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

pub mod config;
pub mod routes;
pub mod searcher;
pub mod tantivy_process;

pub extern crate contracts;
pub extern crate custom_tantivy;
pub extern crate indexers;

mod file_watcher;
