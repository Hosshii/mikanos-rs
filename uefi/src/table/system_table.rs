use crate::{
    protocol::console::{SimpleTextInputProtocol, SimpleTextOutputProtocol},
    types::{CStr16, Char16, Handle, Status, Uint32, Uintn},
};

use super::{boot_services::BootServices, header::TableHeader};

#[repr(C)]
pub struct SystemTable {
    pub hdr: TableHeader,
    pub firmware_vender: *const Char16,
    pub firmware_revision: Uint32,
    pub console_handle: Handle,
    pub con_in: *mut SimpleTextInputProtocol,
    pub console_out_handle: Handle,
    pub con_out: *mut SimpleTextOutputProtocol,
    pub standard_error_handle: Handle,
    // pub std_error: *mut SimpleTextOutputProtocol,
    pub std_error: *mut usize,
    // pub runtime_services: *const RuntimeServices,
    pub runtime_services: *const usize,
    pub boot_services: *const BootServices,
    // pub boot_services: *const usize,
    pub number_of_table_entries: Uintn,
    // pub configuration_table: *const ConfigurationTable,
    pub configuration_table: *const usize,
}

impl SystemTable {
    pub fn boot_services(&self) -> &BootServices {
        unsafe { &*self.boot_services }
    }

    pub fn stdout(&mut self) -> &mut SimpleTextOutputProtocol {
        unsafe { &mut *(self.con_out) }
    }

    pub fn clear_screen(&mut self) -> Status {
        let stdout = self.stdout();
        (stdout.clear_screen)(stdout)
    }
}
