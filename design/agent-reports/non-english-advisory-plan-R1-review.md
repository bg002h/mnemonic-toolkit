# R0 Review (R1) — IMPLEMENTATION_PLAN_non_english_seed_advisory.md

Opus architect, continuing from plan-R0 (RED 0C/3I/3M). Verified fold vs live source. Persisted by controller.

## Fold verification
- **I1 RESOLVED** — convert `:947` insertion is past EVERY `return Err` refusal (§3 loop `:882-886`, WIF+path `:890`, xpub-prefix `:896`, bip38 `:905`) + the existing advisory cluster (`:926-947`), before `compute_outputs` `:952`. `targets`/`args.language`/`stderr` in scope. A refused edge errors WITHOUT advising.
- **I2 RESOLVED** — slip39 split emit after `parse_master_to_entropy` `:437` (errors via `?` first); combine emit between successful `slip39_combine` `:644` and the `match args.to` render `:647`. Bindings in scope; emits only after the fallible op.
- **I3 RESOLVED (highest-value)** — `FRENCH_12` = `abaisser ×11 + abeille` checksum-valid: French `abaisser`=idx0, `abeille`=idx3; all-zeros entropy → checksum word at idx3 in BOTH wordlists (English canonical `abandon ×11 about`, `about`=idx3) → mutually confirming. Generation via `convert --from entropy=0…0 --to phrase --language french` sound. `ENGLISH_12` canonical.
- **M2 RESOLVED** — import-json fires-note accurate (`emit_args = args.clone()` preserves `--language`; `:1670-1674`; gated on `any_secret_bearing()`).
- **M3 RESOLVED** — `Slip39ToShape::Phrase` `:187` (one residual `:188` cite at plan line 149, non-load-bearing).

## Additional checks
- All 4 emit snippets compile (helper sig; `PartialEq`/`Default`=English; `human_name()` kebab; `NodeType::Entropy` `convert.rs:40`; bundle/convert `args.language.unwrap_or_default()` Option vs slip39 direct `CliLanguage`).
- French combine-test share-gen works end-to-end (split parses french → entropy → shares to stdout).
- `--to seedqr → exit 64` non-regression sound (rejected at clap parse, never reaches `:947`).

## CRITICAL — None.  ## IMPORTANT — None.
## MINOR
- M-r1-1 plan line 149 `Phrase :188` → `:187` (cosmetic). M-r1-2 note per-file const copies (separate compilation units). M-r1-3 sharpen M2 note (import-json fires only when secret-bearing).

## VERDICT: GREEN (0C/0I/3M) — clear to implement.
All 3 R0 Importants resolved against live source; 4 emit snippets compile; no load-bearing hand-wave remains.
