# R0 Review (round 2) — IMPLEMENTATION_PLAN_verify_bundle_nonchunked_canonicalization.md

**Reviewer:** Fable architect (`model:"fable"`), 2026-07-12. Focused delta-only pass on the round-1 folds. **Source SHA:** `de140a08`. **Usage:** 26 tool-uses, ~489s, 97999 tokens.
**Main-loop verification:** I-A both claims confirmed against source — keyed md1 without `--template` → ModeViolation (verify_bundle.rs:435-443); general path prints lowercase `result: ok` (:558-567); existing keyed test asserts `contains("result: ok")` with `--template bip84` (cli_verify_bundle_full.rs:30-56).

## Verdict: RED — 0 Critical / 1 Important (I-A) + 1 Minor (M-A)
All 4 Important + 3 Minor round-1 folds landed; 6/7 correct against source. The I-1 re-fixture introduced one new defect.

## Important
### I-A — Task 1 Step 4 `verify_bundle_keyed_multichunk_unchanged` is RED on master (violates its own "GREEN before AND after" requirement)
Two failures in the one cell:
1. **Omits `--template`.** A KEYED wallet-policy md1 reassembles at the classify gate but skips both template branches (is_wallet_policy true, verify_bundle.rs:389-406); `descriptor_mode` false → hits verify_bundle.rs:435-443 `if args.template.is_none() → ModeViolation "--template is required …"`. Source comment :430-434 confirms "any md1 reaching here is a keyed wallet-policy md1 … that DOES need an explicit `--template`." → `.assert().success()` panics. Both keyed-verify precedents pass `--template` (cli_verify_bundle_full.rs:37-38, :119-122).
2. **`stdout.contains("OK")` never matches the general path.** "OK (single-sig template recomposed)" is template-path-only (:824, :1030). The general/keyed path prints per-check lines + lowercase `result: ok` (:558-567); the existing keyed test asserts `contains("result: ok")` (cli_verify_bundle_full.rs:56).
**Fix:** add `"--template".into(), "bip84".into()` to the verify args; assert `stdout.contains("result: ok")`. Rest of the cell sound: default md1-form IS keyed policy (bundle.rs:169-170 `default_value_t = Md1Form::Policy`); `md1.len() > 1` structurally guaranteed (65-byte pubkey = 520 bits > 400-bit cap; frozen wsh(pk) card is 3 chunks).

## Minor
### M-A — stale line-cite: `verify_bundle_canonical_multisig_template_id_search_ok` spans `:293-361` (plan says `:293-321`). Harmless; update on fold.

## Fold-by-fold confirmation (all others CORRECT)
- **I-1** (i) keyed default confirmed (bundle.rs:169-170); (ii) multi-chunk guaranteed (tlv.rs:32, codex32.rs:25); (iii) → I-A (keyed verifies "result: ok" on master, but only with `--template`).
- **I-2** all helpers exist with matching signatures in cli_verify_bundle_md1_template_multisig.rs (SEED_A :33, SEED_B :34, emit_template_md1 :154, emit_template_mk1_stubs :163, emit_template_wallet_id :175, emit_cosigner_mk1 :188, push_md1 :244, push_mk1_stubs :251, push_cosigners :260, verify_json :270); keyless 2-of-2 < 400 bits; no-from floor message has "--from"+"seed" on STDERR (:876-883, :484-503); RED-on-master confirmed (fall-through ModeViolation text names "--template", not "--from"/"seed").
- **I-3** faithfully mirrors :293-361; GREEN after Facet 1 alone (verify_multisig_template :860-1037 never touches raw args.md1; WDT-id compare :937-941; Facet 2 touches only :696).
- **I-4** repo-root fuzz/Cargo.lock:578-579 pins 0.89.0; plan uses `cd fuzz` + `git add fuzz/Cargo.lock`.
- **M-1** compares stdout+stderr+json; sound (all check details static, envelope form-independent :799-828).
- **M-2** post-impl report committed before tag (Task 3 Step 7).
- **M-3** `.examples-build/` confirm added (Task 3 Step 3).

## New-defect scan (beyond I-A) — CLEAN
- keyed_cards stdout parsing matches bundle headers (`# ms1`/`# mk1`/`# md1`, bundle.rs:1018-1048); engraving card is stderr-only → no stdout pollution; parser byte-identical to proven template_cards.
- to_nonchunked defined once per test crate (multisig Step 1, single-sig Step 4); grep confirms no current definition/collision.
- SS_MD1_ORIGIN verbatim; md-codec re-exports all present; PHRASE_A :7; verify_args sig matches; mode "single-sig-template" matches :801.

(Full proof-of-work table in transcript.)
