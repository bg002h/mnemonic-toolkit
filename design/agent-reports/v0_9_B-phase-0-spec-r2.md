# v0.9.0 Cycle B Phase 0 R2 architect review

**Reviewer:** Opus 4.7 (1M context), invoked as architect-review on `design/SPEC_secret_memory_hygiene_v0_9_B.md` (commit `0c02247`, master, atop R1 fold).
**Date:** 2026-05-13.
**Predecessor:** Phase 0 R1 (`design/agent-reports/v0_9_B-phase-0-spec-r1.md`).
**Verdict:** **CLEAR — 0 Critical / 0 Important** at confidence ≥ 80. SPEC is shippable for P0 close pending the standard FOLLOWUP/companion-file housekeeping listed under §4 P0.

## Summary

Total findings at confidence ≥ 80: **0 Critical / 0 Important / 0 Low / 0 Nit**.

All five R1 findings are genuinely resolved by the R1 fold. No new high-confidence issues were introduced by the rewrite. The items flagged in the review prompt as worth re-examining (Miri-gate sufficiency, G4 ordering achievability, P2 mass-balance, `Send + Sync`, `pin_pages_for` thread-safety) either come in below the ≥ 80 confidence threshold or are explicitly captured by the SPEC's existing text.

## R1 fold verification

### C-R1-1 (peer composition + manual Drop orchestration) — RESOLVED

- §2 row 1 (line 33): now reads "**Behavioral superset** of Cycle A's `Zeroizing<T>` … but **NOT a compositional wrapper** … peer of `Zeroizing<T>`." The original "Wraps `Zeroizing<T>`" framing is gone. Confidence the fix matches R1's option (a): 95.
- §4 P2 (line 84): Drop body explicitly enumerated: (i) munlock → (ii) `Zeroize::zeroize(&mut *self.value)` in place via `unsafe` deref → (iii) cfg(test) probe → (iv) deallocate page-aligned Box. This is the ordering R1 said was the only achievable one (option (c) collapsed into option (a)).
- §6 G4 (lines 187-205): rewritten to mirror P2's four-step ordering. Probe sees the zeroed buffer between steps (ii) and (iv), matching the original G4 assertion target ("observed buffer is all-zero"). Miri gate added at line 205: `cargo +nightly miri test -p mnemonic-toolkit mlock::`. Miri failure fails G4.

The G4 ordering (i → ii → iii → iv) **is** achievable now: because the Drop owns a raw pointer to a page-aligned Box<T>, the drop impl can `Zeroize::zeroize` through the pointer before reading the same memory through a `&[u8]` and finally deallocating — there's no intervening inner-Drop semantics to interpose. Probe runs against still-allocated, zeroed memory (no use-after-free). Confidence the ordering works: 92.

### C-R1-2 (diff-manifest tightening) — RESOLVED

- §6 G6 (lines 213-239): now a "diff manifest" with explicit five-item list: `pin_pages_for`, `PinnedPageRange + Drop`, `MlockState` (fields + accessor + `record_failure`), `report_at_exit`, private errno-handling helpers. `MlockedZeroizing<T>` is explicitly toolkit-only and excluded.
- Normalization rules pinned (lines 227-233): PRESERVE `use` statements; PRESERVE `#[cfg]` attributes; strip line/doc comments at start-of-trimmed-line; preserve internal string-literal whitespace.
- Manifest-under-test (lines 236-237): the manifest itself is a static `&[&str]` in the CI test code, asserted against extracted names — directly addresses R1's helper-fn-circumvention concern.
- Name-export parity (line 235) adds a second layer: extracted names from both `mlock.rs` files compared against `{manifest ∪ {MlockedZeroizing}}` for toolkit and `{manifest}` for ms-cli.
- §2 row 8 (d) (line 40) and §5 (line 130) updated to match the manifest framing.

The §4 P3b duplication list (lines 107-108) covers `pin_pages_for`, `PinnedPageRange`, `MlockState`, `report_at_exit`. G6's five-item list covers all four plus "private errno-handling helpers." No item duplicated in P3b is missing from G6. Confidence the manifest is complete: 88.

### I-R1-1 (env-var cache shape pin) — RESOLVED

§4 P2 (line 90) pins `static FAIL_MODE: OnceLock<FailMode> = OnceLock::new();` with explicit cross-thread coherence story ("first-writer-wins; all threads observe the same `FailMode`") and the subprocess-isolation requirement for per-test variation (`assert_cmd::Command::cargo_bin`). §2 row 8 (b) (line 40) cross-cites.

### I-R1-2 (zero-length no-op) — RESOLVED

§2 row 2 (line 34) explicitly: `pin_pages_for(&[])` returns `PinnedPageRange { start: ptr::null(), page_count: 0 }`; no `mlock(2)` syscall issued; Drop is also a no-op. Both Linux `EINVAL` and macOS `success` are explicitly cited. §6 G1.3 (line 155) is the integration test for the no-op path. §6 G2.3 (line 164) explicitly tests EINVAL via env-var fault injection, not via empty slice. PinnedPageRange::Drop guard `page_count > 0` per §4 P2 line 86. Internally consistent.

