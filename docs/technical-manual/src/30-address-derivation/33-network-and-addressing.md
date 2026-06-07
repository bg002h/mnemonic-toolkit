# Network and Addressing

`Descriptor::derive_address` (`descriptor-mnemonic/crates/md-codec/src/derive.rs::Descriptor::derive_address`) takes a `bitcoin::Network` parameter and returns an `Address<NetworkUnchecked>`. Networks affect the *encoding* of the address — not the *script* it commits to. The same descriptor at the same `(chain, index)` produces the same redeem-script / witness-program / taproot output-key under every network; only the human-readable wrapper changes.

This chapter walks the five-network surface and the SLIP-0132 prefix interactions. The deeper end-user workflow (using `mnemonic convert` to translate among `xpub` / `tpub` / `ypub` / `zpub` / `upub` / `vpub` prefix forms) lives in the end-user manual's `mnemonic` CLI chapter, §`mnemonic convert` (`mnemonic-toolkit/docs/manual/src/40-cli-reference/41-mnemonic.md:89-128`), and is not duplicated here.

## The four networks

`bitcoin::Network` enumerates five variants relevant to md1:

| Variant | Magic | Address HRPs / version bytes | BIP-32 xpub version bytes |
|---|---|---|---|
| `Network::Bitcoin` | mainnet | `1...` (P2PKH), `3...` (P2SH), `bc1q...` (segwit v0), `bc1p...` (taproot) | `0x0488B21E` |
| `Network::Testnet` | testnet3 | `m...` / `n...` (P2PKH), `2...` (P2SH), `tb1q...` / `tb1p...` | `0x043587CF` |
| `Network::Testnet4` | testnet4 | (same as Testnet) | `0x043587CF` |
| `Network::Signet` | signet | (same as Testnet) | `0x043587CF` |
| `Network::Regtest` | regtest | (legacy + `bcrt1q...` / `bcrt1p...` bech32) | `0x043587CF` |

The four non-mainnet variants share the BIP-32 `tpub` version-byte family (`0x043587CF`); md-cli treats them as one group when validating xpub-prefix bytes (`crates/md-cli/src/parse/keys.rs::parse_key`). The bech32 *address* HRPs do diverge: `tb1` (testnet3, testnet4, signet), `bcrt1` (regtest).

## The encoding-vs-script asymmetry

The transformation that the chapter §III.1's three-tier model walks ends in `miniscript::Descriptor::at_derivation_index(index).address(network)` (`descriptor-mnemonic/crates/md-codec/src/derive.rs::Descriptor::derive_address`). At that point the per-network address rendering happens entirely inside rust-miniscript and rust-bitcoin: the script bytes are already fixed; only the encoding adapter changes.

A concrete example: the BIP-84 `wpkh(@0/<0;1>/*)` descriptor with the abandon-mnemonic account-0 xpub produces

- `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu` under `Network::Bitcoin` (`address_derivation.rs::bip84_wpkh_receive_address_zero`),
- `tb1q...` (matching the same witness program with the `tb1` HRP) under `Network::Testnet` (`address_derivation.rs::bip84_wpkh_testnet_address`).

The witness program — the SHA-256 hash of the compressed pubkey at `m/84'/0'/0'/0/0` — is byte-identical across the two; only the bech32 HRP differs. Per-network signing, fee economics, and PSBT flow are entirely the caller's concern; md-codec exposes the address surface, nothing more.

The legacy P2PKH and P2SH cases are similar but use prefix bytes rather than HRPs: `0x00` (mainnet) / `0x6F` (testnet) for P2PKH; `0x05` / `0xC4` for P2SH. These bytes are part of the base58check\index{base58check} envelope, not the script.

## SLIP-0132 prefix interactions

SLIP-0132 defines alternative BIP-32 version bytes that hint at the intended descriptor *shape* (e.g., `zpub` for BIP-84 P2WPKH, `ypub` for BIP-49 P2SH-P2WPKH, `Zpub` for BIP-48 P2WSH multisig). These are **purely cosmetic** at the BIP-32 level — the chain-code and pubkey bytes are identical to the canonical `xpub` form; only the leading 4 version bytes change.

md1's address derivation does not consult the SLIP-0132 hint:

- `md address --key @N=<xpub>` accepts only the canonical `xpub` / `tpub` version bytes (`crates/md-cli/src/parse/keys.rs::parse_key`). A `zpub`, `ypub`, etc. input is rejected with `BadXpub { ... expected mainnet xpub version 0488B21E, got ... }`.
- The descriptor's actual script shape comes from the on-card BIP-388 template, not from any hint baked into the xpub prefix.

End users who hold their xpubs in SLIP-0132 form work through the toolkit's `mnemonic convert` subcommand: `mnemonic convert --from "zpub=…" --to xpub --network mainnet` normalizes the prefix without changing the underlying material. The reverse — emitting a SLIP-0132 prefix on output — is `--xpub-prefix zpub|ypub|Zpub|Ypub|...` on the convert side (`docs/manual/src/40-cli-reference/41-mnemonic.md:118`).

This separation is intentional: md1 stores the *template* (BIP-388 wallet policy) authoritatively. The xpub prefix-hint is end-user-presentation metadata, useful for wallet-software interop but redundant given the on-card template.

## Per-network address worked example

The §III.1 BIP-84 example produces `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu` on mainnet. Re-running it with `--network testnet` produces the matching `tb1q...` form against the same descriptor + xpub; the test at `address_derivation.rs::bip84_wpkh_testnet_address` asserts `s.starts_with("tb1q")` for this case.

Network-parameter changes do not require a card re-engraving: md1 stores no network bytes. The same physical card can produce mainnet addresses (`md address ... --network mainnet`) or testnet addresses (`md address ... --network testnet`) by command-line selection. The xpubs supplied via `--key @N=...` must match the network-family version bytes, however — a mainnet `xpub6...` cannot be paired with `--network testnet` without first converting through `mnemonic convert` (the CLI rejects mismatched prefixes pre-derivation).

## Source pointers

- `descriptor-mnemonic/crates/md-codec/src/derive.rs::Descriptor::derive_address` — `Descriptor::derive_address` with the `network` parameter.
- `descriptor-mnemonic/crates/md-cli/src/parse/keys.rs::parse_key` — xpub version-byte validation by network family.
- `descriptor-mnemonic/crates/md-codec/tests/address_derivation.rs::bip84_wpkh_testnet_address` — testnet/mainnet parity test for `wpkh`.
- `mnemonic-toolkit/docs/manual/src/40-cli-reference/41-mnemonic.md` §`mnemonic convert` — end-user `convert` subcommand reference (SLIP-0132 prefix translation; not duplicated here).
- `mnemonic-toolkit/docs/manual/src/60-appendices/61-glossary.md` §"SLIP-0132" — end-user glossary entry for the prefix family.
- BIP-32 §"Serialization format" — canonical version-byte allocation.
- SLIP-0132 — alternative version bytes (informational; not normative for md1 wire layout).
