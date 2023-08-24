use crate::graphic::error::Error as GraphicError;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(ErrorKind);

impl Error {
    pub fn invalid_addr() -> Error {
        Error(ErrorKind::InvalidAddr)
    }

    pub fn too_many_devices() -> Error {
        Error(ErrorKind::TooManyDevices)
    }
}

impl From<GraphicError> for Error {
    fn from(value: GraphicError) -> Self {
        Self(ErrorKind::Graphic(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ErrorKind {
    InvalidAddr,
    TooManyDevices,
    Graphic(GraphicError),
}
