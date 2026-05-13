# v0.9.0 Cycle B Phase 0 R1 architect review

**Reviewer:** Opus 4.7 (1M context), invoked as architect-review on `design/SPEC_secret_memory_hygiene_v0_9_B.md` (commit `444e833`, master).
**Date:** 2026-05-13.
**Predecessor pattern:** Cycle A Phase 0 R1 (`design/agent-reports/v0_9_0-phase-0-spec-plan-r1.md`).
**Verdict:** **HOLD** — 2 Critical, 3 Important; SPEC needs an R2 pass before P0 ship.

## Summary

Total findings at confidence ≥ 80: **2 Critical / 3 Important / 0 Low / 0 Nit**.

Top-3 most important:
1. **C-R1-1 (§4 P2 / §6 G4)** — the `new_with_drop_probe` ordering claim ("AFTER munlock+zeroize, BEFORE deallocation") is not achievable in safe Rust as specified; Drop semantics collapse those phases for the inner `Zeroizing<T>`-owned allocation.
2. **C-R1-2 (§2 row 7 / §6 G6)** — inline-copy invariant under-specified: G6 only diffs `pin_pages_for`, but §2 row 7 says the test compares "impls"; meanwhile `PinnedPageRange`, `MlockState`, and `report_at_exit` are ALSO duplicated in ms-cli (per §4 P3b) and can silently drift.
3. **I-R1-1 (§2 row 3 / §4 P2)** — env-var fault injection "checks once on first use (cached)" cross-thread coherence is unspecified; an `AtomicU8` or `OnceLock` cache shape needs to be pinned in the SPEC or the test path is racy.

The SPEC is structurally close — §s 1/3/5/7 are clean, the threat-model framing in §1 is honest, and §6 G1/G2/G3/G5/G7 are well-formed. The findings cluster in §4 P2 (drop-probe + cache shape), §6 G4/G6 (gate weaknesses), and one §2-row-1 superset claim that needs softening. None are SPLIT-CYCLE-class.

---

## Findings (by severity)

### Critical

#### C-R1-1 — `new_with_drop_probe` ordering claim not achievable in safe Rust (confidence 92)

**Where:** §4 P2 lines 90-91; §6 G4 lines 181-194.

**Finding:** The SPEC states the probe "is invoked AFTER munlock + zeroize but BEFORE deallocating the inner Box, allowing the test to inspect the zeroed buffer (no `unsafe` post-free read)." This ordering is described as if `MlockedZeroizing<T>` can interpose between the inner `Zeroizing<T>`'s Drop and its deallocation.

In Rust, when `MlockedZeroizing<T>` wraps `Zeroizing<T>` (per §2 row 1: "Wraps `Zeroizing<T>` from Cycle A"), Drop runs outer-first then inner: the outer `MlockedZeroizing::drop()` runs, THEN the field-position inner `Zeroizing<T>` is dropped (which runs `zeroize()` then deallocates the `Vec<u8>` heap buffer). The outer Drop cannot reach into the inner buffer AFTER `Zeroizing` has zeroized it but BEFORE the inner has deallocated — those two events are inside a single `<Zeroizing as Drop>::drop()` invocation that runs only AFTER the outer Drop returns. The probe callback, called from the outer Drop, can only observe the buffer EITHER (a) before the inner Drop runs (still original bytes) or (b) by being invoked from inside the inner Drop, which the outer cannot orchestrate.

Workable alternatives the SPEC should pin down:
- (a) `MlockedZeroizing<T>` owns the buffer directly (not via `Zeroizing<T>`), implements its own `Zeroize + munlock` in a single Drop, and invokes the probe at a controlled point. This contradicts §2 row 1's "Wraps `Zeroizing<T>`" framing — it makes `MlockedZeroizing<T>` a *peer* of `Zeroizing<T>`, not a *superset wrapper*.
- (b) The probe runs at the end of the outer Drop, observes the buffer AFTER the outer's munlock but BEFORE the inner's zeroize+free. The probe then asserts the buffer is STILL `[0xAA; 32]` (unzeroed at probe time). The G4 test as written ("probe callback asserts the observed buffer is all-zero") would FAIL under this ordering.
- (c) Inline the Zeroize logic into `MlockedZeroizing::drop()` directly (do not delegate to inner `Zeroizing<T>`); track the buffer pointer separately so the outer Drop can: munlock → zeroize the buffer in place → invoke probe (buffer is all-zero) → deallocate. Requires `unsafe` for the manual zeroize-before-deallocate, which contradicts the SPEC's "no `unsafe` post-free read" claim (though the read is pre-free here, not post-free).

