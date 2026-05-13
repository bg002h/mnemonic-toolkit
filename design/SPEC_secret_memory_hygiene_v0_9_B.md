# SPEC — Secret-memory hygiene Cycle B (mlock infrastructure)

**Cycle:** v0.9.0 Cycle B (mlock page-pinning, POSIX-only).
**Status:** Phase 0 (this SPEC). Reviewer-loop until 0 critical / 0 important on Opus R-pass.
**Predecessors:** v0.9.0 Cycle A (`SPEC_secret_memory_hygiene_v0_9_0.md`, shipped 2026-05-13 — tags `mnemonic-toolkit-v0.9.2`, `ms-codec-v0.1.3`, `ms-cli-v0.2.2`).
**Pre-SPEC questions resolved:** `cycle-b-pre-spec-questions` FOLLOWUP (toolkit `design/FOLLOWUPS.md`, commit `1efac85`).
**Authoring session:** 2026-05-13, v1.0 roadmap-survey Bucket-1 drill-down + brainstorming pass (5 Qs locked).

---

## §1. Purpose

Cycle B layers `mlock(2)` page-pinning onto the OWNED-buffer sites that Cycle A made Zeroizing-discipline-compliant. POSIX-only (Linux + macOS); Windows `VirtualLock` deferred to a follow-on cycle (see §3 `OOS-windows-virtuallock`). Cross-repo scope: toolkit (sites 1-4) + `ms-cli` (site 5).

### Threat model addressed

`mlock(2)`-pinned pages cannot be swapped to disk by the kernel — this eliminates the "secret material leaks to swap on memory-pressured systems" exposure. On Linux, mlock-pinned regions are excluded from coredumps by default per `/proc/PID/coredump_filter` (locked anonymous regions are dumped only with bit 1 of the filter set; most distributions ship with bit 1 unset). macOS coredump behavior (`gcore`) similarly excludes mlocked regions.

### Threat model NOT addressed

Live RAM disclosure — `ptrace(PTRACE_PEEKDATA)`, `/proc/PID/mem` reads, or kernel debugger access by an attacker with the same UID or root. That is an OS-level isolation problem; userland mlock does not defend against it.

### Discipline preserved

Cycle A's Drop-zeroize discipline is preserved on every Cycle B path. mlock is additive: even on mlock soft-fail, the underlying `Zeroizing<T>` from Cycle A still scrubs the buffer at Drop. Cycle B → Cycle A is a strict superset relationship at the wrapper level (see §2 row 1).

---

## §2. Coverage deltas (over Cycle A)

