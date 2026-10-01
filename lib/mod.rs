//! Kernel-internal data structures.
//!
//! Generic containers with no hardware knowledge and no policy — the Rust
//! equivalent of Linux's `lib/`. Allocation-free, suitable for use with
//! interrupts disabled.

// TODO(lib): intrusive linked list (for run queues and wait queues), bitmap
// (for priority lookup), fixed-capacity ring buffer (for message queues).
