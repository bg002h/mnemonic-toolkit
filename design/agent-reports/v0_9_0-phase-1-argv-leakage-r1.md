# v0.9.0 Phase 1 — argv-leakage closure R1

**Reviewer:** Opus 4.7 via `feature-dev:code-reviewer` agent, 2026-05-13.
**Branch:** `v0_9_0-phase-1-argv-leakage`.
**Pre-review HEAD:** `aed4559`.
**Master (Phase 0 close):** `889767e`.

## Verdict

**0 Critical / 1 Important / 2 Notable.**

Cycle-A impl is sound; advisory wording is locked; stdin-mutex
invariants hold at every call-site; lint anchor is present and
exercised; the stderr-tail migration preserves byte-exactness where
SPEC §6.6 mandates it. The one Important finding is a CLAUDE.md
mirror-invariant breach on the manual; folding it before Phase 1
close keeps Phase E clean.

## Important findings

### I-1 — manual flag tables not updated in lockstep (conf 95)

**Files:** `docs/manual/src/40-cli-reference/41-mnemonic.md`.

**Mandate:** CLAUDE.md mirror invariant — *"any flag/API addition or
removal in this repo's CLI surface … must update the manual under
`docs/manual/src/40-cli-reference/` **in lockstep with the
implementing PR**"*. Bidirectional flag-coverage lint at
`docs/manual/tests/lint.sh:62-98` greps every `--flag` from
`<cmd> --help` against the chapter; missing entries fail the lint.

**Missing rows (4 new clap flags + 2 `--slot` `=-` carve-out notes):**

- `bundle` table — missing `--passphrase-stdin`; `--slot` row should
  note `=-` for secret-bearing subkeys.
- `verify-bundle` table — missing `--passphrase-stdin`; same `--slot`
  `=-` note.
- `derive-child` table — missing `--passphrase-stdin`.
- `convert` table — missing `--bip38-passphrase-stdin`.

**Why `cargo test --workspace` didn't catch:** manual lint lives at
`docs/manual/tests/lint.sh`, invoked only via `make -C docs/manual
lint`. CI workflow at `.github/workflows/manual.yml` runs it at tag
time; Phase E would block on it.

**Fix:** add 4 stdin-flag rows + 2 `--slot` `=-` notes. Run
`make -C docs/manual lint` to confirm.

## Notable findings

### N-1 — `tests::warning_byte_exact_for_slot_flag` is not byte-exact (conf 35)

`secret_advisory.rs:48-55` asserts `starts_with` + `contains` +
`ends_with` rather than full equality. The simple-flag test already
byte-exact-locks the wording; the slot variant rebuilds the same
template via `format!`. Polish: rename to
`warning_shape_for_slot_flag` to reflect the actual assertion
strictness.

### N-2 — `from_node_is_argv_secret` duplicates `is_secret_bearing` minus MiniKey (conf 45)

`convert.rs:1355-1367` defines a local free fn that mirrors
`NodeType::is_secret_bearing` except MiniKey is included. Doc-comment
explains the gap (widening the existing method would entrain the
existing `secret-on-stdout` redaction, out of scope). Polish: lift
to a `NodeType::is_argv_secret_bearing` method so both predicates
live on the type.

## Per-scope confirmation (no findings)

**(A) GREEN-impl correctness.** Advisory emit sites correctly
iterate per-occurrence with no double-emissions. `apply_slot_stdin`
enforces ≤ 1 stdin-sentinel + NULL-byte preservation via the
`\r?\n` strip precedent. `args.clone() → synthetic_args` pattern
correctly threads through bundle/verify-bundle subsequent dispatch
(incl. verify-bundle's bundle-json intake). Triple-stdin-mutex in
convert covers all three pairwise combinations
(`--passphrase-stdin` × `--bip38-passphrase-stdin` × `--from =-`).
xprv-runtime-reject ordering is locked by
`cli_secret_in_argv_warning` cell 4.

**(B) `secret_advisory.rs`.** Surface is minimal; byte-exact wording
pinned; mirrors `secret-on-stdout` precedent (`bundle.rs:697`).

**(C) xprv-slot structural lint anchor.** `slot_stdin` evidence
present at `bundle.rs:1209,1229` and `verify_bundle.rs:568,588`.

**(D) Test-fixture migration.** 34 sites migrated from `assert_eq!`
to `assert!(...ends_with...)`; SPEC §6.6 tail-byte-exactness
preserved; advisory wording independently locked.

**(E) Clippy baseline fold.** 5 BIP-85 precedence fixes are
semantically identical (guarded by spec-vector tests). 2x
`if_same_then_else` collapse is semantically lossless. Dead-code
allows are at-site (not crate-wide).

**(F) SPEC §1 item 1 closure.** All 9 flag-rows have stdin routes;
advisory fires at every inline site; lint enumerates 20 canonical
rows. SPEC §6 gate 3 satisfied. libc-OsString + argv-overwrite
remain explicitly OOS per SPEC §3.

## Disposition

**Required fold (before Phase 1 close):** I-1.

**Opportunistic folds (this commit or next):** N-1, N-2.

**Post-fold:** R2 verification (Sonnet or Opus, single round) to
confirm 0C/0I; mark Phase 1 closed; advance to Phase 2 (Zeroizing
wrappers).
