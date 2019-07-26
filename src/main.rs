use log::error;

use edge_proxy::app::Settings;
use edge_proxy::{app, Error};

fn main() {
    env_logger::init();

    if let Err(e) = run() {
        error!("{}", e);
        std::process::exit(1)
    }
}

fn run() -> Result<(), Error> {
    let settings = app::init();

    let main = Main::new(settings);
    main.run_until()?;

    Ok(())
}

pub struct Main {
    settings: Settings,
}

impl Main {
    pub fn new(settings: Settings) -> Self {
        Main { settings }
    }

    pub fn run_until(&self) -> Result<(), Error> {
        Ok(())
    }
}
