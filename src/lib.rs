//! A double-ended priority queue.
//!
//! A min-max-heap is like a binary heap, but it allows extracting both
//! the minimum and maximum value efficiently. In particular, finding
//! either the minimum or maximum element is `O(1)`. A removal of either
//! extremum, or an insertion, is `O(log n)`.
//!
//! ## Usage
//!
//! It’s [on crates.io](https://crates.io/crates/min-max-heap), so add
//! this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! min-max-heap = "*"
//! ```
//!
//! And add this to your crate root:
//! ```rust
//! extern crate min_max_heap;
//! ```
//!
//! ## References
//!
//! My reference for a min-max heap is
//! [here](http://cglab.ca/~morin/teaching/5408/refs/minmax.pdf). Much
//! of this code is also based on `BinaryHeap` from the standard
//! library.


#![warn(missing_docs)]

use std::iter::FromIterator;
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

    /// Pushes an element, then pops the minimum element.
    ///
    /// Unlike a push followed by a pop, this combined operation will
    /// not allocate.
    pub fn push_pop_min(&mut self, mut element: T) -> T {
        if self.is_empty() { return element; }

        if &element < &self.0[0] { return element; }

        mem::swap(&mut element, &mut self.0[0]);
        Hole::new(&mut self.0, 0).trickle_down_min();
        element
    }

    /// Pushes an element, then pops the maximum element in an optimized
    /// fashion.
    ///
    /// Unlike a push followed by a pop, this combined operation will
    /// not allocate.
    pub fn push_pop_max(&mut self, mut element: T) -> T {
        if let Some(i) = self.find_max() {
            if &element > &self.0[i] { return element }

            mem::swap(&mut element, &mut self.0[i]);
            Hole::new(&mut self.0, i).trickle_down_max();
            element
        } else { element }
    }

    /// Pops the minimum, then pushes an element in an optimized
    /// fashion.
    pub fn replace_min(&mut self, mut element: T) -> Option<T> {
        if self.is_empty() {
            self.push(element);
            return None;
        }

        mem::swap(&mut element, &mut self.0[0]);
        Hole::new(&mut self.0, 0).trickle_down_min();
        Some(element)
    }

    /// Pops the maximum, then pushes an element in an optimized
    /// fashion.
    pub fn replace_max(&mut self, mut element: T) -> Option<T> {
        if let Some(i) = self.find_max() {
            mem::swap(&mut element, &mut self.0[i]);
            Hole::new(&mut self.0, i).trickle_down_max();
            Some(element)
        } else {
            self.push(element);
            None
        }
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

impl<T: Ord> FromIterator<T> for MinMaxHeap<T> {
    fn from_iter<I>(iter: I) -> Self
            where I: IntoIterator<Item = T> {
        let mut result = MinMaxHeap::new();
        result.extend(iter);
        result
    }
}

//
// Extend
//

impl<T: Ord> Extend<T> for MinMaxHeap<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for elem in iter {
            self.push(elem)
        }
    }
}

impl<'a, T: Ord + Clone + 'a> Extend<&'a T> for MinMaxHeap<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        for elem in iter {
            self.push(elem.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use super::*;
    use self::rand::{Rng,StdRng};

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

    // This test catches a lot:
    #[test]
    fn random_vectors() {
        for i in 0 .. 300 {
            check_size_min(i);
            check_size_max(i);
        }
    }

    fn check_size_min(len: usize) {
        let heap = random_heap(len);
        let vec  = into_vec_asc(heap);
        assert_eq!(iota_asc(len), vec);
    }

    fn check_size_max(len: usize) {
        let heap = random_heap(len);
        let vec  = into_vec_desc(heap);
        assert_eq!(iota_desc(len), vec);
    }

    fn random_vec(len: usize) -> Vec<usize> {
        let mut result = (0 .. len).collect::<Vec<_>>();
        StdRng::new().unwrap().shuffle(&mut result);
        result
    }

    fn random_heap(len: usize) -> MinMaxHeap<usize> {
        use std::iter::FromIterator;
        MinMaxHeap::from_iter(random_vec(len))
    }

    fn into_vec_asc(mut heap: MinMaxHeap<usize>) -> Vec<usize> {
        let mut result = Vec::with_capacity(heap.len());
        while let Some(elem) = heap.pop_min() {
            result.push(elem)
        }
        result
    }

    fn into_vec_desc(mut heap: MinMaxHeap<usize>) -> Vec<usize> {
        let mut result = Vec::with_capacity(heap.len());
        while let Some(elem) = heap.pop_max() {
            result.push(elem)
        }
        result
    }

    fn iota_asc(len: usize) -> Vec<usize> {
        (0 .. len).collect()
    }

    fn iota_desc(len: usize) -> Vec<usize> {
        let mut result = (0 .. len).collect::<Vec<_>>();
        result.reverse();
        result
    }
}
