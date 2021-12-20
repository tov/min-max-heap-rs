#![doc(html_root_url = "https://docs.rs/min-max-heap/1.3.0")]
//! A double-ended priority queue.
//!
//! A min-max-heap is like a binary heap, but it allows extracting both
//! the minimum and maximum value efficiently. In particular, finding
//! either the minimum or maximum element is *O*(1). A removal of either
//! extremum, or an insertion, is *O*(log *n*).
//!
//! ## Usage
//!
//! It’s [on crates.io](https://crates.io/crates/min-max-heap), so add
//! this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! min-max-heap = "1.3.0"
//! ```
//!
//! This crate supports Rust version 1.41.1 and later.
//!
//! ## References
//!
//! My reference for a min-max heap is
//! [here](http://cglab.ca/~morin/teaching/5408/refs/minmax.pdf). Much
//! of this code is also based on `BinaryHeap` from the standard
//! library.

#![warn(missing_docs)]

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

use std::iter::FromIterator;
use std::{fmt, mem, slice, vec};
use std::ops::{Deref, DerefMut};

mod hole;
mod index;

use self::hole::*;

/// A double-ended priority queue.
///
/// Most operations are *O*(log *n*).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MinMaxHeap<T>(Vec<T>);

impl<T> Default for MinMaxHeap<T> {
    fn default() -> Self {
        MinMaxHeap::new()
    }
}

impl<T> MinMaxHeap<T> {
    /// Creates a new, empty `MinMaxHeap`.
    ///
    /// *O*(1).
    pub fn new() -> Self {
        MinMaxHeap(Vec::new())
    }

    /// Creates a new, empty `MinMaxHeap` with space allocated to hold
    /// `len` elements.
    ///
    /// *O*(n).
    pub fn with_capacity(len: usize) -> Self {
        MinMaxHeap(Vec::with_capacity(len))
    }