**Severity rationale:** This is the centerpiece of G4 — the gate the SPEC chose specifically to avoid UB. If the ordering claim is wrong, the G4 test either won't compile, will deadlock on Drop borrowing semantics, or will pass via undefined behavior. The SPEC must either redesign the wrapper composition (option (a)/(c)) and accept some `unsafe` inside the module, OR redesign G4's assertion to match what's actually observable.

**Fix:** Pick one of (a)/(b)/(c) and rewrite §4 P2 + §6 G4 with the matching observable semantics. Option (a) is cleanest: `MlockedZeroizing<T>` owns a `Box<T>` with a page-aligned allocator and implements `Zeroize` + munlock manually inside its Drop; §2 row 1 reframes "Wraps `Zeroizing<T>`" → "Drop-time behavioral superset of `Zeroizing<T>`" (semantic superset, not compositional).

#### C-R1-2 — Inline-copy invariant under-specifies what's compared (confidence 88)

**Where:** §2 row 7 lines 39 (test surface (d)); §4 P3b lines 108-111; §5 lines 130-132; §6 G6 lines 202-204.

**Finding:** The four mentions of the inline-copy invariant disagree on scope:
- §2 row 8 (d): "CI invariant test comparing `pin_pages_for` impls across repos"
- §4 P3b: ms-cli inlines `pin_pages_for` + `PinnedPageRange` + `MlockState` + `report_at_exit` (FOUR items)
- §5: "Both repos' impls must remain functionally equivalent. Enforcement: a workspace-level test in toolkit reads both source files (toolkit's local copy + ms repo's copy via a path computed from a shared CI checkout), normalizes ..."
- §6 G6: "toolkit's `mlock.rs::pin_pages_for` and ms-cli's `mlock.rs::pin_pages_for` functionally equivalent"

§6 G6 is the binding gate and it ONLY checks `pin_pages_for`. But three other items (`PinnedPageRange`, `MlockState`, `report_at_exit`) are also duplicated per §4 P3b and can silently drift. For example: a future PR could refactor toolkit's `MlockState::record_failure(errno: i32)` to also stash `byte_count`, while ms-cli's copy stays single-arg; G6 would not catch the drift, and the stderr summary format (§6 G2.5) would diverge between binaries in the constellation.

Additionally, the normalization spec ("strip whitespace, comments, doc-comments") is incomplete:
- What about `use` statements? toolkit may need `use crate::Foo;` while ms-cli needs `use ms_cli::Foo;` — these are NOT noise.
- What about `extern crate libc` style differences between repos?
- What about cfg-gated paths (`#[cfg(target_os = "linux")]`)? Normalization could strip them, hiding platform divergence.
- The reviewer-checklist Q8 raises this directly: "Could the test be circumvented by a refactor that adds an unrelated helper fn?" The SPEC does not address this.

**Fix:** Tighten in three steps:
1. Promote G6 from `pin_pages_for`-only to "the duplicated module surface" with an explicit list: `{pin_pages_for, PinnedPageRange (struct + Drop impl), MlockState (struct + accessor + report_at_exit), errno-handling helpers}`. The list is the "diff manifest"; the test compares all items in the manifest by normalized source.
2. Pin the normalization spec: strip leading/trailing whitespace per line; preserve internal whitespace inside string literals; strip `//` line comments only at start-of-trimmed-line; strip `/// ` doc-comments; do NOT strip `#[cfg(...)]` attributes; do NOT strip `use` statements (so `use` divergences trip the test loudly, forcing explicit alignment).
3. Document the "helper-fn circumvention" mitigation: the test should compare the post-expansion (post-macro / post-helper-inlining is too strong; instead) the test should hash the SOURCE TEXT of every named item in the diff manifest, and the manifest itself is also under test (a separate static-list assertion: "toolkit's `mlock.rs` exports exactly these names; ms-cli's `mlock.rs` exports exactly these names; the two lists are identical modulo `MlockedZeroizing<T>` which is toolkit-only").

**Severity rationale:** This is the cross-repo correctness backstop. If G6 silently allows drift on `MlockState` or `report_at_exit`, the v0.10.0 ship can stderr-emit differently from the two binaries in a pipeline (`mnemonic bundle | ms-cli decode`) — exactly the cross-repo coordination invariant Q5 was designed to defend.

### Important

#### I-R1-1 — Env-var fault injection cache cross-thread coherence (confidence 84)

**Where:** §2 row 8 (b) line 40; §4 P2 line 90.

**Finding:** "every mlock callsite checks `std::env::var("MNEMONIC_TEST_MLOCK_FAIL_MODE")` once on first use (cached)" — the cache shape isn't specified. `std::env::var` is racy if the process modifies its environment from another thread (Rust 2024 / unsafe `set_var` discipline); but assuming tests don't `set_var` mid-flight, the question is: what holds the cached value?

