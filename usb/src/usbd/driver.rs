use crate::xhci::{
    driver::{Controller, Running, Uninitialized},
    port::PortConfigPhase,
};

use super::error::Result;

pub struct Driver<'a> {
    xhcid: Controller<'a, Running>,
}

impl<'a> Driver<'a> {
    pub fn new(xhcid: Controller<'a, Uninitialized>) -> Result<Self> {
        let mut xhcid = xhcid.initialize()?.run();

        for mut port in xhcid.ports_mut() {
            if port.is_connected() {
                port.set_phase(PortConfigPhase::WaitingAddressed)
            }
        }

        Ok(Self { xhcid })
    }

    pub fn process(&mut self) -> Result<()> {
        self.xhcid.process_primary_event()?;
        let dev = self.xhcid.devices_mut();
        Ok(())
    }
}
