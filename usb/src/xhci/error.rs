pub type Result<T> = core::result::Result<T, Error>;
pub struct Error(ErrorKind);

impl Error {
    pub fn lack_of_max_slots() -> Self {
        Self(ErrorKind::LackOfMaxSlots)
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    LackOfMaxSlots,
}
