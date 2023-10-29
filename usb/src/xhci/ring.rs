use super::{
    register_map::InterrupterRegisterSet,
    trb::{Link, TrbRaw},
};
use common::{debug, ring_buf::RingBuffer, Zeroed as _};
use core::mem::MaybeUninit;
use macros::bitfield_struct;

/// Transfer or Communicate ring.
#[repr(C, align(64))]
#[derive(Debug, Clone, PartialEq, Eq)]
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
            cycle_bit: true,
        }
    }

    pub fn zeroed() -> Self {
        debug_assert!(2 <= SIZE);

        Self {
            ring_buf: RingBuffer::<_, SIZE>::zeroed(),
            cycle_bit: true,
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
}

impl<const SIZE: usize> EventRing<SIZE> {
    pub fn new() -> Self {
        Self {
            buf: [TrbRaw::zeroed(); SIZE],
            cycle_bit: true,
        }
    }

    pub fn pop<T>(&mut self, irs: &mut InterrupterRegisterSet<'_>) -> Option<T>
    where
        T: From<TrbRaw>,
    {
        let ptr = irs.event_ring_dequeue_pointer().read().get_data_ptr() << 4;
        let ptr = ptr as *mut TrbRaw;
        assert!(self.buf.as_mut_ptr_range().contains(&ptr));

        if unsafe { *ptr }.get_remain_cycle_bit() != self.cycle_bit {
            return None;
        }

        // forward erdp
        let segment_begin = irs
            .event_ring_segment_table_base_address()
            .read()
            .get_data_ptr()
            << 6;
        let segment_begin = segment_begin as *mut TrbRaw;

        let segment_size = irs
            .event_ring_segment_table_size()
            .read()
            .get_data_event_ring_segment_table_size();

        let segment_end = unsafe { segment_begin.add(segment_size as usize) };

        if ptr == segment_end {
            self.cycle_bit = !self.cycle_bit;
        }

        let next_ptr = if ptr == segment_end {
            segment_begin
        } else {
            unsafe { ptr.add(1) }
        };

        let erdp = irs
            .event_ring_dequeue_pointer()
            .read()
            .with_data_ptr((next_ptr as u64) >> 4);
        irs.event_ring_dequeue_pointer_mut().write(erdp);

        Some(T::from(unsafe { *ptr }))
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
