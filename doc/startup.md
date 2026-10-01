# RISC-V Rust Toolchain Setup — Andes N25F (32-bit)

Detailed setup notes for bringing up a `no_std` Rust environment targeting the
Andes N25F core (32-bit RISC-V). This expands on the summary slide in
`okapi.html` with the full reasoning and all options discussed.

---

## Contents

- [1. General 32-bit RISC-V background](#1-general-32-bit-risc-v-background)
  - [1.1 Picking a target triple](#11-picking-a-target-triple)
  - [1.2 `.cargo/config.toml`](#12-cargoconfigtoml)
  - [1.3 Runtime crates](#13-runtime-crates)
  - [1.4 Optional GNU binutils](#14-optional-gnu-binutils)
  - [1.5 QEMU for testing](#15-qemu-for-testing)
- [2. Andes N25F specifics](#2-andes-n25f-specifics)
  - [2.1 Confirm the exact ISA string first](#21-confirm-the-exact-isa-string-first)
  - [2.2 Target triple problem: no stock target for `imafc`](#22-target-triple-problem-no-stock-target-for-imafc)
  - [2.3 `.cargo/config.toml` (soft-float baseline)](#23-cargoconfigtoml-soft-float-baseline)
  - [2.4 Runtime crate caveat — Andes interrupt controller](#24-runtime-crate-caveat--andes-interrupt-controller)
  - [2.5 `memory.x` — required, no generic default](#25-memoryx--required-no-generic-default)
  - [2.6 Minimal project skeleton](#26-minimal-project-skeleton)
  - [2.7 Validating before touching real silicon](#27-validating-before-touching-real-silicon)
- [3. Basic bring-up — worked example (the `okapi` crate)](#3-basic-bring-up--worked-example-the-okapi-crate)
  - [3.1 Starting layout problems](#31-starting-layout-problems)
  - [3.2 Fixing Flag 3, attempt 1 — `memory.x` alone is not enough](#32-fixing-flag-3-attempt-1--memoryx-alone-is-not-enough)
  - [3.3 First `build.rs` attempt — wrong template copied](#33-first-buildrs-attempt--wrong-template-copied)
  - [3.4 Correct `build.rs`](#34-correct-buildrs)
  - [3.5 Still failing — `memory.x` found but not loaded](#35-still-failing--memoryx-found-but-not-loaded)
  - [3.6 Final state — clean build](#36-final-state--clean-build)
- [4. What these crates give you, and what is actually "the OS"](#4-what-these-crates-give-you-and-what-is-actually-the-os)
  - [4.1 What the crates provide](#41-what-the-crates-provide)
  - [4.2 Where "main" sits, corrected](#42-where-main-sits-corrected)
  - [4.3 Layering, restated](#43-layering-restated)
  - [4.4 The one place the crates actively mislead](#44-the-one-place-the-crates-actively-mislead)
- [5. Interrupt controller — adopt a crate, or write the PLIC driver?](#5-interrupt-controller--adopt-a-crate-or-write-the-plic-driver)
  - [5.1 The three layers, and why only one of them is `riscv-rt`'s](#51-the-three-layers-and-why-only-one-of-them-is-riscv-rts)
  - [5.2 Options considered](#52-options-considered)
  - [5.3 Dependency impact: none](#53-dependency-impact-none)
  - [5.4 Where this code lands](#54-where-this-code-lands)
  - [5.5 The dispatch path](#55-the-dispatch-path)
  - [5.6 What must be confirmed before writing register offsets](#56-what-must-be-confirmed-before-writing-register-offsets)
- [6. What `riscv-rt` 0.18 actually provides for interrupts](#6-what-riscv-rt-018-actually-provides-for-interrupts)
  - [6.1 The generic flow, as implemented](#61-the-generic-flow-as-implemented)
  - [6.2 Where the generic path stops](#62-where-the-generic-path-stops)
  - [6.3 Steps to enable the Andes PLIC on top of this](#63-steps-to-enable-the-andes-plic-on-top-of-this)
  - [6.4 Consequence for the §5 decision](#64-consequence-for-the-5-decision)
- [7. The machine timer — the easy path through the same machinery](#7-the-machine-timer--the-easy-path-through-the-same-machinery)
  - [7.1 What `riscv-rt` does and does not do for it](#71-what-riscv-rt-does-and-does-not-do-for-it)
  - [7.2 The pieces Okapi must supply](#72-the-pieces-okapi-must-supply)
  - [7.3 `DefaultHandler` is declared but not defined](#73-defaulthandler-is-declared-but-not-defined)
  - [7.4 The level-triggered trap — the main bring-up hazard](#74-the-level-triggered-trap--the-main-bring-up-hazard)
  - [7.5 RV32-specific caveat: `mtimecmp` is 64-bit](#75-rv32-specific-caveat-mtimecmp-is-64-bit)
  - [7.6 Andes naming](#76-andes-naming)
  - [7.7 Dependencies — what the timer actually needs](#77-dependencies--what-the-timer-actually-needs)
  - [7.8 Clocks — the reference clock, and the other two](#78-clocks--the-reference-clock-and-the-other-two)
    - [7.8.0 Why there are three clocks at all](#780-why-there-are-three-clocks-at-all)
    - [7.8.1 Timebase frequency is not the CPU frequency](#781-timebase-frequency-is-not-the-cpu-frequency)
    - [7.8.2 Where the value comes from](#782-where-the-value-comes-from)
    - [7.8.3 Is it fixed, or does it need setting up?](#783-is-it-fixed-or-does-it-need-setting-up)
    - [7.8.4 Verifying the value empirically](#784-verifying-the-value-empirically)
    - [7.8.5 Checklist](#785-checklist)
    - [7.8.6 The CPU clock — `mcycle` / `mcycleh`](#786-the-cpu-clock--mcycle--mcycleh)
    - [7.8.7 The bus / peripheral clock](#787-the-bus--peripheral-clock)
    - [7.8.8 The rule that ties the domains together, and `init/` ordering](#788-the-rule-that-ties-the-domains-together-and-init-ordering)
    - [7.8.9 Diagnosing the wrong clock](#789-diagnosing-the-wrong-clock)
  - [7.9 Testing on QEMU — useful, but not for the register map](#79-testing-on-qemu--useful-but-not-for-the-register-map)
  - [7.10 Why bring this up before the PLIC](#710-why-bring-this-up-before-the-plic)
- [8. FAQ — details that came up while reading the crate source](#8-faq--details-that-came-up-while-reading-the-crate-source)
  - [8.1 What does `__CORE_INTERRUPTS`' type actually mean?](#81-what-does-__core_interrupts-type-actually-mean)
  - [8.2 Why `N + 1`, and why the gaps?](#82-why-n--1-and-why-the-gaps)
  - [8.3 Why must the handlers be `extern "C"`? They are just ISRs.](#83-why-must-the-handlers-be-extern-c-they-are-just-isrs)
  - [8.4 Why do the function *pointers* need `extern "C"` as well?](#84-why-do-the-function-pointers-need-extern-c-as-well)
  - [8.5 Why `Option<fn()>` instead of a null pointer like C?](#85-why-optionfn-instead-of-a-null-pointer-like-c)
  - [8.6 What is the double `Some` in the dispatcher doing?](#86-what-is-the-double-some-in-the-dispatcher-doing)
  - [8.7 When does any of this require action from us?](#87-when-does-any-of-this-require-action-from-us)
  - [8.8 The machine timer is a core interrupt — does `riscv-rt` handle it?](#88-the-machine-timer-is-a-core-interrupt--does-riscv-rt-handle-it)
  - [8.9 Why does my timer interrupt fire continuously / the board appear hung?](#89-why-does-my-timer-interrupt-fire-continuously--the-board-appear-hung)
  - [8.10 Why can't I just write `mtimecmp` in one store on RV32?](#810-why-cant-i-just-write-mtimecmp-in-one-store-on-rv32)
  - [8.11 Should the timer driver live in `drivers/irqchip/`?](#811-should-the-timer-driver-live-in-driversirqchip)
- [9. Open items / follow-ups](#9-open-items--follow-ups)
- [10. Summary of recommended order of operations](#10-summary-of-recommended-order-of-operations)

---

## 1. General 32-bit RISC-V background

Before narrowing to the N25F specifically, here's the general shape of a
32-bit RISC-V Rust setup.

### 1.1 Picking a target triple

```bash
rustup target add riscv32imac-unknown-none-elf   # mul/div + atomics + compressed
rustup target add riscv32imc-unknown-none-elf    # no atomics
rustup target add riscv32i-unknown-none-elf      # minimal, no M/C extensions
rustup target add riscv32gc-unknown-linux-gnu    # Linux userspace, not bare-metal
```

The target must match what the core's ISA actually implements. Using a target
that assumes extensions the CPU lacks produces illegal instructions at
runtime — this is not a compile-time-checkable mismatch.

### 1.2 `.cargo/config.toml`

```toml
[target.riscv32imac-unknown-none-elf]
rustflags = ["-C", "link-arg=-Tlink.x"]
linker = "rust-lld"

[build]
target = "riscv32imac-unknown-none-elf"
```

`rust-lld` ships with `rustc`, so no external GNU toolchain is strictly
required just to link.

### 1.3 Runtime crates

```toml
[dependencies]
riscv = "0.16"
riscv-rt = "0.18"   # startup/vector table, analogous to cortex-m-rt for ARM
```

### 1.4 Optional GNU binutils

For `objdump`/`objcopy`/`gdb`:

```bash
sudo apt install gcc-riscv64-unknown-elf   # works for 32-bit ELF tooling too
# or:
rustup component add llvm-tools-preview
cargo install cargo-binutils
```

### 1.5 QEMU for testing

```bash
sudo apt install qemu-system-misc
qemu-system-riscv32 -machine virt -nographic -kernel target/riscv32imac-unknown-none-elf/debug/your-binary
```

---

## 2. Andes N25F specifics

The Andes N25F is a 32-bit RISC-V core with the **F extension**
(single-precision hardware float) in addition to the base integer set. Andes
cores are configurable per SoC instance, so nothing here should be assumed —
it must be confirmed against the actual chip.

### 2.1 Confirm the exact ISA string first

Check the SoC datasheet / vendor config for which extensions are actually
enabled on this specific instance:

- **I** — base integer (always present)
- **M** — mul/div (usually present)
- **A** — atomics (not guaranteed — often omitted on smaller MCU-class chips)
- **F** — single-precision float (this is what makes it "N25F" vs plain "N25")
- **C** — compressed instructions (common, reduces code size)

N25F likely implements **RV32IMAFC**, but this must be verified per SoC — do
not assume.

This determines everything downstream: which target to use, whether a custom
target spec is needed, and the ABI.

### 2.2 Target triple problem: no stock target for `imafc`

Rust does not ship a built-in target for exactly `rv32imafc` with hardware
single-precision float in bare-metal (`none-elf`) form. Checking available
targets:

```bash
rustc --print target-list | grep riscv32
```

`riscv32gc-unknown-none-elf` exists, but `gc` implies `imafdc` — i.e. it
includes **double-precision D**, which the N25F (single-precision only) does
not have. Using it would generate double-float instructions the core can't
execute.

Two practical paths:

**Option A — soft-float first (recommended starting point).** Use
`riscv32imac-unknown-none-elf` and let floats be emulated in software,
ignoring the hardware FPU entirely. This avoids ABI mismatches and de-risks
initial bring-up — get something booting, doing UART I/O, etc., before
tackling float codegen as a separate, isolated problem.

**Option B — custom target JSON for hardware single-precision float.** Needed
if/when you want the compiler to actually emit F-extension instructions and
use the `ilp32f` ABI.

```bash
rustc +nightly -Z unstable-options \
  --target riscv32imac-unknown-none-elf \
  --print target-spec-json > riscv32imafc.json
```

Edit the generated JSON:
- `"features"`: `"+m,+a,+f,+c"`
- `"llvm-abiname"`: `"ilp32f"`
- adjust `"max-atomic-width"` as appropriate for the core

Build with:

```bash
cargo +nightly build -Z build-std=core --target riscv32imafc.json
```

This requires **nightly** Rust plus `-Z build-std`, since a custom target
isn't tier-2/built-in and the standard library isn't prebuilt for it.

### 2.3 `.cargo/config.toml` (soft-float baseline)

```toml
[build]
target = "riscv32imac-unknown-none-elf"

[target.riscv32imac-unknown-none-elf]
rustflags = ["-C", "link-arg=-Tlink.x"]
linker = "rust-lld"

[unstable]
build-std = ["core"]
```

### 2.4 Runtime crate caveat — Andes interrupt controller

`riscv-rt` is the standard startup/trap-vector crate, but it assumes the
generic RISC-V privileged spec trap model (SiFive-style CLINT/PLIC). The
**Andes N25 series does not use that model** — it uses Andes' own
**AndeStar V5** trap/interrupt architecture, with its own CSRs and behavior:

- Vectored interrupts via `mtvt` (vector table base, not a single `mtvec`
  handler dispatched by software)
- `mmisc_ctl` and other Andes-specific CSRs controlling core behavior
- A PLIC-like interrupt controller (`PLIC_SW`/`PLMT`) that isn't a drop-in
  match for the generic SiFive PLIC/CLINT that `riscv-rt` and the `riscv`
  crate assume

Practical implication: stock `riscv-rt` trap handling may not vector
interrupts correctly on Andes hardware without patches. Two ways to proceed:

1. Use `riscv-rt`'s basic (non-vectored, polling/dispatch-in-software)
   exception handling to start, deliberately ignoring Andes' vectored
   extensions — simpler, gets you running, leaves performance/latency on the
   table.
2. Write a custom `_start` / trap entry that respects Andes' CSR extensions
   (`mtvt`-based vectoring, `mmisc_ctl` setup) — more correct for this core,
   more upfront work.

No vendor BSP was available at the time of this discussion (confirmed
starting from scratch, upstream Rust/LLVM only) — this needs to be revisited
once real hardware/datasheet access is in hand, or if Andes' own tooling
(AndeSight, riscv-gnu-toolchain fork) becomes available with a reference
linker script and startup code to compare against.

### 2.5 `memory.x` — required, no generic default

`riscv-rt` requires a `memory.x` describing FLASH/RAM regions. There is no
generic value that works — it must come from the actual Andes SoC's memory
map (datasheet or vendor-provided linker script reference). Example shape
(**placeholder addresses only — do not use as-is**):

```
MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 512K
  RAM   : ORIGIN = 0x80000000, LENGTH = 64K
}
REGION_ALIAS("REGION_TEXT", FLASH);
REGION_ALIAS("REGION_RODATA", FLASH);
REGION_ALIAS("REGION_DATA", RAM);
REGION_ALIAS("REGION_BSS", RAM);
REGION_ALIAS("REGION_HEAP", RAM);
REGION_ALIAS("REGION_STACK", RAM);
```

Using placeholder addresses will link successfully but will not boot on real
silicon.

### 2.6 Minimal project skeleton

```bash
cargo new --bin n25f-firmware
cd n25f-firmware
rustup target add riscv32imac-unknown-none-elf
cargo add riscv riscv-rt
cargo add panic-halt   # or panic-abort — something must satisfy the panic handler
```

Minimal `main.rs` to prove the build/link pipeline works before adding real
logic:

```rust
#![no_std]
#![no_main]

use riscv_rt::entry;
use panic_halt as _;

#[entry]
fn main() -> ! {
    loop {}
}
```

### 2.7 Validating before touching real silicon

```bash
sudo apt install qemu-system-misc
qemu-system-riscv32 -machine virt -nographic -kernel target/riscv32imac-unknown-none-elf/debug/n25f-firmware
```

QEMU's generic `virt` machine does **not** match the Andes N25F exactly (no
AndeStar V5 CSRs, different interrupt controller) — it only validates that
the Rust build/link pipeline produces a working RISC-V ELF that boots to
`main`. It cannot validate Andes-specific interrupt/trap behavior; that must
be tested on real hardware or an Andes-specific simulator (e.g. AndeSim, if
available).

---

## 3. Basic bring-up — worked example (the `okapi` crate)

This section documents the actual step-by-step bring-up of the `okapi` crate
at `src/okapi/`, including the mistakes made and how each was diagnosed and
fixed. Kept in full because each failure exposes a real, non-obvious
requirement of the `riscv-rt` toolchain that isn't obvious from its docs
alone.

### 3.1 Starting layout problems

The crate was first created with `cargo new`, then hand-edited. Three issues
were flagged before the first real build attempt:

**Flag 1 — build/target config in the wrong file.** `[build]` and
`[target.riscv32imac-unknown-none-elf]` were initially placed inside
`Cargo.toml`. Cargo does not read target/build configuration from
`Cargo.toml` — it reads it from `.cargo/config.toml`. Left as-is, cargo
silently ignores those sections, so the target and linker flag never actually
apply.

Fix: move those sections into `src/okapi/.cargo/config.toml`, leaving
`Cargo.toml` with only `[package]` and `[dependencies]`.

**Flag 2 — `main.rs` was still the `cargo new` template.**

```rust
fn main() {
    println!("Hello, world!");
}
```

This assumes a hosted `std` environment and an implicit entry point — neither
exists on `riscv32imac-unknown-none-elf`. `println!` has no backing
allocator/stdout on bare metal, and there is no `std` to link against.

Fix: rewrite as a `no_std`/`no_main` entry point using `riscv-rt`:

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use riscv_rt::entry;

#[entry]
fn main() -> ! {
    loop {}
}
```

**Flag 3 — no `memory.x`.** Even with the above fixed, the link step fails,
because `-Tlink.x` refers to a linker script `riscv-rt` generates, and that
script needs `memory.x` to resolve `REGION_TEXT`/`REGION_RAM`/etc. — and no
such file existed in the crate yet.

### 3.2 Fixing Flag 3, attempt 1 — `memory.x` alone is not enough

`memory.x` was added at the crate root:

```
MEMORY
{
  FLASH : ORIGIN = 0xff000000, LENGTH = 320K
  RAM   : ORIGIN = 0xff100000, LENGTH = 448K
}
REGION_ALIAS("REGION_TEXT", FLASH);
REGION_ALIAS("REGION_RODATA", FLASH);
REGION_ALIAS("REGION_DATA", RAM);
REGION_ALIAS("REGION_BSS", RAM);
REGION_ALIAS("REGION_HEAP", RAM);
REGION_ALIAS("REGION_STACK", RAM);
```

Just creating the file is not sufficient — the linker never automatically
searches the crate root for it. Building at this point still failed:

```
rust-lld: error: .../riscv-rt-.../out/link.x:25: memory region not defined: REGION_TEXT
        >>> PROVIDE(_stext = ORIGIN(REGION_TEXT));
```

Root cause: `riscv-rt`'s generated `link.x` references `REGION_TEXT` etc. but
does **not** `INCLUDE memory.x` itself (confirmed by inspecting the generated
`link.x` in `target/.../build/riscv-rt-*/out/link.x` — no `INCLUDE` line
present). The crate must supply `memory.x` on the link-search path itself,
via its own `build.rs`.

### 3.3 First `build.rs` attempt — wrong template copied

The first `build.rs` written was copied from `riscv-rt`'s **own internal**
`build.rs` (the one used to generate `link.x.in` → `link.x` inside the
`riscv-rt` crate itself), not the much simpler template a *consumer* crate is
supposed to use:

```rust
use riscv_target::Target;   // wrong — this is riscv-rt's own internal tool
// ...ISA-detection logic to generate link.x from link.x.in...
```

This failed immediately:

```
error[E0432]: unresolved import `riscv_target`
```

Lesson: `riscv-rt`'s repo contains two different `build.rs`-shaped things —
the one that builds the `riscv-rt` crate itself (ISA detection, generates
`link.x` from `link.x.in`, depends on the internal `riscv-target` crate), and
the much smaller one every *downstream* application crate needs (just make
`memory.x` visible to the linker). Copying the wrong one is an easy mistake
because both live in/near the same repo.

### 3.4 Correct `build.rs`

```rust
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::copy("memory.x", out_dir.join("memory.x")).unwrap();
    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rerun-if-changed=memory.x");
}
```

Placed at the crate root, next to `Cargo.toml`. This only copies `memory.x`
into `OUT_DIR` and adds that directory to the linker's search path — no
target/ISA detection needed, since the target is already fixed via
`.cargo/config.toml`.

### 3.5 Still failing — `memory.x` found but not loaded

After fixing `build.rs`, the *same* `REGION_TEXT` error recurred:

```
rust-lld: error: .../link.x:25: memory region not defined: REGION_TEXT
```

Even though `memory.x` was now correctly copied into `OUT_DIR` and that
directory was on the link-search path (`-L .../build/okapi-*/out`), nothing
was telling the linker to actually *load* `memory.x` as a linker script.
Being on the search path only means `INCLUDE memory.x` would resolve *if*
something used `INCLUDE` — and, as established in 3.2, `link.x` never does.

Fix: pass `memory.x` as its own explicit linker script argument, in addition
to `link.x`, in `.cargo/config.toml`:

```toml
[target.riscv32imac-unknown-none-elf]
rustflags = ["-C","link-arg=-Tmemory.x","-C","link-arg=-Tlink.x"]
```

Order matters here conceptually (though ld/lld merges linker-script content
rather than strictly sequencing it): `memory.x` supplies the bare `MEMORY {}`
block and `REGION_ALIAS`es; `link.x` supplies the `SECTIONS` block that
references those regions by name.

### 3.6 Final state — clean build

With all of the above in place:

- `Cargo.toml` — only `[package]` + `[dependencies]` (`riscv`, `riscv-rt`,
  `panic-halt`)
- `.cargo/config.toml` — target set to `riscv32imac-unknown-none-elf`,
  `rustflags` passing **both** `-Tmemory.x` and `-Tlink.x`
- `memory.x` — real FLASH/RAM addresses for the target SoC, at the crate root
- `build.rs` — copies `memory.x` into `OUT_DIR`, adds it to the link-search
  path
- `src/main.rs` — `#![no_std]`, `#![no_main]`, `#[entry]`, `panic_halt`

```bash
$ cargo build
   Finished `dev` profile [unoptimized + debuginfo] target(s)
```

This confirms the build/link pipeline is structurally correct. It does
**not** confirm the `memory.x` addresses are actually correct for the real
Andes N25F SoC (a wrong-but-plausible address still links fine — see §2.5) or
that interrupts will vector correctly (see §2.4, AndeStar V5 caveat still
open). Both remain to be validated against hardware/datasheet, then on real
silicon or an Andes-specific simulator.

---

## 4. What these crates give you, and what is actually "the OS"

Worth stating explicitly, since it's easy to lose track of once the build is
passing: `riscv`, `riscv-rt`, and friends (from
[rust-embedded/riscv](https://github.com/rust-embedded/riscv)) are **not** an
OS, and using them does not mean the OS is being built for you. They are the
hardware-access and boot-runtime layer underneath where the OS starts.

### 4.1 What the crates provide

- **`riscv`** — safe wrappers around CSR access (`mstatus`, `mie`, `mcause`,
  `mtvec`, …) and core instructions (`ecall`, `wfi`, fences). This is register
  access, not OS behavior.
- **`riscv-rt`** — the boot runtime: `_start` assembly, stack pointer setup,
  `.bss` clearing, `.data` copy from flash to RAM, trap vector installation,
  and the `#[entry]` macro that hands control to your first Rust function.
  This is the piece that gets execution from silicon reset to a safely
  initialized `main` — i.e. everything covered in §3 of this document.
- **`riscv-pac`** (and, in newer versions, `riscv-peripheral`) — trait/enum
  plumbing for interrupt numbers and generic SiFive-style CLINT/PLIC register
  layouts. Generic by design — does not know about Andes' AndeStar V5
  interrupt model (§2.4).

None of the above contains a scheduler, a task/thread abstraction, a syscall
ABI, memory management, or synchronization primitives. There is no crate that
provides these generically — that absence is not a gap to fill by finding
the right dependency; it is the actual content of the Okapi project.

### 4.2 Where "main" sits, corrected

`#[entry] fn main() -> !` is **not** the kernel. It is the first line of the
kernel's own code — the point where `riscv-rt` hands off a safely
initialized machine (stack set, `.bss`/`.data` correct, trap vector
installed) to code Okapi is responsible for. Everything reachable from that
point — trap dispatch, scheduler, tasks, sync primitives, memory management,
syscalls, HAL/drivers — is the kernel, and none of it currently exists beyond
the `loop {}` placeholder in `main.rs`.

### 4.3 Layering, restated

```
riscv-rt        _start → stack/bss/data init → trap vector install → main()
                (boot + runtime; from the crate, mostly as-is)
   |
   v
main()          entry point into Okapi — currently loop {}
   |
   v
Okapi kernel    trap dispatcher (Andes-correct, not riscv-rt's default)
                  -> scheduler
                       -> task/thread abstraction
                            -> sync primitives (mutex, semaphore, queue)
                                 -> memory management
                                      -> HAL / drivers
                                           -> syscall (ecall) ABI
                (all of this is Okapi's own code — matches okapi.html
                 roadmap slide steps 04-09)
```

### 4.4 The one place the crates actively mislead

`riscv-rt` installs *a* trap vector by default, using the generic RISC-V
privileged-spec model. On Andes N25 (AndeStar V5), this is not just
incomplete — it is architecturally wrong for taking full advantage of
vectored interrupts (`mtvt`), and needs to be bypassed or replaced once
interrupt-driven work begins (§2.4). Relying on `riscv-rt`'s default trap
handling past the "does it boot" stage will need deliberate revisiting, not
just extension.

---

## 5. Interrupt controller — adopt a crate, or write the PLIC driver?

Picking up from §4.4: once interrupt-driven work begins, the Andes PLIC has
to come from somewhere. The question that actually arose was narrower than
"how do interrupts work" — it was a **sourcing** decision:

> We already depend on `riscv-rt` for interrupts. To bring up the Andes PLIC,
> do we add PLIC support to that crate, pull in an existing PLIC crate, or
> write our own driver inside Okapi?

**Conclusion: keep `riscv-rt` as-is for trap entry, and write the Andes PLIC
as Okapi's own driver.** Do not fork `riscv-rt`; do not adopt a generic PLIC
crate. The reasoning below is worth recording because the same argument
recurs for every vendor peripheral we hit later.

### 5.1 The three layers, and why only one of them is `riscv-rt`'s

The premise that needed correcting first: `riscv-rt` is not "the interrupt
crate." Three distinct concerns get collapsed under the word "interrupt," and
`riscv-rt` owns exactly one of them.

**1 — `riscv-rt` owns trap *entry*, not interrupt *routing*.**
It provides the reset handler, `.data`/`.bss` init, `mtvec` setup, and the
`#[riscv_rt::core_interrupt]` / `#[exception]` macros that dispatch on
`mcause`. That is the **CLINT-level** world: machine software, machine timer,
machine external. It stops at *"an external interrupt arrived"*
(`MachineExternal`, cause 11) **by design**. Deciding *which* of 50+ device
sources fired, at what priority, under what threshold, is the PLIC's job and
is deliberately out of scope for a boot-runtime crate.

**2 — The Andes PLIC is not the standard PLIC.**
This is the part that decides the question. The divergences are real, not
cosmetic:

- Andes parts typically ship a **`PLIC` + `PLIC_SW` pair** — the second
  dedicated to software/IPI interrupts, with its own base address and
  register block. A standard-PLIC abstraction has no slot for it.
- Base addresses are **vendor-specific**, coming from the core configuration
  — not the SiFive-canonical `0x0C00_0000` that crates commonly assume or
  hardcode.
- Andes adds a **preemptive-priority / nested-interrupt** scheme driven by
  vendor CSRs that the standard RISC-V PLIC spec has no notion of. The spec
  defines priority and threshold to gate *whether* an interrupt is presented,
  and says nothing about preempting a handler already running — nesting is
  left entirely to software. On Andes cores the nesting discipline (save
  `mepc`/`mstatus`/`mcause`, re-enable, claim, complete, restore) is **the
  vendor's**, and touching `mxstatus` is part of it.
- Vendor CSRs sit **in the hot path**: `mxstatus` for the AndeStar
  extensions, `mmisc_ctl` for interrupt behavior (§2.4). No generic crate
  exposes these.

So a generic PLIC crate models the *standard* register map and gives us
**nothing** for the Andes-specific half — while adding a dependency whose
abstraction we then have to work around. Both "adopt a crate" and "write our
own" require writing the Andes-specific code; only the latter avoids paying
for an abstraction we discard.

**3 — For an OS, the interrupt controller is a kernel subsystem.**
Okapi is still `main.rs` with a `loop {}`. The PLIC is about to become
load-bearing for the scheduler (timer-tick preemption), the device model, and
the IRQ-masking / critical-section primitives. That is architecture to own
and shape to the kernel's needs, not inherit from a crate designed for
single-application bare-metal firmware.

### 5.2 Options considered

| Option | What it means | Andes fit | Cost / risk | Call |
|---|---|---|---|---|
| **A** — fork or extend `riscv-rt` | Add PLIC support upstream or in a vendored fork, so the boot runtime does device routing too | Wrong layer | Permanent rebase burden against 0.18+; the crate deliberately has no device-IRQ model, so this pushes against its design rather than extending it | No |
| **B** — generic PLIC crate | Pull an existing SiFive-style PLIC crate, configure a base address | Partial | Standard register map only: no `PLIC_SW`, no preemptive-priority nesting, no vendor CSRs. We fight its abstraction, then write the Andes parts anyway | No |
| **C** — own driver in Okapi | A PLIC driver plus a kernel-facing IRQ API; on the order of 150 lines of MMIO behind a stable interface | Exact | Zero new dependencies. We own the nesting discipline and can shape the API around the scheduler. Cost is writing and testing the register map — which B needs too | **Yes** |
| **D** — FFI to the Andes C BSP | Bind to AndeSight's `nds_` PLIC routines | Exact | Drags a C toolchain and build system into the kernel, plus `unsafe` FFI at the hottest path in the system. Valuable as a **reference** for register offsets, not as a runtime dependency | No |

### 5.3 Dependency impact: none

No manifest change is needed. The PLIC is new *code*, not a new
*dependency*:

```toml
[dependencies]
riscv      = "0.16"   # CSR access
riscv-rt   = "0.18"   # startup + trap entry
panic-halt = "0.2"
```

`riscv` 0.16 already ships what the driver needs — `riscv::register::{mstatus,
mie, mcause}` and `riscv::interrupt::machine::{enable, disable, free}` for the
global machine-mode mask. The vendor CSRs (`mxstatus`, `mmisc_ctl`) are *not*
in the `riscv` crate; reach them with raw `csrr`/`csrw` in the arch layer —
a handful of lines, no crate required.

One flag may become relevant later, but is **not** needed for the PLIC:
`riscv-rt`'s `custom-interrupts` feature, which excludes the generated
core-interrupt dispatch table and requires us to define
`_dispatch_core_interrupt` ourselves. The rule of thumb:

- Enable it when we want to own **core-cause** routing (AndeStar `mtvt`
  vectoring, per §2.4).
- Leave it off while we only own **device** routing — the PLIC sits *behind*
  `MachineExternal`, so the `#[core_interrupt]` macro is sufficient.

### 5.4 Where this code lands

Okapi follows Linux's structural conventions (folder taxonomy, driver model,
subsystem separation) while keeping RTOS semantics underneath — single
address space, static tasks, PMP-based protection rather than an MMU, since
the N25F has no MMU and 448K of RAM. Applied to the PLIC, that gives three
separate concerns rather than one file:

- **arch layer** (`arch/riscv/`) — trap entry glue and the Andes vendor CSRs.
  Bridges `riscv-rt`'s core-interrupt macro to the kernel's dispatch, and
  owns the nesting discipline. Vendor CSR access is isolated here so a
  non-Andes port swaps this layer alone.
- **driver layer** (`drivers/irqchip/` in Linux's taxonomy) — the Andes PLIC
  register map and nothing else: `priority[]`, `pending[]`, `enable[]`,
  `threshold`, `claim`/`complete`. No kernel types, no handler table.
- **kernel IRQ layer** (`kernel/irq`) — handler registration, `mask`/`unmask`,
  dispatch by IRQ id. This is what the scheduler and device drivers call.

**The one boundary that matters:** the PLIC driver knows register offsets and
nothing else; the kernel IRQ layer is what the rest of Okapi calls and
**never mentions the PLIC by name**. The consequence is that moving to a
different Andes part — or adding `PLIC_SW` for IPIs on a multicore variant —
changes one file.

**Status: this layout now exists.** The crate was promoted from
`src/okapi/` to the repo root and restructured along Linux's taxonomy, with
the PLIC/IRQ modules scaffolded and building:

```
arch/riscv/{mod,trap,andes}.rs      kernel/{irq,sched}/mod.rs
drivers/irqchip/andes_plic.rs       lib/mod.rs
include/okapi/mod.rs                init/main.rs
```

Register constants are placeholders carrying `TODO(bsp)` markers (§5.6); the
module boundaries and layering rules are real. See **`doc/conventions.md`**
for the full folder taxonomy, naming rules, and the governing precedence
rule — Linux supplies the structure, Rust supplies the language conventions,
and where they conflict Rust wins.

(The build system — whether these subsystems become Cargo workspace crates
or stay modules in one crate, and whether a Kconfig-style `defconfig` layer
is added — is deliberately still open. The *boundaries* above hold under any
of those choices, which is why it can wait.)

### 5.5 The dispatch path

The shape of the hot path, with the one detail that causes lost interrupts if
missed:

```rust
#[riscv_rt::core_interrupt(CoreInterrupt::MachineExternal)]
fn machine_external() {
    // loop, not a single claim: the PLIC can have several sources pending
    while let Some(irq) = plic::claim() {
        irq::dispatch(irq);
        plic::complete(irq);
    }
}
```

Several device sources can be pending behind the single `MachineExternal`
core line, and the PLIC deasserts it only once all of them have been
completed. A single `claim()` per trap would **silently drop the rest** — so
loop until the claim register returns id `0`.

The claim/complete handshake itself:

```rust
pub fn claim() -> Option<u32> {
    let id = unsafe { read_volatile(CLAIM) };
    (id != 0).then_some(id)              // 0 == nothing pending
}

pub fn complete(irq: u32) {
    unsafe { write_volatile(CLAIM, irq) }  // same register address
}
```

Reading the claim register atomically returns the highest-priority pending id
*and* clears its pending bit; writing the same id back to the complete
register tells the gateway it may forward the next one. Both use the same
address. **Claim id 0 means "nothing pending"** — that is the loop's exit
condition, and the reason `claim()` returns an `Option`.

### 5.6 What must be confirmed before writing register offsets

These come from the **core configuration**, not from the RISC-V PLIC
specification — the spec will give plausible-looking values that are wrong
for our part:

- **PLIC base address** for our specific N25F core build.
- **Whether `PLIC_SW` exists** in this configuration, and its base address.
  Determines whether the IRQ layer needs an IPI path now or later.
- **Vectored vs. non-vectored trap entry** — whether the core was configured
  with `mtvec.MODE=1`. This decides whether `riscv-rt`'s `v-trap` feature
  should be on, and changes the shape of the arch-layer trap code.
- **Maximum IRQ id / source count** — sizes the handler table and the
  enable-bit arrays.

Sources, in order of authority: (1) the Andes core configuration report for
this SoC build; (2) AndeSight BSP `nds_` headers, for the PLIC base macros;
(3) the N25F datasheet / AndeStar V5 manual; (4) existing C BSP init code —
read as a reference, not linked as a dependency (option D above).

None of this blocks the *structure*. The modules can be scaffolded against
the standard register layout with the base address as a single `const`, so
the tree compiles and links today:

```rust
// TODO(bsp): every value below needs confirmation against the
// Andes core config / AndeSight nds_ headers before hardware bring-up.
const PLIC_BASE: usize = 0xE400_0000;   // placeholder, not verified

const PRIORITY:  usize = PLIC_BASE + 0x0000_0000;
const PENDING:   usize = PLIC_BASE + 0x0000_1000;
const ENABLE:    usize = PLIC_BASE + 0x0000_2000;
const THRESHOLD: usize = PLIC_BASE + 0x0020_0000;
const CLAIM:     usize = PLIC_BASE + 0x0020_0004;
```

Those offsets are the **standard** PLIC layout used as a starting scaffold,
not verified Andes values. Correcting them later is a one-line change;
getting the layering right is not.

---

## 6. What `riscv-rt` 0.18 actually provides for interrupts

A claim worth testing came up: *"the `riscv-rt` crate does not provide any
interrupt handling."* Checked against the vendored source at
`~/.cargo/registry/src/index.crates.io-*/riscv-rt-0.18.0/`, this is **not
accurate** — `riscv-rt` 0.18 provides a complete generic interrupt-handling
path. The distinction that matters:

| Crate | Interrupt handling? |
|---|---|
| `riscv` 0.16 | **No.** CSR access and instruction wrappers only (`mstatus`, `mie`, `mcause`, `wfi`). Setting a bit in `mie` is a register write, not handling. |
| `riscv-rt` 0.18 | **Yes.** Installs the trap vector, decodes `mcause`, and dispatches to per-cause handlers through a generated table. |

This section records the verified flow, because the Andes integration steps
in §6.3 are defined entirely in terms of which part of it we replace.

### 6.1 The generic flow, as implemented

Four stages, all in crate code. Source references are to
`riscv-rt-0.18.0/src/` and `riscv-macros-0.4.1/src/`.

**Stage 1 — trap vector installation (`lib.rs:793`).**
`default_setup_interrupts` writes `mtvec` during startup, before `main`:

```rust
#[riscv_macros::setup_interrupts]
unsafe fn default_setup_interrupts() {
    let xtvec_val = match () {
        #[cfg(not(feature = "v-trap"))]
        _ => Xtvec::new(_start_trap as *const () as usize, TrapMode::Direct),
        #[cfg(feature = "v-trap")]
        _ => Xtvec::new(_vector_table as *const () as usize, TrapMode::Vectored),
    };
    xtvec::write(xtvec_val);
}
```

So the mode is a compile-time choice: **Direct** by default (one entry
point, software decodes the cause), or **Vectored** with `v-trap` (hardware
indexes a generated table).

**Stage 2 — cause decode (`lib.rs:889`).**
`_start_trap` assembly saves the trap frame and calls into Rust:

```rust
#[export_name = "_start_trap_rust"]
pub unsafe extern "C" fn start_trap_rust(trap_frame: *const TrapFrame) {
    match xcause::read().cause() {
        #[cfg(not(feature = "v-trap"))]
        xcause::Trap::Interrupt(code) => _dispatch_core_interrupt(code),
        #[cfg(feature = "v-trap")]
        xcause::Trap::Interrupt(_) => DefaultHandler(),
        xcause::Trap::Exception(code) => _dispatch_exception(&*trap_frame, code),
    }
}
```

Note the `v-trap` arm: in vectored mode hardware dispatches interrupts
directly, so reaching this path *with* an interrupt is abnormal and falls
through to `DefaultHandler`.

**Stage 3 — the dispatch table is generated, not hand-written
(`riscv-macros/src/riscv.rs:262`).**
`#[riscv::pac_enum(unsafe CoreInterruptNumber)]` applied to the standard
cause enum emits both the handler array and the dispatch function:

```rust
pub static __CORE_INTERRUPTS: [Option<unsafe extern "C" fn()>; N + 1] = [ ... ];

unsafe extern "C" fn _dispatch_core_interrupt(code: usize) {
    match __CORE_INTERRUPTS.get(code) {
        Some(Some(handler)) => handler(),
        _ => DefaultHandler(),
    }
}
```

The enum it is applied to (`interrupts.rs:24`) is the standard set, and
**nothing else**:

```rust
enum Interrupt {
    SupervisorSoft = 1,  MachineSoft = 3,
    SupervisorTimer = 5, MachineTimer = 7,
    SupervisorExternal = 9, MachineExternal = 11,
}
```

**Stage 4 — your handler.** `#[riscv_rt::core_interrupt(...)]` places a
function into the table slot for that cause.

### 6.2 Where the generic path stops

Two hard limits, both by design:

1. **Core causes only.** `__CORE_INTERRUPTS` is indexed by `mcause` code,
   and the enum tops out at `MachineExternal = 11`. Device sources behind
   the PLIC have no representation here — `MachineExternal` is a *single*
   slot meaning "some external device fired."
2. **No controller driver.** Nothing in the crate reads a PLIC claim
   register or knows a base address. `riscv-rt` never touches the
   interrupt controller.

There is, however, a **second dispatch hook** intended precisely for this
gap. The macro crate defines an external-interrupt path alongside the core
one (`riscv-macros/src/riscv.rs:226-236`):

| | Core interrupts | External interrupts |
|---|---|---|
| Handler array | `__CORE_INTERRUPTS` | `__EXTERNAL_INTERRUPTS` |
| Dispatch fn | `_dispatch_core_interrupt` | `_dispatch_external_interrupt` |
| Attribute | `#[riscv_rt::core_interrupt(..)]` | `#[riscv_rt::external_interrupt(..)]` |
| Marker trait | `CoreInterruptNumber` | `ExternalInterruptNumber` |

`_dispatch_external_interrupt` is **not** called by `riscv-rt` itself — the
crate provides the plumbing and the attribute, but something must invoke it
from the `MachineExternal` handler. That something is the PLIC driver. This
is the designed seam for exactly our case.

### 6.3 Steps to enable the Andes PLIC on top of this

Given the above, enabling the PLIC is additive — no fork, no patch to
`riscv-rt` (§5.2). Concretely:

**Step 1 — confirm the trap mode.** Check whether the core is configured
for vectored entry (`mtvec.MODE`). If non-vectored, leave `v-trap` off and
the Direct path above applies unchanged. If vectored, `v-trap` changes
stage 1-2 and the handler in step 3 must be reached via the vector table
instead. This gates everything else (§5.6).

**Step 2 — define the Andes IRQ source enum.** The device sources are
Andes-specific, so declare them and mark them as external interrupts:

```rust
#[riscv::pac_enum(unsafe ExternalInterruptNumber)]
enum AndesIrq {
    Uart0 = 1,
    Gpio  = 2,
    // ... from the core configuration report
}
```

This generates `__EXTERNAL_INTERRUPTS` and `_dispatch_external_interrupt`
for *our* source list, using the same machinery the crate uses for core
causes.

**Step 3 — claim the `MachineExternal` slot.** Take over the one core cause
that the PLIC sits behind, and drive the claim/complete loop from it:

```rust
#[riscv_rt::core_interrupt(CoreInterrupt::MachineExternal)]
fn machine_external() {
    while let Some(irq) = plic::claim() {      // our driver
        unsafe { _dispatch_external_interrupt(irq as usize) };
        plic::complete(irq);                   // our driver
    }
}
```

The loop (not a single claim) is required — see §5.5.

**Step 4 — write the PLIC driver** (`drivers/irqchip/andes_plic.rs`):
`init`, `set_priority`, `enable`, `threshold`, `claim`, `complete` against
the confirmed base addresses. This is the part being implemented by hand,
and the only part no crate provides.

**Step 5 — initialize in order** from `init/main.rs`: set the PLIC
threshold, set per-source priorities, enable the sources, then enable
`mie.MEIE` and finally `mstatus.MIE` via
`riscv::interrupt::machine::enable()`. Global enable comes **last**.

**Step 6 — only if steps 1-5 prove insufficient**, reach for
`custom-interrupts` to replace `_dispatch_core_interrupt` wholesale. That is
needed only if the Andes *core-cause* numbering diverges from the standard
enum in §6.1, or for `mtvt`-based AndeStar vectoring (§2.4) — **not** for
device routing, which steps 2-4 already cover.

### 6.4 Consequence for the §5 decision

This verification *strengthens* §5 rather than changing it. `riscv-rt`
provides the generic half (vector install, cause decode, dispatch tables,
and an external-interrupt hook) and deliberately omits the vendor half (the
controller driver, vendor CSRs, nesting policy). The split lands exactly on
the boundary Okapi already draws: crate code up to `MachineExternal`, Okapi
code from there down.

---

## 7. The machine timer — the easy path through the same machinery

`MachineTimer` (cause 7) is a core interrupt, so it sits in the same
`__CORE_INTERRUPTS` table as `MachineExternal` (§6.1). It is worth treating
separately because it is the **simplest useful interrupt to bring up**: one
source, no controller to demultiplex, and no dependency on the PLIC base
addresses still outstanding in §5.6.

### 7.1 What `riscv-rt` does and does not do for it

Verified against `riscv-rt-0.18.0/src/`:

| Concern | Provided by `riscv-rt`? |
|---|---|
| Slot for `MachineTimer` (cause 7) in `__CORE_INTERRUPTS` | **Yes** (`interrupts.rs:24`) |
| Routing the trap to our handler | **Yes** (`lib.rs:889` → generated dispatch) |
| `#[core_interrupt]` attribute to register one | **Yes** |
| Programming `mtimecmp` (setting the deadline) | **No** |
| Enabling `mie.MTIE` | **No** — it *clears* `mie` at reset |
| Enabling `mstatus.MIE` | **No** |
| Knowing the CLINT/PLMT base address | **No** |

The negatives were checked, not assumed: `grep -rni "mtimecmp|mtime|clint"`
over the crate's `src/` returns **nothing**. And `asm.rs:74` issues
`csrw mie, 0` during reset, so every interrupt source starts masked.

**Consequence: without our code, a machine timer interrupt never fires.**
The dispatch path is complete and waiting; nothing arms it. `riscv-rt`
handles the *plumbing*, never the *peripheral* — the same split as the PLIC
(§6.4).

### 7.2 The pieces Okapi must supply

Four, in dependency order. Only the first is trivial.

1. **A handler**, registered with
   `#[riscv_rt::core_interrupt(CoreInterrupt::MachineTimer)]`. This is all
   `riscv-rt` requires of us.
2. **A timer driver** — read `mtime`, write `mtimecmp`. New device class, so
   `drivers/timer/` per the taxonomy in `conventions.md` §1. It does **not**
   belong in `irqchip/`: that directory is for interrupt controllers, and the
   timer is a timer that happens to raise an interrupt.
3. **Initialization order** in `init/main.rs`: arm the first deadline, unmask
   the source (`mie.MTIE`), then enable interrupts globally
   (`mstatus.MIE`) — **global enable last**, so a pending interrupt cannot
   fire before the handler's state is ready.
4. **A `DefaultHandler`** (§7.3 below), so an unexpected cause is diagnosable
   rather than silently fatal.

### 7.3 `DefaultHandler` is declared but not defined

`riscv-rt` only *declares* `DefaultHandler` (`lib.rs:895`); it never provides
a body. The crate documentation states:

> If `DefaultHandler` is not defined, the linker will use the `abort`
> function instead.

This is observable in the current Okapi binary — `MachineExternal` and
`DefaultHandler` resolve to the same address, because no handler is
registered yet:

```
ff0001e0 T DefaultHandler
ff0001e0 T MachineExternal
ff0035c0 R __CORE_INTERRUPTS
```

So an unhandled interrupt **aborts**. Defining our own `DefaultHandler`
early is worth doing for bring-up: it converts "the board died" into
"cause N fired and nothing handled it", which is a far better starting point
for diagnosis.

### 7.4 The level-triggered trap — the main bring-up hazard

The machine timer has **no claim/complete handshake**. It is
level-triggered: `mip.MTIP` stays asserted for as long as
`mtime >= mtimecmp`. There is no acknowledge register — **pushing
`mtimecmp` forward is the acknowledgement.**

If the handler does not rewrite `mtimecmp`, the interrupt re-fires
immediately and forever. The symptom is a board that appears hung, which
looks nothing like the actual cause. This is the single most common mistake
on this path, and it is the key behavioural difference from the PLIC:

| | Machine timer | PLIC (external) |
|---|---|---|
| Sources behind the cause | 1 | many |
| Acknowledgement | rewrite `mtimecmp` | `claim` / `complete` |
| Trigger style | level | gateway-latched |
| Handler shape | do work, re-arm | loop until claim returns 0 |

### 7.5 RV32-specific caveat: `mtimecmp` is 64-bit

On a 32-bit core, `mtimecmp` cannot be written atomically — it takes two
32-bit stores, and a deadline can be momentarily bogus between them,
producing a spurious interrupt.

The standard safe sequence is: write the low half to all-ones, then write
the high half, then write the real low half. When the tick interval is short
and no 32-bit boundary is crossed, a single low-half store is often
sufficient in practice — but the three-step form is the correct one and
should be what the driver implements.

### 7.6 Andes naming

Andes calls this block **PLMT** (platform-level machine timer), not CLINT —
consistent with §2.4. The register layout is PLMT's own, so the SiFive CLINT
maps that appear in most RISC-V examples do not apply (§7.9).

### 7.7 Dependencies — what the timer actually needs

Worth enumerating before starting, because two of these are easy to conflate
and one of them fails silently when wrong.

**Clock (see §7.8 for detail).** The timebase frequency, which is *not* the
CPU clock.

**Hardware access:**

| Need | Why |
|---|---|
| PLMT base address | From core config, not the spec |
| Memory-mapped **or** CSR? | Changes the driver's shape entirely: volatile loads vs. `csrr` |
| `mtime` / `mtimecmp` offsets | Within the PLMT block |
| Per-hart layout | `mtimecmp` is per-hart; is it `base + 0x8`, or indexed by hart id? |
| Is `mtime` readable? | Some configs expose only `mtimecmp` |
| 64-bit presentation on RV32 | One 64-bit location, or split `mtimelo`/`mtimehi`? |

**Core/ISA:** `mtvec.MODE` — vectored or direct, which decides whether
`v-trap` is enabled and how the handler is reached (§6.3 step 1).

**Software, inside Okapi:**

| Dependency | Status |
|---|---|
| `riscv-rt` trap dispatch | Available (§6.1) |
| `riscv::register::{mie, mstatus}` | Available in `riscv` 0.16 |
| 64-bit math on RV32 | Free via `core` — LLVM intrinsics, no `libgcc` needed |
| `drivers/timer/andes_plmt.rs` | Scaffolded, not implemented |
| `arch/riscv/trap.rs` handler | Scaffolded |
| `DefaultHandler` | **Not defined — currently aborts** (§7.3) |
| `kernel/sched::tick()` | Does not exist — **not a blocker**, see below |

Note that 64-bit arithmetic on a 32-bit core means every `mtime` read and
compare is a multi-instruction sequence. The compiler handles it, but it is
not free, and this code runs in an ISR.

**`kernel::sched::tick()` is deliberately not on the critical path.** The
first milestone should be a counter increment, not a scheduler — increment a
`static` in the handler and observe it in a debugger. That separates "does
the interrupt fire?" from "does the scheduler work?", the same isolation
argument as doing the timer before the PLIC (§7.10).

**Tooling, to observe success:** a debugger (OpenOCD + GDB, or AndeSight) to
inspect the counter and read `mtime`/`mcause`. A UART would also work but is
a second unproven driver, so debugger-first is the better order. On QEMU, see
the caveat in §7.9.

**The actual critical path** is only two items: the **PLMT base address**
(plus MMIO-vs-CSR) and the **timebase frequency**. Everything else is either
already available or under our control. Both come from the same document as
the PLIC unknowns (§5.6), so one lookup unblocks both.

### 7.8 Clocks — the reference clock, and the other two

This is the dependency most likely to be got wrong, because "the clock" is
really three different things. The timebase is what §7 needs; the other two
govern the UART divisor (§7.8.7), delay loops anywhere in the tree (§7.8.6),
and the clock-setup ordering in `init/` (§7.8.8).

#### 7.8.0 Why there are three clocks at all

The physical topology is the reason, and the rest follows from it:

```
                      +--> PLL ---> CPU clock      (mcycle, instruction rate)
  XTAL --+            |
         +--> clkgen --+--> /N ----> bus clock      (MMIO, UART divisors)
                       |
                       +---------->  timebase       (mtime)  <-- what §7 needs
     (or a second, slower always-on XTAL / RTC feeds the timebase directly)
```

One crystal is multiplied up by a PLL for the core, divided down for the
peripheral bus, and the timer is fed from the raw crystal or from a separate
always-on branch. They differ **by design**, not by accident: the timebase
must keep counting when the core is slowed, clock-gated or asleep, otherwise
time stops whenever the CPU idles. That requirement is what forces it into
its own slow, always-on domain — which is precisely why it cannot be the CPU
clock (§7.8.1).

#### 7.8.1 Timebase frequency is not the CPU frequency

`mtime` is **not** driven by the CPU clock. On most RISC-V designs it
increments from a separate, slower reference — typically an external crystal
or a divided always-on bus clock, commonly in the 32.768 kHz to 50 MHz range.
Three distinct values:

| Value | Drives | Used for |
|---|---|---|
| **Timebase frequency** | `mtime` increment rate | **All timer deadlines** |
| CPU clock | Instruction execution, `mcycle` | Delay loops, cycle counting |
| Bus/peripheral clock | Peripheral registers | UART divisors etc. |

Only the first matters for `mtimecmp`. Linux encodes exactly this distinction
in the device tree — `timebase-frequency` on the CPU node, separate from
`clock-frequency` — precisely because the two commonly differ.

One further property of the timebase worth stating, because it is what makes
it the right basis for the tick: `mtime` is **monotonic and never reset by
software**. It counts from power-on and keeps going. That is why it answers
"how long has it been" correctly, and why only `mtimecmp` is ours to write —
never zero `mtime` to start a measurement.

The tick interval follows from the timebase alone:

```
TICK_INTERVAL = TIMEBASE_HZ / TICK_HZ

e.g. timebase 1 MHz, desired 1 kHz tick  ->  1_000_000 / 1_000 = 1_000
```

**The failure mode is silent.** Use the CPU frequency (say 100 MHz) where the
timebase is 1 MHz and every interval is 100× too long: nothing errors, no
exception fires, the system simply runs at the wrong rate. A 1 ms tick
becomes 100 ms. This is the single most common way to get a "working" timer
that is wrong, and it is why the value deserves a named constant with its
provenance recorded.

#### 7.8.2 Where the value comes from

In order of authority:

1. **Andes core configuration report** for this SoC build — the definitive
   source; look for *timebase*, *RTC clock*, or *PLMT clock* rather than
   "CPU frequency".
2. **AndeSight BSP headers** — often a `KERNEL_TIMER_CLK`-style or
   `MTIME_FREQ`-style macro.
3. **Board schematic / datasheet** — identifies the physical crystal feeding
   the timer, which is frequently a separate part from the CPU oscillator.
4. **Measure it** (§7.8.4) — the fallback when documentation is absent or
   suspect, and a worthwhile cross-check even when it is not.

#### 7.8.3 Is it fixed, or does it need setting up?

Three possibilities, and which one applies must be confirmed:

- **Fixed by hardware.** Crystal straight into the timer; nothing to
  configure. Record the constant and move on. Most likely case.
- **Divided from a faster clock.** The PLMT (or a clock-control block) has a
  divider. Then you need either the reset value of that divider, or to
  program it during `init()`. If programmable, do it *before* arming the
  first deadline — changing the divider afterwards invalidates any pending
  `mtimecmp`.
- **Gated off at reset.** Some SoCs hold peripheral clocks disabled until
  software enables them in a clock/power-control register. If so, `mtime`
  reads back **zero and never advances** — a distinctive symptom worth
  recognising, since it looks like a dead timer rather than a clocking
  problem.

Because of the third case, the driver's `init()` should make the "is it
running?" check explicit rather than assume:

```
// shape only -- read mtime twice with a delay between, confirm it advanced.
// If it never advances: clock gated, divider misconfigured, or wrong base
// address. Three very different fixes, so distinguish them early.
```

#### 7.8.4 Verifying the value empirically

Do this once, before trusting any documented number. Two approaches:

**Against a known interval** — if any independent time reference exists (a
scope on a GPIO toggle, a stopwatch for long intervals), set a deadline of
*N* ticks, toggle a pin or increment a counter in the handler, and compare
measured elapsed time to the expectation. A consistent integer ratio
(100×, 8×, 2×) between expected and measured is the signature of using the
wrong clock, and the ratio usually identifies which one.

**Against the CPU clock** — if the CPU frequency is known and trusted, spin a
calibrated instruction loop and sample `mtime` before and after. This gives
the timebase as a fraction of the CPU clock, which is enough to identify a
divider.

Record whatever is concluded as a single documented constant:

```
// Timebase for mtime. NOT the CPU clock (§7.8.1).
// Source: <core config report / BSP macro / measured on <date>>
const TIMEBASE_HZ: u64 = ...;
```

#### 7.8.5 Checklist

- [ ] Timebase frequency identified, and confirmed to be the *timer's* clock
      rather than the CPU's
- [ ] Source of that number recorded (document, BSP macro, or measurement)
- [ ] Fixed / divided / gated established (§7.8.3)
- [ ] If divided: divider reset value known, or programmed in `init()`
- [ ] If gated: clock enabled before first use
- [ ] `mtime` observed actually advancing
- [ ] Tick interval computed from the timebase, not the CPU clock
- [ ] Value verified empirically at least once (§7.8.4)
- [ ] Bus/peripheral clock identified separately, for UART and other
      divisors (§7.8.7)
- [ ] Per-peripheral clock gates enabled before first register access
      (§7.8.7)
- [ ] Clock setup ordered before any divisor or deadline is derived
      (§7.8.8)

#### 7.8.6 The CPU clock — `mcycle` / `mcycleh`

A separate counter in a separate domain, for a different purpose. Worth
stating explicitly, because it is the clock reached for by mistake when the
timebase is what was meant.

| | `mtime` (§7.8.1) | `mcycle` |
|---|---|---|
| Rate | timebase — slow, fixed | CPU clock — fast, possibly variable |
| Access | MMIO or CSR (§7.7, TBD) | CSR `mcycle` / `mcycleh` |
| Stops when the core halts | No | **Yes** |
| Changes with PLL / DVFS | No | **Yes** |
| Reset by software | Never (§7.8.1) | Writable |
| Use for | deadlines, elapsed wall time | cycle counts, sub-tick delays, profiling |

What it is legitimately for:

- **Sub-tick delays.** A peripheral needing "wait 500 ns after reset
  deassert" is far below one timebase tick. `mcycle` has the resolution;
  `mtime` does not.
- **Profiling and cycle budgets.** How many cycles trap entry costs — `mtime`
  is too coarse to see it. Relevant here because the 64-bit arithmetic noted
  in §7.7 runs inside an ISR.
- **Calibrating the timebase** — the cross-check in §7.8.4: sample `mcycle`
  and `mtime` across the same interval, and their ratio exposes a divider.

Three hazards specific to it:

1. **It stops in debug halt.** Single-step in GDB and `mcycle` freezes while
   `mtime` keeps running, so an `mcycle` delay loop behaves differently under
   the debugger than in free-run. A confusing class of bug, and a good reason
   not to build timeouts on it — relevant given the debugger-first bring-up
   plan in §7.7.
2. **It is not valid across a frequency change.** Any PLL reprogram or DVFS
   step changes cycles-per-second mid-measurement, so a cycle count spanning
   that change means nothing. Re-derive any cycles-per-µs constant afterwards
   (§7.8.8).
3. **On RV32 it is a 64-bit counter in two CSRs**, so it has the same split
   problem as `mtime` — read high, read low, re-read high, and retry if high
   changed. The same shape as the `mtimecmp` write hazard in §7.5, inverted
   (read rather than write).

Note also that `mcycle` is not `minstret`: cycles versus retired
instructions. They diverge on any stall, and only `mcycle` tracks time.

#### 7.8.7 The bus / peripheral clock

The domain that bites once the timer works. It never affects `mtimecmp`, so
it stays invisible during timer bring-up and then matters immediately
afterwards, when the UART goes in.

Its job is **derived divisors**. A UART baud rate is not programmed in baud;
a divisor is computed from the peripheral clock:

```
divisor = PCLK_HZ / (16 * baud)      # 16x oversampling, typical
```

Get `PCLK_HZ` wrong and the symptom is characteristic: the UART initialises
cleanly, no error bit is set anywhere, and it emits **garbage bytes** —
framing noise, or plausible-looking wrong characters. As with the timebase
failure in §7.8.1 it is silent and wrong by a ratio, and a consistent integer
ratio between intended and actual baud points straight at using the wrong
domain for the divisor. The same applies to SPI prescalers, I²C timing and
watchdog periods: each takes its divisor from the bus clock — not the CPU
clock, and not the timebase.

Two wrinkles beyond the arithmetic:

- **Clock gating is per-peripheral.** The gating case in §7.8.3 generalises:
  the timer may be ungated at reset while the UART is not, and each block can
  need an enable bit in a clock-control register before its registers respond
  at all. Reads returning all-zeros or all-ones from a peripheral that should
  be present is the signature.
- **If the bus clock is PLL-derived, changing the PLL breaks every divisor
  downstream.** Raise the core frequency and the UART silently changes baud.
  This is the main argument for setting frequencies once in early boot,
  before any divisor is programmed (§7.8.8).

#### 7.8.8 The rule that ties the domains together, and `init/` ordering

> A derived value is invalidated by a change in **its own** domain, and only
> its own.

`TICK_INTERVAL` depends on the timebase; a UART divisor depends on the bus
clock; a cycles-per-µs constant depends on the CPU clock. Changing one domain
invalidates its own derived values and leaves the others alone. That gives a
concrete ordering constraint for `init/`, which owns ordering
(`doc/conventions.md` §7, rule 5):

1. **Clocks first** — PLL, dividers, peripheral gates — *before* any divisor
   or deadline is computed from them.
2. **Then** program divisors and arm the first `mtimecmp`. The §7.2 enable
   order still applies on top: arm the deadline, then `mie.MTIE`, then
   `mstatus.MIE` last.
3. If a frequency changes later, re-derive every value in that domain.

Note this is a stronger statement than §7.8.3's "program the divider before
arming the first deadline" — that is the timebase instance of the same rule.

#### 7.8.9 Diagnosing the wrong clock

All three domains fail silently, so they are distinguishable mainly by
symptom. Worth keeping to hand during bring-up:

| Symptom | Domain | Likely cause |
|---|---|---|
| `mtime` reads 0, never advances | timebase | clock gated, or wrong base address (§7.8.3) |
| Tick fires, but at an integer-ratio wrong rate | timebase | CPU frequency used as the timebase (§7.8.1) |
| Delay loop differs under debugger vs free-run | CPU | `mcycle` halts with the core (§7.8.6) |
| Cycle timing nonsense after a PLL change | CPU | measurement spanned a frequency change (§7.8.6) |
| UART emits garbage, no error flags | bus | wrong `PCLK_HZ` in the divisor (§7.8.7) |
| Peripheral registers read all-0 / all-1 | bus | peripheral clock gate not enabled (§7.8.7) |

An integer ratio between expected and observed is the fingerprint of a
wrong-domain error in every case, and the ratio itself usually identifies
which clock was substituted.

### 7.9 Testing on QEMU — useful, but not for the register map

QEMU's `virt` machine implements a **standard SiFive-style CLINT** at the
conventional base address, not the Andes PLMT. The consequence:

- QEMU **can** validate the trap plumbing, handler logic, re-arming
  discipline (§7.4), the 64-bit write sequence (§7.5), and the `mie`/
  `mstatus` enable order;
- QEMU **cannot** validate the Andes base address, register offsets, or the
  timebase frequency.

This is worth exploiting rather than ignoring: the logic can be de-risked
before hardware access — but only if the base address and timebase stay
behind named constants instead of being scattered as literals through the
driver. Keep them configurable and the same code serves both targets.

### 7.10 Why bring this up before the PLIC

The timer exercises `mtvec` installation, trap entry, cause dispatch, and
the `mie`/`mstatus` enable sequence — the entire §6.1 flow — *without*
needing a controller driver or the PLIC addresses we are still waiting on:

```
MachineTimer    (cause 7)  → one source, no controller → handler directly
MachineExternal (cause 11) → N sources, PLIC demuxes   → claim loop → device
```

Getting the timer working first isolates "is our trap plumbing correct?"
from "is our PLIC register map correct?" — two failures that are painful to
debug simultaneously. It also delivers the scheduler tick, which is the
first thing the kernel actually needs.

---

## 8. FAQ — details that came up while reading the crate source

Questions that arose while working through §6. Kept separate from the flow
description because these are *why it is built this way* rather than *what it
does*.

### 8.1 What does `__CORE_INTERRUPTS`' type actually mean?

```rust
pub static __CORE_INTERRUPTS: [Option<unsafe extern "C" fn()>; N + 1] = [ ... ];
```

Read inside-out:

| Piece | Meaning |
|---|---|
| `fn()` | Takes no arguments, returns nothing |
| `extern "C"` | Uses the platform ABI, not Rust's (§7.3) |
| `unsafe` | Calling it is unsafe — it is an ISR, running in interrupt context |
| `unsafe extern "C" fn()` | A function **pointer** type (4 bytes on RV32), not a body |
| `Option<...>` | `Some(ptr)` = handler registered; `None` = nothing registered |
| `[T; N + 1]` | Fixed-size array, length fixed at compile time |
| `static` | Fixed address in the binary, typically `.rodata` |

In one line: **an array of optional function pointers, indexed by `mcause`
code.** A direct-indexed jump table.

### 8.2 Why `N + 1`, and why the gaps?

`N` is the largest discriminant in the cause enum (§6.1), i.e.
`MachineExternal = 11`. The array is therefore `[_; 12]`, because indices
`0..=11` inclusive need 12 slots — the `+1` converts "highest index" into
"length". In the macro source this is literally `#max_discriminant + 1`.

Expanded, it looks like this:

```rust
pub static __CORE_INTERRUPTS: [Option<unsafe extern "C" fn()>; 12] = [
    None,                     // 0  — no machine interrupt at this code
    Some(SupervisorSoft),     // 1
    None,                     // 2
    Some(MachineSoft),        // 3
    None,                     // 4
    Some(SupervisorTimer),    // 5
    None,                     // 6
    Some(MachineTimer),       // 7
    None,                     // 8
    Some(SupervisorExternal), // 9
    None,                     // 10
    Some(MachineExternal),    // 11  ← the PLIC sits behind this one
];
```

The RISC-V spec assigns no interrupt to codes 0, 2, 4, 6, 8, 10, but the
array must stay contiguous for O(1) indexing, so those slots hold `None`.
A few wasted words of `.rodata` buy a lookup with no search and no branching
over a list — which matters in interrupt context.

### 8.3 Why must the handlers be `extern "C"`? They are just ISRs.

`extern "C"` is doing **two** jobs here, and the ABI one is almost a side
effect of the more important one.

**Job 1 — the symbol must be findable by name.** This is the real reason.
`riscv-rt` is compiled *before* Okapi exists, so it cannot `use` our handler;
it has no idea which module the function lives in. All it can do is declare
the symbol and let the linker resolve it:

```rust
// generated inside riscv-rt — a different crate
extern "C" {
    fn MachineExternal();        // a name only, no definition
}
pub static __CORE_INTERRUPTS: [...] = [ /* ... */ Some(MachineExternal) ];
```

That requires a **stable, predictable** symbol name. Rust normally mangles
names, including a hash of crate, module path and types:

```
_ZN5okapi4arch5riscv4trap16machine_external17h8f3a2b1c9d4e5f6aE
```

That hash is not knowable in advance and changes if the module is renamed or
the crate version bumps, so no declaration in `riscv-rt` could ever match it.
`extern "C"` — together with the `#[no_mangle]`/`#[export_name]` the macro
applies — opts out of mangling, and the symbol becomes exactly
`MachineExternal`. The linker can then match declaration to definition.

Inspect it on a real build with:

```bash
nm target/riscv32imac-unknown-none-elf/debug/okapi | grep -i trap
```

**Job 2 — assembly calls into Rust directly.** The trap entry point is
hand-written assembly in `riscv-rt`'s `asm.rs`:

```asm
_start_trap:
    # ... save the trap frame ...
    jal  _start_trap_rust      # assembly calling Rust
```

Assembly knows nothing of Rust's ABI; it follows the RISC-V calling
convention — first argument in `a0`, return address in `ra`, defined
caller/callee-saved rules. Rust's own ABI is **deliberately unspecified**:
the compiler may pass arguments in different registers or change between
releases. For Rust-to-Rust calls that is fine because the compiler sees both
sides; for assembly-to-Rust it would be silent register corruption.

This is why the entry point is declared as:

```rust
#[export_name = "_start_trap_rust"]
pub unsafe extern "C" fn start_trap_rust(trap_frame: *const TrapFrame)
```

`extern "C"` guarantees `trap_frame` arrives in `a0` — exactly where the
assembly left it.

Summarised:

| Attribute | Job |
|---|---|
| `#[no_mangle]` / `#[export_name]` | Predictable symbol name → the linker can find it |
| `extern "C"` | Predictable calling convention → assembly can call it |
| `unsafe` | Marks that calling it arbitrarily is not safe |

Note that `"C"` is slightly misleading: no C is involved. It means *"use the
platform's standard documented ABI rather than Rust's private one"* — here,
the RISC-V calling convention.

### 8.4 Why do the function *pointers* need `extern "C"` as well?

Because in Rust the ABI is part of a function pointer's **type**. `fn()` and
`extern "C" fn()` are distinct, incompatible types. Since the handlers are
declared `extern "C"`, the array holding them must be
`Option<unsafe extern "C" fn()>` — otherwise it would not typecheck, and more
importantly the indirect call in the dispatcher

```rust
Some(Some(handler)) => handler(),
```

would emit the wrong calling sequence.

### 8.5 Why `Option<fn()>` instead of a null pointer like C?

C would use a nullable `void (*)(void)` and test it by hand. Rust gets the
same representation for free: `Option<T>` where `T` is a function pointer is
**niche-optimised** — a function pointer can never be null, so `None` is
represented *as* null. `Option<unsafe extern "C" fn()>` is therefore 4 bytes
on RV32, byte-identical to C's nullable pointer, but the type system forces
the check instead of leaving it to discipline.

### 8.6 What is the double `Some` in the dispatcher doing?

```rust
match __CORE_INTERRUPTS.get(code) {
    Some(Some(handler)) => handler(),
    _                   => DefaultHandler(),
}
```

The two layers mean different things, which is easy to misread:

- **Outer `Some`** — from `.get(code)`: the index was **in bounds**. Using
  `.get` rather than `[code]` means a bogus `mcause` returns `None` instead
  of panicking in interrupt context.
- **Inner `Some`** — from the array element: a handler was **actually
  registered** for that cause.

Either failing falls through to `DefaultHandler()`, so an out-of-range code
or an unhandled cause degrades safely rather than jumping to garbage.

### 8.7 When does any of this require action from us?

Normally never — `#[riscv_rt::core_interrupt(...)]` applies `#[no_mangle]`
and `extern "C"` on our behalf, so a handler is written as plain Rust.

It matters in exactly one case from §6.3 step 6: supplying a custom
dispatcher under the `custom-interrupts` feature. Then the signature must be
written by hand and must match what `riscv-rt` declares:

```rust
#[no_mangle]
unsafe extern "C" fn _dispatch_core_interrupt(code: usize) {
    // Andes-aware routing
}
```

Worth noting the asymmetry in failure modes: getting the **name** wrong
produces an undefined-symbol link error — loud and trivial to fix. Getting
the **ABI** wrong (omitting `extern "C"`) may link successfully and then
corrupt registers at runtime, which is far harder to diagnose. That is the
reason to understand this rather than copy it.

### 8.8 The machine timer is a core interrupt — does `riscv-rt` handle it?

**Partly, and the split is the same as for the PLIC.** It handles the
*dispatch* of `MachineTimer` (cause 7) — the table slot, the routing, the
`#[core_interrupt]` attribute — but nothing about the timer peripheral:
`mtimecmp` is never programmed, `mie.MTIE` is never set, and the crate does
not know the CLINT/PLMT base address. `grep -rni "mtimecmp|mtime|clint"`
over the crate `src/` returns nothing, and `asm.rs:74` does `csrw mie, 0` at
reset, so all sources start masked.

Net effect: **without our code the timer interrupt never fires.** Full
breakdown in §7.1.

### 8.9 Why does my timer interrupt fire continuously / the board appear hung?

Almost certainly because the handler did not rewrite `mtimecmp`. The machine
timer is **level-triggered** with no claim/complete handshake: `mip.MTIP`
stays asserted while `mtime >= mtimecmp`, so pushing the deadline forward
*is* the acknowledgement. Omit it and the interrupt re-fires immediately and
forever, which presents as a hang rather than as an interrupt storm. See
§7.4 — this is the most common mistake on this path, and the key behavioural
difference from the PLIC.

### 8.10 Why can't I just write `mtimecmp` in one store on RV32?

Because `mtimecmp` is 64-bit while the core's stores are 32-bit, so it takes
two of them, and between the two the deadline is momentarily a value you
never intended — which can raise a spurious interrupt. The safe sequence
writes the low half to all-ones first, then the high half, then the real low
half. §7.5 has the detail and the one case where a single store is usually
adequate.

### 8.11 Should the timer driver live in `drivers/irqchip/`?

No. `irqchip/` is for **interrupt controllers** — things that demultiplex
many sources, like the PLIC. The machine timer is a *timer* that happens to
raise an interrupt, so it belongs in its own class directory,
`drivers/timer/`, per the taxonomy rule in `conventions.md` §1 (directory
names the class, file names the vendor). Linux draws the same line:
`drivers/clocksource/` is separate from `drivers/irqchip/`.

---

## 9. Open items / follow-ups

These were identified during the discussion but require information not yet
available (no datasheet, no vendor BSP at the time):

- **Exact ISA extensions** for the specific N25F SoC instance in use (confirm
  A extension presence in particular — not guaranteed on MCU-class parts).
- **Real memory map** (FLASH/RAM base addresses and sizes) for `memory.x`.
- **AndeStar V5 interrupt controller details** — the *sourcing* question is
  settled (§5: own driver, no new crate) and the crate-side mechanism is now
  verified (§6), but the concrete values still need the SoC reference manual
  or Andes SDK: PLIC/`PLIC_SW` base addresses, `mtvec.MODE` configuration,
  and max IRQ id (§5.6). `mtvec.MODE` is the gate on §6.3 step 1 — it decides
  whether `v-trap` is enabled, which changes the trap-entry shape.
- **Andes core-cause numbering** — whether the Andes core uses the standard
  `mcause` codes in §6.1 or diverges. Only if it diverges (or if `mtvt`
  vectoring is pursued) does `custom-interrupts` become necessary; device
  routing alone does not need it (§6.3 step 6).
- **Hardware single-precision float** (`ilp32f` custom target) — deferred
  until after soft-float bring-up succeeds; not needed for initial boot.
- Whether Andes provides an official Rust-compatible BSP, reference linker
  script, or QEMU/AndeSim model that should be used instead of building the
  above from scratch.

---

## 10. Summary of recommended order of operations

1. Confirm exact ISA extensions from the SoC datasheet.
2. Set up `riscv32imac-unknown-none-elf` (soft-float) target and toolchain.
3. Get a minimal `#![no_std]` `main` linking and running end-to-end on QEMU's
   generic `virt` machine to validate the build pipeline.
4. Obtain the real memory map and write a correct `memory.x`.
5. **Bring up the machine timer first** (§7). One source, no controller, no
   dependency on the PLIC addresses — it validates `mtvec` install, trap
   entry, cause dispatch and the `mie`/`mstatus` enable order in isolation,
   and yields the scheduler tick. Define a real `DefaultHandler` at the same
   time (§7.3).
6. Then address the PLIC: confirm its base addresses and `mtvec.MODE` (§5.6)
   and write the driver plus kernel IRQ layer as Okapi's own code (§5, §6.3).
   By this point the trap path is already proven, so a failure here is
   unambiguously the PLIC register map.
7. Move to real Andes hardware/simulator for interrupt and timing validation.
8. Only once the above is stable, revisit hardware single-precision float via
   a custom target spec + nightly `-Z build-std`.
