use core::fmt;

use crate::types::{Bool, CStr16, Char16, Event, Int32, Status, Uint16, Uintn};

#[repr(C)]
pub struct SimpleTextInputProtocol {
    pub reset: InputReset,
    pub read_key_stroke: InputReadKey,
    pub wait_for_key: Event,
}

pub type InputReset =
    extern "efiapi" fn(this: &mut SimpleTextInputProtocol, extended: Bool) -> Status;
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

impl fmt::Write for SimpleTextOutputProtocol {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        fn flush(out: &SimpleTextOutputProtocol, buf: &[u16]) -> fmt::Result {
            let cstr16 = unsafe { CStr16::from_u16_unchecked(buf) };
            let status = unsafe { (out.output_string)(out, cstr16.as_ptr()) };
            if status.is_err() {
                Err(fmt::Error)
            } else {
                Ok(())
            }
        }

        fn write_to_buf(
            out: &SimpleTextOutputProtocol,
            c: u16,
            idx: &mut usize,
            buf: &mut [u16],
        ) -> fmt::Result {
            if *idx < buf.len() - 1 {
                buf[*idx] = c;
                *idx += 1;
            } else {
                buf[*idx] = 0;
                *idx = 0;
                flush(out, buf)?;
                write_to_buf(out, c, idx, buf)?;
            }
            Ok(())
        }

        const BUF_SIZE: usize = 2;
        let mut buf = [0u16; BUF_SIZE];
        let mut idx = 0;

        for c in s.encode_utf16() {
            if c == b'\n' as u16 {
                write_to_buf(self, b'\r' as u16, &mut idx, &mut buf)?;
            }
            write_to_buf(self, c, &mut idx, &mut buf)?;
        }
        write_to_buf(self, 0, &mut idx, &mut buf)?;
        flush(self, &buf)?;

        Ok(())
    }
}

pub type TextReset =
    extern "efiapi" fn(this: &SimpleTextOutputProtocol, extended_verification: Bool) -> Status;
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
    extern "efiapi" fn(this: &SimpleTextOutputProtocol, visible: Bool) -> Status;

#[repr(C)]
pub struct SimpleTextOutputMode {
    pub max_mode: Int32,
    pub mode: Int32,
    pub attribute: Int32,
    pub cursor_column: Int32,
    pub cursor_row: Int32,
    pub cursor_visible: Bool,
}
