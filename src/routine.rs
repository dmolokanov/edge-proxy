use std::net::ToSocketAddrs;

use failure::ResultExt;
use futures::future::join_all;
use hyper::server::conn::AddrIncoming;
use hyper::Server;
use log::info;
use tokio::runtime::Runtime;

use crate::{Error, ErrorKind, ProxyService, ServiceSettings, Settings};

pub struct Routine {
    settings: Settings,
}

impl Routine {
    pub fn new(settings: Settings) -> Self {
        Routine { settings }
    }

    pub fn run_until(&self) -> Result<(), Error> {
        let mut runtime = tokio::runtime::Runtime::new().context(ErrorKind::Tokio)?;

        let services = self
            .settings
            .services()
            .iter()
            .map(|settings| start_proxy(&mut runtime, settings).unwrap());

        runtime.block_on(join_all(services));

        info!("Shutdown completed");

        Ok(())
    }
}

fn start_proxy(
    runtime: &mut Runtime,
    settings: &ServiceSettings,
) -> Result<Server<AddrIncoming, ProxyService>, Error> {
    info!(
        "Starting proxy server {} {}",
        settings.name(),
        settings.entrypoint()
    );

    let url = settings.entrypoint().clone();
    let addr = url
        .to_socket_addrs()
        .context(ErrorKind::InvalidUrl(url.to_string()))?
        .next()
        .ok_or_else(|| {
            ErrorKind::InvalidUrlWithReason(url.to_string(), "URL has no address".to_string())
        })?;

    let new_service = ProxyService {};

    let server = Server::bind(&addr).serve(new_service);

    Ok(server)
}
