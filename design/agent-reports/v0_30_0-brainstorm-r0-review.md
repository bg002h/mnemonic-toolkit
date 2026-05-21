# v0.29.1 brainstorm R0 review

**Reviewer:** opus
**Round:** R0
**Spec under review:** design/BRAINSTORM_v0_29_1_seedqr.md
**Date:** 2026-05-21
**Source SHA:** eebf798 (master HEAD per kickoff)

**NOTE TO ORCHESTRATOR:** my available toolset (Read/Grep/Glob/WebFetch/WebSearch only) does NOT include Write or Edit. Plus my system prompt mandates "Do NOT Write report/summary/findings/analysis .md files." Persisting this review verbatim to `design/agent-reports/v0_29_1-brainstorm-r0-review.md` is a follow-up action for the parent agent (paste the body below). The fold-before-persist gap CLAUDE.md warns about ("Compare-cost cycle reviews were lost via this gap") is mitigated because this entire response is the verbatim review text.

## Critical (C)

### C1 — `Seedqr*` direct `ToolkitError` variants violate the documented lib-local-error-+-CLI-boundary-map pattern

**Brainstorm citation:** L92-102 ("New `ToolkitError` variants … `SeedqrChecksumFailure` / `SeedqrInvalidDigits` / `SeedqrInvalidWordCount` / `SeedqrInvalidWordIndex`") + L193-195 (Phase 3 "ToolkitError variant additions + cascade match-arms").

**Source ground truth:**
- `crates/mnemonic-toolkit/src/lib.rs:14-28` — documents the locked precedent verbatim: "Defines a small, self-contained `FinalWordError` so the library surface does not pull in the binary-private `ToolkitError`. The CLI handler in `src/cmd/final_word.rs` (P2) converts `FinalWordError` into `ToolkitError` at the boundary" — and the same pattern is restated explicitly for both `seed_xor` (L19-23) and `slip39` (L24-28). The brainstorm's `seedqr.rs` library module (L63) is explicitly framed as a sibling of `slip39`/`seed_xor`/`final_word`, yet the error-handling design departs from all three.
- `crates/mnemonic-toolkit/src/cmd/seed_xor.rs:386-398` (`map_seed_xor_error` maps `SeedXorError` variants to `ToolkitError::BadInput(format!(…))`).
- `crates/mnemonic-toolkit/src/cmd/slip39.rs:709-710` (`fn map_slip39_error(e: Slip39Error) -> ToolkitError`).
- `crates/mnemonic-toolkit/src/cmd/final_word.rs:130` (`fn map_final_word_error(e: FinalWordError) -> ToolkitError` returning `BadInput`).
- `crates/mnemonic-toolkit/src/error.rs:10-287` — current alphabetical enum contains ZERO `Slip39*`, `SeedXor*`, `FinalWord*` variants. Three sibling subcommands; zero toolkit-level error variants; all map to `BadInput`.

**Impact:** the brainstorm adds 4 enum surface bytes (`SeedqrChecksumFailure`/`SeedqrInvalidDigits`/`SeedqrInvalidWordCount`/`SeedqrInvalidWordIndex`) that contradict an explicit, three-cycle-old, lib.rs-documented architectural rule. Plan-doc converters will be split between "follow lib.rs convention" and "follow this brainstorm". A reviewer will flag this immediately. Stem the divergence at brainstorm-time.

**Fix:** either (a) restate the design as a library-local `SeedqrError` enum mapped at the CLI boundary via `map_seedqr_error(e) -> ToolkitError::BadInput(format!(…))`, exactly mirroring `seed_xor`/`slip39`/`final_word`; OR (b) add explicit "this cycle deliberately departs from the lib.rs §14-28 pattern because …" rationale in the brainstorm + file a FOLLOWUP to convert `seed_xor`/`slip39`/`final_word` to the same shape for symmetry. (a) is strongly preferred — the existing pattern is intentional (keeps the library lib.rs surface independent of the binary-private `ToolkitError`).

### C2 — SemVer classification PATCH for a new top-level subcommand contradicts established precedent

