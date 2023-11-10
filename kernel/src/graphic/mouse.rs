use crate::graphic::PixelPosition;

use super::{error::Result, Color, PixelWriter};

const MOUSE_CURSOR_HEIGHT: usize = 24;
const MOUSE_CURSOR_WIDTH: usize = 15;
const MOUSE_CURSOR_SHAPE: [&str; MOUSE_CURSOR_HEIGHT] = [
    "@              ",
    "@@             ",
    "@.@            ",
    "@..@           ",
    "@...@          ",
    "@....@         ",
    "@.....@        ",
    "@......@       ",
    "@.......@      ",
    "@........@     ",
    "@.........@    ",
    "@..........@   ",
    "@...........@  ",
    "@............@ ",
    "@......@@@@@@@@",
    "@......@       ",
    "@....@@.@      ",
    "@...@ @.@      ",
    "@..@   @.@     ",
    "@.@    @.@     ",
    "@@      @.@    ",
    "@       @.@    ",
    "         @.@   ",
    "         @@@   ",
];
const fn assrt() {
    if MOUSE_CURSOR_SHAPE[0].len() != MOUSE_CURSOR_WIDTH {
        panic!()
    }
}
const _: () = assrt();

pub struct MouseCursor {
    pos: PixelPosition,
}

impl MouseCursor {
    pub fn new() -> Self {
        Self {
            pos: PixelPosition::new(0, 0),
        }
    }

    pub fn write<W>(&self, mut w: W) -> Result<()>
    where
        W: PixelWriter,
    {
        for (y, row) in MOUSE_CURSOR_SHAPE.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                let pos = PixelPosition::new(x as u32, y as u32);
                let pos = self.pos + pos;
                if c == '@' {
                    w.write_pixel(pos, Color::WHITE)?;
                } else if c == '.' {
                    w.write_pixel(pos, Color::BLACK)?;
                }
            }
        }

        Ok(())
    }

    pub fn erase<W>(&self, mut w: W) -> Result<()>
    where
        W: PixelWriter,
    {
        for y in 0..MOUSE_CURSOR_HEIGHT {
            for x in 0..MOUSE_CURSOR_WIDTH {
                let pos = PixelPosition::new(x as u32, y as u32);
                let pos = self.pos + pos;
                w.write_pixel(pos, Color::WHITE)?;
            }
        }

        Ok(())
    }

    pub fn move_relative(&mut self, x: i8, y: i8) {
        self.pos.move_relative(x as i32, y as i32);
    }
}

impl Default for MouseCursor {
    fn default() -> Self {
        Self::new()
    }
}
