# End-of-cycle R0 review — `path-raw-bracketed-vs-bare-convention-unification`

**Branch `path-raw-unification`, base `dd7c228`. Opus feature-dev:code-reviewer. Persisted verbatim.**

Reviewed the 3-commit refactor deleting `ResolvedSlot.path_raw` and deriving the origin annotation on demand via `origin_path_bare()` / `bracketed_origin()`. Verified against live source on the branch. Stressed all 7 requested axes adversarially.

## Critical
None.

## Important
None.

## Minor

**M1 — stale `path_raw` references in three live comments (doc-only).** `bundle.rs:734` (origin_path_for_json comment), `sparrow.rs:234/238` (normalize_derivation docstring + param), `coldcard.rs:306` (per-slot comment). Doc-only; not gating. *(Folded post-R0.)*

**M2 — Phase 6 release-prep pending (expected at this gate).** Version still `0.37.8` (Cargo.toml/lock, both READMEs, install.sh); no CHANGELOG `[0.37.9]`; FOLLOWUP still `open`. Correct ordering (R0 = Phase 5, release-prep = Phase 6); not an R0 blocker. Ship sequence must complete the full Phase-6 checklist. `readme_version_current` guard fails CI if the bump is partial.

## Verification notes (the seven axes)

1. **Byte-identity of `bracketed_origin()` vs former producers — CONFIRMED.** All six foreign-format parsers parse `path` from `format!("m{path_raw_inner}")` where group 2 `((?:/\d+'?)+)` carries a leading `/` and `'`-only hardening. Old `path_raw = format!("[{fp_hex}{path_raw_inner}]")`. `bracketed_origin()` = `[<fp lowercased>/<DerivationPath Display>]`; Display is `/`-joined, no leading `m`/`/`, `'`-hardened → reproduces the old inner string exactly. Only divergence: fp **case** (uppercase-hex foreign descriptors lowercased) — R0-M-1 benign: single-sig export band-aided to bare `m/...` (no fp; emitters took fp from `slot.fingerprint` lowercased); wallet-import re-emit stripped to bare. fp-case never load-bearing. Pinned `synthesize.rs` T5 (`ABCD1234`→`abcd1234`).
2. **`key_origin_str` (`pipeline.rs`) — CONFIRMED both arms.** Path-bearing → `bracketed_origin()`, byte-identical to old. Pathless/default → keeps the path-bearing fallback (R0 C-2 fix present), fp lowercased in both arms. Pinned T5(e).
3. **Completeness — CONFIRMED no missed site.** `\.path_raw` field-access across `src/`: only `coldcard_multisig` local `ResolvedCosigner` (§7) + doc comments. Struct no longer declares the field; suite compiles 2482/0 (field deletion = forcing function). `origin_path_from_bracket` fully removed. `normalize_origin_path` retained (live via `origin_path_for_json`/C5).
4. **A2 distinctness — CONFIRMED correct.** `check_resolved_slots_distinctness` keys on typed `path`, converging with `check_key_vector_distinctness`. Intentional `h`/`'` fold.
5. **Tuple collapses — CONFIRMED clean arity/order.** Single-sig 4→3 + `network_from_origins` 3-tuple; descriptor-mode 5→4 in bundle + verify_bundle. No reorder.
6. **`origin_path_bare()` empty sentinel — CONFIRMED every consumer.** bundle --json n=1 → null (T4); multisig CosignerEntry → `""`; SlotCardBlock `""`→None; electrum/coldcard/sparrow → template fallback. All reproduce old `path_raw.is_empty()`.
7. **Stale comments — see M1.** No comment misleads about behavior. Manual+transcript grep for `m/[` found zero polluted samples (only the new T1 regression-guard assertions).

F5 band-aid removed (`export_wallet.rs`); `mk1_card_to_resolved_slot` populates typed `path` only — F5 dissolved structurally. T1 pins the §1.1 fix.

## VERDICT: GREEN (0C / 0I)
0 Critical, 0 Important, 2 Minor (M1 doc-only, folded; M2 Phase-6 next step). The byte-identity (#1) and `key_origin_str` (#2) axes — where a bracket-byte divergence would have been Critical — are clean: the only difference from prior output is fp lowercasing, provably non-load-bearing on every reachable consumer. Cleared past the end-of-cycle gate.

## Post-R0 addendum (verify-examples drift caught + fixed)
`make verify-examples` (run after this review) FAILED on `recipe-2-bitcoin-core-to-bundle.err`: the A1 + C7 change rendered the engraving-card origin line AND the `ImportWalletSeedMismatch` `at path` clause from bracketed `[b8688df1/84'/0'/0']` to bare `m/84'/0'/0'`. This is the §10 manual-mirror drift the SPEC anticipated — the manual-prose-execution gate correctly detected the intentional A1/C7 behavior change. Expected `.err` re-captured (capture-not-author) to the bare form. No other transcript affected (grep clean).
