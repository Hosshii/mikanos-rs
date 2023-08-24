use core::fmt::{self, Write};

use super::{
    error::{Error, Result},
    font::{self, FontWriter, FONT_HEIGHT, FONT_WIDTH},
    pixel::{Color, FrameBufferInfo, PixelFormat},
    PixelPosition, PixelWriter, StringWriter,
};

const ROW_NUM: usize = 38;
const COL_NUM: usize = 80;

pub struct Console<W, const ROW: usize = ROW_NUM, const COL: usize = COL_NUM>
where
    W: FontWriter,
{
    writer: W,
    buffer: [[u8; COL]; ROW],
    font_color: Color,
    font_bg_color: Option<Color>,
    bg_color: Color,
    cursor_pos: FontPosition,
}

impl<W> Console<W, ROW_NUM, COL_NUM>
where
    W: FontWriter,
{
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            buffer: [[0; COL_NUM]; ROW_NUM],
            font_color: Color::BLACK,
            font_bg_color: None,
            bg_color: Color::WHITE,
            cursor_pos: FontPosition::new(0, 0),
        }
    }
}

impl<W, const ROW: usize, const COL: usize> Console<W, ROW, COL>
where
    W: FontWriter,
{
    fn newline(&mut self) -> Result<()> {
        self.cursor_pos.x = 0;
        if self.cursor_pos.y < ROW as u32 - 1 {
            self.cursor_pos.y += 1;
        } else {
            self.clear_screen()?;

            for row in 0..ROW - 1 {
                let (lhs, rhs) = self.buffer.split_at_mut(row + 1);
                lhs[row].copy_from_slice(&rhs[0][..COL]);

                let string = core::str::from_utf8(&lhs[row]).map_err(Error::utf8)?;
                let pos = PixelPosition::new(0, (row * FONT_HEIGHT) as u32);
                self.writer
                    .write_string(pos, string, self.font_color, self.font_bg_color)?;
            }
        }

        Ok(())
    }

    /// カーソルは動かさない
    pub fn clear_screen(&mut self) -> Result<()> {
        for y in 0..FONT_HEIGHT * ROW {
            for x in 0..FONT_WIDTH * COL {
                self.writer
                    .write_pixel(PixelPosition::new(x as u32, y as u32), self.bg_color)?;
            }
        }

        Ok(())
    }

    pub fn clear_cursor(&mut self) {
        self.cursor_pos = FontPosition::new(0, 0);
    }

    pub fn graphic(&mut self) -> &mut W {
        &mut self.writer
    }

    pub const fn row_num(&self) -> usize {
        ROW
    }

    pub const fn col_num(&self) -> usize {
        COL
    }
}

impl<W, const ROW: usize, const COL: usize> Write for Console<W, ROW, COL>
where
    W: FontWriter,
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            if c == '\n' {
                self.newline().map_err(|_| fmt::Error)?;
            } else if self.cursor_pos.x < COL as u32 {
                let pos = PixelPosition::from(self.cursor_pos);
                let font = font::get_font(c).ok_or(fmt::Error)?;
                self.writer
                    .write_font(pos, font, self.font_color, self.font_bg_color)
                    .map_err(|_| fmt::Error)?;
                self.buffer[self.cursor_pos.y as usize][self.cursor_pos.x as usize] = c as u8;
                self.cursor_pos.x += 1;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct FontPosition {
    x: u32,
    y: u32,
}

impl FontPosition {
    fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

impl From<FontPosition> for PixelPosition {
    fn from(value: FontPosition) -> Self {
        PixelPosition::new(value.x * FONT_WIDTH as u32, value.y * FONT_HEIGHT as u32)
    }
}

impl<W, const ROW: usize, const COL: usize> PixelWriter for Console<W, ROW, COL>
where
    W: FontWriter,
{
    fn write_pixel(&mut self, pos: PixelPosition, color: Color) -> Result<()> {
        self.graphic().write_pixel(pos, color)
    }

    unsafe fn write_pixel_unchecked(&mut self, pos: PixelPosition, color: Color) {
        self.graphic().write_pixel_unchecked(pos, color)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Info {
    frame_buffer_size: usize,
    pixels_per_scan_line: u32,
    horizontal_resolution: u32,
    vertical_resolution: u32,
    pixel_format: PixelFormat,
}

impl From<FrameBufferInfo> for Info {
    fn from(value: FrameBufferInfo) -> Self {
        Self {
            frame_buffer_size: value.buffer_size(),
            pixels_per_scan_line: value.pixels_per_scan_line(),
            horizontal_resolution: value.horizontal_resolution(),
            vertical_resolution: value.vertical_resolution(),
            pixel_format: value.pixel_format(),
        }
    }
}
