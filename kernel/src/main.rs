#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

use kernel::{
    graphic::{pixel::FrameBufferInfo, Color, PixelPosition, PixelWriter, StringWriter},
    println, KernelArg,
};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// entry point
#[no_mangle]
pub extern "sysv64" fn kernel_main(arg: KernelArg) -> ! {
    let frame_buffer_info = FrameBufferInfo::from(arg);
    kernel::init(frame_buffer_info);

    let graphic = kernel::console().graphic();

    for x in 0..graphic.info().horizontal_resolution() {
        for y in 0..graphic.info().vertical_resolution() {
            let pos = PixelPosition::new(x, y);
            graphic.write_pixel(pos, Color::WHITE).unwrap();
        }
    }

    for x in 0..200 {
        for y in 0..100 {
            let pos = PixelPosition::new(x, y);
            graphic.write_pixel(pos, Color::GREEN).unwrap();
        }
    }

    let string = r#"`!?#@"'()_\$<>-^&*/~|={};:+[]%qdrfbashtgzxmcjwupvyneoil,.k1234567890"#;
    graphic
        .write_string(
            PixelPosition::new(0, 10),
            string,
            Color::BLACK,
            Some(Color::WHITE),
        )
        .unwrap();

    println!(r##"!?#@"'()_\$<>-^&*/~|={{}};:+[]%"##);
    println!("qdrfbashtgzxmcjwupvyneoil,.k");
    println!("1234567890");
    println!("hello {}", "world");

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
