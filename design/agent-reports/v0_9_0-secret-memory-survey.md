# v0.9.0 secret-memory surface survey — `mnemonic-secret` + `mnemonic-toolkit`

**Built:** 2026-05-13 as Phase 0 deliverable of the v0.9.0
secret-memory-hygiene cycle.
**Method:** read-only Grep + Read survey by `general-purpose` agent
(Opus 4.7) over `/scratch/code/shibboleth/mnemonic-secret/` and
`/scratch/code/shibboleth/mnemonic-toolkit/`. No files modified.
**Companion SPEC:** [`../SPEC_secret_memory_hygiene_v0_9_0.md`](../SPEC_secret_memory_hygiene_v0_9_0.md).
**Cycle plan:** `/home/bcg/.claude/plans/v0_9_0-secret-memory-hygiene.md`.

## Headline finding

Neither `mnemonic-secret/Cargo.toml` (ms-codec / ms-cli) nor
`mnemonic-toolkit/Cargo.toml` declares a `zeroize` dependency. There
is **zero zeroize discipline today** across the entire secret pipeline
of both crates — every owned secret buffer is dropped via the default
`Vec`/`String` allocator without scrub. This is the baseline against
which the v0.9.0 cycle will work.

## §1 Survey table — secret-material touch points

Disposition legend: **OWNED** (we hold the buffer; can
`Zeroizing::new(...)`-wrap), **BORROWED** (`&str`/`&[u8]`;
caller-owned), **CRATE-OWNED** (third-party type with no
zeroize-on-drop), **STDOUT-LEAK** (emitted via `println!`/`writeln!`).

### `mnemonic-secret` repo

| File:line | Function / type | Secret data type | Lifetime | Zeroize | argv | Notes |
|-----------|-----------------|------------------|----------|---------|------|-------|
| `crates/ms-codec/src/payload.rs:29` | `Payload::Entr(Vec<u8>)` | BIP-39 entropy 16-32 B | enum-payload lifetime | OWNED, none | — | The crate's only secret-bearing variant. `#[non_exhaustive]`. Used as both encoder input and decoder output. |
| `crates/ms-codec/src/decode.rs:45` | `decode()` → `Payload::Entr(payload_bytes)` | entropy | until caller drops `Payload` | OWNED, none | — | Built via `payload_with_prefix[1..].to_vec()` in `envelope::discriminate`. |
| `crates/ms-codec/src/envelope.rs:122-131` | `discriminate()` | `Vec<u8>` payload (entropy with reserved prefix stripped) | function-local then moved into Payload | OWNED, none | — | Allocates fresh `Vec` from `c.parts().data()`. The `Codex32String` itself holds the bytes — see CRATE-OWNED row below. |
| `crates/ms-codec/src/envelope.rs:141-156` | `package()` | `data: Vec<u8>` (prefix+entropy) | function-local | OWNED, none | — | Encoder buffer. Outlives only until `Codex32String::from_seed` returns. |
| upstream `codex32::Codex32String` | rust-codex32 v0.1 internal | full string + parts including data bytes | held by ms-codec callers | CRATE-OWNED | — | Cannot wrap. See §3. |
| `crates/ms-cli/src/parse.rs:31-37` | `read_phrase_input()` | `String` (BIP-39 phrase) | per CLI command | OWNED, none | NO (stdin or arg-clone) | Returns a normalized phrase. Callers: encode + verify. |
| `crates/ms-cli/src/parse.rs:44-50` | `read_stdin()` | `String` raw buffer | until trimmed copy returned | OWNED, none | NO | Reads full stdin via `read_to_string`. The raw buffer is dropped without scrub even though phrase normalization produces a second copy. **Two live copies of the phrase exist briefly.** |
| `crates/ms-cli/src/cmd/encode.rs:30` | `EncodeArgs::phrase: Option<String>` | BIP-39 phrase | for entire process | OWNED via clap, none | **YES** | Inline `--phrase "abandon …"` lives in argv + the parsed `EncodeArgs` for the full process lifetime. |
| `crates/ms-cli/src/cmd/encode.rs:34` | `EncodeArgs::hex: Option<String>` | hex-encoded entropy | for entire process | OWNED via clap, none | **YES** | Same argv exposure as `--phrase`. |
| `crates/ms-cli/src/cmd/encode.rs:52-57` | `run()` local `phrase`, `mnemonic`, `entropy` | phrase String + `bip39::Mnemonic` + `Vec<u8>` | function-local | OWNED + CRATE-OWNED, none | — | `Mnemonic::parse_in` constructs a `bip39::Mnemonic`; `to_entropy()` allocates a fresh `Vec<u8>`; both are dropped un-scrubbed at end-of-`run`. |
| `crates/ms-cli/src/cmd/encode.rs:69` | `entropy.clone()` into `Payload::Entr` | duplicate entropy Vec | until ms_codec::encode returns | OWNED, none | — | A second copy of the entropy buffer is allocated for the codec call. |
| `crates/ms-cli/src/cmd/encode.rs:73,93-94,111` | `emit_json`/`emit_text` printing `hex::encode(entropy)` | hex-encoded entropy | stdout | STDOUT-LEAK + OWNED `String` from hex | — | The hex String is intentionally on stdout. The intermediate String is OWNED and could be wrapped. |
| `crates/ms-cli/src/cmd/decode.rs:46-55` | `run()` locals `entropy`, `mnemonic`, `phrase` | entropy + `bip39::Mnemonic` + phrase String | function-local | OWNED + CRATE-OWNED, none | — | Decoder builds entropy via `Payload::Entr(b) => b` move, derives a Mnemonic, then `mnemonic.to_string()` allocates yet another phrase copy. **Three live secret buffers at peak.** All stdout-emitted. |
| `crates/ms-cli/src/cmd/decode.rs:67-94` | `emit_json`/`emit_text` | hex(entropy) + phrase | stdout | STDOUT-LEAK | — | `phrase.to_string()` allocates a fourth copy into `DecodeJson`. |
| `crates/ms-cli/src/cmd/verify.rs:27` | `VerifyArgs::phrase: Option<String>` | BIP-39 phrase | full process | OWNED via clap, none | **YES** | `--phrase "..."` round-trip check input. Argv-exposed when inline. |
| `crates/ms-cli/src/cmd/verify.rs:52-77` | `run()` locals `entropy`, `supplied`, `supplied_mnemonic`, `derived_mnemonic` | entropy Vec + 2× phrase String + 2× `bip39::Mnemonic` | function-local | OWNED + CRATE-OWNED, none | — | Most secrets simultaneously live of any path. `derived_mnemonic.to_string()` allocates a 5th phrase copy for the success log line (line 126). |
| `crates/ms-codec/src/lib.rs:18-19,29-30` | doc example | demo entropy `Vec<u8>` | doc-test | OWNED, none | — | Public API surface mirrors `Payload::Entr(Vec<u8>)`. |

