# Independent oracle for `mnemonic nostr` address fixtures

`regen_nostr_vectors.py` is a **pure-Python** reference (no rust-bitcoin / no
external crates) computing the Bitcoin addresses for a nostr x-only key:

- secp256k1 affine point math + BIP-340 `lift_x` (even-y);
- BIP-341/BIP-86 key-path taptweak `Q = lift_x(x) + tagged_hash("TapTweak", x)·G`;
- HASH160 + bech32 (BIP-173) / bech32m (BIP-350) / base58check.

It is the independent witness for `src/nostr.rs::cross_impl_fixture`
(`pinned_addresses_match_independent_oracle`). If `address_for` ever drifts from
Bitcoin's address rules, that test fails against this oracle.

## Regenerate

```
python3 crates/mnemonic-toolkit/tests/external/regen_nostr_vectors.py
```

Current key: NIP-19 `npub10elfcs4fr0l0r8af98jlmgdh9c8tcxjvz9qkw038js35mp4dma8qzvjptg`
(x-only `7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e`, mainnet).
Paste the four printed addresses into the `EXPECTED_*` constants in
`src/nostr.rs::cross_impl_fixture`.
