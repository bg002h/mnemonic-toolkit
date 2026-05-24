# R1 ARCHITECT REVIEW — `BRAINSTORM_v0_37_0_from_import_json_template_reemit.md`

**Round:** R1 (post-R0-fold)
**Date:** 2026-05-24
**Reviewer:** feature-dev:code-reviewer (opus), continuation of R0 agent
**Spec SHA basis:** `36e6bfa`
**Verdict:** RED (0 Critical / 2 Important)

Files re-read against source: `cmd/export_wallet.rs` (full direct + from-import-json paths), `wallet_export/{mod,sparrow,coldcard,electrum,jade,bip388,green}.rs`, the 1209-line `tests/cli_export_wallet_from_import_json.rs` (helper + all cited cells + fixture map), the `bsms-2line-sortedmulti-2of3.txt` / `coldcard-ms-2of3-p2wsh-with-xfp.txt` / `envelope_v0_27_0.json` fixtures, chapter-45 manual + `40-cli-reference/41-mnemonic.md`.

## Fold-verification summary

**I1 (stale citations) — RESOLVED, all exact.** `template: None` `:666` ✓, `threshold_user_supplied: false` `:671` ✓, `let threshold` `:659` ✓, `parsed_ms` `:613` ✓, taproot refusal `:629-639` ✓. Direct-path anchors `:435`/`:454` and `CliExportFormat` def `:22` (I5) also exact. Emitter citations all exact: sparrow refuse `:104` / name·label `:125,137`; coldcard refuse `:111` / `Name:` `:302,353`; electrum refuse `:52` / label `:122,181`; jade refuse `:36`; bip388 branch `:33`; `mod.rs` arms `:220/:221/:229/:237`.

**I4 (coldcard-multisig singlesig) — RESOLVED.** §2.6 correct; verified guard `:493-515` (dispatch) + emit-guard `:713-735`; singlesig→`Bip84`→`_ => Err` holds.

**I5 (predicate placement) — RESOLVED.** `format_requires_template` correctly pinned to `cmd/export_wallet.rs` (where `CliExportFormat` lives, `:22`); exhaustive no-`_` match prescribed. Partition `{Sparrow, Coldcard, ColdcardMultisig, Jade, Electrum}` re-confirmed exhaustive (green reads `script_type` only — green.rs:31/33/35 are comment-only).

**§0 invariant — ACCURATE, does not overclaim.** Confirmed sparrow/electrum emitters consume only `inputs.account` + `inputs.network` (deterministic) beyond name/threshold — no timestamp/random/ordering nondeterminism. The "modulo wallet_name/account/threshold" enumeration is complete for the newly-unblocked formats. The §0 self-defining-test strategy (assert each cell == direct `--template <derived>`) is robust (verified: `--format coldcard` + `WshSortedMulti` routes via `ColdcardEmitter::emit` `:42-54` to `emit_coldcard_multisig_text`, NOT the singlesig refusal at `coldcard.rs:111-137`).

## CRITICAL
None.

## IMPORTANT

### I-R1-1 — §5.1's bsms claim and the `p11a` re-point option (a) are factually wrong: NO `happy_path_fixture` source is `sh(multi)`/P2shMulti; bsms is `wsh(sortedmulti)`→`WshSortedMulti`
`tests/cli_export_wallet_from_import_json.rs:542` maps `"bsms" → bsms-2line-sortedmulti-2of3.txt`, whose descriptor is `wsh(sortedmulti(2,…))` → `WshSortedMulti` (not P2shMulti). All 8 `ALL_SOURCES` (`:563`) resolve to `Bip84` (3 singlesig) or `WshSortedMulti` (5 multisig: bsms/coldcard-multisig/jade/sparrow/specter). The only `sh(multi)`/P2shMulti fixture is `envelope_v0_27_0.json` (Cell 3, `:97`), which `run_export_from_import_envelope` never touches. The R0 review itself seeded this error (R0 asserted "the bsms fixture is `sh(multi(...))`", incorrect). Consequences: (1) §5.1 bsms parenthetical false; (2) `p11a` option (a) unbuildable. **Remedy:** correct §5.1 to bsms→`WshSortedMulti`; drop `p11a` option (a), keep only "singlesig → coldcard-multisig". Confidence 90.

### I-R1-2 — `REFUSAL_STDERR_PATTERNS` fold over-specifies a P2shMulti literal that no `p11c`/`p11a` cell can reach; only Cell 3 exercises it
P2shMulti is never reached by any matrix/helper cell driven through `run_export_from_import_envelope`; the only P2shMulti path is Cell 3 (direct `Command::cargo_bin` `:101-118` with its own inline assertion `:114-117`). The coldcard-multisig literal item (b) is **already present** at `REFUSAL_STDERR_PATTERNS:817`. **Remedy:** the new P2shMulti literal belongs in Cell 3's inline assertion, not the shared const; note item (b) already exists (no edit). Confidence 80.

## MINOR
- **M-R1-a** — §3/Phase2 cite `45:353` for prose update, but `:353` is the still-valid taproot-gated round-trip note; correct target is `45:347`. Leave `:352-357` unchanged.
- **M-R1-b** — M5 cli-ref note at `41-mnemonic.md:669` must make clear the derivation is internal (user still cannot pass `--template`), else the "mutually exclusive with --template" row self-contradicts. Phase-2 wording task.
- **M-R1-c** — §0/§5.3 account-match relies on fixtures being account-0 (all are; direct default is 0). Add one sentence to §5.3.
- **M1–M5 (R0)** — all handled. M1 substring-vs-structural picked whole-`parsed_ms.to_string()` with explicit ordering guard + sound no-taproot justification (verified `wsh(sortedmulti(` is a real Display form in-tree).

## VERDICT
**VERDICT: RED (0C/2I)** — both Important are fold-introduced prose drift (R0's incorrect bsms=`sh(multi)` premise propagated into a phantom `p11a` option and a misplaced `REFUSAL_STDERR_PATTERNS` instruction), not design defects. Mechanical to fix. Fold and re-dispatch for R2.
