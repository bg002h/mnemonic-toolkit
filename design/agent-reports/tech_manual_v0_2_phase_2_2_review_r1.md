# tech-manual v0.2.0 — Phase 2.2 reviewer report (r1)

| Field | Value |
|---|---|
| Cut | `tech-manual-v0.2.0` |
| Phase | 2.2 (Part III §III.2 — Shape coverage) |
| Commit under review | `7f05e50` |
| Date | 2026-05-11 |
| Reviewer | `feature-dev:code-reviewer` |
| Files in scope | `docs/technical-manual/src/30-address-derivation/32-shape-coverage.md` + supporting (`62-index-table.md` rows, `.cspell.json` regex+words, BIP-44 pkh transcript, rendered mermaid figure) |

## Findings: 1 Critical / 2 Important / 0 Low / 0 Nit

---

## Critical

**C-1. NUMS `key_index` wire-presence claim is factually wrong**

`32-shape-coverage.md:81` claims: "the `key_index` field is wire-present (the wire layout doesn't omit it on the `is_nums = 1` arm — see §II.1 §"NUMS handling" for the bit-level form) but ignored at conversion time".

Actual: `docs/technical-manual/src/20-wire-formats/21-md1-wire-format.md` (the §II.1 NUMS subsection) states that the `key_index` field is **suppressed entirely on the wire** when `is_nums = 1`. The wire-layout diagram is `| 1:is_nums | [kiw:key_index iff !is_nums] |` — the kiw-bit `key_index` field is conditional on `!is_nums`.

Confirmed against `descriptor-mnemonic/design/SPEC_v0_30_wire_format.md §7.2`:

```text
Tag::Tr(6) | is_nums(1) | key_index(kiw, present iff !is_nums) | has_tree(1) | [tree if has_tree]
```

The chapter conflates the in-memory Rust struct (`Body::Tr { is_nums: bool, key_index: u8, .. }` — `key_index` always present as a Rust value) with the wire encoding (where the kiw-bit `key_index` field is omitted when `is_nums = 1`). This is a direct factual contradiction of the wire-format spec.

Fix: Replace the parenthetical with: "the `key_index` Rust field is carried in `Body::Tr` even when `is_nums = 1`, but the kiw-bit wire field is **suppressed on the wire** when `is_nums = 1` (see §II.1 §"NUMS encoding for `tr()`" for the bit-level form)."

---

## Important

**I-1. Cross-reference subsection title does not match actual heading**

`32-shape-coverage.md:81` references `§II.1 §"NUMS handling"`. The actual subsection title in `21-md1-wire-format.md` is `## NUMS encoding for tr()`. Same failure mode as Phase 2.1's I-2 (anchor-string drift).

Fix: Change `§"NUMS handling"` to `§"NUMS encoding for tr()"`.

**I-2. `wsh(multi(...))` Bucket 7 row has no backing test; misleadingly cites the sorted test**

`32-shape-coverage.md:137` includes a row: `wsh(multi(2,@0,@1,@2))` BIP-48, citing test `:252-331` with gloss "same shape as `wsh_sortedmulti_2_of_3_address` ... with the sort step elided".

Actual: The test at `:252-331` is `wsh_sortedmulti_2_of_3_address`, which uses `Tag::SortedMulti`. There is **no test anywhere in `address_derivation.rs` for `Tag::Multi` (unsorted) inside `wsh(...)`** — confirmed by source search. The `wsh_inner_to_descriptor` function at `:183-196` handles `Tag::Multi` via fall-through to `node_to_miniscript::<Segwitv0>` (the `Terminal::Multi` arm at `to_miniscript.rs:365-373`), but this path is untested in the integration suite.

Citing a sorted-multi test as cross-validation for unsorted-multi, with the gloss that the sort step is "elided", misleads the reader about test coverage.

Fix: Drop the `wsh(multi(...))` row from the Bucket 7 table OR explicitly note that `Tag::Multi` inside `wsh` is routed through `node_to_miniscript::<Segwitv0>` but lacks a dedicated golden-vector integration test. A FOLLOWUP can request adding the missing paired integration test in the md1 repo.

---

## Verified-correct items (no action needed)

All line-number citations spot-checked and confirmed accurate:

| Cited range | Claimed content | Verified |
|---|---|---|
| `to_miniscript.rs:54-64` | `to_miniscript_descriptor` entry point | PASS |
| `to_miniscript.rs:84-89` | `DescriptorXKey` construction | PASS |
| `to_miniscript.rs:130-168` | `node_to_descriptor` dispatch table | PASS |
| `to_miniscript.rs:135-142` | `Pkh` + `Wpkh` arms | PASS |
| `to_miniscript.rs:149-162` | `Tr` key-path arm | PASS |
| `to_miniscript.rs:163-167` | unsupported top-level error | PASS |
| `to_miniscript.rs:172-179` | `build_nums_internal_key` | PASS |
| `to_miniscript.rs:183-196` | `wsh_inner_to_descriptor` | PASS |
| `to_miniscript.rs:201-235` | `sh_inner_to_descriptor` | PASS |
| `to_miniscript.rs:220-223` | `sh(wpkh)` arm | PASS |
| `to_miniscript.rs:242-253` | `TapTree::combine` recursion | PASS |
| `to_miniscript.rs:254-258` | single-leaf optimization | PASS |
| `to_miniscript.rs:262-443` | `node_to_miniscript` full dispatch | PASS |
| `to_miniscript.rs:387-391` | `SortedMultiA` rejection | PASS |
| `to_miniscript.rs:417-421` | `RawPkH` rejection | PASS |
| `to_miniscript.rs:429-434` | top-level wrapper rejection | PASS |
| `to_miniscript.rs:484-502` | hash-leaf constructors | PASS |
| `address_derivation.rs:67-91` | BIP-84 wpkh test + address | PASS |
| `address_derivation.rs:158-186` | BIP-86 tr keypath test + address | PASS |
| `address_derivation.rs:193-217` | BIP-44 pkh test + address | PASS |
| `address_derivation.rs:252-331` | `wsh_sortedmulti_2_of_3_address` | PASS |
| `address_derivation.rs:337-414` | `sh_wsh_sortedmulti_2_of_3_address` | PASS |
| `address_derivation.rs:484-532` | `sh_sortedmulti_2_of_3_address` | PASS |
| `address_derivation.rs:535-575` | `tr_nums_single_pk_leaf_address` | PASS |
| `address_derivation.rs:580-622` | `tr_single_pk_leaf_address` | PASS |
| `address_derivation.rs:626-685` | `tr_multi_a_2_of_3_leaf_address` | PASS |
| `address_derivation.rs:690-723` | `wsh_check_pk_k_address` | PASS |
| `address_derivation.rs:727-775` | `tr_branching_two_leaf_address` | PASS |
| `address_derivation.rs:779-844` | `tr_branching_with_multi_a_address` | PASS |
| `address_derivation.rs:849-894` | `wsh_and_v_address` | PASS |
| `address_derivation.rs:899-956` | `wsh_thresh_address` | PASS |
| NUMS hex constant | matches `to_miniscript.rs:35` and test at `:538` | PASS |
| Transcript `.out` address | `1LqBGSKuX5yYUonjxT5qGfpUsXKYYWeabA` vs test `:215` | PASS |
| `expand_per_at_N` in `canonicalize.rs` | function exists at line 420 | PASS |
