# tech-manual v0.2.0 — Phase 2.4 reviewer report (r1)

| Field | Value |
|---|---|
| Cut | `tech-manual-v0.2.0` |
| Phase | 2.4 (back-matter accretion) |
| Commit under review | `14189f3` |
| Date | 2026-05-11 |
| Reviewer | `feature-dev:code-reviewer` |
| Files in scope | `60-back-matter/{61-glossary.md, 62-index-table.md, 63-release-history.md, 64-bip-cross-reference.md}` + 12 new `\index{}` markers in `32-shape-coverage.md` + 2 new cspell words |

## Findings: 0 Critical / 1 Important / 1 Low / 0 Nit

---

## Important

**I-1. `Pubkeys TLV` glossary entry inverts chain-code / pubkey order**

`61-glossary.md:177` reads: "33-byte compressed pubkey + 32-byte chain code per `@N`"

Actual wire layout per `descriptor-mnemonic/crates/md-codec/src/derive.rs:41-47` (doc-comment for `xpub_from_tlv_bytes`): "`bytes[0..32]` = chain code; `bytes[32..65]` = compressed public key." The two fields are transposed in the glossary entry.

Fix: "32-byte chain code + 33-byte compressed pubkey per `@N`".

---

## Low

**L-1. BIP-49 cross-reference row missing §III.1 citation**

`64-bip-cross-reference.md:13` lists `§III.2` only. `31-descriptor-to-miniscript.md:80` cites BIP-49 with an index marker in the §III.1 body: `BIP-44/49/84/86` (single-signer wallets). Every other BIP in that inline list (BIP-44, BIP-84, BIP-86) correctly has §III.1 in its cross-reference row; BIP-49 is the sole gap.

Fix: change `§III.2` to `§III.1, §III.2`.

---

## Verified-correct items (no action needed)

- `address derivation` entry cite `derive.rs:92-132` — PASS.
- `DescriptorPublicKey` cite `to_miniscript.rs:84-89` — PASS.
- `H-point (NUMS)` hex and `to_miniscript.rs:34-35` cite — PASS (hex matches `NUMS_H_POINT_X_ONLY_HEX` verbatim).
- `key_index (md1)` glossary entry — correctly states "Suppressed entirely on the wire for `Body::Tr` when `is_nums = 1`" (the inverse fabrication caught at Phase 2.2 is not present).
- `PathDecl` cite `origin_path.rs:82-96` — PASS.
- `script context` three-variant list (Legacy / Segwitv0 / Tap) — matches `to_miniscript.rs:26` imports exactly.
- `SLIP-0132` rejection claim — `parse/keys.rs:43-66` enforces only `xpub`/`tpub` version bytes; non-canonical prefixes fail.
- `wildcard (BIP-389)` hardened-rejection claim — `derive.rs:99-101` confirms.
- BIP cross-ref Part III citations for BIP-44, BIP-86, BIP-379 — PASS.
- SLIP-0132 → §III.3 — PASS.
- `tech-manual-v0.1.0` tag exists.
- All 12 new `\index{}` markers in `32-shape-coverage.md` correctly escape underscores.
- All chapter-anchor links (`[Shape Coverage](#shape-coverage)` etc.) match heading slugs.
