# min-max-heap: a double-ended priority queue

[![Build Status](https://travis-ci.org/tov/min-max-heap-rs.svg?branch=master)](https://travis-ci.org/tov/min-max-heap-rs)
[![Crates.io](https://img.shields.io/crates/v/min-max-heap.svg?maxAge=2592000)](https://crates.io/crates/min-max-heap)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/license-Apache_2.0-blue.svg)](LICENSE-APACHE)

A min-max-heap is like a binary heap, but it allows extracting both the
minimum and maximum value efficiently. In particular, finding either the
minimum or maximum element is `O(1)`. A removal of either extremum, or
an insertion, is `O(log n)`.

## Usage

It’s [on crates.io](https://crates.io/crates/min-max-heap), so add
this to your `Cargo.toml`:

```toml
[dependencies]
min-max-heap = "1.2.2"
```

This crate supports Rust version 1.24.0 and later.

## References

My reference for a min-max heap is
[here](http://cglab.ca/~morin/teaching/5408/refs/minmax.pdf). Much
of this code is also based on `BinaryHeap` from the standard
library.

