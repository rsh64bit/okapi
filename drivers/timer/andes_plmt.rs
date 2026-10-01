//! Andes PLMT — platform-level machine timer.
//!
//! Drives the `MachineTimer` core interrupt (cause 7), which is the scheduler
//! tick. Andes calls this block PLMT rather than CLINT (`doc/startup.md`
//! §2.4, §7.6).
//!
//! This file knows the PLMT register layout and nothing else — no scheduler
//! policy, no handler table (`doc/conventions.md` §7, rule 1). The handler
//! itself belongs in `arch/riscv/trap.rs`; the tick policy belongs in
//! `kernel/sched`.
//!
//! # Why this is the first interrupt to bring up
//!
//! One source, no controller to demultiplex, and no dependency on the PLIC
//! base addresses still outstanding (`doc/startup.md` §5.6). It exercises the
//! whole trap path — `mtvec` install, trap entry, cause dispatch, and the
//! `mie`/`mstatus` enable sequence — in isolation. See §7.7.
//!
//! # Two hazards to respect when implementing
//!
//! 1. **Level-triggered, no acknowledge register.** `mip.MTIP` stays asserted
//!    while `mtime >= mtimecmp`. Rewriting `mtimecmp` *is* the
//!    acknowledgement — omit it and the interrupt re-fires forever, which
//!    presents as a hung board, not as an interrupt storm (§7.4).
//! 2. **`mtimecmp` is 64-bit on a 32-bit core.** It cannot be written
//!    atomically; the safe sequence is low := all-ones, then high, then the
//!    real low (§7.5).
//!
//! # Unverified
//!
//! Nothing below is confirmed for this core. The PLMT base address, whether
//! `mtime`/`mtimecmp` are memory-mapped or CSR-accessed, and the timebase
//! frequency all come from the Andes core configuration report, not from the
//! RISC-V specification (`doc/startup.md` §7.7, §7.8).

// TODO(bsp): confirm the PLMT base address and register offsets against the
// Andes core configuration report / AndeSight `nds_` headers. Do not copy
// these from the RISC-V spec or a SiFive CLINT map -- they will look
// plausible and be wrong. Keep them as named constants: QEMU's virt machine
// has a standard CLINT at a different address, and the logic can be tested
// there if the addresses stay configurable (§7.9).
//
//   const PLMT_BASE: usize = ...;   // from core config
//   const MTIME:     usize = PLMT_BASE + ...;
//   const MTIMECMP:  usize = PLMT_BASE + ...;

// TODO(bsp): confirm the TIMEBASE frequency -- the rate `mtime` increments
// at. This is NOT the CPU clock (§7.8.1): on most RISC-V designs mtime runs
// from a separate, slower reference. Using the CPU frequency here produces a
// tick that is silently wrong by an integer factor -- no error, no exception,
// the system just runs at the wrong rate.
//
//   // Source: <core config report / BSP macro / measured on <date>>
//   const TIMEBASE_HZ: u64 = ...;
//   const TICK_HZ:     u64 = 1_000;              // desired scheduler tick
//   const TICK_INTERVAL: u64 = TIMEBASE_HZ / TICK_HZ;

// TODO(bsp): establish whether the timebase is fixed, divided from a faster
// clock, or gated off at reset (§7.8.3). If divided, the divider must be
// programmed BEFORE arming the first deadline. If gated, `mtime` reads back
// zero and never advances -- a distinctive symptom worth recognising, since
// it looks like a dead timer rather than a clocking problem.

// TODO(timer): implement once the above are known.
//
//   /// Reads the current 64-bit timer counter.
//   pub fn now() -> u64
//
//   /// Arms the next tick deadline. Must be called from the MachineTimer
//   /// handler -- this is the interrupt acknowledgement (§7.4).
//   pub fn schedule_next()
//
//   /// One-time setup: arm the first deadline. Does NOT unmask the source or
//   /// enable interrupts globally -- `init/` owns ordering
//   /// (doc/conventions.md §7, rule 5).
//   ///
//   /// Should verify the timebase is actually running rather than assume it:
//   /// read `mtime` twice with a delay between and confirm it advanced. If it
//   /// never advances the cause is a gated clock, a misconfigured divider, or
//   /// the wrong base address -- three different fixes, so distinguish them
//   /// here rather than downstream (§7.8.3).
//   pub fn init()
//
// Enable order, for reference (§7.2): arm the first deadline, then set
// mie.MTIE, then mstatus.MIE last.
