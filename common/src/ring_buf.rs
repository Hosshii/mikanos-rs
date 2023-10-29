use core::mem::MaybeUninit;
use core::ops::{Index, IndexMut};

use crate::Zeroed;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(ErrorKind);

impl Error {
    pub fn ring_buffer_full() -> Self {
        Error(ErrorKind::RingBufferFull)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    RingBufferFull,
}

/// スタックに確保されるリングバッファ。
/// SIZE 分の要素を入れられる。
/// ptrは`RingBuffer`とおなじアライメントになる
#[derive(Debug)]
#[repr(C)]
pub struct RingBuffer<T, const SIZE: usize> {
    buf: [MaybeUninit<T>; SIZE],
    tail: usize,
    head: usize,
}

impl<T, const SIZE: usize> RingBuffer<T, SIZE> {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tail == self.head
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.tail - self.head == SIZE
    }

    pub fn iter(&self) -> Iter<'_, T, SIZE> {
        Iter::new(self)
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
        if self.is_valid_buf_idx(idx) {
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

    /// ruturns whether `buf[buf_idx]` contains valid value or not.
    #[inline]
    fn is_valid_buf_idx(&self, buf_idx: usize) -> bool {
        if SIZE <= buf_idx {
            return false;
        }

        let head_base = self.head / SIZE;
        let head_base = head_base * SIZE + buf_idx;
        let tail_base = self.tail / SIZE;
        let tail_base = tail_base * SIZE + buf_idx;
        (self.head..self.tail).contains(&head_base) || (self.head..self.tail).contains(&tail_base)
    }

    /// スライスインデックスが範囲内かつvalidなところを指している場合、
    /// それがbufのどこにあるかを返す。それ以外ではNoneを返す。
    fn calc_buf_index(&self, slice_idx: usize) -> Option<usize> {
        if SIZE <= slice_idx {
            return None;
        }

        let buf_idx = (self.head + slice_idx) % SIZE;
        if self.is_valid_buf_idx(buf_idx) {
            Some(buf_idx)
        } else {
            None
        }
    }

    pub fn as_ptr(&self) -> *const MaybeUninit<T> {
        self.buf.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut MaybeUninit<T> {
        self.buf.as_mut_ptr()
    }

    pub fn tail(&self) -> usize {
        self.tail
    }
}

impl<T, const SIZE: usize> Zeroed for RingBuffer<T, SIZE> {
    fn zeroed() -> Self {
        Self {
            buf: unsafe { MaybeUninit::zeroed().assume_init() },
            tail: 0,
            head: 0,
        }
    }
}

impl<T, const SIZE: usize> Index<usize> for RingBuffer<T, SIZE> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        let index = self.calc_buf_index(index).expect("invalid index");
        unsafe { self.buf[index].assume_init_ref() }
    }
}

impl<T, const SIZE: usize> IndexMut<usize> for RingBuffer<T, SIZE> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let index = self.calc_buf_index(index).expect("invalid index");

        unsafe { self.buf[index].assume_init_mut() }
    }
}

impl<T: Clone, const SIZE: usize> Clone for RingBuffer<T, SIZE> {
    fn clone(&self) -> Self {
        let mut new = Self::zeroed();
        new.head = self.head;
        new.tail = self.tail;

        for (i, elem) in new.buf.iter_mut().enumerate() {
            if self.is_valid_buf_idx(i) {
                *elem = MaybeUninit::new(self[i].clone());
            }
        }

        new
    }
}

impl<T: PartialEq, const SIZE: usize> PartialEq for RingBuffer<T, SIZE> {
    fn eq(&self, other: &Self) -> bool {
        if self.iter().count() != other.iter().count() {
            false
        } else {
            self.iter().zip(other.iter()).all(|(l, r)| l == r)
        }
    }
}

impl<T: Eq, const SIZE: usize> Eq for RingBuffer<T, SIZE> {}

pub struct Iter<'a, T, const SIZE: usize> {
    buf: &'a RingBuffer<T, SIZE>,
    count: usize,
}

impl<'a, T, const SIZE: usize> Iter<'a, T, SIZE> {
    pub fn new(buf: &'a RingBuffer<T, SIZE>) -> Self {
        Self { buf, count: 0 }
    }
}

impl<'a, T, const SIZE: usize> Iterator for Iter<'a, T, SIZE> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() <= self.count {
            None
        } else {
            let idx = self.count;
            self.count += 1;
            Some(&self.buf[idx])
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    const SIZE: usize = 10;

    #[test]
    fn test_valid() {
        let mut v = RingBuffer::<usize, SIZE>::zeroed();
        assert!((0..SIZE).all(|idx| !v.is_valid_buf_idx(idx)));

        v.push(1).unwrap();
        assert!(v.is_valid_buf_idx(0));
        assert!(!v.is_valid_buf_idx(1));

        for i in 1..SIZE {
            v.push(i).unwrap();
        }

        assert!((0..SIZE).all(|idx| v.is_valid_buf_idx(idx)));
        v.pop().unwrap();

        assert!(!v.is_valid_buf_idx(0));
    }

    #[test]
    fn test_push_pop() {
        let mut v = RingBuffer::<usize, SIZE>::zeroed();

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
        let mut v = RingBuffer::<usize, SIZE>::zeroed();

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
        let mut v = RingBuffer::<usize, SIZE>::zeroed();

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

    #[test]
    fn test_index() {
        let mut v = RingBuffer::<usize, SIZE>::zeroed();

        for i in 0..SIZE {
            v.push(i).unwrap();
            assert_eq!(v[i], i);
        }

        for i in SIZE..SIZE + 4 {
            v.push_overwrite(i);
            assert_eq!(v[0], i - SIZE + 1);
        }
    }

    #[test]
    fn test_iter() {
        let mut v = RingBuffer::<usize, SIZE>::zeroed();

        assert!(v.iter().next().is_none());

        for i in 0..SIZE {
            v.push(i).unwrap();
        }

        let mut iter = v.iter();
        for i in 0..SIZE {
            assert_eq!(i, *iter.next().unwrap())
        }

        for i in 0..SIZE {
            v.push_overwrite(i + SIZE);
        }

        let mut iter = v.iter();
        for i in 0..SIZE {
            assert_eq!(i + SIZE, *iter.next().unwrap());
        }

        assert!(iter.next().is_none())
    }
}
