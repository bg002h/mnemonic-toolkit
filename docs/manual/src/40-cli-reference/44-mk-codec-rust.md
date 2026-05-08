# Using `mk-codec` from Rust

`mk-codec` is the Rust crate implementing the mk1 format
(mnemonic-key). Unlike md1 and ms1, mk1 has no standalone CLI in
v0.1 — instead, library consumers use `mk-codec` directly, or
indirectly via `mnemonic convert --from mk1=… --to xpub …`.

This chapter covers the public API surface for direct Rust use.

## Cargo dependency

```toml
[dependencies]
mk-codec = { git = "https://github.com/bg002h/mnemonic-key", tag = "mk-codec-v0.2.2" }
```

## Public surface

The crate's top-level re-exports define the integration point:

```rust
pub use consts::{
    CHUNKED_FRAGMENT_LONG_BYTES, CHUNKED_FRAGMENT_REGULAR_BYTES,
    CROSS_CHUNK_HASH_BYTES, GENERATOR_FAMILY, HRP, MAX_CHUNKS,
    MAX_PATH_COMPONENTS, MK_LONG_CONST, MK_REGULAR_CONST,
    NUMS_DOMAIN, ORIGIN_FINGERPRINT_BYTES, POLICY_ID_STUB_BYTES,
    SINGLE_STRING_LONG_BYTES, SINGLE_STRING_REGULAR_BYTES,
    XPUB_COMPACT_BYTES,
};
pub use error::{Error, Result};
pub use key_card::{KeyCard, decode, encode, encode_with_chunk_set_id};
```

## Encoding an mk1 card from a `KeyCard`

`KeyCard` is `#[non_exhaustive]`, so external callers must construct
it via `KeyCard::new(...)`:

```rust
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
use mk_codec::{KeyCard, encode};

let xpub: Xpub = "xpub6CatWdiZi...".parse().unwrap();
let card = KeyCard::new(
    vec![[0u8; 4]],                                   // policy_id_stubs (non-empty)
    Some(Fingerprint::from([0x73, 0xc5, 0xda, 0x0a])), // origin_fingerprint (None for privacy-preserving mode)
    "m/84'/0'/0'".parse::<DerivationPath>().unwrap(),  // origin_path
    xpub,                                              // xpub: bitcoin::bip32::Xpub
);

let strings: Vec<String> = encode(&card)?;
for s in strings {
    println!("{s}");
}
```

The function takes `&KeyCard` (borrows, doesn't consume) and returns
one or more BCH-checksummed strings, depending on whether the card
fits in the regular code or needs the long code (`MK_LONG_CONST` vs
`MK_REGULAR_CONST`).

## Decoding an mk1 card

```rust
use mk_codec::decode;

let card = decode(&[
    "mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4",
    "mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh",
])?;

println!("xpub fingerprint: {:?}", card.origin_fingerprint);
println!("origin path: {}", card.origin_path);
println!("xpub: {}", card.xpub);
println!("policy_id_stubs: {} stub(s)", card.policy_id_stubs.len());
```

`origin_fingerprint` is `Option<Fingerprint>` (None when the card
was emitted in privacy-preserving mode); `policy_id_stubs` is
`Vec<[u8; 4]>` (always non-empty after a successful decode).

## Cross-binding with md-codec

Each mk1 card carries one or more 4-byte `policy_id_stub`s (each is
the first 4 bytes of `SHA-256(canonical wallet-policy preimage)`).
Toolkits combining mk-codec with md-codec compute the stub on the
policy side and embed it on the key side, so that mismatched cards
can be detected:

```rust
let mk_stubs = &mk_card.policy_id_stubs;
let md_stub = compute_policy_id_stub(&md_template, &xpubs);
assert!(mk_stubs.contains(&md_stub));
```

The md-codec crate exposes `compute_policy_id_stub`; see the
descriptor-mnemonic README for that surface.

## Modules

- **`consts`** — wire-format constants (HRP `mk`, byte sizes, BCH
  generator constants, NUMS domain).
- **`bytecode`** — the bit-level layout under the BCH layer.
- **`string_layer`** — the alphabet / chunking / checksum machinery.
- **`key_card`** — the high-level `KeyCard` struct, `encode`, `decode`.
- **`error`** — the `Error` and `Result` types.
The crate ships a `gen_mk_vectors` binary target (`src/bin/`) for
fixture regeneration; it is a maintainer tool, not a library module
(no `mk_codec::bin` import path).

## Stability

`mk-codec` is at v0.2 (post-cycle close-out). v0.1 of the manual
targets v0.2.2; semver-major bumps may break the API. Track the
crate's CHANGELOG for breaking changes; minor bumps add features
without breaking existing callers.

For non-Rust consumers, `mnemonic convert --from mk1=… --to xpub
--to fingerprint --to path` is the cross-language integration point;
see [the convert reference](#mnemonic-convert).