### `mnemonic-toolkit` repo

| File:line | Function / type | Secret data type | Lifetime | Zeroize | argv | Notes |
|-----------|-----------------|------------------|----------|---------|------|-------|
| `crates/mnemonic-toolkit/src/cmd/bundle.rs:42` | `BundleArgs::passphrase: Option<String>` | BIP-39 passphrase | full process | OWNED via clap, none | **YES** | Inline `--passphrase` enters argv. No stdin equivalent for bundle. |
| `crates/mnemonic-toolkit/src/cmd/bundle.rs:77` | `BundleArgs::slot: Vec<SlotInput>` | each `SlotInput { value: String }` carries phrase / entropy-hex / wif / xprv depending on subkey | full process | OWNED via clap, none | **YES** | `--slot @0.phrase=abandon … about` puts the entire phrase in argv. Hot path for v0.4+ unified dispatch. |
| `crates/mnemonic-toolkit/src/slot_input.rs:68-72` | `SlotInput.value` | secret-bearing when `subkey ∈ {Phrase, Entropy, Wif, Xprv}` (`is_secret_bearing` at :56-58) | clap-parsed, lives full process | OWNED, none | **YES** | Defined in the parser; cloned at `bundle.rs:182` (`let slots = args.slot.clone()`) producing a 2nd copy. |
| `crates/mnemonic-toolkit/src/cmd/bundle.rs:313-329` | `resolve_slots` `Phrase` arm | borrows phrase via `s.value.as_str()` → calls `derive::derive_full` → returns `DerivedAccount { entropy: Vec<u8>, account_xpriv, … }` | function-local | BORROWED + OWNED + CRATE-OWNED, none | — | `acc.entropy` is moved into `ResolvedSlot.entropy`. `acc.account_xpriv` is silently dropped at scope end (lives in `DerivedAccount`). |
| `crates/mnemonic-toolkit/src/derive.rs:14-20` | `DerivedAccount` struct | `entropy: Vec<u8>`, `account_xpriv: Xpriv` (private key + chain code) | one per call site | OWNED Vec + CRATE-OWNED Xpriv, none | — | **Hot spot.** Holds *both* derived entropy and a full BIP-32 xpriv (32-B privkey + 32-B chain code inside `bitcoin::bip32::Xpriv`). |
| `crates/mnemonic-toolkit/src/derive_slot.rs:30-32` | `derive_bip32_from_entropy` `mnemonic`, `seed` | `bip39::Mnemonic` + 64-B `[u8; 64]` BIP-32 master seed | function-local | CRATE-OWNED + OWNED stack array, none | — | **Hot spot.** `mnemonic.to_seed(passphrase)` returns the 64-byte PBKDF2-HMAC-SHA512 output by value. Stored on stack as `[u8; 64]`. Dropped by value at end-of-function without scrub. |
| `crates/mnemonic-toolkit/src/derive_slot.rs:35-43` | `derive_bip32_from_entropy` `master`, `account_xpriv` | 2× `bitcoin::bip32::Xpriv` | function-local | CRATE-OWNED, none | — | Master xpriv + derived account xpriv simultaneously live. |
| `crates/mnemonic-toolkit/src/derive_slot.rs:78-92` | `derive_bip32_at_path` (mnemonic, seed, master) | same as above + a leaf `Xpriv` returned | function-local + returned | CRATE-OWNED, none | — | Returns a leaf-path Xpriv that the convert WIF/address branches dereference for `.private_key`. |
| `crates/mnemonic-toolkit/src/cmd/bundle.rs:884-905` | `bundle_run_unified_descriptor` `Phrase` arm | `mnemonic`, `entropy = mnemonic.to_entropy()`, `seed = mnemonic.to_seed(&passphrase)`, `master`, `acct_xpriv` | function-local | OWNED + CRATE-OWNED, none | — | A parallel BIP-39→BIP-32 spine for descriptor mode. Same secrets as `derive_slot`, separate code path. `args.passphrase.clone().unwrap_or_default()` (line 883) allocates a passphrase copy. |
| `crates/mnemonic-toolkit/src/cmd/bundle.rs:951-963` | `bundle_run_unified_descriptor` `Entropy` arm | same | function-local | OWNED + CRATE-OWNED, none | — | Same shape, entropy-rooted. |
| `crates/mnemonic-toolkit/src/cmd/bundle.rs:1026` | `entropy_at_0.clone()` into per-slot `ResolvedSlot.entropy` | `Vec<u8>` | full bundle build | OWNED, none | — | Cloned (not moved) so the entropy buffer is duplicated for the synthesize call. |
| `crates/mnemonic-toolkit/src/parse_descriptor.rs:858-865` | `bind_full_mode` `mnemonic`, `entropy`, `seed`, `master` | identical spine to derive_slot | function-local | OWNED + CRATE-OWNED, none | — | **Third** BIP-39→BIP-32 derivation spine (parallel to `derive_slot.rs` and `bundle_run_unified_descriptor`). `at0_xpriv` (line 880-882) also lives here. |
| `crates/mnemonic-toolkit/src/synthesize.rs:289-323` | `synthesize_multisig_full(seed_mnemonic, passphrase, …)` | `&Mnemonic` (borrowed) + 64-B `seed` + master Xpriv | function-local | BORROWED + OWNED + CRATE-OWNED, none | — | A fourth BIP-39→BIP-32 spine, for the legacy multisig path. |
| `crates/mnemonic-toolkit/src/synthesize.rs:393` | `synthesize_multisig_full` `let entropy = seed_mnemonic.to_entropy()` | `Vec<u8>` | function-local, moved into `Payload::Entr` | OWNED, none | — | Cloned-by-move into Payload. |
| `crates/mnemonic-toolkit/src/synthesize.rs:569-582` | `ResolvedSlot { entropy: Option<Vec<u8>>, … }` | per-slot entropy + Xpub (Xpub is public-only) | for full unified dispatch | OWNED, none | — | The canonical post-derivation carrier across the unified pipeline. Entropy moves via `clone()` (bundle.rs:1026) — 2× live. |
| `crates/mnemonic-toolkit/src/synthesize.rs:678-687` | `synthesize_unified` ms1 build | clones `e.clone()` into `Payload::Entr` | function-local | OWNED, none | — | 3rd entropy copy at this layer. |
| `crates/mnemonic-toolkit/src/bip85.rs:21-46` | `derive_entropy(master, …)` | `&Xpriv` + derived `child: Xpriv` + 64-byte HMAC output `[u8; 64]` | function-local + returned | BORROWED + CRATE-OWNED + OWNED stack, none | — | **BIP-85 master-secret derivation.** `child.private_key.secret_bytes()` (line 41) materializes the child privkey into the Hmac engine. The returned 64-B array is the *child seed* feeding all BIP-85 apps. |
| `crates/mnemonic-toolkit/src/bip85.rs:63-76` | `format_bip39_phrase` `entropy`, `Mnemonic`, returned phrase String | 64-B entropy buffer + Mnemonic + phrase | function-local then returned String | OWNED + CRATE-OWNED + STDOUT-LEAK, none | — | Returns a `String` containing a brand-new BIP-39 phrase. Printed to stdout at `derive_child.rs:201`. |
| `crates/mnemonic-toolkit/src/bip85.rs:88-102` | `format_hd_seed_wif` | `entropy[..32]` → `SecretKey` → WIF String | function-local then returned | CRATE-OWNED + OWNED String, none | — | WIF is itself the secret. |
| `crates/mnemonic-toolkit/src/bip85.rs:111-129` | `format_xprv_child` | 64-B entropy + chain-code + privkey scalar + new `Xpriv` + String | function-local then returned | CRATE-OWNED + OWNED, none | — | Emits a fresh master xprv. |
| `crates/mnemonic-toolkit/src/bip85.rs:137-176` | `format_hex_bytes`, `format_password_base64`, `format_password_base85` | 64-B entropy + encoded String | function-local then returned | OWNED, none | — | Hex/base64/base85 encoding of derived entropy. |
| `crates/mnemonic-toolkit/src/bip85.rs:193-244` | `format_dice_rolls` | 64-B entropy + SHAKE256 state + roll buffer + roll String | function-local then returned | OWNED, none | — | SHAKE state holds derived entropy. |
| `crates/mnemonic-toolkit/src/cmd/derive_child.rs:26` | `DeriveChildArgs::from: FromInput { value: String }` | xprv string OR phrase (depending on `node`) | full process | OWNED via clap, none | **YES** when `--from xprv=<value>` or `--from phrase=<value>` inline | `=-` syntax routes to stdin (good); inline value-form is argv-exposed. |
| `crates/mnemonic-toolkit/src/cmd/derive_child.rs:61` | `DeriveChildArgs::passphrase: Option<String>` | BIP-39 passphrase | full process | OWNED via clap, none | **YES** | No stdin alternative for derive-child passphrase. |
| `crates/mnemonic-toolkit/src/cmd/derive_child.rs:76-103` | `run()` locals `from_value` + `mnemonic` + `seed` + `master` | phrase + 64-B seed + Xpriv | function-local | OWNED + CRATE-OWNED, none | — | Same BIP-39→BIP-32 spine, 5th implementation. |
| `crates/mnemonic-toolkit/src/cmd/convert.rs:108-111` | `FromInput { value: String }` (Vec at `:147`) | phrase / entropy hex / xprv / wif / bip38 / minikey / electrum-phrase / ms1 | full process | OWNED via clap, none | **YES** | `--from phrase=<inline>` etc. Stdin alternative `=-` exists. |
| `crates/mnemonic-toolkit/src/cmd/convert.rs:165` | `ConvertArgs::passphrase: Option<String>` | BIP-39 passphrase / BIP-38 fallback | full process | OWNED via clap, none | **YES** | `--passphrase-stdin` (line 181) is the stdin escape hatch (preserves NULLs). Inline form still argv-exposed. |
| `crates/mnemonic-toolkit/src/cmd/convert.rs:175` | `bip38_passphrase: Option<String>` | BIP-38 Scrypt passphrase | full process | OWNED via clap, none | **YES** | **No stdin alternative** for `--bip38-passphrase`. NULL-byte passphrases (BIP-38 V3 spec) cannot be supplied via this flag at all. |
| `crates/mnemonic-toolkit/src/cmd/convert.rs:557-569` | `read_stdin_passphrase` `buf: String` | passphrase via stdin (preserves NULLs) | function-local then returned | OWNED, none | NO | Returned `String` becomes `effective_passphrase` at :614 → cloned into bookkeeping. |
| `crates/mnemonic-toolkit/src/cmd/convert.rs:621-625` | `primary_value: String` | the BIP-39 phrase / entropy hex / xprv / wif / etc | per-invocation | OWNED, none | YES if not stdin | Cloned from `primary.value`. |
| `crates/mnemonic-toolkit/src/cmd/convert.rs:832-967` | `compute_outputs` Phrase/Entropy arm | `entropy: Vec<u8>` + `derived` (DerivedAccount with xpriv) + per-target locals (`leaf_xpriv`, `pk`, `wif`) | function-local | OWNED + CRATE-OWNED, none | — | `leaf_xpriv.private_key` is moved into `PrivateKey { inner }` (line 895), `pk.to_wif()` allocates the WIF String. WIF is printed to stdout. |
| `crates/mnemonic-toolkit/src/cmd/convert.rs:1027-1063` | `Wif` arm | `pk: PrivateKey` + `pubkey` + `sentinel_xpub` | function-local | CRATE-OWNED, none | — | `PrivateKey::from_wif` holds a `SecretKey`. |
| `crates/mnemonic-toolkit/src/cmd/convert.rs:1065-1094` | `Bip38` arm | `(raw, compressed)` from decrypt + `SecretKey` + `pk: PrivateKey` | function-local | OWNED + CRATE-OWNED, none | — | `<str as Decrypt>::decrypt` returns the raw 32-B privkey via the bip38 crate. |
| `crates/mnemonic-toolkit/src/cmd/convert.rs:1095-1123` | `Ms1` arm | `entropy = ms_codec::decode → Payload::Entr(bytes)` + `Mnemonic` | function-local | OWNED + CRATE-OWNED, none | — | Symmetric to ms-cli decode. |
| `crates/mnemonic-toolkit/src/cmd/convert.rs:1155-1188` | `MiniKey` arm | `raw` 32-B privkey + `SecretKey` + `pk` + WIF String | function-local | OWNED stack + CRATE-OWNED + STDOUT-LEAK, none | — | `sha256::Hash::hash(value.as_bytes()).to_byte_array()` is the 32-B privkey scalar. |
| `crates/mnemonic-toolkit/src/cmd/convert.rs:1189-1213` | `ElectrumPhrase` arm | `entropy: Vec<u8>` | function-local | OWNED, none | — | Decoded Electrum seed entropy. |
| `crates/mnemonic-toolkit/src/electrum.rs:88-180` | `phrase_to_entropy` / `entropy_to_phrase` | `Vec<u8>` accumulator + per-step BigInt-style buffers | function-local | OWNED, none | — | The accumulator (`acc: Vec<u8>`) holds the secret integer during base-N decoding. |
| `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:43,218,273,352,425` | `VerifyBundleArgs::passphrase` + downstream `args.passphrase.as_deref()` calls | BIP-39 passphrase | full process | OWNED via clap, none | **YES** | Drives `resolve_slots` → derive spines. |
| `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:440-461,482` | `entropy_at_0: Option<Vec<u8>>` + per-cosigner `slot.entropy.clone()` | entropy | per-invocation | OWNED, none | — | Cloned twice per cosigner during descriptor verify. |

