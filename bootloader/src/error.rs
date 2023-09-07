use core::fmt;

use uefi::types::Status;

use log::info;

pub type Result<T, E = Error> = core::result::Result<T, E>;

pub trait ToRusult {
    type Ok;
    type Err;
    fn to_result(self) -> Result<Self::Ok, Self::Err>;
}

impl ToRusult for Status {
    type Ok = ();

    type Err = Error;

    fn to_result(self) -> Result<Self::Ok, Self::Err> {
        if self.is_success() {
            Ok(())
        } else {
            info!("status {} {}, {}", self, self.is_err(), self.is_success());
            Err(Error::Uefi(self))
        }
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    Uefi(Status),
    ElfParse(&'static str),
    StdFmt(fmt::Error),
    Custom(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Uefi(status) => write!(f, "{:?}", status),
            Error::ElfParse(s) => write!(f, "{}", s),
            Error::StdFmt(e) => write!(f, "{}", e),
            Error::Custom(e) => write!(f, "{e}"),
        }
    }
}

impl From<fmt::Error> for Error {
    fn from(value: fmt::Error) -> Self {
        Self::StdFmt(value)
    }
}

#[macro_export]
macro_rules! custom {
    ($msg:expr $(,)?) => {
        $crate::error::Error::Custom(::core::format_args!($msg))
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::Error::Custom(::core::format_args!($fmt, $($arg)*))
    };
}