Options:
- `static MODE: OnceLock<FailMode> = OnceLock::new();` — coherent across threads via the `OnceLock` memory ordering, BUT all threads see the first-resolved value, which may be a problem if integration tests want different modes in different test fns (parallel test threads in `cargo test`).
- `thread_local! { static MODE: ... }` — per-thread caching, multiple parallel tests can inject independently, but the FIRST mlock call from each thread reads env vars (slightly slower).
- An `AtomicU8` indexing a `[FailMode; 4]` table — fast, but mutation racy.

Cycle A's lint-tests run with `cargo test`'s default parallel thread pool. If `MNEMONIC_TEST_MLOCK_FAIL_MODE=eperm` is set process-wide before the test binary runs (CI env var), `OnceLock` is fine. If individual `#[test]` fns try to set/unset the env var per-test (a natural test ergonomics pattern), `OnceLock` poisoning will cause all-but-first to see the wrong value.

**Fix:** Pin the cache shape in §4 P2. Recommended: `OnceLock<FailMode>` resolved at first mlock call from `std::env::var` — and the G2 test pattern is "set the env var BEFORE the binary launches; do not mutate mid-run." Document that `cargo test` invocations needing different modes must run in separate `--test-threads=1` invocations or use `assert_cmd`-style subprocess isolation. This is the same pattern Cycle A uses for its lint-test infrastructure.

#### I-R1-2 — `pin_pages_for(&[])` zero-length slice edge case unspecified (confidence 82)

**Where:** §2 row 2 line 34; §4 P2 line 86.

**Finding:** §2 row 2 says "Pins the page range covering `buf` (start rounded down, end rounded up to page boundary)." For a zero-length slice (`buf.len() == 0`), the page range is technically empty (end == start after rounding). Behaviors:
- Linux `mlock(addr, 0)` returns `EINVAL` (man mlock(2): "The value of len may not be zero...").
- macOS `mlock(addr, 0)` returns success (BSD-derived; consistent with `mlock(2)` returning 0 for 0-length).

The SPEC's errno discipline (§2 row 7) says "`EINVAL` SHOULD be unreachable from the hybrid API by construction" and "If an `EINVAL` ever surfaces it indicates a bug in this module: debug builds trip `debug_assert!`." But Site 1 (clap fields) and Site 5 (ms-cli stdin) can naturally receive empty strings — e.g., user passes `--passphrase ''` or stdin is closed without input. Each empty `pin_pages_for("".as_bytes())` call would trip `debug_assert!` on Linux under default test config, breaking G2.3.

**Fix:** Add to §2 row 2: "Zero-length slice is a no-op; `pin_pages_for` returns an empty `PinnedPageRange { start: dangling-or-null, page_count: 0 }` whose Drop is also a no-op. No mlock syscall is issued." Then add to §6 G2.3 (einval test) clarification: "test injects EINVAL via the env-var fault-injection path, not via passing a zero-length slice." And add to G1: an integration test asserting `pin_pages_for(&[])` is a no-op (no `MlockState.failure_count` increment, no syscall, no panic).

#### I-R1-3 — `pin_pages_for` page-aligned slice edge case unspecified (confidence 80)

**Where:** §2 row 2 line 34; reviewer checklist Q4.

**Finding:** "start rounded down, end rounded up" — for a slice that is EXACTLY page-aligned (start address is a page boundary, length is exactly N * page_size), the rounding rules need to be explicit. If "end rounded up" means "ceil(end / page_size) * page_size" then a slice covering exactly one page rounds to one page (correct). If "end rounded up" means "end + page_size if not aligned else end" then a slice with length exactly equal to page_size rounds to one page (correct). But if implemented as `(end + page_size - 1) & !(page_size - 1)`, a slice whose end is already page-aligned rounds to the same address — correct. The ambiguity is in the SPEC prose, not the math; pin one formula.

Also: a slice spanning > 1 page (e.g., 4KiB clap-parsed String stored in a heap allocation that straddles two pages) is the common case for Site 1 fields; the SPEC should explicitly say `page_count` may be > 1 and the test surface §6 G1 must include a multi-page case (today G1 only mentions "the relevant page range" generically).

**Fix:** Pin the formula in §2 row 2: "page-aligned-down start = `addr & !(PAGE_SIZE - 1)`; page-aligned-up end = `(addr + len + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)`; `page_count = (end - start) / PAGE_SIZE`." Add G1 test case: "multi-page slice (length > PAGE_SIZE) asserts `PinnedPageRange.page_count > 1` and all spanned pages report `Locked > 0` in `/proc/self/smaps`."

### Low / Nit

None at confidence ≥ 80.

---

## Items examined and CLEARED (no finding)

These were on the reviewer's checklist; I examined them and found nothing reportable at confidence ≥ 80:

- **§1 threat-model framing**: the `/proc/PID/coredump_filter` bit assertion is correct. Per `core(5)`: bit 0 controls private anonymous mappings; bit 4 controls ELF headers; bit 3 controls private huge pages; mlocked private anonymous regions are dumped only when bit 3 is set (private huge pages) OR specific bits for shared file-backed are set. The SPEC says "bit 1 of the filter" which is shared anonymous mappings — this is *slightly* imprecise (the bit-for-locked-private-anon is actually a function of bits 0+3 on modern kernels and the relationship has shifted across kernel versions), but the overall framing (default-distro filters exclude locked regions) is correct and the imprecision doesn't rise to Important. Cite source: `man 5 core` `/proc/PID/coredump_filter` section. Confidence the imprecision is a finding: 60.
- **§1 macOS `gcore` claim**: macOS coredump behavior for mlocked regions is essentially "wired memory is included if the dumper has the entitlement; mostly not"; the SPEC's "similarly excludes" is defensible at the level of detail given. Confidence the imprecision is a finding: 55.
- **§2 row 1 superset claim**: "strict superset behavior" of `Zeroizing<T>` — this depends on C-R1-1's resolution. If option (a) is taken (peer, not wrapper), this row needs softening from "Wraps" to "Drop-time behavioral superset." Filed under C-R1-1 rather than as a separate finding.
- **§3 OOS completeness (reviewer Q12)**: `vm.swappiness` doesn't need OOS — it's a tuning knob users set, not something the toolkit affects. `madvise(MADV_DONTDUMP)` as an alternative to mlock for coredump-prevention IS worth a brief mention as an OOS class — but only at confidence ~65 since the SPEC's threat model in §1 already covers coredumps via mlock. Not reporting.
- **§4 P1 mass-balance (~150 LOC)**: bip85 return-type swap (1 fn + 6 callees) + Zeroizing wrapping at the callees + tests + byte-determinism check + 2-pass review feels about right at ~150 LOC. Honest estimate. Confidence as a finding: 30.
- **§5 cross-repo CI configuration**: the SPEC commits to "adds a checkout step for `mnemonic-secret` alongside its own" but doesn't pin the checkout SHA/branch resolution policy (pin to `main`? to a tag?). This is plan-level detail, not SPEC-level; honestly under-specified but not at Critical/Important threshold for the SPEC artifact. Confidence: 65.
- **§6 G7 wire-format pins**: the pin table cites v0.1 + v0.2 corpora. v0.9.x might have additional corpora (the SPEC says "if v0.3+ corpora exist..."). This is a P0-time check, not a SPEC defect. Confidence: 40.
- **`report_at_exit` if no mlock attempts (reviewer Q5)**: §6 G2.5 says "absent when 0" — so if `report_at_exit` runs with `failure_count == 0`, no stderr emission. This is correct and the SPEC handles it. Confidence as a finding: 20.
- **Panic paths in `MlockState` (reviewer Q5)**: `AtomicUsize` is panic-safe; `OnceLock<i32>` is panic-safe; `report_at_exit` called from `main()` returns normally on success and from a panic handler... actually, on a panic, `main()` returns early and `report_at_exit` is NOT called. This is a minor gap (panic-during-secret-handling skips the summary) but not at Important threshold because the user-visible failure is "process panicked," which dominates the missing mlock summary. Confidence: 70.

---

## Carry-forward

None. All 5 findings fold into SPEC text changes; no plan-level or design-level escalations needed.

---

## Verdict

**HOLD — 2C / 3I.** Cycle B SPEC cannot ship P0 as-is per the 0C/0I ship gate. Recommended next pass: fold C-R1-1 (pick wrapper composition shape (a)/(b)/(c) and rewrite §4 P2 + §6 G4 to match), fold C-R1-2 (tighten G6 manifest + normalization spec + helper-fn-circumvention mitigation), fold I-R1-1/2/3 (cache shape, zero-length handling, page-rounding formula). Expected R2 turnaround: small (~1 hour) — none of these need architectural redesign, only SPEC text tightening. Once R2 returns 0C/0I, P0 is shippable.

**Relevant files:**
- `/scratch/code/shibboleth/mnemonic-toolkit/design/SPEC_secret_memory_hygiene_v0_9_B.md` — primary artifact under review
- `/scratch/code/shibboleth/mnemonic-toolkit/design/SPEC_secret_memory_hygiene_v0_9_0.md` — Cycle A precedent for structural comparison
- `/scratch/code/shibboleth/mnemonic-toolkit/design/agent-reports/v0_9_0-phase-0-spec-plan-r1.md` — Cycle A R1 shape reference
- `/scratch/code/shibboleth/mnemonic-toolkit/design/FOLLOWUPS.md` (`cycle-b-pre-spec-questions` entry, lines 165-196) — locked-decision context