## §2 Hot-spots (worst secrecy/lifetime ratio)

Ranked by combined **(lifetime × multiplicity × argv-exposure ×
downstream reach)**:

1. **`SlotInput.value: String` (`slot_input.rs:71`, lifecycle full
   process)** — when subkey ∈ {phrase, entropy, wif, xprv}, this is
   the most-exposed buffer in the entire toolkit. The single
   `--slot @0.phrase="abandon … about"` invocation simultaneously
   (a) lands the phrase in `/proc/self/cmdline`, (b) lives in
   `BundleArgs.slot: Vec<SlotInput>` for the full process,
   (c) is cloned at `bundle.rs:182` for a 2nd copy, (d) is borrowed
   into `resolve_slots` and again into `derive_full`/
   `derive_bip32_from_entropy`. Every secret-bearing bundle /
   verify-bundle / convert invocation routes through this single field.

2. **`derive_slot::derive_bip32_from_entropy` local `seed: [u8; 64]`
   (`derive_slot.rs:32`)** — the 64-byte BIP-32 master seed
   (PBKDF2-HMAC-SHA512 output) is the highest-value derived secret in
   the system. It exists on the stack alongside the source `Mnemonic`
   and the just-built `Xpriv master`. Same shape replicated in **five
   other locations**: `synthesize.rs:323`, `bundle.rs:887,954`,
   `parse_descriptor.rs:861`, `cmd/derive_child.rs:89`. None scrub.