    /// The number of elements in the heap.
    ///
    /// *O*(1).
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Is the heap empty?
    ///
    /// *O*(1).
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T: Ord> MinMaxHeap<T> {
    /// Adds an element to the heap.
    ///
    /// Amortized *O*(log *n*); worst-case *O*(*n*) when the backing vector needs to
    /// grow.
    pub fn push(&mut self, element: T) {
        let pos = self.len();
        self.0.push(element);
        // SAFETY: `pos` is the index of the new element
        unsafe {
            self.bubble_up(pos);
        }
    }

    /// Gets a reference to the minimum element, if any.
    ///
    /// *O*(1).
    pub fn peek_min(&self) -> Option<&T> {
        self.0.first()
    }

    /// Returns a mutable reference to the minimum element, if any. Once this reference is dropped,
    /// the heap is adjusted if necessary.
    ///
    /// Note: If the `PeekMinMut` value is leaked, the heap may be in an
    /// inconsistent state.
    ///
    /// *O*(1) for the peek; *O*(log *n*) when the reference is dropped.
    pub fn peek_min_mut(&mut self) -> Option<PeekMinMut<T>> {
        if self.is_empty() {
            None
        } else {
            Some(PeekMinMut {
                heap: self,
                sift: false,
            })
        }
    }

    /// Gets a reference to the maximum element, if any.
    ///
    /// *O*(1).
    pub fn peek_max(&self) -> Option<&T> {
        // SAFETY: `i` is a valid index in `self.0`
        self.find_max().map(|i| unsafe { self.0.get_unchecked(i) })
    }

    /// Returns a mutable reference to the maximum element, if any. Once this reference is dropped,
    /// the heap is adjusted if necessary.
    ///
    /// Note: If the `PeekMaxMut` value is leaked, the heap may be in an
    /// inconsistent state.
    ///
    /// *O*(1) for the peek; *O*(log *n*) when the reference is dropped.
    pub fn peek_max_mut(&mut self) -> Option<PeekMaxMut<T>> {
        self.find_max().map(move |i| PeekMaxMut {
            heap: self,
            max_index: i,
            sift: false,
        })
    }

    fn find_max_slice(slice: &[T]) -> Option<usize> {
        match slice.len() {
            0 => None,
            1 => Some(0),
            2 => Some(1),
            _ => if slice[1] > slice[2] { Some(1) } else { Some(2) },
        }
    }

    fn find_max(&self) -> Option<usize> {
        Self::find_max_slice(&self.0)
    }

    /// Removes the minimum element, if any.
    ///
    /// *O*(log *n*).
    pub fn pop_min(&mut self) -> Option<T> {
        self.0.pop().map(|mut item| {
            if let Some(min) = self.0.first_mut() {
                mem::swap(&mut item, min);
                // SAFETY: `self.0` is not empty
                unsafe {
                    self.trickle_down_min(0);
                }
            }

            item
        })
    }

    /// Removes the maximum element, if any.
    ///
    /// *O*(log *n*).
    pub fn pop_max(&mut self) -> Option<T> {
        self.find_max().map(|max| {
            let mut item = self.0.pop().unwrap();

            if let Some(max_element) = self.0.get_mut(max) {
                mem::swap(&mut item, max_element);
                // SAFETY: `max` is a valid index in `self.0`
                unsafe {
                    self.trickle_down_max(max);
                }
            }

            item
        })
    }

    /// Pushes an element, then pops the minimum element.
    ///
    /// Calling `push_pop_min` is equivalent to calling [`push`]
    /// followed by [`pop_min`], except that it avoids allocation.
    ///
    /// This means that if the element you give it is smaller than
    /// anything already in the heap then `push_pop_min` gives you back
    /// that same element. If the heap is empty then `push_pop_min`
    /// returns its argument and leaves the heap empty.  In order to
    /// always insert the element, even if it would be the new minimum,
    /// see [`replace_min`], which equivalent to [`pop_min`] followed by
    /// [`push`].
    ///
    /// [`push`]:
    ///     <struct.MinMaxHeap.html#method.push>
    ///
    /// [`pop_min`]:
    ///     <struct.MinMaxHeap.html#method.pop_min>
    ///
    /// [`replace_min`]:
    ///     <struct.MinMaxHeap.html#method.replace_min>
    ///
    /// *O*(log *n*).
    pub fn push_pop_min(&mut self, mut element: T) -> T {
        if let Some(mut min) = self.peek_min_mut() {
            if element > *min {
                mem::swap(&mut element, &mut min);
            }
        }
        element
    }

    /// Pushes an element, then pops the maximum element.
    ///
    /// Calling `push_pop_max` is equivalent to calling [`push`]
    /// followed by [`pop_max`], except that it avoids allocation.
    ///
    /// This means that if the element you give it is greater than
    /// anything already in the heap then `push_pop_max` gives you back
    /// that same element. If the heap is empty then `push_pop_max`
    /// returns its argument and leaves the heap empty.  In order to
    /// always insert the element, even if it would be the new maximum,
    /// see [`replace_max`], which equivalent to [`pop_max`] followed by
    /// [`push`].
    ///
    /// [`push`]:
    ///     <struct.MinMaxHeap.html#method.push>
    ///
    /// [`pop_max`]:
    ///     <struct.MinMaxHeap.html#method.pop_max>
    ///
    /// [`replace_max`]:
    ///     <struct.MinMaxHeap.html#method.replace_max>
    ///
    /// *O*(log *n*).
    pub fn push_pop_max(&mut self, mut element: T) -> T {
        if let Some(mut max) = self.peek_max_mut() {
            if element < *max {
                mem::swap(&mut element, &mut max);
            }
        }
        element
    }

    /// Pops the minimum element and pushes a new element, in an
    /// optimized fashion.
    ///
    /// Except for avoiding allocation, calling `replace_min` is
    /// equivalent to calling [`pop_min`] followed by [`push`].
    ///
    /// This means that `replace_min` will always leaves the element you
    /// give it in the heap and gives you back the *old* minimum
    /// element; it never returns the element you just gave it. If the
    /// heap is empty there is no old minimum, so `replace_min` pushes
    /// the element and returns `None`. If you want to get back the
    /// smallest element *including* the one you are about to push, see
    /// [`push_pop_min`].
    ///
    /// [`push`]:
    ///     <struct.MinMaxHeap.html#method.push>
    ///
    /// [`pop_min`]:
    ///     <struct.MinMaxHeap.html#method.pop_min>
    ///
    /// [`push_pop_min`]:
    ///     <struct.MinMaxHeap.html#method.push_pop_min>
    ///
    /// *O*(log *n*).
    pub fn replace_min(&mut self, mut element: T) -> Option<T> {
        if let Some(mut min) = self.peek_min_mut() {
            mem::swap(&mut element, &mut min);
            return Some(element);
        }

        // Heap was empty, so no reordering is necessary
        self.0.push(element);
        None
    }

    /// Pops the maximum element and pushes a new element, in an
    /// optimized fashion.
    ///
    /// Except for avoiding allocation, calling `replace_max` is
    /// equivalent to calling [`pop_max`] followed by [`push`].
    ///
    /// This means that `replace_max` will always leaves the element you
    /// give it in the heap and gives you back the *old* maximum
    /// element; it never returns the element you just gave it. If the
    /// heap is empty there is no old maximum, so `replace_max` pushes
    /// the element and returns `None`. If you want to get back the
    /// largest element *including* the one you are about to push, see
    /// [`push_pop_max`].
    ///
    /// [`push`]:
    ///     <struct.MinMaxHeap.html#method.push>
    ///
    /// [`pop_max`]:
    ///     <struct.MinMaxHeap.html#method.pop_max>
    ///
    /// [`push_pop_max`]:
    ///     <struct.MinMaxHeap.html#method.push_pop_max>
    ///
    /// *O*(log *n*).
    pub fn replace_max(&mut self, mut element: T) -> Option<T> {
        if let Some(mut max) = self.peek_max_mut() {
            // If `element` is the new min, swap it with the current min
            // (unless the min is the same as the max)
            if max.heap.len() > 1 {
                let min = &mut max.heap.0[0];
                if element < *min {
                    mem::swap(&mut element, min);
                }
            }
            mem::swap(&mut element, &mut max);
            return Some(element);
        }

        // Heap was empty, so no reordering is necessary
        self.0.push(element);
        None
    }

    /// Returns an ascending (sorted) vector, reusing the heap’s
    /// storage.
    ///
    /// *O*(*n* log *n*).
    pub fn into_vec_asc(mut self) -> Vec<T> {
        let mut elements = &mut *self.0;
        while elements.len() > 1 {
            let max = Self::find_max_slice(elements).unwrap();
            let (last, elements_rest) = elements.split_last_mut().unwrap();
            elements = elements_rest;
            if let Some(max_element) = elements.get_mut(max) {
                mem::swap(max_element, last);
                // SAFETY: `max < elements.len()`
                unsafe {
                    Self::trickle_down_slice(elements, max);
                }
            }
        }
        self.into_vec()
    }

    /// Returns an descending (sorted) vector, reusing the heap’s
    /// storage.
    ///
    /// *O*(*n* log *n*).
    pub fn into_vec_desc(mut self) -> Vec<T> {
        let mut elements = &mut *self.0;
        while elements.len() > 1 {
            let (last, elements_rest) = elements.split_last_mut().unwrap();
            elements = elements_rest;
            mem::swap(&mut elements[0], last);
            // SAFETY: `elements` is not empty
            unsafe {
                Self::trickle_down_min_slice(elements, 0);
            }
        }
        self.into_vec()
    }

    /// Caller must ensure that `pos` is a valid index in `self.0`.
    #[inline]
    unsafe fn trickle_down_min(&mut self, pos: usize) {
        Self::trickle_down_min_slice(&mut self.0, pos);
    }

    /// Caller must ensure that `pos` is a valid index in `self.0`.
    #[inline]
    unsafe fn trickle_down_max(&mut self, pos: usize) {
        debug_assert!(pos < self.len());
        Hole::new(&mut self.0, pos).trickle_down_max();
    }

    /// Caller must ensure that `pos` is a valid index in `self.0`.
    #[inline]
    unsafe fn trickle_down(&mut self, pos: usize) {
        Self::trickle_down_slice(&mut self.0, pos);
    }

    /// Caller must ensure that `pos` is a valid index in `slice`.
    #[inline]
    unsafe fn trickle_down_min_slice(slice: &mut [T], pos: usize) {
        debug_assert!(pos < slice.len());
        Hole::new(slice, pos).trickle_down_min();
    }

    /// Caller must ensure that `pos` is a valid index in `slice`.
    #[inline]
    unsafe fn trickle_down_slice(slice: &mut [T], pos: usize) {
        debug_assert!(pos < slice.len());
        Hole::new(slice, pos).trickle_down();
    }

    /// Caller must ensure that `pos` is a valid index in `self.0`.
    #[inline]
    unsafe fn bubble_up(&mut self, pos: usize) {
        debug_assert!(pos < self.len());
        Hole::new(&mut self.0, pos).bubble_up();
    }

    fn rebuild(&mut self) {
        for n in (0..(self.len() / 2)).rev() {
            // SAFETY: `n < self.len()`
            unsafe {
                self.trickle_down(n);
            }
        }
    }
}

impl<T> MinMaxHeap<T> {
    /// Drops all items from the heap.
    ///
    /// *O*(*n*)
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// The number of elements the heap can hold without reallocating.
    ///
    /// *O*(1)
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// Reserves the minimum capacity for exactly `additional` more
    /// elements to be inserted in the given `MinMaxHeap`.
    ///
    /// *O*(*n*)
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
    /// *O*(*n*)
    ///
    /// # Panics
    ///
    /// Panics if the new capacity overflows `usize`.
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional)
    }

    /// Discards extra capacity.
    ///
    /// *O*(*n*)
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }

    /// Consumes the `MinMaxHeap` and returns its elements in a vector
    /// in arbitrary order.
    ///
    /// *O*(*n*)
    pub fn into_vec(self) -> Vec<T> {
        self.0
    }

    /// Returns a borrowing iterator over the min-max-heap’s elements in
    /// arbitrary order.
    ///
    /// *O*(1) on creation, and *O*(1) for each `next()` operation.
    pub fn iter(&self) -> Iter<T> {
        Iter(self.0.iter())
    }

    /// Returns a draining iterator over the min-max-heap’s elements in
    /// arbitrary order.
    ///
    /// *O*(1) on creation, and *O*(1) for each `next()` operation.
    pub fn drain(&mut self) -> Drain<T> {
        Drain(self.0.drain(..))
    }

    /// Returns a draining iterator over the min-max-heap’s elements in
    /// ascending (min-first) order.
    ///
    /// *O*(1) on creation, and *O*(log *n*) for each `next()` operation.
    pub fn drain_asc(&mut self) -> DrainAsc<T> {
        DrainAsc(self)
    }

    /// Returns a draining iterator over the min-max-heap’s elements in
    /// descending (max-first) order.
    ///
    /// *O*(1) on creation, and *O*(log *n*) for each `next()` operation.
    pub fn drain_desc(&mut self) -> DrainDesc<T> {
        DrainDesc(self)
    }
}

