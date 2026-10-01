//! AndeStar V5 vendor CSRs.
//!
//! These are Andes-specific and absent from the `riscv` crate, so they are
//! reached with raw `csrr`/`csrw`. Confining them here keeps the vendor
//! dependency out of `kernel/` and `drivers/` entirely
//! (`doc/conventions.md` §7, rule 3).
//!
//! CSR numbers and field layouts must be confirmed against the AndeStar V5
//! manual for this core configuration before use — see `doc/startup.md` §5.6.

// Unused until the accessors below are implemented; the CSR numbers are
// recorded now because they are the hard-to-find part.
#![allow(dead_code)]

/// `mxstatus` — AndeStar extension status/control.
pub const CSR_MXSTATUS: u16 = 0x7c4;

/// `mmisc_ctl` — misc. control, including interrupt behaviour.
pub const CSR_MMISC_CTL: u16 = 0x7d0;

// TODO(bsp): confirm CSR numbers above, then implement typed accessors for
// the fields Okapi actually needs (vectored-plus behaviour, preemptive
// priority). Raw read/write helpers come first; see doc/startup.md §5.1 for
// why nesting discipline is owned here rather than in the PLIC driver.