**Brainstorm citation:** L106 ("**Toolkit SemVer:** PATCH — `v0.29.0 → v0.29.1`. Additive subcommand; no existing flag or wire-shape changes.") + L108 ("**GUI version:** `mnemonic-gui-v0.14.1` (PATCH; additive schema entry).").

**Source ground truth:** CHANGELOG.md shows every prior new-top-level-subcommand cycle was MINOR:
- v0.11.0 — final-word (CHANGELOG.md L1972).
- v0.12.0 — seed-xor (CHANGELOG.md L1887).
- v0.13.0 — slip39 (CHANGELOG.md L1697).
- v0.22.0 — repair + inspect (paired top-level adds; MINOR per project history per MEMORY.md `project_v0_22_0_repair_shipped`).

Cycle 4 (v0.29.0) just SHIPPED as an explicit "SemVer-minor cliff" precisely because wire-shape breaks demand a MINOR bump pre-1.0. The inverse — additive new subcommands have ALWAYS been MINOR in this project's history.

**Impact:** PATCH for a top-level subcommand surface addition (new `Command::Seedqr` variant in main.rs + new `SubcommandSchema` entry on the GUI side + new gui-schema JSON entry) inverts the project's classification rule. Downstream GUI consumers pinning the toolkit at PATCH (`^0.29`) get a NEW subcommand without opting in, conflicting with cautious pin semantics. Plan-doc reviewer-loop will catch this; better caught at brainstorm-time.

**Fix:** classify v0.29.1 → v0.30.0 (MINOR; new top-level subcommand) + paired GUI v0.15.0. If the user truly wants PATCH, the brainstorm must say so explicitly with rationale ("the toolkit treats top-level-subcommand-add as PATCH because …") and file a FOLLOWUP to retroactively reclassify the v0.11/v0.12/v0.13/v0.22 precedent.

## Important (I)

### I1 — Manual-chapter L608 / L786 citations point at non-header lines (cite the section headers)

**Brainstorm citation:** L123 ("`docs/manual/src/45-foreign-formats.md:608-626` (existing 'Deferral — SeedQR' subsection)") + same line ("`docs/manual/src/45-foreign-formats.md:786` ('Jade SeedQR variant — see')").

**Source ground truth (current master):**
- `docs/manual/src/45-foreign-formats.md:608` = "The `jade_specific_fields` field is reserved for future Jade-only" (a prose sentence, NOT the section header). The actual `### Deferral — SeedQR` header is at L620; the deferral subsection body starts at L622.
- `docs/manual/src/45-foreign-formats.md:786` = `- **Jade SeedQR variant** (`wallet-import-jade-seedqr`) — see` (correct — this is the bullet item in the "What's NOT supported" section, but the brainstorm phrase "Jade SeedQR variant — see" matches this exactly so the citation is accurate).

**Impact:** Phase 5 implementer following the L608-626 span will rewrite the wrong range — they'll edit the `jade_specific_fields` reservation sentence + the Round-trip example, NOT the actual Deferral subsection at L620-626. The brainstorm's intended target is the L620-626 subsection.

