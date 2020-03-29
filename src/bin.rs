use std::thread;

extern crate podium_lib;
use podium_lib::tantivy_process::start_tantivy;
use podium_lib::query_executor::QueryResponse;

#[macro_use]
extern crate log;

use std::io;

use actix_web::{http, middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use actix_cors::Cors;
use crossbeam::channel::{Sender, Receiver, unbounded};

struct AppState {
    query_sender: Sender<String>,
    result_receiver: Receiver<QueryResponse>,
}

async fn index(
    app_state: web::Data<AppState>,
    req: HttpRequest,
) -> HttpResponse {
    println!("{:?}", req);

    let query: String = req.match_info().query("query").parse().unwrap();
    println!("{:?}", query);

    println!("Getting query channel");
    let query_channel = &app_state.query_sender;

    println!("Sending from query channel");
    if let Err(err) = query_channel.send(query) {
        println!("{:?}",err);
    }

    println!("Getting response channel");
    let resp = &app_state.result_receiver;
    
    println!("Getting from response channel");
    let result = match resp.recv() {
        Err(err) => format!("Err: {:?}", err),
        Ok(r) => {
            println!("Ok!: {:?}", r);
            serde_json::to_string(&r).unwrap()
        }
    };
    
    info!("Found results: {:?}", &result);

    HttpResponse::Ok().body(result)
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    simple_logger::init().unwrap();
    let (query_tx, query_rx):(Sender<String>, Receiver<String>) = unbounded();
    let (result_tx, result_rx):(Sender<QueryResponse>, Receiver<QueryResponse>) = unbounded();
    let tantivy_query_tx = query_tx.clone();
    let tantivy_thread = thread::Builder::new()
        .name("tantivy".to_string())
        .spawn(move || start_tantivy((tantivy_query_tx, query_rx), result_tx));

    let app_state = web::Data::new(AppState {
        query_sender: query_tx.clone(),
        result_receiver: result_rx.clone(),
    });

    HttpServer::new(move || {
            App::new()
                .wrap(
                    Cors::new() // <- Construct CORS middleware builder
                    .send_wildcard()
                    .finish())
                .app_data(app_state.clone())
                .route("/search/{query}", web::get().to(index))
        })
        .bind("127.0.0.1:8080")?
        .run()
        .await

    // if tantivy_thread.unwrap().join().is_err() {
    //     error!("Failed to join tantivy thread");
    // }
}