### I-R1-3 (page-rounding formula pin + multi-page test) — RESOLVED

§2 row 2 (line 34) pins the formula: `start = addr & !(PAGE_SIZE - 1)`; `end = (addr + len + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)`; `page_count = (end - start) / PAGE_SIZE`. §6 G1.1 / G1.2 / G1.3 / G1.4 (lines 153-156) split into single-page / multi-page / zero-length / page-aligned cases.

## Items re-examined per review prompt §3-6 — no findings at ≥ 80

The review prompt called out six follow-on areas to investigate. Each is examined below.

### Prompt item 2 — Did the fold introduce new issues?

- **`unsafe` justification in §4 P2.** The SPEC pins the `unsafe` to one Drop impl plus one helper, gates it with Miri, and frames it as the cost of achieving the G4 ordering R1 demanded. The trade-off is explicitly noted on line 33 ("Implementation requires a small amount of `unsafe`"). Miri coverage on `mlock::` tests is appropriate scope. Confidence this is under-specified: 35 (well below threshold).
- **Miri-gate sufficiency.** Miri does not model `mlock(2)` syscalls, but the SPEC's Miri gate is scoped to the *Rust* `unsafe` (zeroize-through-pointer, deallocation ordering) — which is exactly what Miri is good at. The mlock-syscall correctness is covered by G1's `/proc/self/smaps` integration tests on real Linux/macOS. The two-track separation is correct. Confidence Miri is insufficient: 25.
- **G4 ordering achievability.** Walked through manually: (i) `libc::munlock(ptr, len)` — pointer still valid; (ii) `Zeroize::zeroize(&mut *ptr)` — writes through the raw pointer to still-allocated memory; (iii) `probe(slice::from_raw_parts(ptr as *const u8, len))` — reads the zeroed bytes (no aliasing violation if no &mut is live at probe time); (iv) `dealloc(ptr, layout)` — frees. Achievable in unsafe Rust. Confidence the ordering doesn't work: 12.
- **G6 manifest completeness vs P3b enumeration.** Compared §4 P3b's "inline copy of `pin_pages_for` + `PinnedPageRange` + `MlockState` + `report_at_exit`" against G6's five-item list. All four covered; the fifth ("private errno-handling helpers") is a forward-looking placeholder enumerated at P2 commit time, which is honest. Confidence anything is missing: 30.

### Prompt item 3 — Has confidence on the §1 coredump_filter / macOS gcore notes risen?

No. R1 left these at confidence 60 / 55. The R1 fold did not touch §1. The text remains "slightly imprecise" but not at ≥ 80 threshold. Not reportable.

### Prompt item 4 — P2 mass-balance honesty (~270 LOC claim)

The new P2 surface includes: `MlockedZeroizing<T>` struct + page-aligned allocator + unsafe Drop with 4-step orchestration; `pin_pages_for` + `PinnedPageRange + Drop`; `MlockState` (3 fields + accessor + `record_failure`); `report_at_exit`; `OnceLock<FailMode>` cache + parse helper; cfg(test) `new_with_drop_probe`; private errno helpers.

Rough sizing: page-aligned allocator (~30 LOC for `Layout::from_size_align_unchecked` + `alloc/dealloc` wrappers), `MlockedZeroizing<T>` impl (~60 LOC for new + Deref/DerefMut + Drop), `pin_pages_for + PinnedPageRange` (~40 LOC), `MlockState + accessor + record_failure` (~40 LOC), `report_at_exit` + format (~30 LOC), env-var cache + `parse_fail_mode` (~30 LOC), cfg(test) probe constructor + probe storage (~25 LOC), unit tests for state aggregation (~30 LOC — these may or may not count in the 270). Total in the 255-285 range without unit tests, 285-315 with them. **The ~270 LOC estimate is plausibly within ±15% of actuals.** R1's 350 LOC worry doesn't materialize unless tests are also rolled into the same count, in which case the SPEC's wording "Approximate scope" (line 93) covers the ambiguity. Confidence the estimate is materially off: 45. Not reportable.

### Prompt item 5 — `MlockedZeroizing<T>: Send + Sync` story

