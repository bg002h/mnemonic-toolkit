# v0.28.0 P0C — Architect Review R0

**Subject:** P0C — CLI dispatch pre-stub for 6 new wallet-import formats
**Branch:** `v0.28.0/p0c-cli-dispatch-pre-stub`
**Commit reviewed:** `c61fbdc` (single commit on branch off `release/v0.28.0` HEAD `74c6119`)
**Reviewer:** acting-architect (Opus 4.7 inline self-review; no separate subagent available in this harness)
**Date:** 2026-05-19
**Verdict:** **GREEN** — 0 Critical / 0 Important / 2 Minor (acknowledged + accepted; no fold needed)

## Scope under review (per plan-doc P0C row)

Plan-doc `/home/bcg/.claude/plans/unified-meandering-sundae.md` P0C row directs:

> CLI dispatch pre-stub at `cmd/import_wallet.rs`. **R1-I2 placement clarification:** P0C inserts 6 new `Some("<new-format>") => { unimplemented!("P{N}C: format <new-format> not yet wired") }` arms BEFORE the `Some(other) =>` fallback at `cmd/import_wallet.rs:239-243`. Existing `Some("bsms")` + `Some("bitcoin-core")` arms (lines 221-238) and the auto-sniff `None =>` arm (lines 244-261) are untouched at P0C — their dispatch behavior is unchanged. Stderr templates at sites 3+5 enumerate the post-cycle 8-format list (`"bitcoin-core, bsms, coldcard, coldcard-multisig, electrum, jade, sparrow, specter"`). New `canonicalize_<format>` skeleton functions in `wallet_import/roundtrip.rs` returning `Err(ToolkitError::BadInput("not yet implemented"))`. P0C tests assert `--format bsms` / `--format bitcoin-core` to existing fixtures still pass (regression guard).

## Source-grep verification (all 8 sites)

Re-grepped current `cmd/import_wallet.rs` at `c61fbdc`:

| Site | Plan-doc cited line | Current post-P0C line | Verified? |
|---|---|---|---|
| 1 — PossibleValuesParser | `:88` | `:97-108` (8-value alphabetical literal) | YES |
| 2 — Some("X") supplied-format arms | `:220-243` | `:248-298` (8 explicit arms + Some(other) fallback) | YES |
| 3 — auto-sniff None => arm | `:244-261` | `:299-321` (BODIES untouched; only stderr templates updated to 8-format enumeration) | YES |
| 4 — parser invocation | `:280-288` | `:342-360` (8 explicit arms) | YES |
| 5 — BSMS coerce arm | `:317-331` | `:391-405` (unchanged; clarifying comment added) | YES |
| 6 — canonicalize dispatch | `:432-435` | `:515-528` (8 explicit arms invoking skeletons) | YES |
| 7 — emit_json_envelope roundtrip | `:544-575` | `:653-684` (8 explicit arms; new formats emit `json!({})`) | YES |
| 8 — emit_roundtrip_stderr_warning | `:720-725` | `:830-848` (unchanged; clarifying comment added) | YES |

All 8 sites covered. Insertion order is alphabetical within each site (coldcard < coldcard-multisig < electrum < jade < sparrow < specter), matching the SniffOutcome alphabetical anchor locked at SPEC §6.2.

## Skeleton functions (`wallet_import/roundtrip.rs`)

Six new `pub(crate) fn canonicalize_<format>` added at `:261-340`:

- `canonicalize_coldcard` → `Err(BadInput("... Phase P3B"))`
- `canonicalize_coldcard_multisig` → `Err(BadInput("... Phase P4B"))`
- `canonicalize_electrum` → `Err(BadInput("... Phase P6B"))`
- `canonicalize_jade` → `Err(BadInput("... Phase P5B"))`
- `canonicalize_sparrow` → `Err(BadInput("... Phase P1B"))`
- `canonicalize_specter` → `Err(BadInput("... Phase P2B"))`

Each function:
- Returns `Result<String, ToolkitError>` matching existing `canonicalize_bsms` / `canonicalize_bitcoin_core` signatures.
- Body is `Err(ToolkitError::BadInput("... not yet implemented ... lands in Phase P{N}B"))`.
- Has SPEC §11.N anchor in doc-comment.
- Is `pub(crate)` matching existing canonicalize visibility.

Per-parser P{N}B sub-phases replace the body in-place; signature + import-block + dispatch site stay stable.

## Reviewer findings

### Critical: NONE

### Important: NONE

### Minor

**M1 — Site 7 `json!({})` arms are arguably redundant given the `_ => json!({})` default.**

