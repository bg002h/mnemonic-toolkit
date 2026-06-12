# PLAN — minor coverage cluster: md-codec goldens + verify-bundle cells (GAP 5)

**Date:** 2026-06-12 · **SemVer:** NO-BUMP both repos (test-only; PATCH only if a golden reveals a mis-render)
**Source SHAs:** descriptor-mnemonic `origin/main` = `96aaab3` (after Cycle 1); mnemonic-toolkit `origin/master` = `6c27585` (after Cycle 3). **Recon:** `cycle-prep-recon-minor-coverage-gaps.md`. **FOLLOWUP:** `verify-bundle-bip388-policy-intake` already filed (hygiene pass).

## Part 5a — md-codec golden cells (descriptor-mnemonic, `tests/proptest_to_miniscript.rs`)

All follow the `self_test_*` house pattern (build tree via `common/mod.rs` constructors → `descriptor_with_pubkeys(tree)` → `addr = p6_chain(&d)` → `assert_eq!(addr, "bc1…")`). Derive-once-then-pin: write with a placeholder, run, copy the real address from the failure, pin it. `p6_chain`'s reparse fixed-point is the mis-render oracle (a render bug → it panics → escalate to a PATCH, not expected).

1. **Item 1 — `multi` n∈17..=20 (the only VALID window with zero render/address coverage).** `self_test_wsh_multi_17_of_20`: `descriptor_with_pubkeys(wrap(Tag::Wsh, multikeys(Tag::Multi, 17, (0..20).collect())))` → golden `bc1q…`. wsh-ONLY (legacy `sh` caps multi at 15 keys; 17–20 is valid only under wsh). The 32-key pool (`test_xpubs()`) + `descriptor_with_pubkeys`'s `1..=32` acceptance make this need zero new key material. **Also fix the stale comment** at `common/mod.rs:~886` ("the TLV-attached key pool caps n at 16" → the pool is 32; the cap is the deliberate T-tier key-budget, not an infra limit).
2. **Item 2 — `after` positive golden.** `self_test_wsh_and_v_pk_after_800000`: mirror `self_test_wsh_and_v_pk_older_144` (`:136`) with `timelock(Tag::After, 800000)`. Oracle-independent anchor for `after` rendering (the differential moves both sides together; the golden catches an upstream `after`-Display shift).
3. **Item 3 — hash256 / ripemd160 / hash160 goldens.** 3 cells mirroring `self_test_tr_nums_and_v_sha256_pk` (`:173`) byte-for-byte: `tr_node(true, 0, Some(node2(Tag::AndV, wrap(Tag::Verify, hash32(Tag::Hash256,[..32]) | hash20(Tag::Ripemd160,[..20]) | hash20(Tag::Hash160,[..20])), keyarg(Tag::PkK, 0))))` → golden `bc1p…`. (sha256 already has its golden; this completes the four.)

## Part 5b — toolkit verify-bundle cells (mnemonic-toolkit, `tests/cli_verify_bundle_*.rs`)

The recon CORRECTED the premise: timelock general policies ARE already verify-bundle-tested (`cli_verify_bundle_multi_cosigner_mk1.rs:248/:372/:438`, `cli_verify_bundle_entropy_slot.rs:16`) → **drop the timelock sub-goal.** Real residue:

4. **Hashlock verify-bundle round-trip cell** (genuinely absent — zero `sha256|hash256|ripemd|hash160` hits in `tests/cli_verify_bundle*.rs`). A `wsh(and_v(v:sha256(H),pk(…)))`-class bundle→verify-bundle round-trip mirroring the existing `non_canonical_*_round_trips_via_bundle_json` cell. Toolkit `parse_descriptor.rs:639-650` has all four hashlock arms → not blocked. ~40 LOC. **Exact shape + invocation fixed at impl via a probe** (the verify-bundle round-trip needs the cosigner phrase(s); follow the existing cell's bundle→verify-bundle JSON pattern).
5. **BIP-388 verify-bundle pinned-REFUSAL cell** (item 4(a), NOT the feature). `verify_bundle.rs` has no `is_bip388_policy_shape` probe — a leading-`{` wallet-policy JSON fed to `verify-bundle --descriptor` fails as a descriptor parse error TODAY. Pin that current behavior (assert it refuses, with the actual error) so the asymmetry-vs-bundle/export-wallet is documented + a future fix flips it red-then-green. The feature (mirror bundle.rs probe→expand into verify_bundle intake) stays the FOLLOWUP `verify-bundle-bip388-policy-intake` (already filed). **Exact error message pinned at impl via a probe.**

## Verification
- 5a: `cargo test -p md-codec --test proptest_to_miniscript -- self_test_` + full suite + clippy + fmt (descriptor-mnemonic fmt is repo-wide-clean; run `cargo fmt --all`).
- 5b: `cargo test -p mnemonic-toolkit --test cli_verify_bundle_*` + clippy + fmt (toolkit: format ONLY the touched file via `rustfmt`, NEVER `cargo fmt --all` — the mlock.rs exemption).
- Probe the 5b shapes/messages with the built `mnemonic` binary before pinning (like Cycles 2–3).

## Lockstep / SemVer
- NO-BUMP both repos (test + comment only). No clap surface → no manual/GUI/schema_mirror. No wire/API change. md-codec and toolkit are INDEPENDENT commits (5a → descriptor-mnemonic; 5b → mnemonic-toolkit); no pin/lockstep between them.
- 5a escalates to a md-codec PATCH ONLY if a golden bring-up reveals a render bug (not expected — all four shapes are P6-property-covered already).

## R0 questions
1. 5a: one n=17-of-20 cell (upper boundary) sufficient, or also an n=17 cap+1 edge cell? (Lean: the single n=17/20… actually k=17,n=20 covers the window top; an n=17 cell is marginal. Lean one cell, R0 to confirm.)
2. 5b item 5: pin the BIP-388 REFUSAL (current behavior, test-only) vs implement the expand-probe feature now? (Lean: pin the refusal this cycle, feature → the filed FOLLOWUP — keeps GAP-5 NO-BUMP test-only.)
3. 5b item 4: is the hashlock verify-bundle cell single-sig-keyed (`wsh(and_v(v:sha256(H),pk(seed-key)))` watch... no, verify-bundle needs the seed to re-derive) — confirm the right invocation (bundle from a phrase → verify-bundle with that phrase) at impl probe.
