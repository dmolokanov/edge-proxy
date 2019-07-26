use edge_proxy::{app, Error};
use edge_proxy::{logging, Settings};

fn main() {
    if let Err(e) = run() {
        logging::failure(&e);
        std::process::exit(1)
    }
}

fn run() -> Result<(), Error> {
    let settings = app::init()?;

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
