//! A double-ended priority queue.
//!
//! Insertion, removal of the largest element, and removal of the
//! smallest element all have `O(log n)` time complexity. Checking the
//! largest or smallest element is `O(1)`.

#![warn(missing_docs)]

use std::{mem, slice, vec};

mod hole;
mod index;

use self::hole::*;

/// A double-ended priority queue.
#[derive(Clone, Debug)]
pub struct MinMaxHeap<T>(Vec<T>);

impl<T> Default for MinMaxHeap<T> {
    fn default() -> Self {
        MinMaxHeap::new()
    }
}

impl<T> MinMaxHeap<T> {
    /// Creates a new, empty `MinMaxHeap`.
    pub fn new() -> Self {
        MinMaxHeap(Vec::new())
    }

    /// Creates a new, empty `MinMaxHeap` with space allocated to hold
    /// `len` elements.
    pub fn with_capacity(len: usize) -> Self {
        MinMaxHeap(Vec::with_capacity(len))
    }

    /// The number of elements in the heap.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Is the heap empty?
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T: Ord> MinMaxHeap<T> {
    /// Adds an element to the heap.
    pub fn push(&mut self, element: T) {
        let pos = self.len();
        self.0.push(element);
        Hole::new(&mut self.0, pos).bubble_up()
    }

    /// Gets a reference to the minimum element, if any.
    pub fn peek_min(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            Some(&self.0[0])
        }
    }

    /// Gets a reference to the maximum element, if any.
    pub fn peek_max(&self) -> Option<&T> {
        self.find_max().map(|i| &self.0[i])
    }

    fn find_max(&self) -> Option<usize> {
        match self.len() {
            0 => None,
            1 => Some(0),
            2 => Some(1),
            _ => if &self.0[1] > &self.0[2] { Some(1) } else { Some(2) }
        }
    }

    /// Removes the minimum element, if any.
    pub fn pop_min(&mut self) -> Option<T> {
        self.0.pop().map(|mut item| {
            if !self.is_empty() {
                mem::swap(&mut item, &mut self.0[0]);
                Hole::new(&mut self.0, 0).trickle_down_min();
            }

            item
        })
    }

    /// Removes the maximum element, if any.
    pub fn pop_max(&mut self) -> Option<T> {
        self.find_max().map(|max| {
            let mut item = self.0.pop().unwrap();

            if max < self.len() {
                mem::swap(&mut item, &mut self.0[max]);
                Hole::new(&mut self.0, max).trickle_down_max();
            }

            item
        })
    }
}

impl<T> MinMaxHeap<T> {
    /// Drops all items from the heap.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// The number of elements the heap can hold without reallocating.
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// Reserves the minimum capacity for exactly `additional` more
    /// elements to be inserted in the given `MinMaxHeap`.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity overflows `usize`.
    pub fn reserve_exact(&mut self, additional: usize) {
        self.0.reserve_exact(additional)
    }

    /// Reserves the minimum capacity for at least `additional` more
    /// elements to be inserted in the given `MinMaxHeap`.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity overflows `usize`.
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional)
    }

    /// Discards extra capacity.
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }

    /// Consumes the `MinMaxHeap` and returns its elements in a vector
    /// in arbitrary order.
    pub fn into_vec(self) -> Vec<T> {
        self.0
    }

    /// Returns a borrowing iterator over the min-max-heap’s elements in
    /// arbitrary order.
    pub fn iter(&self) -> Iter<T> {
        Iter(self.0.iter())
    }

    /// Returns a owning iterator over the min-max-heap’s elements in
    /// arbitrary order.
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self.0.into_iter())
    }

    /// Returns a draining iterator over the min-max-heap’s elements in
    /// arbitrary order.
    pub fn drain(&mut self) -> Drain<T> {
        Drain(self.0.drain(..))
    }
}

//
// Iterators
//

/// A borrowed iterator over the elements of the min-max-heap in
/// arbitrary order.
pub struct Iter<'a, T: 'a>(slice::Iter<'a, T>);

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> { self.0.next() }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> { self.0.next_back() }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> { }

impl<'a, T> IntoIterator for &'a MinMaxHeap<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

/// An owning iterator over the elements of the min-max-heap in
/// arbitrary order.
pub struct IntoIter<T>(vec::IntoIter<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> { self.0.next() }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> { self.0.next_back() }
}

impl<T> ExactSizeIterator for IntoIter<T> { }

impl<'a, T> IntoIterator for MinMaxHeap<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter { self.into_iter() }
}

/// A draining iterator over the elements of the min-max-heap in
/// arbitrary order.
pub struct Drain<'a, T: 'a>(vec::Drain<'a, T>);

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> { self.0.next() }
}

impl<'a, T> DoubleEndedIterator for Drain<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> { self.0.next_back() }
}

impl<'a, T> ExactSizeIterator for Drain<'a, T> { }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example() {
        let mut h = MinMaxHeap::new();
        assert!(h.is_empty());

        h.push(5);
        assert!(!h.is_empty());
        assert_eq!(Some(&5), h.peek_min());
        assert_eq!(Some(&5), h.peek_max());

        h.push(7);
        assert_eq!(Some(&5), h.peek_min());
        assert_eq!(Some(&7), h.peek_max());

        h.push(6);
        assert_eq!(Some(&5), h.peek_min());
        assert_eq!(Some(&7), h.peek_max());

        assert_eq!(Some(5), h.pop_min());
        assert_eq!(Some(6), h.pop_min());
        assert_eq!(Some(7), h.pop_min());
        assert_eq!(None, h.pop_min());
    }
}
