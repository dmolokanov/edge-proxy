use std::net::ToSocketAddrs;

use failure::{Fail, ResultExt};
use futures::future::join_all;
use futures::{Future, IntoFuture};
use hyper::Server;
use log::{debug, info};
use tokio::runtime::Runtime;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Receiver;

use crate::proxy::{get_config, Client, ProxyService};
use crate::signal::ShutdownSignal;
use crate::{Error, ErrorKind, ServiceSettings, Settings};

pub struct Routine {
    settings: Settings,
}

impl Routine {
    pub fn new(settings: Settings) -> Self {
        Routine { settings }
    }

    pub fn run_until(&self, signal: ShutdownSignal) -> Result<(), Error> {
        let mut runtime = Runtime::new().context(ErrorKind::Tokio)?;

        let services = self.settings.services().to_vec();

        let (mut senders, mut proxies) = (Vec::new(), Vec::new());
        for settings in services.into_iter() {
            let (tx, rx) = oneshot::channel();
            senders.push(tx);

            let proxy = start_proxy(&settings, rx);
            proxies.push(proxy);
        }

        let shutdown_signal = signal.map(move |_| {
            debug!("Shutdown signalled");
            for tx in senders {
                tx.send(()).unwrap_or(())
            }
        });
        runtime.spawn(shutdown_signal);

        runtime.block_on(join_all(proxies))?;

        info!("Shutdown completed");

        Ok(())
    }
}

fn start_proxy(
    settings: &ServiceSettings,
    shutdown: Receiver<()>,
) -> impl Future<Item = (), Error = Error> {
    let settings = settings.clone();

    info!(
        "Starting proxy server {} {}",
        settings.name(),
        settings.entrypoint()
    );

    settings
        .entrypoint()
        .to_socket_addrs()
        .map_err(|err| {
            Error::from(err.context(ErrorKind::InvalidUrl(settings.entrypoint().to_string())))
        })
        .and_then(|mut addrs| {
            addrs.next().ok_or_else(|| {
                let err = ErrorKind::InvalidUrlWithReason(
                    settings.entrypoint().to_string(),
                    "URL has no address".to_string(),
                );
                Error::from(err)
            })
        })
        .and_then(move |addr| {
            let config = get_config(&settings)?;
            let client = Client::new(config);
            let new_service = ProxyService::new(client);

            let server = Server::bind(&addr)
                .serve(new_service)
                .with_graceful_shutdown(shutdown)
                .map_err(Error::from);

            info!(
                "Listening on {} with 1 thread for {}",
                settings.entrypoint(),
                settings.name()
            );

            Ok(server)
        })
        .into_future()
        .flatten()
}
