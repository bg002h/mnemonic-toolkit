# v0.8.0 Phase 3 — mnemonic-toolkit BIP-85 v85.3 — R1 architect review + disposition

**Date:** 2026-05-13
**Reviewer:** `feature-dev:code-reviewer` (Sonnet 4.6), dispatched
per plan Phase 3 reviewer-loop discipline.
**Phase commit reviewed:** `d269dda` on branch
`v0_8_0-bip39-bip85-fill` — adds the
`cell_2b_bip39_24_words_reference_vector` test cell to
`crates/mnemonic-toolkit/tests/cli_derive_child.rs`; updates SPEC
§2 with the pre-cycle BIP-39 closure annotation; adds 3
FOLLOWUPS entries.

## R1 verdict

**0C / 0I.** MERGE — Phase 3 cleared as written.

## R1 checklist passes

All six review items came back clean (transcribed verbatim from
the R1 report):

1. **Cell correctness — PASS (confidence 100).** Cell uses
   `--application bip39 --length 24 --index 0` and asserts the
   exact 24-word output from the BIP-85 §"BIP39 24 word mnemonic"
   block: `"puppy ocean match cereal symbol another shed magic
   wrap hammer bulb intact gadget divorce twin tonight reason
   outdoor destroy simple truth cigar social volcano\n"`. Path,
   length, index, and expected output all match the canonical
   block verbatim.

2. **`MASTER_XPRV` constant — PASS (confidence 100).** Line 14–15
   of `cli_derive_child.rs` matches the spec-provided MASTER
   BIP32 ROOT KEY character-for-character.

3. **`cell_2b` naming convention — PASS (confidence 95).** Fits
   the established `cell_6a` / `cell_6b` / `cell_9b` suffix
   pattern already in the file. No inconsistency introduced.

4. **BIP-39 carry-over claim — PASS (confidence 100).** Verified
   the parametric loader at `cli_convert_bip39_vectors.rs` line 109
   asserts `rows.len() == 24` and the loop covers all 24 English
   Trezor vectors. SPEC §2 row annotation is accurate.

5. **FOLLOWUPS additions — PASS (confidence 100).** Three entries
   verified:
   - `bip-vector-adoption-v0_8` — cross-repo cycle companion;
     all three cited sibling repos confirmed present at same
     short-id.
   - `bip340-schnorr-signing-surface-evaluation` — companion
     `bip341-keypath-signing-vector-coverage` in
     `descriptor-mnemonic` confirmed with reciprocal back-reference.
   - `bip39-japanese-wordlist-support` — single-repo concern,
     no companion, correctly declared.
   All three follow predecessor v0.7.1 body format
   (Surfaced / Where / What / Status / Tier / Companion).

6. **Pre-existing clippy errors disposition — PASS (confidence 85).**
   The new cell is structurally identical to adjacent cells; no
   new clippy surface introduced. 16 pre-existing errors in
   `src/` are unchanged from master.

## Self-clear

R1 returned 0C/0I — no folds required. **Phase 3 close gate:
CLEAR.** This report is the canonical Phase 3 R1 record; Phase 4
audit-matrix successor will cross-cite it.

## Cycle-state snapshot post-Phase-3

| Phase | Repo | Status | Closing commit |
|---|---|---|---|
| 0 | mnemonic-toolkit | ✅ CLEAR | `d0e6afc` (1C/2I → folded) |
| 1 | descriptor-mnemonic | ✅ CLEAR | `b464f3f` (0C/2I → folded) |
| 2 | mnemonic-secret | ✅ CLEAR | `d0a76b2` (0C/1I → folded) |
| 3 | mnemonic-toolkit | ✅ CLEAR | `d269dda` (this report) |
| 4 | all 4 repos | pending | audit-matrix successor doc |
| E | all 3 touched | pending | patch-tag rollup |

Companion no-scope entry in `mnemonic-key`: `37d4fca` on branch
`v0_8_0-bip-vector-adoption-companion`.
