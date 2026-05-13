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
| 1 | API: wrapper type | `MlockedZeroizing<T>` (`Deref`/`DerefMut` to `T`; page-aligned allocator for the inner Box; mlock-on-construct; Drop = munlock + zeroize). Wraps `Zeroizing<T>` from Cycle A; strict superset behavior. |
| 2 | API: slice fn | `pin_pages_for(buf: &[u8]) -> PinnedPageRange`. Page-granularity is explicit in the return type: `PinnedPageRange { start: *const u8, page_count: usize }` with `Drop` impl that munlocks. Pins the page range covering `buf` (start rounded down, end rounded up to page boundary). Callers accept page-residue from co-resident non-secret allocations (Cycle C addresses at allocator level — see §3 `OOS-page-residue-elimination`). |
| 3 | API: state singleton | `MlockState` — process-static via `std::sync::OnceLock<MlockState>`. Fields: `failure_count: AtomicUsize`, `total_bytes_unlocked: AtomicUsize`, `first_errno: OnceLock<i32>`. Thread-safe; lock-free reads on the hot path. |
| 4 | API: end-of-process emit | `pub fn report_at_exit()`. Called from `main()` in both `mnemonic-toolkit` (bin) and `ms-cli` (bin). Emits a 2-line stderr summary iff `failure_count > 0`. Format pinned in §6 G2. |
| 5 | Precursor refactor | `bip85::derive_entropy(index: u32) -> [u8; 64]` heap-promoted to `-> Zeroizing<Vec<u8>>`. 6 callees in `format_*` functions updated. P1-only; no new public API surface beyond the return-type change. |
| 6 | Site applications | **Site 1 (toolkit)**: clap fields (passphrase / phrase / slot, ~12 fields across 6 cmd structs: `BundleArgs`, `VerifyBundleArgs`, `ConvertArgs`, `DeriveChildArgs`, `EncodeArgs`, `VerifyArgs`) — `pin_pages_for(...)` call after clap parse. **Site 2 (toolkit)**: `ResolvedSlot.entropy: Option<Zeroizing<Vec<u8>>>` → `Option<MlockedZeroizing<Vec<u8>>>`. **Site 3 (toolkit)**: `DerivedAccount.entropy: Zeroizing<Vec<u8>>` → `MlockedZeroizing<Vec<u8>>`. **Site 4 (toolkit)**: bip85's heap-promoted Vec wrapped. **Site 5 (ms-cli)**: `read_stdin()` String at `parse.rs:45` — `pin_pages_for(s.as_bytes())` post-receipt. The Site-1 collective handle expands to per-field enumeration in Phase 3a prose. |
| 7 | Errno discipline | All errno classes soft-fail in release: `EPERM`, `ENOMEM` (RLIMIT_MEMLOCK or cgroup memory.lock_size), `EAGAIN` (per-process lock limit), `ENOTSUP`, others. `EINVAL` SHOULD be unreachable from the hybrid API by construction (wrapper allocates page-aligned; slice fn rounds to page boundary). If an `EINVAL` ever surfaces it indicates a bug in this module: debug builds trip `debug_assert!`; release builds soft-fail like the other errno classes. No `Result`-typed return at the user-facing API; mlock outcomes are reported via `MlockState`, not propagated to callers. |
| 8 | Test surface | (a) `MlockState` aggregation unit tests; (b) `#[cfg(test)]` env-var hook `MNEMONIC_TEST_MLOCK_FAIL_MODE={eperm,enomem,einval,off}` for fault injection at every mlock callsite; (c) `#[cfg(test)]` drop-probe instrumentation on `MlockedZeroizing<T>` (see §6 G4); (d) CI invariant test comparing `pin_pages_for` impls across repos (see §5); (e) POSIX integration tests asserting page-locked state during scope + munlocked after Drop via `/proc/self/smaps` (Linux) and `mach_vm_region` (macOS) test-only helpers. |

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
  - `pub struct MlockedZeroizing<T>` with `pub fn new(value: T) -> Self`, `Deref`/`DerefMut` to `T`, `Drop` impl (munlock then drop the inner `Zeroizing<T>` which zeroizes).
  - `pub fn pin_pages_for(buf: &[u8]) -> PinnedPageRange`.
  - `pub struct PinnedPageRange { pub start: *const u8, pub page_count: usize }` with `Drop` impl (munlock).
  - `MlockState` private struct + `static MLOCK_STATE: OnceLock<MlockState>` accessor.
  - `pub fn report_at_exit()`.
