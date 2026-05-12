# v0.8.1 Phase 2 R2 — reviewer report

## Verdict
**0C / 0I — converge**

## R1 fold verification

### C-1: verified

`crates/mnemonic-toolkit/src/wallet_export/sparrow.rs:215-219` — The `TrMultiA | TrSortedMultiA` arm applies `desc.rfind('#').map_or(desc, |pos| &desc[..pos])`. This is correct on all cases:

- Descriptor with `#checksum` suffix (normal path): `rfind('#')` finds the BIP-380 checksum delimiter; the slice `&desc[..pos]` drops the suffix cleanly.
- Descriptor without `#` (impossible under the current pipeline, but safe by design): `map_or(desc, ...)` returns the full string unchanged.
- `#` inside a key origin path or xpub: not possible. BIP-380 reserves `#` exclusively as the checksum delimiter; key origin paths use `[fp/path]` (hex, `/`, `'`), and xpubs use base58 (no `#`). Using `rfind` rather than `find` is the right choice since it anchors to the outermost, last occurrence — which is always the checksum when present.

The pinned fixture `sparrow_tr_multi_a_nums_2of3.json` line 9 contains no `#` in the `script` value. Confirmed.

SPEC §7 trailing paragraph (line 209 of `design/SPEC_export_wallet_v0_8.md`) mandates the strip and cites "Phase 2 R1 fold C-1". Implementation and SPEC are coherent.

### I-1: verified

- `tests/export_wallet/sparrow_tr_multi_a_nums_2of3.json` exists.
- `cell_5_sparrow_tr_multi_a_nums_2of3_byte_exact` uses `assert_eq!(stdout, expected)` — byte-exact.
- The secondary `serde_json` parse + `!script.contains('#')` invariant is meaningful. By the time it executes, `assert_eq` has already confirmed `stdout` matches the fixture byte-for-byte, so the `unwrap()` on JSON parse is safe in practice. The invariant provides a targeted regression guard independent of fixture contents.
- Cells 1-4 (`cell_1` through `cell_4`) are structurally unchanged from the pre-R1 baseline; their byte-exact assertions and fixtures are intact.

## New findings

None above confidence threshold (80).

## Confidence-filtered: omitted

- `cell_5` `serde_json::from_str(&stdout).unwrap()` parse after the byte-exact `assert_eq` is redundant in the success path but harmless; confidence 45 — not a defect.
- `wallet_name` used as `label` for all cosigners in the fixture (`"tr-multi-a-0"`) — already noted in R1 omissions as not a defect. Confidence 50 — carried forward, not new.
