use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, Instant};

extern crate podium_lib;
use podium_lib::tantivy_process::start_tantivy;
use podium_lib::ui;

fn main() {
    simple_logger::init().unwrap();
    let (query_tx, query_rx) = channel();
    let (result_tx, result_rx) = channel();
    let tantivy_query_tx = query_tx.clone();
    let tantivy_thread = thread::Builder::new().name("tantivy".to_string()).spawn(move || {
        start_tantivy((tantivy_query_tx, query_rx), result_tx)
    });

    ui::run_window(query_tx, result_rx);

    //     let now = Instant::now();
    // query_tx.send(query).unwrap();
    // data.results = result_rx.recv().unwrap();
    // println!("It took {} microseconds to execute query", now.elapsed().as_micros());

    // TODO: Handle error
    tantivy_thread.unwrap().join();
}