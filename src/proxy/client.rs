use futures::{Future, IntoFuture};
use http::header;
use hyper::client::connect::Connect;
use hyper::client::HttpConnector;
use hyper::{Body, Client as HyperClient, Request, Response};
use hyper_tls::HttpsConnector;

use crate::proxy::{Config, TokenSource};
use crate::Error;

#[derive(Clone)]
pub struct Client<T: TokenSource, S> {
    config: Config<T>,
    client: S,
}

impl<T: TokenSource> Client<T, HyperHttpClient<HttpsConnector<HttpConnector>>> {
    pub fn new(config: Config<T>) -> Self {
        let mut http = HttpConnector::new(4);
        http.enforce_http(false);

        let https = HttpsConnector::from((http, config.tls().clone()));
        let client = HyperHttpClient(HyperClient::builder().build(https));

        Client::with_client(client, config)
    }
}

impl<T: TokenSource, S> Client<T, S> {
    pub fn with_client(client: S, config: Config<T>) -> Self {
        Client { config, client }
    }
}

impl<T: TokenSource, S: HttpClient> Client<T, S> {
    pub fn request(
        &self,
        mut req: Request<Body>,
    ) -> impl Future<Item = Response<Body>, Error = Error> {
        self.config
            .host()
            .join(req.uri().path_and_query().map_or("", |p| p.as_str()))
            .map_err(Error::from)
            .and_then(|url| url.as_str().parse().map_err(Error::from))
            .and_then(|uri| {
                // set a full URL to redirect request to
                *req.uri_mut() = uri;

                // add authorization header with bearer token to authenticate request
                if let Some(token) = self.config.token().get() {
                    let token = format!("Bearer {}", token).parse()?;
                    req.headers_mut().insert(header::AUTHORIZATION, token);
                }

                Ok(req)
            })
            .map(|req| self.client.request(req))
            .into_future()
            .flatten()
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

pub type ResponseFuture = Box<dyn Future<Item = Response<Body>, Error = Error>>;

pub trait HttpClient {
    fn request(&self, req: Request<Body>) -> ResponseFuture;
}
