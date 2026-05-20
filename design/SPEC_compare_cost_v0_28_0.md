# Plan: `compare-cost` v0.28.0 — single-leaf `tr(IK, {M})` input support

**Target release**: `mnemonic-toolkit-v0.28.0` (toolkit-only — no GUI lockstep this cycle).
**Supersedes**: [`SPEC_compare_cost_v0_26_0.md`](./SPEC_compare_cost_v0_26_0.md) (v0.26.0 baseline — all §1–§10 inherited verbatim; this document only adds §11).
**Branch**: `v0.28.0/i-compare-cost-tr`.
**Source SHA at write time**: `c460eda` (worktree base) / `33ec61d` (`release/v0.28.0` HEAD at time of write — citations re-verified against worktree HEAD).
**Closes FOLLOWUP**: `compare-cost-single-leaf-tr-input` (filed v0.27.0 cycle close; cited at `cost/strip.rs:5,51` + `cost/mod.rs:75` pre-fold).

---

## Context

v0.26.0's `compare-cost` shipped with `--descriptor tr(...)` deferred (exit 3
+ "tr-input deferred" message pointing users at the FOLLOWUP slug). v0.28.0
P12 closes that gap: single-leaf `tr(IK, {M})` is now a first-class input
form. Multi-leaf `tr(IK, {M1, M2, ...})` remains rejected (one-leaf-at-a-
time via `--miniscript`).

This document carries the SPEC §-anchor for the new behavior. All other SPEC
sections (§1–§10) are inherited verbatim from `SPEC_compare_cost_v0_26_0.md`
— the wire-shape of the JSON envelope changes additively (new
`keypath_spend` field; same `schema_version: 1` since the change is purely
additive and backward-compatible per the JSON-envelope evolution policy
established in `inspect-json-schema-version-backfill` FOLLOWUP).

---

## §11 — Single-leaf `tr(IK, {M})` input (v0.28.0)

### §11.1 — Surface change

`--descriptor tr(<internal-key>, <single-leaf-script>)` is accepted. The
internal key may be the BIP-341 NUMS H-point or any other valid x-only
public key.

Acceptance matrix:

| Input shape                          | Outcome                                                                    |
| ------------------------------------ | -------------------------------------------------------------------------- |
| `tr(NUMS, {M})`                      | Accepted. Comparison is `wsh(M) vs tr(NUMS, {M})` (unchanged).             |
| `tr(IK, {M})` (IK ≠ NUMS)            | Accepted. Comparison is `wsh(M) vs tr(NUMS, {M})` (script-path-only); keypath-spend cost via IK surfaces separately. |
| `tr(IK)` (keypath-only, no script)   | **Refused** with `UnsupportedWrapper` (exit 3); message points at `--miniscript` for the script-path case. |
| `tr(IK, {M1, M2, ...})` (multi-leaf) | **Refused** with `MultiLeafTr` (exit 3); message points at `--miniscript` for one-leaf-at-a-time. |

### §11.2 — Reverse projection Tap → Segwitv0 (lift-x even-y LOCK)

To run the wsh-vs-tr comparison, the inner miniscript `M` must be parseable
in both Segwitv0 and Tap contexts. v0.26.0's `--miniscript` path projects
Segwitv0 → Tap via "drop the parity byte" (compressed 33B → x-only 32B); see
`cost/strip.rs::segv0_string_to_tap_string`. v0.28.0 needs the reverse:
Tap → Segwitv0 for `--descriptor tr(...)` inputs.

**The reverse projection is not canonically defined** — an x-only key has
two valid compressed forms (`02<x>` and `03<x>`), corresponding to the two
possible y-parities. We **LOCK** the choice to `02` (BIP-340 lift-x even-y
convention).

**Cost-domain parity-invariance claim**: the wsh/tr vbyte counts produced
by `compare-cost` are invariant under `02 ↔ 03` substitution on the
projected key, because miniscript witness-size is signature-shape-only
(the prefix byte does not appear in the witness — only the signature, which
is the same size whether the public key has even or odd y). This is pinned
by the parity-invariance smoke test
(`tests/cli_compare_cost.rs::cost_is_parity_invariant_02_vs_03`).

The lift-x LOCK is therefore **convention-only** — it makes the round-trip
deterministic so the parsed Segwitv0 miniscript is reproducible across
invocations, not because `02` is cost-load-bearing.

