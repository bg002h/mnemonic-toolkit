# C2 (verify-bundle BIP-388 intake) impl-review round 1 — code reviewer (verbatim)

> Reviewer: opus code reviewer (general-purpose, full tools). Diff: branch
> `feature/verify-bundle-bip388-intake` vs `master` @ `a69a9e3`. Verdict RED (1C/1I);
> both findings folded post-review (see footer).

---

**Verdict: RED — 1 Critical, 1 Important**

The implementation logic, the 6-line insertion, and the 3 test cells are all correct and genuinely non-vacuous (I empirically confirmed all 3 go RED on revert — cells 2a/2b on exit code, cell 2c on the message discriminator, exactly as planned). The behavior change is a faithful mirror of `bundle.rs:319`. The defect is **lockstep incompleteness**: two release-ritual version sites were missed, and one of them makes the test suite RED right now — so the prompt's premise ("full test suite already passed, exit 0") is stale.

---

## Critical

**C1 — `crates/mnemonic-toolkit/README.md:9` left at `0.56.0`; the `readme_version_current` guard test is RED. The suite does NOT pass.**

- Evidence: There are **two** `<!-- toolkit-version: -->` markers, not one. The plan-doc (line 139) and both R0 reviews assert "exactly ONE README marker (`README.md:13`)" — this is factually wrong. The crate-level README at `crates/mnemonic-toolkit/README.md:9` still reads `<!-- toolkit-version: 0.56.0 -->`.
- The guard `crates/mnemonic-toolkit/tests/readme_version_current.rs::both_readmes_carry_current_version_marker` iterates `["README.md", "../../README.md"]` (line 27) and asserts each contains `<!-- toolkit-version: {CARGO_PKG_VERSION} -->`. With `CARGO_PKG_VERSION = 0.57.0` and the crate README at 0.56.0, I ran it: **FAILED** —
  > `crates/mnemonic-toolkit/README.md must carry `<!-- toolkit-version: 0.57.0 -->``
- This is part of the release ritual: the prior release commit `a674879` bumped this exact marker `0.55.3 → 0.56.0` in lockstep with the root README. The plan's "do not hunt for a second [marker]" instruction (line 146) sent the implementer past it.
- Consequence: `cargo test -p mnemonic-toolkit` is RED on this branch. The "full suite passed (exit 0)" claim was a stale run (pre-Cargo.toml-bump, or this test was filtered out).
- Fix: bump `crates/mnemonic-toolkit/README.md:9` → `<!-- toolkit-version: 0.57.0 -->`, re-run `cargo test -p mnemonic-toolkit --test readme_version_current` (and the full suite) to confirm GREEN before tag.

## Important

**I1 — `fuzz/Cargo.lock:575` left at `version = "0.56.0"` for the `mnemonic-toolkit` package; main `Cargo.lock` was bumped but the fuzz workspace lockfile was not.**

- Evidence: `fuzz/Cargo.lock` lines 573-575 hold the `[[package]] name = "mnemonic-toolkit" version = "0.56.0"` entry. The prior release `a674879` bumped this file `0.55.3 → 0.56.0` (`git show a674879 -- fuzz/Cargo.lock` shows the `-version = "0.55.3"` / `+version = "0.56.0"` hunk). MEMORY explicitly flags the "`cfg(fuzzing)` dual-home locksteps" + "re-run suite+fuzz after bump, before tag" (`project_older_timelock_advisory_v0_55_2`).
- Not gated by a Rust unit test (separate cargo workspace), so it's **silent drift** — exactly the class of miss the prompt asked me to catch beyond the green bar. Left unbumped, the next fuzz build re-stamps it and the discrepancy lingers in history.
- Fix: `cargo update -p mnemonic-toolkit --manifest-path fuzz/Cargo.toml --precise 0.57.0` (or rebuild the fuzz lockfile) so `fuzz/Cargo.lock` reads `0.57.0`, mirroring the prior-release pattern.

## Minor

