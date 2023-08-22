#![no_std]

#[repr(C)]
pub struct KernelArg {
    pub frame_buffer_base: u64,
    pub frame_buffer_size: usize,
}
