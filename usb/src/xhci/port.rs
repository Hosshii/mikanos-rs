use common::debug;

use crate::xhci::register_map::PortRegisterSet;

use super::{
    error::{Error, Result},
    register_map::MAX_PORT_REGISTER_SET_NUM,
};

#[derive(Debug)]
pub struct Disabled;

#[derive(Debug)]
pub struct Enabled;

#[derive(Debug)]
pub struct PortWrapper<'a, 'b, 'c> {
    set: &'a mut PortRegisterSet<'b>,
    port_num: u8,
    phase: &'c mut PortConfigPhase,
}

impl<'a, 'b, 'c> PortWrapper<'a, 'b, 'c> {
    pub fn new(
        set: &'a mut PortRegisterSet<'b>,
        port_num: u8,
        phase: &'c mut PortConfigPhase,
    ) -> Self {
        Self {
            set,
            port_num,
            phase,
        }
    }

    pub fn number(&self) -> u8 {
        self.port_num
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

    pub fn is_enabled(&self) -> bool {
        self.set
            .port_status_and_control()
            .read()
            .get_data_port_enabled_disabled()
    }

    pub fn is_port_reset_changed(&self) -> bool {
        self.set
            .port_status_and_control()
            .read()
            .get_data_port_reset_change()
    }

    pub fn clear_port_reset_change(&mut self) {
        let v = self
            .set
            .port_status_and_control()
            .read()
            .clear_rw1s()
            .with_data_port_reset_change(true);
        self.set.port_status_and_control_mut().write(v);
    }

    pub fn set_phase(&mut self, phase: PortConfigPhase) {
        *self.phase = phase;
    }

    fn reset(&mut self) -> Result<()> {
        debug!("reset port");
        self.set_phase(PortConfigPhase::ResettingPort);

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

    pub fn phase(&self) -> &PortConfigPhase {
        self.phase
    }
}

pub const MAX_PORTS_NUM: usize = MAX_PORT_REGISTER_SET_NUM;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortsConfigPhase {
    status: [PortConfigPhase; MAX_PORTS_NUM],
}

impl PortsConfigPhase {
    pub fn set_phase(&mut self, idx: u8, phase: PortConfigPhase) {
        self.status[idx as usize] = phase
    }

    pub fn phase(&self, idx: u8) -> PortConfigPhase {
        self.status[idx as usize]
    }

    pub fn phases_mut(&mut self) -> &mut [PortConfigPhase; MAX_PORTS_NUM] {
        &mut self.status
    }
}

impl Default for PortsConfigPhase {
    fn default() -> Self {
        Self {
            status: [PortConfigPhase::NotConnected; MAX_PORTS_NUM],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortConfigPhase {
    NotConnected,
    WaitingAddressed,
    ResettingPort,
    EnablingSlot,
    AddressingDevice,
    InitializingDevice,
    ConfiguringEndpoints,
    Configured,
}
