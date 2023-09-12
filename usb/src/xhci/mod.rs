use self::register_map::{CapabilityRegisters, OperationalRegisters};
use common::debug;
use core::marker::PhantomData;

mod context;
mod endian;
mod register_map;
mod ring_buf;
mod trb;

pub struct Initial;
pub struct Initialized;

pub struct Controller<State> {
    _phantomdata: PhantomData<State>,
    capability_registers: CapabilityRegisters<'static>,
    operational_registers: OperationalRegisters<'static>,
}

impl Controller<Initial> {
    /// # Safety
    /// bar must be correct address.
    pub unsafe fn new(bar: u64) -> Self {
        let capability_registers = unsafe { CapabilityRegisters::new(bar as *const u8) };

        let op_offset = capability_registers.cap_length().read().get_data();
        let op_base = bar + op_offset as u64;
        let operational_registers = OperationalRegisters::new(op_base as *mut u8);

        Self {
            _phantomdata: PhantomData,
            capability_registers,
            operational_registers,
        }
    }

    pub fn initialize(mut self) -> Controller<Initialized> {
        self.reset();

        Controller {
            _phantomdata: PhantomData,
            capability_registers: self.capability_registers,
            operational_registers: self.operational_registers,
        }
    }

    fn reset(&mut self) {
        debug!("start xhci reset");
        debug!("wait host controller halted");
        while !self
            .operational_registers
            .usb_status()
            .read()
            .get_data_host_controller_halted()
        {}

        // reset host controller
        debug!("reset host controller");
        let mut cmd = self.operational_registers.usb_command().read();
        cmd.set_data_host_controller_reset(true);
        self.operational_registers.usb_command_mut().write(cmd);
        while self
            .operational_registers
            .usb_command()
            .read()
            .get_data_host_controller_reset()
        {}

        debug!("wait controller not ready");
        while self
            .operational_registers
            .usb_status()
            .read()
            .get_data_controller_not_ready()
        {}
    }
}
