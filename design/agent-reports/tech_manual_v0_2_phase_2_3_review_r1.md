# tech-manual v0.2.0 — Phase 2.3 reviewer report (r1)

| Field | Value |
|---|---|
| Cut | `tech-manual-v0.2.0` |
| Phase | 2.3 (Part III §III.3 — Network and addressing) |
| Commit under review | `73428a1` |
| Date | 2026-05-11 |
| Reviewer | `feature-dev:code-reviewer` |
| Files in scope | `docs/technical-manual/src/30-address-derivation/33-network-and-addressing.md` + supporting (`62-index-table.md` row, `.cspell.json` word) |

## Findings: 0 Critical / 1 Important / 0 Low / 0 Nit

---

## Important

**I-1. `Network::Testnet4` omitted from variants table; non-mainnet count miscounted**

`33-network-and-addressing.md:9` opens "enumerates four variants relevant to md1"; the table lists Bitcoin / Testnet / Signet / Regtest. But `bitcoin::Network` has a fifth arm — `Testnet4` — distinct from `Testnet`. The match arm at `descriptor-mnemonic/crates/md-cli/src/parse/keys.rs:46-49` enumerates all four testnet flavors:

```rust
bitcoin::Network::Testnet
| bitcoin::Network::Testnet4
| bitcoin::Network::Signet
| bitcoin::Network::Regtest => (TESTNET_XPUB_VERSION, "testnet"),
```

The chapter's "Testnet" row's Magic column reads "testnet3 / testnet4" as if `Testnet4` were a sub-flavor; this is incorrect at the type level. The text on line 18 compounds the error: "The three non-mainnet variants share the BIP-32 `tpub` version-byte family" — there are *four* non-mainnet variants.

Fix: add a `Network::Testnet4` row between Testnet and Signet (Magic = "testnet4", HRPs/version-bytes = same as Testnet); update the opening sentence to "five variants" and the prose to "four non-mainnet variants".

---

## Verified-correct items (no action needed)

All other cited line ranges pass spot-check:

| Cite | Verified |
|---|---|
| `derive.rs:92-132` (derive_address) | PASS |
| `derive.rs:121-130` (at_derivation_index/address) | PASS |
| `parse/keys.rs:43-49` (testnet match arm) | PASS |
| `parse/keys.rs:43-77` (full validation block) | PASS |
| `address_derivation.rs:222-245` (testnet parity test) | PASS |
| `41-mnemonic.md:89-128` (mnemonic convert section) | PASS |
| `41-mnemonic.md:118` (--xpub-prefix flag row) | PASS |
| `61-glossary.md:192` (SLIP-0132 entry) | PASS |
| Mainnet `xpub` version bytes `0x0488B21E` | PASS |
| Testnet-family `tpub` version bytes `0x043587CF` | PASS |
| Legacy version bytes (P2PKH/P2SH × mainnet/testnet) | PASS |
| Bech32 HRPs (`bc1`, `tb1`, `bcrt1`) | PASS |
| Error message format from `parse/keys.rs:55-64` | PASS |
| `base58check` index row | PASS |
| `PSBT` cspell entry | PASS |
