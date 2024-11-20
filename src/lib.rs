//! A no_std friendly RingBuffer data structure using const generics for buffer size.

#![deny(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(not(test), no_std)]

use embedded_io::ErrorType;

/// A RingBuffer holds SIZE elements of type T.
pub struct RingBuffer<T, const SIZE: usize> {
    data: [T; SIZE],

    oldest: usize,
    num_elems: usize,
}

/// Errors while handling a RingBuffer
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Buffer is full.
    #[error("Buffer is full")]
    BufferFull,
}

impl<T: Default + Copy, const SIZE: usize> Default for RingBuffer<T, SIZE> {
    fn default() -> Self {
        RingBuffer {
            data: [T::default(); SIZE],
            oldest: 0,
            num_elems: 0,
        }
    }
}

impl<T: Default + Copy, const SIZE: usize> RingBuffer<T, SIZE> {
    /// Make a new RingBuffer!
    pub fn new() -> Self {
        Default::default()
    }

    /// Push something onto the buffer. If the buffer is full, the oldest element is returned.
    pub fn push_overwrite(&mut self, elem: T) -> Option<T> {
        let index = (self.oldest + self.num_elems) % SIZE;
        if self.is_full() {
            let oldest = self.data[self.oldest];
            self.oldest = (self.oldest + 1) % SIZE;
            self.data[index] = elem;
            Some(oldest)
        } else {
            self.data[index] = elem;
            self.num_elems += 1;
            None
        }
    }

    /// Push something onto the buffer, unless the buffer is full.
    pub fn push_unless_full(&mut self, elem: T) -> Result<(), Error> {
        if self.is_full() {
            return Err(Error::BufferFull);
        }
        let index = (self.oldest + self.num_elems) % SIZE;
        self.data[index] = elem;
        self.num_elems += 1;
        Ok(())
    }

    /// Pop the buffer. If it is empty, then None is returned.
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let elem = self.data[self.oldest];
        self.oldest = (self.oldest + 1) % SIZE;
        self.num_elems -= 1;
        Some(elem)
    }

    /// Peek at the next element in the buffer. None if buffer empty.
    pub fn peek(&self) -> Option<&T> {
        if self.is_empty() {
            return None;
        }
        let elem = &self.data[self.oldest];
        Some(elem)
    }

    /// How many elements are in the buffer?
    pub fn len(&self) -> usize {
        self.num_elems
    }

    /// Is the buffer empty?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Is the buffer full?
    pub fn is_full(&self) -> bool {
        self.len() == SIZE
    }

    /// Pop some elements into the buffer
    pub fn pop_many(&mut self, buf: &mut [T]) -> usize {
        let count = usize::min(self.len(), buf.len());
        for p in buf.iter_mut().take(count) {
            *p = self.pop().unwrap();
        }
        count
    }
}

/// Consuming IntoIterator for Ringbuffer
pub struct ConsumingIntoIteratorRingbuffer<T, const SIZE: usize> {
    buffer: RingBuffer<T, SIZE>,
}

impl<T: Default + Copy, const SIZE: usize> IntoIterator for RingBuffer<T, SIZE> {
    type Item = T;
    type IntoIter = ConsumingIntoIteratorRingbuffer<T, SIZE>;

    fn into_iter(self) -> Self::IntoIter {
        ConsumingIntoIteratorRingbuffer { buffer: self }
    }
}

impl<T: Default + Copy, const SIZE: usize> Iterator for ConsumingIntoIteratorRingbuffer<T, SIZE> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.pop()
    }
}

/// IntoIterator for Ringbuffer
pub struct IntoIteratorRingbuffer<'a, T, const SIZE: usize> {
    buffer: &'a RingBuffer<T, SIZE>,
    current: usize,
}

impl<'a, T: Default + Copy, const SIZE: usize> IntoIterator for &'a RingBuffer<T, SIZE> {
    type Item = &'a T;
    type IntoIter = IntoIteratorRingbuffer<'a, T, SIZE>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIteratorRingbuffer {
            buffer: self,
            current: 0,
        }
    }
}

impl<'a, T: Default + Copy, const SIZE: usize> Iterator for IntoIteratorRingbuffer<'a, T, SIZE> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.buffer.num_elems {
            let index = (self.buffer.oldest + self.current) % SIZE;
            let elem = &self.buffer.data[index];
            self.current += 1;
            Some(elem)
        } else {
            None
        }
    }
}

impl<T: Default + Copy, const SIZE: usize> RingBuffer<T, SIZE> {
    /// Make an immutable non-consuming iterator
    pub fn iter(&self) -> IntoIteratorRingbuffer<T, SIZE> {
        self.into_iter()
    }
}

impl<T, const SIZE: usize> ErrorType for RingBuffer<T, SIZE> {
    type Error = crate::Error;
}

