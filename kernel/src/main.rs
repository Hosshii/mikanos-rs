#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

use kernel::{
    graphic::pixel::FrameBufferInfo,
    graphic::{Color, Graphic, PixelWriter, Position, StringWriter},
    KernelArg,
};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// entry point
#[no_mangle]
pub extern "sysv64" fn kernel_main(arg: KernelArg) -> ! {
    let frame_buffer_info = FrameBufferInfo::from(arg);
    let mut graphic = Graphic::new(frame_buffer_info);

    for x in 0..graphic.info().horizontal_resolution() {
        for y in 0..graphic.info().vertical_resolution() {
            let pos = Position::new(x as i32, y as i32);
            graphic.write_pixel(pos, Color::WHITE);
        }
    }

    for x in 0..200 {
        for y in 0..100 {
            let pos = Position::new(x as i32, y as i32);
            graphic.write_pixel(pos, Color::GREEN);
        }
    }

    let string = r#"`!?#@"'()_\$<>-^&*/~|={};:+[]%qdrfbashtgzxmcjwupvyneoil,.k1234567890"#;
    graphic.write_string(
        Position::new(0, 10),
        string,
        Color::BLACK,
        Some(Color::WHITE),
    );

    halt()
}

#[cfg(target_arch = "x86_64")]
#[inline]
fn halt() -> ! {
    loop {
        unsafe {
            asm! {"hlt"}
        }
    }
}
#[cfg(target_arch = "aarch64")]
#[inline]
fn halt() -> ! {
    loop {
        unsafe {
            asm! {"wfi"}
        }
    }
}
