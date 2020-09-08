use std::{mem, ptr};
use std::mem::ManuallyDrop;

use super::index::*;

// From std::collections::BinaryHeap:
pub struct Hole<'a, T: 'a> {
    data: &'a mut [T],
    elt: ManuallyDrop<T>,
    pos: usize,
}

#[derive(Copy, Clone, Debug)]
enum Generation { Same, Parent, Grandparent }

impl<'a, T> Hole<'a, T> {
    /// Create a new Hole at index `pos`.
    ///
    /// Caller must ensure that `pos` is a valid index in `data`.
    pub unsafe fn new(data: &'a mut [T], pos: usize) -> Self {
        debug_assert!(pos < data.len());
        let elt = ptr::read(data.get_unchecked(pos));
        Hole { data, elt: ManuallyDrop::new(elt), pos }
    }

    #[inline]
    fn pos(&self) -> usize {
        self.pos
    }

    /// Return a reference to the element removed
    #[inline]
    pub fn element(&self) -> &T {
        &self.elt
    }

    /// Return a reference to the element at `index`.
    ///
    /// Caller must ensure that `index` is a valid index in `data`
    /// and not equal to `pos`.
    #[inline]
    unsafe fn get(&self, index: usize) -> &T {
        debug_assert!(index != self.pos);
        debug_assert!(index < self.data.len());
        self.data.get_unchecked(index)
    }

    /// Move hole to new location
    ///
    /// Caller must ensure that `index` is a valid index in `data`
    /// and not equal to `pos`.
    #[inline]
    unsafe fn move_to(&mut self, index: usize) {
        debug_assert!(index != self.pos);
        debug_assert!(index < self.data.len());
        let elt = ptr::read(self.data.get_unchecked(index));
        ptr::write(self.data.get_unchecked_mut(self.pos), elt);
        self.pos = index;
    }

    /// Swaps the contents of the hole with its parent without
    /// moving the hole.
    ///
    /// Caller must ensure that the hole has a parent.
    #[inline]
    pub unsafe fn swap_with_parent(&mut self) {
        debug_assert!(self.pos().has_parent());
        let parent = self.data.get_unchecked_mut(self.pos().parent());
        mem::swap(parent, &mut self.elt);
    }

    /// Caller must ensure that the hole has a parent.
    #[inline]
    unsafe fn get_parent_unchecked(&self) -> &T {
        debug_assert!(self.pos().has_parent());
        self.get(self.pos().parent())
    }

    #[inline]
    pub fn get_parent(&self) -> Option<&T> {
        if self.pos().has_parent() {
            // SAFETY: parent is a valid index and not equal to `pos`
            Some(unsafe { self.get_parent_unchecked() })
        } else {
            None
        }
    }

    #[inline]
    fn get_grandparent(&self) -> Option<&T> {
        if self.pos().has_grandparent() {
            // SAFETY: grandparent is a valid index and not equal to `pos`
            Some(unsafe { self.get(self.pos().grandparent()) })
        } else {
            None
        }
    }

    /// Caller must ensure that the hole has a parent.
    #[inline]
    unsafe fn move_to_parent(&mut self) {
        debug_assert!(self.pos().has_parent());
        self.move_to(self.pos().parent());
    }

    /// Caller must ensure that the hole has a grandparent.
    #[inline]
    unsafe fn move_to_grandparent(&mut self) {
        debug_assert!(self.pos().has_grandparent());
        self.move_to(self.pos().grandparent());
    }

    #[inline]
    fn on_min_level(&self) -> bool {
        self.pos().is_min_level()
    }

    /// Caller must ensure that `len <= data.len()`.
    #[inline]
    unsafe fn index_of_best_child_or_grandchild<F>(&self, len: usize, f: F)
        -> (usize, Generation)
    where
        F: Fn(&T, &T) -> bool,
    {
        debug_assert!(len <= self.data.len());
        let data = &*self.data;
        let here = self.pos();

        let mut pos     = here;
        let mut depth   = Generation::Same;
        let mut element = self.element();

        {
            let mut check = |i, gen| {
                if i < len {
                    // SAFETY: `i < len <= data.len()`
                    let candidate = data.get_unchecked(i);
                    if f(candidate, element) {
                        pos = i;
                        depth = gen;
                        element = candidate;
                    }

                    true
                } else {
                    false
                }
            };

            let _ =
                check(here.child1(), Generation::Parent) &&
                check(here.child2(), Generation::Parent) &&
                check(here.grandchild1(), Generation::Grandparent) &&
                check(here.grandchild2(), Generation::Grandparent) &&
                check(here.grandchild3(), Generation::Grandparent) &&
                check(here.grandchild4(), Generation::Grandparent);
        }

        (pos, depth)
    }

