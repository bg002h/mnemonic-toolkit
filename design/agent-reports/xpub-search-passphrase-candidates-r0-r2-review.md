# R0 Architect Re-Review (round 2) — SPEC_xpub_search_passphrase_candidates_file.md

**Reviewer:** opus `feature-dev:code-reviewer`. **Date:** 2026-06-05. **Branch:** `xpub-search-passphrase-candidates-file` (master `26ce377`).
**Verdict:** **0 Critical / 1 Important — RED.** (3 Minors.) The 3 round-1 Importants (I1/I2/I3) all landed correctly + source-verified; the single Important is a fold-introduced cross-section drift. Re-dispatch after fold.

> Persisted verbatim per CLAUDE.md before folding.

---

### Fold landing — all three round-1 folds verified correct
- **I1 (dispatch) — LANDED.** Inline resolve confirmed `passphrase_of_xpub.rs:260-289`, `else→BadInput` at `:282-289` = candidates mode. "Branch at top, skip `:260-318`" is correct + complete: shared seed resolution (`resolve_seed`, `:252`) + the unconditional STDERR_ADVISORY (`:249`) sit OUTSIDE the skipped range; `run_candidate_scan` reuses the resolved mnemonic + re-runs target parse (`:294-296`) + `build_candidate_paths` (`:304-311`). No shared setup dropped.
- **I2 (count/message) — LANDED.** New `XpubSearchPassphraseCandidatesExhausted { candidates_tried }` consistent with `error.rs` (`#[non_exhaustive]`, alphabetical; sorts right after `XpubSearchNoMatch` `:328`; exit_code/kind/message arms compiler-forced). Exit 4 correct. The SPEC's "mirror existing run() `--json` no-match" is grounded: `:365-384` prints the NoMatch envelope FIRST (`--json`), THEN returns `Err(XpubSearchNoMatch)`. `searched_count` (paths) vs `candidates_tried` distinct + non-confusing.
- **I3 (secret class) — LANDED, airtight.** `--decrypt-password-file`/`--secret-file` non-secret in `secrets.rs` (test `:106,:127`). `lint_argv_secret_flags.rs::flag_axis_set_equals_gui_schema` (`:185-199`) set-equals over `secret==true && kind!="boolean"` only — a `secret:false` path flag never enters → no Route. Not extending `flag_is_secret` correct.

### Residual checks (clear)
- `cli_help_fixtures.rs:34-38` asserts only `btcrecover`+URL+`2026-05-25` (all retained) → §5 stays green.
- `cli_gui_schema.rs:107` is a subcommand-NAME freeze (29 subcommands); a flag-add to an existing subcommand adds no subcommand → green. `choices.len()` (`:182,:294`) are dropdown checks, untouched.
- `Cargo.toml` 0.45.0 → §8 v0.46.0 correct.

## Critical
None.

## Important

**I-A (§7 line 88 + §1 line 12 — fold-introduced variant-name drift). Confidence 90.** §7's "miss" cell still reads *"exit 4 `XpubSearchNoMatch`, `candidates_tried == #non-blank lines`"* — contradicts the I2 fold (§3:53, §4:68 now use the NEW `XpubSearchPassphraseCandidatesExhausted{candidates_tried}` and state the exit path is NOT `XpubSearchNoMatch`; also `candidates_tried` is a field on the NEW variant only — `XpubSearchNoMatch` carries `{mode, searched}`). §1 line 12 echoes the same stale "`XpubSearchNoMatch`/4". This is the after-every-fold cross-section drift the round-2 gate exists to catch.
**Fix:** §7:88 → *"exit 4 `XpubSearchPassphraseCandidatesExhausted`, `candidates_tried == #non-blank lines` (`--json` no-match envelope carries `candidates_tried`)."* §1:12 → same variant name. (§7:94 empty-file cell is fine — maps to the `candidates_tried==0` tailored note.)

## Minor (fold inline)
- **M-1 (§4 serde).** Scan `--json` no-match reuses shared `PassphraseOfXpubResult::NoMatch` (`:189-197`); adding `candidates_tried: Option<usize>` WITHOUT `#[serde(skip_serializing_if="Option::is_none")]` emits `candidates_tried: null` on the single-`--passphrase` no-match path too (harmless additive change, but state the `skip_serializing_if` choice so Phase 2 doesn't silently alter the single-passphrase envelope).
- **M-2 (§2 ArgGroup pick).** R0 should make the call: use clap **`ArgGroup`** (`required=true, multiple=false`) over the 3 sources AND remove the now-redundant per-field `conflicts_with`/`required_unless_present` on `--passphrase`/`--passphrase-stdin` (`:80-81,:89-90`) to avoid double-validation.
- **M-3 (§1 echo).** §1 line 12 stale name — fix in lockstep with I-A.

## VERDICT: 0 Critical / 1 Important — RED.
One mechanical text reconcile (I-A/M-3 stale variant name) + 2 minor SPEC clarifications (M-1 serde, M-2 ArgGroup pick). The 3 round-1 Importants are correctly landed. Fold → re-dispatch → expect GREEN.

---

## Fold note (applied after persisting)
- **I-A + M-3 — FOLDED:** §7:88 + §1:12 → `XpubSearchPassphraseCandidatesExhausted`.
- **M-1 — FOLDED:** §4 states `#[serde(skip_serializing_if = "Option::is_none")]` on the new optional fields (single-passphrase envelope unchanged).
- **M-2 — FOLDED:** §2 picks clap `ArgGroup(required, single)` + removes the redundant pairwise `conflicts_with`/`required_unless_present`.
- Re-dispatched R0-r3.
