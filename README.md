# okapi

An RTOS for Andes RISC-V (N25F), written in Rust.

Okapi follows **Linux's structure** — folder taxonomy, subsystem
decomposition, driver model — and writes **idiomatic Rust** inside it. Where
the two conflict, Rust wins; see [`doc/conventions.md`](doc/conventions.md).

Semantics are RTOS, not Linux: single address space, static tasks, and
PMP-based protection rather than an MMU, since the N25F has no MMU.

## Layout

```
init/        kernel entry, subsystem init order
arch/riscv/  traps, context switch, AndeStar V5 vendor CSRs
kernel/      scheduler, tasks, IRQ management — no MMIO
drivers/     device drivers, grouped by class (irqchip/, later tty/, gpio/)
lib/         intrusive lists, bitmaps, ring buffers
include/     public API surface (re-export facade)
doc/         design docs and bring-up notes
```

## Build

```sh
cargo build
```

Targets `riscv32imac-unknown-none-elf` (soft-float baseline) via
`.cargo/config.toml`.

## Status

Boots to an idle loop. Interrupt controller is scaffolded but not yet
implemented — the PLIC base addresses need confirming against the Andes core
configuration before hardware bring-up.

## Docs

- [`doc/conventions.md`](doc/conventions.md) — folder taxonomy, naming, coding rules
- [`doc/startup.md`](doc/startup.md) — toolchain setup, bring-up journal, interrupt-controller design (§5)
- [`doc/okapi.html`](doc/okapi.html) — roadmap slides
