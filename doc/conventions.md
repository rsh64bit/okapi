# Okapi Coding Conventions

Okapi borrows **Linux's structure** — folder taxonomy, subsystem
decomposition, driver model — and writes **idiomatic Rust** inside it.

## Contents

- [0. Precedence rule](#0-precedence-rule)
- [1. Directory layout](#1-directory-layout)
  - [1.1 `include/okapi/` — the one Linux idea that needs translating](#11-includeokapi--the-one-linux-idea-that-needs-translating)
- [2. File and module naming](#2-file-and-module-naming)
- [3. Naming identifiers](#3-naming-identifiers)
- [4. Formatting](#4-formatting)
- [5. Wiring directories into one crate](#5-wiring-directories-into-one-crate)
- [6. Documentation comments](#6-documentation-comments)
  - [6.1 `unsafe` requires justification](#61-unsafe-requires-justification)
- [7. Layering rules](#7-layering-rules)
- [8. Error handling](#8-error-handling)
- [9. Deferred](#9-deferred)

---

## 0. Precedence rule

> **Rust guidelines and rules come first.**
>
> Linux supplies the *organization*: what the directories are called, how
> subsystems are separated, where a driver lives, which layer owns which
> concern. It does not supply the *language conventions*. Where Linux's C
> idioms conflict with Rust's, **Rust wins — without exception.**

The reasoning: Okapi's code must be legible to Rust developers, work with
unmodified `cargo fmt` and `cargo clippy`, and sit comfortably alongside the
crates it depends on (`riscv`, `riscv-rt`). Linux's C conventions exist to
compensate for things C lacks — namespaces, modules, generics, ownership.
Rust has all of them. Importing C workarounds into a language that doesn't
need them produces code that is neither good Rust nor real Linux.

What this means concretely:

| Concern | Source of truth |
|---|---|
| Directory names and layering | Linux |
| Which subsystem owns a concern | Linux |
| Driver/irqchip taxonomy | Linux |
| File naming, identifiers, formatting | **Rust** |
| Error handling, ownership, generics | **Rust** |
| Comment/doc syntax | **Rust** |
| Module boundaries and visibility | **Rust** |

---

## 1. Directory layout

Linux's top-level taxonomy, applied to an RTOS on a core with no MMU:

```
okapi/
├── Cargo.toml
├── build.rs                 # links memory.x
├── memory.x                 # FLASH/RAM regions for the linker
├── .cargo/config.toml       # target + link args
├── init/
│   └── main.rs              # #[entry]; kernel entry and bring-up order
├── arch/
│   └── riscv/               # core-specific: traps, context switch, CSRs
│       ├── mod.rs
│       ├── trap.rs          # trap entry glue, dispatch to kernel::irq
│       └── andes.rs         # AndeStar V5 vendor CSRs (mxstatus, mmisc_ctl)
├── kernel/                  # core OS, hardware-independent
│   ├── mod.rs
│   ├── irq/                 # IRQ registration, masking, dispatch
│   │   └── mod.rs
│   └── sched/               # scheduler, tasks (not yet implemented)
│       └── mod.rs
├── drivers/                 # device drivers, one subdir per class
│   ├── mod.rs
│   └── irqchip/             # interrupt controllers
│       ├── mod.rs
│       └── andes_plic.rs    # Andes PLIC register map
├── lib/                     # kernel-internal data structures
│   └── mod.rs               # intrusive lists, bitmaps, ring buffers
├── include/
│   └── okapi/               # public API surface (see §1.1)
│       └── mod.rs
└── doc/
```

Directory rules:

- **`arch/<isa>/`** — anything that changes when the ISA or core changes.
  Vendor CSR access is confined here so a non-Andes port swaps this subtree
  alone.
- **`kernel/`** — no MMIO, no register offsets, no vendor CSRs. If a file
  under `kernel/` needs a hardware address, the layering is wrong.
- **`drivers/<class>/`** — grouped by *class*, not by vendor, exactly as
  Linux does (`drivers/irqchip/`, later `drivers/tty/`, `drivers/gpio/`).
  The vendor appears in the *file* name, not the directory.
- **`lib/`** — generic data structures with no hardware or policy knowledge.
- **`init/`** — bring-up only. Ordering of subsystem initialization lives
  here and nowhere else.

### 1.1 `include/okapi/` — the one Linux idea that needs translating

Linux separates headers from implementation because C requires it. Rust has
no headers, so a literal `include/` would be meaningless.

Okapi keeps the directory but gives it the Rust-equivalent job: **it is the
public API surface**, a re-export facade defining what application code and
out-of-tree drivers may depend on.

```rust
// include/okapi/mod.rs — the stable surface, nothing else
pub use crate::kernel::irq::{self, IrqHandler, IrqNumber};
```

Everything not re-exported here is kernel-internal and may change freely.
This gives Linux's "public vs. internal" distinction using Rust's actual
mechanism — visibility and re-exports — rather than file placement.

Because Cargo requires one crate root, the Linux directories are attached as
modules with explicit `#[path]` in `init/main.rs` (§5).

---

## 2. File and module naming

**Rust rules, not Linux's.** Linux writes `irq-andes-plic.c`; hyphens are not
legal in Rust module names, and `rustfmt`/Cargo assume `snake_case`.

| | Linux | **Okapi** |
|---|---|---|
| Driver file | `irq-andes-plic.c` | `andes_plic.rs` |
| Arch trap code | `traps_32.c` | `trap.rs` |
| Subsystem dir entry | `irq/Makefile` | `irq/mod.rs` |

- Module files: `snake_case.rs`.
- Directory modules: `mod.rs` inside the directory.
- The directory already provides context — `drivers/irqchip/andes_plic.rs`,
  not `drivers/irqchip/irqchip_andes_plic.rs`.

---

## 3. Naming identifiers

Rust's API Guidelines apply in full. The key divergence from Linux: **no
subsystem prefixes on identifiers.** Linux writes `okapi_irq_register()`
because C has a single global namespace; Rust has modules.

```rust
// Okapi — the module path *is* the namespace
irq::register(IRQ_UART0, uart_isr);
plic::claim();

// Not this
okapi_irq_register(IRQ_UART0, uart_isr);
okapi_plic_claim();
```

Standard Rust casing throughout:

- `snake_case` — functions, methods, variables, modules
- `UpperCamelCase` — types, traits, enum variants
- `SCREAMING_SNAKE_CASE` — consts and statics
- Getters are `fn threshold()`, not `fn get_threshold()`

Abbreviations follow the Rust convention of treating acronyms as words in
type names: `PlicDriver`, not `PLICDriver`; but `const PLIC_BASE` keeps the
acronym uppercase, since consts are fully uppercase anyway.

---

## 4. Formatting

`rustfmt` defaults. No `rustfmt.toml` — if the repo needs one, that is a
discussion, not a silent addition.

- 4 spaces, no hard tabs (Linux's 8-wide tabs do not apply)
- 100-column limit (not 80)
- `cargo fmt --check` and `cargo clippy` must pass before commit

Linux's 80-column/8-tab style is deliberately **not** adopted: it would put
every Okapi file in conflict with the formatter that ships with the
toolchain, and with every dependency.

---

## 5. Wiring directories into one crate

Rust expects modules under `src/`. Linux puts subsystems at the top level.
Reconciled with `#[path]` in the crate root — this is the single concession
needed to get the Linux tree, and it is contained to one file:

```rust
// init/main.rs
#![no_std]
#![no_main]

#[path = "../arch/mod.rs"]
mod arch;
#[path = "../drivers/mod.rs"]
mod drivers;
#[path = "../include/okapi/mod.rs"]
pub mod api;
#[path = "../kernel/mod.rs"]
mod kernel;
#[path = "../lib/mod.rs"]
mod lib;
```

Nested modules below these resolve normally — no further `#[path]` needed.

---

## 6. Documentation comments

Rust doc comments, not Linux kernel-doc blocks.

```rust
/// Registers `handler` for PLIC source `irq`.
///
/// Replaces any previous handler. The IRQ remains masked until
/// [`unmask`] is called.
///
/// # Panics
/// Panics if `irq` exceeds [`MAX_IRQ`].
pub fn register(irq: IrqNumber, handler: IrqHandler) { ... }
```

Not the Linux form:

```c
/*
 * okapi_irq_register - register a handler
 * @irq: PLIC source id
 */
```

Rationale: `cargo doc` renders the former, intra-doc links (`[`unmask`]`) are
checked by the compiler, and `# Panics` / `# Safety` are conventional
headings Rust developers look for.

### 6.1 `unsafe` requires justification

Every `unsafe` block carries a comment stating why it is sound. MMIO is the
common case:

```rust
// SAFETY: CLAIM is a valid PLIC MMIO register for this core, mapped by the
// SoC and never aliased by Rust references. A volatile read is the
// architecturally defined way to claim the pending IRQ.
let id = unsafe { read_volatile(CLAIM) };
```

`unsafe fn` additionally documents its contract under `# Safety`.

---

## 7. Layering rules

Enforced by review; these are the rules that keep the taxonomy meaningful.

1. **Register offsets live in exactly one file per device.** Only
   `drivers/irqchip/andes_plic.rs` knows the PLIC's register layout.
2. **`kernel/` never names a specific device.** `kernel::irq` must not
   mention "plic" in a type, function, or import. It talks to an interface.
3. **Vendor CSRs stay in `arch/riscv/andes.rs`.** Not in drivers, not in
   `kernel/`.
4. **Dependencies point inward.** `drivers/` and `arch/` may use `lib/` and
   `kernel/` interfaces; `kernel/` may not reach into `drivers/`.
5. **`init/` owns ordering.** No subsystem self-initializes via hidden
   constructors.

The payoff, per `startup.md` §5.4: changing Andes part, or adding `PLIC_SW`
for IPIs on a multicore variant, touches one file.

---

## 8. Error handling

Rust conventions:

- `Result<T, E>` for fallible operations; no C-style negative error codes.
- No `panic!` in interrupt context or scheduler internals — return `Result`
  and let the caller decide.
- `#[must_use]` on anything whose result being ignored is a bug.
- No `unwrap()` outside tests and `init/`, where a failure genuinely is
  unrecoverable and panicking is the honest response.

---

## 9. Deferred

Not decided yet, deliberately:

- **Build configuration** — Kconfig/`defconfig`-style selection, and whether
  subsystems become Cargo workspace crates or stay modules in one crate. The
  layering in §7 holds either way, which is why this can wait.
- **Out-of-tree driver support** — whether `include/okapi/` becomes a real
  published crate boundary.
- **Multicore** — `PLIC_SW`/IPI paths, per-CPU state.
