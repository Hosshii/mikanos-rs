use crate::xhci::error::Error as XHCIError;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(ErrorKind);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    XHCIError(XHCIError),
}

impl From<XHCIError> for Error {
    fn from(value: XHCIError) -> Self {
        Self(ErrorKind::XHCIError(value))
    }
}
