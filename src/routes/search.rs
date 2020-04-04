use actix_web::{web, HttpRequest, HttpResponse};
use crate::contracts::AppState::*;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/search/{query}", web::get().to(index));
}

async fn index(app_state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    println!("{:?}", req);

    let query: String = req.match_info().query("query").parse().unwrap();
    println!("{:?}", query);

    println!("Getting query channel");
    let query_channel = &app_state.query_sender;

    println!("Sending from query channel");
    if let Err(err) = query_channel.send(query) {
        println!("{:?}", err);
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
