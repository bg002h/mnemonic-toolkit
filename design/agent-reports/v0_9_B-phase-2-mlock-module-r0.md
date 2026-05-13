# v0.9.0 Cycle B Phase 2 R0 design pass (mlock module)

**Reviewer:** Opus 4.7 (1M context), invoked as design-review on Cycle B Phase 2 (mlock module) before any code lands.
**Date:** 2026-05-13.
**SPEC:** `design/SPEC_secret_memory_hygiene_v0_9_B.md` (master @ `f84d998`).
**Plan:** `~/.claude/plans/v0_9_B-secret-memory-hygiene-cycle-b.md`, §"Phase 2" (R0 draft, 2026-05-13).
**Phase 1 R1 (predecessor):** `design/agent-reports/v0_9_B-phase-1-bip85-heap-promote-r1.md` (CLEAR 0C/0I, commits `4465940`/`3be9b77`/`c3509af` atop `f84d998`).
**Scope of review:** Phase 2 design (crate-structure lock, allocator+page-size lock, unsafe discipline, Miri compatibility, test strategy, cfg(test) probe storage). No code reviewed (none exists yet).
**Verdict:** **RE-DRAFT — 1 Critical / 4 Important** at confidence ≥ 80. The single Critical is a SPEC-level shape defect: the SPEC's `MlockedZeroizing<Vec<u8>>` design pins only the 24-byte Vec header, **not the heap-allocated secret bytes**. Sites 2-4 as typed do not get the property the cycle is selling. Cycle B P2 cannot TDD-RED against this design until SPEC §2 row 1 / §4 P2 / P3a are reconciled. Crate-structure decision and other locks are detailed below for the post-reconcile re-draft.

---

## Summary

Total findings at confidence ≥ 80: **1 Critical / 4 Important / 3 Nit**.

