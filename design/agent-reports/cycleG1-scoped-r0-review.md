# Cycle G.1 scoped R0 review — g4_a mlock fix + ms-cli-v0.14.1 pin bump

**Reviewer:** Fable (scoped R0, read-only), per user directive. Full cross-repo diff + release checklist.
**Dispatched:** 2026-07-09 (Cycle G.1, scoped R0 round 1). Persisted verbatim per CLAUDE.md.

## VERDICT: GREEN — 0 Critical / 0 Important (4 Minor/observations, none ship-blocking)

## Empirical results (all run live in this review)

| Suite | Result |
|---|---|
| `cargo +nightly miri test -p mnemonic-toolkit --lib mlock::` | **7 passed / 0 failed / 3 ignored** — g4_a green (was the red job) |
| `cargo test -p mnemonic-toolkit --lib mlock::` (real OS) | 7 / 0 / 3 |
| `cargo test -p ms-cli --bins mlock::` (real OS) | 7 / 0 / 3 |
| g6 invariant (`SIBLING_REPO_PATH=…/mnemonic-secret`, `--include-ignored`) | **2/2 passed** (normalized byte-equal + manifest parity) |
| Raw `diff` of the two test-module regions (toolkit :357-460 vs ms :362-465) | byte-identical |

## Per-item verification

1. **Code fix correctness — CONFIRMED, architect's exact fix.** Both `round_to_pages` (mlock.rs:129, private) and `page_size()` (:29, private) are in scope via `use super::*` (:358). Range bound sound: len=64 ≤ page (≥4096; miri stubs 4096 at :32-35) ⇒ touches exactly 1 or 2 pages, never 0 (len>0 path of round_to_pages) or 3. Compute-equality sound: `pin_pages_for` calls `round_to_pages(buf.as_ptr() as usize, buf.len(), page_size())` (:94) with identical args; no mutation of `v` between pin and `expected` ⇒ same pointer. **Tautology judgment: the assert_eq is near-tautological on the success path, but NOT pure** — it fails hard if mlock soft-fails (`PinnedPageRange::empty()` → page_count 0, :111-125), preserving the old test's implicit "pin actually succeeded" assertion, and guards future pin/round divergence. The `(1..=2)` range assert carries the layout-independent contract. Architect's keep-both judgment is correct.
2. **g6 byte-identity — PASSES** (2/2, above). Diffs in the two repos are token-for-token identical including comments.
3. **Miri green — CONFIRMED** (7/0/3; previously this exact test failed `left: 2, right: 1`). Real-OS both repos green; the 3 ignored are the fault-injection tests, still ignored.
4. **Release-site completeness — COMPLETE.** install.sh:38, manual.yml:90, quickstart.yml:87, technical-manual.yml:117 all `ms-cli-v0.14.1`. Repo-wide sweep: every remaining `ms-cli-v0.14.0` string is historical (CHANGELOG.md v0.82.0 entry, design/SPEC/PLAN/agent-reports for Cycle F). No `--tag ms-cli` lines exist in `docs/manual/src/` or `docs/quickstart/src/` (grep empty). sibling-pin-check will pass: all live install lines match install.sh.
5. **Examples.md regen scope — EXACT.** One-line diff: the `--list` table ms row `ms-cli-v0.14.0`→`v0.14.1` (~line 104). gen.sh's version FATAL still pins `mnemonic 0.82.0` (gen.sh:44) — consistent with the unchanged binary; nothing else moved.
6. **ms-cli bump — CORRECT.** Cargo.toml 0.14.0→0.14.1; Cargo.lock bumped only the `ms-cli` package (all other 0.14.x hits are vendored third-party: hashbrown/bitcoin/bitcoin_hashes). No README self-pin. CHANGELOG: see Minor 2.
7. **No toolkit bump — CORRECT SemVer.** The toolkit mlock.rs hunk is entirely inside `#[cfg(test)] mod tests` (:357; hunk at :426+) — zero production lines. Pin/workflow/doc-text edits only otherwise. No tag ⇒ changelog-check doesn't fire; precedent already set by post-tag doc commit 293a887a.
8. **v0.82.0 soundness — UNTOUCHED.** Fix is uncommitted working tree; master is one doc-only commit past `mnemonic-toolkit-v0.82.0`; no production code in the diff.
9. **Ship order — the planned order (ms first) is correct and NECESSARY.** Toolkit's g6 job resolves the ms-cli tag dynamically from install.sh and checks out mnemonic-secret at that ref (toolkit rust.yml:288-305) — a toolkit-first push would hard-fail the checkout on the nonexistent tag AND red all 3 doc workflows (each self-triggers on its own `.yml` change and runs `cargo install --git --tag ms-cli-v0.14.1`). The examples required check is order-safe: gen.sh runs install.sh only in `--list`/`--dry-run` (pure text, no install), so it needs neither the tag nor crates.io. crates.io publish gates nothing in toolkit CI (ritual/user-facing only). **See Minor 1 for the one residual hazard.**
10. **FOLLOWUP accuracy — ACCURATE.** Status flipped with correct empirical claims; the "NO re-vendor" correction is verified true: toolkit depends on `ms-codec = "0.7"` (crates.io; crates/mnemonic-toolkit/Cargo.toml:32), no `ms-cli` package in toolkit Cargo.lock.

