# Phase P2.4 sub-batch 6c (Track M — md address) — R0 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-v1`
**Scope:** 6c — `59-address.md` (NEW, ~155 LOC, 9 flags + 1 positional + 1 enumerated-flag outline). Closes batch 6 (50-md chapter complete).

**Verdict:** **LOCK 0C / 0I / 0N / 0n.** Source-truth fidelity exact across all checks.

## R0 verification matrix (all PASS)

| Check | Result |
|---|---|
| 9-bullet outline matching ADDRESS_FLAGS order | PASS |
| All 9 per-flag anchors `#md-address-<flag>` | PASS |
| `--network` outline + 4 variants byte-correct | PASS |
| `--chain` range 0..65_535 | PASS (matches `Number { min: 0, max: 65_535 }`) |
| `--index` range 0..2_147_483_647 | PASS |
| `--count` range 1..10_000 | PASS |
| Conditional visibility (positional → template/key/fingerprint Disabled; neither → template Required) | PASS (matches `form/conditional::md_address` exactly) |
| `--change` sugar + conflict-not-confirmed caveat | PASS (faithfully copies the `conditional.rs:288-290` comment) |
| Canonical first address `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu` | PASS (bit-identical to upstream `md-cli/tests/snapshots/cmd_address_json__wpkh_mainnet_receive_0_to_2.snap:8`) |
| ADDRESS_POSITIONALS name `phrases` (required:false, repeating:true) | PASS |
| `--json` Boolean | PASS (matches schema line 384) |

## Refusals table

8 distinct error paths documented; rows 1 and 2 align with the `build_descriptor` defense-in-depth at `address.rs:74-78` and the clap `conflicts_with = "phrases"`.

## Lint state

- Phase 4 schema-coverage RED at **72 missing** (was 86 → -14 = 1 sub + 9 flags + 4 variants).
- Phase 5 outline-coverage RED at **9 missing** (was 11 → -2 = 1 subcommand-outline + 1 flag-outline for --network).
- Phases 1-3 GREEN.
- HTML 34 H1 chapters (was 33 → +1).
- PDF 155 pages (was 151 → +4).

## Cycle close

Batch 6 (50-md chapter) closes here:
- 9 files (1 overview + 8 subcommand chapters)
- ~750 LOC
- 3 sub-batches: 6a (overview + 6 small/medium), 6b (encode), 6c (address)
- 3 R0 rounds total (6a/6b/6c — none required R1; the cycle's md tab landed cleaner than the mnemonic tab)

R1 not dispatched — R0 was clean LOCK with zero findings.
