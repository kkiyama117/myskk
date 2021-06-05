use std::fmt::Display;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KeyEventFormatError {
    #[error("{}", .0)]
    ParseFailed(KeyEventParseFailedError),
    #[error("keysymNotFound")]
    KeysymNotFound,
}

#[derive(Debug, Error, Clone)]
pub struct KeyEventParseFailedError {
    // position: u8,
    // #[source]
    // source: String,
    pub msg: String,
}

impl Into<KeyEventFormatError> for KeyEventParseFailedError {
    fn into(self) -> KeyEventFormatError {
        KeyEventFormatError::ParseFailed(self)
    }
}

impl Display for KeyEventParseFailedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}
