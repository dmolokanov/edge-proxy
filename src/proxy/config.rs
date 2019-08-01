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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use openssl::hash::MessageDigest;
    use openssl::nid::Nid;
    use openssl::pkey::PKey;
    use openssl::rsa::Rsa;
    use openssl::x509::{X509Name, X509};
    use tempfile::TempDir;
    use url::Url;

    use crate::proxy::{get_config, TokenSource};
    use crate::ServiceSettings;

    #[test]
    fn it_loads_config_from_filesystem() {
        let dir = TempDir::new().unwrap();

        let token = dir.path().join("token");
        fs::write(&token, "token").unwrap();

        let cert = dir.path().join("cert.pem");
        generate_cert(&cert);

        let settings = ServiceSettings::new(
            "management".to_owned(),
            Url::parse("http://localhost:3000").unwrap(),
            Url::parse("https://iotedged:30000").unwrap(),
            &cert,
            &token,
        );

        let config = get_config(&settings).expect("Config loading");
        assert_eq!(config.token().get(), Some("token".to_string()));
    }

    fn generate_cert(path: &Path) {
        let rsa = Rsa::generate(2048).unwrap();
        let pkey = PKey::from_rsa(rsa).unwrap();

        let mut name = X509Name::builder().unwrap();
        name.append_entry_by_nid(Nid::COMMONNAME, "localhost")
            .unwrap();
        let name = name.build();

        let mut builder = X509::builder().unwrap();
        builder.set_version(2).unwrap();
        builder.set_subject_name(&name).unwrap();
        builder.set_issuer_name(&name).unwrap();
        builder.set_pubkey(&pkey).unwrap();
        builder.sign(&pkey, MessageDigest::sha256()).unwrap();

        let certificate: X509 = builder.build();
        let pem = certificate.to_pem().unwrap();

        fs::write(path, pem).unwrap();
    }
}