## Findings

**Minor 1 — one-sided transient g6 red on the ms push (ordering residual; plan is still the right order).** mnemonic-secret has a REVERSE g6 job that checks out mnemonic-toolkit at **`ref: master`** at job-run time (ms rust.yml `g6-invariant`, ~:245-250), and ms rust.yml fires on this push (paths include `crates/ms-cli/**` + `Cargo.toml`/`Cargo.lock`). If that job runs before the toolkit commit lands, it compares new-ms vs old-toolkit-master → red. ms master is NOT branch-protected (verified: API 404), so it's cosmetic and self-heals on re-run. Mitigation: push the toolkit commit promptly after tagging, then re-run the ms `rust` workflow if its g6 recorded red. (ms rust.yml does not fire on tags, so the tag push itself only triggers man-release.)

**Minor 2 — ms CHANGELOG entry skipped, and the premise "no CHANGELOG" is factually wrong.** mnemonic-secret HAS a per-release CHANGELOG.md whose convention covers ms-cli PATCH releases (0.13.1 and 0.13.2 both have entries). It has no 0.14.0 entry (pre-existing Cycle-F omission — the funds-messaging demotion is undocumented there) and this fix adds no 0.14.1 entry. No CI gate exists in mnemonic-secret, so not ship-blocking; recommend backfilling `## ms-cli [0.14.1]` (+ ideally the missing `[0.14.0]`) in the ms commit, or filing a FOLLOWUP.

**Minor 3 — stale fmt-exemption comments (pre-existing, optional tidy):** toolkit rust.yml:45 says the g6 pin is "currently `ms-cli-v0.13.2`"; ms rust.yml:51 says "g6 pins the FROZEN `ms-cli-v0.7.0` tag". Both were already wrong before this diff.

**Minor 4 — man-release side effect (informational):** the `ms-cli-v0.14.1` tag rebuilds and attaches man pages + musl binaries per the normal ritual; version-bump-only content is fine, no tag↔Cargo.toml consistency gate to trip.

## Release-readiness

**READY TO SHIP as planned (ms-cli push+tag first, then the single atomic toolkit commit): 0 Critical / 0 Important; the fix is verified green on miri, real-OS (both repos), and g6, and every live pin site is complete — only optional Minors (CHANGELOG backfill, expect-and-rerun the transient ms-side g6) remain.**

---
**Orchestrator dispositions (2026-07-09):**
- **Minor 1** → handled operationally in the ship sequence (toolkit commit pushed immediately after the ms tag; ms-side `rust`/g6 re-run if transient-red).
- **Minor 2** → FOLDED: backfilled `## ms-cli [0.14.1]` **and** the missing `[0.14.0]` (Cycle F funds demotion) into `mnemonic-secret/CHANGELOG.md` in the ms commit. Corrected the stale "ms has no CHANGELOG" memory note.
- **Minor 3** → left as tracked pre-existing tidy (comment-only, g6-invisible; not folded to keep the shipped diff ≈ the reviewed diff).
- **Minor 4** → informational; expected.
- **crates.io:** ms-cli-v0.14.1 is a **test-only PATCH → NOT published to crates.io**, matching the explicit `[0.13.2]` precedent ("binary-asset-only PATCH; the tag ships the binary"); crates.io gates nothing and the install path is `--git --tag`. Documented in the `[0.14.1]` CHANGELOG entry.
