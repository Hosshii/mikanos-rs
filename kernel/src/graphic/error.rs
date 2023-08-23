use super::pixel::Position;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Error(ErrorKind);

impl Error {
    pub fn invalid_pos(pos: Position) -> Error {
        Error(ErrorKind::InvalidPos(pos))
    }

    pub fn unsupported_font(c: char) -> Error {
        Error(ErrorKind::UnsuportedFont(c))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ErrorKind {
    InvalidPos(Position),
    UnsuportedFont(char),
}
