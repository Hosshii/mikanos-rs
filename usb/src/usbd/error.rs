use crate::xhci::{
    error::Error as XHCIError,
    trb::{Trb, TrbType},
};

use super::descriptor::TryFromBytesError;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(ErrorKind);

impl Error {
    pub fn unexpected_trb(expected: TrbType, actual: Trb) -> Self {
        Self(ErrorKind::UnexpectedTrb(expected, actual))
    }

    pub fn unexpected_descriptor() -> Self {
        Self(ErrorKind::UnexpectedDescriptor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    UnexpectedTrb(TrbType, Trb),
    UnexpectedDescriptor,
    Descriptor(TryFromBytesError),
    XHCIError(XHCIError),
}

impl From<TryFromBytesError> for Error {
    fn from(value: TryFromBytesError) -> Self {
        Self(ErrorKind::Descriptor(value))
    }
}

impl From<XHCIError> for Error {
    fn from(value: XHCIError) -> Self {
        Self(ErrorKind::XHCIError(value))
    }
}