Helpers:
- `cost/strip.rs::tap_string_to_segv0_string` (orchestrator).
- `cost/strip.rs::inflate_xonly_to_compressed_even_y` (BIP-340 lift-x).
- Inverse multi rewrite via reused `replace_fragment` helper:
  `multi_a → multi`, `sortedmulti_a → sortedmulti`.

### §11.3 — Keypath-spend cost surface

When `IK ≠ NUMS`, the user can spend via the taproot key-path (signing
with the IK) in addition to the script-path. Cost:
- Witness shape under SIGHASH_DEFAULT: `1 (stack-count varint) + 1 (sig-length-prefix) + 64 (Schnorr) = 66 witness bytes`.
- Total vbytes: `(SEGWIT_INPUT_BASE_WU=164 + 66 + 3) / 4 = 58 vB`.

Surface:
- **JSON envelope**: new top-level `keypath_spend` field. `null` when `IK == NUMS` (or when not a tr() input); populated to `{ internal_key_xonly_hex, vbytes: 58, sats }` otherwise. Field shape is stable so consumers can branch on type (`isNullable`-style) rather than key-existence.
- **Plaintext table**: an annotation line **below** the per-condition table:
  ```
  Keypath-spend (via IK <hex>): 58 vB | <sats> sats
  ```
  The annotation is below-table (not a vertical column) because column widths in the per-condition table are byte-aligned with v0.27.x output for downstream consumers that diff against pinned-baseline fixtures.
- **`notes[]`**: a parallel advisory message (mirrors SPEC §2.3 wording for IK ≠ NUMS), pinned in the catalog at `docs/manual/src/40-cli-reference/41-mnemonic.md`.

### §11.4 — Error variants

Two pre-existing `CompareCostError` variants exit the `#[allow(dead_code)]`
holding pattern as part of this phase (they were declared in v0.26.0 with
attribute `#[allow(dead_code)] // Phase 2 — surfaces with --descriptor`):

- `CompareCostError::UnsupportedWrapper(String)` — fires for `tr(IK)` keypath-only and for `Bare`/`Pkh`/`Wpkh` wrappers (pre-existing) and `sh(non-wsh)` (pre-existing). Exit 3.
- `CompareCostError::MultiLeafTr` — NEW fire site: `Tr` descriptor with TapTree leaf count != 1. Exit 3.

Both variants now have live fire sites; the `#[allow(dead_code)]` attributes are removed.

### §11.5 — Test scope

P12A:
- Unit test `cost::strip::tests::lift_x_prefix_is_exactly_02` — pins the LOCK convention.
- Unit test `cost::strip::tests::lift_x_inflates_multi_a_to_multi_with_02_prefix` — pins the multi-rewrite half of the projection.

P12B/C — integration tests in `tests/cli_compare_cost.rs`:
- `tr_descriptor_nums_single_leaf_pk_happy_path` — IK == NUMS; no keypath_spend; no advisory.
- `tr_descriptor_non_nums_ik_surfaces_keypath_spend_and_advisory` — IK ≠ NUMS; JSON keypath_spend populated; advisory note present.
- `tr_descriptor_non_nums_ik_keypath_spend_plaintext_annotation_line` — plaintext annotation line literal-match.
- `tr_descriptor_single_leaf_and_v_pk_pk_two_signers` — shape variant.
- `tr_descriptor_single_leaf_multi_a_2_of_3` — shape variant (Tap multi_a).
- `tr_descriptor_multi_leaf_refused_exit_3` — multi-leaf rejection.
- `tr_descriptor_with_valid_checksum_succeeds` — checksum-pass (no-checksum form).
- `tr_descriptor_with_bad_checksum_exit_2` — checksum-fail (exit 2 parse error).
- `tr_descriptor_nums_keypath_only_refused_no_script` — `tr(IK)` no-script rejection.
- `cost_is_parity_invariant_02_vs_03` — R1-I4 (b) cost-domain parity smoke (SPEC §11.2).

### §11.6 — Manual update

`docs/manual/src/40-cli-reference/41-mnemonic.md`'s `## mnemonic
compare-cost` section gains a worked example for `--descriptor tr(...)`
input — both NUMS and non-NUMS IK shapes — and the notes-catalog row for
`input had a non-NUMS internal key IK; …` is updated from
"(v0.27+ FOLLOWUP)" to "(v0.28.0)" with the current behavior.

The flag table is unchanged (no new flags added in this phase — `tr()` is a
new value at the `--descriptor` argument site, not a new flag).
