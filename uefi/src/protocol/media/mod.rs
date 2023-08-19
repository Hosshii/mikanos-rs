use crate::types::{Char16, Guid, Status, Time, Uint64, Uintn, UnusedPtr, Void};

#[allow(clippy::unusual_byte_groupings)]
pub const SIMPLE_FILE_SYSTEM_PROTOCOL_GUID: Guid = Guid::new(
    0x0964E5B22,
    0x6459,
    0x11D2,
    [0x8E, 0x39, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B],
);

pub type SimpleFileSystemOpenVoleme =
    extern "efiapi" fn(this: &SimpleFileSystemProtocol, root: *mut *mut FileProtocol) -> Status;

#[repr(C)]
pub struct SimpleFileSystemProtocol {
    pub revision: Uint64,
    pub open_volume: SimpleFileSystemOpenVoleme,
}

pub type FileOpen = extern "efiapi" fn(
    this: &FileProtocol,
    new_handle: *mut *mut FileProtocol,
    file_name: *const Char16,
    open_mode: Uint64,
    attributes: Uint64,
) -> Status;

pub const FILE_MODE_READ: Uint64 = 0x0000000000000001;
pub const FILE_MODE_WRITE: Uint64 = 0x0000000000000002;
pub const FILE_MODE_CREATE: Uint64 = 0x8000000000000000;

pub const EFI_FILE_READ_ONLY: Uint64 = 0x0000000000000001;
pub const EFI_FILE_HIDDEN: Uint64 = 0x0000000000000002;
pub const EFI_FILE_SYSTEM: Uint64 = 0x0000000000000004;
pub const EFI_FILE_RESERVED: Uint64 = 0x0000000000000008;
pub const EFI_FILE_DIRECTORY: Uint64 = 0x0000000000000010;
pub const EFI_FILE_ARCHIVE: Uint64 = 0x0000000000000020;
pub const EFI_FILE_VALID_ATTR: Uint64 = 0x0000000000000037;

pub type FileRead =
    extern "efiapi" fn(this: &FileProtocol, buffer_size: &mut Uintn, buffer: *mut Void) -> Status;

pub type FileGetInfo = extern "efiapi" fn(
    this: &FileProtocol,
    information_type: &Guid,
    buffer_size: &mut Uintn,
    buffer: *mut Void,
) -> Status;

#[repr(C)]
pub struct FileProtocol {
    pub revision: Uint64,
    pub open: FileOpen,
    pub close: UnusedPtr,
    pub delete: UnusedPtr,
    pub read: FileRead,
    pub write: UnusedPtr,
    pub get_position: UnusedPtr,
    pub set_position: UnusedPtr,
    pub get_info: FileGetInfo,
    pub set_info: UnusedPtr,
    pub flush: UnusedPtr,
    pub open_ex: UnusedPtr,
    pub read_ex: UnusedPtr,
    pub write_ex: UnusedPtr,
    pub flush_ex: UnusedPtr,
}

#[allow(clippy::unusual_byte_groupings)]
pub const FILE_INFO_GUID: Guid = Guid::new(
    0x09576E92,
    0x6D3F,
    0x11D2,
    [0x8E, 0x39, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B],
);
#[repr(C)]
pub struct FileInfo {
    pub size: Uint64,
    pub file_size: Uint64,
    pub physical_size: Uint64,
    pub create_time: Time,
    pub last_access_time: Time,
    pub modifimation_time: Time,
    pub attribute: Uint64,
    // dst を raw pointer から作るのはめんどくさいのでここでは省略する
    // uefi-rs ではここもちゃんとやっている
    // pub file_name: [Char16],
}
