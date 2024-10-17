use std::error::Error;
use std::fmt;
#[derive(Debug)]
pub struct SendRequestError {
    kind: ErrorKind,
    message: String,
    source: Option<Box<dyn Error + 'static + Send + Sync>>,
}

#[derive(Debug)]
pub enum ErrorKind {
    ConnectionError,
    StatusError,
    InvalidPhoneNumber,
}

impl Error for SendRequestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_deref()
            .map(|err| err as &(dyn Error + 'static))
    }
}

impl fmt::Display for SendRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::ConnectionError => write!(f, "Connection Error"),
            ErrorKind::StatusError => write!(f, "Status Error"),
            ErrorKind::InvalidPhoneNumber => write!(f, "Invalid Phone Number"),
        }
    }
}

impl SendRequestError {
    pub fn new(
        kind: ErrorKind,
        message: String,
        source: Option<Box<dyn Error + 'static + Send + Sync>>,
    ) -> Self {
        Self {
            kind,
            message,
            source,
        }
    }
}
