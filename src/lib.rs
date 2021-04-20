//! A no_std friendly RingBuffer data structure using const generics for buffer size.

#![deny(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(not(test), no_std)]

/// A RingBuffer holds SIZE elements of type T.
pub struct RingBuffer<T, const SIZE: usize> {
    data: [T; SIZE],

    oldest: usize,
    num_elems: usize,
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

    /// Push something onto the buffer. If the buffer is full, the oldest element is overwritten.
    pub fn push(&mut self, elem: T) {
        let index: usize = (self.oldest + self.num_elems) % SIZE;
        self.data[index] = elem;
        if self.num_elems == SIZE {
            self.oldest = (self.oldest + 1) % SIZE;
        } else {
            self.num_elems += 1;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut buffer = RingBuffer::<u16, 8>::new();
        assert!(buffer.is_empty());

        buffer.push(1);
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
        buffer.push(1);
        assert!(buffer.is_full());
        assert_eq!(1, buffer.pop().unwrap());
        buffer.push(2);
        buffer.push(3);
        assert_eq!(3, buffer.pop().unwrap());
    }

    #[test]
    fn consuming_iterator() {
        let mut buffer = RingBuffer::<u8, 8>::new();
        buffer.push(5);
        buffer.push(125);
        buffer.push(0);
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
        buffer.push(1);
        buffer.push(2);
        buffer.push(3);
        let mut iter = std::iter::IntoIterator::into_iter(&buffer);
        assert_eq!(&1, iter.next().unwrap());
        assert_eq!(&2, iter.next().unwrap());
        assert_eq!(&3, iter.next().unwrap());
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next());
        buffer.push(4);
        buffer.push(5);
        buffer.push(6);
        assert_eq!(4, buffer.len());
    }

    #[test]
    fn iter_convenience() {
        let mut buffer = RingBuffer::<u8, 4>::new();

        buffer.push(1);
        buffer.push(2);
        buffer.push(3);

        let iter = std::iter::IntoIterator::into_iter(&buffer);
        let iter2 = buffer.iter();

        for (x, y) in iter.zip(iter2) {
            assert_eq!(x, y);
        }
    }
}