3. **`bitcoin::bip32::Xpriv` (CRATE-OWNED, six call sites)** — master
   Xpriv + account Xpriv + leaf Xpriv across `derive.rs`,
   `derive_slot.rs`, `bundle.rs`, `parse_descriptor.rs`,
   `synthesize.rs`, `bip85.rs`, `cmd/convert.rs`, `cmd/derive_child.rs`.
   The `bitcoin` crate's `Xpriv` is `Copy`-able (the inner `SecretKey`
   from secp256k1 is `Copy`!), which actively defeats zeroize: copies
   leave behind un-scrubbed stack/heap residue at every assignment.

4. **`bip85::derive_entropy` returned `[u8; 64]` (`bip85.rs:21-46`)** —
   long-lived through `format_*` callees in `derive-child`. Returned by
   value, then sliced and re-used in chained derivations. Every BIP-85
   invocation (in-scope: BIP-39 / hd-seed / xprv / hex / base64 / base85
   / dice) materializes this buffer.

5. **`ConvertArgs.passphrase` + `bip38_passphrase` + `passphrase`
   mirror in BundleArgs / VerifyBundleArgs / DeriveChildArgs
   (`Option<String>`, all argv-exposed)** — every passphrase flag
   inline-form lands in `/proc/N/cmdline`. `--bip38-passphrase` has
   **no stdin escape** at all, so BIP-38 use is forced through argv.

