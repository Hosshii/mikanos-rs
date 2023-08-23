use crate::graphic::{
    font_gen,
    pixel::{Color, PixelWriter, Position},
};

pub use font_gen::get_font;

use super::error::{Error, Result};

pub type Font = [u8; 16];

pub trait FontWriter: PixelWriter {
    fn write_font(
        &mut self,
        pos: Position,
        font: &Font,
        fg_color: Color,
        bg_color: Option<Color>,
    ) -> Result<()> {
        for (dy, row) in font.iter().enumerate() {
            for dx in 0..8 {
                let pos = pos + Position::new(dx, dy as i32);
                if ((row << dx) & 0x80) != 0 {
                    self.write_pixel(pos, fg_color)?;
                } else if let Some(color) = bg_color {
                    self.write_pixel(pos, color)?;
                }
            }
        }
        Ok(())
    }

    /// # Safety
    /// x and y must be valid
    unsafe fn write_font_unchecked(
        &mut self,
        pos: Position,
        font: &Font,
        fg_color: Color,
        bg_color: Option<Color>,
    ) {
        for (dy, row) in font.iter().enumerate() {
            for dx in 0..8 {
                let pos = pos + Position::new(dx, dy as i32);
                if ((row << dx) & 0x80) != 0 {
                    self.write_pixel_unchecked(pos, fg_color);
                } else if let Some(color) = bg_color {
                    self.write_pixel_unchecked(pos, color);
                }
            }
        }
    }
}

impl<T> FontWriter for T where T: PixelWriter {}

pub trait StringWriter: FontWriter {
    fn write_string(
        &mut self,
        pos: Position,
        string: &str,
        fg_color: Color,
        bg_color: Option<Color>,
    ) -> Result<()> {
        for (idx, c) in string.chars().enumerate() {
            let font = font_gen::get_font(c).ok_or(Error::unsupported_font(c))?;
            let pos = pos + Position::new(idx as i32 * 8, 0);
            self.write_font(pos, &font, fg_color, bg_color)?;
        }
        Ok(())
    }

    /// # Safety
    /// `pos` must be valid.
    /// `string` must be supported character.
    unsafe fn write_string_unchecked(
        &mut self,
        pos: Position,
        string: &str,
        fg_color: Color,
        bg_color: Option<Color>,
    ) {
        for (idx, c) in string.chars().enumerate() {
            let font = font_gen::get_font(c).unwrap_unchecked();
            let pos = pos + Position::new(idx as i32 * 8, 0);
            self.write_font_unchecked(pos, &font, fg_color, bg_color);
        }
    }
}

impl<T> StringWriter for T where T: FontWriter {}
