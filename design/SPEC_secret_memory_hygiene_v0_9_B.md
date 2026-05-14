# SPEC — Secret-memory hygiene Cycle B (mlock infrastructure)

**Cycle:** v0.9.0 Cycle B (mlock page-pinning, POSIX-only).
**Status:** Phase 0 closed (R2 CLEAR commit `f84d998`); Phase 1 shipped (commits `4465940`/`3be9b77`/`c3509af`/`eae66c6`); **Phase 0 R3 Fix B fold in flight** (this revision, resolving Phase 2 R0 C-1 indirection trap — commit `8193e22`). Reviewer-loop continues until 0 critical / 0 important on the R3 Opus pass.
**Predecessors:** v0.9.0 Cycle A (`SPEC_secret_memory_hygiene_v0_9_0.md`, shipped 2026-05-13 — tags `mnemonic-toolkit-v0.9.2`, `ms-codec-v0.1.3`, `ms-cli-v0.2.2`).
**Pre-SPEC questions resolved:** `cycle-b-pre-spec-questions` FOLLOWUP (toolkit `design/FOLLOWUPS.md`, commit `1efac85`).
**Authoring session:** 2026-05-13, v1.0 roadmap-survey Bucket-1 drill-down + brainstorming pass (5 Qs locked).
**Phase 2 R0 Fix B fold:** 2026-05-13, applied per `design/agent-reports/v0_9_B-phase-2-mlock-module-r0.md` (commit `8193e22`). The wrapper type `MlockedZeroizing<T>` was RETIRED; the slice-fn primitive `pin_pages_for(&[u8])` is the sole mlock API. Sites 2/3 add `_entropy_pin: PinnedPageRange` sibling fields; Site 4 adds a function-local pin. See §2 row 1 + §4 P2 + §4 P3a + §6 G4 + §6 G6 for the patched text.
**Phase 3a R0 v2 LOCK:** 2026-05-13, applied per `design/agent-reports/v0_9_B-phase-3a-toolkit-applications-r0.md` §10b/§10c (commit `9be0f0f`). Locked: (a) Cycle A baseline-shape narrative corrected in §2 row 5 + §4 P3a (Cycle A actually shipped `Vec<u8>` + `impl Drop` and `Option<Vec<u8>>`; Phase 3a completes the deferred FOLLOWUP `resolved-slot-entropy-zeroizing-field` by migrating to `Zeroizing<Vec<u8>>` resp. `Option<Zeroizing<Vec<u8>>>` AND adds `_entropy_pin` siblings in lockstep); (b) `ResolvedSlot._entropy_pin` is `Option<Arc<PinnedPageRange>>` (Arc-wrapped to preserve `derive(Clone)`); (c) Site 1 pin lands AFTER `apply_stdin_substitutions` (synthetic-args mutation window); (d) `.github/workflows/rust.yml` gains a release-build subprocess job for G2.3-release coverage (Linux-only).