Honorable mentions:
- `bip39::Mnemonic` instances (CRATE-OWNED, every command). Each holds
  the wordlist-resolved phrase verbatim.
- `ms-cli` `read_stdin()` raw buffer (parse.rs:44-50) — un-scrubbed,
  even though the trimmed copy supersedes it.
- `cmd::convert::compute_outputs` Phrase/Entropy arm `leaf_xpriv`
  (`:887`) — destructured into a `PrivateKey { inner }`, leaving the
  original Xpriv on the stack.

## §3 Third-party gaps (no zeroize-on-drop)

| Type | Crate / version | Holds | Wrappable? |
|------|-----------------|-------|------------|
| `bip39::Mnemonic` | bip39 = "2" (both repos) | full phrase text + wordlist binding | **Partially.** Cannot wrap the type itself. Workaround: pass phrase as `Zeroizing<String>` to `Mnemonic::parse_in`, then immediately call `to_entropy()` / `to_seed()` and drop the Mnemonic ASAP. The Mnemonic's interior wordlist-index buffer remains un-scrubbed; upstream PR needed to fix completely. |
| `bip39::Mnemonic::to_seed()` → `[u8; 64]` | bip39 = "2" | 64-B BIP-32 master seed | **Yes.** Returned by value as `[u8; 64]`; can be received into `let seed = Zeroizing::new(mnemonic.to_seed(...))` (`Zeroizing<[u8; 64]>`) at each of the 6 call sites. |
| `bip39::Mnemonic::to_entropy()` → `Vec<u8>` | bip39 = "2" | 16-32 B entropy | **Yes.** Wrap: `Zeroizing::new(mnemonic.to_entropy())`. Used in encode.rs, decode.rs, convert.rs, bundle.rs, parse_descriptor.rs, synthesize.rs. |
| `bitcoin::bip32::Xpriv` | bitcoin = "0.32" | 32-B privkey scalar + 32-B chain code + 4-B fingerprint + depth/network | **Partially.** Xpriv is `Copy` (inherits from `secp256k1::SecretKey: Copy`). Wrapping each binding in `Zeroizing<Xpriv>` only scrubs *that* binding's bytes — every `xpriv.derive_priv()` / `Xpub::from_priv(&secp, &xpriv)` call copies the bytes. Real fix requires upstream `bitcoin` (and `secp256k1`) to remove `Copy` and implement `Zeroize`. Until then, this is **upstream-blocked**. |
| `bitcoin::secp256k1::SecretKey` | secp256k1 = "0.29" (via bitcoin 0.32) | 32-B scalar | **Upstream-blocked.** Same `Copy` issue. Note: secp256k1 has a `global-context` feature but no zeroize feature gate. |
| `bitcoin::PrivateKey` | bitcoin = "0.32" | wraps `SecretKey` + flags | **Upstream-blocked** (same root cause). |
| `bip38::Error` / `bip38` crate API | bip38 = "1.1" | crate's internal decrypt buffers + returned `(Vec<u8>, bool)` from `Decrypt::decrypt` (the 32-B raw privkey) | **Partially.** Crate internals out of reach; the *returned* tuple can be wrapped: `let (raw, compressed) = ...; let raw = Zeroizing::new(raw);` Crate internals are upstream-only. |
| `codex32::Codex32String` (rust-codex32 v0.1) | codex32 (workspace dep) | full validated codex32 string + interior `Parts` data | **Upstream-only.** `Parts::data() -> Vec<u8>` returns a fresh allocation (good), but the underlying string-form held by `Codex32String` is the secret in chunk-text form. Fields are non-pub (per envelope.rs:7-13 comments). |
| `hex::encode(&entropy)` → `String` | hex = "0.4" | hex-encoded copy of any secret bytes | **Yes (caller-side).** The returned `String` is OWNED and could be wrapped before stdout flush — though if destined for stdout it's leaking anyway. |
| `sha3::Shake256` XOF reader state | sha3 = "0.10" | absorbs the 64-B BIP-85 entropy, produces dice-roll bytes | **Upstream-only.** Shake256 state could carry secret material until drop. |