//
// Iterators
//

/// A borrowed iterator over the elements of the min-max-heap in
/// arbitrary order.
///
/// This type is created with
/// [`MinMaxHeap::iter`](struct.MinMaxHeap.html#method.iter).
pub struct Iter<'a, T: 'a>(slice::Iter<'a, T>);

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> { self.0.next() }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<T> ExactSizeIterator for IntoIter<T> { }

impl<'a, T> IntoIterator for MinMaxHeap<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_iter())
    }
}

/// A draining iterator over the elements of the min-max-heap in
/// arbitrary order.
///
/// This type is created with
/// [`MinMaxHeap::drain`](struct.MinMaxHeap.html#method.drain).
pub struct Drain<'a, T: 'a>(vec::Drain<'a, T>);

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> { self.0.next() }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
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

/// A draining iterator over the elements of the min-max-heap in
/// ascending (min-first) order.
///
/// Note that each `next()` and `next_back()` operation is
/// *O*(log *n*) time, so this currently provides no performance
/// advantage over `pop_min()` and `pop_max()`.
///
/// This type is created with
/// [`MinMaxHeap::drain_asc`](struct.MinMaxHeap.html#method.drain_asc).
#[derive(Debug)]
pub struct DrainAsc<'a, T: 'a>(&'a mut MinMaxHeap<T>);

/// A draining iterator over the elements of the min-max-heap in
/// descending (max-first) order.
///
/// Note that each `next()` and `next_back()` operation is
/// *O*(log *n*) time, so this currently provides no performance
/// advantage over `pop_max()` and `pop_min()`.
///
/// This type is created with
/// [`MinMaxHeap::drain_desc`](struct.MinMaxHeap.html#method.drain_desc).
#[derive(Debug)]
pub struct DrainDesc<'a, T: 'a>(&'a mut MinMaxHeap<T>);

impl<'a, T> Drop for DrainAsc<'a, T> {
    fn drop(&mut self) {
        let _ = (self.0).0.drain(..);
    }
}

impl<'a, T> Drop for DrainDesc<'a, T> {
    fn drop(&mut self) {
        let _ = (self.0).0.drain(..);
    }
}

impl<'a, T: Ord> Iterator for DrainAsc<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.0.pop_min()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a, T: Ord> Iterator for DrainDesc<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.0.pop_max()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a, T: Ord> DoubleEndedIterator for DrainAsc<'a, T> {
    fn next_back(&mut self) -> Option<T> {
        self.0.pop_max()
    }
}

impl<'a, T: Ord> DoubleEndedIterator for DrainDesc<'a, T> {
    fn next_back(&mut self) -> Option<T> {
        self.0.pop_min()
    }
}

impl<'a, T: Ord> ExactSizeIterator for DrainAsc<'a, T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, T: Ord> ExactSizeIterator for DrainDesc<'a, T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

//
// From<Vec<_>>
//

impl<T: Ord> From<Vec<T>> for MinMaxHeap<T> {
    fn from(vec: Vec<T>) -> Self {
        let mut heap = MinMaxHeap(vec);
        heap.rebuild();
        heap
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

/// Structure wrapping a mutable reference to the minimum item on a
/// `MinMaxHeap`.
///
/// This `struct` is created by the [`peek_min_mut`] method on [`MinMaxHeap`]. See
/// its documentation for more.
///
/// [`peek_min_mut`]: struct.MinMaxHeap.html#method.peek_min_mut
/// [`MinMaxHeap`]: struct.MinMaxHeap.html
pub struct PeekMinMut<'a, T: Ord> {
    heap: &'a mut MinMaxHeap<T>,
    sift: bool,
}

impl<T: Ord + fmt::Debug> fmt::Debug for PeekMinMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("PeekMinMut")
         .field(&**self)
         .finish()
    }
}

impl<'a, T: Ord> Drop for PeekMinMut<'a, T> {
    fn drop(&mut self) {
        if self.sift {
            // SAFETY: `heap` is not empty
            unsafe {
                self.heap.trickle_down_min(0);
            }
        }
    }
}

impl<'a, T: Ord> Deref for PeekMinMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        debug_assert!(!self.heap.is_empty());
        // SAFE: PeekMinMut is only instantiated for non-empty heaps
        unsafe { self.heap.0.get_unchecked(0) }
    }
}

