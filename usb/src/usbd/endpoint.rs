#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    Out = 0,
    In = 1,
}

impl From<bool> for Direction {
    fn from(value: bool) -> Self {
        if value {
            Direction::In
        } else {
            Direction::Out
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EndpointID(u8, Direction);
impl EndpointID {
    pub fn new(id: u8, dir: impl Into<Direction>) -> Self {
        Self(id, dir.into())
    }

    pub fn dci(self) -> u8 {
        self.0 * 2 + self.1 as u8
    }
}
pub const HCP_ENDPOINT_ID: EndpointID = EndpointID(0, Direction::In);
