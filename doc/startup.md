# RISC-V Rust Toolchain Setup — Andes N25F (32-bit)

Detailed setup notes for bringing up a `no_std` Rust environment targeting the
Andes N25F core (32-bit RISC-V). This expands on the summary slide in
`okapi.html` with the full reasoning and all options discussed.

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
riscv = "0.11"
riscv-rt = "0.12"   # startup/vector table, analogous to cortex-m-rt for ARM
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

## 5. Open items / follow-ups

These were identified during the discussion but require information not yet
available (no datasheet, no vendor BSP at the time):

- **Exact ISA extensions** for the specific N25F SoC instance in use (confirm
  A extension presence in particular — not guaranteed on MCU-class parts).
- **Real memory map** (FLASH/RAM base addresses and sizes) for `memory.x`.
- **AndeStar V5 interrupt controller details** — whether to patch `riscv-rt`
  or write a custom trap entry; needs the SoC reference manual or Andes SDK
  documentation to do correctly rather than by guesswork.
- **Hardware single-precision float** (`ilp32f` custom target) — deferred
  until after soft-float bring-up succeeds; not needed for initial boot.
- Whether Andes provides an official Rust-compatible BSP, reference linker
  script, or QEMU/AndeSim model that should be used instead of building the
  above from scratch.

---

## 6. Summary of recommended order of operations

1. Confirm exact ISA extensions from the SoC datasheet.
2. Set up `riscv32imac-unknown-none-elf` (soft-float) target and toolchain.
3. Get a minimal `#![no_std]` `main` linking and running end-to-end on QEMU's
   generic `virt` machine to validate the build pipeline.
4. Obtain the real memory map and write a correct `memory.x`.
5. Address the AndeStar V5 interrupt/trap model (patch `riscv-rt` or write a
   custom trap entry) before relying on interrupts.
6. Move to real Andes hardware/simulator for interrupt and timing validation.
7. Only once the above is stable, revisit hardware single-precision float via
   a custom target spec + nightly `-Z build-std`.