| # | Layer | What's new |
|---|---|---|
| 1 | API: wrapper type | `MlockedZeroizing<T: Zeroize>` (`Deref`/`DerefMut` to `T`; page-aligned allocator for an *owned* Box<T>; mlock-on-construct; Drop = munlock → `Zeroize::zeroize(&mut value)` in place → deallocate). **Behavioral superset** of Cycle A's `Zeroizing<T>` (every guarantee Cycle A's `Zeroizing<T>` gives, this gives plus mlock) but **NOT a compositional wrapper** of `Zeroizing<T>` — composition is "owns the buffer directly" (peer of `Zeroizing<T>`). Required to make the cfg(test) drop-probe ordering observable (Drop body manually orchestrates munlock → zeroize → optional probe → dealloc; see §6 G4). Implementation requires a small amount of `unsafe` for the manual zeroize-then-dealloc ordering; the `unsafe` is contained within `mlock.rs`, Miri-checked in CI (per §6 G4). |
| 2 | API: slice fn | `pin_pages_for(buf: &[u8]) -> PinnedPageRange`. Page-granularity is explicit in the return type: `PinnedPageRange { start: *const u8, page_count: usize }` with `Drop` impl that munlocks. Pins the page range covering `buf`. **Page-rounding formula** (pinned to avoid ambiguity): `start = addr & !(PAGE_SIZE - 1)` (round down); `end = (addr + len + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)` (round up); `page_count = (end - start) / PAGE_SIZE`. A slice exactly fitting one page → `page_count = 1`; exactly two pages → `page_count = 2`. **Zero-length slice (`buf.len() == 0`) is a no-op**: returns `PinnedPageRange { start: ptr::null(), page_count: 0 }` whose Drop is also a no-op. No `mlock(2)` syscall is issued for zero-length (Linux `mlock(addr, 0)` returns EINVAL; macOS returns success; the no-op avoids both). Callers accept page-residue from co-resident non-secret allocations (Cycle C addresses at allocator level — see §3 `OOS-page-residue-elimination`). |
| 3 | API: state singleton | `MlockState` — process-static via `std::sync::OnceLock<MlockState>`. Fields: `failure_count: AtomicUsize`, `total_bytes_unlocked: AtomicUsize`, `first_errno: OnceLock<i32>`. Thread-safe; lock-free reads on the hot path. |
| 4 | API: end-of-process emit | `pub fn report_at_exit()`. Called from `main()` in both `mnemonic-toolkit` (bin) and `ms-cli` (bin). Emits a 2-line stderr summary iff `failure_count > 0`. Format pinned in §6 G2. |
| 5 | Precursor refactor | `bip85::derive_entropy(index: u32) -> [u8; 64]` heap-promoted to `-> Zeroizing<Vec<u8>>`. 6 callees in `format_*` functions updated. P1-only; no new public API surface beyond the return-type change. |
| 6 | Site applications | **Site 1 (toolkit)**: clap fields (passphrase / phrase / slot, ~12 fields across 6 cmd structs: `BundleArgs`, `VerifyBundleArgs`, `ConvertArgs`, `DeriveChildArgs`, `EncodeArgs`, `VerifyArgs`) — `pin_pages_for(...)` call after clap parse. **Site 2 (toolkit)**: `ResolvedSlot.entropy: Option<Zeroizing<Vec<u8>>>` → `Option<MlockedZeroizing<Vec<u8>>>`. **Site 3 (toolkit)**: `DerivedAccount.entropy: Zeroizing<Vec<u8>>` → `MlockedZeroizing<Vec<u8>>`. **Site 4 (toolkit)**: bip85's heap-promoted Vec wrapped. **Site 5 (ms-cli)**: `read_stdin()` String at `parse.rs:45` — `pin_pages_for(s.as_bytes())` post-receipt. The Site-1 collective handle expands to per-field enumeration in Phase 3a prose. |
| 7 | Errno discipline | All errno classes soft-fail in release: `EPERM`, `ENOMEM` (RLIMIT_MEMLOCK or cgroup memory.lock_size), `EAGAIN` (per-process lock limit), `ENOTSUP`, others. `EINVAL` SHOULD be unreachable from the hybrid API by construction (wrapper allocates page-aligned; slice fn rounds to page boundary). If an `EINVAL` ever surfaces it indicates a bug in this module: debug builds trip `debug_assert!`; release builds soft-fail like the other errno classes. No `Result`-typed return at the user-facing API; mlock outcomes are reported via `MlockState`, not propagated to callers. |
| 8 | Test surface | (a) `MlockState` aggregation unit tests; (b) `#[cfg(test)]` env-var hook `MNEMONIC_TEST_MLOCK_FAIL_MODE={eperm,enomem,einval,off}` for fault injection at every mlock callsite, cache shape pinned `OnceLock<FailMode>` (subprocess isolation required for per-test mode variation; see §4 P2); (c) `#[cfg(test)]` drop-probe instrumentation on `MlockedZeroizing<T>` with verifiable Drop ordering (see §6 G4); (d) CI invariant test enforcing the diff-manifest equivalence between toolkit's and ms-cli's `mlock.rs` (see §6 G6 + §5); (e) POSIX integration tests asserting page-locked state during scope + munlocked after Drop via `/proc/self/smaps` (Linux) and `mach_vm_region` (macOS) test-only helpers, covering single-page / multi-page / zero-length / page-aligned cases (see §6 G1.1-G1.4); (f) Miri pass on the `unsafe` blocks in `MlockedZeroizing::drop` via `cargo +nightly miri test -p mnemonic-toolkit mlock::` (see §6 G4 Miri gate). |

Cross-repo coordination details are in §5 (not duplicated here).

---

## §3. Out-of-scope (filed for explicit closure)

