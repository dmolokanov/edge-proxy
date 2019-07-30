use std::fmt;
use std::fmt::Display;

use failure::{Backtrace, Context, Fail};
use hyper::Error as HyperError;
use native_tls::Error as NativeTlsError;

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "Could not load settings")]
    LoadSettings,

    #[fail(display = "Could not initialize tokio runtime")]
    Tokio,

    #[fail(display = "Invalid URL {:?}", _0)]
    InvalidUrl(String),

    #[fail(display = "Invalid URL {:?}: {}", _0, _1)]
    InvalidUrlWithReason(String, String),

    #[fail(display = "HTTP connection error")]
    Hyper,

    #[fail(display = "A native TLS error occurred.")]
    NativeTls,

    #[fail(display = "Error")]
    Generic,
}

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        self.inner.get_context()
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Self {
        Error { inner }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<HyperError> for Error {
    fn from(error: HyperError) -> Self {
        Error {
            inner: error.context(ErrorKind::Hyper),
        }
    }
}

impl From<NativeTlsError> for Error {
    fn from(error: NativeTlsError) -> Self {
        Error {
            inner: error.context(ErrorKind::NativeTls),
        }
    }
}
