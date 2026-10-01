//! Trap entry glue.
//!
//! Bridges `riscv-rt`'s core-interrupt dispatch to Okapi's IRQ layer, and
//! owns the nesting discipline (save `mepc`/`mstatus`/`mcause`, re-enable,
//! claim, complete, restore).
//!
//! `riscv-rt` routes *core* causes and stops at "an external interrupt
//! arrived". Demultiplexing the device behind `MachineExternal` is the
//! PLIC's job, reached via [`crate::kernel::irq`] — see `doc/startup.md`
//! §5.1 and §5.5.

// TODO(irq): once the PLIC base address is confirmed (doc/startup.md §5.6),
// install the MachineExternal handler:
//
//     #[riscv_rt::core_interrupt(CoreInterrupt::MachineExternal)]
//     fn machine_external() {
//         // Loop, not a single claim: several sources can be pending behind
//         // one core line, and the PLIC deasserts only once all are
//         // completed. A single claim would silently drop the rest.
//         while let Some(irq) = plic::claim() {
//             crate::kernel::irq::dispatch(irq);
//             plic::complete(irq);
//         }
//     }
