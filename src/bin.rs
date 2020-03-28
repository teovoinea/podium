use std::sync::mpsc::channel;
use std::thread;

extern crate podium_lib;
use podium_lib::tantivy_process::start_tantivy;

#[macro_use]
extern crate log;

fn main() {
    simple_logger::init().unwrap();
    let (query_tx, query_rx) = channel();
    let (result_tx, result_rx) = channel();
    let tantivy_query_tx = query_tx.clone();
    let tantivy_thread = thread::Builder::new()
        .name("tantivy".to_string())
        .spawn(move || start_tantivy((tantivy_query_tx, query_rx), result_tx));

    if tantivy_thread.unwrap().join().is_err() {
        error!("Failed to join tantivy thread");
    }
}
