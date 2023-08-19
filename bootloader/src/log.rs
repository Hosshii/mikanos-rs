// 並列だと安全でない

use core::fmt::{self, Write};
use uefi::{protocol::console::SimpleTextOutputProtocol, table::system_table::SystemTable};

pub static mut LOGGER: &dyn Log = &NoopLogger;
static mut LOGGER_INNER: Option<Logger> = None;

pub trait Log {
    fn log(&self, msg: fmt::Arguments);
}

struct NoopLogger;

impl Log for NoopLogger {
    fn log(&self, _: fmt::Arguments) {}
}

pub struct Logger(*mut SimpleTextOutputProtocol);

impl Logger {
    /// # Safety
    /// boot serviceがexitした後はこれを使ってはいけない
    pub unsafe fn new(out: &mut SimpleTextOutputProtocol) -> Self {
        Self(out as *mut SimpleTextOutputProtocol)
    }
}

impl Log for Logger {
    fn log(&self, msg: fmt::Arguments) {
        let stdout = unsafe { &mut *self.0 };
        writeln!(stdout, "{msg}").unwrap();
    }
}

#[macro_export]
macro_rules! info {
    ($msg:expr $(,)?) => {
        $crate::log::logger().log(::core::format_args!($msg))
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::log::logger().log(::core::format_args!($fmt, $($arg)*))
    };
}

/// # Safety
/// 並列に呼んではいけない
pub unsafe fn init_logger(st: &mut SystemTable) {
    let stdout = st.stdout();
    unsafe {
        LOGGER_INNER = Some(Logger::new(stdout));
        LOGGER = LOGGER_INNER.as_ref().unwrap();
    }
}

pub fn logger() -> &'static dyn Log {
    unsafe { LOGGER }
}