The SPEC does not explicitly state whether `MlockedZeroizing<T>` is `Send + Sync`. Examining Sites 2/3/4:
- Site 2 (`ResolvedSlot.entropy: Option<MlockedZeroizing<Vec<u8>>>`) — `ResolvedSlot` is consumed synchronously inside `synthesize`; no Send/Sync requirement surfaces.
- Site 3 (`DerivedAccount.entropy: MlockedZeroizing<Vec<u8>>`) — `DerivedAccount` is constructed and consumed inside the derive flow; no cross-thread move.
- Site 4 (bip85's heap-promoted Vec wrapped) — local-scope use.

Toolkit is single-threaded by design (CLI with synchronous flow). The `*const u8` (raw pointer) in `PinnedPageRange` is the auto-implementor-blocker, not `MlockedZeroizing<T>` itself — its owned `Box<T>` is `Send + Sync` whenever `T` is. So `MlockedZeroizing<Vec<u8>>` will likely be `Send + Sync` by auto-derivation, which is what the sites need anyway.

This is under-specified but **not at ≥ 80 confidence as a SPEC defect** because:
(a) the sites are all single-threaded;
(b) v0.10.0 is a CLI shipping target, not an async/multi-threaded library;
(c) if auto-derivation gives the right answer, no SPEC text is needed.

Confidence as a finding: 55. Could be a Phase 1 implementer-note rather than a SPEC blocker. Not reportable at the gate.

### Prompt item 6 — `pin_pages_for` overlapping-page-range thread-safety

This is the most interesting item the prompt raised. Examining:

Linux `mlock(2)` is documented as idempotent — multiple `mlock` calls on the same page do not stack into a reference count; the kernel tracks "locked or not" per page (per `Documentation/admin-guide/mm/concepts.rst` and the long-standing `VM_LOCKED` semantics). Therefore: if Thread A calls `pin_pages_for(slice_A)` and Thread B calls `pin_pages_for(slice_B)` where slice_A and slice_B share a page, and Thread A drops its `PinnedPageRange` first, Thread A's `munlock` **will unlock the shared page** even though Thread B still holds a `PinnedPageRange` over it.

In the toolkit's actual sites, this would matter if:
- Two clap-parsed `String`s landed adjacent in the heap (Site 1) — plausible.
- Site 2's `Vec<u8>` shared a page with Site 3's `Vec<u8>` — implausible (different allocations, different scopes).

However, the toolkit is single-threaded at the binary level (CLI, synchronous flow); the only concurrent users would be `cargo test` parallelism, which is exactly the scenario where the `OnceLock<FailMode>` subprocess-isolation requirement in §4 P2 already pushes tests into separate processes.

**Verdict:** Real concern in principle; not realized in Cycle B's sites due to single-threaded CLI flow. The SPEC's §3 OOS list does not call out "concurrent overlapping mlock ranges" but the threat model in §1 ("`mlock(2)`-pinned pages cannot be swapped to disk by the kernel — this eliminates the 'secret material leaks to swap'") is about a property of the *snapshot* — pages locked at the moment swap pressure happens. A transient un-lock window from racing Drops doesn't break the snapshot property unless swap pressure perfectly coincides with the racing window, which is vanishingly improbable on a CLI.

Confidence this is a SPEC defect at v0.10.0 ship: 50. Could be filed as a low-priority FOLLOWUP (`pin-pages-for-overlap-thread-safety`) for awareness if Cycle C (allocator-level) doesn't already moot it. Not reportable at ≥ 80.

## Items cleared (no finding)

- §4 P0 / Companion FOLLOWUP filing — P0 ship checklist, not a SPEC defect (confidence 15).
- §5 cross-repo CI checkout-pin policy — still under-specified per R1's prior note (confidence 65), unchanged; below threshold.
- §6 G7 corpora pins — P0-time check, not a SPEC defect (confidence 40).
- `report_at_exit` panic-path coverage — same as R1 (confidence 70), below threshold.
- `MlockedZeroizing::new_with_drop_probe` `'static` bound on the closure — slightly restrictive (cannot capture non-`'static` test fixtures by reference) but workable in practice (clone bytes into the closure); confidence as a finding: 35.

## Verdict

**CLEAR — 0C / 0I at confidence ≥ 80.** The R1 fold lands all five findings, the new `unsafe` is appropriately scoped and Miri-gated, the diff manifest with name-export parity addresses the helper-fn-circumvention concern, and no new high-confidence issues were introduced.

The items raised by the review prompt for re-examination (Send/Sync underspec, overlapping-range thread-safety, Miri sufficiency, mass-balance honesty, coredump_filter precision, macOS gcore framing) all came in below the ≥ 80 threshold. Some of them (Send/Sync, overlapping ranges) could be filed as low-priority FOLLOWUPS during Phase 1 implementation if the implementer wants belt-and-braces awareness, but none gate P0 ship.

**P0 is shippable.** Next step per §4 P0 close criteria: update `cycle-b-pre-spec-questions` FOLLOWUP with `resolved by P0 ship` and file the `secret-memory-hygiene-cycle-b` companion entry in `mnemonic-secret/design/FOLLOWUPS.md` with reciprocal `Companion:` lines.

**Relevant files:**
- `/scratch/code/shibboleth/mnemonic-toolkit/design/SPEC_secret_memory_hygiene_v0_9_B.md` — primary artifact under review
- `/scratch/code/shibboleth/mnemonic-toolkit/design/agent-reports/v0_9_B-phase-0-spec-r1.md` — R1 report (folds verified against this)
- `/scratch/code/shibboleth/mnemonic-toolkit/design/SPEC_secret_memory_hygiene_v0_9_0.md` — Cycle A SPEC (structural precedent)
