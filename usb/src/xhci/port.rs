use core::marker::PhantomData;

use crate::xhc::interface::register_map::PortRegisterSet;

use super::error::{Error, Result};

pub enum PortWrapper<'a, 'b> {
    Disabled(Port<'a, 'b, Disabled>),
    Enabled(Port<'a, 'b, Enabled>),
}

pub struct Disabled;
pub struct Enabled;

pub struct Port<'a, 'b, STATUS> {
    set: &'a mut PortRegisterSet<'b>,
    _phantom_data: PhantomData<STATUS>,
}

impl<'a, 'b, STATUS> Port<'a, 'b, STATUS> {
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
}

impl<'a, 'b> Port<'a, 'b, Disabled> {
    pub fn reset(self) -> Result<Port<'a, 'b, Enabled>> {
        if !self.is_connected() || !self.is_connecded_status_changed() {
            return Err(Error::port_not_newly_connected());
        }

        // TODO: care rw1cs
        let portsc_reg = self.set.port_status_and_control_mut();
        let mut portsc = portsc_reg.read();
        portsc.set_data_port_reset(true);
        portsc.set_data_connect_status_change(true);
        portsc_reg.write(portsc);

        while portsc_reg.read().get_data_port_reset() {}

        Ok(Port {
            set: self.set,
            _phantom_data: PhantomData,
        })
    }
}
