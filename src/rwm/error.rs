use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum MiniWMError {
    #[error("display not found")]
    DisplayNotFound,
    #[error("screen not found")]
    ScreenNotFound,
    #[error("generic error")]
    GenericError(String),
}
