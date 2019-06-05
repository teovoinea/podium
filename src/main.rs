extern crate app_dirs;
extern crate blake2b_simd;
extern crate config;
#[macro_use] extern crate log;
extern crate notify;
extern crate notify_rust;
extern crate simple_logger;
extern crate sysbar;
extern crate tantivy;

use std::sync::mpsc::channel;
use std::thread;

mod indexers;
mod query_executor;
mod tantivy_api;
mod tantivy_process;
mod file_watcher;
use tantivy_process::start_tantivy;

fn main() {
    simple_logger::init().unwrap();
    let (query_tx, query_rx) = channel();
    let tantivy_query_tx = query_tx.clone();
    let tantivy_thread = thread::Builder::new().name("tantivy".to_string()).spawn(move || {
        start_tantivy((tantivy_query_tx, query_rx))
    });

    let mut bar = sysbar::Sysbar::new("P");
    bar.add_item(
        "Search",
        Box::new(move || {
            println!("Searching!");
            query_tx.send("digimon".to_string()).unwrap();
        }),
    );

    bar.add_quit_item("Quit");

    bar.display();

    trace!("Taskbar has quit, cleaning up remaining threads...");

    // TODO: Handle error
    tantivy_thread.unwrap().join();
}