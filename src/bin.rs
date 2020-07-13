extern crate podium_lib;
use podium_lib::contracts::app_state::*;
use podium_lib::routes::search;
use podium_lib::tantivy_process::{start_tantivy, tantivy_init};

#[macro_use]
extern crate log;

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};

use app_dirs::*;
use config::*;

const APP_INFO: AppInfo = AppInfo {
    name: "Podium",
    author: "Teodor Voinea",
};

#[tokio::main]
async fn main() -> io::Result<()> {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    let local = tokio::task::LocalSet::new();

    // Get or create settings
    let settings = get_or_create_settings();

    let (searcher, mut tantivy_wrapper) = tantivy_init(&settings).unwrap();

    let _tantivy_thread = tokio::spawn(async move {
        start_tantivy(settings, &mut tantivy_wrapper).await.unwrap();
    });

    let sys = actix_rt::System::run_in_tokio("server", &local);

    let app_state = web::Data::new(AppState { searcher: searcher });

    let server_res = HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::new() // <- Construct CORS middleware builder
                    .send_wildcard()
                    .finish(),
            )
            .app_data(app_state.clone())
            .configure(search::config)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    sys.await?;

    Ok(server_res)

    // if tantivy_thread.unwrap().join().is_err() {
    //     error!("Failed to join tantivy thread");
    // }
}

fn get_or_create_settings() -> HashMap<String, Vec<String>> {
    let index_path = app_dir(AppDataType::UserData, &APP_INFO, "index").unwrap();
    info!("Using index file in: {:?}", index_path);

    let state_path = app_dir(AppDataType::UserData, &APP_INFO, "state").unwrap();
    let mut initial_processing_file = state_path.clone();
    initial_processing_file.push("initial_processing");

    let config_path = app_dir(AppDataType::UserConfig, &APP_INFO, "config").unwrap();
    let mut config_file = config_path.clone();
    config_file.push("config");
    config_file.set_extension("json");

    if !config_file.as_path().exists() {
        info!("Config file not found, copying default config");
        let default_config_path = Path::new("debug_default_config.json");
        fs::copy(default_config_path, &config_file).unwrap();
    }

    info!("Loading config file from: {:?}", config_file);
    let mut settings = Config::default();
    settings.merge(File::from(config_file)).unwrap();

    // TODO: define a better config file
    let mut settings_dict = settings.try_into::<HashMap<String, Vec<String>>>().unwrap();

    settings_dict.insert(
        String::from("index_path"),
        vec![String::from(index_path.to_str().unwrap())],
    );
    settings_dict.insert(
        String::from("state_path"),
        vec![String::from(state_path.to_str().unwrap())],
    );
    settings_dict.insert(
        String::from("config_path"),
        vec![String::from(config_path.to_str().unwrap())],
    );

    settings_dict.insert(
        String::from("initial_processing"),
        vec![format!("{:?}", initial_processing_file.exists())],
    );

    settings_dict
}
