# Phase P2.4 sub-batch 5d (Track M — export-wallet + derive-child) — R0 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-v1`
**Scope:** 5d — `45-export-wallet.md` (NEW, ~340 lines, 15 flags); `46-derive-child.md` (NEW, ~325 lines, 9 flags); `.cspell.json` (+2 words: `subkeys`, `codepoints`).

**Verdict:** **ITERATE 1C / 1I / 0N / 3n.**

`45-export-wallet.md` was clean across all 15 flags + 5 enumerated-flag outlines + 8 EXPORT_FORMATS variants + Taproot TaggedOrIndexed widget + refusals (all byte-exact against `cmd/export_wallet.rs`) + worked example (canonical mainnet xpub). `46-derive-child.md` was clean across all 9 flags + 4 enumerated-flag outlines + 9 BIP85_APPLICATIONS variants + per-app `--length` validator table + worked example. Two byte-exactness drifts in derive-child's Refusals table (R0 caught the cycle's recurring pattern):

## Critical

### C-1 — `DeriveChildUnsupportedApp` text not byte-exact

`46-derive-child.md` Refusals row for `--application rsa|rsa-gpg` paraphrased the refusal as `derive-child: --application <X> is not supported (RSA crate vulnerability RUSTSEC-2023-0071 unpatched)`. Source at `crates/mnemonic-toolkit/src/error.rs:366-369` is `--application <rsa|rsa-gpg> is out-of-scope: the rsa crate has unpatched timing-attack advisory RUSTSEC-2023-0071 and BIP-85 RSA / RSA-GPG demand is limited; deferred pending crate stability + user demand.` (the `// SPEC_derive_child_v0_8.md §7 byte-exact stderr text` comment makes this an explicitly-byte-pinned constant). Fixed inline.

## Important

### I-1 — "Unknown `--from` node" text was byte-incorrect (collapsed two refusal layers)

The chapter had `unknown --from node "<token>"; expected one of: xprv, phrase`. Source has TWO distinct refusal layers:
- Parser-level (`cmd/convert.rs:135-140`, shared with `mnemonic convert`): emits the long 13-node-name list when the token is unknown to the parser.
- Handler-level (`cmd/derive_child.rs:159-164`): rejects valid-but-unsupported nodes (e.g. `wif=…`) with the "must be xprv or phrase" message.

Fixed by splitting the row into two: one for unknown-token (with the byte-exact 13-node list) and one for recognized-but-unsupported (referencing the handler-level refusal).

## Nitpicks

### n-1 — `--length` out-of-range example was paraphrased

The chapter said `--application bip39 requires --length in {12, 15, 18, 21, 24}`. Source format at `error.rs::DeriveChildLengthOutOfRange` is `--length <N> out of range for --application bip39 (valid: 12 | 15 | 18 | 21 | 24 words)`. Fixed inline (with markdown pipe-escape to keep the table-cell format).

### n-2 — `export-wallet` "neither --template nor --descriptor" refusal byte-exact

Verified clean — chapter cites `cmd/export_wallet.rs:215-219` correctly with the verbatim BadInput message.

### n-3 — `derive-child` worked-example child phrase content unverifiable

The chapter prints a specific 12-word phrase as the BIP-85 child of the canonical master. Per the R0 spec, the literal phrase content is shape-only (12 words; deterministic-claim disclaimer present); not blocking.

## Markdown table pipe-escape

Folding C-1/I-1/n-1 introduced literal `|` characters inside the byte-exact strings. Initial edit broke markdownlint (table-column-count + no-space-in-code) — fixed by escaping `|` as `\|` within the table cells.

## Lint state (post-fold)

- Phase 4 schema-coverage RED at **201 missing** (was 286 → -85 = export-wallet 1+15+34=50 + derive-child 1+9+25=35 = 85). No orphans.
- Phase 5 outline-coverage RED at **28 missing** (was 39 → -11 = 6 export-wallet outlines + 5 derive-child outlines).
- Phases 1-3 GREEN (post pipe-escape fix).
- HTML 21 H1 chapters (was 19 → +2).
- PDF 109 pages (was 89 → +20).

After folds, R1 should LOCK.
