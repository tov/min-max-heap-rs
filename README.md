# min-max-heap: a double-ended priority queue

[![Build Status]][CI]
[![Crates.io]][crate]
[![License: MIT]](LICENSE-MIT)
[![License: Apache 2.0]](LICENSE-APACHE)

[Build Status]:
  <https://github.com/tov/min-max-heap-rs/actions/workflows/ci.yml/badge.svg>  

[CI]:
  <https://github.com/tov/min-max-heap-rs/actions>

[Crates.io]:
  <https://img.shields.io/crates/v/min-max-heap.svg?maxAge=2592000>

[crate]:
  <https://crates.io/crates/min-max-heap>

[License: MIT]:
  <https://img.shields.io/badge/license-MIT-blue.svg>

[License: Apache 2.0]:
  <https://img.shields.io/badge/license-Apache_2.0-blue.svg>

A min-max-heap is like a binary heap, but it allows extracting both the
minimum and maximum value efficiently. In particular, finding either the
minimum or maximum element is worst-case *O*(1) time. A removal of either
extreme, or an insertion, is worst-case *O*(log *n*) time.

## Usage

It’s [on crates.io][crate], so add this to your `Cargo.toml`:

```toml
[dependencies]
min-max-heap = "1.3.0"
```

This crate supports Rust version 1.46 and later.

## References

  - M. D. Atkinson, J.-R. Sack, N. Santoro, and T. Strothot.
    Ian Munro (ed.). “[Min-Max Heaps and Generalized Priority
    Queues][Atkinson86].” In *Communications of the ACM.* 29(10):
    996–1000, June 1996. \[[pdf][Atkinson86]\]

  - The Rust Standard Library’s [`BinaryHeap`] API and
    implementation. \[[src][binary_heap.rs]\]

[Atkinson86]:
  <http://akira.ruc.dk/~keld/teaching/algoritmedesign_f03/Artikler/02/Atkinson86.pdf>

[`BinaryHeap`]:
  std::collections::binary_heap::BinaryHeap

[binary_heap.rs]:
  <https://doc.rust-lang.org/stable/src/alloc/collections/binary_heap.rs.html>
