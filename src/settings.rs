use std::path::{Path, PathBuf};

use config::{Config, ConfigError, File, FileFormat};
use failure::Fail;
use serde::Deserialize;
use url::Url;
use url_serde;

use crate::{Error, ErrorKind};

pub const DEFAULTS: &str = include_str!("../config/default.yaml");

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    services: Vec<ServiceSettings>,
    api: ApiSettings,
}

impl Settings {
    pub fn new(path: Option<&Path>) -> Result<Settings, Error> {
        let mut config = Config::default();
        config.merge(File::from_str(DEFAULTS, FileFormat::Yaml))?;

        if let Some(path) = path {
            config.merge(File::from(path))?;
        }

        let settings = config.try_into()?;
        Ok(settings)
    }

    pub fn services(&self) -> &Vec<ServiceSettings> {
        &self.services
    }

    pub fn api(&self) -> &ApiSettings {
        &self.api
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServiceSettings {
    name: String,

    #[serde(with = "url_serde")]
    entrypoint: Url,

    #[serde(with = "url_serde")]
    backend: Url,

    certificate: PathBuf,
}

impl ServiceSettings {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn entrypoint(&self) -> &Url {
        &self.entrypoint
    }

    pub fn backend(&self) -> &Url {
        &self.backend
    }

    pub fn certificate(&self) -> &Path {
        &self.certificate
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ApiSettings {
    #[serde(with = "url_serde")]
    entrypoint: Url,
}

impl ApiSettings {
    pub fn entrypoint(&self) -> &Url {
        &self.entrypoint
    }
}

impl From<ConfigError> for Error {
    fn from(error: ConfigError) -> Self {
        Error::from(error.context(ErrorKind::LoadSettings))
    }
}
