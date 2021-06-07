use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum KeyEventError {
  #[error("Parse failed:{msg}")]
  ParseFailed { msg: String },
}
