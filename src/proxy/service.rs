use failure::Compat;
use futures::future::FutureResult;
use futures::{future, Future};
use hyper::service::{NewService, Service};
use hyper::{Body, Request, Response};

use crate::Error;

#[derive(Clone)]
pub struct ProxyService {
    //    client: Client<T, S>,
}

impl Service for ProxyService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Compat<Error>;
    type Future = Box<Future<Item = Response<Self::ResBody>, Error = Self::Error> + Send>;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        Box::new(future::ok(Response::new(Body::from("Hello"))))
    }
}

impl NewService for ProxyService {
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
