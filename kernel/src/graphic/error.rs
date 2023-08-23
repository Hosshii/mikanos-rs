use core::str::Utf8Error;

use super::pixel::PixelPosition;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(ErrorKind);

impl Error {
    pub fn invalid_pos(pos: PixelPosition) -> Error {
        Error(ErrorKind::InvalidPos(pos))
    }

    pub fn unsupported_font(c: char) -> Error {
        Error(ErrorKind::UnsuportedFont(c))
    }

    pub fn utf8(e: Utf8Error) -> Error {
        Error(ErrorKind::Utf8(e))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ErrorKind {
    InvalidPos(PixelPosition),
    UnsuportedFont(char),
    Utf8(Utf8Error),
}
