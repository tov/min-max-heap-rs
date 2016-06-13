use std::{mem, ptr};

use super::index::*;

// From std::collections::BinaryHeap:
pub struct Hole<'a, T: 'a> {
    data: &'a mut [T],
    elt: Option<T>,
    pos: usize,
}

#[derive(Copy, Clone, Debug)]
enum Generation { Same, Parent, Grandparent }

impl<'a, T> Hole<'a, T> {
    /// Create a new Hole at index `pos`.
    pub fn new(data: &'a mut [T], pos: usize) -> Self {
        unsafe {
            let elt = ptr::read(&data[pos]);
            Hole {
                data: data,
                elt: Some(elt),
                pos: pos,
            }
        }
    }

    #[inline]
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Return a reference to the element removed
    #[inline]
    pub fn element(&self) -> &T {
        self.elt.as_ref().unwrap()
    }

    /// Return a reference to the element at `index`.
    ///
    /// Panics if the index is out of bounds.
    #[inline]
    pub fn get(&self, index: usize) -> &T {
        assert!(index != self.pos);
        &self.data[index]
    }

    /// Move hole to new location
    ///
    /// Unsafe because index must not equal pos.
    #[inline]
    pub fn move_to(&mut self, index: usize) {
        assert!(index != self.pos);
        let index_ptr: *const _ = &self.data[index];
        let hole_ptr = &mut self.data[self.pos];
        unsafe { ptr::copy_nonoverlapping(index_ptr, hole_ptr, 1); }
        self.pos = index;
    }

    /// Swaps the contents of the hole with another position without
    /// moving the hole.
    #[inline]
    pub fn swap_with_parent(&mut self) {
        assert!(self.pos != 0);
        let parent = self.pos.parent();
        mem::swap(&mut self.data[parent], self.elt.as_mut().unwrap())
    }

    #[inline]
    pub fn has_parent(&self) -> bool {
        self.pos().has_parent()
    }

    #[inline]
    pub fn has_grandparent(&self) -> bool {
        self.pos().has_grandparent()
    }

    #[inline]
    pub fn get_parent(&self) -> &T {
        self.get(self.pos().parent())
    }

    #[inline]
    pub fn get_grandparent(&self) -> &T {
        self.get(self.pos().grandparent())
    }

    #[inline]
    pub fn move_to_parent(&mut self) {
        let dest = self.pos().parent();
        self.move_to(dest);
    }

    #[inline]
    pub fn move_to_grandparent(&mut self) {
        let dest = self.pos().grandparent();
        self.move_to(dest);
    }

    #[inline]
    pub fn on_min_level(&self) -> bool {
        self.pos().is_min_level()
    }

    #[inline]
    fn index_of_best_child_or_grandchild<F>(&self, f: F)
                                            -> (usize, Generation)
            where F: Fn(&T, &T) -> bool {

        let data = &self.data;
        let len  = data.len();

        let mut pos     = self.pos();
        let mut depth   = Generation::Same;
        let mut element = self.element();

        for &(i, gen) in &[(pos.child1(), Generation::Parent),
                           (pos.child2(), Generation::Parent),
                           (pos.grandchild1(), Generation::Grandparent),
                           (pos.grandchild2(), Generation::Grandparent),
                           (pos.grandchild3(), Generation::Grandparent),
                           (pos.grandchild4(), Generation::Grandparent)] {
            if i < len {
                if f(&data[i], element) {
                    pos = i;
                    depth = gen;
                    element = &data[i];
                }
            } else {
                break
            }
        }

        (pos, depth)
    }
}

impl<'a, T: Ord + 'a> Hole<'a, T> {
    #[inline]
    fn index_of_smallest_child_or_grandchild(&self) -> (usize, Generation) {
        self.index_of_best_child_or_grandchild(|a, b| a < b)
    }

    #[inline]
    fn index_of_largest_child_or_grandchild(&self) -> (usize, Generation) {
        self.index_of_best_child_or_grandchild(|a, b| a > b)
    }

    pub fn bubble_up(&mut self) {
        if self.on_min_level() {
            if self.has_parent() && self.element() > self.get_parent() {
                self.move_to_parent();
                self.bubble_up_max();
            } else {
                self.bubble_up_min();
            }
        } else if self.has_parent() && self.element() < self.get_parent() {
            self.move_to_parent();
            self.bubble_up_min();
        } else {
            self.bubble_up_max();
        }
    }

    fn bubble_up_min(&mut self) {
        while self.has_grandparent()
                && self.element() < self.get_grandparent() {
            self.move_to_grandparent()
        }
    }

    fn bubble_up_max(&mut self) {
        while self.has_grandparent()
                && self.element() > self.get_grandparent() {
            self.move_to_grandparent()
        }
    }

    pub fn trickle_down(&mut self) {
        if self.on_min_level() {
            self.trickle_down_min();
        } else {
            self.trickle_down_max();
        }
    }

    pub fn trickle_down_min(&mut self) {
        loop {
            let (m, gen) = self.index_of_smallest_child_or_grandchild();
            match gen {
                Generation::Grandparent => {
                    self.move_to(m);
                    if self.element() > self.get_parent() {
                        self.swap_with_parent();
                    }
                }

                Generation::Parent => {
                    self.move_to(m);
                    return;
                }

                Generation::Same => {
                    return;
                }
            }
        }
    }

    pub fn trickle_down_max(&mut self) {
        loop {
            let (m, gen) = self.index_of_largest_child_or_grandchild();
            match gen {
                Generation::Grandparent => {
                    self.move_to(m);
                    if self.element() < self.get_parent() {
                        self.swap_with_parent();
                    }
                }

                Generation::Parent => {
                    self.move_to(m);
                    return;
                }

                Generation::Same => {
                    return;
                }
            }
        }
    }
}

impl<'a, T> Drop for Hole<'a, T> {
    fn drop(&mut self) {
        unsafe {
            ptr::write(&mut self.data[self.pos], self.elt.take().unwrap());
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn hole() {
        let mut v = vec![0, 1, 2, 3, 4, 5];
        {
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