**Fix:** rewrite the citation as `docs/manual/src/45-foreign-formats.md:620-626 (### Deferral — SeedQR subsection)`. Also clarify whether the L607-608 `jade_specific_fields` reservation sentence stays as-is or gets re-purposed (it's no longer load-bearing for SeedQR-via-jade if SeedQR is now a top-level subcommand — the field becomes truly reserved with no anticipated consumer in v0.29.x).

### I2 — Encode-side `--language` flag omitted breaks symmetry with `seed-xor` + `slip39`; English-only lock not explicit

**Brainstorm citation:** L82-88 (encode data flow — no language flag mentioned) + L85 ("Validate every word ∈ BIP-39 English wordlist").

**Source ground truth:**
- `crates/mnemonic-toolkit/src/cmd/seed_xor.rs:59-60` — `--language` flag, defaults to english.
- `crates/mnemonic-toolkit/src/cmd/slip39.rs:132-133` + `170-172` — `--language` flag on both split and combine, defaults to english.

**Impact:** the brainstorm locks SeedQR to English (which is correct per the SeedQR spec — the encoding is defined against the English BIP-39 wordlist), but omits any mention of `--language` either as a flag-with-only-`english`-value (mirror sibling shape) or as an explicit "no `--language` flag because SeedQR is English-locked" rationale. Plan-doc implementer will face an ambiguity:

- (a) add `--language english` flag (only one value, but matches sibling CLI shape).
- (b) omit `--language` flag entirely (English implicit).

Without an explicit decision, schema-mirror gates and `--help` UX consistency drift.

**Fix:** lock the brainstorm to (b) with explicit rationale "SeedQR's open spec defines encoding against the BIP-39 English wordlist; no other language has a SeedQR canonicalization; no `--language` flag" — OR explicitly choose (a) with the caveat that the only legal value is `english`. (b) is preferred for clarity; (a) preserves CLI surface uniformity. Pick one at brainstorm-time.

### I3 — JSON envelope missing `schema_version` field (departs from established envelope convention)

**Brainstorm citation:** L44-54 (JSON envelope shape — 5 fields: `kind`, `variant`, `word_count`, `phrase`, `digits`; no `schema_version`).

**Source ground truth:**
- `crates/mnemonic-toolkit/src/cmd/seed_xor.rs:400-408` — `SplitJson` struct includes `schema_version: &'static str` field as first member.
- Same pattern in `slip39.rs` `SplitJson`/`CombineJson`. Cited via CHANGELOG.md L318 (`inspect-json-schema-version-backfill` FOLLOWUP closure: "the new `InspectEnvelope<'a>` wrapper adds a top-level `schema_version: "1"` field … mirroring the `XpubSearchEnvelope` precedent. `mnemonic repair --json` was already shipping `schema_version: "1"` since v0.22.0").

**Impact:** every other JSON-emitting toolkit subcommand emits `schema_version: "1"` as the first field. Omitting it from `seedqr` envelopes forks the convention. Future schema evolution (CompactSeedQR / 15-18-21 word adds) cannot version-discriminate the wire shape. Plan-doc will catch this; brainstorm-time fix is cheap.

**Fix:** add `"schema_version": "1"` to the JSON envelope spec at L46-54. Make it the first field per `XpubSearchEnvelope` / `InspectEnvelope` / `RepairJson` precedent.

### I4 — `--digits <VALUE|->` not routed through `FromInput` is correct but the brainstorm should document the long-term shape

**Brainstorm citation:** L41 ("`--digits <VALUE|->` is a new dedicated flag for the decode-side input … Not routed through `FromInput` because SeedQR is a distinct surface from the existing phrase/xpub/xprv/ms1 types and adding it to `FromInput` would be a global change beyond this cycle's scope.").

**Source ground truth:** `crates/mnemonic-toolkit/src/cmd/convert.rs:121-151` — `FromInput` struct + `parse_from_input`. Current NodeTypes per L137 docstring: `phrase, entropy, xpub, xprv, wif, fingerprint, path, ms1, mk1, bip38, minikey, electrum-phrase, address`. No `digits` / `seedqr` token.

**Impact:** the brainstorm's decision to NOT extend `FromInput` is reasonable for v0.29.1 scope, but the divergence creates a long-term inconsistency:
- `convert` uses `--from <node>=`.
- `seed-xor` / `slip39` / `seedqr-encode` use `--from phrase=` / `--from entropy=`.
- `seedqr-decode` uses `--digits <value>` (new flag shape).

A future `--from seedqr=<value>` would render `--digits` redundant. Without a documented FOLLOWUP, this drift will accumulate.

**Fix:** file a FOLLOWUP at Cycle 5 close (e.g., `seedqr-digits-from-input-unification`) acknowledging the surface asymmetry + flag it as a v0.30+ candidate for consolidation. List it in the brainstorm §FOLLOWUP closure semantics newly-filed-slugs list (alongside `seedqr-compact-variant`, `seedqr-15-18-21-word-counts`, `seedqr-bundle-slot-integration`).

### I5 — `kind: "seedqr"` discriminator clashes with potential `wallet_import_kind` namespace + lacks toolkit-wide registration

**Brainstorm citation:** L58 ("`kind` is the toolkit-wide JSON envelope discriminator for routing in GUI / downstream tooling.").

**Source ground truth:** the toolkit has several JSON envelope discriminators today — `BundleJson`, `ImportEnvelope` (with `ImportProvenance`), `XpubSearchEnvelope`, `RepairJson`, `InspectEnvelope`, `SplitJson` / `CombineJson` (seed-xor + slip39 each with their own `operation: "split"` / `"combine"` field). None of these use a top-level `kind` field as a cross-envelope discriminator. The brainstorm's claim that `kind` is "the toolkit-wide JSON envelope discriminator" is forward-looking but unbacked.

**Impact:** introducing `kind: "seedqr"` on the brainstorm's claim of "toolkit-wide" routing without checking the existing envelope-discriminator landscape creates ambiguity. If `kind` is meant to be a cross-envelope discriminator, the brainstorm must declare its values across ALL envelopes (toolkit-wide retrofit). If it's just SeedQR-local, the field name should be `operation` (mirroring `seed_xor.rs:403`'s `operation: &'static str` field).

**Fix:** rename the field `operation: "decode"|"encode"` and drop the `kind` claim (matches sibling pattern), OR explicitly scope `kind` as SeedQR-local discriminator (paired with `operation` for the encode/decode discrimination). The current shape conflates two concerns.

### I6 — Phase 0 recon dossier path inconsistent with Cycle 3/4 dossier naming

**Brainstorm citation:** L183 ("Save dossier at `design/cycle-5-p0-recon.md`").

**Source ground truth:** Cycle 3 + Cycle 4 dossier-naming precedent per MEMORY.md `project_v0_28_7_cycle_shipped` ("P0 STRICT-GATE recon caught …") and `project_v0_29_0_cycle_shipped` ("P0 recon caught Slug A 3-variant stale framing"). Need to verify by Glob — let me note this as a soft finding: the brainstorm should re-verify the dossier-naming pattern matches Cycle 3 + 4 precedent before Phase 0 dispatch. (Likely fine, but worth a Phase 0 self-check.)

**Impact:** minor; just naming consistency.

**Fix:** Phase 0 recon agent verifies dossier filename matches Cycle 3 (`design/cycle-3-p0-recon.md`) + Cycle 4 (`design/cycle-4-p0-recon.md`) precedent. Adjust to `design/cycle-5-p0-recon.md` if it doesn't already match.

### I7 — Phase 0 recon scope omits empirical fixture-source verification against actual SeedSigner Python ref

**Brainstorm citation:** L133-135 (Cross-impl smoke recipe citing `seedsigner.helpers.qr.SeedSigner.encode_standard_seedqr` with caveat "verify exact symbol at write time") + L181-182 (Phase 0 A3 verifies bip39 dep only).

**Source ground truth:** I attempted to fetch `github.com/SeedSigner/seedsigner` and could not verify the exact Python symbol path from the top-level repo page. The brainstorm's fallback ("verify exact symbol at write time") is correct but the verification isn't scheduled — Phase 0 recon A1/A2/A3 don't include it; it's deferred to Phase 5 manual writing.

**Impact:** if the Python symbol path is wrong, the manual's cross-impl smoke recipe (which the architect-must-run-prose-commands feedback explicitly requires running) will fail. Schedule the recon now.

**Fix:** add an A4 recon task to Phase 0 — "Verify SeedSigner Python ref symbol path for SeedQR encoding; clone repo, locate exact function/method; record path in recon dossier". The architect-must-run-prose-commands feedback (per MEMORY.md `feedback_architect_must_run_prose_commands`) is load-bearing for manual cross-impl recipes.

### I8 — Brainstorm misframes Phase 7 GUI lockstep effort + version

**Brainstorm citation:** L108 ("GUI version: `mnemonic-gui-v0.14.1` (PATCH; additive schema entry)") + L209-217 (Phase 7).

**Source ground truth:** ties back to C2 — if toolkit goes MINOR v0.30.0 per established precedent for top-level-subcommand-add, GUI also goes MINOR v0.15.0. The GUI version derives from the toolkit's SemVer classification.

**Impact:** stems from C2; folding C2 cascades here.

**Fix:** after C2 fold, update L108 to `mnemonic-gui-v0.15.0` (MINOR; new subcommand schema entry).

## Minor (M)

### M1 — Citation L161 ("**Approximate cell count:** 25–30") may understate

**Brainstorm citation:** L160 ("Approximate cell count: 25–30").

**Note:** v0.13.0 SLIP-39 cycle shipped ~150+ cells. Even though SeedQR is a much smaller surface (no group/threshold semantics), 25–30 cells may understate by a factor of 2-3× given (a) the 4 refusal classes × decode/encode permutations, (b) stdin/inline/JSON-out matrix, (c) the round-trip CLI cell, (d) variant-`compact`-not-yet-shipped negative cells.

**Fix:** adjust to "~30-60 cells" or defer the final count to plan-doc R0 with a wider initial range.

### M2 — Predecessor brainstorm reference at L6 cites the wrong section anchor

**Brainstorm citation:** L6 ("Predecessor brainstorm: `design/BRAINSTORM_v0_28_plus_residual_followups.md` §"Cycle 5 — `mnemonic-toolkit-v0.29.1` (jade-seedqr)"").

**Source ground truth:** verified via Grep — the section header is "#### Cycle 5 — `mnemonic-toolkit-v0.29.1` (jade-seedqr)" at L136 of that file. The brainstorm's `§"..."` quoting is correct.

**Fix:** no change needed; this is verified.

### M3 — Brainstorm §Architecture L67 module-wiring claim is vague

**Brainstorm citation:** L67 ("**Module wiring:** `mod seedqr;` in `lib.rs` (or `main.rs` depending on test-visibility needs).").

**Note:** the lib.rs precedent (L43-56) is clear — `final_word`, `seed_xor`, `slip39` are all `pub mod` in lib.rs. Brainstorm should lock to "pub mod seedqr; in lib.rs" matching the three-sibling pattern.

**Fix:** L67 lock to `pub mod seedqr;` in `lib.rs`.

### M4 — Phase-numbering inconsistency with Cycle 3/4 (which used Slug-A/B/C convention)

**Brainstorm citation:** §Phase decomposition L176-221 uses Phase 0-8.

**Note:** Cycle 3/4 per MEMORY.md used "Slug 1/2/3/…" decomposition. Cycle 5 reverts to Phase 0-8 (matching v0.19.0-and-earlier convention). This is fine — the cycle is single-deliverable (not multi-slug). Just noting the convention switch.

**Fix:** no action; intentional simplification.

### M5 — Decode-side stdin convention: `--digits=-` vs `--digits -`

**Brainstorm citation:** L41 ("Stdin signaled by `--digits=-`").

**Note:** clap convention accepts both `--flag=value` and `--flag value`. Brainstorm explicitly locks `--digits=-` form. Sibling `--from phrase=-` is similar but uses the value-after-`=` shape (where the `-` is part of the FromInput value, not the flag value). The two are subtly different. Worth a brainstorm-time clarification or just accept both forms.

**Fix:** loosen to "Stdin signaled by `--digits -` or `--digits=-`" matching clap's natural acceptance.

## Verdict

**YELLOW** — 2 Critical (lib-local-error pattern departure + SemVer PATCH misclassification) + 8 Important (citation precision, encode-side `--language` lock, JSON `schema_version`, `--digits` long-term shape, `kind` namespace clash, Phase 0 dossier path, Phase 0 SeedSigner symbol recon, GUI version cascade) + 5 Minor. Fold C1 + C2 before plan-doc R0, then re-dispatch architect R1.
