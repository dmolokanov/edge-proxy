use futures::Future;
use hyper::client::connect::Connect;
use hyper::client::HttpConnector;
use hyper::{Body, Client as HyperClient, Request, Response};
use hyper_tls::HttpsConnector;

use crate::proxy::{Config, TokenSource};
use crate::Error;

#[derive(Clone)]
pub struct Client<T, S> {
    config: Config<T>,
    client: S,
}

impl<T: TokenSource> Client<T, HyperHttpClient<HttpsConnector<HttpConnector>>> {
    pub fn new(config: Config<T>) -> Self {
        let mut http = HttpConnector::new(4);
        http.enforce_http(false);

        let https = HttpsConnector::from((http, config.tls().clone()));

        Client {
            config,
            client: HyperHttpClient(HyperClient::builder().build(https)),
        }
    }
}

impl<T, S: HttpClient> Client<T, S> {
    pub fn request(&self, req: Request<Body>) -> impl Future<Item = Response<Body>, Error = Error> {
        self.client.request(req)
    }
}

pub struct HyperHttpClient<C>(HyperClient<C>);

impl<C> HttpClient for HyperHttpClient<C>
where
    C: Connect + Sync + 'static,
{
    fn request(&self, req: Request<Body>) -> ResponseFuture {
        Box::new(self.0.request(req).map_err(Error::from))
    }
}

pub type ResponseFuture = Box<Future<Item = Response<Body>, Error = Error>>;

pub trait HttpClient {
    fn request(&self, req: Request<Body>) -> ResponseFuture;
}
