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

    pub fn device_mnager_out_of_range() -> Self {
        Self(ErrorKind::DeviceManagerOutOfRange)
    }

    pub fn already_port_processing() -> Self {
        Self(ErrorKind::AlreadyPortProcessing)
    }

    pub fn empty_processing_port() -> Self {
        Self(ErrorKind::EmptyProcessingPort)
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    LackOfDeviceContext,
    PortNotNewlyConnected,
    PortDisabled,
    PortResetNotFinished,
    DeviceManagerOutOfRange,
    AlreadyPortProcessing,
    EmptyProcessingPort,
}