| OOS class | Rationale | Where it goes |
|---|---|---|
| `OOS-windows-virtuallock` | Q4 decision (2026-05-13 brainstorming). `VirtualLock` has structurally different semantics (no `EPERM` equivalent; soft-fail signals are `ERROR_NOT_ENOUGH_QUOTA` + working-set limits). Forcing one abstraction over POSIX + Windows risks lowest-common-denominator design. | Future cycle once the POSIX abstraction has settled. Cycle B's mlock module API MUST NOT lock in a shape that prevents future VirtualLock addition (informal compatibility check: a `cfg(windows)` branch can be added without breaking the public API). |
| `OOS-secret-arena` | Carried from Cycle A SPEC §3. Full page-aligned-with-guard-pages allocator (libsodium `sodium_malloc`-style) is allocator-level work that Cycle B does not undertake. | Cycle C `dedicated-secret-arena` FOLLOWUP (toolkit `design/FOLLOWUPS.md`). |
| `OOS-upstream-zeroize-mlock` | Q1 decision. The hygiene-matrix §4 alternative list (substituting `secp256k1::SecretKey` + `bip39::Mnemonic` interior buffers for SPEC slots 2 and 3) is a defense-in-depth reframing for upstream-blocked Drop+Zeroize gaps, not OWNED-buffer hygiene. Mlock-ing upstream-owned memory is fragile (the upstream library can reallocate without notice). | Existing FOLLOWUPS: `rust-secp256k1-secretkey-zeroize-upstream`, `rust-bip39-mnemonic-zeroize-upstream`. Revisit when those upstreams gain Zeroize support. |
| `OOS-secrets-on-stack` | `mlock(2)` requires stable virtual addresses; stack regions get remapped on every function call. Survey §4 (lines 206-210) lists short-lifetime stack secrets (~5+ `seed: [u8; 64]` locals beyond Site 4); none are in Cycle B's scope. | Future heap-promotion cycles per-site (Site 4 is Cycle B Phase 1; others TBD). |
| `OOS-capability-probe` | Q2 decision. Upfront capability probing (`CAP_IPC_LOCK` detection via libcap / capget / `/proc/self/status` parsing) is TOCTOU-prone (capabilities can be revoked between probe and use via cgroup transitions or `prctl(PR_CAPBSET_DROP)`) and platform-specific. Try-and-soft-fail per call is honest and uniform across POSIX. | N/A (intentional final shape, not deferred). |
| `OOS-cross-process-aggregation` | Each binary's `MlockState` is process-local. Pipelines (e.g., `mnemonic bundle \| ms-cli decode`) emit independent per-process summaries on each binary's exit. | N/A (intentional). |
| `OOS-page-residue-elimination` | The slice fn pins entire pages, so non-secret data co-resident on the same page is incidentally pinned. Cycle B accepts this; SPEC documents it. | Cycle C `dedicated-secret-arena` (allocator-level) eliminates co-residency. |
| `OOS-suppression-flag` | Q3 decision. No `--quiet-mlock` flag or `MNEMONIC_MLOCK_QUIET` env var in Cycle B. Default-loud is acceptable for v0.10.0 ship; user feedback may add suppression in a future patch. | Future cycle if user feedback demands. |
| `OOS-shared-mlock-crate` | Q5 decision. `pin_pages_for` is inline-duplicated in toolkit and ms-cli; the CI invariant test (§5) prevents drift. Constellation stays at 4 crates. Matches the `mc-codex32-extraction-retired-2026-05-03` precedent (mnemonic-key FOLLOWUPS) of "fork indefinitely; document the pattern" over premature crate extraction. | Future cycle if a second cross-repo mlock concern surfaces. |

---

## §4. Phase structure (cross-ref to plan)

Five phases (P0–P3 + PE). Tighter than Cycle A's matrix-heavy three-phase structure because Cycle B is "first-pass" mlock per the SPLIT-CYCLE rationale (Cycle A Phase 0 R3 finding). Cross-repo coordination lives inside P3 as 3a/3b subsections rather than a separate phase.

### P0 — SPEC (this artifact)

This SPEC + reviewer-loop. Reviewer dispatch: Opus on `feature-dev:code-reviewer` per `feedback_opus_primary_review_agent`. P0 ships when:
- 0 critical / 0 important findings remain
- Pre-SPEC FOLLOWUP `cycle-b-pre-spec-questions` updated with per-question resolutions (already partially done; final commit closes it as `resolved by P0 ship`)
- Companion FOLLOWUP filed in `mnemonic-secret/design/FOLLOWUPS.md` (id: `secret-memory-hygiene-cycle-b`; reciprocal Companion line pointing at the toolkit-side parent entry)

