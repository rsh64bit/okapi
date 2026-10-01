//! Device drivers, grouped by device *class*.
//!
//! Following Linux, the directory names the class and the file names the
//! vendor — `irqchip/andes_plic.rs`, not `andes/plic.rs`. Future classes:
//! `tty/`, `gpio/`.

pub mod irqchip;
pub mod timer;
