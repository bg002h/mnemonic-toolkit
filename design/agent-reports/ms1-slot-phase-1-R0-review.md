# ms1-slot — Phase 1 R0 Review
**Verdict:** GREEN (0C/0I)

Phase 1 diff `git diff fce28e6..3e3a6df` (5 files). Gate: 2636 passed / 0 failed, clippy clean.

## Critical (0) / Important (0) / Minor (2)

## Edit-by-edit verification
- **`slot_input.rs`:** `Ms1` declared `:37` immediately after `Entropy` (`:29`), before `Xpub` (`:38`) → Ord `Phrase<Seedqr<Entropy<Ms1<Xpub<MasterXpub<Fingerprint<Path<Wif<Xprv` (SPEC §1). `from_token` `:58` `"ms1"=>Ms1`; `as_str` `:73` `Ms1=>"ms1"`; `is_secret_bearing` `:85` `| Self::Ms1` (→ stdin sentinel free, test `:556`); error string `:172` lists `ms1` after `entropy`; macro `:406-417` includes `Ms1` (exhaustiveness tripwire enforces). `exempted_v0_19_0` `:299-307` adds `[Ms1,Path] | [Ms1,Fingerprint,Path]`; `is_legal_set` `:342-367` adds `[Ms1]` `:349`, `[Ms1,Fingerprint,Path]` `:364`, `[Ms1,Path]` `:365` — sort-order spelling correct (`Ms1<Fingerprint<Path`). Full parity w/ phrase.
- **`secret_taxonomy.rs:111`:** `SECRET_SLOT_SUBKEYS = &["phrase","seedqr","entropy","ms1","xprv","wif"]`. HARD parity gate (`slot_input.rs:420`) + round-trip (`:444`) pass iff matches `Ms1.is_secret_bearing()`. Correct.
- **`cmd/bundle.rs` canonical gate `:1151-1162`:** adds `has_seedqr`/`has_ms1`, widens to `(has_phrase || has_seedqr || has_ms1) && has_path`; existing `SlotInputViolation{kind:"conflict", message}` body preserved verbatim.
- **`cmd/bundle.rs` default-path-override `:1231-1234`:** continue-guard `!Phrase && !Seedqr && !Ms1`.
- **`cmd/verify_bundle.rs` default-path-override `:720-723`:** same `!Phrase && !Seedqr && !Ms1` — symmetric.
- **`tests/cli_ms1_slot.rs` (new):** sound (see test quality).

## Seedqr-normalization + no-weakened-test check
- Baseline ACCURATE: pre-fix canonical `[Seedqr,Path]` did not match `has_phrase && has_path`; default-path-override is skipped in canonical mode (`if is_non_canonical`, `bundle.rs:1174`); reached descriptor binding loop (`:1309-1419`, NO Seedqr arm) → `else→BadInput` (`:1412-1418`, exit 1). Widened gate → exit-2 `SlotInputViolation{kind:"conflict"}`.
- NO pre-existing test asserted the old behavior. Grepped all four seedqr test files + `cli_non_canonical_descriptor.rs`: `cli_bundle_seedqr_slot.rs` exit-1 assertions (`:103,:129,:186`) are all template-mode decode/stdin errors, none `[Seedqr,Path]`+descriptor; `cli_non_canonical_descriptor.rs:283` uses **phrase** (exit-2 unchanged) + `.failure()` (no specific code). No seedqr test was modified/weakened/deleted (only `cli_ms1_slot.rs` added). Safe.

## Intermediate-state + no-overreach check
- No over-reach: diff is exactly the 5 files. No `slot_ms1.rs` (Glob: none); no `mod slot_ms1`/`resolve_ms1_slot` (the lone `slot_ms1` grep hit is a pre-existing `synthesize.rs:1448` test-fn name). `resolve_slots` (`:486-718`) has NO Ms1 arm; only the gate (`:1153`) + default-path guard (`:1233`) in bundle, default-path guard (`:722`) in verify.
- Intermediate states CLEAN (no panic): template bare `@0.ms1=` → resolve_slots catch-all `else` (`:709-714`) → `BadInput` exit 1; `[Ms1,Path]` non-canonical descriptor → default-path routes, binding loop `else→BadInput` exit 1; `[Ms1,Path]` canonical → new gate exit-2 conflict. All terminate in typed `ToolkitError`.

## Test quality
`cli_ms1_slot.rs` non-vacuous: asserts `.code(2)` (SlotInputViolation→exit 2, `error.rs:519`) + verbatim conflict substring `"has both secret-bearing input and watch-only input; pick one per slot."` (load-bearing, `bundle.rs:1159`). `CANONICAL_DESC="wsh(sortedmulti(2,@0,@1))"` genuinely canonical (`canonical_origin` Some → `is_non_canonical=false` → gate fires; cross-confirmed vs `cli_non_canonical_descriptor.rs:284`). Supplies well-formed `@1.xpub=<valid xpub>`; gate precedes decode so the ms1 stub value need not decode. seedqr test carries the R0-I2 baseline note + asserts post-fix exit-2.

## Minor (non-blocking)
- **M1 (cosmetic):** `cli_ms1_slot.rs:25` provenance comment for VALID_XPUB. No action.
- **M2 (pre-existing, Phase-3 opportunistic):** `is_legal_set` doc-comment (`slot_input.rs:332-341`) still lists Ord as `Phrase<Entropy<Xpub<…` (omits Seedqr AND Ms1) — pre-existing staleness; fold opportunistically, not a Phase-1 defect.

## Verdict rationale
Every edit matches SPEC/plan; Ord/legal-set/exemption spellings correct; fix-the-class gate widening applied identically in the bundle gate + both default-path loops with the conflict body preserved; seedqr normalization intended + safe (no weakened test); diff exactly 5 files, no over-reach; all intermediate paths clean typed errors; tests load-bearing against a genuinely-canonical descriptor. **GREEN (0C/0I) — clear to proceed to Phase 2.**
