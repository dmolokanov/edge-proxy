use std::sync::Arc;

use failure::Compat;
use futures::future::FutureResult;
use futures::{future, Future};
use hyper::service::{NewService, Service};
use hyper::{Body, Request, Response};

use crate::proxy::{Client, TokenSource};
use crate::Error;

pub struct ProxyService<T: TokenSource, S> {
    client: Arc<Client<T, S>>,
}

impl<T: TokenSource, S> ProxyService<T, S> {
    pub fn new(client: Client<T, S>) -> Self {
        ProxyService {
            client: Arc::new(client),
        }
    }
}

impl<T: TokenSource, S> Clone for ProxyService<T, S> {
    fn clone(&self) -> Self {
        ProxyService {
            client: self.client.clone(),
        }
    }
}

impl<T: TokenSource, S> Service for ProxyService<T, S> {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Compat<Error>;
    type Future = Box<dyn Future<Item = Response<Self::ResBody>, Error = Self::Error> + Send>;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        Box::new(future::ok(Response::new(Body::from("Hello"))))
    }
}

impl<T: TokenSource, S> NewService for ProxyService<T, S> {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Compat<Error>;
    type Service = Self;
    type Future = FutureResult<Self::Service, Self::InitError>;
    type InitError = Compat<Error>;

    fn new_service(&self) -> Self::Future {
        future::ok(self.clone())
    }
}
