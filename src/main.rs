use edge_proxy::logging;
use edge_proxy::{app, Error, Routine};

fn main() {
    if let Err(e) = run() {
        logging::failure(&e);
        std::process::exit(1)
    }
}

fn run() -> Result<(), Error> {
    let settings = app::init()?;

    let main = Routine::new(settings);
    main.run_until()?;

    Ok(())
}
