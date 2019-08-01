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

    use openssl::asn1::Asn1Time;
    use openssl::hash::MessageDigest;
    use openssl::nid::Nid;
    use openssl::pkey::PKey;
    use openssl::rsa::Rsa;
    use openssl::x509::extension::{
        AuthorityKeyIdentifier, BasicConstraints, ExtendedKeyUsage, KeyUsage, SubjectKeyIdentifier,
    };
    use openssl::x509::{X509Name, X509};

    use tempfile::TempDir;
    use url::Url;

    use crate::proxy::{get_config, TokenSource};
    use crate::{ErrorKind, ServiceSettings};

    #[test]
    fn it_loads_config_from_filesystem() {
        let dir = TempDir::new().unwrap();

        let token = dir.path().join("token");
        fs::write(&token, "token").unwrap();

        let cert = dir.path().join("cert.pem");
        generate_certificate(&cert);

        let settings = ServiceSettings::new(
            "management".to_owned(),
            Url::parse("http://localhost:3000").unwrap(),
            Url::parse("https://iotedged:30000").unwrap(),
            &cert,
            &token,
        );

        let config = get_config(&settings).unwrap();

        assert_eq!(config.token().get(), Some("token".to_string()));
        assert_eq!(
            config.host(),
            &Url::parse("https://iotedged:30000").unwrap()
        );
    }

    #[test]
    fn it_fails_to_load_config_if_token_file_not_exist() {
        let dir = TempDir::new().unwrap();

        let token = dir.path().join("token");
        let cert = dir.path().join("cert.pem");

        let settings = ServiceSettings::new(
            "management".to_owned(),
            Url::parse("http://localhost:3000").unwrap(),
            Url::parse("https://iotedged:30000").unwrap(),
            &cert,
            &token,
        );

        let err = get_config(&settings).err().unwrap();

        assert_eq!(err.kind(), &ErrorKind::File(token.display().to_string()));
    }

    #[test]
    fn it_fails_to_load_config_if_cert_not_exist() {
        let dir = TempDir::new().unwrap();

        let token = dir.path().join("token");
        fs::write(&token, "token").unwrap();

        let cert = dir.path().join("cert.pem");

        let settings = ServiceSettings::new(
            "management".to_owned(),
            Url::parse("http://localhost:3000").unwrap(),
            Url::parse("https://iotedged:30000").unwrap(),
            &cert,
            &token,
        );

        let err = get_config(&settings).err().unwrap();

        assert_eq!(err.kind(), &ErrorKind::File(cert.display().to_string()));
    }

    #[test]
    fn it_fails_to_load_config_if_cert_is_invalid() {
        let dir = TempDir::new().unwrap();

        let token = dir.path().join("token");
        fs::write(&token, "token").unwrap();

        let cert = dir.path().join("cert.pem");
        fs::write(&cert, "cert").unwrap();

        let settings = ServiceSettings::new(
            "management".to_owned(),
            Url::parse("http://localhost:3000").unwrap(),
            Url::parse("https://iotedged:30000").unwrap(),
            &cert,
            &token,
        );

        let err = get_config(&settings).err().unwrap();

        assert_eq!(err.kind(), &ErrorKind::NativeTls);
    }

    fn generate_certificate(path: &Path) {
        let rsa = Rsa::generate(2048).unwrap();
        let pkey = PKey::from_rsa(rsa).unwrap();

        let mut name = X509Name::builder().unwrap();
        name.append_entry_by_nid(Nid::COMMONNAME, "iotedged")
            .unwrap();
        let name = name.build();

        let mut builder = X509::builder().unwrap();
        builder.set_version(2).unwrap();
        builder.set_subject_name(&name).unwrap();
        builder.set_issuer_name(&name).unwrap();
        builder
            .set_not_before(&Asn1Time::days_from_now(0).unwrap())
            .unwrap();
        builder
            .set_not_after(&Asn1Time::days_from_now(365).unwrap())
            .unwrap();
        builder.set_pubkey(&pkey).unwrap();

        let basic_constraints = BasicConstraints::new().critical().ca().build().unwrap();
        builder.append_extension(basic_constraints).unwrap();
        let key_usage = KeyUsage::new()
            .digital_signature()
            .key_encipherment()
            .build()
            .unwrap();
        builder.append_extension(key_usage).unwrap();
        let ext_key_usage = ExtendedKeyUsage::new()
            .client_auth()
            .server_auth()
            .build()
            .unwrap();
        builder.append_extension(ext_key_usage).unwrap();
        let subject_key_identifier = SubjectKeyIdentifier::new()
            .build(&builder.x509v3_context(None, None))
            .unwrap();
        builder.append_extension(subject_key_identifier).unwrap();
        let authority_key_identifier = AuthorityKeyIdentifier::new()
            .keyid(true)
            .build(&builder.x509v3_context(None, None))
            .unwrap();
        builder.append_extension(authority_key_identifier).unwrap();

        builder.sign(&pkey, MessageDigest::sha256()).unwrap();

        let x509 = builder.build();

        fs::write(path, x509.to_pem().unwrap()).unwrap();
    }
}
