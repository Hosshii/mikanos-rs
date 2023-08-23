#![no_std]

pub mod graphic;

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct KernelArg {
    frame_buffer_base: *mut u8,
    frame_buffer_size: usize,
    pixels_per_scan_line: u32,
    horizontal_resolution: u32,
    vertical_resolution: u32,
    pixel_format: PixelFormat,
}

impl KernelArg {
    /// # Safety
    /// frame_buffer_base must be correct address
    pub unsafe fn new(
        frame_buffer_base: *mut u8,
        buffer_size: usize,
        pixels_per_scan_line: u32,
        horizontal_resolution: u32,
        vertical_resolution: u32,
        pixel_format: PixelFormat,
    ) -> Self {
        Self {
            frame_buffer_base,
            frame_buffer_size: buffer_size,
            pixels_per_scan_line,
            horizontal_resolution,
            vertical_resolution,
            pixel_format,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum PixelFormat {
    PixelRGBResv8BitPerColor,
    PixelBGRResv8BitPerColor,
}

pub type KernelMain = extern "sysv64" fn(arg: KernelArg) -> !;
