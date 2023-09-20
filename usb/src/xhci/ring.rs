use super::trb::{Link, TrbRaw};
use common::ring_buf::RingBuffer;
use core::mem::MaybeUninit;
use macros::bitfield_struct;

/// Transfer or Communicate ring.
#[repr(C, align(64))]
#[derive(Debug)]
pub struct TCRing<const SIZE: usize> {
    ring_buf: RingBuffer<TrbRaw, SIZE>,
    cycle_bit: bool,
}

impl<const SIZE: usize> TCRing<SIZE> {
    /// SIZE must greater than 2.
    pub fn new() -> Self {
        debug_assert!(2 <= SIZE);

        Self {
            ring_buf: RingBuffer::<_, SIZE>::zeroed(),
            cycle_bit: false,
        }
    }

    pub fn zeroed() -> Self {
        debug_assert!(2 <= SIZE);

        Self {
            ring_buf: RingBuffer::<_, SIZE>::zeroed(),
            cycle_bit: false,
        }
    }

    pub fn push(&mut self, v: impl Into<TrbRaw>) {
        let mut v: TrbRaw = v.into();
        v.set_remain_cycle_bit(self.cycle_bit);
        self.ring_buf.push_overwrite(v);

        if self.ring_buf.tail() % SIZE == SIZE - 1 {
            let link = Link::new(self.ring_buf.as_ptr() as *const ())
                .with_remain_toggle_cycle(true)
                .with_remain_cycle_bit(self.cycle_bit);
            let base = TrbRaw::from(link);
            self.ring_buf.push_overwrite(base);

            self.cycle_bit = !self.cycle_bit;

            debug_assert!(self.ring_buf.is_full());
        }
    }

    pub fn as_ptr(&self) -> *const MaybeUninit<TrbRaw> {
        self.ring_buf.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut MaybeUninit<TrbRaw> {
        self.ring_buf.as_mut_ptr()
    }

    pub fn cycle_bit(&self) -> bool {
        self.cycle_bit
    }
}

#[repr(C, align(64))]
pub struct EventRing<const SIZE: usize> {
    buf: [TrbRaw; SIZE],
    cycle_bit: bool,
    // position next read
    head: usize,
}

impl<const SIZE: usize> EventRing<SIZE> {
    pub fn new() -> Self {
        Self {
            buf: [TrbRaw::zeroed(); SIZE],
            cycle_bit: true,
            head: 0,
        }
    }

    pub fn pop<T>(&mut self) -> Option<T>
    where
        T: From<TrbRaw>,
    {
        if self.buf[self.head].get_remain_cycle_bit() == self.cycle_bit {
            let idx = self.head;

            if self.head == self.buf.len() - 1 {
                self.head = 0;
                self.cycle_bit = !self.cycle_bit;
            } else {
                self.head += 1;
            }

            Some(T::from(self.buf[idx]))
        } else {
            None
        }
    }

    pub fn as_ptr(&self) -> *const TrbRaw {
        self.buf.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut TrbRaw {
        self.buf.as_mut_ptr()
    }
}

bitfield_struct! {
    #[repr(C, align(64))]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    #[endian = "little"]
    pub struct EventRingSegmentTableEntry {
        ring_segment_base_address: u64 => {
            #[bits(6)]
            _rsvdz: u8,
            #[bits(58)]
            data: u64,
        },
        ring_segment_size: u32 => {
            #[bits(16)]
            data: u16,
            #[bits(16)]
            _rsvdz: u16,
        }
    }
}

impl EventRingSegmentTableEntry {
    pub fn zeroed() -> Self {
        Self::default()
    }
}