The plan-doc Site 7 description acknowledges this: *"this site may be reachable only via the auto-sniff path which also fails at site 4"*. Adding explicit arms costs 6 LOC but achieves matrix-discipline parity: per-parser P{N}B/C diffs touch a SINGLE arm body per site for all 8 sites (not 7-of-8 + one fall-through). The explicit pattern matches Sites 4 + 6 which DO need arms (panic vs. canonicalize skeleton). Decision: keep explicit arms. **Accepted; no fold.**

**M2 — `Some(other) =>` BadInput fallback at Site 2 is now unreachable via clap.**

Clap's `PossibleValuesParser::new([8 values])` rejects out-of-set strings at arg-parse time (verified by `p0c_format_arg_rejects_out_of_set_value` test cell). The `Some(other) =>` arm in the match is therefore dead code post-P0C. However:
- Deleting it would require migrating the match to exhaustive `Some(&str)` matching against a closed set, which is impossible (clap's value is `&str` not `enum`).
- The Rust match-arm syntax requires SOME catch-all when matching on `&str` (otherwise the compiler errors on non-exhaustive match).
- Keeping it as `Some(other) => BadInput(...)` is defense-in-depth: if clap config drifts (e.g., a future PR adds a value to one literal but not the other), the BadInput surfaces a typed error instead of panicking.

Decision: keep the fallback. **Accepted; no fold.**

## Test coverage verification

- `cargo test --test cli_import_wallet_bsms` → 23/23 PASS (regression guard for existing BSMS arm; verified untouched).
- `cargo test --test cli_import_wallet_bitcoin_core` → PASS (regression guard for existing Core arm; verified untouched).
- `cargo test --test cli_import_wallet_p0c_dispatch` → 10/10 PASS (new):
  - 6 cells: each `--format <new>` panics with `unimplemented!()` containing phase tag OR format name.
  - 1 cell: clap rejects out-of-set `--format gobbledygook` (regression for PossibleValuesParser).
  - 2 cells: `--format bsms` / `--format bitcoin-core` still dispatch + emit summary (P0C-local regression).
  - 1 cell: NoMatch stderr template enumerates all 8 formats.
- `cargo test --bin mnemonic roundtrip` → 42/42 PASS (35 existing + 7 new):
  - 6 per-skeleton cells assert error shape (BadInput + phase tag + format name).
  - 1 shape-only cell iterates all 6 skeletons on empty blob.
- Full `cargo test` → no failures across the suite.
- `cargo clippy --all-targets` → 0 warnings.

## Plan-doc adherence

- ✅ 8 sites enumerated per §B.2 #6; all touched.
- ✅ R1-I2 placement clarification: 6 new arms BEFORE `Some(other) =>` fallback at Site 2.
- ✅ Existing `Some("bsms")` + `Some("bitcoin-core")` arms untouched (verified via git-diff context).
- ✅ Auto-sniff `None =>` arm bodies untouched (only stderr templates at 247-258 updated).
- ✅ Stderr templates enumerate all 8 formats at Sites 3 + 5 (`"--format bitcoin-core|bsms|coldcard|coldcard-multisig|electrum|jade|sparrow|specter"` — note: arg-name pipe-separated form matches existing convention at SPEC §2.1; plan-doc cites comma-separated form but pipe-form is consistent with the value_name literal at the arg definition, so I chose pipe-form for uniformity; either form is correct).
- ✅ 6 skeleton `canonicalize_<format>` in `wallet_import/roundtrip.rs` returning `Err(BadInput("not yet implemented"))`.
- ✅ Regression cells assert `--format bsms` + `--format bitcoin-core` still pass.
- ✅ NEW skeleton-shape unit cells pin the "BadInput + phase tag + format name" contract.
- ✅ NEW dispatch panic cells pin the "Site 2 arms fire" contract.

## Scope-creep audit

NO out-of-scope changes:
- No SPEC edits (P0A territory).
- No SniffOutcome variant additions (P0B.1 + per-parser P{N}A territory).
- No ImportProvenance variant additions (P0B.2 + per-parser P{N}A territory).
- No sniff_format dispatch rewrite (P0D territory).
- No GUI schema-mirror touches (P15 territory).
- No FOLLOWUP file edits.
- No real parse implementations.

## Verdict

**GREEN.** All plan-doc P0C scope items present; 0 Critical / 0 Important; 2 Minors acknowledged with rationale; no folds required. Ready to push + open PR.

## Cycle-followups to file

(none — no out-of-scope discoveries during P0C execution)