The C-1 finding is the **MlockedZeroizing<Vec<u8>> indirection trap**: the SPEC instructs the wrapper to page-align-allocate `Box<T>`, then mlock `layout.size()` (≈ a page) starting at the Box's address. For `T = Vec<u8>`, the Box holds the Vec's 24-byte header (ptr + len + cap). The actual secret bytes live in a **second**, independently-allocated heap region — the one `Vec::with_capacity(64)` or `vec![0u8; 64]` requests from the global allocator. That second allocation is not page-aligned, is not under mlocked pages, and is freely swappable. SPEC Sites 2/3/4 all type `entropy: MlockedZeroizing<Vec<u8>>`, so 100% of the OWNED-buffer wrapper applications miss the threat model. The cycle would ship and the secret bytes would remain swappable; the Site 1 `pin_pages_for(buf: &[u8])` path is unaffected (it pins the buffer's own pages), but only Site 1 (clap fields, ~12 String fields) and Site 5 (ms-cli stdin String) would actually enjoy page-pinning. Sites 2-4 (Vec<u8>) would be cosmetic.

This invalidates the Phase 2 design as written. R0 cannot LOCK; SPEC §2 row 1, §4 P2, and §4 P3a require a fix. Concrete fix options enumerated in §3.

The 4 Important findings cover: (I-1) crate-structure decision — recommended **Option C** (hybrid: `lib.rs` exposing only `mlock`), with detailed rationale; (I-2) the toolkit has **no Rust CI workflow today** — Phase 2's Miri gate + G3 platform matrix add a workflow from scratch, not a job to an existing one (plan line 322 says "add to CI workflow" but there is no workflow); (I-3) macOS Apple Silicon page size is 16 KiB, not 4 KiB — Miri stub must use the runtime sysconf result, not a hard-coded 4096; (I-4) the cfg(test) drop-probe is **not reachable from integration tests** because cfg(test) is only true when the crate itself is being compiled as a test binary — the design must either use a Cargo feature flag or keep G4 as a unit test inside `src/mlock.rs`'s test module.

The 3 Nits are diff-manifest carve-outs the implementer should know about going in.

---

## §1. Crate-structure lock — Option C (hybrid: lib + binary, lib exposes ONLY `mlock`)

**Decision:** Adopt Option C — create `crates/mnemonic-toolkit/src/lib.rs` exposing `pub mod mlock;` and **only** that. The binary's other modules stay private to `main.rs`. The bin's `[[bin]]` target stays at `path = "src/main.rs"`; Cargo auto-detects `src/lib.rs` as the library target.

### Why not Option A (full hybrid)

Full hybrid would require auditing every existing `mod foo;` in `main.rs` (18 modules), deciding which to make `pub` in `lib.rs`, and reconciling the `use crate::error::ToolkitError` patterns currently in those modules. That's out of scope for Phase 2 (which is supposed to be ~270 LOC of mlock and nothing else). It also expands the public-API surface in a way that has cross-repo implications (manual mirror obligation per CLAUDE.md) for modules that have no reason to be public.

### Why not Option B (binary-private mlock module)

The plan's RED tests rely on `mnemonic_toolkit::mlock::*` integration-test access (plan line 201, line 219). Without a lib target, that path doesn't exist. The two B sub-options both have problems:

- **B1 (move tests inside `src/mlock.rs` `#[cfg(test)]`)**: forfeits the plan-envisioned "integration test surface" separation, BUT it actually solves the cfg(test) drop-probe reachability problem (I-4 below). However, the G1.1/G1.2 integration tests want to use `/proc/self/smaps` post-construction in a test that *also* observes the wrapper from outside — that's awkward to set up purely as unit tests inside the binary's test target without expanding `pub` surface accidentally.
- **B2 (subprocess via `cargo_bin("mnemonic")`)**: there is no production callsite that exercises mlock in Phase 2 (Phase 3a/3b are the apply phases). No CLI invocation triggers mlock, so subprocess testing requires a hidden `--test-mlock-pin` flag, which pollutes the CLI for the duration of a phase. Reject.

### Why Option C wins

- **Minimum cascade**: one new file (`lib.rs` with 1 line: `pub mod mlock;`), zero changes to existing module visibility, zero changes to existing tests.
- **Phase 2 reviewable in isolation**: the lib target adds exactly one public namespace (`mnemonic_toolkit::mlock`), trivially auditable.
- **No bin breakage**: `[[bin]] name = "mnemonic" path = "src/main.rs"` continues to work; `main.rs` adds `use mnemonic_toolkit::mlock;` (and `mlock::report_at_exit();` per plan line 244 + Phase 3a wiring). Cargo handles bin+lib coexistence natively (see `clap`, `bitcoin`, `ripgrep` precedents).
- **Phase 3a clean**: `cmd/*` modules in the binary import `use mnemonic_toolkit::mlock::pin_pages_for;` — same path the integration tests use. Single source of truth.
- **Phase 3b parity preserves**: ms-cli's inline copy lands at `crates/ms-cli/src/mlock.rs` (the plan already pinned this); whether ms-cli adopts the same lib+bin shape is the ms-cli reviewer's call at Phase 3b R0. Toolkit's choice does not lock ms-cli's.

### Cascade audit (what changes vs stays put)

| Item | Status |
|---|---|
| `Cargo.toml` `[[bin]]` clause | unchanged (still `name = "mnemonic" path = "src/main.rs"`) |
| New file: `crates/mnemonic-toolkit/src/lib.rs` | one line: `pub mod mlock;` |
| `crates/mnemonic-toolkit/src/main.rs` | one new `use mnemonic_toolkit::mlock;` import (or fully-qualified call sites); current `mod foo;` lines stay binary-private |
| `crates/mnemonic-toolkit/src/mlock.rs` | new file (Phase 2 implementation) |
| `manual.yml` / `quickstart.yml` | unchanged (these build docs, not the binary's `cargo test`) |
| `cargo publish` | not yet performed per CLAUDE.md ("git deps until they hit crates.io in lockstep with v0.1"); when it eventually happens, both lib + bin ship together — no special handling. |
| Existing tests (`tests/lint_*.rs`, `tests/cli_*.rs`) | unaffected; they `use assert_cmd::Command::cargo_bin("mnemonic")` (binary-shape) or `use std::fs` (file-reading lint shape), neither of which cares about library shape. |

**Verification anchor for R1 (Phase 2):** after `git add src/lib.rs src/mlock.rs`, `cargo build --tests -p mnemonic-toolkit` must report **two compile artifacts** (lib + bin); `cargo test -p mnemonic-toolkit` exercises both. The toolkit version stays at `0.9.2` for the Phase-2 push (no tag); the lib's first tag is `mnemonic-toolkit-v0.10.0` at PE.

Confidence: 92.

### Plan/SPEC fold required pre-Phase-2 RED commit

Plan line 205 says *"Add `pub mod mlock;` to `crates/mnemonic-toolkit/src/lib.rs`"* — assumes `lib.rs` exists, which it does not. The corrected wording is **"Create `crates/mnemonic-toolkit/src/lib.rs` with the single line `pub mod mlock;`; do not modify any other module's visibility."** The implementer of P2.T2 reads this report and folds.

Plan line 233 stages `crates/mnemonic-toolkit/src/lib.rs` for the RED commit — correct path; no fold needed on the staging step itself.

---

## §2. Plan/SPEC reconciliation — Important findings (folded inline)

### I-R0-1 — Crate has no Rust CI workflow today; Phase 2's Miri gate adds a workflow from scratch (Confidence: 95)

**Plan §"Phase 2" T4 line 322** says: *"Add to CI workflow: a Miri job that runs the same command (gated on a matrix key like `MIRI=1` to avoid blocking the main CI matrix on a nightly toolchain)."*

**Actual state:** `.github/workflows/` contains only `manual.yml` and `quickstart.yml` — both are PDF-build pipelines for the docs. **There is no `cargo test`, `cargo clippy`, `cargo build`, or Rust toolchain CI at all.** Grep confirms zero matches for `cargo (test|build|clippy|miri)` across `.github/`.

This has several Phase 2 implications:

1. **G3 platform coverage gate (SPEC §6 G3 lines 178-184)** — *"CI matrix runs on Ubuntu + macOS; both green required"* — is not satisfiable today. No matrix exists.
2. **Plan T4 "add a Miri job"** is not "add a job to existing workflow"; it's "add a workflow with cargo-test + clippy + miri matrix, including the Ubuntu/macOS matrix from G3, including the `ulimit -l ≥ 65536` step from G3 line 180". That's a substantially larger Phase 2 deliverable than line 322 suggests.
3. Phase 1 R1 noted no CI changes (Phase 1 was internal-only); so the absence of Rust CI is not a Phase-1-introduced regression — it's a pre-existing gap (the toolkit relies on local-dev `cargo test` discipline).

**Fold (corrected Phase 2 T4 scope):**

P2.T4 must create a new workflow file `.github/workflows/rust.yml` (or similar name) with three jobs minimum:

```yaml
jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]   # SPEC §6 G3
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - run: ulimit -l   # diagnostic
      - run: ulimit -l 65536 || true   # G3 line 180; macOS default unlimited, Linux may need setting
      - run: cargo build --tests -p mnemonic-toolkit
      - run: cargo test -p mnemonic-toolkit

  miri:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
        with:
          components: miri
      - run: cargo +nightly miri test -p mnemonic-toolkit mlock::

  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo clippy --all-targets -- -D warnings
```

The plan §"Phase 2" T4 must be re-prose'd to read **"Create `.github/workflows/rust.yml` with the test (matrix ubuntu/macOS), miri (ubuntu-only nightly), and clippy jobs."** This is roughly +25 LOC to the workflow file beyond what the plan estimates.

If the toolkit team wants to defer the full Rust-CI build-out to a separate cycle (orthogonal to Cycle B's mlock concern), the Phase 2 R0 reviewer can accept a **narrower scope**: just the `miri` job (one new workflow file with one job) covering SPEC §6 G4 only, with G3 (platform-matrix) explicitly punted to a follow-up FOLLOWUP entry filed in the Phase 2 RED commit. The reviewer's preference: **add all three jobs in Phase 2** because G3 is named in the SPEC and missing G3 would fail the cycle-close R1 at PE; better to land it now than discover the gap at PE.

### I-R0-2 — macOS Apple Silicon page size is 16 KiB, not 4 KiB; Miri stub must mirror runtime (Confidence: 95)

**Plan §"Phase 2" T3 step 7 line 287** says: *"Under `cfg(miri)`, stub `libc::mlock` / `libc::munlock` to no-ops; `_SC_PAGESIZE` returns 4096."*

**Issue:** Hard-coding 4096 in the Miri stub creates a divergence between Miri-test behavior and real-runtime behavior on macOS aarch64 (Apple Silicon), where `sysconf(_SC_PAGESIZE) == 16384`. The G1.4 page-aligned test (SPEC §6 line 156) asserts `page_count == 1` for an exactly-page-sized buffer; that test's input length is computed from `page_size()`, so Miri's 4096 vs runtime's 16384 makes the same test pass under different concrete sizes. The Miri test exercises the page-rounding formula, not the syscall — so:

**Fold (corrected Miri stub):** `_SC_PAGESIZE` returns 4096 **is acceptable** as a Miri-only constant because Miri runs on the host's CPU (x86_64 in CI) regardless of the runtime target, and 4096 is the universal x86_64 + Linux-aarch64 (typically) value. The plan's wording is fine for the Miri-environment. BUT: the **production code path** must not hard-code 4096 anywhere — it must come from `libc::sysconf(_SC_PAGESIZE)` so that macOS aarch64 binaries built natively pick up 16 KiB. The plan line 250 already gets this right (`fn page_size() -> usize` uses `libc::sysconf`).

**Action:** the implementer must verify that no test or code path conflates the Miri stub's 4096 with the runtime constant on macOS aarch64. The G1.1/G1.2/G1.4 tests should compute buffer sizes as multiples of `page_size()`, not hard-coded 4096. The plan does not say "use 4096 in the test bodies" — but the implementer should be explicit. Add a discipline note to P2.T2 line 222: *"All page-rounding tests express sizes as `n * page_size()`, never as `n * 4096`, so the same test passes on Linux x86_64 (4096) and macOS aarch64 (16384)."*

Confidence: 90.

### I-R0-3 — `lint_safety_third_party_blocked.rs` does NOT cover first-party `unsafe` blocks; Phase 2 needs explicit SAFETY-comment discipline (Confidence: 92)

**Plan §"Phase 2" T1 step 4 line 193** says: *"Each unsafe block carries a SAFETY comment per the project's existing `lint_safety_third_party_blocked.rs` discipline."*

**Issue:** I read `lint_safety_third_party_blocked.rs` in full (`tests/lint_safety_third_party_blocked.rs:1-135`). The lint's `CALL_PATTERNS` (lines 47-56) are: `Mnemonic::parse_in`, `Mnemonic::from_entropy_in`, `Xpriv::new_master`, `.derive_priv(`, `SecretKey::from_slice`. These are **third-party-blocked secret-bearing constructors**, not arbitrary `unsafe` blocks. The lint asserts a `SAFETY: third-party-blocked` doc-comment within ±15 lines of each call. The string `SAFETY: third-party-blocked` is the literal needle (line 58).

The first-party `unsafe` blocks Phase 2 will add (`std::alloc::alloc`, `std::ptr::write`, `libc::mlock`, `libc::munlock`, `Zeroize::zeroize` through a raw ptr, `std::alloc::dealloc`, `NonNull::as_ref`, `NonNull::as_mut`, `std::slice::from_raw_parts`) are **NOT** caught by this lint. They're a different category — first-party UB-management, not third-party-Zeroize-gap-documentation.

**Fold (Phase 2 discipline):** the implementer has two options:

1. **Extend `lint_safety_third_party_blocked.rs`** (or add a peer lint `lint_safety_first_party_unsafe.rs`) that scans `src/mlock.rs` for `unsafe {` opener tokens and asserts a `SAFETY:` doc-comment (different needle — just `SAFETY:`, since first-party unsafe is not "third-party-blocked") within ±5 lines above each opener. Discipline is enforced mechanically.
2. **Hand-discipline only** (no lint extension). Every `unsafe {` block in `mlock.rs` carries a `// SAFETY: ...` comment immediately above. The R1 reviewer audits each one manually. No lint enforcement → drift risk in Phase 3a/3b/PE follow-on edits.

**Recommendation: Option 1.** Add a peer lint `tests/lint_safety_first_party_mlock.rs` in Phase 2 P2.T2 (in the RED commit alongside the test module surface). It is RED until P2.T3 lands SAFETY comments inside `src/mlock.rs`. The pattern matches Cycle A's discipline (a SAFETY-comment lint catches drift mechanically; in v0.9.0 it was the third-party variant, in Cycle B it's the first-party variant). This protects the toolkit through Phase 3a where new `unsafe` blocks (if any) at apply sites land, and through cross-repo Phase 3b where ms-cli's inline copy must mirror the discipline.

This adds ~50 LOC of test discipline to Phase 2 beyond the plan's estimate. Worth it for the audit-trail mechanization.

Confidence: 90.

### I-R0-4 — cfg(test) drop-probe storage is unreachable from integration tests; G4 must use unit-test placement or a Cargo feature (Confidence: 95)

**SPEC §4 P2 line 91 + Plan §"Phase 2" Module Surface line 174-177 + G4 line 192-203** describe `new_with_drop_probe<F>` as `#[cfg(test)] pub fn` on `MlockedZeroizing<T>`, storing the probe in an `Option<Box<dyn FnOnce(&[u8])>>` field `#[cfg(test)]`-gated.

**Issue (subtle Cargo idiom):** `cfg(test)` is ONLY true when the crate itself is being compiled as a test target — i.e., for unit tests in `src/*.rs` (`#[cfg(test)] mod tests { ... }`). When an integration test in `tests/mlock_unit.rs` is compiled, **that** binary has `cfg(test)` set on its own code, but it links against the library `mnemonic-toolkit` (or `mnemonic_toolkit`) which is compiled **without** `cfg(test)`. So:

- `MlockedZeroizing::new_with_drop_probe` does NOT exist when `tests/mlock_unit.rs` calls it via `mnemonic_toolkit::mlock::MlockedZeroizing::new_with_drop_probe(...)`. The library was compiled without `cfg(test)`, the method was elided.
- The `probe: Option<Box<dyn FnOnce(&[u8])>>` field similarly does not exist on the library-side struct. Even if `new_with_drop_probe` were callable, there's nowhere to store the probe.

This is well-documented Rust idiom (see RFC 1604, "cfg(test) is per-crate-not-per-build"). The plan's G4 design is broken.

**Three fixes; recommend Option B:**

- **Option A (Cargo feature flag).** Add `[features] test-probe = []` to `Cargo.toml`. Replace every `#[cfg(test)]` on `new_with_drop_probe`, the `probe` field, and Drop's probe-invocation step with `#[cfg(any(test, feature = "test-probe"))]`. In `[dev-dependencies]` (or `[features]`'s default-features test layer), enable `test-probe` when building integration tests. Integration tests can then call `MlockedZeroizing::new_with_drop_probe`. This is the clean library-API solution but adds a Cargo feature surface that must be documented and tested in both states (with and without). Slight smell: the library now ships a public method `new_with_drop_probe` (under a feature) that's exposed to downstream consumers who happen to enable the feature.

- **Option B (G4 lives as a unit test inside `src/mlock.rs`'s `#[cfg(test)] mod tests`).** Move the drop-probe test from `tests/mlock_unit.rs` to a `#[cfg(test)] mod tests { ... }` block inside `src/mlock.rs`. Inside that block, `cfg(test)` IS true (it's the same compilation unit), and `new_with_drop_probe` is callable. The probe field exists. The test runs as part of `cargo test -p mnemonic-toolkit`. **This is the most cargo-idiomatic and lowest-cost fix.** It does not change the library's public API.

- **Option C (probe field always present, gated only the constructor).** Keep `probe: Option<Box<dyn FnOnce(&[u8])>>` always in the struct (no `#[cfg(test)]` gate on the field). Production builds carry a `None` field that costs `mem::size_of::<Option<Box<dyn FnOnce(&[u8])>>>() == 16 bytes` on 64-bit per `MlockedZeroizing<T>` instance. Constructor `new_with_drop_probe` remains `#[cfg(test)]` (test crates only). Drop-body's `if let Some(probe) = ...` is always present. Struct layout is identical across test and non-test, eliminating the `size_of` / `Layout` consistency concern raised in the prompt. Cost: 16 bytes per OWNED secret allocation in production. Sites 2-4 expect ≤4 active allocations at peak; 64 bytes total. Negligible.

**Recommendation: Option B** (unit-test placement). Reasoning:

- Most cargo-idiomatic; no new Cargo features, no library API expansion.
- Aligns with how `serde`, `bincode`, `rmp` test their internal unsafe (unit-test placement for internal-state-observation tests).
- The other tests in `tests/mlock_unit.rs` (G1, G2 happy-path) do NOT need the drop-probe; they observe through `/proc/self/smaps` (Linux) / `mach_vm_region_info` (macOS) using normal pub API. So only G4 moves to `src/mlock.rs`'s test module; G1/G2/G6 stay in `tests/mlock_unit.rs`.
- Adds zero library API surface beyond the documented `mlock::` pub items.
- Phase 3b ms-cli inline copy does not include `MlockedZeroizing<T>` (SPEC §6 G6 line 223), so this decision is toolkit-only.

**Fold (Phase 2 plan correction):**

Plan §"Phase 2" T2 line 218 — move `g4_drop_probe_observes_zeroed_buffer` test from `tests/mlock_unit.rs` to `crates/mnemonic-toolkit/src/mlock.rs`'s `#[cfg(test)] mod tests { ... }` block. Drop the integration-test entry; add a unit-test entry. Plan §"Phase 2" T3 step 6 stays as written (`#[cfg(test)] new_with_drop_probe`).

The cleanup of the plan's RED-test list (P2.T2 step 3):

| Test | Location post-fold |
|---|---|
| `g1_1_single_page_mlock_observable_via_smaps` (Linux) | `tests/mlock_unit.rs` (integration) |
| `g1_2_multi_page_mlock_observable_via_smaps` (Linux) | `tests/mlock_unit.rs` (integration) |
| `g1_3_zero_length_is_no_op_no_syscall_no_panic` | `tests/mlock_unit.rs` (integration) |
| `g1_4_page_aligned_slice_one_page_count` | `tests/mlock_unit.rs` (integration) |
| `g2_*` (5 subprocess-isolated tests) | see §6 — recommend deferring most to Phase 3a |
| `g4_drop_probe_observes_zeroed_buffer` | **`src/mlock.rs` `#[cfg(test)] mod tests`** (unit) |
| `g6_diff_manifest_*` placeholder | `tests/mlock_unit.rs`, gated `#[ignore]` until P3b |
| Unit tests (page-rounding, mlockstate, etc.) | `src/mlock.rs` `#[cfg(test)] mod tests` (already correct in plan) |

Confidence: 95.

---

## §3. Allocator + page-size design lock — **C-1 Critical: MlockedZeroizing<Vec<u8>> indirection trap**

### C-1: `MlockedZeroizing<Vec<u8>>` does NOT mlock the secret bytes (Confidence: 100)

**Statement of the trap.** SPEC §2 row 1 specifies `MlockedZeroizing<T>` as a wrapper that *"page-aligned allocator for an owned Box<T>"*. Plan §"Phase 2" T3 step 4 line 264 codifies: `Layout::from_size_align(size_of::<T>(), page_size())`. SPEC §4 P2 line 84 reinforces: *"owns a page-aligned Box<T> directly... (ii) `Zeroize::zeroize(&mut *self.value)` in place via `unsafe` deref of the owned ptr"*. The mlock'd region in `Drop` step (i) is `self.layout.size()` bytes starting at the owned-Box pointer (plan line 272).

For `T = Vec<u8>`:

- `size_of::<Vec<u8>>() == 24` on 64-bit (three machine words: `ptr`, `len`, `cap`). The Layout requests 24 bytes at page alignment; the actual allocation rounds up to one page (4 KiB on x86_64 Linux, 16 KiB on macOS aarch64).
- After `std::ptr::write(ptr.cast(), value)`, the Vec's header (24 bytes) sits at `ptr`. The Vec's **data buffer** — the actual `vec![0u8; 64]` bytes that the BIP-85 entropy or BIP-39 entropy gets copied into — is allocated by the global allocator at a **different**, **separate** heap address, in a different page. That second allocation is **not page-aligned, not under mlocked pages, and freely swappable** by the kernel under memory pressure.
- The mlock'd page region contains: 24 bytes of Vec header (ptr/len/cap, NOT secret) + (page_size - 24) bytes of arbitrary heap garbage that happened to land in the same page. The secret bytes are elsewhere.
- The Drop chain: (i) munlock the page region containing the Vec header — does nothing useful for secrecy. (ii) `Zeroize::zeroize(&mut *self.value)` where `*self.value: Vec<u8>` — this DOES call `Vec::zeroize`, which zeroes the full capacity of the Vec's data buffer (per zeroize crate docs: *"The Zeroize impls for Vec... zeroize the entire capacity of their backing buffer"*). So the secret bytes ARE zeroed at drop. (iii) cfg(test) probe sees the (now-zeroed) page region — but the page region contains only the 24-byte Vec header and arbitrary surrounding bytes; it doesn't observe the secret-bearing data buffer either pre- or post-zeroize, except by chance if the data happens to fall in the same page. Drop-probe G4 tests as written may pass by happenstance or fail unpredictably. (iv) dealloc the page-aligned allocation.

**Net effect:** Cycle B's Sites 2/3/4 (per SPEC §6 row 6 lines 38 + §4 P3a lines 100-102) — `ResolvedSlot.entropy: MlockedZeroizing<Vec<u8>>`, `DerivedAccount.entropy: MlockedZeroizing<Vec<u8>>`, bip85-derived `MlockedZeroizing<Vec<u8>>` — all hold their secret bytes in heap pages that are **not** mlocked. The Cycle's headline threat-model claim ("eliminates secret-material-leaks-to-swap exposure" per SPEC §1) holds for Sites 1 + 5 (clap String fields and ms-cli stdin String, via `pin_pages_for(&[u8])` which DOES pin the buffer's own pages) but FAILS for Sites 2/3/4.

Sites 2/3/4 are exactly the OWNED-buffer entropy stores the Cycle A Drop-zeroize discipline was meant to harden. They are the load-bearing applications of `MlockedZeroizing<T>`. The wrapper-shape design as specified delivers zero mlock value at those sites.

This is not a fixable-by-implementation issue. SPEC §2 row 1 + §4 P2 prescribe the wrapper's structural shape (Box<T> page-aligned allocator). Phase 2 cannot TDD-RED against this design — the G1.1/G1.2 integration tests would observe `is_page_range_locked(addr_of_vec_header, 24) == true` but `is_page_range_locked(vec.as_ptr(), 64) == false`, and the test assertion ambiguity (which address do you check?) reveals the design flaw. The Phase 2 implementer would either:

- Write the tests against the Vec **header's** address (G1 passes, but the cycle is hollow — the secret bytes aren't mlocked); OR
- Write the tests against the Vec **data buffer's** address (G1 fails because the data buffer was never mlocked); OR
- Be confused, ask for guidance, and surface this exact concern.

R0 must surface it BEFORE implementation. Confidence: 100.

### C-1 fix options (SPEC-level, requires reviewer-loop on SPEC §2 row 1 + §4 P2 + P3a)

Three concrete fixes, each requires a SPEC revision:

#### Fix A — Constrain T to fixed-size byte arrays (or types whose data is bytes-of-T inline)

Change `MlockedZeroizing<T: Zeroize>` to `MlockedZeroizing<const N: usize>` (or `MlockedZeroizing<T: Zeroize>` with the documented constraint that T must not internally heap-allocate). Sites 2/3/4 retype:

- Site 2: `ResolvedSlot.entropy: Option<MlockedZeroizing<64>>` (or `Option<MlockedZeroizing<[u8; 64]>>` if a wrapper struct over const-generic is used)
- Site 3: `DerivedAccount.entropy: MlockedZeroizing<64>` similarly
- Site 4: bip85 derives `MlockedZeroizing<64>` directly (Phase 1's Zeroizing<Vec<u8>> would need a follow-up: change `derive_entropy` to return `MlockedZeroizing<64>` directly, OR change it back to `Zeroizing<[u8; 64]>` and wrap differently).

Pros: the wrapper genuinely owns the bytes-of-T inline. mlock pins the actual secret bytes. Simple semantics.
Cons: cascades through Phase 1's design decision. Site 4 (BIP-85) is fine (always 64 bytes). Sites 2/3 are also fine (BIP-39 entropy is 16-32 bytes; can use const-generic). But: Site 2's `ResolvedSlot.entropy` is variable-length depending on word count (12 words → 16 bytes, 24 words → 32 bytes); a const-generic param is awkward. Workaround: use `MlockedZeroizing<32>` always with a separate `length` field, or use the upper-bound length (64). Mild type ergonomics regression vs. `Vec<u8>`.

#### Fix B — Use the slice fn (`pin_pages_for`) for Sites 2/3/4 instead of the wrapper

Sites 2/3/4 retain `Zeroizing<Vec<u8>>` (Cycle A's shape), and **add** a `_pin: PinnedPageRange` field bound to the same struct as the entropy:

```rust
struct DerivedAccount {
    entropy: Zeroizing<Vec<u8>>,
    _entropy_pin: PinnedPageRange,  // Drop-pinned to entropy's lifetime
    ...
}
```

The wrapper `MlockedZeroizing<T>` then drops out entirely; the cycle becomes purely "slice-fn pin_pages_for at all 5 sites, no wrapper". The Box<T> page-aligned allocator design (SPEC §2 row 1, the cfg(test) drop-probe in §6 G4) is removed.

Pros: simplest fix. The slice fn already pins the correct pages (it operates on the Vec's data buffer via `&entropy[..]`, which yields the data-buffer pointer). Site 1 + Site 5 already use this shape. Uniformity across all 5 sites.
Cons: loses the wrapper-API ergonomics. SPEC §2 row 1's "API: wrapper type" row disappears. The cfg(test) drop-probe G4 (verifiable Drop ordering) is harder to set up without the wrapper. But: G4 can be reframed as a `Zeroizing<Vec<u8>>` Drop test inside zeroize's normal discipline (Cycle A already has this implicitly).

#### Fix C — Wrapper around `Vec<u8>`'s data buffer (custom heap-allocated array, not Box<Vec<u8>>)

`MlockedZeroizing<T>` becomes essentially `MlockedZeroizing` with no type param (or with the byte-length only): a thin wrapper around a raw heap pointer to N bytes, page-aligned. Construction takes a `Vec<u8>` and **moves the bytes** (via `copy_from_slice`) into the page-aligned buffer; the original Vec is dropped and its (now-deallocated) data buffer is zeroized by zeroize's Vec impl. Sites 2/3/4 type as `MlockedZeroizing` (size-erased) or `MlockedZeroizing<N>`.

Pros: clean ergonomic wrapper. Page-aligned-and-mlocked allocation actually contains the bytes. Drop-probe G4 works as intended (the wrapper owns the bytes).
Cons: the wrapper effectively reimplements `libsodium::sodium_malloc` — closer to the OOS-secret-arena Cycle C scope. Page-residue concern (SPEC §3 `OOS-page-residue-elimination`) is unchanged. But more importantly: variable-length entropy at Sites 2 (depending on word count) requires either runtime-known size at construction (constructible) or a fixed upper-bound + length field.

### R0's recommended fix: **Fix B** (drop the wrapper; use `pin_pages_for` everywhere)

Reasoning:

- **Smallest SPEC revision.** §2 row 1 is removed; §4 P2 line 84-87 simplifies to just `pin_pages_for` + `PinnedPageRange` + `MlockState` + `report_at_exit`. §4 P3a sites 2/3/4 swap to "add `_entropy_pin: PinnedPageRange` field" pattern. SPEC §6 G4 reframes around `Zeroizing<Vec<u8>>` Drop ordering (no wrapper-Drop manual orchestration needed; zeroize's own Drop is sufficient). Approximately 30 lines of SPEC diff.
- **Phase 2 LOC shrinks.** No `MlockedZeroizing<T>` to implement; no manual `alloc/ptr::write/dealloc` `unsafe`; no Layout/PAGE_SIZE-as-alignment dance. The 4-step Drop orchestration disappears. Module LOC drops from ~270 to ~150. Fewer `unsafe` blocks (just munlock in PinnedPageRange::drop and mlock in pin_pages_for).
- **Miri scope shrinks.** With no manual alloc/dealloc UB to verify, Miri's role narrows to "the pin_pages_for's null-checks and pointer arithmetic are sound" — a much smaller surface.
- **Cycle A discipline preserved verbatim.** Sites 2/3/4 keep `Zeroizing<Vec<u8>>` types unchanged from post-Cycle-A state; only an additional `_entropy_pin: PinnedPageRange` field is added per struct. The Drop order is straightforwardly: struct's fields drop in declaration order — `entropy` first (zeroizes via Zeroizing's Drop), then `_entropy_pin` (munlocks). To ensure entropy zeroizes BEFORE munlock (so the freshly-zeroed bytes remain in pinned pages during the zero-write, then the pages release), reverse the field order: `_entropy_pin` first, `entropy` second... actually, Rust drops in declaration order top-to-bottom — so `entropy` first (zeroize), `_entropy_pin` last (munlock). This is the desired order (zeroize-while-still-pinned, then release). Confirmed via RFC 1857.
- **Page-residue is the same as today.** Either approach pins entire pages. Sites 2/3/4 are 16-64 byte entropies; co-resident heap data is incidentally pinned. SPEC §3 `OOS-page-residue-elimination` covers this; Cycle C addresses at the allocator level. Fix B preserves this disposition.
- **Future-proofing.** If Cycle C eventually builds a `sodium_malloc`-style arena allocator, Fix C's wrapper would be a stepping-stone. Fix B's "pin via slice fn" is allocator-agnostic and would work cleanly with an arena.

**Recommended SPEC revision (concrete patch sketches):**

- §2 row 1: remove entirely; reframe §2 row 6 Site 2/3/4 column to: *"Add `_entropy_pin: PinnedPageRange` field (Drop bound after entropy's Drop)"*.
- §4 P2 lines 83-93: remove the `MlockedZeroizing<T>` paragraph; keep the `pin_pages_for` + `PinnedPageRange` + `MlockState` + `report_at_exit` paragraphs.
- §4 P3a lines 100-102: rewrite Sites 2/3/4 with `_entropy_pin` field pattern instead of type swap.
- §6 G4: reframe as "Zeroize-on-Drop order verified through cfg(test) probe on Zeroizing<Vec<u8>>'s drop callback, OR via Cycle A's existing zeroize-discipline tests".
- §6 G6: `MlockedZeroizing<T>` no longer needs the toolkit-only carve-out in the diff manifest — fully equivalent inline-copy across both repos.

Approximate SPEC diff: 30 lines removed, 15 lines added. Plan §"Phase 2" diff: ~100 LOC removed (the `MlockedZeroizing<T>` impl), ~5 LOC added (field declaration patterns). Phase 2 LOC estimate revises from ~270 to ~150.

### If the SPEC team REJECTS Fix B and prefers Fix A or Fix C

Both A and C are reviewable. Fix A is preferable to C for first-pass mlock per SPLIT-CYCLE discipline (Cycle B is "first-pass"; Cycle C is the arena-allocator follow-on). Fix C drifts toward arena-allocator scope.

### R0 verdict on §3: cannot LOCK Phase 2 against the current SPEC

The SPEC must be patched (Fix B recommended) before Phase 2 RED can land. This is a SPEC-level Critical, not a Phase-2-level Critical — the issue is upstream of Phase 2's design. R0 returns **RE-DRAFT** with the SPEC patch as the primary action item.

---

## §4. Unsafe discipline + SAFETY-comment lint integration

Covered in I-R0-3 above. Recap:

- The existing `lint_safety_third_party_blocked.rs` does NOT cover first-party `unsafe`. Plan line 193 is incorrect on this point.
- Recommendation: add a peer lint `tests/lint_safety_first_party_mlock.rs` in Phase 2 P2.T2 RED commit. Scans `src/mlock.rs` for `unsafe {` and requires a `SAFETY:` comment within ±5 lines above. RED on first commit; GREEN once P2.T3 lands SAFETY comments.
- Under **Fix B** (recommended C-1 resolution), the `unsafe` block count shrinks dramatically:
  - `pin_pages_for`: 1 unsafe (the `libc::mlock` syscall call)
  - `PinnedPageRange::drop`: 1 unsafe (the `libc::munlock` syscall call)
  - That's it. No `std::alloc::alloc/dealloc`, no `ptr::write`, no `Zeroize::zeroize` through raw ptr.

- Under **Fix A** (the wrapper survives but only over `[u8; N]`), the wrapper's unsafe count is similar to plan: ~4-6 unsafe blocks. The SAFETY-lint discipline is more valuable.

- Under **Fix C** (custom byte-array wrapper), similar to Fix A.

Confidence: 90 for the lint extension recommendation regardless of fix choice. The first-party SAFETY-comment discipline is independently useful (it catches future drift in `mlock.rs` regardless of the wrapper shape).

---

## §5. Miri compatibility lock

- **Miri scope (under Fix B):** verify `pin_pages_for` and `PinnedPageRange::drop` are UB-free on the Rust side. `libc::mlock` / `libc::munlock` are not modeled by Miri; stub to no-ops under `cfg(miri)`. `libc::sysconf(_SC_PAGESIZE)` may not be modeled; stub to 4096. Plan line 287 is correct.
- **Miri job placement (under I-R0-1 fold):** new `.github/workflows/rust.yml` with a `miri` job on `ubuntu-latest` nightly. Cargo invocation: `cargo +nightly miri test -p mnemonic-toolkit mlock::`.
- **Miri scope (under Fix A or C):** more relevant; the manual `alloc/ptr::write/zeroize-through-ptr/dealloc` orchestration's UB risk is what Miri catches. Without Miri, a Drop-after-zeroize-where-the-zeroize-was-actually-a-no-op-because-the-raw-ptr-was-already-dangling kind of bug could slip in.
- **Independent of fix choice:** if any `unsafe` survives, Miri is worth the CI cost. Recommendation: add the Miri job in Phase 2 regardless.

Confidence: 92 for Miri-job scope under Fix B; 95 under Fix A/C.

---

## §6. Test strategy lock (per-gate)

Working under Fix B (the recommended SPEC revision):

| Gate | Test placement | Phase 2 (this phase) | Phase 3a (apply) | Notes |
|---|---|---|---|---|
| G1.1 single-page mlock observable | `tests/mlock_unit.rs` (integration) | YES — exercise `pin_pages_for` directly | — | Linux + macOS; `is_page_range_locked()` helper reads `/proc/self/smaps` (Linux) / `mach_vm_region_info` (macOS) |
| G1.2 multi-page mlock observable | `tests/mlock_unit.rs` | YES | — | Same shape, length > PAGE_SIZE |
| G1.3 zero-length no-op | `tests/mlock_unit.rs` | YES | — | Pure unit-level; verify no syscall via fault-injection-set-to-einval + assert no fault injected |
| G1.4 page-aligned exact-page | `tests/mlock_unit.rs` | YES | — | `vec![0; page_size()]`; assert `page_count == 1` |
| G2.1 eperm increments failure_count | unit (`src/mlock.rs`'s test mod) | YES — in-process single test that synthesizes `FAIL_MODE=eperm` via `OnceLock<FailMode>::set` BEFORE any other mlock call | — | Subprocess isolation NOT required for G2.1 alone — only required if multiple G2.* tests run together (since FAIL_MODE is set once per process). Solution: run G2.1 as the first / only G2 test in this profile; OR move G2.1 to subprocess to be safe. Recommend: keep G2.1 as unit test, subprocess the rest. |
| G2.2 enomem | subprocess (via `cargo_bin("mnemonic")`) | **DEFER to Phase 3a.** Phase 2 has no callsite that runs mlock. Without a callsite, no `mnemonic` binary invocation triggers mlock; subprocess can't exercise it. Phase 3a applies mlock at Site 1 (clap fields), which gives subprocess a callsite. | YES | Deferred per the prompt's recommendation. |
| G2.3 einval debug | unit (debug-assert; `#[should_panic]`-style) | YES — in-process; debug-build only | — | Debug-build-only test; `cargo test` exercises it. Release-build assertion is separate. |
| G2.3 einval release | subprocess (`cargo test --release` subprocess from inside an `assert_cmd`) | **DEFER to Phase 3a** | YES | Same callsite issue as G2.2. |
| G2.4 off (control) | unit | YES | — | Trivial; runs as default test |
| G2.5 summary stderr emission | subprocess | **DEFER to Phase 3a** | YES | Same callsite issue. Phase 3a wires `report_at_exit` into `main()`; without that, no subprocess exercises summary emission. |
| G3 platform matrix | CI matrix | YES — add Ubuntu + macOS matrix to new `rust.yml` | — | Plan I-R0-1 fold |
| G4 drop-probe (Zeroize ordering verification) | **`src/mlock.rs` `#[cfg(test)] mod tests`** | YES — but under Fix B, this is a Zeroize<Vec<u8>>-on-drop-probe test, not a MlockedZeroizing-drop-probe test. Reframed shape: register a Drop-callback (via a trait or test-only newtype wrapper) on a `Zeroizing<Vec<u8>>` and assert the callback observes zeroed bytes. Pattern matches Cycle A's existing zeroize-discipline tests. | — | Under Fix B, G4 simplifies to "Zeroizing's Drop scrubs Vec<u8>'s data" — already verified implicitly by Cycle A discipline. Could fold into existing Cycle A tests. |
| G4 Miri | `cargo +nightly miri test -p mnemonic-toolkit mlock::` | YES | — | Under Fix B, Miri exercises pin_pages_for + PinnedPageRange::drop (Rust-level safety). Under Fix A/C, larger scope. |
| G5 lockstep tags | manual PE check | — | — | PE-only |
| G6 inline-copy invariant | `tests/cross_repo_mlock_diff_manifest.rs` | RED placeholder (skipped via `#[ignore]`) | — | Full impl in Phase 3b. |
| G6 name-export parity | same test | RED placeholder | — | Same |
| G7 SHA pins | reproduction step in P3a.T4 | — | YES | Phase 3a only |

### G2 deferral recommendation (recap)

**Defer G2.2, G2.3-release, G2.5 to Phase 3a.** Phase 2 has no mlock callsites in production code; subprocess-based fault injection has nothing to inject into. Phase 3a adds the callsites (Site 1 clap parse, sites 2-5 entropy stores), and at that point a `mnemonic bundle --phrase "..."` invocation under `MNEMONIC_TEST_MLOCK_FAIL_MODE=enomem` exercises mlock at known callsites. Subprocess tests then assert the stderr summary format (G2.5) and per-errno failure counts (G2.2, G2.3-release).

Phase 2 retains: G1.1-G1.4 (full integration coverage via direct `pin_pages_for` call), G2.1 (in-process single-shot, OK because no other G2 test runs in the same process), G2.3-debug (single-process debug-assert), G2.4 (control), G4 (unit-test placement per I-R0-4), Miri (G4 gate).

Phase 3a adds: G2.2, G2.3-release, G2.5, G1 integration coverage of Sites 2-4 (the actual entropy stores), and the G7 SHA pin reproductions.

Confidence: 88. This sizing makes Phase 2 reviewable in isolation against ~6-8 RED tests instead of 12+.

---

## §7. Risks and open questions

### N-1 — Doc-comment header in `bip85.rs:4-5` still says "The 6 in-scope apps" (Confidence: 70)

Pre-existing, flagged in Phase 1 R0 §2 I-R0-2 and Phase 1 R1's Nit list. The DICE app is the 7th. Phase 2 doesn't touch `bip85.rs` source, but if the implementer is making a related sweep (e.g., updating doc-comments referring to Phase 1 work), a one-line fold of this nit could land in a separate `docs(bip85)` commit alongside Phase 2 substeps. Not blocking.

Suggested commit (optional, opportunistic):
```
docs(bip85): fold N-1 — "6 in-scope apps" → "7 in-scope apps (incl. DICE)"
```

### N-2 — Page-residue carve-out documentation in P3a (Confidence: 60)

Under Fix B, Sites 2/3/4's `_entropy_pin: PinnedPageRange` pins pages containing the `Vec<u8>::as_ptr()`-pointed bytes. Co-resident heap data on the same page is incidentally pinned (SPEC §3 `OOS-page-residue-elimination`). Phase 3a R0 should add a one-line cite to this OOS row at each apply site, so reviewers don't double-flag the concern.

### N-3 — `Cargo.toml` `libc` dependency add (Confidence: 75)

Plan line 243 adds `libc = "0.2"` to `Cargo.toml`. The current `Cargo.toml` (lines 19-34) has no `libc` entry. Add to `[dependencies]` block: `libc = "0.2"`. Phase 2 P2.T2 RED commit stages this alongside `Cargo.toml`. Verify `libc 0.2.x` exposes `_SC_PAGESIZE`, `mlock`, `munlock` on both `target_os = "linux"` and `target_os = "macos"` (it does; these are core POSIX symbols).

### Open question 1 — Under Fix B, is `pin_pages_for(&entropy[..])` resilient to `Vec` reallocation during a `MlockedZeroizing<Vec<u8>>`-typed field's lifetime?

If a holder ever calls `.push()` / `.extend()` / `.reserve()` on a pinned Vec's mut-borrow, the Vec may reallocate its data buffer to a new heap address. The `PinnedPageRange` is bound to the OLD address and would now munlock the wrong (post-realloc-stale) page on Drop. **Mitigation:** Sites 2/3/4 are construct-and-pin idioms (entropy is built once via `copy_from_slice` and never appended); the pattern in `bip85.rs:52` is `let mut out = Zeroizing::new(vec![0u8; 64]); out.copy_from_slice(mac.as_byte_array())` — pre-sized at construction, never resized. Document the discipline in `pin_pages_for`'s doc-comment: *"The caller must ensure the buffer's heap address remains stable for the lifetime of the returned PinnedPageRange. Vec reallocation invalidates the pin."* Phase 3a R0 reviewer audits each apply site for reallocation immunity.

### Open question 2 — Should the Phase 2 R1 reviewer block on Fix B's Cycle-A G4-test equivalence, or accept a Phase 2 deferred-G4-test FOLLOWUP?

Under Fix B, G4 (Drop-ordering verification) becomes a thin extension of Cycle A's existing Zeroize-on-drop discipline. Phase 2's net-new G4 surface is small. Phase 2 R1 should verify either (a) a new G4 test lands in `src/mlock.rs`'s unit-test block, or (b) a Cycle-A-equivalent test is cited and confirmed still-passing. Either is acceptable. Confidence: 60.

---

## §8. Verdict

**RE-DRAFT** — Phase 2 cannot LOCK against the current SPEC because of the C-1 indirection trap. The required pre-Phase-2 actions are:

1. **SPEC patch** (Cycle B Phase 0 R3-equivalent reviewer loop) — apply Fix B (recommended) to SPEC §2 row 1, §4 P2, §4 P3a, §6 G4, §6 G6. Approximately 30 lines of SPEC diff. Reviewer dispatch: Opus on the SPEC patch alone.

2. **Plan patch** (Cycle B Phase 2 plan amendment) — fold the C-1 SPEC change, the four I-R0-* findings (crate-structure decision, CI workflow scope, page-size discipline, cfg(test) reachability), and the §6 test-deferral matrix into the plan. Approximately 80 lines of plan diff (most of it test-list reorganization).

3. **Re-dispatch Phase 2 R0** against the patched SPEC + plan. The re-draft should hit LOCK on the first iteration if the four Important findings are folded.

Post-fold, the LOCK-ready Phase 2 design under Fix B is:

- **Module surface:** `pin_pages_for(buf: &[u8]) -> PinnedPageRange`, `struct PinnedPageRange { start, page_count } impl Drop`, `MlockState` private singleton, `record_failure(errno, bytes)`, `report_at_exit()`, `page_size()` cached, `#[cfg(test)] FAIL_MODE: OnceLock<FailMode>`, `parse_fail_mode`. NO `MlockedZeroizing<T>`. Approximately ~150 LOC.
- **Crate shape:** Option C — add `src/lib.rs` with `pub mod mlock;`. Keep `[[bin]]` as-is.
- **`unsafe` blocks:** 2 (mlock in `pin_pages_for`, munlock in `PinnedPageRange::drop`). Each carries a SAFETY comment.
- **Lint extension:** new `tests/lint_safety_first_party_mlock.rs` (~50 LOC) for first-party `unsafe` SAFETY-comment discipline.
- **Tests in `tests/mlock_unit.rs` (integration):** G1.1-G1.4, G2.1, G2.4. ~150 LOC.
- **Tests in `src/mlock.rs`'s `#[cfg(test)] mod tests` (unit):** page-rounding formula, `MlockState::record_failure` idempotence/monotonicity, G2.3-debug, G4 Drop-ordering. ~100 LOC.
- **Test-deferred to Phase 3a:** G2.2 enomem-subprocess, G2.3-release-subprocess, G2.5 stderr summary.
- **CI:** new `.github/workflows/rust.yml` with `test` (Ubuntu+macOS matrix), `miri` (Ubuntu nightly), `clippy` jobs. ~50 LOC of YAML.
- **Cargo.toml:** add `libc = "0.2"` to `[dependencies]`.
- **Total Phase 2 footprint:** ~150 LOC mlock + ~50 LOC lint + ~250 LOC tests + ~50 LOC CI = ~500 LOC.

That is a re-draft target. Phase 2 R0 (this report) returns RE-DRAFT.

If the SPEC team prefers Fix A (constrained T) or Fix C (custom byte-array wrapper) over Fix B, R0 can re-review against the patched SPEC. The crate-structure decision (Option C), CI workflow scope (I-R0-1), Miri page-size discipline (I-R0-2), and cfg(test) reachability fold (I-R0-4) all apply regardless of the C-1 fix choice.

---
