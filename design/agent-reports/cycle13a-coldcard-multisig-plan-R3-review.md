# R0 REVIEW — cycle-13 Lane A PLAN-DOC (coldcard-multisig fidelity H11+H14) — Round 3

Verified against `origin/master = 9b2a8ae3` (HEAD == origin/master this session).

## VERDICT: GREEN — 0 Critical / 0 Important (1 cosmetic Minor, since folded)

The R2 I-1 fold is correct and complete. The lane may proceed to TDD.

### Axis 1 — the two CLI tests genuinely break and are correctly handled: CONFIRMED
`cli_import_wallet_coldcard_multisig.rs:166` `coldcard_ms_xfp_header_divergence_warns_byte_exact_template` feeds `XFP: DEADBEEF` + a bare `xpub6FQya7zGhR92…` (decoded depth byte = 4) and asserts 4 `stderr.contains` Row-2 clauses; `:209` `coldcard_ms_per_cosigner_xfp_divergence_warns_per_cosigner` feeds `CAFEBABE: <same depth-4 xpub>` and asserts 3 clauses. Both `(Some,Some)` at depth>0 → under H14-c go SILENT → asserts FAIL. The plan's prescribed re-point to depth-0 `XPUB_D0_*` (asserts KEPT) is correct, mechanically identical to the `:990`/`:1265` unit re-points.

### Axis 2 — CLI break-set completeness (decisive): CONFIRMED COMPLETE — no third flip
Independent sweep of ALL `tests/*.rs` (grep `coldcard`/`multisig`/`jade`/`XFP:`/`Derivation:`/warning-template/`DEADBEEF`/`CAFEBABE`):
- The xfp-header WARNING template appears in test assertions in exactly ONE file — `cli_import_wallet_coldcard_multisig.rs` `:188/:200/:224` (the two added tests). Nowhere else.
- `DEADBEEF`/`CAFEBABE` divergent literals appear only in those two tests. Every on-disk `coldcard-ms-*.txt`/`jade-*.json` fixture supplies per-line XFPs matching computed → `(Some,Some)` depth>0 → H14-c silent → STAY GREEN (verified 3of5, 2of3-with/no-xfp, jade, cointype1).
- The "CLEAN" enumeration is accurate: jade Row-1 silent; `coldcard_multisig_mainnet_xpub_on_coin_type_1_rejects` supplies matching XFPs → loop passes → reaches network-mismatch check (truth-table loop `:336` refusal precedes network/coin-type at `~:433`/`:438-444`), unlike unit `:1418` (no XFP, refuses first); `cli_wallet_cross_format_convergence.rs` asserts only `.success()` + key-material; format-mismatch-matrix + p0c-dispatch refuse before `parse_text`; also cleared `cli_bundle_import_json.rs` + `cli_import_wallet_jade.rs:127`.

**Complete cross-file break set = 4 unit fixtures (`:990/:1042/:1265/:1418`) + exactly 2 CLI tests (`:166/:209`). No third breaking test.**

### Axis 3 — No new drift: CONFIRMED
Fold purely additive (§P1 H14-h items 5+6, CLI-clean enumeration line 143, §6 CLI-layer line, fold-history). P1 matrix, P2 Q1 arm + #13b, P3 H11-b sorted-slot pairing, P4 heterogeneous canonicalizer, phase ordering (P1→P2→P3→P4 RED-first), unit-layer break set (round-1 reconciliation intact — `:1042` split, `:1418` per-line XFPs), 16-test inventory, §0 scope/SemVer (MINOR v0.66.0) all byte-stable vs R2.

### Minor (folded post-review)
RED #11's inventory note under-counted H14-h's break set as 4; updated to include the 2 CLI tests for doc-internal consistency. (Was non-blocking — #11 cross-references "the complete break set in H14-h" which authoritatively includes items 5+6, and §6 gates the CLI layer.)

## Disposition
GREEN. The lane may proceed to TDD implementation (P1→P2→P3→P4). Ships as part of toolkit MINOR v0.66.0 with Lanes B + C.
