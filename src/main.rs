#![feature(test)]

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
extern crate notify_rust;
extern crate pdf_extract;
extern crate regex;
extern crate reverse_geocoder;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate sysbar;
extern crate simple_logger;
extern crate tantivy;
extern crate test;
extern crate tract_core;
extern crate tract_tensorflow;
extern crate web_view;

use web_view::*;

use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, Instant};

mod ui;
mod indexers;
mod query_executor;
mod tantivy_api;
mod tantivy_process;
mod file_watcher;
use tantivy_process::start_tantivy;


#[derive(Debug, Serialize, Deserialize)]
struct UserData {
    query: String,
    results: Vec<String>,
}

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


fn render(webview: &mut WebView<UserData>) -> WVResult {
    let render_tasks = {
        let data = webview.user_data();
        format!("rpc.render({})", serde_json::to_string(&data.results).unwrap())
    };
    webview.eval(&render_tasks)
}

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
    Init,
    Log { text: String },
    Search { query: String },
}

fn inline_style(s: &str) -> String {
    format!(r#"<style type="text/css">{}</style>"#, s)
}

fn inline_script(s: &str) -> String {
    format!(r#"<script type="text/javascript">{}</script>"#, s)
}