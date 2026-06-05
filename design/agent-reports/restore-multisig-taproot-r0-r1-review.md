# R0 Architect Review (round 1) — SPEC_restore_multisig_taproot.md

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory pre-implementation R0 gate).
**Date:** 2026-06-05. **Branch:** `restore-multisig-taproot-reconstruction` (master `b9d6ea1`).
**Verdict:** **2 Critical / 2 Important — RED. Premise-invalidating; re-scope + re-R0 required (NOT a fold). Implementation MUST NOT proceed.**

> Persisted verbatim per CLAUDE.md. C1/C2 independently re-verified by the orchestrator against `template.rs:194-215,446` before accepting (the wrapper_node tr body hard-codes `is_nums: false, key_index: 0, indices:(0..n)`; comment `:203-208` says it SHOULD be NUMS, FOLLOWUP `toolkit-trmultia-nums-internal-key`; test `:446` locks it).

---

**Environment note:** reviewer had no Bash/exec — facts derived from source ground truth + the captured-runtime unit test `template.rs:455 tr_sortedmulti_a_2_of_2_round_trips_via_md_codec` (lossless split→reassemble, asserts `is_wallet_policy()` true, `is_nums:false`).

## CRITICAL

### C1 — The SPEC's central premise is false: `bundle` emits `is_nums: false` (cosigner-internal @0), NOT NUMS. The feature reconstructs ZERO real toolkit-emitted tr md1s.
`synthesize.rs:399` → `template.wrapper_node(k, n)` feeds the tree straight into `Descriptor` + `chunk::split` (`:405-450`) with no NUMS substitution. `wrapper_node` for `TrMultiA|TrSortedMultiA` hard-codes `Body::Tr { is_nums: false, key_index: 0 }` (`template.rs:209-210`) with leaf `indices:(0..n)` (`:215`) — internal key = cosigner @0, all N in the leaf. Locked by `template.rs:446` (`assert!(!is_nums, "… key_index=0 (real key), not NUMS sentinel")`) + open FOLLOWUP `toolkit-trmultia-nums-internal-key` (`:206`). `substitute_nums_sentinel` (the only `is_nums:true` setter) lives only in `parse_descriptor.rs` (`--descriptor` intake), never reached by bundle. **Consequence:** SPEC §3.1 reconstructs `is_nums:true` and REFUSES `is_nums:false` → every bundle-produced tr md1 (every test fixture) hits the refusal branch. The advertised capability fires on no toolkit card.

### C2 — Even with the match inverted, `build_descriptor_string` cannot reproduce the bundle md1's descriptor under ANY `TaprootInternalKey` → silent wrong-wallet.
Bundle's true descriptor is `tr(@0, sortedmulti_a(k, @0,@1,…,@n-1))` — @0 is internal key AND in the leaf (`template.rs:213-215`). `build_tr_multi_a_descriptor` (`pipeline.rs:110`): `Nums` → `tr(NUMS, sortedmulti_a(k, @0..@n-1))` (wrong internal key → different taproot output key → different `bc1p` addresses); `Cosigner(0)` → `tr(@0, sortedmulti_a(k, @1..@n-1))` (drops @0 from the leaf, `:144-149`). Neither matches. The §2 empirical claim is accurate *for `export-wallet --taproot-internal-key nums`* — but export-wallet emits a descriptor/wallet-file, NEVER an md1, so it's irrelevant to what `restore --md1` consumes. (Orchestrator's recon conflated the two: the runtime check used `export-wallet`'s NUMS output, not a decoded bundle md1.)

## IMPORTANT
- **I1 — citations target `restore.rs`; real path `crates/mnemonic-toolkit/src/cmd/restore.rs`.** Line numbers match (`:857` Tag::Tr gate ACCURATE; `build_multisig_import_payload` `taproot_internal_key:None` is `:696`, SPEC said ~690). Fix path in any descendant plan-doc.
- **I2 — §3.3's `build_multisig_import_payload` `taproot_internal_key:None` fix is correctly identified (`restore.rs:696`) but moot until C1/C2** (no correct `tap_ik` exists for a bundle tr md1).

## MINOR
- **M1** — `derive_receive_address` generalization of `derive_first_address` (`derive_address.rs:26`, `into_single_descriptors`+`derive_at_index`) is sound in isolation (miniscript v13 supports it for tr multipath; renders `bc1p`). Matters only once a correct tr descriptor exists.
- **M2** — §5 round-trip oracle is the right (non-tautological) shape but untestable until C1/C2; heed `[[feedback_recapture_golden_only_when_current_correct]]`.
- **M3** — SemVer v0.46.0 MINOR correct; no GUI change correct; manual scope lines `41-mnemonic.md:961-965` (+`:742-744`,`:767`) need the lockstep edit; `bundle` genuinely has no `--taproot-internal-key` (§2 ACCURATE).

## What verified clean (the half of the thesis that DOES hold)
- **Slot/threshold/policy plumbing is tree-shape-agnostic:** `expand_per_at_n` (`canonicalize.rs:420`, iterates `0..d.n` over `d.tlv.pubkeys`, never inspects `d.tree.tag`) → returns N keys for tr; `is_wallet_policy()` (`encode.rs:50`, `matches!(tlv.pubkeys, Some(v) if !v.is_empty())`) → true for tr; `extract_multisig_threshold` recurses `Body::Tr` (`bundle.rs:1021`). All ACCURATE — the defect is entirely the `is_nums` premise.
- Types/variants exist + reachable (`Body::Tr` pub, child `Node.tag` pub, `Tag::SortedMultiA`/`MultiA`, `CliTemplate::Tr*`). `to_miniscript_descriptor`/`template_from_descriptor` genuinely refuse `Tr` (skip-for-tr justified). `build_descriptor_string`+`build_tr_multi_a_descriptor` present + behave as described for NUMS.

## VERDICT: 2 Critical / 2 Important — RED.
The slot/threshold/`is_wallet_policy` reuse is sound, but the SPEC is built on a false premise: bundle emits `is_nums:false` (cosigner-internal @0), not NUMS. As written the feature refuses every real toolkit md1; and `build_descriptor_string` can't reconstruct the bundle md1's `tr(@0, sortedmulti_a(@0..@n-1))` shape under any `TaprootInternalKey`. **Dependency-order inversion:** the clean NUMS reuse only becomes correct AFTER FOLLOWUP `toolkit-trmultia-nums-internal-key` makes bundle emit `is_nums:true` — or this cycle must take on the cosigner-internal leaf shape it tried to defer. Re-scope + re-R0; do not implement.
