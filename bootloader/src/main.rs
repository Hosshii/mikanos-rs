#![no_std]
#![no_main]

mod uefi;

use core::{hint::black_box, panic::PanicInfo};

use uefi::{CStr16, Handle, Status, SystemTable};

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

    for i in 0..10000000000i64 {
        black_box(i);
    }

    Status::Success
}
