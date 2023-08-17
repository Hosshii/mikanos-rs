use core::ffi::c_void;

#[repr(usize)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    Success = 0,
}

pub type Handle = *mut c_void;
pub type Uint16 = u16;
pub type Uint32 = u32;
pub type Uint64 = u64;
pub type Uintn = usize;
pub type Int16 = i16;
pub type Int32 = i32;
pub type Int64 = i64;
pub type Event = *mut c_void;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Char16(u16);
impl TryFrom<char> for Char16 {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        if (value as u32) < 0xffff {
            Ok(Char16(value as u16))
        } else {
            Err(())
        }
    }
}

pub const NULL_16: Char16 = Char16(0);

#[repr(transparent)]
pub struct CStr16 {
    innner: [Char16],
}

impl CStr16 {
    pub fn as_ptr(&self) -> *const Char16 {
        self.innner.as_ptr()
    }

    pub fn from_str_with_buf<'a>(input: &str, buf: &'a mut [u16]) -> Result<&'a Self, ()> {
        let mut idx = 0;
        for c in input.encode_utf16() {
            let Some(slot) = buf.get_mut(idx) else {
                return Err(());
            };
            if c == 0 && idx != input.len() {
                return Err(());
            }
            *slot = c;
            idx += 1;
        }
        *buf.get_mut(idx).ok_or(())? = 0;

        Ok(unsafe { &*(buf as *const [u16] as *const CStr16) })
    }
}

#[repr(C)]
pub struct SystemTable {
    pub hdr: TableHeader,
    pub firmware_vender: *const Char16,
    pub firmware_revision: Uint32,
    pub console_handle: Handle,
    pub con_in: *mut SimpleTextInputProtocol,
    pub console_out_handle: Handle,
    pub con_out: *mut SimpleTextOutputProtocol,
    pub standard_error_handle: Handle,
    // pub std_error: *mut SimpleTextOutputProtocol,
    pub std_error: *mut usize,
    // pub runtime_services: *const RuntimeServices,
    pub runtime_services: *const usize,
    // pub boot_services: *const BootServices,
    pub boot_services: *const usize,
    pub number_of_table_entries: Uintn,
    // pub configuration_table: *const ConfigurationTable,
    pub configuration_table: *const usize,
}

#[repr(C)]
pub struct TableHeader {
    pub signature: Uint64,
    pub revision: Uint32,
    pub header_size: Uint32,
    pub crc32: Uint32,
    pub reserved: Uint32,
}

#[repr(C)]
pub struct SimpleTextInputProtocol {
    pub reset: InputReset,
    pub read_key_stroke: InputReadKey,
    pub wait_for_key: Event,
}

pub type InputReset =
    extern "efiapi" fn(this: &mut SimpleTextInputProtocol, extended: bool) -> Status;
pub type InputReadKey =
    extern "efiapi" fn(this: &mut SimpleTextInputProtocol, key: *mut InputKey) -> Status;

#[repr(C)]
pub struct InputKey {
    pub scan_code: Uint16,
    pub unicode_char: Char16,
}

#[repr(C)]
pub struct SimpleTextOutputProtocol {
    pub reset: TextReset,
    pub output_string: TextString,
    pub test_string: TextTestString,
    pub query_mode: TextQueryMode,
    pub set_mode: TextSetMode,
    pub set_attribute: TextSetAttribute,
    pub clear_screen: TextClearScreen,
    pub set_cursor_position: TextSetCursorPosition,
    pub enable_cursor: TextEnableCursor,
    pub mode: *const SimpleTextOutputMode,
}

pub type TextReset =
    extern "efiapi" fn(this: &SimpleTextOutputProtocol, extended_verification: bool) -> Status;
pub type TextString =
    unsafe extern "efiapi" fn(this: &SimpleTextOutputProtocol, string: *const Char16) -> Status;
pub type TextTestString =
    unsafe extern "efiapi" fn(this: &SimpleTextOutputProtocol, string: *const Char16) -> Status;
pub type TextQueryMode = extern "efiapi" fn(
    this: &SimpleTextOutputProtocol,
    mode_number: Uintn,
    columns: &mut Uintn,
    rows: &mut Uintn,
) -> Status;
pub type TextSetMode =
    extern "efiapi" fn(this: &SimpleTextOutputProtocol, mode_number: Uintn) -> Status;
pub type TextSetAttribute =
    extern "efiapi" fn(this: &SimpleTextOutputProtocol, attribute: Uintn) -> Status;
pub type TextClearScreen = extern "efiapi" fn(this: &SimpleTextOutputProtocol) -> Status;
pub type TextSetCursorPosition =
    extern "efiapi" fn(this: &SimpleTextOutputProtocol, column: Uintn, row: Uintn) -> Status;
pub type TextEnableCursor =
    extern "efiapi" fn(this: &SimpleTextOutputProtocol, visible: bool) -> Status;

#[repr(C)]
pub struct SimpleTextOutputMode {
    pub max_mode: Int32,
    pub mode: Int32,
    pub attribute: Int32,
    pub cursor_column: Int32,
    pub cursor_row: Int32,
    pub cursor_visible: bool,
}
