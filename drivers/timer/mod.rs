//! Timer drivers.
//!
//! Separate from `irqchip/` on purpose: an interrupt controller demultiplexes
//! many sources, whereas a timer is a timer that happens to raise an
//! interrupt. Linux draws the same line between `drivers/clocksource/` and
//! `drivers/irqchip/`. See `doc/conventions.md` §1 and `doc/startup.md` §8.11.

pub mod andes_plmt;
