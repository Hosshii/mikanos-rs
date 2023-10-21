#![no_std]
#![no_main]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::{arch::asm, panic::PanicInfo, pin::pin};

use common::{debug, info, Zeroed as _};
use kernel::{
    error::Error as LibError,
    graphic::{
        error::Error as GraphicError, pixel::FrameBufferInfo, Color, PixelPosition, PixelWriter,
        RectWriter, StringWriter,
    },
    logger,
    pci::{Device, Pci, PciExtUsb as _},
    println, KernelArg,
};
use usb::xhci::{
    driver::{Context, Controller},
    error::Error as UsbError,
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
    {
        let mut console = kernel::console_mut();
        console.clear_cursor();
        console.clear_screen().unwrap();
    }

    println!("panic {:?}", info);
    loop {}
}

// entry point
#[no_mangle]
pub extern "sysv64" fn kernel_main(arg: KernelArg) -> ! {
    logger::init_logger();

    if let Err(e) = kernel_main_impl(arg) {
        println!("{:?}", e)
    }

    halt();
}

fn kernel_main_impl(arg: KernelArg) -> Result<()> {
    let frame_buffer_info = FrameBufferInfo::from(arg);
    kernel::init(frame_buffer_info);

    let mut console = kernel::console_mut();

    for x in 0..console.graphic().info().horizontal_resolution() {
        for y in 0..console.graphic().info().vertical_resolution() {
            let pos = PixelPosition::new(x, y);
            console.write_pixel(pos, Color::WHITE)?;
        }
    }

    for x in 0..200 {
        for y in 0..100 {
            let pos = PixelPosition::new(x, y);
            console.write_pixel(pos, Color::GREEN)?;
        }
    }

    let string = r#"`!?#@"'()_\$<>-^&*/~|={};:+[]%qdrfbashtgzxmcjwupvyneoil,.k1234567890"#;
    console.write_string(
        PixelPosition::new(0, 10),
        string,
        Color::BLACK,
        Some(Color::WHITE),
    )?;

    drop(console);

    println!(r##"!?#@"'()_\$<>-^&*/~|={{}};:+[]%"##);
    println!("qdrfbashtgzxmcjwupvyneoil,.k");
    println!("1234567890");
    println!("hello {}", "world");

    for (y, row) in MOUSE_CURSOR_SHAPE.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            let pos = PixelPosition::new(200 + x as u32, 100 + y as u32);
            if c == '@' {
                kernel::console_mut().write_pixel(pos, Color::WHITE)?;
            } else if c == '.' {
                kernel::console_mut().write_pixel(pos, Color::BLACK)?;
            }
        }
    }

    let per_line = kernel::console_mut()
        .graphic()
        .info()
        .pixels_per_scan_line();

    kernel::console_mut().fill_rect(
        PixelPosition::new(0, 0),
        PixelPosition::new(per_line, 50),
        Color::new(45, 118, 237),
    )?;

    let mut pci = Pci::new();

    pci.scan_all_bus()?;
    info!("scan all bus");

    // for dev in pci.devices() {
    //     debug!("{:?}", dev.class_code());
    // }

    let usb = pci
        .find_usb()
        .ok_or(Error::custom("cannot find usb device"))?;
    let bar = read_xhci_bar(usb)?;
    info!("bar: {:p}", bar as *const u8);

    if usb.read_vender_id()?.is_intel() {
        pci.switch_ehci2xhci(usb)?;
    }

    let cx = Context::zeroed();
    let cx = pin!(cx);
    let xhci: Controller<_> = unsafe { Controller::new(bar, cx) };

    info!("initialize usb...");
    let xhci = xhci.initialize()?;
    info!("initialize finished!");
    info!("run xhci");
    let mut xhci = xhci.run();

    for (idx, mut port) in xhci.ports_mut().enumerate() {
        // debug!("port {}. is connected = {}", idx, port.is_connected());
        if port.is_connected() {
            port.configure()?;
        }
    }

    loop {
        xhci.process_primary_event()?;
    }
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

#[derive(Debug)]
struct Error(ErrorKind);

impl Error {
    fn custom(v: &'static str) -> Self {
        Self(ErrorKind::Custom(v))
    }
}

impl From<LibError> for Error {
    fn from(value: LibError) -> Self {
        Self(ErrorKind::Lib(value))
    }
}

impl From<GraphicError> for Error {
    fn from(value: GraphicError) -> Self {
        Self(ErrorKind::Graphic(value))
    }
}

impl From<UsbError> for Error {
    fn from(value: UsbError) -> Self {
        Self(ErrorKind::Usb(value))
    }
}

#[derive(Debug)]
enum ErrorKind {
    Lib(LibError),
    Graphic(GraphicError),
    Usb(UsbError),
    Custom(&'static str),
}

type Result<T> = core::result::Result<T, Error>;

fn read_xhci_bar(dev: &Device) -> Result<u64> {
    let bar0 = dev.read_bar(0)? as u64;
    debug!("bar0: {}", bar0);
    let bar1 = dev.read_bar(1)? as u64;
    debug!("bar1: {}", bar1);

    Ok((bar1 << 32) | (bar0 & !0xf))
}
