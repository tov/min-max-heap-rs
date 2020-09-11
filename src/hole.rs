use std::{mem, ptr};
use std::mem::ManuallyDrop;

use super::index::*;

// From std::collections::BinaryHeap:
pub struct Hole<'a, T: 'a> {
    data: &'a mut [T],
    elt: ManuallyDrop<T>,
    pos: usize,
}

enum Generation {
    Child,
    Grandchild,
}

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

    #[inline]
    pub fn get_parent(&mut self) -> Option<HoleSwap<'a, '_, T>> {
        if self.pos().has_parent() {
            // SAFETY: parent is a valid index and not equal to `pos`
            Some(unsafe { HoleSwap::new(self, self.pos().parent()) })
        } else {
            None
        }
    }

    #[inline]
    fn get_grandparent(&mut self) -> Option<HoleSwap<'a, '_, T>> {
        if self.pos().has_grandparent() {
            // SAFETY: grandparent is a valid index and not equal to `pos`
            Some(unsafe { HoleSwap::new(self, self.pos().grandparent()) })
        } else {
            None
        }
    }

    #[inline]
    fn on_min_level(&self) -> bool {
        self.pos().is_min_level()
    }

    /// Caller must ensure that `len <= data.len()`.
    #[inline]
    unsafe fn best_child_or_grandchild<F>(&mut self, len: usize, f: F)
        -> Option<(HoleSwap<'a, '_, T>, Generation)>
    where
        F: Fn(&T, &T) -> bool,
    {
        debug_assert!(len <= self.data.len());
        let data = &*self.data;
        let here = self.pos();

        let mut best    = None;
        let mut element = self.element();

        {
            let mut check = |index, generation| {
                if index < len {
                    // SAFETY: `i < len <= data.len()`
                    let candidate = data.get_unchecked(index);
                    if f(candidate, element) {
                        best = Some((index, generation));
                        element = candidate;
                    }

                    true
                } else {
                    false
                }
            };

            let _ =
                check(here.child1(), Generation::Child) &&
                check(here.child2(), Generation::Child) &&
                check(here.grandchild1(), Generation::Grandchild) &&
                check(here.grandchild2(), Generation::Grandchild) &&
                check(here.grandchild3(), Generation::Grandchild) &&
                check(here.grandchild4(), Generation::Grandchild);
        }

        match best {
            Some((index, generation)) => {
                Some((HoleSwap::new(self, index), generation))
            }
            None => None,
        }
    }

    /// Caller must ensure that `len <= data.len()`.
    unsafe fn trickle_down_best_len<F>(&mut self, len: usize, f: F)
    where
        F: Fn(&T, &T) -> bool,
    {
        debug_assert!(len <= self.data.len());
        while let Some((best, generation)) = self.best_child_or_grandchild(len, &f) {
            best.move_to();
            match generation {
                Generation::Grandchild => {
                    let mut parent = HoleSwap::new(self, self.pos().parent());
                    if f(parent.other_element(), parent.hole_element()) {
                        parent.swap_with();
                    }
                }
                Generation::Child => return,
            }
        }
    }
}

impl<'a, T: Ord> Hole<'a, T> {
    pub fn bubble_up(&mut self) {
        if self.on_min_level() {
            match self.get_parent() {
                Some(parent) if parent.hole_element() > parent.other_element() => {
                    parent.move_to();
                    self.bubble_up_max();
                }
                _ => self.bubble_up_min(),
            }
        } else {
            match self.get_parent() {
                Some(parent) if parent.hole_element() < parent.other_element() => {
                    parent.move_to();
                    self.bubble_up_min();
                }
                _ => self.bubble_up_max(),
            }
        }
    }

    fn bubble_up_grandparent<F>(&mut self, f: F) where F: Fn(&T, &T) -> bool {
        while let Some(grandparent) = self.get_grandparent() {
            if f(grandparent.hole_element(), grandparent.other_element()) {
                grandparent.move_to();
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

/// A hole, along with a potential new position to move it to.
/// This replaces some unsafe blocks with safety requirements on the constructor.
pub struct HoleSwap<'a, 'b, T> {
    hole: &'b mut Hole<'a, T>,
    index: usize,
}

impl<'a, 'b, T> HoleSwap<'a, 'b, T> {
    /// Caller must ensure that `index` is a valid index in `data`
    /// and not equal to `pos`.
    unsafe fn new(hole: &'b mut Hole<'a, T>, index: usize) -> Self {
        debug_assert!(index != hole.pos());
        debug_assert!(index < hole.data.len());
        HoleSwap { hole, index }
    }

    /// The element currently pulled out of the hole.
    pub fn hole_element(&self) -> &T {
        self.hole.element()
    }

    /// The element at the index to potentially move to.
    pub fn other_element(&self) -> &T {
        // SAFETY: `index` is a valid index in `data` and not a hole
        unsafe { self.hole.data.get_unchecked(self.index) }
    }

    /// Move `other_element()` into the current hole
    /// and move the hole to where `other_element()` was.
    /// This invalidates the `HoleSwap`.
    pub fn move_to(self) {
        unsafe {
            // SAFETY: `index` is a valid index in `data` and not a hole
            let elt = ptr::read(self.other_element());
            // SAFETY: `pos` is a valid index in `data` and a hole
            ptr::write(self.hole.data.get_unchecked_mut(self.hole.pos()), elt);
        }
        self.hole.pos = self.index;
    }

    /// Swaps `hole_element()` with `other_element()`, without moving the hole
    pub fn swap_with(&mut self) {
        // SAFETY: `index` is a valid index in `data` and not a hole
        let other_element = unsafe { self.hole.data.get_unchecked_mut(self.index) };
        mem::swap(other_element, &mut self.hole.elt);
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
            assert_eq!(2, h.data[2]);

            HoleSwap::new(&mut h, 4).move_to();

            assert_eq!(4, h.pos());
            assert_eq!(1, *h.element());
            assert_eq!(4, h.data[1]);
            assert_eq!(2, h.data[2]);
        }

        assert_eq!(vec![0, 4, 2, 3, 1, 5], v);
    }
}