impl<'a, T: Ord> DerefMut for PeekMinMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        debug_assert!(!self.heap.is_empty());
        self.sift = true;
        // SAFE: PeekMinMut is only instantiated for non-empty heaps
        unsafe { self.heap.0.get_unchecked_mut(0) }
    }
}

impl<'a, T: Ord> PeekMinMut<'a, T> {
    /// Removes the peeked value from the heap and returns it.
    pub fn pop(mut self) -> T {
        // Sift is unnecessary since pop_min() already reorders heap
        self.sift = false;
        self.heap.pop_min().unwrap()
    }
}

/// Structure wrapping a mutable reference to the maximum item on a
/// `MinMaxHeap`.
///
/// This `struct` is created by the [`peek_max_mut`] method on [`MinMaxHeap`]. See
/// its documentation for more.
///
/// [`peek_max_mut`]: struct.MinMaxHeap.html#method.peek_max_mut
/// [`MinMaxHeap`]: struct.MinMaxHeap.html
pub struct PeekMaxMut<'a, T: Ord> {
    heap: &'a mut MinMaxHeap<T>,
    max_index: usize,
    sift: bool,
}

impl<T: Ord + fmt::Debug> fmt::Debug for PeekMaxMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("PeekMaxMut")
         .field(&**self)
         .finish()
    }
}