    /// Caller must ensure that `len <= data.len()`.
    unsafe fn trickle_down_best_len<F>(&mut self, len: usize, f: F)
    where
        F: Fn(&T, &T) -> bool,
    {
        debug_assert!(len <= self.data.len());
        loop {
            let (min, gen) = self.index_of_best_child_or_grandchild(len, &f);
            match gen {
                Generation::Grandparent => {
                    self.move_to(min);
                    // SAFETY: element has a parent
                    let parent = self.get_parent_unchecked();
                    if f(parent, self.element()) {
                        self.swap_with_parent();
                    }
                }

                Generation::Parent => {
                    self.move_to(min);
                    return;
                }

                Generation::Same => {
                    return;
                }
            }
        }
    }
}

impl<'a, T: Ord> Hole<'a, T> {
    pub fn bubble_up(&mut self) {
        if self.on_min_level() {
            match self.get_parent() {
                Some(parent) if self.element() > parent => {
                    // SAFETY: element has a parent
                    unsafe {
                        self.move_to_parent();
                    }
                    self.bubble_up_max();
                }
                _ => self.bubble_up_min(),
            }
        } else {
            match self.get_parent() {
                Some(parent) if self.element() < parent => {
                    // SAFETY: element has a parent
                    unsafe {
                        self.move_to_parent();
                    }
                    self.bubble_up_min();
                }
                _ => self.bubble_up_max(),
            }
        }
    }

    fn bubble_up_grandparent<F>(&mut self, f: F) where F: Fn(&T, &T) -> bool {
        while let Some(grandparent) = self.get_grandparent() {
            if f(self.element(), grandparent) {
                // SAFETY: element has a grandparent
                unsafe {
                    self.move_to_grandparent();
                }
            } else {
                return;
            }
        }
    }

    fn bubble_up_min(&mut self) {
        self.bubble_up_grandparent(PartialOrd::lt);
    }

    fn bubble_up_max(&mut self) {
        self.bubble_up_grandparent(PartialOrd::gt);
    }

    pub fn trickle_down(&mut self) {
        // SAFETY: `data.len() <= data.len()`
        unsafe {
            self.trickle_down_len(self.data.len());
        }
    }

    pub fn trickle_down_min(&mut self) {
        // SAFETY: `data.len() <= data.len()`
        unsafe {
            self.trickle_down_min_len(self.data.len());
        }
    }

    pub fn trickle_down_max(&mut self) {
        // SAFETY: `data.len() <= data.len()`
        unsafe {
            self.trickle_down_max_len(self.data.len());
        }
    }

    /// Caller must ensure that `len <= data.len()`.
    pub unsafe fn trickle_down_len(&mut self, len: usize) {
        debug_assert!(len <= self.data.len());
        if self.on_min_level() {
            self.trickle_down_min_len(len);
        } else {
            self.trickle_down_max_len(len);
        }
    }

    /// Caller must ensure that `len <= data.len()`.
    pub unsafe fn trickle_down_min_len(&mut self, len: usize) {
        debug_assert!(len <= self.data.len());
        self.trickle_down_best_len(len, PartialOrd::lt);
    }

    /// Caller must ensure that `len <= data.len()`.
    pub unsafe fn trickle_down_max_len(&mut self, len: usize) {
        debug_assert!(len <= self.data.len());
        self.trickle_down_best_len(len, PartialOrd::gt);
    }
}

impl<'a, T> Drop for Hole<'a, T> {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: `elt` is being moved into the hole
            let elt = ptr::read(&*self.elt);
            // SAFETY: `pos` is a valid index in `data` and is a hole
            ptr::write(self.data.get_unchecked_mut(self.pos()), elt);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn hole() {
        let mut v = vec![0, 1, 2, 3, 4, 5];
        unsafe {
            let mut h = Hole::new(&mut v, 1);

            assert_eq!(1, h.pos());
            assert_eq!(1, *h.element());
            assert_eq!(2, *h.get(2));

            h.move_to(4);

            assert_eq!(4, h.pos());
            assert_eq!(1, *h.element());
            assert_eq!(4, *h.get(1));
            assert_eq!(2, *h.get(2));
        }

        assert_eq!(vec![0, 4, 2, 3, 1, 5], v);
    }
}
