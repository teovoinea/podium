extern crate app_dirs;
extern crate blake2b_simd;
extern crate config;
#[macro_use] extern crate log;
extern crate notify;
extern crate notify_rust;
extern crate simple_logger;
extern crate sysbar;
#[macro_use] extern crate tantivy;

use blake2b_simd::blake2b;
use sysbar::Sysbar;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::Index;
use tantivy::ReloadPolicy;
use tantivy::directory::*;
use app_dirs::*;
use walkdir::{DirEntry, WalkDir};
use config::*;
use notify_rust::Notification;

use std::collections::HashMap;
use std::path::Path;
use std::fs;
use std::thread;
use std::thread::*;
use std::io;
use std::io::prelude::*;

mod indexers;
use indexers::TextIndexer;
use indexers::Indexer;
use indexers::DocumentSchema;

const APP_INFO: AppInfo = AppInfo{name: "Podium", author: "Teodor Voinea"};


fn main() {
    simple_logger::init().unwrap();
    let tantivy_thread = thread::Builder::new().name("tantivy".to_string()).spawn(move || {
        start_tantivy()
    });

    let mut bar = sysbar::Sysbar::new("P");
    bar.add_item(
        "Say 'bar'",
        Box::new(move || {
            println!("bar");
        }),
    );

    bar.add_quit_item("Quit");

    bar.display();

    trace!("Taskbar has quit, cleaning up remaining threads...");

    tantivy_thread.unwrap().join();
}


fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with("."))
         .unwrap_or(false)
}

fn build_schema() -> Schema {
    let mut schema_builder = Schema::builder();

    schema_builder.add_text_field("title", TEXT | STORED);

    schema_builder.add_bytes_field("hash");

    schema_builder.add_facet_field("location");

    schema_builder.add_text_field("body", TEXT);

    schema_builder.build()
}

fn start_tantivy() -> tantivy::Result<()> {
    let index_path = app_dir(AppDataType::UserData, &APP_INFO, "index").unwrap();
    info!("Using index file in: {:?}", index_path);

    let config_path = app_dir(AppDataType::UserConfig, &APP_INFO, "config").unwrap();
    let mut config_file = config_path.clone();
    config_file.push("config");
    config_file.set_extension("json");

    if !config_file.as_path().exists() {
        info!("Config file not found, copying default config");

        Notification::new().summary("Welcome!")
                        .body("Since this is your first time running podium, it will take a few minutes to index your files.")
                        .show()
                        .unwrap();

        let default_config_path = Path::new("debug_default_config.json");
        fs::copy(default_config_path, &config_file).unwrap();
    }  

    info!("Loading config file from: {:?}", config_file);
    let mut settings = Config::default();
    settings.merge(File::from(config_file)).unwrap();

    debug!("\n{:?} \n\n-----------", settings.try_into::<HashMap<String, Vec<String>>>().unwrap());

    let schema = build_schema();

    let index = Index::open_or_create(MmapDirectory::open(&index_path).unwrap(), schema.clone())?;

    let mut index_writer = index.writer(50_000_000)?;

    let title = schema.get_field("title").unwrap();
    let hash = schema.get_field("hash").unwrap();
    let location = schema.get_field("location").unwrap();
    let body = schema.get_field("body").unwrap();


    let walker = WalkDir::new("test_files").into_iter();
    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = entry.unwrap();
        if !entry.file_type().is_dir() {
            let entry_path = entry.path();
            let canonical_path = entry_path.canonicalize().unwrap();
            let location_facet = canonical_path.to_str().unwrap();
            if TextIndexer::supports_extension(entry_path.extension().unwrap()) {
                let file_hash;
                {
                    let mut file = fs::File::open(&entry_path).unwrap();
                    let mut file_buffer = Vec::new();
                    file.read_to_end(&mut file_buffer);
                    file_hash = blake2b(file_buffer.as_slice());
                }
                trace!("Hash of file is: {:?}", file_hash);
                let mut new_doc = Document::default();
                let indexed_content = TextIndexer::index_file(entry_path);
                trace!("{:?}", indexed_content);
                new_doc.add_text(title, &indexed_content.name);
                new_doc.add_facet(location, location_facet);
                new_doc.add_bytes(hash, file_hash.as_bytes().to_vec());
                new_doc.add_text(body, &indexed_content.body);
                index_writer.add_document(new_doc);
            }
        }
    }

    index_writer.commit()?;

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;

    let searcher = reader.searcher();

    let query_parser = QueryParser::for_index(&index, vec![title, body]);

    info!("Searching for a file with \"digimon\"...");
    let query = query_parser.parse_query("digimon")?;

    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;

    for (_score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address)?;
        info!("{}", schema.to_json(&retrieved_doc));
    }

    Ok(())
}