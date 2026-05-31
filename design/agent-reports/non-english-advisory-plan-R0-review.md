# R0 Review — IMPLEMENTATION_PLAN_non_english_seed_advisory.md

Opus architect, mandatory pre-impl R0. Branch `non-english-seed-advisory`, base `master`
`9f11a31`. Verified plan code vs live source. Persisted by controller. (RED 0C/3I/3M.)

## Headline confirmations
- **Helper compiles:** `CliLanguage` derives `PartialEq` (`language.rs:8`); variants `French` (`:16`), `SimplifiedChinese` (`:13`), `English` (`:12`) REAL; `human_name()` → `"french"` (`:32`), `"simplified-chinese"` (`:29`) EXACT (tests pass). `pub(crate)` reachable from `cmd/{bundle,convert,slip39}` (sibling crate-root mods).
- **bundle fires once:** `emit_unified` (`bundle.rs:698`) has `args`/`bundle`/`stderr` in scope; `use std::io::Write` (`:15`); all 3 dispatch branches converge on ONE `emit_unified`. `any_secret_bearing()` (`synthesize.rs:35`) ✓.
- **slip39 bindings real:** `run_split` (`:359`, `args.language: CliLanguage` `:133`, `stderr` `:363`); `run_combine` (`:566`, `args.to: Slip39ToShape` `:168`, `args.language` `:172`, `stderr` `:570`); `Slip39ToShape::Entropy` (`:185`)/`Phrase` (`:187`).

## CRITICAL — None.

## IMPORTANT
- **I1 — convert insertion `~:890` emits BEFORE 3 more refusal guards.** The §3 pre-check loop is `:882-886`, but WIF+`--path` (`:889-891`), `--xpub-prefix`-no-`--network` (`:894-898`), BIP-38-no-passphrase (`:902-906`) all `return Err` AFTER it → a refused edge would print the advisory then error. **Fix: insert after `:947`** (after all refusals + the existing advisory cluster `:926-947`, before `compute_outputs` `:952`); `targets`/`args.language`/`stderr` all in scope.
- **I2 — slip39 emits must fire only after the op succeeds.** `run_combine` parses+combines (`:631-643`, can Err) → pin the combine emit AFTER successful `slip39_combine` at `:644` (before output `:647`). `run_split`: prefer after `parse_master_to_entropy` succeeds (`:437`), not before (a bad phrase shouldn't advise-then-error).
- **I3 — no valid French BIP-39 test vector in the repo (suite is English-only).** All 4 integration tests say "a real French const" but supply none; `Mnemonic::parse_in(French, …)` is strict → an invalid phrase fails parse before the advisory. French word[0]=`abaisser` (not transliterable from `abandon`). **Fix: name a concrete checksum-valid French 12-word const** (generate via `convert --from entropy=00000000000000000000000000000000 --to phrase --language french` + paste the literal); the slip39 combine test's French shares flow from that vector via `slip39 split`.

## MINOR
- **M1** split fires on `--from entropy=<hex> --language french` (language "ignored" for entropy input, `:131`) — harmless (shares lose language) but message slightly off; note in comment.
- **M2** `bundle --import-json --language <non-english>` (secret-bearing) also fires (emit in `emit_unified`, reached at `:1674`) — correct, but add a test assertion/comment so it's not read as accidental.
- **M3** plan cites `Slip39ToShape::Phrase :188`; actual `:187`. Cosmetic.

## VERDICT: RED (0C / 3I / 3M)
Buildable + bundle fires once, but fold I1 (convert emit → after `:947`), I2 (slip39 emits after the op succeeds: combine `:644`, split `:437`), I3 (name a concrete French vector + generation cmd). Then re-dispatch R1.
