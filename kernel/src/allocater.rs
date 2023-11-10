use core::{
    alloc::GlobalAlloc,
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};

#[global_allocator]
static HEAP: StaticAllocator<{ 4096 * 1000 }> = StaticAllocator::new();

pub struct StaticAllocator<const SIZE: usize> {
    buf: [u8; SIZE],
    head: AtomicUsize,
}

impl<const SIZE: usize> StaticAllocator<SIZE> {
    pub const fn new() -> Self {
        Self {
            buf: [0; SIZE],
            head: AtomicUsize::new(0),
        }
    }
}

unsafe impl<const SIZE: usize> GlobalAlloc for StaticAllocator<SIZE> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let head = self.head.load(Ordering::SeqCst);
        let align = layout.align();
        let res = head % align;
        let start = if res == 0 { head } else { head + (align - res) };
        if start + align > self.buf.len() {
            ptr::null_mut()
        } else if self
            .head
            .compare_exchange(head, start + align, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            start as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, _: *mut u8, _: core::alloc::Layout) {}
}
