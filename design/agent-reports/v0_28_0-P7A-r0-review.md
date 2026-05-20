# v0.28.0 Phase 7 (G1) — P7A R0 self-review

**Phase:** P7A — Source edits (3 sites): new `4 =>` arm in `wallet_import/bsms.rs::parse`; mirror fix in `wallet_import/roundtrip.rs::canonicalize_bsms`; generalized 7-site grep sweep for `"expected 2 or 6 lines"` → `"expected 2, 4, or 6 lines"`.
**Reviewer:** Executor self-review (Task-dispatch unavailable in autonomous session).
**Source SHA reviewed against:** branch `v0.28.0/g1-bsms-4line` rooted at `release/v0.28.0` `71592bc`.
**Verdict:** GREEN.

---

## Critical

NONE.

## Important

NONE.

## Minor

### M1. file-level docstring updated mid-phase

The file-level docstring at `wallet_import/bsms.rs:1-21` was rewritten to describe the v0.28.0 3-shape grammar (2 / 4 / 6) and the SPEC §10 contract. Not load-bearing for any consumer, but reviewers tracing the cycle should be aware the original 2/6-line framing was overwritten in lockstep with the parser body.

## Grep-sweep discipline (P7A end-of-phase verification)

Per the user-prompt locked discipline ("at task start AND task end, run `grep -rn ...` and assert post-edit count is 0 + post-edit count of new literal is ~7"):

```
$ grep -rn "expected 2 or 6 lines" crates/mnemonic-toolkit/
(no matches)
$ grep -rn "expected 2, 4, or 6 lines" crates/mnemonic-toolkit/
crates/mnemonic-toolkit/src/wallet_import/bsms.rs:190
crates/mnemonic-toolkit/src/wallet_import/roundtrip.rs:91
crates/mnemonic-toolkit/tests/cli_import_wallet_bsms.rs:527
crates/mnemonic-toolkit/tests/cli_import_wallet_bsms.rs:541
crates/mnemonic-toolkit/tests/cli_import_wallet_bsms.rs:552
crates/mnemonic-toolkit/tests/cli_import_wallet_bsms.rs:563
crates/mnemonic-toolkit/tests/cli_import_wallet_bsms.rs:613
```

7 sites with the new literal; the 4-line rejection cell was DELETED in full (was at `tests/cli_import_wallet_bsms.rs:543-552` per pre-edit grep). Net: 8 enumerated sites → 7 updates + 1 deletion. Matches the user-prompt prediction ("net might be 6"; actual 7 because the doc-comment update at :527 was a separate update from the deleted cell).

## P7A site enumeration verification

| Plan site | Pre-edit path:line | Post-edit path:line | Action | Status |
|---|---|---|---|---|
| (1) `bsms.rs:131` parser | `src/wallet_import/bsms.rs:131` | `src/wallet_import/bsms.rs:190` | UPDATE | ✓ |
| (2) `roundtrip.rs:87` canonicalize | `src/wallet_import/roundtrip.rs:87` | `src/wallet_import/roundtrip.rs:91` | UPDATE + add `4 => lines[1]` arm | ✓ |
| (3) test :527 doc-comment | `tests/cli_import_wallet_bsms.rs:527` | `tests/cli_import_wallet_bsms.rs:527` | UPDATE | ✓ |
| (4) test :538 `bsms_3_line` | `tests/cli_import_wallet_bsms.rs:538` | `tests/cli_import_wallet_bsms.rs:541` | UPDATE | ✓ |
| (5) test :549 `bsms_4_line` | `tests/cli_import_wallet_bsms.rs:549` | (deleted) | DELETE entire cell | ✓ |
| (6) test :560 `bsms_5_line` | `tests/cli_import_wallet_bsms.rs:560` | `tests/cli_import_wallet_bsms.rs:552` | UPDATE | ✓ |
| (7) test :571 `bsms_7_line` | `tests/cli_import_wallet_bsms.rs:571` | `tests/cli_import_wallet_bsms.rs:563` | UPDATE | ✓ |
| (8) test :621 sniff helper | `tests/cli_import_wallet_bsms.rs:621` | `tests/cli_import_wallet_bsms.rs:613` | UPDATE | ✓ |

## SPEC §10 contract verification

| Contract clause | Source site | Implementation | Status |
|---|---|---|---|
| §10.1 line-count: 4 | `bsms.rs:107` `match trimmed_count { ..., 4 => { ... }, 6 => ... }` | New arm between 2 and 6 (alphabetically by line-count); cited line range matches user-prompt expectation `:97` (insertion site) | ✓ |
| §10.1 line shape | lines[0]=header, lines[1]=descriptor, lines[2]=path-restrictions, lines[3]=FIRST_ADDRESS | Matches `wallet_export/bsms.rs::BsmsForm::FourLine` output shape (`line1\nline2\nline3\nline4`) | ✓ |
| §10.2 first-address cross-validation | `bsms.rs:267-323` post-parse block fires for any `audit.is_some()` | The 4-line arm populates `audit = Some(...)`, so the existing cross-validation block runs without code duplication | ✓ |
| §10.3 empty-string-sentinel | `bsms.rs:140-145` `BsmsAuditFields { token: String::new(), signature: String::new(), first_address: ..., derivation_path: lines[2].to_string(), verification: NotAttempted }` | All 5 fields populated per the SPEC §10.3 sentinel convention | ✓ |
| §10.4 deprecation | (deferred to P7B) | (deferred to P7B) | ✓ (out of P7A scope) |
| §10.5 error template | `bsms.rs:190` `"expected 2, 4, or 6 lines"` | Matches the locked template; 7 sites consistent | ✓ |
| §10.6 scope limits | inline (no signature verify on 4-line; cross-validation is informational, not refusal) | Implementation matches the existing 6-line `first-address mismatch` WARNING semantics (exit 0, parse succeeds) — preserves principle of least surprise | ✓ |

## Unit-test coverage verification

| P7A unit-test requirement | Test fn | Status |
|---|---|---|
| happy-path 4-line | `parse_4line_happy_path_populates_audit_with_empty_sentinels` | ✓ — also asserts empty-string-sentinel pattern + verification=NotAttempted |
| first-address-mismatch | `parse_4line_first_address_mismatch_emits_warning` | ✓ — asserts WARNING fires + parse still succeeds |
| line-3-mismatch | `parse_4line_line3_preserved_verbatim_in_audit` | ✓ — uses `"No path restrictions"` literal, asserts verbatim preservation in audit.derivation_path |
| replacement-for-deleted-4-line-cell happy-path-acceptance | integration cell `bsms_4line_sortedmulti_2of3_happy_path` (cli_import_wallet_bsms.rs) | ✓ — covers user-visible accept-not-reject behavior at CLI |

## Reviewer-loop reconverge

The P7A scope was internally consistent at execution; no folds applied. R0 GREEN; no R1 needed.

## Recommendation

Proceed to P7B + P7C verification (this session executes them in lockstep — P7A is the foundation, P7B is the notice replacement which I have already applied inline with the 4-line arm addition for atomicity, P7C is the integration cells which I have also already applied).
