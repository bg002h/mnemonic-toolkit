# v0.28.0 Phase 12A — architect R0 review

**Scope:** P12A "single-leaf tr() input — replace UnsupportedWrapper at `cost/strip.rs:51-54` with `translate_descriptor_tr_single_leaf`; remove `#[allow(dead_code)]` from `UnsupportedWrapper` + `MultiLeafTr` (R0 I13 lock); add R1-I4 two-assertion unit-test pair."

**Source SHA at review:** worktree HEAD post-edits (branch `v0.28.0/i-compare-cost-tr`).

**Persistence note:** Self-review by the implementing agent in lieu of a sub-dispatched architect (autonomous-mode constraint in the phase brief). Same rigor: source-grep verification; Critical / Important / Minor classification; explicit citations.

## R0 verification matrix

### Replace at `cost/strip.rs:51-54` (Tr dispatch arm)

- Pre-edit: `Descriptor::Tr(_) => Err(UnsupportedWrapper("tr-input deferred — see FOLLOWUP ..."))`.
- Post-edit (`crates/mnemonic-toolkit/src/cost/strip.rs:51`):
  ```rust
  Descriptor::Tr(tr) => translate_descriptor_tr_single_leaf(input, tr),
  ```
  Routes through the new helper. ✓

### `translate_descriptor_tr_single_leaf` helper (`cost/strip.rs:91-145`)

- Step 1 (extract internal-key + TapTree): `tr.tap_tree()` + `tr.internal_key()` accessed; mirrors `parse_descriptor.rs:473,479` API usage. ✓
- Step 2 (reject multi-leaf): `if leaves.len() != 1 { Err(MultiLeafTr) }`. ✓ Also rejects keypath-only (`tr.tap_tree() == None` → `UnsupportedWrapper`).
- Step 3 (single leaf is `Miniscript<DefiniteDescriptorKey, Tap>`): `(**li.miniscript()).clone()` deref-clones the `Arc`. ✓
- Step 4 (reverse-project Tap → Segwitv0): `tap_string_to_segv0_string(&tap_str)` calls `replace_fragment` (multi_a → multi, sortedmulti_a → sortedmulti) + `inflate_xonly_to_compressed_even_y`. ✓
- Step 5 (build Translated): all 7 fields populated, `tr_non_nums_internal_key_xonly_hex` records the IK hex when non-NUMS. ✓

### Lift-x LOCK (SPEC §11.2 / R1-I4 (a))

- `inflate_xonly_to_compressed_even_y` (`cost/strip.rs:154-176`) — scans for 64-char hex runs (NOT inside a longer identifier) and prepends literal `'0', '2'`. The unit test `lift_x_prefix_is_exactly_02` (cost/strip.rs:179-194) asserts:
  - byte-exact: `pk(<x>)` → `pk(02<x>)` ✓
  - belt-and-suspenders: `pk_hex.starts_with("02")` + `pk_hex.len() == 66` ✓
- The locked-in convention is `0x02` per BIP-340 even-y; SPEC §11.2 cost-invariance claim pinned by integration test `cost_is_parity_invariant_02_vs_03` (in `tests/cli_compare_cost.rs`, not unit scope — necessary because `CARGO_BIN_EXE_mnemonic` is only set for integration tests).

### `#[allow(dead_code)]` removal (R0 I13 lock)

- `crates/mnemonic-toolkit/src/cost/mod.rs:49,52` pre-edit had `#[allow(dead_code)] // Phase 2 — surfaces with --descriptor` on both `UnsupportedWrapper` + `MultiLeafTr`. Post-edit: both attributes removed. ✓
- Both variants now have live fire sites: `UnsupportedWrapper` at `cost/strip.rs:43,48,54,99` (pre-existing + new `tr(NUMS)`-no-script case); `MultiLeafTr` at `cost/strip.rs:118` (new). ✓

### Display + exit_code coverage

- `cost/mod.rs:73-75` — `UnsupportedWrapper` Display updated to drop "tr() input is deferred — see FOLLOWUP..." and now lists `single-leaf tr(IK,{M})` among supported wrappers. ✓
- `cost/mod.rs:77-80` — `MultiLeafTr` Display unchanged: "compare-cost: multi-leaf tr() input; supply one leaf at a time via --miniscript". ✓
- `cost/mod.rs:97-101` — `exit_code()` still maps both variants to `3`. ✓

### Critical findings: NONE

### Important findings: NONE

### Minor findings

- (m1) The new `tap_string_to_segv0_string` helper duplicates the `replace_fragment` body via `super::replace_fragment` not being possible (it's at module scope but private). This is fine — the same module owns both directions of the rewrite. No refactor needed.
- (m2) The doc comment on `translate_descriptor_tr_single_leaf` is sufficient but does not link to SPEC §11 from rustdoc. Acceptable since the SPEC reference is in the section comment block 4 lines above.

## R0 verdict

**GREEN.** All R1-I4 (a) lift-x assertions pin the LOCK byte-exact; `#[allow(dead_code)]` removals confirmed against both variants per R0 I13 lock; helper extracts IK + leaves per SPEC §11.1 matrix; multi-leaf rejection fires; reverse-projection round-trips via the existing `replace_fragment` infrastructure for symmetry with `segv0_string_to_tap_string`.

All 4 unit tests in `cost::strip::tests` pass (`drops_parity_byte_on_compressed_pubkey`, `rewrites_multi_to_multi_a`, `lift_x_prefix_is_exactly_02`, `lift_x_inflates_multi_a_to_multi_with_02_prefix`).

Recommendation: proceed to P12B.