impl<'a, T: Ord> Drop for PeekMaxMut<'a, T> {
    fn drop(&mut self) {
        if self.sift {
            // SAFETY: `max_index` is a valid index in `heap`
            let mut hole = unsafe { Hole::new(&mut self.heap.0, self.max_index) };

            if let Some(mut parent) = hole.get_parent() {
                if parent.hole_element() < parent.other_element() {
                   parent.swap_with();
                }
            }

            hole.trickle_down_max();
        }
    }
}

impl<'a, T: Ord> Deref for PeekMaxMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        debug_assert!(self.max_index < self.heap.len());
        // SAFE: PeekMaxMut is only instantiated for non-empty heaps
        unsafe { self.heap.0.get_unchecked(self.max_index) }
    }
}

impl<'a, T: Ord> DerefMut for PeekMaxMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        debug_assert!(self.max_index < self.heap.len());
        self.sift = true;
        // SAFE: PeekMaxMut is only instantiated for non-empty heaps
        unsafe { self.heap.0.get_unchecked_mut(self.max_index) }
    }
}

impl<'a, T: Ord> PeekMaxMut<'a, T> {
    /// Removes the peeked value from the heap and returns it.
    pub fn pop(mut self) -> T {
        // Sift is unnecessary since pop_max() already reorders heap
        self.sift = false;
        self.heap.pop_max().unwrap()
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use super::*;
    use self::rand::seq::SliceRandom;

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
        assert_eq!(Some(7), h.pop_max());
        assert_eq!(Some(6), h.pop_max());
        assert_eq!(None, h.pop_min());
    }

