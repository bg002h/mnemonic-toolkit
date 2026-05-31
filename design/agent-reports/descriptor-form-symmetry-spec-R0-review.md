# SPEC R0 review (R0, formal gate) — descriptor-form symmetry (A1)

**Date:** 2026-05-31 · **Reviewer:** opus architect · **Repo SHA:** `ea8ba88` · **Target:** `design/SPEC_descriptor_form_symmetry.md`
**Verdict: RED (2C/3I/4M).** Fold + re-dispatch.

> Persisted verbatim per CLAUDE.md before the fold-and-commit step.

## Confirmed sound (architect verified against source)
- `key_regex` (`pipeline.rs:38`) does NOT match the `@N`-with-origin form (`@0[fp/path]/<0;1>/*` — no `]xpub` after the bracket; confirmed by test `pipeline.rs:155-156`). The §3.1 "matches both probes → mixed" rule is SAFE. The controller's highest-risk drift hole is a non-issue.
- I1 dropped cleanly: `export-wallet --descriptor` pure passthrough (`export_wallet.rs:332-334`); `@N` redirect before `from_str`. Not lurking.
- M2: `ParsedFingerprint { i, fp:[u8;4] }` (`parse_descriptor.rs:739-742`), not `bitcoin::Fingerprint`. Folded correctly.
- M3 taproot: `NUMS` literal has no `[fp/path]` → `key_regex` skips it. Cell sound.
- `CosignerKeyInfo = ResolvedSlot` (`synthesize.rs:219`) → §3.2's `Vec<ResolvedSlot>` composes with `synthesize_descriptor`.
- SemVer PATCH / no-GUI-lockstep / no-`cli-subcommands.list` correct.

## Critical
**C1 — §3.2 cites the wrong source and omits the load-bearing path/xpub-recovery mechanic.** `bundle.rs:1644-1659` does NOT build ResolvedSlots from a descriptor's inline triples — its ResolvedSlots come from `envelope_to_resolved_slots` (mk1 cards, `bundle.rs:1531`). The real "concrete → watch-only ResolvedSlots" pattern is the import parsers: `bsms.rs:219-265` → `concrete_keys_to_placeholders` → `parse_descriptor` → per-slot `build_slot_fields(body,i)` (`bsms.rs:399-416`) → `extract_origin_components` (`bsms.rs:362-394`). `ParsedKey{i,payload:[u8;65]}` carries NO path and the payload is a LOSSY compact xpub (no depth/parent-fp/child), but `ResolvedSlot.path: DerivationPath` + `ResolvedSlot.xpub: Xpub` (full) are required — so paths AND full xpubs must be re-recovered from the original body's base58 (`build_slot_fields` does `slip0132::normalize_xpub_prefix` → `Xpub::from_str`). The SPEC never states this. *Fix:* re-cite the import-parser pattern; specify path+xpub recovery (re-scan body with the widened `key_regex`, capturing group1=fp / group2=path / group3=xpub_str, then `normalize_xpub_prefix`→`Xpub::from_str` per key); drop the false `bundle.rs:1644-1659` "pure extraction" framing.

**C2 — §3.3 `h`-form fix under-scoped: it widens only `key_regex`, but the import-parser path-recovery (`extract_origin_components`) depends on a SECOND apostrophe-only regex `origin_capture_regex` (`bsms.rs:516`, also `specter.rs:362`, `sparrow.rs:582`, `bitcoin_core.rs:413`).** If the §3.2 helper reuses `extract_origin_components`, an `h`-form descriptor yields N keys from the widened `key_regex` but ZERO origins → slot mismatch (`bsms.rs:404-408`). *Fix:* commit to one — (a) widen `key_regex` AND all 4 `origin_capture_regex`, or (b) the helper recovers `(fp,path,xpub)` via the widened `key_regex` directly (one pass, all 3 groups) and does NOT call `extract_origin_components`, leaving the 4 copies untouched (import-parser h-form support stays pre-existing/out-of-scope → FOLLOWUP). Controller chose (b).

## Important
**I1 — §3.4 claims the watch-only modes (`SingleSigWatchOnly`/`MultisigWatchOnly`, `bundle.rs:1666/1669`) "already exist" for `--descriptor`, but they live inside `bundle_run_from_import_json`, a function `--descriptor` never reaches.** Bare-concrete `bundle --descriptor` routes `run`→`bundle_run_unified` (`:286`)→`bundle_run_unified_descriptor` (`:338`)→`lex_placeholders` which REJECTS it (`parse_descriptor.rs:135`) and requires `--slot` per `@N` (`:1072`). The Concrete arm needs NEW wiring (a sibling synthesis fn), not reuse of the `@N`+`--slot`-coupled `bundle_run_unified_descriptor`. *Fix:* name the classifier insertion point (the `bundle.rs:338` dispatch fork) and state Concrete routes to a NEW fn mirroring `bundle_run_from_import_json`'s synthesis tail.

**I2 — §1/§3.4 reference a `--phrase` flag that does not exist on `BundleArgs`.** Seed input is `--slot @N.phrase=` / `--ms1` (`bundle.rs:94-153`). *Fix:* replace `--phrase` throughout §1/§3.4.

**I3 — §3.1 rule 4 / §3.5 mis-route the origin-less-key error: a "matches neither probe" input never reaches md-codec, so it cannot "surface through md-codec's origin policy."** `md_codec::MissingExplicitOrigin` only fires when the converter feeds `parse_descriptor`; rule-4 inputs never invoke it. *Fix:* the classifier itself emits (and pins) the origin-required error text.

## Minor
- **M1** — §3.3 proposes `(?:'|h|H)?` but live `lex_placeholders` (`parse_descriptor.rs:70`) is `(?:'|h)?` (no `H`). Drop `H` or note deliberate superset.
- **M2** — error-line citations off by 1 (SPEC `75,81,108,117` vs literal bodies `74-76,80-83,107-110,116-119`). Re-pin.
- **M3** — Test 6 convergence partner must be explicitly origin-bearing (both inputs) or default-path inference could diverge (`bundle.rs:1130-1224`).
- **M4** — `build_slot_fields` duplicated in 6 parsers, `extract_origin_components` in 4. The new helper adds a 7th origin-extraction. Acknowledge + FOLLOWUP for consolidation.

## Controller fold plan (2026-05-31)
C2 → option (b) (helper uses widened `key_regex` one-pass; origin_capture_regex untouched; import-parser h-form = FOLLOWUP). C1 → re-cite import-parser pattern + specify path+xpub recovery via widened key_regex + Xpub::from_str. I1 → insertion at bundle.rs:338 fork → new Concrete fn mirroring from_import_json tail (+ verify_bundle.rs:614 analog). I2 → `--slot @N.phrase=`. I3 → classifier emits pinned origin-required error. M1 → drop H. M2 → re-pin. M3 → both origin-bearing. M4 → FOLLOWUP filed.