- `#[cfg(test)]` hooks:
  - Env-var fault injection: every mlock callsite checks `std::env::var("MNEMONIC_TEST_MLOCK_FAIL_MODE")` once on first use (cached); supported values `eperm`, `enomem`, `einval`, `off`. Production code path is `cfg(not(test))` direct mlock; test path branches on the env var.
  - Drop-probe constructor: `MlockedZeroizing::new_with_drop_probe<F: FnOnce(&[u8])>(value: T, probe: F) -> Self`. The probe is invoked AFTER munlock + zeroize but BEFORE deallocating the inner Box, allowing the test to inspect the zeroed buffer (no `unsafe` post-free read). Available only under `#[cfg(test)]`.
- Module is reviewable in isolation; no applications yet.
- Approximate scope: ~250 LOC.

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

- **`pin_pages_for` inline-copy invariant.** Both repos' impls must remain functionally equivalent. Enforcement: a workspace-level test in toolkit reads both source files (toolkit's local copy + ms repo's copy via a path computed from a shared CI checkout), normalizes (strip whitespace, comments, doc-comments), asserts byte-equal. CI fails on drift. Mirror test in ms repo. Documented in both repos' `Companion:` lines.

  Operational note: the test requires both repos to be checked out in the CI environment. The toolkit's CI workflow (`.github/workflows/`) adds a checkout step for `mnemonic-secret` alongside its own. ms repo's CI mirrors with a checkout step for `mnemonic-toolkit`.

- **Cross-repo cycle-close gates.** PE tags push within the same session; FOLLOWUPS resolutions commit with reciprocal SHA citations; CHANGELOG entries cross-cite.

- **Review dispatch (P3b).** Per `feedback_opus_primary_review_agent`, default Opus on the cross-repo reviewer. Recommended dispatch: a single Opus subagent with full constellation context covering both repos' P3b surface (one inline-copy + one apply site = small enough to hold in one head). Fallback: if context budget pushes back, two sibling-scoped reviewers with explicit cross-reference instructions ("the other repo's PR is at <URL>; verify the inline `pin_pages_for` implementations match"). The CI invariant test is the regression backstop for either dispatch shape.

---

## §6. Acceptance gates

Numbered gates with explicit-pass criteria. Mirrors Cycle A SPEC §6 pattern.

### G1 — Functional correctness

Each of the 5 sites successfully mlocks under default test environment (`RLIMIT_MEMLOCK ≥ 64KiB`; no cgroup `memory.lock_size` restriction). Integration test asserts the relevant page range is locked during scope and unlocked after Drop.

Verification mechanism (test-only):
- **Linux**: parse `/proc/self/smaps` for the address range matching the buffer; assert `Locked > 0` for the entry. Test helper `mlock::tests::is_page_range_locked(addr: *const u8, len: usize) -> bool`.
- **macOS**: `mach_vm_region_info` via the `mach` crate (already in transitive deps for some platforms; if not, a dev-only addition is acceptable). Assert region's `user_tag` reflects wired memory. Same test helper signature.

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

### G4 — Cycle A discipline preserved (no `unsafe` post-free reads)

Sites still Drop-zeroize when mlock fails. Verification via `#[cfg(test)]` drop-probe instrumentation (NOT post-free pointer reads — those are undefined behavior):

```rust
#[cfg(test)]
impl<T: Zeroize> MlockedZeroizing<T> {
    pub fn new_with_drop_probe<F>(value: T, probe: F) -> Self
    where F: FnOnce(&[u8]) + 'static { /* ... */ }
    // The probe is invoked AFTER munlock + zeroize but BEFORE
    // deallocating the inner allocation. The test inspects the
    // zeroed buffer via the probe callback.
}
```

Test: hold a `MlockedZeroizing<Vec<u8>>::new_with_drop_probe([0xAA; 32], |buf| {...})` with known bytes under `MNEMONIC_TEST_MLOCK_FAIL_MODE=eperm`; force Drop (let-binding goes out of scope); probe callback asserts the observed buffer is all-zero.

### G5 — Cross-repo lockstep

- PE tags `mnemonic-toolkit-v0.10.0` + `ms-cli-v0.3.0` push within the same session.
- CHANGELOG entries in both repos cross-cite each other's tag commit SHAs.
- Companion FOLLOWUP resolution commits within 24h of each other.

### G6 — Inline-copy equivalence

CI invariant test passes on every PR in both repos: toolkit's `mlock.rs::pin_pages_for` and ms-cli's `mlock.rs::pin_pages_for` functionally equivalent after normalization (strip whitespace, comments, doc-comments). Verification: byte-equal after normalization.

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
