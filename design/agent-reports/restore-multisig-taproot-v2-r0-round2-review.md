# R0 Review — v2 SPEC round 2 (fold verification / convergence) — `SPEC_restore_multisig_taproot.md`

> Persisted verbatim from the opus-architect R0 round-2 dispatch. Converged GREEN.

---

This closes the gap definitively. The grep for `d.` calls in the entire `run_multisig` emission tail (`:835` through the function end at `:1190`) returns **empty** — meaning `d.derive_address` at `:840-842` is the **only** call on the decoded md1 tree `d` in the emission path. Everything else (`first_recv` text/JSON consumers, the `--format` payload) works purely from the reconstructed `descriptor` string + `slots` + `k`.

And `build_multisig_import_payload` (`:639-678`) confirms: it builds `EmitInputs` from `descriptor: &str` (via `CheckedDescriptor::new(descriptor)`) + `slots` + `template` + `k` — it **never touches `d`**. `emit_payload` receives only `&inputs`. So `--format` does not re-enter `to_miniscript_descriptor`; it only needs the correct `taproot_internal_key` (currently hardcoded `None` at `:662`, exactly as I2 states). The `--json` shape consumes `first_recv` (already-rendered strings) at `:1075`. No second re-entry anywhere.

C1(new) is fully closed: replacing the single `d.derive_address` site with the descriptor-string path is sufficient. Both the `--format` and `--json` tails are clean.

All verifications complete. Final verdict.

---

## VERDICT: GREEN — 0 Critical / 0 Important / 0 Minor. Implementation may begin.

The author's fold of all six round-1 findings (C1(new) + I1 + I2 + M1 + M2) is **complete, correct, and source-accurate at `5a22552`** / miniscript `95fdd1c` / md-codec `0.35.0`. No new defect introduced. The two v1 Criticals remain dissolved and undisturbed. Convergence confirmed.

## C1(new) fold — COMPLETE and TECHNICALLY SOUND
The SPEC now mandates the descriptor-string address path and forbids `d.derive_address` for the tr branch (§1 step 7, §4, §5):
- **Path mandated:** step 7 routes the first address through `MsDescriptor::<DescriptorPublicKey>::from_str(&descriptor) → into_single_descriptors() → derive_at_index(0) → .address(network)`, naming `d.derive_address` (`restore.rs:840-842`) as MUST-NOT-use, with the correct causal chain (`d.derive_address` → md-codec `derive.rs:120` `to_miniscript_descriptor` → SortedMultiA error). Verified.
- **Test added:** §5 "Address derivation (C1(new) regression — load-bearing)" exercises the full `restore --md1` CLI on a `tr-sortedmulti-a` md1 end-to-end to `bc1p…`, RED against a naive `d.derive_address` impl.
- **Sequence sound at `95fdd1c`:** miniscript parses `sortedmulti_a` via `from_str`, has `Terminal::SortedMultiA` (`decode.rs:161`) + Display (`astelem.rs:172`); `from_str → into_single_descriptors → derive_at_index(0) → .address` renders `bc1p`. `derive_first_address` `:24-25` "reject tr" caveat is doc-only (body `:34-66` has no tr-rejection logic) — reuse safe.

**No other md-codec re-entry on the tr branch (verified at source):** `grep '\bd\.'` over `run_multisig` emission tail (`:835`→`:1190`) returns ONLY `d.derive_address` at `:840-842`. `--format` (`build_multisig_import_payload` `:639-678`) builds `EmitInputs` from `descriptor:&str` + `slots` + `k`, never touches `d`; `emit_payload` gets only `&inputs`. `--json` (`:1075`) consumes already-rendered `first_recv`. Replacing the one site fully closes C1(new).

## I1 / I2 / M1 / M2 — each landed accurately
- **I1**: false bip86 analogy removed; single-sig uses `render_address_from_xpub` (`:362`) — verified.
- **I2**: `:662` (verified) / call site `:1034` / `:636` doc / single-sig `None` `:606` left; mandates the `taproot_internal_key` param on `build_multisig_import_payload`; stale `:696`=`xpub_from_65_bytes` flagged. Accurate.
- **M1**: honest `is_nums:false` framing (reconstructable, deliberately deferred); basis `to_miniscript.rs:161-165` verified. Refuse-disposition stays safe.
- **M2**: `bundle.rs:1036` (recurse `:1042`); "(was mis-cited `:1021`)". Verified.

## v1 Criticals — remain dissolved, undisturbed
- C1: `template.rs:213 is_nums:true` (`:194` arm), `:450 assert!(is_nums…)` — untouched.
- C2: `pipeline.rs:128` Nums arm keeps all cosigners in leaf (mirrors `template.rs:219`); §3 oracle non-tautological — untouched.

## Disposition — correct
PATCH; no GUI `schema_mirror`; watch-only-out; release-gate §7 complete (dual README markers + install.sh self-pin + readme_version_current + install-pin-check, re-run after bump); `lint_argv_secret_flags` N/A; descriptor-mnemonic companion + FOLLOWUP-collapse carried.

## Non-blocking editorial note (no action)
md-codec SortedMultiA error arm is `to_miniscript.rs:407-411`; SPEC's `:406-410` is off-by-one, immaterial.

**GREEN — converged. Implementation can begin.**
