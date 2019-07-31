use std::fs;

use failure::ResultExt;
use native_tls::{Certificate, TlsConnector};
use url::Url;

use crate::{Error, ErrorKind, ServiceSettings};

#[derive(Clone)]
pub struct Config<T>
where
    T: TokenSource,
{
    host: Url,
    token: T,
    tls: TlsConnector,
}

impl<T> Config<T>
where
    T: TokenSource,
{
    pub fn new(host: Url, token: T, tls: TlsConnector) -> Self {
        Config { host, token, tls }
    }

    pub fn host(&self) -> &Url {
        &self.host
    }

    pub fn tls(&self) -> &TlsConnector {
        &self.tls
    }

    pub fn token(&self) -> &impl TokenSource {
        &self.token
    }
}

pub fn get_config(settings: &ServiceSettings) -> Result<Config<ValueToken>, Error> {
    let token = fs::read_to_string(settings.token())
        .context(ErrorKind::File(settings.token().display().to_string()))?;

    let file = fs::read_to_string(settings.certificate()).context(ErrorKind::File(
        settings.certificate().display().to_string(),
    ))?;
    let cert = Certificate::from_pem(file.as_bytes())?;

    let tls = TlsConnector::builder().add_root_certificate(cert).build()?;

    Ok(Config::new(
        settings.backend().clone(),
        ValueToken(Some(token)),
        tls,
    ))
}

pub trait TokenSource {
    fn get(&self) -> Option<String>;
}

#[derive(Clone, Debug)]
pub struct ValueToken(Option<String>);

impl TokenSource for ValueToken {
    fn get(&self) -> Option<String> {
        self.0.clone()
    }
}