### P1 (toolkit) — bip85 heap-promote precursor

- Change `pub fn derive_entropy(index: u32) -> [u8; 64]` to `-> Zeroizing<Vec<u8>>`. Phase 1 design pass decides between `Zeroizing<Vec<u8>>` and `Box<Zeroizing<[u8; 64]>>` based on callee ergonomics.
- Update 6 callees in `bip85.rs` `format_*` functions.
- Byte-determinism check: encode twice via the bip85 derivation path, assert identical bytes (mirrors `feedback_spike_before_locking_wire_format` discipline that surfaced the v0.1.0 release fix).
- No new public API surface beyond return-type change. Pre-P2 commit.
- Approximate scope: ~150 LOC.

### P2 (toolkit) — mlock module

- New module at `crates/mnemonic-toolkit/src/mlock.rs`:
  - `pub struct MlockedZeroizing<T: Zeroize>` with `pub fn new(value: T) -> Self`, `Deref`/`DerefMut` to `T`. **Composition shape:** owns a page-aligned Box<T> directly (peer of `Zeroizing<T>`, NOT a wrapper around it). `Drop` impl manually orchestrates: (i) munlock the page range; (ii) `Zeroize::zeroize(&mut *self.value)` in place via `unsafe` deref of the owned ptr; (iii) optionally invoke the cfg(test) probe; (iv) deallocate the page-aligned Box. The `unsafe` for steps (ii)-(iv) is contained in this Drop impl and one helper; CI runs Miri on the relevant tests (`cargo +nightly miri test -p mnemonic-toolkit mlock::`) to verify no UB. See §6 G4.
  - `pub fn pin_pages_for(buf: &[u8]) -> PinnedPageRange`. Zero-length is a no-op (returns empty range; no syscall); see §2 row 2 for the page-rounding formula.
  - `pub struct PinnedPageRange { pub start: *const u8, pub page_count: usize }` with `Drop` impl (munlock when `page_count > 0`; no-op otherwise).
  - `MlockState` private struct (fields: `failure_count: AtomicUsize`, `total_bytes_unlocked: AtomicUsize`, `first_errno: OnceLock<i32>`) + `static MLOCK_STATE: OnceLock<MlockState>` accessor + `fn record_failure(errno: i32, bytes: usize)` (idempotent on `first_errno`; monotonic on counters).
  - `pub fn report_at_exit()`. Called from `main()` in both binaries.
- `#[cfg(test)]` hooks:
  - **Env-var fault injection** with pinned cache shape: `static FAIL_MODE: OnceLock<FailMode> = OnceLock::new();` resolved at first mlock call via `std::env::var("MNEMONIC_TEST_MLOCK_FAIL_MODE").ok().and_then(parse)`. Supported values: `eperm`, `enomem`, `einval`, `off`. Production code path is `cfg(not(test))` direct mlock; test path branches on the cached `FAIL_MODE`. Cross-thread coherence: `OnceLock` guarantees first-writer-wins; all threads observe the same `FailMode` for the lifetime of the process. **Tests requiring per-test mode variation MUST use subprocess isolation** (`assert_cmd::Command::cargo_bin(...)` with per-invocation env), not `cargo test`'s default in-process parallelism (which would all share the first-resolved `FAIL_MODE`).
  - **Drop-probe constructor:** `MlockedZeroizing::new_with_drop_probe<F: FnOnce(&[u8]) + 'static>(value: T, probe: F) -> Self`. Stores `probe` as `Option<Box<dyn FnOnce(&[u8])>>` on the struct. Drop body calls `probe(observed_buffer)` AFTER step (ii) (zeroize) and BEFORE step (iv) (dealloc), with `observed_buffer` being a `&[u8]` slice over the (now-zeroed) Box contents. This ordering is achievable because Drop owns the buffer directly (per Row 1 composition change); no UB. Available only under `#[cfg(test)]`.
- Module is reviewable in isolation; no applications yet.
- Approximate scope: ~270 LOC (10-20 LOC over the original estimate to absorb the owns-buffer-directly composition).

### P3 — apply at sites 1-5 (cross-repo)

