// 並列だと安全でない

use core::fmt::Write;
use log::{Log, Payload};
use uefi::{protocol::console::SimpleTextOutputProtocol, table::system_table::SystemTable};

static mut LOGGER_INNER: Option<Logger> = None;

pub struct Logger(*mut SimpleTextOutputProtocol);

impl Logger {
    /// # Safety
    /// boot serviceがexitした後はこれを使ってはいけない
    pub unsafe fn new(out: &mut SimpleTextOutputProtocol) -> Self {
        Self(out as *mut SimpleTextOutputProtocol)
    }
}

impl Log for Logger {
    fn log(&self, payload: &Payload) {
        let stdout = unsafe { &mut *self.0 };

        writeln!(stdout, "{}: {}", payload.level(), payload.msg()).unwrap();
    }
}

/// # Safety
/// 並列に呼んではいけない
pub unsafe fn init_logger(st: &mut SystemTable) {
    let stdout = st.stdout();
    unsafe {
        LOGGER_INNER = Some(Logger::new(stdout));
        log::set_logger(LOGGER_INNER.as_ref().unwrap()).unwrap();
    }
}
