use std::net::ToSocketAddrs;

use failure::{Fail, ResultExt};
use futures::future::join_all;
use futures::{Future, IntoFuture};
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
        let mut runtime = Runtime::new().context(ErrorKind::Tokio)?;

        let services = self.settings.services().to_vec();

        let services = services.into_iter().map(|settings| start_proxy(&settings));

        runtime.block_on(join_all(services))?;

        info!("Shutdown completed");

        Ok(())
    }
}

fn start_proxy(settings: &ServiceSettings) -> impl Future<Item = (), Error = Error> {
    info!(
        "Starting proxy server {} {}",
        settings.name(),
        settings.entrypoint()
    );

    let name = settings.name().to_string();
    let url = settings.entrypoint().clone();

    url.to_socket_addrs()
        .map_err(|err| Error::from(err.context(ErrorKind::InvalidUrl(url.to_string()))))
        .and_then(|mut addrs| {
            addrs.next().ok_or_else(|| {
                let err = ErrorKind::InvalidUrlWithReason(
                    url.to_string(),
                    "URL has no address".to_string(),
                );
                Error::from(err)
            })
        })
        .into_future()
        .and_then(move |addr| {
            let new_service = ProxyService {};
            let server = Server::bind(&addr)
                .serve(new_service)
                .map_err(|err| Error::from(err.context(ErrorKind::Hyper)));

            info!("Listening on {} with 1 thread for {}", url, name);

            server
        })
}
