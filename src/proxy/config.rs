use native_tls::TlsConnector;
use url::Url;

use crate::Error;

#[derive(Clone)]
pub struct Config<T> {
    host: Url,
    token: T,
    tls: TlsConnector,
}

impl<T: TokenSource> Config<T> {
    pub fn new(host: Url, token: T, tls: TlsConnector) -> Self {
        Config { host, token, tls }
    }

    pub fn tls(&self) -> &TlsConnector {
        &self.tls
    }
}

pub fn get_config(host: Url) -> Result<Config<ValueToken>, Error> {
    let token = ValueToken(Some("token".to_owned()));
    let tls = TlsConnector::new()?;

    Ok(Config::new(host, token, tls))
}

pub trait TokenSource {
    fn get(&self) -> Result<Option<String>, Error>;
}

#[derive(Clone, Debug)]
pub struct ValueToken(Option<String>);

impl TokenSource for ValueToken {
    fn get(&self) -> Result<Option<String>, Error> {
        Ok(self.0.clone())
    }
}
