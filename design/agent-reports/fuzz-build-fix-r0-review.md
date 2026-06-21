> Reviewer: opus architect (Claude Opus 4.8, 1M context) — per-change R0 EXECUTION review, source-verified + gates run locally.
> Subject: branch `fix/fuzz-build-unrestorable-advisory-cmd-ref`, single commit `9d5b69458a49384883224940ac9c4ba939a03be7` ("fix(fuzz): move taproot-override predicates to a shared leaf module").
> Repo: `/scratch/code/shibboleth/mnemonic-toolkit` @ HEAD `9d5b6945`. Toolchains: stable rustc 1.85.0, nightly rustc 1.97.0-nightly, cargo-fuzz 0.13.2.
> Date: 2026-06-20.

**Verdict: GREEN — 0 Critical, 0 Important.** Ship NO-BUMP (ff master → push, no tag).

---

## 1. Relocation is byte-identical (no behavior change) — CONFIRMED

Extracted the OLD bodies from the parent (`git show 9d5b6945^:.../cmd/restore.rs`) and the NEW bodies from `crates/mnemonic-toolkit/src/taproot_override_classify.rs`, then `diff`'d and `sha256sum`'d each:

- `taproot_override_card`: `diff` empty → IDENTICAL. sha256 `c63ce53d…91d44` matches on both sides.
- `restorable_taproot_override_card`: `diff` empty → IDENTICAL. sha256 `09724657…d356c` matches on both sides.

Logic verified character-for-character:
- `taproot_override_card` (`taproot_override_classify.rs:32-34`): `matches!(d.tree.tag, md_codec::Tag::Tr) && d.tlv.use_site_path_overrides.is_some()` — unchanged.
- `restorable_taproot_override_card` (`taproot_override_classify.rs:56-74`): same `use md_codec::tree::Body;`, same `if !taproot_override_card(d) { return false; }` short-circuit, same `if md_codec::to_miniscript::has_hardened_use_site(d) { return false; }` guard, same `Body::Tr { is_nums: true, tree: Some(inner), .. } => inner.tag == md_codec::Tag::MultiA` arm, same `_ => false` fallthrough. No conjunct dropped, added, or reordered; the four-conjunct doc block (NUMS internal / plain `MultiA` not `SortedMultiA` / no hardened use-site) moved verbatim with it.

This is a pure relocation. The funds-safety-critical expression `taproot_override_card && !restorable_taproot_override_card` is structurally untouched.

## 2. All call sites resolve the SAME predicates (single-source parity preserved) — CONFIRMED

- `cmd/restore.rs:2613-2616` re-exports `pub(crate) use crate::taproot_override_classify::{restorable_taproot_override_card, taproot_override_card};`, preserving the old `cmd::restore::…` path and `pub(crate)` visibility.
- Restore guard — `cmd/restore.rs:2786`: `if taproot_override_card(&d) && !restorable_taproot_override_card(&d)` (bare names → bind to re-export).
- Classify-reroute — `cmd/restore.rs:2815`: `if is_taproot && restorable_taproot_override_card(&d)` (the P2.2 `Template`-arm reroute at the sole classify caller; bare name → re-export).
- Truth-table tests — `cmd/restore.rs` `mod taproot_override_predicate_tests` with `use super::*` (lines 3436–3486) call both bare names → re-export. Ran them in isolation: all 5 pass (`restorable_nums_multi_a_override_is_true`, `hardened_override_is_not_restorable`, `non_nums_trunk_override_is_not_restorable`, `sortedmulti_a_override_is_not_restorable`, `non_override_taproot_is_not_restorable`), exit 0 — confirming the re-export wiring is live and the full truth table is unchanged.
- Engrave advisory — `unrestorable_advisory.rs:116-117`: now `crate::taproot_override_classify::taproot_override_card(desc) && !crate::taproot_override_classify::restorable_taproot_override_card(desc)`.

Grep of all of `src/` for any remaining `cmd::restore::taproot_override_card` / `cmd::restore::restorable_taproot_override_card` qualified reference: **NONE**. Exactly ONE definition of each predicate exists (`taproot_override_classify.rs:32` and `:56`); no duplicate/second copy anywhere. All three load-bearing call sites (guard / classify-reroute / advisory) plus the tests resolve the one source — refuse ⟺ advise parity is structurally guaranteed by sharing a single module.

## 3. Fix is COMPLETE (sole fuzzing-closure → cmd reference) — CONFIRMED

Scanned every `#[cfg(fuzzing)]`-mounted module in `lib.rs:147-190` (cost, derive, derive_address, derive_slot, error, format, friendly, indel, language, network, parse, parse_descriptor, repair, secret_advisory, slip0132, slot_input, synthesize, taproot_override_classify, template, timelock_advisory, unrestorable_advisory, wallet_export — incl. the `cost/` and `wallet_export/` directory modules) for `crate::cmd::`:

