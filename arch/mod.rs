//! Architecture-specific code.
//!
//! Everything that changes when the ISA or core changes lives under here.
//! A port to a non-Andes RISC-V core, or to another ISA entirely, replaces
//! this subtree and nothing else.

pub mod riscv;
