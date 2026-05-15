# Phase P2.4 sub-batch 5c (Track M — verify-bundle + convert) — R0 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-v1`
**Scope:** 5c — `43-verify-bundle.md` (NEW, ~340 lines, 17 flags); `44-convert.md` (NEW, ~410 lines, 17 flags); `.cspell.json` (+1 word).

**Verdict:** **ITERATE 3C / 3I / 0N / 1n.**

## Critical

### C-1 — verify-bundle worked-example output schema invented

The chapter showed `ms1[0]: PASS / mk1[0]: PASS / ... / verdict: PASS`. Source (`crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:212-235` + the `VerifyCheck` struct emission) uses the SPEC §5.4 9-element single-sig schema with format `<check>: ok|fail [detail]` and a final `result: ok|mismatch` line. Real names: `ms1_decode`, `ms1_entropy_match`, `mk1_decode`, `mk1_xpub_match`, `mk1_fingerprint_match`, `mk1_path_match`, `md1_decode`, `md1_wallet_policy`, `md1_xpub_match`. Multisig uses the `6N+3` per-cosigner-interleaved variant. Exit code 0 for `ok`, 4 for `mismatch` (per `verify_bundle.rs:235`).

### C-2 — verify-bundle invented a `--bundle-json: stdin not supported` byte-quoted error

The refusals row presented this as if it were the upstream error string. Source has no such string; `verify_bundle.rs:526` calls `std::fs::read_to_string(path)` and a literal `-` becomes a generic OS I/O error. Replaced with neutral framing.

### C-3 — convert `--from xpub --to mk1` recommended a wrong fix

The chapter said the refusal could be unblocked with `--fingerprint` and `--path`. Source (`cmd/convert.rs::refusal_xpub_to_mk1`) refuses outright with byte-exact: `--to mk1 requires a policy descriptor binding (mk1 cards bind xpubs to specific policies via policy_id_stubs). Use 'mnemonic bundle --slot @0.xpub=... --template ...' to emit a complete bundle.` Following the chapter's advice would not unblock.

## Important

### I-1 — convert advisories used "use" instead of byte-exact "pipe via"

`crates/mnemonic-toolkit/src/secret_advisory.rs::secret_in_argv_warning` emits `warning: secret material on argv ({flag}) — pipe via {alternative} to avoid /proc/$PID/cmdline exposure`. The chapter wrote `... — use ...`. Fixed all 4 advisory rows.

### I-2 — verify-bundle worked-example mk1+md1 cardinality framing

The chapter pasted 2 mk1 + 3 md1 strings (correct single-sig multi-string serialization) but described the output as a per-card pass/fail, conflating "lines per card-string" with "lines per check". C-1 fold subsumes this; added a clarifying note about the 9-named-check vs multi-string distinction.

### I-3 — convert `--from` "all 13 nodes accept `=-` for stdin" overbroad

Public nodes accept `=-` syntactically but the secret widget treatment fires only for `is_argv_secret_bearing()` (7 secret-class nodes plus `minikey`). Scoped the prose accordingly.

## Nitpicks

### n-1 — convert refusals table missing `--to address` row

Inline mention existed in the per-flag section but the table did not include the `refusal_address_no_script_type` row. Added.

## Lint state (post-fold)

- Phase 4 schema-coverage RED at **286 missing** (was 403 → -117 = verify-bundle ~44 + convert ~73). No orphans.
- Phase 5 outline-coverage RED at **39 missing** (was 52 → -13 = 5 verify-bundle outlines + 8 convert outlines).
- Phases 1-3 GREEN.
- HTML 19 H1 chapters (was 17 → +2).
- PDF 89 pages (was 64 → +25).

After folds, R1 should LOCK.
