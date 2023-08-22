use crate::KernelArg;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PixelColor {
    r: u8,
    g: u8,
    b: u8,
}

impl PixelColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const WHITE: Self = Self::new(255, 255, 255);
    pub const GREEN: Self = Self::new(0, 255, 0);
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FrameBufferInfo {
    frame_buffer_base: *mut u8,
    frame_buffer_size: usize,
    pixels_per_scan_line: u32,
    horizontal_resolution: u32,
    vertical_resolution: u32,
    pixel_format: PixelFormat,
}

impl FrameBufferInfo {
    pub fn new(
        frame_buffer_base: *mut u8,
        frame_buffer_size: usize,
        pixels_per_scan_line: u32,
        horizontal_resolution: u32,
        vertical_resolution: u32,
        pixel_format: PixelFormat,
    ) -> Self {
        Self {
            frame_buffer_base,
            frame_buffer_size,
            pixels_per_scan_line,
            horizontal_resolution,
            vertical_resolution,
            pixel_format,
        }
    }

    /// 何ピクセル目かどうか
    /// アドレスではない
    fn pixel_at(&self, pos: Position) -> usize {
        (self.pixels_per_scan_line as i32 * pos.y + pos.x) as usize
    }

    fn is_valid_pos(&self, pos: Position) -> bool {
        self.pixel_at(pos) < self.buffer_size()
    }

    fn buffer_size(&self) -> usize {
        self.frame_buffer_size
    }

    pub fn pixels_per_scan_line(&self) -> u32 {
        self.pixels_per_scan_line
    }

    pub fn horizontal_resolution(&self) -> u32 {
        self.horizontal_resolution
    }

    pub fn vertical_resolution(&self) -> u32 {
        self.vertical_resolution
    }
}

impl From<KernelArg> for FrameBufferInfo {
    fn from(value: KernelArg) -> Self {
        FrameBufferInfo {
            frame_buffer_base: value.frame_buffer_base,
            frame_buffer_size: value.frame_buffer_size,
            pixels_per_scan_line: value.pixels_per_scan_line,
            horizontal_resolution: value.horizontal_resolution,
            vertical_resolution: value.vertical_resolution,
            pixel_format: value.pixel_format.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PixelFormat {
    PixelRGBResv8BitPerColor,
    PixelBGRResv8BitPerColor,
}

impl From<crate::PixelFormat> for PixelFormat {
    fn from(value: crate::PixelFormat) -> Self {
        match value {
            crate::PixelFormat::PixelRGBResv8BitPerColor => PixelFormat::PixelRGBResv8BitPerColor,
            crate::PixelFormat::PixelBGRResv8BitPerColor => PixelFormat::PixelBGRResv8BitPerColor,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    x: i32,
    y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Error(ErrorKind);

impl Error {
    pub fn invalid_pos(pos: Position) -> Error {
        Error(ErrorKind::InvalidPos(pos))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ErrorKind {
    InvalidPos(Position),
}

trait PixelWriterInner {
    unsafe fn write_pixel(&self, ptr: *mut u8, offset: usize, color: PixelColor);
}

struct RGBWriter;
impl PixelWriterInner for RGBWriter {
    unsafe fn write_pixel(&self, ptr: *mut u8, offset: usize, color: PixelColor) {
        let base = ptr.add(offset);
        unsafe {
            base.write_volatile(color.r);
            base.add(1).write_volatile(color.g);
            base.add(2).write_volatile(color.b);
        }
    }
}

struct BGRWriter;
impl PixelWriterInner for BGRWriter {
    unsafe fn write_pixel(&self, ptr: *mut u8, offset: usize, color: PixelColor) {
        let base = ptr.add(offset);
        unsafe {
            base.write_volatile(color.b);
            base.add(1).write_volatile(color.g);
            base.add(2).write_volatile(color.r);
        }
    }
}

pub struct Graphic {
    info: FrameBufferInfo,
    writer: &'static dyn PixelWriterInner,
}

impl Graphic {
    pub fn new(info: FrameBufferInfo) -> Self {
        let writer: &dyn PixelWriterInner = match info.pixel_format {
            PixelFormat::PixelRGBResv8BitPerColor => &RGBWriter,
            PixelFormat::PixelBGRResv8BitPerColor => &BGRWriter,
        };
        Self { info, writer }
    }

    pub fn write_pixel(&mut self, pos: Position, color: PixelColor) -> Result<()> {
        if !self.info.is_valid_pos(pos) {
            return Err(Error::invalid_pos(pos));
        }
        let offset = self.info.pixel_at(pos) * 4;

        unsafe {
            self.writer
                .write_pixel(self.info.frame_buffer_base, offset, color)
        };
        Ok(())
    }

    pub fn info(&self) -> &FrameBufferInfo {
        &self.info
    }
}
