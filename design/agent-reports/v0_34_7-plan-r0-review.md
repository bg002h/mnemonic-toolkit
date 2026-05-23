# v0.34.7 — Plan-doc R0 architect review (opus) — MANDATORY pre-implementation gate (cross-repo)

**Date:** 2026-05-22
**Cycle:** v0.34.7 argv-hardening (`PR_SET_DUMPABLE`) across 4 m-format CLIs + GUI pin bumps
**Branch:** `v0.34.7-argv-hardening` (toolkit)
**Reviewer:** opus (feature-dev:code-reviewer), R0
**Target:** `design/IMPLEMENTATION_PLAN_v0_34_7_argv_hardening.md`

---

## Critical
(none)

## Important

**I1 — GUI `[mk].tag` pin string wrong in the plan (already a version behind live).** `mnemonic-gui/pinned-upstream.toml:52` live value is `mk-cli-v0.4.0` (the GUI was never bumped for mk-cli v0.4.1). The plan (`:20` verified-facts + Task 5 Step 1 `:120`) asserts/bumps `mk-cli-v0.4.1 → -v0.4.2` — a literal replace would silently no-op, leaving the GUI 2 versions stale. **Fix:** bump `mk-cli-v0.4.0 → mk-cli-v0.4.2` (catch-up) + correct the verified line. (Same bug class as v0.34.6's I3 — GUI pins lag sibling releases.)

## Minor
- **M1** — install.sh `md` pin carries the repo prefix `descriptor-mnemonic-md-cli-v0.6.0`; ms/mk are un-prefixed (`ms-cli-`/`mk-cli-`). Plan's sed targets are correct but non-uniform — note explicitly.
- **M2** — md-cli's `md-codec = "=0.35.0"` exact path-dep means md-codec 0.35.0 must be on crates.io before `cargo publish md-cli`. **RESOLVED at recon:** md-codec 0.35.0, ms-codec 0.2.0, mk-codec 0.3.1 ALL confirmed published on crates.io → all 3 sibling publishes resolve. Add the confirmation to the publish-train task.
- **M3** — unconditional `pub mod process_hardening;` in the toolkit lib is fine despite the GUI consuming the lib on Windows (the body is `#[cfg(target_os="linux")]`-gated → compiles to empty fn elsewhere; `set_non_dumpable()` signature is platform-independent). No action.

---

## Verification summary (positive)
1. **FFI** — `libc::prctl` (variadic), `PR_SET_DUMPABLE` (=4), `PR_GET_DUMPABLE` (return-valued) all exist in libc 0.2; the 2-arg/1-arg calls type-check; `unsafe` sound (no pointers). libc present in toolkit + ms-cli; absent in md-cli + mk-cli (plan adds correctly).
2. **PR_SET_DUMPABLE(0) semantics** — accurate, no overclaim: disables core dumps + restricts other-UID `/proc/$PID` access + blocks non-cap ptrace; does NOT hide cmdline from same UID (plan states this).
3. **Hook placement** — all 4 main() openings confirmed (toolkit :97, md-cli :223, ms-cli :101, mk-cli :49); first-statement-before-parse correct.
4. **Toolkit lib module** — `lib.rs` has `pub mod mlock;` precedent; main.rs already calls `mnemonic_toolkit::mlock::…` → the new `process_hardening` lib path works.
5. **Unit-test side effect** — CONFIRMED HARMLESS: no ptrace/`/proc/self`/process_vm_readv in any test across the 4 repos; no coverage tooling in CI; mlock FAIL_MODE harness doesn't depend on dumpable. Setting the test-binary non-dumpable breaks nothing.
6. **SemVer + lockstep** — all PATCH; no flag → `schema_mirror`/`secret_taxonomy_pin`/supply-chain snapshot unaffected.
7. **Release train** — current versions confirmed (md-cli 0.6.0, ms-cli 0.4.0, mk-cli 0.4.1, toolkit 0.34.6, GUI 0.19.2); install.sh pins all correct; GUI toolkit/md/ms pins correct; **GUI mk pin wrong (I1)**. Order siblings→toolkit→GUI correct.
8. **crates.io** — plan publishes only siblings (toolkit is tag+release only, crates.io-blocked on miniscript patch — correct). All 3 codec prereqs published (M2 resolved).

VERDICT: YELLOW (0C/1I)

Fold I1 (+ M1/M2 notes) → re-dispatch R1.

---

## Fold disposition (controller) — round 0 → R1
Folded: I1 (GUI mk pin v0.4.1→v0.4.0 actual; bump v0.4.0→v0.4.2 catch-up + verified-line fix); M1 (note non-uniform install.sh pin prefixes); M2 (add codec-published confirmation — md-codec 0.35.0 / ms-codec 0.2.0 / mk-codec 0.3.1 all on crates.io). M3 no action. Re-dispatching R1.

---

## R1 (round 1) — VERDICT: GREEN (0C/0I)
I1 fold VERIFIED: GUI `pinned-upstream.toml:52` mk pin is live `mk-cli-v0.4.0`; plan now bumps `v0.4.0 → v0.4.2` (catch-up) with R0-I1 annotation. M2 VERIFIED: md-codec 0.35.0 / ms-codec 0.2.0 / mk-codec 0.3.1 published on crates.io (dep specs `=0.35.0`/`=0.2.0`/`0.3.1` resolve). M1 VERIFIED (non-uniform install.sh prefixes noted). Core mechanics re-confirmed: module+test sound, hook placements (toolkit :97, md-cli :223, ms-cli :101, mk-cli :49), lib-module placement (`pub mod mlock;` precedent), libc status (add to md-cli+mk-cli; present in toolkit+ms-cli), all 5 version bumps match live Cargo.toml, release order intact. One new out-of-scope Minor (pre-existing stale comment in mk-cli Cargo.toml:21-24). **0C/0I gate satisfied — implementation may proceed.**
