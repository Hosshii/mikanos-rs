#![no_std]
#![no_main]

mod uefi;

use core::panic::PanicInfo;

use uefi::{Handle, Status, SystemTable};

use crate::uefi::CStr16;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "efiapi" fn efi_main(image_handle: Handle, system_table: SystemTable) -> Status {
    let stdout = unsafe { &mut *(system_table.con_out) };
    (stdout.clear_screen)(stdout);

    let mut buf = [0u16; 14];
    let hello = CStr16::from_str_with_buf("hello world!!", &mut buf).unwrap();

    unsafe { (stdout.output_string)(stdout, hello.as_ptr()) };

    loop {}

    Status::Success
}
