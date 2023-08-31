use core::mem::MaybeUninit;

use super::{
    error::{Error, Result},
    trb::TRBBase,
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
}

/// Transfer or Communicate ring.
struct TCRing<const SIZE: usize> {
    ring_buf: RingBuffer<TRBBase, SIZE>,
}
