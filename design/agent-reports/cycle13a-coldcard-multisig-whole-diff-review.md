# cycle-13a Lane A — whole-diff review (coldcard/jade multisig fidelity: H11 + H14)

**STATUS: IMPLEMENTER SELF-REVIEW ONLY — the mandatory INDEPENDENT adversarial
whole-diff review (CLAUDE.md §6 / per-phase policy item 5) was NOT run.** No
general-purpose Agent-API / subagent-dispatch tool was available in this
implementer session (only `TaskStop` + worktree tools surfaced). Per CLAUDE.md
policy item 5, this is FLAGGED explicitly and the formal independent review is
DEFERRED to API recovery — it MUST be run by the orchestrator before
integration/ship. The notes below are the implementer's own adversarial pass
over the diff against the plan's §6 focus areas; they are NOT a substitute for
the independent review.

**Branch:** `feature/cycle13a-coldcard-fidelity` (off toolkit `9b2a8ae3` = v0.65.2).
**Commits:** P1 `4a681410` · P2 `4bae1491` · P3 `7fd9a0b9` · P4 `a6ea6474`.
**Diff:** 9 files, +1100/-122. File-disjoint from Lane B (`cmd/restore.rs`) and
Lane C (`cmd/import_wallet.rs` / `cmd/bundle.rs` / `electrum.rs`) — verified.
No version-site / schema-mirror files touched.

## §6 focus-area self-check

- **I-2 scramble (load-bearing):** the divergent export reads `path`, `xpub`,
  `fingerprint` from the SAME sorted slot — `cosigners` is sorted ONCE by the
  xpub-lex key; `sorted_paths` is built from the post-sort `cosigners.iter()`
  and zipped with it in the emit loop. Test #1b uses xpub-sort order [C,B,A] vs
  slot order [A,B,C] with distinct paths (0'/1'/2') — confirmed sort≠slot, so a
  naive slot-order `derivations[i]` fix would pair sorted-pos-0 (C) with path 0'
  (A's) → #1b would stay RED under that fix. GREEN for the right reason.
- **Fixture blast-radius:** the COMPLETE cross-file break set was reconciled —
  unit `coldcard_multisig.rs`: the no-XFP test SPLIT into a depth-0-silent
  (H14-a) `parse_no_header_no_per_cosigner_xfp_uses_computed_silent` + a
  depth>0-REFUSE (H14-b) `parse_no_header_depth_gt0_refuses`; the coin-type test
  carries a top-level `XFP:` so the coin-type check stays reached; the two warn
  fixtures re-pointed to depth-0 `XPUB_D0_*` (asserts KEPT). CLI
  `cli_import_wallet_coldcard_multisig.rs:166/:209` re-pointed to depth-0
  (asserts KEPT). Depth-0 `XPUB_D0_A/B/C` + `FP_D0_*` consts ADDED (none existed
  at `9b2a8ae3`; XPUB_A/B depth 4, XPUB_C depth 3) and PINNED against the bitcoin
  crate (`depth0_const_fingerprints_pinned`). No 7th breaking fixture surfaced.
- **Q1 shared-path non-regression:** the `<XFP>:` arm consumes ONLY
  `pending_per_cosigner_path.take()`; `shared_derivation` is never written.
  Test #13 (3-cosigner shared-path) GREEN before AND after the arm change.
- **H14-c silence (M-2):** depth>0 + supplied XFP takes the first match arm and
  returns `supplied` with NO `writeln!` to stderr and NO `xfp_header_disagreed`
  → the toolkit's own all-agree export (depth-4 account xpubs under a depth-0
  master fp) re-imports SILENT. Confirmed by the round-trip test #5 succeeding
  and the re-pointed CLI fixtures.
- **Refusal exit codes:** #3 empty-origin export → `BadInput` → exit 1
  (asserted via `err.exit_code()==1`). #6/#14 depth>0/no-XFP import → 
  `ImportWalletParse` → exit 2 (#14 asserts the process exit code == 2).
- **Idempotence (#15):** `canon(canon(blob))==canon(blob)` on a divergent blob;
  homogeneous baseline cluster (`_idempotent`, `_with_and_without_xfp_header_match`,
  `_3of5_stable`, `_cosmetic_variants_match`, `_invalid_blob_returns_parse_error`)
  stays GREEN.
- **No fmt drift / no version-site edits / no schema_mirror delta:** verified.

## Residual notes for the independent reviewer

- The `cli_import_wallet_coldcard_multisig.rs` #16 reconstructs the canonical
  OUTPUT side of the unified diff (`roundtrip.diff`) to assert both divergent
  paths survive — a slightly indirect assertion; the direct unit-level proof is
  #15. The reviewer should confirm #16 is RED pre-P4 (it was — the output side
  dropped `m/48'/0'/1'/2'`).
- The empty-origin refusal (#3) is unreachable via the public CLI (template
  family always assigns a path; descriptor import refuses origin-less inputs);
  it is exercised at the emitter boundary with a hand-built `EmitInputs`.
