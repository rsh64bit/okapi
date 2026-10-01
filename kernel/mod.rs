//! Okapi core kernel — hardware-independent.
//!
//! Nothing under `kernel/` may contain MMIO addresses, register offsets, or
//! vendor CSR access. If a file here needs a hardware address, the layering
//! is wrong (`doc/conventions.md` §7, rules 1-3).

pub mod irq;
pub mod sched;
