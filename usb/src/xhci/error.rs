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
}

#[derive(Debug)]
pub enum ErrorKind {
    LackOfDeviceContext,
    PortNotNewlyConnected,
}
