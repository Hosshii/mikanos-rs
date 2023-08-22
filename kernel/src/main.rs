#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

use common::KernelArg;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// entry point
#[no_mangle]
pub extern "C" fn kernel_main(arg: &mut KernelArg) -> ! {
    let frame_buffer = arg.frame_buffer_base as *mut u8;
    for offset in 0..arg.frame_buffer_size {
        unsafe { frame_buffer.add(offset).write_volatile(offset as u8) }
    }
    halt()
}

#[cfg(target_arch = "x86_64")]
#[inline]
fn halt() -> ! {
    loop {
        unsafe {
            asm! {"hlt"}
        }
    }
}
#[cfg(target_arch = "aarch64")]
#[inline]
fn halt() -> ! {
    loop {
        unsafe {
            asm! {"wfi"}
        }
    }
}
