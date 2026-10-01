//! IRQ management — registration, masking, dispatch.
//!
//! This is the interface the scheduler and device drivers use. It is
//! deliberately free of any mention of the PLIC: the controller is reached
//! through a driver in `drivers/irqchip/`, so retargeting to a different
//! Andes part changes one file (`doc/conventions.md` §7, rule 2).

/// A PLIC source identifier.
///
/// Source `0` is reserved by the RISC-V PLIC specification to mean "no
/// interrupt pending", so a valid IRQ is always non-zero.
pub type IrqNumber = u32;

/// An interrupt service routine.
pub type IrqHandler = fn();

/// Largest supported source id.
///
/// Sizes the handler table. The real maximum comes from the core
/// configuration — see `doc/startup.md` §5.6.
// TODO(bsp): confirm against the Andes core configuration report.
pub const MAX_IRQ: IrqNumber = 64;

// TODO(irq): implement once the irqchip driver lands.
//
//   pub fn init()
//   pub fn register(irq: IrqNumber, handler: IrqHandler)
//   pub fn mask(irq: IrqNumber)
//   pub fn unmask(irq: IrqNumber)
//   pub(crate) fn dispatch(irq: IrqNumber)
//
// `dispatch` is crate-internal: it is called from arch trap code, never by
// drivers.