**Phase 3a R0 v3-fold RESCOPE — Path B-lite:** 2026-05-13, supersedes the R0 v2 LOCK. Applied per `~/.claude/plans/2026-05-13-cycle-b-phase-3a-rescope-proposal.md`; reviewer reports `design/agent-reports/v0_9_B-phase-3a-rescope-r0{,-v3,-v3-fold}.md` (final pass LOCK 0/0). Carved out: (a) the Cycle-A→Zeroizing field-type migration (`ResolvedSlot.entropy: Option<Vec<u8>>` and `DerivedAccount.entropy: Vec<u8>` stay UNCHANGED in Phase 3a; deferred to v0.10.1 patch via FOLLOWUP `resolved-slot-derived-account-zeroizing-field` which supersedes `resolved-slot-entropy-zeroizing-field`); (b) `impl Drop for DerivedAccount` PRESERVED (Cycle A scrub stays); (c) `tests/lint_zeroize_discipline.rs` UNTOUCHED in Phase 3a (relabel + new row deferred to v0.10.1); (d) Site 1 per-handler anchors corrected (convert/derive_child have NO `apply_stdin_substitutions` — they pin local-binding effective_*/from_value/stdin_passphrase variables instead); (e) integration tests dropped in favor of in-source `#[cfg(test)]` residency tests using the existing Phase 2 `MNEMONIC_TEST_MLOCK_FAIL_MODE=eperm` + `failure_count_for_test()` mechanism. PRESERVED from R0 v2 LOCK: all struct-sibling pins on `ResolvedSlot` (Arc-wrapped) and `DerivedAccount` (plain), Site 4 bip85 pins, main.rs wire, CI release-build job. Threat-model coverage equivalent to R0 v2 LOCK at all 5 sites.

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
| 1 | API: slice fn (single primitive) | `pin_pages_for(buf: &[u8]) -> PinnedPageRange`. Page-granularity is explicit in the return type: `PinnedPageRange { start: *const u8, page_count: usize }` with `Drop` impl that munlocks. Pins the page range covering `buf`. **Page-rounding formula** (pinned to avoid ambiguity): `start = addr & !(PAGE_SIZE - 1)` (round down); `end = (addr + len + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)` (round up); `page_count = (end - start) / PAGE_SIZE`. A slice exactly fitting one page → `page_count = 1`; exactly two pages → `page_count = 2`. **Zero-length slice (`buf.len() == 0`) is a no-op**: returns `PinnedPageRange { start: ptr::null(), page_count: 0 }` whose Drop is also a no-op. No `mlock(2)` syscall is issued for zero-length (Linux `mlock(addr, 0)` returns EINVAL; macOS returns success; the no-op avoids both). Callers accept page-residue from co-resident non-secret allocations (Cycle C addresses at allocator level — see §3 `OOS-page-residue-elimination`). **Composition with `Zeroizing<T>`:** `pin_pages_for` operates on the buffer's heap pages (via `&buf[..]`-yielded data-pointer), which is what mlock actually pins. A wrapper type around `Box<Zeroizing<T>>` was rejected during Phase 2 R0 (commit `8193e22`, finding C-1): for `T = Vec<u8>` the wrapper would page-align-allocate the 24-byte Vec header (ptr + len + cap) but leave the Vec's heap-allocated data buffer at a separate, NON-mlocked allocation — the wrapper would be hollow at Sites 2-4. Fix B (slice-fn-only) is the locked design; see Phase 2 R0 report `design/agent-reports/v0_9_B-phase-2-mlock-module-r0.md` §3. |
| 2 | API: state singleton | `MlockState` — process-static via `std::sync::OnceLock<MlockState>`. Fields: `failure_count: AtomicUsize`, `total_bytes_unlocked: AtomicUsize`, `first_errno: OnceLock<i32>`. Thread-safe; lock-free reads on the hot path. |
| 3 | API: end-of-process emit | `pub fn report_at_exit()`. Called from `main()` in both `mnemonic-toolkit` (bin) and `ms-cli` (bin). Emits a 2-line stderr summary iff `failure_count > 0`. Format pinned in §6 G2. |
| 4 | Precursor refactor | `bip85::derive_entropy(index: u32) -> [u8; 64]` heap-promoted to `-> Zeroizing<Vec<u8>>`. 7 callees in `format_*` functions updated. P1-only; no new public API surface beyond the return-type change. (Shipped 2026-05-13 at toolkit commits `4465940`/`3be9b77`/`c3509af`/`eae66c6`; per Phase 1 R0/R1 reports the plan's "6 callees" framing was off-by-one — `format_dice_rolls` was missed.) |
| 5 | Site applications | All 5 sites use the slice-fn primitive (no wrapper type). **Site 1 (toolkit)**: clap fields across **4 cmd structs** with secret-bearing user input — `BundleArgs`, `VerifyBundleArgs`, `ConvertArgs`, `DeriveChildArgs`. Direct named secret-string clap fields: 5 (`BundleArgs.passphrase`, `VerifyBundleArgs.passphrase`, `ConvertArgs.{passphrase, bip38_passphrase}`, `DeriveChildArgs.passphrase`). Plus repeating-flag value strings (variable per-invocation): `BundleArgs.slot[i].value` + `VerifyBundleArgs.slot[i].value` + `ConvertArgs.from[i].value` + `DeriveChildArgs.from.value`. **Per-handler anchor placement** (per Path B-lite §3.1; `apply_stdin_substitutions` exists only in `bundle.rs:1227` and `verify_bundle.rs:565`, NOT in convert/derive_child): `bundle.rs` + `verify_bundle.rs` pin via the `&synthetic_args` re-binding after `apply_stdin_substitutions` returns; `convert.rs` pins `effective_passphrase` / `effective_bip38_passphrase` / `primary_value` after they're bound (post-`:668`); `derive_child.rs` pins `from_value: Zeroizing<String>` and `stdin_passphrase: Option<Zeroizing<String>>` after they're bound (post-`:122`). All anchors are post-substitution / post-stdin-read so the pin covers the actual secret bytes consumed downstream. **Site 2 (toolkit)**: `ResolvedSlot` adds sibling `_entropy_pin: Option<Arc<PinnedPageRange>>` (Arc-wrapped to preserve the `derive(Clone)` semantics; Arc refcount ensures the munlock fires exactly once when the final clone drops). The `entropy: Option<Vec<u8>>` field type is UNCHANGED (Cycle A baseline preserved; the deferred FOLLOWUP `resolved-slot-derived-account-zeroizing-field` migrates it to `Option<Zeroizing<Vec<u8>>>` in v0.10.1). **Field declaration order** ensures `entropy` drops first then `_entropy_pin` drops per RFC 1857. For Site 2 the entropy Drop is a plain Vec dealloc (NO scrub under Cycle A baseline); the bytes-may-persist-on-heap-after-dealloc risk is unchanged from Cycle A — mlock pins the page during the buffer's lifetime, but post-dealloc the bytes can persist in the freed allocation until allocator reuse. **Site 3 (toolkit)**: `DerivedAccount` adds sibling `_entropy_pin: PinnedPageRange` field (no Arc — `DerivedAccount` is not Clone and is consumed via `into_parts`). The `entropy: Vec<u8>` field type and `impl Drop for DerivedAccount` are UNCHANGED (preserved from Cycle A). Same declaration-order discipline. For Site 3 the entropy Drop triggers Cycle A's `impl Drop for DerivedAccount` zeroize before `_entropy_pin` munlocks (zeroize-while-still-pinned — the strictest threat-model ordering). **Site 4 (toolkit)**: bip85's 7 `format_*` functions are function-local-scoped; a `let _pin = pin_pages_for(&entropy[..]);` immediately after the `derive_entropy(...)?` binding pins the data-buffer pages for the function-body lifetime. Local-binding drop order is REVERSE of declaration (per Rust Reference §"destructors"): `_pin` drops first (munlock) then `entropy` drops (zeroize-after-munlock; entropy is `Zeroizing<Vec<u8>>` from Phase 1). The post-munlock-pre-zeroize window is microseconds and not load-bearing for the threat model. **Site 5 (ms-cli)**: `read_stdin()` String at `parse.rs:45` — `let _pin = pin_pages_for(s.as_bytes());` post-receipt, scope-bound to `s`. |
| 6 | Errno discipline | All errno classes soft-fail in release: `EPERM`, `ENOMEM` (RLIMIT_MEMLOCK or cgroup memory.lock_size), `EAGAIN` (per-process lock limit), `ENOTSUP`, others. `EINVAL` SHOULD be unreachable from the slice-fn API by construction (rounds to page boundary; zero-length is a no-op short-circuit). If an `EINVAL` ever surfaces it indicates a bug in this module: debug builds trip `debug_assert!`; release builds soft-fail like the other errno classes. No `Result`-typed return at the user-facing API; mlock outcomes are reported via `MlockState`, not propagated to callers. |
| 7 | Test surface | (a) `MlockState` aggregation unit tests; (b) `#[cfg(test)]` env-var hook `MNEMONIC_TEST_MLOCK_FAIL_MODE={eperm,enomem,einval,off}` for fault injection at the `pin_pages_for` callsite, cache shape pinned `OnceLock<FailMode>` (subprocess isolation required for per-test mode variation; see §4 P2). **Phase 2 retains** G2.1 (eperm; in-process single-shot), G2.3-debug (debug_assert), G2.4 (off control). **Phase 3a adds** G2.2 (enomem), G2.3-release, G2.5 (stderr summary) — these defer because Phase 2 has no production mlock callsite for subprocess-based fault injection to invoke; Phase 3a adds the callsites; (c) POSIX integration tests asserting page-locked state during scope + munlocked after Drop via `/proc/self/smaps` (Linux) and `mach_vm_region` (macOS) test-only helpers, covering single-page / multi-page / zero-length / page-aligned cases (see §6 G1.1-G1.4); (d) Zeroize-on-Drop discipline reuses Cycle A's `lint_zeroize_discipline.rs` evidence-anchor pattern for the new `_entropy_pin` fields at Sites 2/3 + the function-local pin at Site 4 + the clap-scope-bound pin at Site 1; (e) CI invariant test enforcing the diff-manifest equivalence between toolkit's and ms-cli's `mlock.rs` (see §6 G6 + §5); (f) Miri pass on the `unsafe` blocks in `pin_pages_for` and `PinnedPageRange::drop` via `cargo +nightly miri test -p mnemonic-toolkit mlock::` (see §6 G4 Miri gate). |

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

Phase 2's design was patched in-flight after R0 surfaced the C-1 indirection trap (commit `8193e22`, report `design/agent-reports/v0_9_B-phase-2-mlock-module-r0.md`). The wrapper type `MlockedZeroizing<T>` is RETIRED; Fix B (slice-fn only) is the locked design. See §2 row 1 for the rationale.

- **Crate-shape change:** create `crates/mnemonic-toolkit/src/lib.rs` exposing `pub mod mlock;` (hybrid lib + bin). `[[bin]]` stays at `path = "src/main.rs"`; other modules stay binary-private. Integration tests can `use mnemonic_toolkit::mlock::*`. Per R0 §1 Option C (smallest cascade).
- New module at `crates/mnemonic-toolkit/src/mlock.rs`:
  - `pub fn pin_pages_for(buf: &[u8]) -> PinnedPageRange`. Zero-length is a no-op (returns empty range; no syscall); see §2 row 1 for the page-rounding formula.
  - `pub struct PinnedPageRange { pub start: *const u8, pub page_count: usize }` with `Drop` impl (munlock when `page_count > 0`; no-op otherwise).
  - `MlockState` private struct (fields: `failure_count: AtomicUsize`, `total_bytes_unlocked: AtomicUsize`, `first_errno: OnceLock<i32>`) + `static MLOCK_STATE: OnceLock<MlockState>` accessor + `fn record_failure(errno: i32, bytes: usize)` (idempotent on `first_errno`; monotonic on counters).
  - `pub fn report_at_exit()`. Called from `main()` in both binaries.
  - Private `fn page_size() -> usize` cached in `OnceLock<usize>`, sourced via `libc::sysconf(libc::_SC_PAGESIZE) as usize`. Linux x86_64 = 4096; macOS aarch64 = 16384. All page-rounding tests express sizes as `n * page_size()`, never hard-coded.
- **CI workflow:** create `.github/workflows/rust.yml` (toolkit has no Rust CI today — `manual.yml` + `quickstart.yml` are docs-build only). Jobs: `test` (Ubuntu + macOS matrix; `ulimit -l ≥ 65536` set on Linux per SPEC §6 G3), `miri` (Ubuntu nightly; `cargo +nightly miri test -p mnemonic-toolkit mlock::`), `clippy` (`cargo clippy --all-targets -- -D warnings`).
- `#[cfg(test)]` hooks:
  - **Env-var fault injection** with pinned cache shape: `static FAIL_MODE: OnceLock<FailMode> = OnceLock::new();` resolved at first mlock call via `std::env::var("MNEMONIC_TEST_MLOCK_FAIL_MODE").ok().and_then(parse)`. Supported values: `eperm`, `enomem`, `einval`, `off`. Production code path is `cfg(not(test))` direct mlock; test path branches on the cached `FAIL_MODE`. Cross-thread coherence: `OnceLock` guarantees first-writer-wins; all threads observe the same `FailMode` for the lifetime of the process. **Tests requiring per-test mode variation MUST use subprocess isolation** (`assert_cmd::Command::cargo_bin(...)` with per-invocation env), not `cargo test`'s default in-process parallelism (which would all share the first-resolved `FAIL_MODE`). **Phase 2 subprocess tests defer to Phase 3a** (no Phase-2 production callsite exists yet for the subprocess to invoke; see §2 row 7).
- **First-party `unsafe` SAFETY-comment discipline:** add `tests/lint_safety_first_party_mlock.rs` (peer of `lint_safety_third_party_blocked.rs`) scanning `src/mlock.rs` for `unsafe {` opener tokens and asserting a `SAFETY:` comment within ±5 lines above. Under Fix B the `unsafe` block count is 2 (mlock in `pin_pages_for`; munlock in `PinnedPageRange::drop`); both carry SAFETY comments.
- Module is reviewable in isolation; no applications yet (P3a + P3b apply at the 5 sites).
- Approximate scope: ~150 LOC mlock + ~50 LOC lint test + ~250 LOC integration/unit tests + ~50 LOC CI YAML = ~500 LOC total Phase 2 footprint (Fix B shrunk the ~270 LOC mlock module estimate from the pre-R0 plan by removing `MlockedZeroizing<T>` and its manual `unsafe` alloc/dealloc/ptr-write Drop body).

### P3 — apply at sites 1-5 (cross-repo)

#### P3a (toolkit)

All Phase 3a sites use the slice-fn primitive `pin_pages_for` (Fix B; no wrapper type). See §2 row 5 for the locked apply pattern.

**Phase 3a R0 v3-fold RESCOPE — Path B-lite** (proposal `~/.claude/plans/2026-05-13-cycle-b-phase-3a-rescope-proposal.md`; reviewer reports `design/agent-reports/v0_9_B-phase-3a-rescope-r0{,-v3,-v3-fold}.md`): supersedes the R0 v2 LOCK (commit `9be0f0f`). The R0 v2 LOCK bundled the mlock pin work with the deferred Cycle-A→Zeroizing field-type migration; Path B-lite carves out the field-type migration to v0.10.1 patch (FOLLOWUP `resolved-slot-derived-account-zeroizing-field`, supersedes `resolved-slot-entropy-zeroizing-field`). All struct-sibling pins on `ResolvedSlot` and `DerivedAccount` are PRESERVED (full Cycle B threat-model coverage); the Cycle A baseline (`entropy: Vec<u8>` + `impl Drop for DerivedAccount`; `entropy: Option<Vec<u8>>` for `ResolvedSlot`) ships UNCHANGED. Cycle A audit-trail (`tests/lint_zeroize_discipline.rs`) UNTOUCHED.

- **Site 1**: insert `let _pin = pin_pages_for(field.as_bytes());` post-substitution / post-stdin-read in each of the 4 secret-bearing cmd handlers. Per-handler anchors (verified): `bundle.rs` + `verify_bundle.rs` pin via `&synthetic_args` after `apply_stdin_substitutions` returns (`bundle.rs:113-119`, `verify_bundle.rs:117-134`); `convert.rs` pins `effective_passphrase` / `effective_bip38_passphrase` / `primary_value` after they're bound (post-`convert.rs:668`); `derive_child.rs` pins `from_value: Zeroizing<String>` and `stdin_passphrase: Option<Zeroizing<String>>` after they're bound (post-`derive_child.rs:122`). The Drop of the returned `PinnedPageRange` is bound to the rest of the handler's `run` scope.
- **Site 2**: `ResolvedSlot` adds sibling field `_entropy_pin: Option<Arc<PinnedPageRange>>` declared AFTER `entropy` (struct fields drop in declaration order per RFC 1857; entropy drops first via Vec dealloc — no scrub under Cycle A baseline — then `_entropy_pin` Arc final-drops and munlocks). Arc-wrap preserves the `derive(Clone)` semantics (the cosigner-bridging clones at `cmd/bundle.rs:1062-1073` continue to compile and share the pin via Arc refcount). The `entropy: Option<Vec<u8>>` field type is PRESERVED unchanged from Cycle A (deferred to v0.10.1 per `resolved-slot-derived-account-zeroizing-field`). 6 construction sites updated to populate `_entropy_pin` (4 sites with `Some(Arc::new(pin_pages_for(...)))` for real entropy; 2 watch-only sites with `None`): `synthesize.rs:1184` (test) + `cmd/bundle.rs:{348,417,449,491,1065}`.
- **Site 3**: `DerivedAccount` adds sibling field `_entropy_pin: PinnedPageRange` declared AFTER `entropy` (no Arc — `DerivedAccount` is not Clone and is consumed via `into_parts`). Same declaration-order discipline. The `entropy: Vec<u8>` field type and `impl Drop for DerivedAccount` (which `self.entropy.zeroize()`) are PRESERVED unchanged from Cycle A. On Drop, `entropy` triggers Cycle A's scrub first (zeroize-while-still-pinned — strictest threat-model ordering), then `_entropy_pin` munlocks. 1 construction site updated: `derive_slot.rs:77` inside `derive_bip32_from_entropy`. `into_parts()` body unchanged.
- **Site 4**: bip85's `format_*` functions (7 of them, post-P1; verified names `format_bip39_phrase`, `format_hd_seed_wif`, `format_xprv_child`, `format_hex_bytes`, `format_password_base64`, `format_password_base85`, `format_dice_rolls`) add `let _pin = pin_pages_for(&entropy[..]);` immediately after the `derive_entropy(...)?` binding. Local-binding drop order is REVERSE of declaration (Rust Reference §"destructors"), so `_pin` drops first (munlock) then `entropy` drops (zeroize-after-munlock; entropy is `Zeroizing<Vec<u8>>` from Phase 1). The post-munlock-pre-zeroize window is microseconds and not load-bearing for the threat model.
- `main()` in `crates/mnemonic-toolkit/src/main.rs` wires `mnemonic_toolkit::mlock::report_at_exit()` between the `match result` close and the `ExitCode` return (covers both Ok and Err paths; the clap-parse-error path at `main.rs:62` early-returns before any mlock callsite is reached, intentionally skipped per SPEC §3 `OOS-cross-process-aggregation` rationale).
- **CI delta**: `.github/workflows/rust.yml` adds a release-build subprocess test job: `cargo test --release --test cli_mlock_g2_subprocess` (Linux-only; the `MNEMONIC_TEST_MLOCK_FAIL_MODE` harness is `cfg(target_os = "linux")`).
- Approximate scope (Path B-lite): ~30 LOC apply edits (Sites 1+4) + ~10 LOC main.rs wire + ~20 LOC sibling-field adds at Sites 2/3 + ~15 LOC ctor-site updates (6 ResolvedSlot + 1 DerivedAccount; populate `_entropy_pin`) + ~80 LOC subprocess + in-source residency tests + ~15 LOC rust.yml release job = ~170 LOC Phase 3a footprint. (No field-type migration; no lint anchor edits; no FOLLOWUP closure; no CHANGELOG migration note — all deferred to v0.10.1.)

#### P3b (ms-cli, cross-repo)

- Inline copy of `pin_pages_for` + `PinnedPageRange` + `MlockState` (process-local; not shared with toolkit's singleton) + `report_at_exit` at `mnemonic-secret/.../ms-cli/src/mlock.rs`. Under Fix B the full toolkit `mlock` module surface inline-copies (no wrapper-type carve-out).
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

### G4 — Cycle A discipline preserved + Rust-level safety verified

Under Fix B (slice-fn-only design), G4 splits into two subgates:

**G4.a — Zeroize-on-Drop discipline preserved.** Sites still Drop-zeroize when mlock fails (where Cycle A established a Drop scrub). Sites 2/3 use struct-field declaration order with `_entropy_pin` declared AFTER `entropy` so that on Drop, `entropy` drops first then `_entropy_pin`. The `entropy` Drop SEMANTIC differs by site under Path B-lite (Cycle A baseline preserved): for Site 3 (`DerivedAccount.entropy: Vec<u8>`) this triggers Cycle A's `impl Drop for DerivedAccount` zeroize (zeroize-while-still-pinned — the strictest threat-model ordering); for Site 2 (`ResolvedSlot.entropy: Option<Vec<u8>>`) Cycle A baseline performs a plain Vec dealloc with NO scrub (per the open FOLLOWUP `resolved-slot-derived-account-zeroizing-field`, deferred to v0.10.1). For Site 2 the bytes-may-persist-on-heap-after-dealloc risk is unchanged from Cycle A — mlock pins the page during the buffer's lifetime, but post-dealloc the bytes can persist in the freed allocation until allocator reuse; mlock does not address this. Closing that gap is the v0.10.1 patch's responsibility. Site 4 uses function-local bindings where the natural `let entropy = ...; let _pin = ...;` order produces reverse-of-declaration drop (Rust Reference §"destructors"): `_pin` munlocks first, then `entropy` zeroizes (entropy is `Zeroizing<Vec<u8>>` from Phase 1). The post-munlock-pre-zeroize window is microseconds and not load-bearing for the threat model (no swap-out can occur in that window in practice).

Verification: Cycle A's existing `tests/lint_zeroize_discipline.rs` evidence-anchor pattern ships UNCHANGED in Cycle B Phase 3a (Path B-lite preserves the Cycle A baseline; the lint anchor relabel + new ResolvedSlot row are deferred to v0.10.1 with the field-type migration per FOLLOWUP `resolved-slot-derived-account-zeroizing-field`). New `_entropy_pin` field residency is verified via in-source `#[cfg(test)] mod path_b_lite_pin_tests` in `synthesize.rs`, `derive.rs`, `bip85.rs`, and `cmd/derive_child.rs` using the existing Phase 2 `MNEMONIC_TEST_MLOCK_FAIL_MODE=eperm` + `failure_count_for_test()` mechanism (asserting failure-count incremented under FAIL_MODE proves `pin_pages_for` was called along the code path under test). No new cfg(test) drop-probe wrapper API is needed (Fix B eliminates `MlockedZeroizing<T>`).

**G4.b — Rust-level safety verified by Miri.** The `unsafe` blocks in `pin_pages_for` (the `libc::mlock` syscall) and `PinnedPageRange::drop` (the `libc::munlock` syscall) are exercised by Miri under `cfg(miri)` stubs (no-op for the syscalls; `_SC_PAGESIZE` returns 4096 since Miri runs x86_64-host). Miri verifies the Rust-side pointer arithmetic, null-checks, and Drop-impl ordering. CI runs `cargo +nightly miri test -p mnemonic-toolkit mlock::` in the new `.github/workflows/rust.yml` `miri` job. Miri failure fails G4.b.

### G5 — Cross-repo lockstep

- PE tags `mnemonic-toolkit-v0.10.0` + `ms-cli-v0.3.0` push within the same session.
- CHANGELOG entries in both repos cross-cite each other's tag commit SHAs.
- Companion FOLLOWUP resolution commits within 24h of each other.

### G6 — Inline-copy equivalence (diff manifest + name-export check)

The duplicated module surface is enumerated as a **diff manifest** (the binding list of items that must remain equivalent across repos). Under Fix B (no wrapper type) the manifest is the complete `mlock.rs` surface — no toolkit-only carve-out:

- `fn pin_pages_for(buf: &[u8]) -> PinnedPageRange`
- `struct PinnedPageRange { start: *const u8, page_count: usize }` + its `impl Drop`
- `struct MlockState` (fields, accessor `static MLOCK_STATE`, methods `record_failure(errno: i32, bytes: usize)`)
- `fn report_at_exit()`
- Private `fn page_size() -> usize` cached in `OnceLock<usize>`
- Any other private errno-handling helpers used by the above (enumerated at P2 commit time)

**CI invariant test (workspace-level; mirrored in ms repo).** Asserts on every PR in both repos:

1. **Source-text equivalence** for every item in the diff manifest, normalized as:
   - Strip leading/trailing whitespace per line
   - Strip `//` line comments at start-of-trimmed-line only
   - Strip `/// ` doc-comments
   - Preserve internal whitespace inside string literals
   - **PRESERVE** `use` statements (so `use crate::Foo` vs `use ms_cli::Foo` divergence fails loudly, forcing explicit alignment via re-exports or refactor)
   - **PRESERVE** `#[cfg(target_os = "linux")]`, `#[cfg(target_os = "macos")]`, etc. (platform-divergence is intentional and must be loud)

2. **Name-export parity:** toolkit's `mlock.rs` and ms-cli's `mlock.rs` export exactly `{diff_manifest_names}` (identical sets under Fix B; no carve-outs). Any name added to one repo's `mlock.rs` without the corresponding update in the other (and in the manifest) fails the test.

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
  - Mlock-module-shape prototype seed (lines 188-260) — superseded by the slice-fn-only design after Phase 2 R0 Fix B fold
  - R3 I-R3-2 architectural trap on byte-len API — addressed by `pin_pages_for` returning `PinnedPageRange` (page-granularity explicit in type)
- **Phase 1 R0/R1 reports**: `design/agent-reports/v0_9_B-phase-1-bip85-heap-promote-r0.md` and `r1.md` — bip85 heap-promote design lock + post-implementation CLEAR
- **Phase 2 R0 report (Fix B trigger)**: `design/agent-reports/v0_9_B-phase-2-mlock-module-r0.md` — surfaced the `MlockedZeroizing<Vec<u8>>` indirection trap (C-1) and recommended Fix B (slice-fn only). This revision of the SPEC folds Fix B per the report's §3 recommendation
- **Pre-SPEC FOLLOWUP (resolved by this SPEC's P0 ship)**: `design/FOLLOWUPS.md` `cycle-b-pre-spec-questions` (commit `1efac85`)
- **Parent cycle FOLLOWUP (resolves on PE)**: `design/FOLLOWUPS.md` `secret-memory-hygiene-cycle-b`
- **Companion FOLLOWUP (filed at P0 SPEC ship)**: `mnemonic-secret/design/FOLLOWUPS.md` `secret-memory-hygiene-cycle-b`
- **Cross-repo precedent (Q5 rationale)**: `mc-codex32-extraction-retired-2026-05-03` (mnemonic-key `design/FOLLOWUPS.md`) — established the "fork-and-document-pattern over shared-crate-extraction" discipline
- **Prior art**: libsodium's two-tier mlock API — `sodium_malloc` (wrapper-shaped, page-aligned) + `sodium_mlock`/`sodium_munlock` (slice-shaped, byte-len with documented page-residue caveat). Basis for Q1's hybrid choice
- **Man pages**: POSIX `mlock(2)`; macOS `mlock(2)` (BSD-derived)
- **Out-of-repo plan artifacts**: Cycle A's plan lived at `~/.claude/plans/v0_9_0-secret-memory-hygiene.md`. Cycle B's plan will live at `~/.claude/plans/v0_9_B-secret-memory-hygiene-cycle-b.md` (drafted post-SPEC via `superpowers:writing-plans`)
