use common::debug;

use crate::xhci::register_map::PortRegisterSet;

use super::error::{Error, Result};

#[derive(Debug)]
pub struct Disabled;

#[derive(Debug)]
pub struct Enabled;

#[derive(Debug)]
pub struct PortWrapper<'a, 'b> {
    set: &'a mut PortRegisterSet<'b>,
    port_num: u8,
}

impl<'a, 'b> PortWrapper<'a, 'b> {
    pub fn new(set: &'a mut PortRegisterSet<'b>, port_num: u8) -> Self {
        Self { set, port_num }
    }

    pub fn is_connected(&self) -> bool {
        self.set
            .port_status_and_control()
            .read()
            .get_data_current_connect_status()
    }

    pub fn is_connecded_status_changed(&self) -> bool {
        self.set
            .port_status_and_control()
            .read()
            .get_data_connect_status_change()
    }

    fn reset(&mut self) -> Result<()> {
        debug!("reset port");
        if !self.is_connected() || !self.is_connecded_status_changed() {
            return Err(Error::port_not_newly_connected());
        }

        let portsc_reg = self.set.port_status_and_control_mut();
        let mut portsc = portsc_reg.read().clear_rw1s();

        portsc.set_data_port_reset(true);

        portsc.set_data_connect_status_change(true);
        portsc_reg.write(portsc);

        while portsc_reg.read().get_data_port_reset() {}

        Ok(())
    }

    pub fn configure(&mut self) -> Result<()> {
        debug!("configure port");
        self.reset()?;
        Ok(())
    }
}
