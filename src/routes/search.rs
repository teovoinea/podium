use crate::contracts::app_state::*;
use actix_web::{web, HttpRequest, HttpResponse};
use tracing::{info, span, Level};

pub fn server_config(cfg: &mut web::ServiceConfig) {
    cfg.route("/search/{query}", web::get().to(index));
}

async fn index(app_state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    println!("{:?}", req);

    let query: String = req.match_info().query("query").parse().unwrap();
    println!("{:?}", query);

    let response = span!(Level::INFO, "search_query").in_scope(|| app_state.searcher.search(query));

    let result = serde_json::to_string(&response).unwrap();

    info!("Found results: {:?}", &result);

    HttpResponse::Ok().body(result)
}
