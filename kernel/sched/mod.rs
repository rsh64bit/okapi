//! Scheduler and task abstraction.
//!
//! Not yet implemented. Depends on the IRQ layer for timer-tick preemption,
//! which is why the interrupt controller is being built first
//! (`doc/startup.md` §5.1).

// TODO(sched): Task struct with its own stack and saved context; context
// switch in naked asm under arch/riscv/; preemptive fixed-priority
// scheduling driven off the machine timer.
