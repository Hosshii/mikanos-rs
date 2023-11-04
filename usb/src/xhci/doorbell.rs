use core::marker::PhantomData;

use common::Zeroed;

use super::{
    device::EndpointID,
    register_map::{Doorbell, DoorbellRegister},
};

pub type HCDoorbell<'a, 'b> = DoorbellWrapper<'a, 'b, HostController>;
pub type DCDoorbell<'a, 'b> = DoorbellWrapper<'a, 'b, DeviceContext>;

pub struct HostController;
pub struct DeviceContext;

pub struct DoorbellWrapper<'a, 'b, T> {
    reg: &'a mut DoorbellRegister<'b>,
    _phantomdata: PhantomData<T>,
}

impl<'a, 'b, T> DoorbellWrapper<'a, 'b, T> {
    pub fn new(reg: &'a mut DoorbellRegister<'b>) -> Self {
        Self {
            reg,
            _phantomdata: PhantomData,
        }
    }

    pub fn ring(&mut self, val: Doorbell) {
        self.reg.write(val);
    }
}
impl<'a, 'b> DoorbellWrapper<'a, 'b, HostController> {
    pub fn notify_host_controller(&mut self) {
        self.ring(Doorbell::zeroed());
    }
}

impl<'a, 'b> DoorbellWrapper<'a, 'b, DeviceContext> {
    pub fn notify_endpoint(&mut self, endpoint_id: EndpointID) {
        // info!("dci: {}", endpoint_id.dci());
        self.ring(Doorbell::zeroed().with_data_db_target(endpoint_id.dci()));
    }
}

impl<'a, 'b> From<&'a mut DoorbellRegister<'b>> for DoorbellWrapper<'a, 'b, HostController> {
    fn from(value: &'a mut DoorbellRegister<'b>) -> Self {
        Self {
            reg: value,
            _phantomdata: PhantomData,
        }
    }
}

impl<'a, 'b> From<&'a mut DoorbellRegister<'b>> for DoorbellWrapper<'a, 'b, DeviceContext> {
    fn from(value: &'a mut DoorbellRegister<'b>) -> Self {
        Self {
            reg: value,
            _phantomdata: PhantomData,
        }
    }
}
