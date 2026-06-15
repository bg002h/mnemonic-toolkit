# R0 round 3 — architect review (verbatim) — SPEC_mstring_display_grouping.md

> Persisted verbatim per CLAUDE.md. Dispatched via Agent (feature-dev:code-architect,
> Opus 4.8). **Verdict: GREEN — 0 Critical / 0 Important / 3 Minor.** The SPEC R0
> gate is MET. Spec SHA at review: toolkit `999cfea`. 3 minors folded post-review.

---

## Fold Verification (round-2 findings)

**C1 — `ms combine -`→stdin multiline intake.** Landed completely + correctly across §6/§9.2/§10/§12-P1/§13/§15. (a) Internally consistent — the `ms split | ms combine -` round-trip is conditioned on the new intake throughout. (b) The bogus `ms split | ms decode` claim is fully absent. (c) Precedent `mk-cli/src/cmd/mod.rs:84` (`read_mk1_strings`) confirmed: reads stdin line-by-line, trims, skips empty. (d) No conflict: `CombineArgs.shares: Vec<String>` `required=true`; a bare `-` is a valid clap positional (only `--` terminates options; no `allow_hyphen_values` needed, matching all mk-cli callers); a valid codex32 share begins `ms1…` so can never be `-`; runtime expands `-` to N stdin shares satisfying the ≥1 arity. ✓

**I1 — technical-manual dead-entry removal.** `54-mnemonic-toolkit-api.md:50` (`chunk_5char`), `:51` (`chunk_md1`) confirmed; §11 instructs removal in Phase 3. (`chunk_mk1` has no manual row — minor below.) `render_codex32_grouped` kept in `51-md-codec-api.md`. ✓

**I2 — Phase-1 test naming.** `ms-cli/tests/encode_canonical_12_word.rs:17` asserts `contains("\n\n")` → goes RED under print-once; named in Phase 1 with `encode_canonical_24_word.rs` + the `format.rs` unit tests. ✓ (24-word caveat — minor below.)

**I3 — toolkit `ms_shares` annotations.** `ms_shares.rs:296-310` confirmed: `run_split` already one-per-line + stderr advisory (change is additive); `run_combine --to ms1` annotation now "apply `render_grouped` to the ms1 output" (non-dangling). ✓

**m1/m2.** Differential-harness no-disk-artifact clarification + TSV strip-row `group_size=0`: both landed. ✓

## New-drift sweep
- `ms combine` stdin strip: §9.2(b) "strip each positional/stdin share" covers both paths. ✓
- clap `-` positional: valid value, no `allow_hyphen_values` needed (mk-cli precedent). ✓
- Exhaustive emit-site scan across all 4 repos: every site present in §9.1 (ms encode 198-201; ms split 147-163; mk encode bare println; md encode 81/84; toolkit bundle 978/989/1001/1020; convert 1119; ms_shares run_split/run_combine). No gap. ✓
- Internal consistency §6/§9.1/§12/§15 (repair unbroken/no-flags; split print-once additive; combine new stdin): no contradiction. ✓
- CLAUDE.md: no new `ToolkitError` variant (combine `-`→stdin is in ms-cli, its own `CliError`; toolkit `ms-shares combine` already handles `--share -`); citation-at-write-time stated; no `git add -A`; GUI paired-PR stated. ✓
- Phase-by-phase implementable, zero open decisions, accurate call-site inventory. ✓

## Minor Findings
**m1.** `chunk_mk1` has no row in `54-mnemonic-toolkit-api.md` (only `chunk_5char`:50, `chunk_md1`:51). §11's "remove chunk_5char/chunk_mk1/chunk_md1 rows" → the chunk_mk1 part is a no-op. Fix: "remove the `chunk_5char` and `chunk_md1` rows."
**m2.** `encode_canonical_24_word.rs` asserts only `starts_with("ms10entrsqqqq")` + `contains("word count: 24")` — neither breaks under print-once. Naming it for positive-coverage rewrite is fine, but the "Phase 1 breaks it" motivation is inaccurate for that file (only the 12-word file goes RED).
**m3.** §11 cites `51-md-codec-api.md:194`; actual line is 189 (5-line drift). Per CLAUDE.md citations decay per merge; each impl PR re-greps. Non-blocking.

## Verdict
GREEN — 0 Critical / 0 Important. All R2 Critical/Important folded correctly; new-drift sweep clean; 3 harmless minors. The SPEC is fully implementable phase-by-phase with zero open decisions.
