use crate::{
    table::{boot_services::MemoryType, system_table::SystemTable},
    types::{Guid, Handle, Uint32, Uint64, UnusedPtr, Void},
};

#[allow(clippy::unusual_byte_groupings)]
pub const LOADED_IMAGE_PROTOCOL_GUID: Guid = Guid::new(
    0x5B1B31A1,
    0x9562,
    0x11D2,
    [0x8E, 0x3F, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B],
);

pub type ImageUnload = UnusedPtr;

#[repr(C)]
pub struct LoadedImageProtocol {
    pub revision: Uint32,
    pub parent_handle: Handle,
    pub system_table: *const SystemTable,

    pub device_handle: Handle,
    pub file_path: UnusedPtr,
    pub reserved: *const Void,

    pub load_oitions_size: Uint32,
    pub load_oitions: *mut Void,

    pub image_base: *mut Void,
    pub image_size: Uint64,
    pub image_code_type: MemoryType,
    pub image_data_type: MemoryType,
    pub image_unload: ImageUnload,
}
