# Phase P2.4 sub-batch 5c — R1 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-v1`
**Scope:** Verify R0 folds (3C/3I/0N/1n) for `43-verify-bundle.md` + `44-convert.md`.

**Verdict:** **LOCK 0C / 0I / 0N / 0n.** All 7 folds byte-verified.

## Per-fold verification

| Fold | Status | Evidence |
|---|---|---|
| **C-1** verify-bundle output schema | PASS | `43-verify-bundle.md:360-370` shows the 10-line SPEC §5.4 schema (`ms1_decode`, `ms1_entropy_match`, `mk1_decode`, `mk1_xpub_match`, `mk1_fingerprint_match`, `mk1_path_match`, `md1_decode`, `md1_wallet_policy`, `md1_xpub_match`, `result: ok`). Exit-code 0/4 cite at lines 381-384 mirrors `verify_bundle.rs:235`. |
| **C-2** bundle-json stdin neutrality | PASS | Refusals row at line 395 neutrally describes I/O fallthrough; no fabricated byte-quoted error string. |
| **C-3** convert xpub→mk1 | PASS | `44-convert.md:217-225` quotes `refusal_xpub_to_mk1` byte-exact + clarifies that `--fingerprint`+`--path` does NOT unblock. Refusals table row 563 also byte-exact. |
| **I-1** advisories pipe via | PASS | All 4 rows in Advisories table use `pipe via` matching `secret_advisory.rs:34-39` byte-exact format; header sentence cites the format string. |
| **I-2** multi-string framing | PASS | `43-verify-bundle.md:372-377` clarifies 2 mk1 + 3 md1 wire format vs 9-named-check schema; `6N+3` multisig variant cited. |
| **I-3** =- claim scoping | PASS | `44-convert.md:57-62` scopes secret-widget treatment to `is_argv_secret_bearing()` per `cmd/convert.rs:107-109`. |
| **n-1** --to address refusal in table | PASS | Refusals table row 564 includes `refusal_address_no_script_type` per `convert.rs:447-451`. |

## Lint + build state

- Phase 4 schema-coverage RED at **286 missing** (unchanged; folds were prose-only).
- Phase 5 outline-coverage RED at **39 missing** (unchanged).
- Phases 1-3 GREEN.
- HTML 19 H1 chapters; PDF 89 pages.

**LOCK — proceed to commit.**