- Parent commit `9d5b6945^`: `unrestorable_advisory.rs` was the ONLY module with `crate::cmd::` references in code (2 hits = the two predicate calls). No other fuzzing-mounted module referenced `cmd`.
- HEAD: the only `crate::cmd::` strings remaining among the mounted set are doc comments — `taproot_override_classify.rs:12` (`//! crate::cmd::restore::… under cfg(fuzzing)` describing the OLD break) and `wallet_export/mod.rs:158` (`/// than crate::cmd::convert::ScriptType`). Neither is a code path; neither resolves a `cmd` item at compile time.

So this one relocation fully closes the E0433 — there is no second latent fuzzing-closure `cmd` reference.

## 4. Gates (run locally) — ALL GREEN

- **`cargo +nightly fuzz build` (THE gate, was E0433):**
  ```
  warning: `mnemonic-toolkit` (lib) generated 54 warnings ...
      Finished `release` profile [optimized + debuginfo] target(s) in 0.08s
  fuzz build (cached re-run) exit status: 0
  ```
  GREEN. (`cargo +nightly fuzz list` → one target `descriptor_parse`; warnings are pre-existing dead-code from the fuzzing mount exposing bin-private items — not errors.)

- **`RUSTFLAGS="--cfg fuzzing" cargo build -p mnemonic-toolkit --lib` (fast proxy):**
  ```
      Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.22s
  cfg-fuzzing lib build exit status: 0
  ```
  GREEN.

- **`cargo test -p mnemonic-toolkit` (full suite):** every test group `test result: ok. … 0 failed`; aggregate exit status 0. Targeted `--bin mnemonic taproot_override_predicate` → `5 passed; 0 failed` (the parity truth table). GREEN.

- **`cargo clippy -p mnemonic-toolkit --tests --bins --lib -- -D warnings`:**
  ```
      Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s
  clippy exit status: 0
  ```
  Clean under `-D warnings`. GREEN.

- **`cargo build -p mnemonic-toolkit --bins` (normal path — proves the main.rs mount + re-export resolve outside fuzzing):** exit 0. GREEN.

- **Changed-file set / g6:** `git diff --name-only 9d5b6945^..9d5b6945` = exactly `cmd/restore.rs`, `lib.rs`, `main.rs`, `taproot_override_classify.rs` (new), `unrestorable_advisory.rs`, `design/FOLLOWUPS.md`. **No `mlock.rs`** touched — g6 fmt-exemption respected. Only the new module + 4 edited source files + the FOLLOWUP doc.

## 5. Mount correctness + crate-shape consistency — CONFIRMED

- `main.rs:33`: `mod taproot_override_classify;` (bin-private, normal builds).
- `lib.rs:181-182`: `#[cfg(fuzzing)] pub mod taproot_override_classify;` — so `unrestorable_advisory` (also `#[cfg(fuzzing)]`-mounted in the lib, `lib.rs:188`) resolves `crate::taproot_override_classify::…` inside the lib-under-fuzzing crate. `cmd` confirmed NOT mounted in `lib.rs` (the root cause); `main.rs:6` `mod cmd;` is bin-only.
- New module deps: the only `use` is the inline `use md_codec::tree::Body;` (`:57`); no toolkit-internal `crate::` code references (sole `crate::` is the line-12 doc comment). It compiles cleanly in BOTH the bin crate and the lib-under-fuzzing crate, as the two builds above prove.
- No `extern crate self`, no `crate as` self-alias, no `self::` path games — clean 74-line leaf module.

## 6. NO-BUMP correctness — CONFIRMED

No user-facing surface change: no flag, subcommand, dropdown-value, `--json` wire-shape, exit-code, or behavior change. The predicate logic is byte-identical (§1) and the suite/truth-table confirm runtime behavior is unchanged (§2,§4). Purely an internal module move + fuzz-build repair → NO-BUMP is correct. No CLI surface touched ⇒ no manual / `gui-schema` / `schema_mirror` lockstep obligation triggered. FOLLOWUP `fuzz-build-broken-unrestorable-advisory-references-bin-only-cmd` flipped to `✓ RESOLVED 2026-06-20 (NO-BUMP)` in this same commit (`FOLLOWUPS.md:53,55`) — status discipline satisfied.

---

## Findings

None. 0 Critical, 0 Important, 0 Minor. The predicate logic did not change (sha256-identical), every call site binds the one re-exported source, no other fuzzing-closure `cmd` code reference remains, all gates pass, mounts are correct in both crate shapes, and the FOLLOWUP is flipped. **GREEN — ship NO-BUMP (fast-forward master → push, no tag).**
