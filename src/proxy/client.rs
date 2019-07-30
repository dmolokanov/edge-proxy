use futures::Future;
use hyper::client::connect::Connect;
use hyper::client::{HttpConnector, ResponseFuture};
use hyper::service::Service;
use hyper::{Body, Client as HyperClient, Error as HyperError, Request, Response};
use native_tls::TlsConnector;
use url::Url;

use crate::proxy::config::TokenSource;
use crate::proxy::Config;
use crate::Error;
use hyper_tls::HttpsConnector;

pub struct Client<T> {
    config: Config<T>,
    client: HttpClient<HttpsConnector<HttpConnector>, Body>,
}

impl<T: TokenSource> Client<T> {
    pub fn new(config: Config<T>) -> Client<T> {
        let mut http = HttpConnector::new(4);
        http.enforce_http(false);

        let https = HttpsConnector::from((http, config.tls().clone()));

        Client {
            config,
            client: HttpClient(HyperClient::builder().build(https)),
        }
    }
}

impl<T> Client<T> {
    pub fn request(&self, req: Request<Body>) -> impl Future<Item = Response<Body>, Error = Error> {
        self.client.0.request(req).map_err(Error::from)
    }
}

pub struct HttpClient<C, B>(HyperClient<C, B>);

//impl<C, B> Service for HttpClient<C, B>
//    where
//        C: Connect + Sync + 'static,
//{
//    type ReqBody = Body;
//    type ResBody = B;
//    type Error = HyperError;
//    type Future = ResponseFuture;
//
//    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
//        self.0.request(req)
//    }
//}