## §4 mlock candidates

`mlock(2)` makes sense for **heap allocations whose lifetime spans a
measurable interval and whose pages would otherwise be eligible for
swap**. Stack-locals in short-lived helper functions are usually not
worth mlock's overhead (page-pinning, RLIMIT_MEMLOCK pressure,
`CAP_IPC_LOCK` requirement on lower ulimits). Prioritized list:

1. **`clap::Args`-derived passphrase / phrase / slot-value fields** —
   these `String`/`Vec<SlotInput>` allocations live for the *entire
   process lifetime* and are visible from `/proc/N/cmdline` (worst
   case). If anything in the codebase warrants mlock-on-arrival, it
   is the clap-parsed argv. Touch points: `BundleArgs.passphrase`,
   `BundleArgs.slot[i].value`, `VerifyBundleArgs.passphrase`,
   `VerifyBundleArgs.slot[i].value`, `ConvertArgs.from[i].value`,
   `ConvertArgs.passphrase`, `ConvertArgs.bip38_passphrase`,
   `DeriveChildArgs.from.value`, `DeriveChildArgs.passphrase`,
   `EncodeArgs.phrase` / `EncodeArgs.hex`, `VerifyArgs.phrase`.
   (Note: mlock cannot retroactively cover the copy in
   `/proc/self/cmdline` — that's a libc/kernel issue, addressable
   only by `prctl(PR_SET_DUMPABLE)` or rewriting `argv[]` post-parse.)

2. **`ResolvedSlot.entropy: Option<Vec<u8>>` (synthesize.rs:575)** —
   lives from `resolve_slots()` exit through `synthesize_unified()` /
   `synthesize_descriptor()` and the entire emission phase. Cloned at
   least twice along the way (bundle.rs:1026; synthesize.rs:682 inside
   `Payload::Entr(e.clone())`). This is the canonical "secret bundle
   carrier" — mlock'ing the underlying `Vec<u8>` buffer would protect
   the longest-lived secret on the toolkit's hot path.

