use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct JobError {
    message: String,
    source: Option<Box<dyn Error + 'static + Send + Sync>>,
}

impl Error for JobError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_deref()
            .map(|err| err as &(dyn Error + 'static))
    }
}

impl fmt::Display for JobError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl JobError {
    pub fn new(message: String, source: Option<Box<dyn Error + 'static + Send + Sync>>) -> Self {
        Self { message, source }
    }
}
