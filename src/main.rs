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

    let html = format!(include_str!("ui/index.html"),
		styles = inline_style(include_str!("ui/styles.css")),
		scripts = inline_script(include_str!("ui/app.js")),
    );

    web_view::builder()
        .title("Podium")
        .content(Content::Html(html))
        .size(800, 600)
        .resizable(true)
        .debug(true)
        .user_data(UserData {
            query: "".to_string(),
            results: vec![],
        })
        .invoke_handler(|webview, arg| {
            use Cmd::*;

            let data = webview.user_data_mut();

            match serde_json::from_str(arg).unwrap() {
                Init => (),
                Log { text } => println!("{}", text),
                Search { query } => {
                    println!("Here is where I would search for {}", query);
                    let now = Instant::now();
                    query_tx.send(query).unwrap();
                    data.results = result_rx.recv().unwrap();
                    println!("It took {} microseconds to execute query", now.elapsed().as_micros());
                },
            }
            render(webview)
        })
        .run()
        .unwrap();

    // let mut bar = sysbar::Sysbar::new("P");
    // bar.add_item(
    //     "Search",
    //     Box::new(move || {
    //         println!("Searching!");
    //         query_tx.send("digimon".to_string()).unwrap();
    //     }),
    // );

    // bar.add_quit_item("Quit");

    // bar.display();

    // trace!("Taskbar has quit, cleaning up remaining threads...");

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