#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// entry point
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    halt()
}

#[cfg(target_arch = "x86_64")]
fn halt() -> ! {
    loop {
        unsafe {
            asm! {"hlt"}
        }
    }
}
#[cfg(target_arch = "aarch64")]
fn halt() -> ! {
    loop {
        unsafe {
            asm! {"wfi"}
        }
    }
}
