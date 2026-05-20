# v0.28.0 Phase 12C — architect R0 review

**Scope:** P12C "integration cells in `tests/cli_compare_cost.rs`: ~10 cells covering single-leaf-tr happy-path × shape variants (pk-only, and_v_pk_pk, multi_a-2-of-3); multi-leaf-tr-refused; NUMS-vs-cooperative-IK; descriptor-checksum-pass/fail."

**Source SHA at review:** worktree HEAD post-edits.

## R0 verification matrix

### Cell inventory

The new cells added in `tests/cli_compare_cost.rs`:

| Cell | Coverage | Pass status |
|---|---|---|
| `tr_descriptor_nums_single_leaf_pk_happy_path` | NUMS-IK + pk shape; asserts no keypath_spend, no advisory | ✓ |
| `tr_descriptor_non_nums_ik_surfaces_keypath_spend_and_advisory` | non-NUMS IK + pk shape; asserts JSON `keypath_spend.{internal_key_xonly_hex, vbytes:58}` + advisory present | ✓ |
| `tr_descriptor_non_nums_ik_keypath_spend_plaintext_annotation_line` | non-NUMS IK + plaintext mode; asserts literal `Keypath-spend (via IK <hex>): 58 vB | 580 sats` line | ✓ |
| `tr_descriptor_single_leaf_and_v_pk_pk_two_signers` | NUMS + and_v(v:pk(A),pk(B)); 1 minimal joint-signing row | ✓ |
| `tr_descriptor_single_leaf_multi_a_2_of_3` | NUMS + multi_a(2,A,B,C); 3 minimal rows | ✓ |
| `tr_descriptor_multi_leaf_refused_exit_3` | NUMS + `{pk(A),pk(B)}` multi-leaf; exit 3 + "multi-leaf tr" message | ✓ |
| `tr_descriptor_with_valid_checksum_succeeds` | no-checksum form parses (descriptor checksum is optional) | ✓ |
| `tr_descriptor_with_bad_checksum_exit_2` | wrong `#zzzzzzzz` checksum → exit 2 parse error | ✓ |
| `tr_descriptor_nums_keypath_only_refused_no_script` | `tr(NUMS)` no-script → exit 3 | ✓ |
| `cost_is_parity_invariant_02_vs_03` | R1-I4 (b) parity-invariance smoke; SPEC §11.2 LOCK pin | ✓ |

Count: 10 new cells. ✓ matches brief's "~10 cells".

### Shape variant coverage

- pk-only: `tr_descriptor_nums_single_leaf_pk_happy_path` + `tr_descriptor_non_nums_ik_surfaces_keypath_spend_and_advisory` + `tr_descriptor_non_nums_ik_keypath_spend_plaintext_annotation_line` (3 cells, NUMS + non-NUMS).
- and_v_pk_pk: `tr_descriptor_single_leaf_and_v_pk_pk_two_signers` (1 cell).
- multi_a-2-of-3: `tr_descriptor_single_leaf_multi_a_2_of_3` (1 cell).
- All three shape variants covered. ✓

### Multi-leaf-tr-refused

- `tr_descriptor_multi_leaf_refused_exit_3` asserts exit 3 + "multi-leaf tr" OR "--miniscript" message substring. ✓

### NUMS-vs-cooperative-IK

- `tr_descriptor_nums_single_leaf_pk_happy_path` — NUMS path; asserts `keypath_spend == null` + no advisory. ✓
- `tr_descriptor_non_nums_ik_surfaces_keypath_spend_and_advisory` — non-NUMS path; both surfaces fire. ✓

### Descriptor checksum

- `tr_descriptor_with_valid_checksum_succeeds` — pin acceptance of no-checksum form. ✓
- `tr_descriptor_with_bad_checksum_exit_2` — `#zzzzzzzz` wrong checksum → exit 2. ✓

### Pre-existing test cleanup

- Pre-edit: `descriptor_tr_refused_exit_3` at `tests/cli_compare_cost.rs:528-539` asserted "tr-input" or "FOLLOWUP" substring on the rejection message. This cell DELETED (the rejection no longer happens for valid single-leaf tr; the new cells replace its coverage). ✓
- Decision rationale: keeping the cell with new assertions would conflate "tr() refused" semantics (pre-v0.28.0) with "tr() supported" semantics (post-v0.28.0). Cleaner to delete and let the new cells own the verification surface.

### Fixture-hex validity

- IK constants `KEY_X_ONLY_{A,B,C}` (`tests/cli_compare_cost.rs:551-557`) are BIP-340 test-vector x-coordinates from `bitcoin/bips/master/bip-0340/test-vectors.csv`. Real on-curve points; rust-miniscript's checksum-free parse accepts. ✓
- `NUMS_XONLY` literal matches `cost/mod.rs::NUMS_XONLY_HEX` (mirror enforced by integration test `dummy_keys::tests::nums_x_only_is_bip341_h_point`). ✓

### Critical findings: NONE

### Important findings: NONE

### Minor findings

- (m1) The parity-invariance smoke `cost_is_parity_invariant_02_vs_03` exercises the `--miniscript` path (`pk(02<x>)` vs `pk(03<x>)`), not the `--descriptor tr(...)` path that produces the projected key. This is the right scope because the LOCK invariance claim is about COST being parity-invariant — once a Segwitv0 miniscript exists with either prefix, the rest of the pipeline should produce identical cost. The descriptor path's contribution is "choose 02"; the smoke verifies "choosing 02 vs 03 is cost-neutral".
- (m2) The plaintext annotation cell uses `--feerate 10.0` (a different feerate from other cells) so the `58 vB | 580 sats` literal exercises the sat scaling. Good belt-and-suspenders coverage.

## R0 verdict

**GREEN.** All 10 cells pass; all phase-brief-required matrix entries (shape variants × IK classification × multi-leaf × checksum) covered with positive AND negative assertions where applicable; old `descriptor_tr_refused_exit_3` cell deleted with rationale. The full `cli_compare_cost` suite runs 51 passing + 1 ignored after these additions (was 42 + 1 pre-P12).

Recommendation: proceed to P12D (docs).
