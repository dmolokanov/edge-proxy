use std::env;
use std::path::Path;

use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use log::{info, LevelFilter};

use crate::{Error, Settings};

pub fn init() -> Result<Settings, Error> {
    let matches = create_app().get_matches();

    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .default_format_module_path(false)
        .parse_filters(&env::var("PROXY_LOG").unwrap_or_default())
        .init();

    info!("Starting proxy server");

    let config_file = matches
        .value_of_os("config")
        .and_then(|name| {
            let path = Path::new(name);
            info!("Using config file: {}", path.display());
            Some(path)
        })
        .or_else(|| {
            info!("Using default configuration");
            None
        });

    let settings = Settings::new(config_file)?;

    Ok(settings)
}

fn create_app() -> App<'static, 'static> {
    App::new(crate_name!())
        .author(crate_authors!("\n"))
        .version(crate_version!())
        .about(crate_description!())
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets proxy configuration file")
                .takes_value(true),
        )
}
