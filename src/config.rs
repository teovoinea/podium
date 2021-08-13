use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg};

use tracing::Level;

use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct AppConfig {
    pub scan_directories: Vec<PathBuf>,
    pub verbosity: Level,
    pub port: u16,
}

pub fn get_config() -> AppConfig {
    let home_dir = dirs::home_dir().unwrap();
    let matches = app_from_crate!()
        .arg(
            Arg::with_name("scan-directories")
                .short("s")
                .long("scan-directories")
                .required(false)
                .default_value(home_dir.to_str().unwrap())
                .takes_value(true)
                .use_delimiter(true)
                .require_delimiter(true)
                .value_delimiter(",")
                .validator(path_validator)
                .help("Directories to scan then watch"),
        )
        .arg(
            Arg::with_name("verbose")
                .multiple(true)
                .short("v")
                .required(false)
                .help("Verbosity level. Up to 4."),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .default_value("8080")
                .required(false)
                .validator(port_validator)
                .help("Port to host query resolver"),
        )
        .get_matches();

    let scan_directories = matches
        .values_of("scan-directories")
        .unwrap()
        .map(|path| PathBuf::from(path))
        .collect::<Vec<PathBuf>>();

    dbg!(matches.occurrences_of("verbose"));

    let verbosity = match matches.occurrences_of("verbose") {
        4 => Level::TRACE,
        3 => Level::DEBUG,
        2 => Level::INFO,
        1 => Level::WARN,
        0 | _ => Level::ERROR,
    };

    let port = match matches.value_of("port") {
        Some(port_val) => port_val.parse::<u16>().unwrap(),
        None => 8080,
    };

    AppConfig {
        scan_directories,
        verbosity,
        port,
    }
}

fn path_validator(v: String) -> Result<(), String> {
    let broken_paths: Vec<&str> = v
        .split(",")
        .filter(|path| !Path::new(path).exists())
        .collect();

    if broken_paths.len() > 0 {
        return Err(format!(
            "The following paths could not be resolved: {:?}",
            broken_paths
        ));
    }
    Ok(())
}

fn port_validator(v: String) -> Result<(), String> {
    let try_port = v.parse::<u16>();
    if let Ok(port) = try_port {
        if port >= 1 {
            return Ok(());
        }
    }

    Err(String::from(
        "The port value needs to be a number >= 1 and <= 65535",
    ))
}
