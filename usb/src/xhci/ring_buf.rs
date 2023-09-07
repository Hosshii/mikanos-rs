use super::trb::{Link, TRBRaw};
use common::ring_buf::RingBuffer;

/// Transfer or Communicate ring.
struct TCRing<const SIZE: usize> {
    ring_buf: RingBuffer<TRBRaw, SIZE>,
    cycle_bit: bool,
}

impl<const SIZE: usize> TCRing<SIZE> {
    /// SIZE must greater than 2.
    pub fn new() -> Self {
        debug_assert!(2 <= SIZE);

        Self {
            ring_buf: RingBuffer::<_, SIZE>::new(),
            cycle_bit: false,
        }
    }

    pub fn push(&mut self, v: TRBRaw) {
        self.ring_buf.push_overwrite(v);
        if self.ring_buf.len() == SIZE - 1 {
            let link = Link::new(self.ring_buf.as_ptr() as *const ())
                .set_toggle(true)
                .set_cycle(self.cycle_bit);
            let base = TRBRaw::from(link);
            self.ring_buf.push_overwrite(base);

            self.cycle_bit = !self.cycle_bit;

            debug_assert!(self.ring_buf.is_full());
        }
    }
}
