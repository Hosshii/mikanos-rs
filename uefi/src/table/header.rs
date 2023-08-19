use crate::types::{Uint32, Uint64};

#[repr(C)]
pub struct TableHeader {
    pub signature: Uint64,
    pub revision: Uint32,
    pub header_size: Uint32,
    pub crc32: Uint32,
    pub reserved: Uint32,
}
