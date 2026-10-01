//! Andes PLIC driver.
//!
//! This file knows the PLIC's register layout and nothing else — no handler
//! table, no kernel types, no policy (`doc/conventions.md` §7, rule 1).
//! Handler registration and dispatch belong to [`crate::kernel::irq`].
//!
//! The Andes PLIC is **not** the standard RISC-V PLIC: Andes parts ship a
//! `PLIC`/`PLIC_SW` pair, use vendor base addresses, and add a preemptive
//! priority scheme driven by vendor CSRs. That is why Okapi owns this driver
//! rather than depending on a generic PLIC crate — see `doc/startup.md` §5.
//!
//! # Register values are unverified
//!
//! Every constant below is the *standard* PLIC layout used as a scaffold so
//! the tree compiles. All of it needs confirmation against the Andes core
//! configuration report / AndeSight `nds_` headers before hardware bring-up
//! (`doc/startup.md` §5.6).

// Unused until the driver body is implemented; the register map is recorded
// as a unit on purpose rather than added piecemeal.
#![allow(dead_code)]

// TODO(bsp): confirm every address below against the Andes core config.
/// PLIC MMIO base. **Placeholder — not verified for this core.**
const PLIC_BASE: usize = 0xE400_0000;

const PRIORITY: usize = PLIC_BASE; // + 0x0000_0000
const PENDING: usize = PLIC_BASE + 0x0000_1000;
const ENABLE: usize = PLIC_BASE + 0x0000_2000;
const THRESHOLD: usize = PLIC_BASE + 0x0020_0000;
const CLAIM: usize = PLIC_BASE + 0x0020_0004;

// TODO(plic): implement once base addresses are confirmed.
//
//   pub fn init(threshold: u32)
//   pub fn set_priority(irq: u32, prio: u32)
//   pub fn enable(irq: u32)
//   pub fn disable(irq: u32)
//
//   /// Claims the highest-priority pending source.
//   ///
//   /// Reading the claim register atomically returns the id *and* clears
//   /// its pending bit. Id 0 means nothing is pending.
//   pub fn claim() -> Option<u32> {
//       // SAFETY: CLAIM is a valid PLIC MMIO register for this core, mapped
//       // by the SoC and never aliased by a Rust reference. A volatile read
//       // is the architecturally defined way to claim a pending IRQ.
//       let id = unsafe { read_volatile(CLAIM as *const u32) };
//       (id != 0).then_some(id)
//   }
//
//   /// Signals end-of-interrupt. Writes the same register `claim` reads.
//   pub fn complete(irq: u32)
//
// Note: `PLIC_SW` (software/IPI interrupts) is a separate controller with its
// own base address, and may not exist in this core configuration. Deferred
// until multicore — doc/conventions.md §9.