#### P3a (toolkit)

- **Site 1**: insert `pin_pages_for(...)` calls after clap parse for each of the ~12 fields. Phase 3a prose enumerates each by name (clap struct + field). The Drop of the returned `PinnedPageRange` is bound to the same scope as the corresponding String / Vec it pins.
- **Site 2**: `ResolvedSlot.entropy: Option<Zeroizing<Vec<u8>>>` → `Option<MlockedZeroizing<Vec<u8>>>` at `synthesize.rs`.
- **Site 3**: `DerivedAccount.entropy: Zeroizing<Vec<u8>>` → `MlockedZeroizing<Vec<u8>>` at `derive.rs`.
- **Site 4**: bip85's heap-promoted `Zeroizing<Vec<u8>>` (post-P1) wrapped in `MlockedZeroizing` at the callsites.
- `main()` in `crates/mnemonic-toolkit/src/main.rs` adds `mnemonic_toolkit::mlock::report_at_exit()` call before exit.
- Approximate scope: ~80 LOC.

#### P3b (ms-cli, cross-repo)

- Inline copy of `pin_pages_for` + `PinnedPageRange` + `MlockState` (process-local; not shared with toolkit's singleton) + `report_at_exit` at `mnemonic-secret/.../ms-cli/src/mlock.rs`. Wrapper type `MlockedZeroizing<T>` is NOT copied (ms-cli does not use it; no OWNED entropy buffer in ms-cli's surface).
- **Site 5**: at `parse.rs:45`, after `read_stdin()` returns `String`, call `pin_pages_for(s.as_bytes())`. Bind the `PinnedPageRange` to the same scope as `s`.
- `main()` in `ms-cli/src/main.rs` adds `ms_cli::mlock::report_at_exit()` call before exit.
- CI invariant test (in toolkit's workspace, mirrored in ms repo): test reads `mlock.rs` slice-fn source in toolkit + reads `mlock.rs` slice-fn source in ms-cli, normalizes (strip whitespace, comments, doc-comments), asserts byte-equal. Fails on drift.
- Cross-repo review surface: Phase 3b is the only cross-repo phase. Default dispatch: one Opus reviewer with full constellation context. Fallback if context budget pushes back: two sibling-scoped reviewers (one per repo) with explicit cross-reference instructions in their prompts.
- Approximate scope: ~40 LOC.

### PE — release rollup

- Audit matrix doc: `design/agent-reports/v0_9_B-secret-memory-hygiene-matrix.md` (mirrors v0.9.0 Cycle A's matrix shape).
- FOLLOWUPS: close `secret-memory-hygiene-cycle-b` (toolkit + ms repo companion) with reciprocal commit-SHA cross-citations.
- Tags: `mnemonic-toolkit-v0.10.0` (new pub API: `mlock` module surface) + `ms-cli-v0.3.0` (no new pub API; ship marker for inline `mlock` module). Both pushed within a single PE session.
- CHANGELOG entries in both repos cross-cite each other's tag commit SHAs.

---

## §5. Cross-repo coordination

- **Lockstep tag discipline.** Mirrors v0.9.0 Cycle A (which shipped `mnemonic-toolkit-v0.9.2` + `ms-codec-v0.1.3` + `ms-cli-v0.2.2` within a single PE session 2026-05-13). Cycle B PE ships `mnemonic-toolkit-v0.10.0` + `ms-cli-v0.3.0` in lockstep.

- **Companion FOLLOWUP.** Filed in `mnemonic-secret/design/FOLLOWUPS.md` at P0 SPEC ship. Same id `secret-memory-hygiene-cycle-b` in both repos (matches Cycle A's `secret-memory-hygiene-v0_9-cycle-a` cross-repo id convention). Reciprocal `Companion:` lines point each entry at the other.

- **Inline-copy invariant (diff manifest).** The duplicated module surface is enumerated as a diff manifest in §6 G6: `{pin_pages_for, PinnedPageRange + Drop impl, MlockState + accessor + record_failure, report_at_exit, private errno-handling helpers}`. `MlockedZeroizing<T>` is toolkit-only and NOT in the manifest. Enforcement: workspace-level CI test in toolkit (and mirror in ms repo) compares the diff-manifest items by normalized source text (per G6's normalization rules: PRESERVE `use` statements and `#[cfg]` attributes; strip line comments and doc-comments at start-of-trimmed-line; preserve internal string-literal whitespace). The manifest itself is also under test (a static name-list assertion in the CI test) — adding a new item to one repo's `mlock.rs` without updating the manifest and the other repo fails the test (helper-fn-circumvention mitigation). Documented in both repos' `Companion:` lines.

  Operational note: the test requires both repos to be checked out in the CI environment. The toolkit's CI workflow (`.github/workflows/`) adds a checkout step for `mnemonic-secret` at the matching tag (or `main` if no tag yet for the in-flight Cycle B PE). ms repo's CI mirrors with a checkout step for `mnemonic-toolkit`. P3b commit pins the exact `actions/checkout` configuration.

- **Cross-repo cycle-close gates.** PE tags push within the same session; FOLLOWUPS resolutions commit with reciprocal SHA citations; CHANGELOG entries cross-cite.

- **Review dispatch (P3b).** Per `feedback_opus_primary_review_agent`, default Opus on the cross-repo reviewer. Recommended dispatch: a single Opus subagent with full constellation context covering both repos' P3b surface (one inline-copy + one apply site = small enough to hold in one head). Fallback: if context budget pushes back, two sibling-scoped reviewers with explicit cross-reference instructions ("the other repo's PR is at <URL>; verify the inline `pin_pages_for` implementations match"). The CI invariant test is the regression backstop for either dispatch shape.

---

## §6. Acceptance gates

Numbered gates with explicit-pass criteria. Mirrors Cycle A SPEC §6 pattern.

### G1 — Functional correctness

Each of the 5 sites successfully mlocks under default test environment (`RLIMIT_MEMLOCK ≥ 64KiB`; no cgroup `memory.lock_size` restriction). Integration tests cover the happy path plus three explicit edge cases:

**Verification mechanism (test-only):**
- **Linux**: parse `/proc/self/smaps` for the address range matching the buffer; assert `Locked > 0` for the entry. Test helper `mlock::tests::is_page_range_locked(addr: *const u8, len: usize) -> bool`.
- **macOS**: `mach_vm_region_info` via the `mach` crate (already in transitive deps for some platforms; if not, a dev-only addition is acceptable). Assert region's wired-memory accounting. Same test helper signature.

**Test cases:**
- **G1.1** Single-page case (length ≤ PAGE_SIZE): assert `PinnedPageRange.page_count == 1`; assert page is locked during scope; assert munlocked after Drop.
- **G1.2** Multi-page case (length > PAGE_SIZE, e.g., `vec![0xAA; 2 * PAGE_SIZE]`): assert `PinnedPageRange.page_count >= 2`; assert all spanned pages report `Locked > 0` in `/proc/self/smaps`; assert all munlocked after Drop.
- **G1.3** Zero-length no-op case (per §2 row 2): `pin_pages_for(&[])` returns `PinnedPageRange { start: ptr::null(), page_count: 0 }`; assert no `MlockState.failure_count` increment; assert no syscall (verify via `strace`-style instrumentation OR by setting `MNEMONIC_TEST_MLOCK_FAIL_MODE=einval` and confirming no fault is injected since no syscall runs); assert no panic.
- **G1.4** Page-aligned slice exactly fitting one page: assert `page_count == 1` (rounding formula doesn't double-count an exactly-aligned end).

### G2 — Soft-fail coverage

Per-errno-class unit test via `MNEMONIC_TEST_MLOCK_FAIL_MODE={eperm,enomem,einval,off}`:

- **G2.1** eperm: assert `MlockState.failure_count` incremented; `first_errno` recorded as `EPERM`.
- **G2.2** enomem: same; `first_errno = ENOMEM`.
- **G2.3** einval split:
  - **Debug build** (`cargo test`): assert `debug_assert!` fires (use `should_panic` with the expected debug-assert message, or capture via `catch_unwind`).
  - **Release build** (`cargo test --release`): assert `MlockState.failure_count` incremented; `first_errno = EINVAL`; no panic.
- **G2.4** off (control): no failure path exercised; `failure_count == 0`.
- **G2.5** Summary emission: 2-line stderr emitted iff `failure_count > 0`; absent when 0. Format pinned:
  ```
  warning: <K> of <N> secret regions could not be locked
           (first errno: <ERRNO_NAME>, <BYTES> bytes total); secret
           data remains in heap and may be swappable.
  hint:    set RLIMIT_MEMLOCK >= 64KiB or grant CAP_IPC_LOCK
           to eliminate this warning.
  ```

### G3 — Platform coverage

CI matrix runs on:
- Ubuntu (latest LTS, GitHub Actions image), with `ulimit -l unlimited` set in the workflow OR a CI step asserting `ulimit -l ≥ 65536`.
- macOS (current GitHub Actions image; default `ulimit -l` is unlimited).

Both green required. Rationale: macOS coredump_filter behavior and mlock errno conventions differ from Linux; both code paths must compile and pass.

### G4 — Cycle A discipline preserved (verifiable Drop ordering, no UB)

Sites still Drop-zeroize when mlock fails. Verification via `#[cfg(test)]` drop-probe instrumentation. The ordering claim is achievable because `MlockedZeroizing<T>` owns its buffer directly (composition shape per §2 row 1, NOT a wrapper around `Zeroizing<T>`); the Drop body manually orchestrates munlock → zeroize → probe → dealloc.

```rust
#[cfg(test)]
impl<T: Zeroize> MlockedZeroizing<T> {
    pub fn new_with_drop_probe<F>(value: T, probe: F) -> Self
    where F: FnOnce(&[u8]) + 'static { /* stores probe on struct */ }
    // Drop body order (all four steps inside MlockedZeroizing::drop):
    //   (i)   munlock the page range
    //   (ii)  Zeroize::zeroize(&mut *value) in place
    //   (iii) probe(observed_buffer) — probe sees the zeroed buffer
    //   (iv)  deallocate the page-aligned Box
    // No post-free read; no UB.
}
```

Test pattern: hold `MlockedZeroizing::<Vec<u8>>::new_with_drop_probe(vec![0xAA; 32], move |buf| { ... })` with known bytes under `MNEMONIC_TEST_MLOCK_FAIL_MODE=eperm` (subprocess-isolated per §4 P2 cache-shape note). Force Drop. Probe callback (captured `'static`) asserts the observed buffer is all-zero.

**Miri gate:** CI runs `cargo +nightly miri test -p mnemonic-toolkit mlock::` on the drop-probe tests to verify the `unsafe` blocks in §4 P2 (manual zeroize-then-dealloc ordering) are UB-free. Miri failure fails G4.

### G5 — Cross-repo lockstep

- PE tags `mnemonic-toolkit-v0.10.0` + `ms-cli-v0.3.0` push within the same session.
- CHANGELOG entries in both repos cross-cite each other's tag commit SHAs.
- Companion FOLLOWUP resolution commits within 24h of each other.

### G6 — Inline-copy equivalence (diff manifest + name-export check)

The duplicated module surface is enumerated as a **diff manifest** (the binding list of items that must remain equivalent across repos):

- `fn pin_pages_for(buf: &[u8]) -> PinnedPageRange`
- `struct PinnedPageRange { start: *const u8, page_count: usize }` + its `impl Drop`
- `struct MlockState` (fields, accessor `static MLOCK_STATE`, methods `record_failure(errno: i32, bytes: usize)`)
- `fn report_at_exit()`
- Any private errno-handling helpers used by the above (enumerated at P2 commit time)

`MlockedZeroizing<T>` is **toolkit-only** and is NOT in the manifest.

**CI invariant test (workspace-level; mirrored in ms repo).** Asserts on every PR in both repos:

1. **Source-text equivalence** for every item in the diff manifest, normalized as:
   - Strip leading/trailing whitespace per line
   - Strip `//` line comments at start-of-trimmed-line only
   - Strip `/// ` doc-comments
   - Preserve internal whitespace inside string literals
   - **PRESERVE** `use` statements (so `use crate::Foo` vs `use ms_cli::Foo` divergence fails loudly, forcing explicit alignment via re-exports or refactor)
   - **PRESERVE** `#[cfg(target_os = "linux")]`, `#[cfg(target_os = "macos")]`, etc. (platform-divergence is intentional and must be loud)

2. **Name-export parity:** toolkit's `mlock.rs` exports exactly `{diff_manifest_names ∪ {MlockedZeroizing}}`; ms-cli's `mlock.rs` exports exactly `{diff_manifest_names}`. Any name added to one repo's `mlock.rs` without the corresponding update in the other (or in the manifest exception list for `MlockedZeroizing`) fails the test.

3. **Manifest under test:** the diff manifest itself is a static `&[&str]` list in the CI test code, asserted against the names extracted from both `mlock.rs` files. Adding a new fn or struct to one repo's `mlock.rs` without updating the manifest fails the test — this is the helper-fn-circumvention mitigation.

Both repos' CI workflows must check out the other repo at the matching tag (or `main` if no tag yet) before running this test. Documented operationally in §5.

### G7 — No wire-format regression

Existing fixture corpus SHA pins continue to hold post-Cycle-B. Proves mlock is functionally transparent.

| Pin | Corpus | Set by |
|---|---|---|
| `81828299c927783d915108f32c9752b3dbf815c1caba5b6f6e7ce7b810ddcbf6` | `crates/mnemonic-toolkit/tests/vectors/v0_1/` | mnemonic-toolkit-v0.1.0 |
| `a381761656fd165e8e5af28574a5baaa55973e562c610254ae6f31d6b1f06171` | `crates/mnemonic-toolkit/tests/vectors/v0_2/` | mnemonic-toolkit-v0.2.0 |

Reproduction command (per CHANGELOG.md v0.2 section):

```bash
shasum -a 256 crates/mnemonic-toolkit/tests/vectors/v0_X/*.txt | sort | shasum -a 256
```

If v0.3+ corpora exist with additional pins (verified at P0 ship), append to this table.

---

## §7. Cross-refs

- **Cycle A SPEC**: `design/SPEC_secret_memory_hygiene_v0_9_0.md`
  - §3 `OOS-mlock-cycle-b` is realized by this SPEC
  - §3 `OOS-secret-arena` is carried forward to Cycle C (not Cycle B)
  - §6 acceptance gates pattern mirrored by this SPEC's §6
- **Cycle A audit matrix**: `design/agent-reports/v0_9_0-secret-memory-hygiene-matrix.md` §4 (lines 247-269; carry-overs to Cycle B)
- **Cycle A survey**: `design/agent-reports/v0_9_0-secret-memory-survey.md` §4 (lines 161-210; explicitly designated as shared with future Cycle B)
- **Phase 0 R1 report**: `design/agent-reports/v0_9_0-phase-0-spec-plan-r1.md`
  - R3 SPLIT-CYCLE finding (lines 90-104) — rationale for Cycle B's existence
  - Mlock-module-shape prototype seed (lines 188-260) — superseded by this SPEC's hybrid API
  - R3 I-R3-2 architectural trap on byte-len API — addressed by `pin_pages_for` returning `PinnedPageRange` (page-granularity explicit in type)
- **Pre-SPEC FOLLOWUP (resolved by this SPEC's P0 ship)**: `design/FOLLOWUPS.md` `cycle-b-pre-spec-questions` (commit `1efac85`)
- **Parent cycle FOLLOWUP (resolves on PE)**: `design/FOLLOWUPS.md` `secret-memory-hygiene-cycle-b`
- **Companion FOLLOWUP (filed at P0 SPEC ship)**: `mnemonic-secret/design/FOLLOWUPS.md` `secret-memory-hygiene-cycle-b`
- **Cross-repo precedent (Q5 rationale)**: `mc-codex32-extraction-retired-2026-05-03` (mnemonic-key `design/FOLLOWUPS.md`) — established the "fork-and-document-pattern over shared-crate-extraction" discipline
- **Prior art**: libsodium's two-tier mlock API — `sodium_malloc` (wrapper-shaped, page-aligned) + `sodium_mlock`/`sodium_munlock` (slice-shaped, byte-len with documented page-residue caveat). Basis for Q1's hybrid choice
- **Man pages**: POSIX `mlock(2)`; macOS `mlock(2)` (BSD-derived)
- **Out-of-repo plan artifacts**: Cycle A's plan lived at `~/.claude/plans/v0_9_0-secret-memory-hygiene.md`. Cycle B's plan will live at `~/.claude/plans/v0_9_B-secret-memory-hygiene-cycle-b.md` (drafted post-SPEC via `superpowers:writing-plans`)
