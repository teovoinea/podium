extern crate app_dirs;
extern crate blake2b_simd;
extern crate calamine;
extern crate config;
extern crate csv;
extern crate docx;
extern crate exif;
extern crate image;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate msoffice_pptx;
extern crate msoffice_shared;
extern crate notify;
extern crate pdf_extract;
extern crate regex;
extern crate reverse_geocoder;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate simple_logger;
extern crate tantivy;


pub mod indexers;
pub mod tantivy_process;
pub mod ui;

mod query_executor;
mod tantivy_api;
mod file_watcher;