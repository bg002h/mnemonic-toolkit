# v0.28.0 Phase 12D — architect R0 review

**Scope:** P12D "NEW `design/SPEC_compare_cost_v0_28_0.md` (CREATE-NEW per R0 I5 lock; supersedes v0_26_0). §11 carries single-leaf tr() spec + BIP-340 lift-x lock + cost-domain parity-invariance claim. Manual chapter update at `docs/manual/src/40-cli-reference/41-mnemonic.md:2455` — add tr() worked example."

**Source SHA at review:** worktree HEAD post-edits.

## R0 verification matrix

### SPEC v0.28.0 doc

- File at `design/SPEC_compare_cost_v0_28_0.md` — CREATE-NEW per R0 I5 lock. ✓ Does not overwrite v0_26_0; supersedes-relationship documented in the file's header (`**Supersedes**: [SPEC_compare_cost_v0_26_0.md]...`).
- §11.1 acceptance matrix lists all 4 shapes (NUMS+script, non-NUMS+script, NUMS-keypath-only, multi-leaf) with the implemented outcome for each. ✓
- §11.2 BIP-340 lift-x LOCK documented; cost-domain parity-invariance claim explicitly stated and cross-referenced to the pinning test. ✓
- §11.3 keypath-spend cost surface: bit-exact 66-byte witness + (164+66+3)/4=58 vbytes formula documented; JSON `keypath_spend` field shape + plaintext annotation line + `notes[]` advisory all enumerated. ✓
- §11.4 error variants: documents the `#[allow(dead_code)]` removal for `UnsupportedWrapper` + `MultiLeafTr` per R0 I13 lock. ✓
- §11.5 test scope: lists all 12 cells across unit (`cost::strip::tests`) + integration (`tests/cli_compare_cost.rs`). ✓
- §11.6 manual-update mention. ✓

### Manual chapter update

- File at `docs/manual/src/40-cli-reference/41-mnemonic.md`, anchor `## mnemonic compare-cost` at line 2455. ✓ (R0 I14 corrected location confirmed; not at the bygone `:2453` figure pre-update.)
- Flag table row updated (line 2485): `--descriptor` description now says "wsh(M), sh(wsh(M)), or single-leaf tr(IK, {M}) (v0.28.0)" + multi-leaf/keypath-only refusal note. ✓
- Notes-catalog row updated (line ~2623): old "(v0.27+ FOLLOWUP)..." replaced with "(v0.28.0)..." carrying the full advisory + JSON-field shape + plaintext-annotation-line spec. ✓
- Exit-codes table updated (line ~2633): old "unsupported wrapper (pkh, wpkh, bare, tr(...) deferred)" split into:
  - "unsupported wrapper (pkh, wpkh, bare, keypath-only `tr(IK)` with no script-tree)"
  - "multi-leaf `tr(IK, {M1, M2, ...})` (one-leaf-at-a-time via `--miniscript`)"
  Both → exit 3. ✓
- NEW worked example **4** (after example 3 at the descriptor-via-stdin/JSON case): single-leaf `tr(IK, {M})` with a non-NUMS IK. The example's stdout is **byte-verified** against an actual `cargo run --bin mnemonic -- compare-cost --descriptor ... --feerate 25.0` invocation; the keypath-spend annotation line + advisory note match the captured output exactly. ✓ (Per `[[feedback-architect-must-run-prose-commands]]` discipline.)

### Cross-doc consistency

- Canonical advisory wording lives in `cost/mod.rs:189-194` (the `format!()` at runtime). Manual quotes that string literally; SPEC §11.3 paraphrases. No drift risk because the literal in the manual example was captured from a live binary run, not paraphrased.
- The vbytes figure `58` appears in 4 places: (1) source `KEYPATH_SPEND_WITNESS_BYTES=66` + `(164+66+3)/4=58` formula in `cost/mod.rs:204` + the SPEC; (2) SPEC §11.3; (3) manual notes-catalog; (4) manual worked-example #4 captured output. All consistent. ✓

### Critical findings: NONE

### Important findings: NONE

### Minor findings

- (m1) The CI manual lint at `.github/workflows/manual.yml` (`make lint`) runs `flag-coverage` which extracts CLI flags from `--help` and cross-checks against documented flags. Since P12 adds no NEW flags (only new accepted values for `--descriptor`), `flag-coverage` should pass unchanged. Cannot verify locally because the sandbox blocks running the built `mnemonic` binary directly; CI will catch.
- (m2) `manual.yml` will rebuild the PDF / mdBook on tag. The new worked example 4 uses code-block syntax that mdBook handles natively (no special directives). Acceptable.
- (m3) The SPEC v0.28.0 cites worktree SHA `c460eda` (worktree-base) + `33ec61d` (release/v0.28.0 HEAD at write time). The release branch may advance further; SPEC citations are time-stamped snapshots per the CLAUDE.md grep-verify-at-write-time discipline.

## R0 verdict

**GREEN.** New SPEC doc created (does NOT modify v0_26_0); manual chapter updates are surgical (flag table + notes catalog + exit codes table + new example 4); worked example output is byte-verified against the live binary; all cross-doc references (canonical wording, vbytes figure, IK-classification rules) are consistent.

Cycle ready for commit + PR per phase brief ("Open PR to release/v0.28.0; do NOT self-merge").