impl embedded_io::Error for crate::Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        match self {
            Error::BufferFull => embedded_io::ErrorKind::OutOfMemory,
        }
    }
}

impl<const SIZE: usize> embedded_io::Read for RingBuffer<u8, SIZE> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(self.pop_many(buf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut buffer = RingBuffer::<u16, 8>::new();
        assert!(buffer.is_empty());

        buffer.push_overwrite(1);
        assert_eq!(1, buffer.len());

        let one = buffer.pop().unwrap();
        assert_eq!(1, one);
    }

    #[test]
    fn pop_empty() {
        let mut buffer = RingBuffer::<u8, 8>::new();
        assert_eq!(buffer.pop(), None);
    }

    #[test]
    fn single_element() {
        let mut buffer = RingBuffer::<u8, 1>::new();
        assert!(buffer.is_empty());
        buffer.push_overwrite(1);
        assert!(buffer.is_full());
        assert_eq!(1, buffer.pop().unwrap());
        buffer.push_overwrite(2);
        buffer.push_overwrite(3);
        assert_eq!(3, buffer.pop().unwrap());
    }

    #[test]
    fn consuming_iterator() {
        let mut buffer = RingBuffer::<u8, 8>::new();
        buffer.push_overwrite(5);
        buffer.push_overwrite(125);
        buffer.push_overwrite(0);
        let mut iter = std::iter::IntoIterator::into_iter(buffer);
        assert_eq!(5, iter.next().unwrap());
        assert_eq!(125, iter.next().unwrap());
        assert_eq!(0, iter.next().unwrap());
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn iterator_immutable() {
        let mut buffer = RingBuffer::<u8, 4>::new();
        buffer.push_overwrite(1);
        buffer.push_overwrite(2);
        buffer.push_overwrite(3);
        let mut iter = std::iter::IntoIterator::into_iter(&buffer);
        assert_eq!(&1, iter.next().unwrap());
        assert_eq!(&2, iter.next().unwrap());
        assert_eq!(&3, iter.next().unwrap());
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next());
        buffer.push_overwrite(4);
        buffer.push_overwrite(5);
        buffer.push_overwrite(6);
        assert_eq!(4, buffer.len());
    }

    #[test]
    fn iter_convenience() {
        let mut buffer = RingBuffer::<u8, 4>::new();

        buffer.push_overwrite(1);
        buffer.push_overwrite(2);
        buffer.push_overwrite(3);

        let iter = std::iter::IntoIterator::into_iter(&buffer);
        let iter2 = buffer.iter();

        for (x, y) in iter.zip(iter2) {
            assert_eq!(x, y);
        }
    }

    #[test]
    fn push_overwrite() {
        let mut buffer = RingBuffer::<u128, 4>::new();
        assert!(buffer.push_overwrite(1).is_none());
        assert!(buffer.push_overwrite(2).is_none());
        assert!(buffer.push_overwrite(3).is_none());
        assert!(buffer.push_overwrite(4).is_none());
        assert_eq!(1, buffer.push_overwrite(5).unwrap());
        assert_eq!(2, buffer.push_overwrite(6).unwrap());
        assert_eq!(3, buffer.push_overwrite(7).unwrap());
        assert_eq!(4, buffer.push_overwrite(8).unwrap());
        assert_eq!(5, buffer.push_overwrite(9).unwrap());
        assert_eq!(4, buffer.len());
        assert!(buffer.is_full());
    }

    #[test]
    fn fail_overwrite() {
        let mut buffer = RingBuffer::<u128, 4>::new();
        assert!(buffer.push_overwrite(1).is_none());
        assert!(buffer.push_overwrite(2).is_none());
        assert!(buffer.push_overwrite(3).is_none());
        assert!(buffer.push_overwrite(4).is_none());

        assert!(buffer.push_unless_full(5).is_err());
        assert!(buffer.push_unless_full(6).is_err());
        match buffer.push_unless_full(192) {
            Err(Error::BufferFull) => assert!(true),
            _ => unreachable!("Wrong error variant!"),
        }

        assert_eq!(1, buffer.pop().unwrap());
        assert!(buffer.push_unless_full(5).is_ok());
    }

    #[test]
    fn pop_many() {
        let mut buffer = RingBuffer::<u8, 4>::new();
        buffer.push_overwrite(1);
        buffer.push_overwrite(2);
        buffer.push_overwrite(3);
        buffer.push_overwrite(4);

        let two = &mut [0, 0];
        assert_eq!(2, buffer.pop_many(two));
        assert_eq!(two, &[1, 2]);

        assert_eq!(2, buffer.pop_many(two));
        assert_eq!(two, &[3, 4]);

        assert_eq!(0, buffer.pop_many(two));
        assert_eq!(two, &[3, 4]);
    }
}
