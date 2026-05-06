# v0.6.1 Phase 0a — SLIP-0132 prefix-tolerance spike

Read-only inspection of `bitcoin = "0.32.8"` source at `~/.cargo/registry/src/index.crates.io-*/bitcoin-0.32.8/src/bip32.rs` (lines 789-815) and `bitcoin-0.32.8/src/base58.rs`. No new code; no library writes. Confirms the decode-swap-reencode pattern works against the locked dependency before the v0.6.1 SPEC commits to a normalizer design.

## Findings

### 1. `Xpub::decode` rejects non-neutral version bytes

`bitcoin::bip32::Xpub::decode(data: &[u8])` (line 789) accepts ONLY:

- `VERSION_BYTES_MAINNET_PUBLIC` = `[0x04, 0x88, 0xB2, 0x1E]` (xpub)
- `VERSION_BYTES_TESTNETS_PUBLIC` = `[0x04, 0x35, 0x87, 0xCF]` (tpub)

Any other 4-byte version prefix returns `Error::UnknownVersion([b0, b1, b2, b3])`.

`FromStr for Xpub` (line 871) calls `decode` after base58check verification. SLIP-0132 strings are base58-check valid but fail at the version-byte check — i.e., the failure is downstream of base58check, so a normalizer must intercept at the raw `bitcoin::base58::decode_check` layer.

### 2. 78-byte serialized layout (verified at lines 794-813)

| Offset | Bytes | Field |
|--------|-------|-------|
| 0..4   | 4     | version (what we swap) |
| 4      | 1     | depth |
| 5..9   | 4     | parent_fingerprint |
| 9..13  | 4     | child_number |
| 13..45 | 32    | chain_code |
| 45..78 | 33    | public_key (compressed) |

Total: 78 bytes pre-checksum. The 74-byte payload (everything after the version prefix) is byte-identical across SLIP-0132 variants of the same key — normalization is purely encoding-level, no derivation, no key-material change. Implementation-side invariant: `raw.len() == 78` (NOT 74).

### 3. SLIP-0132 prefix bytes (well-known) for the normalizer's swap table

| Variant | Net | BIP intent | Version bytes |
|---------|-----|------------|---------------|
| ypub | mainnet | BIP-49 single-sig | `0x04 9D 7C B2` |
| Ypub | mainnet | BIP-49 multisig (P2SH-P2WSH) | `0x02 95 B4 3F` |
| zpub | mainnet | BIP-84 single-sig | `0x04 B2 47 46` |
| Zpub | mainnet | BIP-84 multisig (P2WSH) | `0x02 AA 7E D3` |
| upub | testnet | BIP-49 single-sig | `0x04 4A 52 62` |
| Upub | testnet | BIP-49 multisig | `0x02 42 89 EF` |
| vpub | testnet | BIP-84 single-sig | `0x04 5F 1C F6` |
| Vpub | testnet | BIP-84 multisig | `0x02 57 54 83` |

Mainnet swap target: `0x04 88 B2 1E` (xpub). Testnet swap target: `0x04 35 87 CF` (tpub).

### 4. Decode-swap-reencode pattern (canonical implementation sketch)

```rust
fn normalize_slip0132_xpub(s: &str) -> Result<String, ToolkitError> {
    let raw = bitcoin::base58::decode_check(s)
        .map_err(|e| ToolkitError::BadInput(format!("base58check decode: {e}")))?;
    if raw.len() != 78 {
        return Err(ToolkitError::BadInput(format!(
            "extended-key serialization is 78 bytes; got {}", raw.len()
        )));
    }
    let prefix: [u8; 4] = raw[0..4].try_into().unwrap();
    let neutral_prefix = match prefix {
        [0x04, 0x88, 0xB2, 0x1E] => return Ok(s.to_string()),  // already xpub
        [0x04, 0x35, 0x87, 0xCF] => return Ok(s.to_string()),  // already tpub
        // SLIP-0132 mainnet → xpub
        [0x04, 0x9D, 0x7C, 0xB2] | [0x02, 0x95, 0xB4, 0x3F]
        | [0x04, 0xB2, 0x47, 0x46] | [0x02, 0xAA, 0x7E, 0xD3] => [0x04, 0x88, 0xB2, 0x1E],
        // SLIP-0132 testnet → tpub
        [0x04, 0x4A, 0x52, 0x62] | [0x02, 0x42, 0x89, 0xEF]
        | [0x04, 0x5F, 0x1C, 0xF6] | [0x02, 0x57, 0x54, 0x83] => [0x04, 0x35, 0x87, 0xCF],
        _ => return Err(ToolkitError::BadInput(format!(
            "unknown extended-key version prefix: {:02x}{:02x}{:02x}{:02x}",
            prefix[0], prefix[1], prefix[2], prefix[3]
        ))),
    };
    let mut swapped = raw.clone();
    swapped[0..4].copy_from_slice(&neutral_prefix);
    Ok(bitcoin::base58::encode_check(&swapped))
}
```

`bitcoin::base58::decode_check` and `encode_check` are public functions in `bitcoin::base58` (verified by `grep` against `~/.cargo/registry/src/index.crates.io-*/bitcoin-0.32.8/src/base58.rs`). This is a self-contained ~30-line helper.

### 5. Network-mismatch consideration

The SLIP-0132 normalizer does NOT validate `--network` against the SLIP-0132 prefix's implied network. An xprv-network-cross-check elsewhere in the toolkit (`derive_slot::derive_bip32_from_entropy` lines 45-51) catches some cases downstream, but not all xpub-flow paths route through that helper. Policy: document the implicit network-from-prefix mapping in the SPEC; treat user-supplied `--network`/prefix-network mismatch as user responsibility (matching existing toolkit behavior for raw `tpub` supplied with `--network mainnet`). Do NOT add a redundant pre-check in the normalizer.

## Bottom line

**Pattern CONFIRMED.** Decode-swap-reencode against `bitcoin = "0.32.8"` works as designed:

- Public surface (`bitcoin::base58::{decode_check, encode_check}` + `Xpub::from_str` post-swap) is sufficient.
- 78-byte invariant + 4-byte version-prefix swap covers all 8 SLIP-0132 variants.
- No `bitcoin` crate change required; no upstream PR needed.

SPEC may proceed (Phase 0) to formalize the normalizer behavior + the `--xpub-prefix` output grammar.
