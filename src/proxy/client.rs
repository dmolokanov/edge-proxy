use futures::{Future, IntoFuture};
use http::header;
use hyper::client::connect::Connect;
use hyper::client::HttpConnector;
use hyper::{Body, Client as HyperClient, Request, Response};
use hyper_tls::HttpsConnector;
use log::info;

use crate::proxy::{Config, TokenSource};
use crate::Error;

#[derive(Clone)]
pub struct Client<T, S>
where
    T: TokenSource,
{
    config: Config<T>,
    client: S,
}

impl<T> Client<T, HyperHttpClient<HttpsConnector<HttpConnector>>>
where
    T: TokenSource,
{
    pub fn new(config: Config<T>) -> Self {
        let mut http = HttpConnector::new(4);
        http.enforce_http(false);

        let https = HttpsConnector::from((http, config.tls().clone()));
        let client = HyperHttpClient(HyperClient::builder().build(https));

        Client::with_client(client, config)
    }
}

impl<T, S> Client<T, S>
where
    T: TokenSource,
{
    pub fn with_client(client: S, config: Config<T>) -> Self {
        Client { config, client }
    }
}

impl<T, S> Client<T, S>
where
    T: TokenSource,
    S: HttpClient,
{
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

                // set host value in request header
                if let Ok(host) = req.uri().host().unwrap_or_default().parse() {
                    req.headers_mut().insert(header::HOST, host);
                }

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
        let request = format!("{} {} {:?}", req.method(), req.uri(), req.version());

        let fut = self.0.request(req).map_err(Error::from).map(move |res| {
            let body_length = res
                .headers()
                .get(header::CONTENT_LENGTH)
                .and_then(|length| length.to_str().ok().map(ToString::to_string))
                .unwrap_or_else(|| "-".to_string());

            info!("\"{}\" {} {}", request, res.status(), body_length);

            res
        });

        Box::new(fut)
    }
}

pub type ResponseFuture = Box<dyn Future<Item = Response<Body>, Error = Error> + Send>;

pub trait HttpClient {
    fn request(&self, req: Request<Body>) -> ResponseFuture;
}