**M1 — Plan-doc line 139 + both persisted R0 reviews state "exactly ONE README marker," which is false (two markers, both guard-enforced).** Not a code defect, but the R0 architect repeated the plan's claim without independently grepping `toolkit-version:` across the tree — the root cause of C1. Worth a one-line correction in the plan-doc / a note in the impl-review report so the next cycle's "version-marker lockstep" checklist lists **both** READMEs (the guard already enforces both; the prose checklist should too).

**M2 — Behavior/code quality: all clean, recorded for the audit trail.**
- The insertion (`verify_bundle.rs:687-698`) is a faithful mirror of `bundle.rs:319-323`: same `is_bip388_policy_shape` → `expand_bip388_policy(&…)?` shape, shadowing `descriptor_str` so all 5 downstream consumers (classify :707, body_no_csum :708, lex :730, canonicity :734, parse :1042) see the expanded form. Expanded `@N`-free descriptor routes through the Concrete fork (:707) and returns at :717, never reaching the `@N` machinery — no double-expansion, no interaction with the @N-template path.
- Dropping policy-name (mirroring `bundle.rs`, not `export_wallet.rs:435` `bip388_policy_name`) is correct: verify-bundle has no wallet-name surface, so name-preservation would be dead code.
- Error propagation via `?` surfaces the expander's pre-existing `BadInput` (exit 1, malformed JSON) and `DescriptorParse` (exit 2, `@N` beyond keys_info) cleanly — no new `ToolkitError` variant, so the alphabetical-ordering convention is correctly N/A.
- The expanded descriptor carries no `#checksum`; `descriptor_body_no_csum` (:708) tolerates that, proven by the cell 2a/2b round-trips passing E2E.
- fmt gate respected: `cargo fmt -p mnemonic-toolkit -- --check` shows a diff **only** in `mlock.rs` (the permanent g6 exemption) — `verify_bundle.rs` and the test file are clean. Clippy clean. Manual lint `[lint] OK`. No `git add -A` overreach (9 files, all intentional).

**M3 — Test non-vacuity empirically verified (the green bar's blind spot the prompt asked about).** I reverted the 6-line insertion and re-ran: `verify_bundle_accepts_bip388_policy_json` and `verify_bundle_accepts_bip388_singlesig_policy_json` both fail at exit 2 ("mixes @N placeholders with inline keys"); `verify_bundle_refuses_bip388_policy_at_n_beyond_keys_info` fails because the stderr message reverts from `"@N beyond keys_info"` to `"mixes @N…"` — confirming the M1-discriminator design holds (cell 3 keys on the message, not a bare `.code(2)`). Only the pre-existing hashlock cell stays GREEN. All three positives assert `result: ok` (not a bare exit code). No test passes for the wrong reason.

---

**Summary:** The feature itself is correct, complete, and well-tested — I'd green-light the code unreservedly. The gate fails purely on release-ritual lockstep: the crate-level README marker (C1, currently RED in the suite) and the fuzz lockfile (I1, silent drift) were both missed, both bumped in every prior release, and both traceable to the plan's incorrect "exactly ONE README marker" premise. Bump those two files, re-run `cargo test -p mnemonic-toolkit` to confirm GREEN, and this converges to 0C/0I.

---

## FOLD (post-review, by implementer)

- **C1 fixed:** `crates/mnemonic-toolkit/README.md:9` bumped `0.56.0 → 0.57.0`. `cargo test -p mnemonic-toolkit --test readme_version_current` → GREEN (`both_readmes_carry_current_version_marker ... ok`). The earlier "full suite exit 0" was indeed a pre-bump run at 0.56.0.
- **I1 fixed:** `cargo update -p mnemonic-toolkit --precise 0.57.0` in `fuzz/` → `fuzz/Cargo.lock:575` now `version = "0.57.0"`. `RUSTFLAGS="--cfg fuzzing" cargo build` in `fuzz/` → exit 0 (the plain `cargo build` E0433 is the expected `cfg(fuzzing)` gating, not a regression).
- **M1 fixed:** plan-doc lockstep section + this report corrected — there are **TWO** guard-enforced README markers (`README.md:13` + `crates/mnemonic-toolkit/README.md:9`); the next cycle's checklist must list both.
