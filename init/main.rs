//! Okapi kernel entry point.
//!
//! `riscv-rt` hands control here once the machine is safely initialized
//! (stack set, `.bss` cleared, `.data` copied, trap vector installed).
//! Everything reachable from [`main`] is Okapi's own code.
//!
//! See `doc/conventions.md` for the layering rules, and `doc/startup.md` §5
//! for the interrupt-controller design.

#![no_std]
#![no_main]

use panic_halt as _;
use riscv_rt::entry;

// The Linux-style tree lives at the repo root rather than under `src/`, so
// each top-level subsystem is attached explicitly. Modules nested below
// these resolve normally. See doc/conventions.md §5.
#[path = "../arch/mod.rs"]
mod arch;
#[path = "../drivers/mod.rs"]
mod drivers;
#[path = "../kernel/mod.rs"]
mod kernel;
#[path = "../lib/mod.rs"]
mod lib;

/// Public API surface — the only items application code should depend on.
#[path = "../include/okapi/mod.rs"]
pub mod api;

/// Kernel entry. Owns subsystem initialization order.
#[entry]
fn main() -> ! {
    // TODO(irq): kernel::irq::init() once the PLIC base address is
    // confirmed against the BSP (doc/startup.md §5.6).
    // TODO(sched): kernel::sched::init() and hand off to the scheduler.

    // Idle loop. `wfi` parks the core until the next interrupt instead of
    // spinning; once the scheduler exists, control never reaches here.
    loop {
        riscv::asm::wfi();
    }
}
