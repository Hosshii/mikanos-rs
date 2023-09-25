pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(ErrorKind);

impl Error {
    pub fn lack_of_device_contexts() -> Self {
        Self(ErrorKind::LackOfDeviceContext)
    }

    pub fn port_not_newly_connected() -> Self {
        Self(ErrorKind::PortNotNewlyConnected)
    }

    pub fn port_disabled() -> Self {
        Self(ErrorKind::PortDisabled)
    }

    pub fn port_reset_not_finished() -> Self {
        Self(ErrorKind::PortResetNotFinished)
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    LackOfDeviceContext,
    PortNotNewlyConnected,
    PortDisabled,
    PortResetNotFinished,
}
