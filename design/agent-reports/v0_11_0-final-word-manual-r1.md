# v0.11.0 Phase 3 — manual mirror R1 reviewer report

**Phase:** P3 — manual chapter + cli-subcommands lint mirror
**Round:** R1 round 1
**Reviewer:** Opus (`feature-dev:code-reviewer`)
**Date:** 2026-05-14
**Commit under review:** `69be728` (P3 ship)
**Predecessor:** `f34a453` (P2 R1 LOCK)

## Verdict

**0 Critical / 0 Important / 3 Nice-to-have — R1 LOCK round 1.**

Phase 3 ships pending user authorization to push.

## Scope reviewed

All 12 mandatory reviewer checks: flag-table completeness, flag-name
precision, refusal-table accuracy, advisory-table accuracy, JSON-output
schema accuracy, worked-example reproducibility, anchor-link integrity,
cli-subcommands.list invariant, cspell-additions justification, style
consistency, lint-uncovered prose accuracy, no premature version-tag
attribution. All 12 checks PASS.

Files reviewed:
- `docs/manual/src/40-cli-reference/41-mnemonic.md` — intro update + new `## mnemonic final-word` section
- `docs/manual/tests/cli-subcommands.list` — `mnemonic final-word` row
- `docs/manual/.cspell.json` — `cmdline` + `simplifiedchinese` + `traditionalchinese`
- Cross-referenced against `src/cmd/final_word.rs`, `src/secret_advisory.rs`, `src/language.rs`, SPEC §2.2/§2.3/§2.5/§2.6, `docs/manual/tests/lint.sh`

## Key validations

1. **Flag table is complete + precise.** All four flags (`--from`,
   `--language`, `--json-out`, `--help`) appear in the manual table with
   the byte-exact `value_name` rendering (`<phrase=<value-or-->>`,
   `<PATH>`). The 10 `CliLanguage` enum values are correctly rendered as
   `simplifiedchinese` etc (clap `rename_all="lower"` convention), NOT
   kebab-case (kebab-case is the JSON envelope rendering, distinct).

2. **Refusals + advisories are byte-faithful to source.** All 4 refusal
   stems and all 3 advisory texts match the emit sites in
   `cmd/final_word.rs` and `secret_advisory.rs` exactly. The "world-readable
   mode 644" advisory wording matches the runtime format.

3. **Worked example reproducible.** `abandon × 23 → 8 candidates including
   art` claim verified via `tests/cli_final_word_happy_paths.rs::n24_*`
   and the BIP-39 canonical zero-entropy 24-word vector ending in `art`.

4. **Lint mirror correct.** `cli-subcommands.list` row matches kebab-case
   `final-word` clap output exactly; `lint.sh` flag-coverage runs cleanly;
   all 6 lint stages green (`make -C docs/manual lint`).

5. **cspell additions justified.** All three additions (`cmdline`,
   `simplifiedchinese`, `traditionalchinese`) are forced by clap output
   or literal advisory text and are not reworkable away.

6. **No glossary drift introduced.** Pre-existing drift at
   `61-glossary.md:152` (omits both `gui-schema` and now `final-word`)
   is out-of-scope per "do not surface pre-existing issues" rule, but
   noted as a PE fold-in candidate (see N2 below).

## Nice-to-have findings (non-blocking)

**N1.** `41-mnemonic.md` refusal row 4 — manual omits the dynamic
`; got <node>=` suffix from the non-Phrase refusal message. Editorial
choice acceptable (suffix is variable); for consistency with row 1/2's
`...` ellipsis convention, consider appending `...` or quoting the
template form. Trivial nit.

**N2.** `docs/manual/src/60-appendices/61-glossary.md:152` — pre-existing
drift (says "Five subcommands"; now 7). Not introduced by this commit.
PE fold-in candidate.

**N3.** `41-mnemonic.md` worked example — doesn't cross-reference the
TTY "candidate list is secret material" advisory documented in the
advisories table. Cosmetic cross-reference nit.

## R1 LOCK

Phase 3 R1 LOCK round 1. Ship pending user-authorized push.

PE fold list (gathered across reviewer rounds):
- SPEC §2.4/§2.5 narrative exit-code drift (64 → 1) — from P2 R1
- Glossary subcommand-count drift fix — from P3 N2
- (Optional) N1/N3 editorial polish — from P3