3. **`DerivedAccount` (derive.rs:13-20)** — holds `entropy: Vec<u8>` +
   `account_xpriv: Xpriv`. Lifetime spans the whole bundle synthesis.
   The `Vec<u8>` allocation backing entropy is on the heap.

4. **`bip85::derive_entropy` returned `[u8; 64]` and downstream
   `entropy[..]` slicings** — moved through 6 `format_*` functions.
   These are *stack* allocations today; an mlock-friendly design
   would heap-allocate this into a `Zeroizing<[u8; 64]>` boxed into a
   struct and mlock the page. Lower priority than #1-#3.

5. **`ms-cli` `read_stdin()` String buffer (parse.rs:45)** — long
   enough lifetime to potentially span a context switch; mlock would
   prevent the raw stdin buffer from being paged out before
   normalization completes.

Lower priority (short stack lifetimes that mlock would mostly waste):
the `seed: [u8; 64]` locals in derive_slot / synthesize /
parse_descriptor / cmd/derive_child / bundle. Address with
`Zeroizing<[u8; 64]>` heap-promotion first; mlock those *if* they get
heap-promoted as part of the v0.9 refactor.

## §5 argv-leakage hot-spots — every `clap::Args` field that takes a secret inline

Each row is a flag whose inline value lands in `/proc/N/cmdline` for
the full process lifetime. "Stdin alternative?" = whether the user has
a way to avoid argv exposure today.

### mnemonic-secret (`ms` CLI)

| Flag | File:line | Secret class | Stdin alternative? |
|------|-----------|--------------|---------------------|
| `ms encode --phrase <PHRASE>` | `ms-cli/src/cmd/encode.rs:30` | BIP-39 phrase | YES (`--phrase -`) |
| `ms encode --hex <HEX>` | `ms-cli/src/cmd/encode.rs:34` | BIP-39 entropy (hex) | YES (`--hex -`) |
| `ms verify --phrase <PHRASE>` | `ms-cli/src/cmd/verify.rs:27` | BIP-39 phrase | YES (`--phrase -`) |
| `ms decode <MS1>` (positional) | `ms-cli/src/cmd/decode.rs:22` | ms1 string (encrypted-form-equivalent: the entropy can be recovered) | YES (`-` or omit) |
| `ms verify <MS1>` (positional) | `ms-cli/src/cmd/verify.rs:21` | ms1 string | YES (`-` or omit) |

### mnemonic-toolkit (`mnemonic` CLI)

| Flag | File:line | Secret class | Stdin alternative? |
|------|-----------|--------------|---------------------|
| `mnemonic bundle --passphrase <PP>` | `cmd/bundle.rs:42` | BIP-39 passphrase | **NO** |
| `mnemonic bundle --slot @N.phrase=<PHRASE>` | `slot_input.rs:71` (via `cmd/bundle.rs:77`) | BIP-39 phrase | **NO** (no `=-` carve-out documented; parse_slot_input does not special-case `-`) |
| `mnemonic bundle --slot @N.entropy=<HEX>` | same | BIP-39 entropy | **NO** |
| `mnemonic bundle --slot @N.wif=<WIF>` | same | WIF privkey | **NO** |
| `mnemonic bundle --slot @N.xprv=<XPRV>` | same | BIP-32 xprv (currently rejected at runtime per v0.4.2; spec-reserved for v0.5+) | **NO** |
| `mnemonic verify-bundle --passphrase <PP>` | `cmd/verify_bundle.rs:43` | BIP-39 passphrase | **NO** |
| `mnemonic verify-bundle --slot @N.<secret>=<…>` | `cmd/verify_bundle.rs:88` | as above | **NO** |
| `mnemonic convert --from phrase=<PHRASE>` | `cmd/convert.rs:147` (via `parse_from_input`) | BIP-39 phrase | YES (`--from phrase=-`) |
| `mnemonic convert --from entropy=<HEX>` | same | BIP-39 entropy | YES (`=-`) |
| `mnemonic convert --from xprv=<XPRV>` | same | BIP-32 xprv | YES (`=-`) |
| `mnemonic convert --from wif=<WIF>` | same | WIF privkey | YES (`=-`) |
| `mnemonic convert --from ms1=<MS1>` | same | ms1 string | YES (`=-`) |
| `mnemonic convert --from bip38=<...>` | same | BIP-38 encrypted key (lower risk; useless without passphrase) | YES (`=-`) |
| `mnemonic convert --from minikey=<…>` | same | Casascius mini-key | YES (`=-`) |
| `mnemonic convert --from electrum-phrase=<…>` | same | Electrum native seed | YES (`=-`) |
| `mnemonic convert --passphrase <PP>` | `cmd/convert.rs:165` | BIP-39 passphrase | YES (`--passphrase-stdin`, preserves NULLs) |
| `mnemonic convert --bip38-passphrase <PP>` | `cmd/convert.rs:175` | BIP-38 Scrypt passphrase | **NO** (no `--bip38-passphrase-stdin` flag; cannot represent NULL-byte passphrases — gap noted by SPEC v0.8 §5.a only for the BIP-39 channel) |
| `mnemonic derive-child --from xprv=<XPRV>` | `cmd/derive_child.rs:26` | BIP-32 master xprv | YES (`=-`) |
| `mnemonic derive-child --from phrase=<PHRASE>` | same | BIP-39 phrase | YES (`=-`) |
| `mnemonic derive-child --passphrase <PP>` | `cmd/derive_child.rs:61` | BIP-39 passphrase | **NO** |

