#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

use kernel::{
    console::{FrameBufferInfo, Graphic, PixelColor, Position},
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
            graphic.write_pixel(pos, PixelColor::WHITE);
        }
    }

    for x in 0..200 {
        for y in 0..100 {
            let pos = Position::new(x as i32, y as i32);
            graphic.write_pixel(pos, PixelColor::GREEN);
        }
    }

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
