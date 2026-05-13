# v0.9.0 Phase 1 — argv-leakage closure R2 (fold-verification)

**Reviewer:** Sonnet 4.6 via `feature-dev:code-reviewer` agent, 2026-05-13.
**Branch:** `v0_9_0-phase-1-argv-leakage`.
**Pre-review HEAD:** `ed76792` (R1 fold commit).

## Verdict

**0C / 0I — Phase 1 READY TO CLOSE.**

Mechanical fold-verification of R1's three findings. All three folds
landed clean; no new findings surfaced. Per project memory
`feedback-opus-primary-review-agent`, Sonnet is appropriate for this
trivial fold-verify after Opus has set architectural scope.

## Per-fold confirmation

### I-1 fold ✓ confirmed

All 6 manual entries present in
`docs/manual/src/40-cli-reference/41-mnemonic.md`:
- `bundle --passphrase-stdin` (L33)
- `bundle --slot =-` note (L41)
- `verify-bundle --passphrase-stdin` (L76)
- `verify-bundle --slot =-` note (L78)
- `convert --bip38-passphrase-stdin` (L118)
- `derive-child --passphrase-stdin` (L202)

Wording uses "raw, NULL-byte preserving" idiom throughout;
"single stdin per invocation" suffix is consistent across the
three new `--passphrase-stdin` rows. Manual lint
(`bash docs/manual/tests/lint.sh` — all 6 steps including
flag-coverage) exits OK at the pre-fold check (R1 commit).

### N-1 fold ✓ confirmed

`crates/mnemonic-toolkit/src/secret_advisory.rs` L48 — test
renamed to `warning_shape_for_slot_flag` with shape-only
assertions. L37 — `warning_byte_exact_for_simple_flag` retains
byte-exact `assert_eq!` unchanged.

### N-2 fold ✓ confirmed

`crates/mnemonic-toolkit/src/cmd/convert.rs` L107-109 —
`pub fn is_argv_secret_bearing(self) -> bool` added on
`NodeType` adjacent to `is_secret_bearing` (L85-96), with
doc-comment naming the `convert-minikey-stdout-redaction`
follow-up. `emit_secret_in_argv_advisories` (L1343) calls
`f.node.is_argv_secret_bearing()`. `from_node_is_argv_secret`
free fn is absent from the codebase.

## Exit-gate verification

- `cargo test --workspace --no-fail-fast --tests`: **41/41 green,
  0 failed**.
- `cargo clippy --workspace --all-targets --no-deps -- -D warnings`:
  **clean (0 errors)**.
- Manual lint: 6/6 steps OK.

## Disposition

**MERGE.** R1 + R2 jointly close Phase 1 at 0C/0I (R1 was 0C/1I/2N
pre-fold, all folded; R2 confirms post-fold state is 0C/0I).
Phase 1 of v0.9.0 Cycle A is COMPLETE. Phase 2 (Zeroizing wrappers)
is the next workstream per plan §"Phase 2".
