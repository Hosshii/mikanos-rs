use common::{info, Zeroed};

use super::{
    device::EndpointID,
    register_map::{Doorbell, DoorbellRegister},
};

pub struct DoorbellWrapper<'a, 'b> {
    reg: &'a mut DoorbellRegister<'b>,
}

impl<'a, 'b> DoorbellWrapper<'a, 'b> {
    pub fn new(reg: &'a mut DoorbellRegister<'b>) -> Self {
        Self { reg }
    }

    pub fn notify_host_controller(&mut self) {
        self.reg.write(Doorbell::zeroed())
    }

    pub fn notify_endpoint(&mut self, endpoint_id: EndpointID) {
        info!("dci: {}", endpoint_id.dci());
        self.reg
            .write(Doorbell::zeroed().with_data_db_target(endpoint_id.dci()))
    }
}

impl<'a, 'b> From<&'a mut DoorbellRegister<'b>> for DoorbellWrapper<'a, 'b> {
    fn from(value: &'a mut DoorbellRegister<'b>) -> Self {
        Self { reg: value }
    }
}