    #[test]
    fn drain_asc() {
        let mut h = MinMaxHeap::from(vec![3, 2, 4, 1]);
        let mut i = h.drain_asc();
        assert_eq!( i.next(), Some(1) );
        assert_eq!( i.next(), Some(2) );
        assert_eq!( i.next(), Some(3) );
        assert_eq!( i.next(), Some(4) );
        assert_eq!( i.next(), None );
    }

    // This test catches a lot:
    #[test]
    fn random_vectors() {
        for i in 0 .. 300 {
            check_heap(&random_heap(i));
        }
    }

    #[test]
    fn from_vector() {
        for i in 0 .. 300 {
            check_heap(&MinMaxHeap::from(random_vec(i)))
        }
    }

    fn check_heap(heap: &MinMaxHeap<usize>) {
        let asc  = iota_asc(heap.len());
        let desc = iota_desc(heap.len());

        assert_eq!(asc, into_vec_asc(heap.clone()));
        assert_eq!(desc, into_vec_desc(heap.clone()));
        assert_eq!(asc, heap.clone().into_vec_asc());
        assert_eq!(desc, heap.clone().into_vec_desc());
    }

    fn random_vec(len: usize) -> Vec<usize> {
        let mut result = (0 .. len).collect::<Vec<_>>();
        result.shuffle(&mut rand::thread_rng());
        result
    }

