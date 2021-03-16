extern crate podium_lib;
use podium_lib::config::{get_config, AppConfig};
use podium_lib::routes::app_state::*;
use podium_lib::routes::search;
use podium_lib::tantivy_process::{start_tantivy, tantivy_init, TantivyConfig};

use std::io;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use app_dirs::*;
use tokio;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, prelude::*, registry::Registry};

use tracing_flame::FlameLayer;

const APP_INFO: AppInfo = AppInfo {
    name: "Podium",
    author: "Teodor Voinea",
};

#[tokio::main]
async fn main() -> io::Result<()> {
    let config = get_config();

    setup_global_subscriber(&config);

    let local = tokio::task::LocalSet::new();

    // Get or create settings
    let settings = get_or_create_settings(&config);

    let (searcher, mut tantivy_wrapper) = tantivy_init(&settings).unwrap();

    let _tantivy_thread = tokio::spawn(async move {
        start_tantivy(&settings, &mut tantivy_wrapper)
            .await
            .unwrap();
    });

    let sys = actix_rt::System::run_in_tokio("server", &local);

    let app_state = web::Data::new(AppState { searcher: searcher });

    let server_res = HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default() // <- Construct CORS middleware builder
                    .send_wildcard(),
            )
            .app_data(app_state.clone())
            .configure(search::server_config)
    })
    .bind(format!("127.0.0.1:{}", config.port))?
    .run()
    .await?;

    sys.await?;

    Ok(server_res)

    // if tantivy_thread.unwrap().join().is_err() {
    //     error!("Failed to join tantivy thread");
    // }
}

fn get_or_create_settings(app_config: &AppConfig) -> TantivyConfig {
    let index_path = app_dir(AppDataType::UserData, &APP_INFO, "index").unwrap();
    info!("Using index file in: {:?}", index_path);

    let state_path = app_dir(AppDataType::UserData, &APP_INFO, "state").unwrap();
    let mut initial_processing_file = state_path.clone();
    initial_processing_file.push("initial_processing");

    TantivyConfig {
        index_path: index_path,
        scan_directories: app_config.scan_directories.clone(),
        initial_processing_file: initial_processing_file,
    }
}

fn setup_global_subscriber(config: &AppConfig) -> impl Drop {
    let (flame_layer, _guard) = FlameLayer::with_file("./tracing.folded").unwrap();
    let _t = tracing_subscriber::fmt()
        .with_max_level(config.verbosity.clone())
        .finish()
        .with(flame_layer)
        .try_init();

    _guard
}
