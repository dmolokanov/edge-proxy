use hyper::client::HttpConnector;
use hyper::service::service_fn;
use hyper::{rt, Client, Server};
use hyper_tls::HttpsConnector;
use log::*;
use native_tls::{Certificate, TlsConnector};
use std::fs::File;
use std::io::Read;
use tokio::prelude::future::Future;

fn main() {
    env_logger::init();

    let addr = ([127, 0, 0, 1], 3001).into();
    let new_service = move || {
        service_fn(move |req| {
            let mut f = File::open("device.cert").unwrap();
            let mut buffer = vec![];
            f.read_to_end(&mut buffer).unwrap();

            let cert = Certificate::from_der(buffer.as_slice()).unwrap();

            let mut http = HttpConnector::new(4);
            http.enforce_http(false);

            let tls = TlsConnector::builder()
                .add_root_certificate(cert)
                .build()
                .unwrap();

            let https = HttpsConnector::from((http, tls));

            let client = Client::builder().build::<_, hyper::Body>(https);
            client
                .get("https://hyper.rs".parse().unwrap())
                .map(|res| {
                    info!("{}", res.status());
                    res
                })
                .map_err(|e| {
                    error!("request error: {}", e);
                    e
                })
        })
    };

    let server = Server::bind(&addr)
        .serve(new_service)
        .map_err(|e| eprintln!("server error: {}", e));

    info!("starting server : {}", addr);

    rt::run(server);
}
/*
    [ ] Create transparent HTTP proxy that changes only URL part of the request
    [ ] Establish outgoing TLS connection
    [ ] Attach Authorization header to the request
    [ ] Setup live reloading of config file, token and CA
*/
/*

1. proxy should listen to connection on a TCP port on any network interface
2. for each connection spawn a new future using tokio
3. read URL, headers and body if exists and make a request to remote server and flush response to
    the client
4. close connection?

*/
