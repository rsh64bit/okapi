//! Okapi public API surface.
//!
//! Linux uses `include/` because C needs headers. Rust does not, so this
//! directory takes the Rust-equivalent job: a re-export facade defining what
//! application code and out-of-tree drivers may depend on
//! (`doc/conventions.md` §1.1).
//!
//! Anything not re-exported here is kernel-internal and may change without
//! notice.

pub use crate::kernel::irq::{IrqHandler, IrqNumber, MAX_IRQ};
