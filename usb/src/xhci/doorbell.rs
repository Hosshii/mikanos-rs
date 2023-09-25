use super::register_map::{Doorbell, DoorbellRegister};

pub struct DoorbellWrapper<'a, 'b> {
    reg: &'a mut DoorbellRegister<'b>,
}

impl<'a, 'b> DoorbellWrapper<'a, 'b> {
    pub fn new(reg: &'a mut DoorbellRegister<'b>) -> Self {
        Self { reg }
    }

    pub fn notify_host_controller(&mut self) {
        self.reg.write(Doorbell::default())
    }
}

impl<'a, 'b> From<&'a mut DoorbellRegister<'b>> for DoorbellWrapper<'a, 'b> {
    fn from(value: &'a mut DoorbellRegister<'b>) -> Self {
        Self { reg: value }
    }
}
