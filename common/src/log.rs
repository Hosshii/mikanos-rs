use core::{
    fmt,
    sync::atomic::{AtomicU8, Ordering},
};

pub type Result<T> = core::result::Result<T, Error>;

static mut LOGGER: &dyn Log = &NoopLogger;

const UNINITIALIZED: u8 = 0;
const INITIALIZING: u8 = 1;
const INITIALIZED: u8 = 2;

static STATUS: AtomicU8 = AtomicU8::new(UNINITIALIZED);

/// これ以上のレベルだと出力される
static LOG_LEVEL_THRESHOLD: AtomicU8 = AtomicU8::new(LogLevel::Debug as u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LogLevel {
    Debug,
    Info,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            LogLevel::Error => "error",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
        };
        write!(f, "{s}")
    }
}

impl TryFrom<u8> for LogLevel {
    type Error = ();

    fn try_from(value: u8) -> core::result::Result<Self, <LogLevel as TryFrom<u8>>::Error> {
        match value {
            x if x == LogLevel::Error.into() => Ok(LogLevel::Error),
            x if x == LogLevel::Debug.into() => Ok(LogLevel::Debug),
            x if x == LogLevel::Info.into() => Ok(LogLevel::Info),
            _ => Err(()),
        }
    }
}

impl From<LogLevel> for u8 {
    fn from(value: LogLevel) -> Self {
        value as u8
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Payload<'a> {
    level: LogLevel,
    msg: fmt::Arguments<'a>,
}

impl<'a> Payload<'a> {
    pub fn new(level: LogLevel, msg: fmt::Arguments<'a>) -> Self {
        Self { level, msg }
    }

    pub fn level(&self) -> &LogLevel {
        &self.level
    }

    pub fn msg(&self) -> fmt::Arguments<'_> {
        self.msg
    }
}

pub trait Log {
    fn log(&self, payload: &Payload);
}

struct NoopLogger;

impl Log for NoopLogger {
    fn log(&self, _: &Payload) {}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Error(ErrorKind);
impl Error {
    fn initialize_error() -> Self {
        Self(ErrorKind::InitializeError)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorKind {
    InitializeError,
}

pub fn set_logger(logger: &'static dyn Log) -> Result<()> {
    if STATUS
        .compare_exchange(
            UNINITIALIZED,
            INITIALIZING,
            Ordering::SeqCst,
            Ordering::SeqCst,
        )
        .is_err()
    {
        return Err(Error::initialize_error());
    }

    unsafe {
        LOGGER = logger;
    }

    if STATUS
        .compare_exchange(
            INITIALIZING,
            INITIALIZED,
            Ordering::SeqCst,
            Ordering::SeqCst,
        )
        .is_err()
    {
        return Err(Error::initialize_error());
    }

    Ok(())
}

pub fn logger() -> &'static dyn Log {
    unsafe { LOGGER }
}

pub fn log_level_threshold() -> LogLevel {
    LogLevel::try_from(LOG_LEVEL_THRESHOLD.load(Ordering::SeqCst)).unwrap()
}

pub fn set_log_level_threshold(level: LogLevel) {
    LOG_LEVEL_THRESHOLD.store(level.into(), Ordering::SeqCst);
}

#[macro_export]
macro_rules! error {
    ($msg:expr $(,)?) => {
        if $crate::log::log_level_threshold() <= $crate::log::LogLevel::Error {
            $crate::log::logger().log(&$crate::log::Payload::new($crate::log::LogLevel::Error, ::core::format_args!($msg)))
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        if $crate::log::log_level_threshold() <= $crate::log::LogLevel::Error {
            $crate::log::logger().log(&$crate::log::Payload::new($crate::log::LogLevel::Error, ::core::format_args!($fmt, $($arg)*)))
        }
    };
}

#[macro_export]
macro_rules! info {
    ($msg:expr $(,)?) => {
        if $crate::log::log_level_threshold() <= $crate::log::LogLevel::Info {
            $crate::log::logger().log(&$crate::log::Payload::new($crate::log::LogLevel::Info, ::core::format_args!($msg)))
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        if $crate::log::log_level_threshold() <= $crate::log::LogLevel::Info {
            $crate::log::logger().log(&$crate::log::Payload::new($crate::log::LogLevel::Info, ::core::format_args!($fmt, $($arg)*)))
        }
    };
}

#[macro_export]
macro_rules! debug {
    ($msg:expr $(,)?) => {
        if $crate::log::log_level_threshold() <= $crate::log::LogLevel::Debug {
            $crate::log::logger().log(&$crate::log::Payload::new(
                $crate::log::LogLevel::Debug,
                ::core::format_args!($msg),
            ))
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        if $crate::log::log_level_threshold() <= $crate::log::LogLevel::Debug {
            $crate::log::logger().log(&$crate::log::Payload::new(
                $crate::log::LogLevel::Debug,
                ::core::format_args!($fmt, $($arg)*),
            ))
        }
    };
}
