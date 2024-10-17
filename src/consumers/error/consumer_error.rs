use std::fmt;

#[derive(Debug)]
/// 錯誤類別 for Consumer
pub enum ConsumerError {
    StartStateError(Box<dyn std::error::Error + Send + Sync>),
    UpdateStateError(Box<dyn std::error::Error + Send + Sync>),
    EndStateError(Box<dyn std::error::Error + Send + Sync>),
}

impl std::error::Error for ConsumerError {}

impl fmt::Display for ConsumerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            self::ConsumerError::StartStateError(err) => write!(f, "Start State Error: {}", err),
            self::ConsumerError::UpdateStateError(err) => write!(f, "Update State Error {}", err),
            self::ConsumerError::EndStateError(err) => write!(f, "End State Error: {}", err),
        }
    }
}
