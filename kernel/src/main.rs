#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

use kernel::{
    graphic::{
        pixel::FrameBufferInfo, Color, PixelPosition, PixelWriter, RectWriter, StringWriter,
    },
    println, KernelArg,
};

const MOUSE_CURSOR_HEIGHT: usize = 24;
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

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut console = kernel::console_mut();
    console.clear_cursor();
    console.clear_screen();
    println!("panic {:?}", info);
    loop {}
}

// entry point
#[no_mangle]
pub extern "sysv64" fn kernel_main(arg: KernelArg) -> ! {
    let frame_buffer_info = FrameBufferInfo::from(arg);
    kernel::init(frame_buffer_info);

    let mut console = kernel::console_mut();

    for x in 0..console.graphic().info().horizontal_resolution() {
        for y in 0..console.graphic().info().vertical_resolution() {
            let pos = PixelPosition::new(x, y);
            console.write_pixel(pos, Color::WHITE).unwrap();
        }
    }

    for x in 0..200 {
        for y in 0..100 {
            let pos = PixelPosition::new(x, y);
            console.write_pixel(pos, Color::GREEN).unwrap();
        }
    }

    let string = r#"`!?#@"'()_\$<>-^&*/~|={};:+[]%qdrfbashtgzxmcjwupvyneoil,.k1234567890"#;
    console
        .write_string(
            PixelPosition::new(0, 10),
            string,
            Color::BLACK,
            Some(Color::WHITE),
        )
        .unwrap();

    drop(console);

    println!(r##"!?#@"'()_\$<>-^&*/~|={{}};:+[]%"##);
    println!("qdrfbashtgzxmcjwupvyneoil,.k");
    println!("1234567890");
    println!("hello {}", "world");

    for i in 0..35 {
        let num = kernel::console().row_num();
        println!("line: {i}, {num}");
    }

    for (y, row) in MOUSE_CURSOR_SHAPE.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            let pos = PixelPosition::new(200 + x as u32, 100 + y as u32);
            if c == '@' {
                kernel::console_mut()
                    .write_pixel(pos, Color::WHITE)
                    .unwrap();
            } else if c == '.' {
                kernel::console_mut()
                    .write_pixel(pos, Color::BLACK)
                    .unwrap();
            }
        }
    }

    let per_line = kernel::console_mut()
        .graphic()
        .info()
        .pixels_per_scan_line();

    kernel::console_mut()
        .fill_rect(
            PixelPosition::new(0, 0),
            PixelPosition::new(per_line, 50),
            Color::new(45, 118, 237),
        )
        .unwrap();

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