    fn random_heap(len: usize) -> MinMaxHeap<usize> {
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

    #[test]
    fn replace_min() {
        let mut h = MinMaxHeap::from(vec![1, 2]);
        assert_eq!(Some(1), h.replace_min(0));
        assert_eq!(Some(&0), h.peek_min());
        assert_eq!(Some(&2), h.peek_max());

        assert_eq!(Some(0), h.replace_min(3));
        assert_eq!(Some(&2), h.peek_min());
        assert_eq!(Some(&3), h.peek_max());
    }

    #[test]
    fn replace_min_edge_cases() {
        let mut empty_heap = MinMaxHeap::new();
        assert_eq!(None, empty_heap.replace_min(1));
        assert_eq!(Some(1), empty_heap.pop_min());
        assert_eq!(None, empty_heap.pop_min());

        let mut one_element_heap = MinMaxHeap::from(vec![2]);
        assert_eq!(Some(2), one_element_heap.replace_min(1));
        assert_eq!(Some(1), one_element_heap.pop_min());
        assert_eq!(None, one_element_heap.pop_min());
    }

    #[test]
    fn replace_max() {
        let mut h = MinMaxHeap::from(vec![1, 2]);
        assert_eq!(Some(2), h.replace_max(3));
        assert_eq!(Some(&1), h.peek_min());
        assert_eq!(Some(&3), h.peek_max());

        assert_eq!(Some(3), h.replace_max(0));
        assert_eq!(Some(&0), h.peek_min());
        assert_eq!(Some(&1), h.peek_max());
    }

    #[test]
    fn replace_max_edge_cases() {
        let mut empty_heap = MinMaxHeap::new();
        assert_eq!(None, empty_heap.replace_max(1));
        assert_eq!(Some(1), empty_heap.pop_max());
        assert_eq!(None, empty_heap.pop_max());

        let mut one_element_heap = MinMaxHeap::from(vec![1]);
        assert_eq!(Some(1), one_element_heap.replace_max(2));
        assert_eq!(Some(2), one_element_heap.pop_max());
        assert_eq!(None, one_element_heap.pop_max());
    }

    #[test]
    fn peek_min_mut() {
        let mut h = MinMaxHeap::from(vec![2, 3, 4]);
        *h.peek_min_mut().unwrap() = 1;
        assert_eq!(Some(&1), h.peek_min());
        assert_eq!(Some(&4), h.peek_max());

        *h.peek_min_mut().unwrap() = 8;
        assert_eq!(Some(&3), h.peek_min());
        assert_eq!(Some(&8), h.peek_max());

        assert_eq!(3, h.peek_min_mut().unwrap().pop());
        assert_eq!(Some(&4), h.peek_min());
        assert_eq!(Some(&8), h.peek_max());
    }

    #[test]
    fn peek_max_mut() {
        let mut h = MinMaxHeap::from(vec![1, 2]);
        *h.peek_max_mut().unwrap() = 3;
        assert_eq!(Some(&1), h.peek_min());
        assert_eq!(Some(&3), h.peek_max());

        *h.peek_max_mut().unwrap() = 0;
        assert_eq!(Some(&0), h.peek_min());
        assert_eq!(Some(&1), h.peek_max());

        assert_eq!(1, h.peek_max_mut().unwrap().pop());
        assert_eq!(Some(&0), h.peek_min());
        assert_eq!(Some(&0), h.peek_max());
    }

    #[test]
    fn peek_max_mut_one() {
        let mut h = MinMaxHeap::from(vec![1]);
        {
            let mut max = h.peek_max_mut().unwrap();
            assert_eq!(*max, 1);
            *max = 2;
        }
        assert_eq!(h.peek_max(), Some(&2));
    }

    #[test]
    fn push_pop_max() {
        let mut h = MinMaxHeap::from(vec![1, 2]);
        assert_eq!(3, h.push_pop_max(3));
        assert_eq!(2, h.push_pop_max(0));
        assert_eq!(Some(&0), h.peek_min());
        assert_eq!(Some(&1), h.peek_max());
    }

    #[test]
    fn peek_mut_format() {
        let mut h = MinMaxHeap::from(vec![1, 2, 3]);
        assert_eq!("PeekMinMut(1)", format!("{:?}", h.peek_min_mut().unwrap()));
        assert_eq!("PeekMaxMut(3)", format!("{:?}", h.peek_max_mut().unwrap()));
    }
}