**Argv-leakage worst offenders (no stdin alternative exists today):**

1. `mnemonic bundle --slot @N.<secret>=<value>` — the entire v0.4+
   secret-input grammar lacks a `=-` stdin escape. (The non-slot
   `--from <node>=-` syntax in `convert` shows it is parser-feasible.)
2. `mnemonic bundle --passphrase` / `mnemonic verify-bundle --passphrase` /
   `mnemonic derive-child --passphrase` — no stdin alternative
   anywhere except `convert`'s `--passphrase-stdin`.
3. `mnemonic convert --bip38-passphrase` — the *only* passphrase flag
   in the entire toolkit with no stdin route, and BIP-38 V3 explicitly
   permits NULL-byte passphrases that argv cannot carry.

## §6 Misc cross-cutting observations

- **No dependency on `zeroize` declared in any Cargo.toml** of either
  repo. The v0.9 cycle will need a coordinated addition to
  `crates/ms-codec/Cargo.toml`, `crates/ms-cli/Cargo.toml`, and
  `crates/mnemonic-toolkit/Cargo.toml`. ms-codec adding `zeroize` may
  force a ms-codec minor version bump (the public `Payload::Entr(Vec<u8>)`
  shape would ideally become `Payload::Entr(Zeroizing<Vec<u8>>)`, but
  that's a breaking API change — alternative is `#[non_exhaustive]`
  already in place + internal-only zeroize discipline in encode/decode
  helpers).
- **Five parallel BIP-39→BIP-32 derivation spines**
  (`derive_slot.rs`, `synthesize.rs:288-326`, `bundle.rs:884-905`,
  `bundle.rs:951-963`, `parse_descriptor.rs:858-865`,
  `cmd/derive_child.rs:84-95`) mean any zeroize discipline must be
  applied at all five sites. Consolidating these into a single helper
  would be a strong v0.9 preparatory move.
- **Test fixtures** under `synthesize.rs` (lines 849, 886-895, 935,
  968, 1004-1008, 1158-1162) construct `Mnemonic` + `seed` + `Xpriv`
  in `#[cfg(test)]` blocks. These are zero-leak-risk in production
  but should still use whatever zeroize wrapper the audit produces,
  to avoid divergent patterns and to make grep for "to_seed" useful
  as a discipline tripwire.
- **The `secret-on-stdout` warning** (`bundle.rs:697`, `convert.rs:799`,
  `derive_child.rs:205`) is currently the only secrecy advisory the
  toolkit emits. It is uniform byte-for-byte — good. Suggest a parallel
  `secret-in-argv` warning when clap parses an inline secret value
  while a `=-` / stdin alternative exists, to nudge users to the safer
  path.
- **`bip39::Mnemonic`** (used everywhere) is the only mandatory
  CRATE-OWNED secret carrier in the BIP-39 layer; the *only* way to
  avoid its un-scrubbed drop is to minimize its lifetime — construct
  it, call `to_entropy()` / `to_seed()` into a `Zeroizing` wrapper,
  immediately let it go out of scope. Mirroring the GUI's
  `SecretLineEdit` discipline at the CLI's clap-parsed-arg boundary
  would tighten the worst hot-spot (#1 above).

## §7 File inventory — v0.9.0 implementation cycle touch points

Absolute paths in scope:

- `/scratch/code/shibboleth/mnemonic-secret/crates/ms-codec/src/payload.rs`
- `/scratch/code/shibboleth/mnemonic-secret/crates/ms-codec/src/decode.rs`
- `/scratch/code/shibboleth/mnemonic-secret/crates/ms-codec/src/encode.rs`
- `/scratch/code/shibboleth/mnemonic-secret/crates/ms-codec/src/envelope.rs`
- `/scratch/code/shibboleth/mnemonic-secret/crates/ms-cli/src/parse.rs`
- `/scratch/code/shibboleth/mnemonic-secret/crates/ms-cli/src/cmd/encode.rs`
- `/scratch/code/shibboleth/mnemonic-secret/crates/ms-cli/src/cmd/decode.rs`
- `/scratch/code/shibboleth/mnemonic-secret/crates/ms-cli/src/cmd/verify.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/derive.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/derive_slot.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/bip85.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/synthesize.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/parse_descriptor.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/slot_input.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/bundle.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/verify_bundle.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/convert.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/derive_child.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/electrum.rs`
