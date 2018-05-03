#![feature(test)]

extern crate min_max_heap;
extern crate test;

use min_max_heap::MinMaxHeap;
use test::Bencher;

#[bench]
fn push_seq(b: &mut Bencher) {
    b.iter(|| {
        let n = 1000;
        let mut heap = MinMaxHeap::with_capacity(n);
        for i in 0..n {
            heap.push(i);
        }
    });
}

#[bench]
fn pop_max_seq(b: &mut Bencher) {
    b.iter(|| {
        let n = 1000;
        let mut heap: MinMaxHeap<_> = (0..n).collect();
        for _ in 0..n {
            heap.pop_max();
        }
    });
}

#[bench]
fn pop_min_seq(b: &mut Bencher) {
    b.iter(|| {
        let n = 1000;
        let mut heap: MinMaxHeap<_> = (0..n).collect();
        for _ in 0..n {
            heap.pop_min();
        }
    });
}

