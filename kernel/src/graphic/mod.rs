pub mod console;
pub mod error;
pub mod font;
pub mod font_gen;
pub mod pixel;

use core::{
    cell::{Ref, RefCell, RefMut},
    mem::MaybeUninit,
    sync::atomic::{AtomicU8, Ordering},
};

pub use console::Console;
pub use font::{FontWriter, StringWriter};
pub use pixel::{Color, Graphic, PixelPosition, PixelWriter, RectWriter};

use self::pixel::{FrameBufferInfo, PixelWriterInner};

static mut CONSOLE: MaybeUninit<RefCell<Console<Graphic<'static, dyn PixelWriterInner>>>> =
    MaybeUninit::uninit();

// 0: uninitalized
// 1: initializing
// 2: initizlized
static IS_INITIALIZED: AtomicU8 = AtomicU8::new(UNINITIALIZED);

const UNINITIALIZED: u8 = 0;
const INITIALIZING: u8 = 1;
const INITIALIZED: u8 = 2;

pub fn init(info: FrameBufferInfo) {
    match IS_INITIALIZED.load(Ordering::SeqCst) {
        UNINITIALIZED => {
            if IS_INITIALIZED
                .compare_exchange(
                    UNINITIALIZED,
                    INITIALIZING,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                )
                .is_ok()
            {
                let graphic = Graphic::new(info);
                let console = RefCell::new(Console::new(graphic));
                unsafe {
                    CONSOLE.write(console);
                }

                match IS_INITIALIZED.compare_exchange(
                    INITIALIZING,
                    INITIALIZED,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                ) {
                    Ok(_) => {}
                    Err(_) => panic!("cannot initialize console"),
                }
            }
        }
        INITIALIZING | INITIALIZED => {}
        _ => unreachable!(),
    }
}

fn _console() -> &'static RefCell<Console<Graphic<'static, dyn PixelWriterInner>>> {
    if IS_INITIALIZED.load(Ordering::SeqCst) == INITIALIZED {
        unsafe { &*CONSOLE.as_mut_ptr() }
    } else {
        panic!("uninitialized console")
    }
}

pub fn console() -> Ref<'static, Console<Graphic<'static, dyn PixelWriterInner>>> {
    _console().borrow()
}

pub fn console_mut() -> RefMut<'static, Console<Graphic<'static, dyn PixelWriterInner>>> {
    {
        _console().borrow_mut()
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::graphic::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    console_mut().write_fmt(args).unwrap();
}
