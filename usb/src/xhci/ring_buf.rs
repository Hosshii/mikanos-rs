use core::mem::MaybeUninit;

use super::{
    error::{Error, Result},
    trb::{Link, TRBRaw},
};

// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// スタックに確保されるリングバッファ。
/// SIZE 分の要素を入れられる。
struct RingBuffer<T, const SIZE: usize> {
    buf: [MaybeUninit<T>; SIZE],
    tail: usize,
    head: usize,
}

impl<T, const SIZE: usize> RingBuffer<T, SIZE> {
    pub fn new() -> Self {
        Self {
            buf: unsafe { MaybeUninit::zeroed().assume_init() },
            tail: 0,
            head: 0,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tail == self.head
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.tail - self.head == SIZE
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.tail - self.head
    }

    pub fn push(&mut self, v: T) -> Result<()> {
        if self.is_full() {
            return Err(Error::ring_buffer_full());
        }

        let idx = self.tail % SIZE;
        if self.contain_valid(idx) {
            unsafe {
                self.buf[idx].assume_init_drop();
            }
        }

        self.buf[idx].write(v);
        self.tail += 1;

        Ok(())
    }

    pub fn push_overwrite(&mut self, v: T) {
        if self.is_full() {
            self.head += 1;
        }

        self.push(v).expect("push error")
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            let idx = self.head % SIZE;
            let v = unsafe { self.buf[idx].assume_init_read() };
            self.buf[idx] = MaybeUninit::uninit();
            self.head += 1;
            Some(v)
        }
    }

    #[inline]
    fn contain_valid(&self, idx: usize) -> bool {
        let base = self.head / SIZE;
        let base = base * SIZE + idx;
        (self.head..self.tail).contains(&base)
    }

    pub fn as_ptr(&self) -> *const MaybeUninit<T> {
        self.buf.as_ptr()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIZE: usize = 10;

    #[test]
    fn test_valid() {
        let mut v = RingBuffer::<usize, SIZE>::new();
        assert!((0..SIZE).all(|idx| !v.contain_valid(idx)));

        v.push(1).unwrap();
        assert!(v.contain_valid(0));
        assert!(!v.contain_valid(1));

        for i in 1..SIZE {
            v.push(i).unwrap();
        }

        assert!((0..SIZE).all(|idx| v.contain_valid(idx)));
        v.pop().unwrap();

        assert!(!v.contain_valid(0));
    }

    #[test]
    fn test_push_pop() {
        let mut v = RingBuffer::<usize, SIZE>::new();

        assert_eq!(v.pop(), None);

        v.push(1).unwrap();
        v.push(2).unwrap();
        assert_eq!(v.pop(), Some(1));
        assert_eq!(v.pop(), Some(2));

        for i in 0..SIZE {
            v.push(i).unwrap();
        }
        assert!(v.push(10).is_err());
    }

    #[test]
    fn test_len() {
        let mut v = RingBuffer::<usize, SIZE>::new();

        assert!(v.is_empty());
        assert!(!v.is_full());
        assert_eq!(v.len(), 0);

        v.push(0).unwrap();
        assert!(!v.is_empty());
        assert!(!v.is_full());
        assert_eq!(v.len(), 1);

        for i in 1..SIZE {
            v.push(i).unwrap();
        }

        assert!(!v.is_empty());
        assert!(v.is_full());
        assert_eq!(v.len(), SIZE);
    }

    #[test]
    fn test_push_overwrite() {
        let mut v = RingBuffer::<usize, SIZE>::new();

        for i in 0..SIZE {
            v.push(i).unwrap();
        }
        assert!(v.push(0).is_err());

        for i in SIZE..SIZE + 4 {
            v.push_overwrite(i);
            assert!(v.is_full());
        }

        for i in 0..SIZE {
            assert_eq!(v.pop().unwrap(), i + 4);
        }
    }
}

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
