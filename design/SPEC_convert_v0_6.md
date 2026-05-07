# mnemonic-toolkit v0.6 SPEC — `convert` subcommand

**Version:** 0.7.0
**Date:** 2026-05-06
**Status:** v0.7.0 amendments DRAFT (v0.6.0 architect-approved 0C/0I at r3; v0.6.1/v0.6.2 amendments architect-approved 0C/0I; v0.7.0 amendments converged 0C/0I after 3 user-rounds + 2 architect-rounds — see SPEC change history below)
**Predecessor:** [SPEC_mnemonic_toolkit_v0_5.md](SPEC_mnemonic_toolkit_v0_5.md) (covers `bundle` / `verify-bundle`).

This SPEC covers a new orthogonal subcommand `mnemonic convert`. v0.5 SPEC carries forward unchanged.

**v0.6.1 amendment summary** (see in-section markers `(v0.6.1)` for the exact deltas):

- **SPEC-A** — `phrase/entropy → wif` edge moved from deferred-in-code to supported (§2 edge addition + §2 path-requirement note + §8 PBKDF2 cross-reference). No spike required: derivation is well-understood BIP-32 + WIF serialization atop the existing `derive_slot::derive_bip32_from_entropy` shape (the v0.6.0 convert-cycle spike `design/agent-reports/spike-convert-v0_6_0-pre-spec.md` covers the BIP-39/BIP-32 surface).
- **SPEC-B** — new §11 documenting the SLIP-0132 prefix-tolerant input normalizer (applied universally wherever the toolkit reads an xpub).
- **SPEC-C** — new §11.a documenting the `--xpub-prefix <variant>` output flag (5-value modifier; `--network` required when non-default).
- **SPEC-D** — partner amendment in `SPEC_mnemonic_toolkit_v0_5.md` extending the §7 secret-on-stdout warning to `bundle`.

**v0.7.0 amendment summary** (see in-section markers `(v0.7)` for the exact deltas):

- **SPEC-E** — §1 NodeType extension: 4 new variants (`Bip38`, `MiniKey`, `ElectrumPhrase`, `Address`); `Slip39Shares` deferred to v0.8.
- **SPEC-F** — §2 edge table: 7 new edges (BIP-38 encrypt/decrypt, mini-key decode, Electrum phrase ↔ entropy, xpub → address, sibling-pivot refusal for `Phrase ↔ ElectrumPhrase`).
- **SPEC-G** — §3 refusal taxonomy: explicit enumeration of new "obvious" refusal pairs (Bip38↔Bip38 identity; Bip38→non-Wif via Wif intermediate; MiniKey→non-Wif; Address→*; *→MiniKey; *→Address-without-path; 2FA Electrum).
- **SPEC-H** — §10 address derivation: replaces v0.6.1 "deferred to v0.7" stub with the full spec.
- **SPEC-I** — §11 SLIP-0132: HTML-comment fence markers around the canonical info-line block; verify-bundle Option B clause documented.
- **SPEC-J** — new §12 BIP-38 encrypted WIF (Scrypt parameters, NFC normalization, refusal taxonomy).
- **SPEC-K** — new §13 Casascius mini-key (SHA256 self-checksum, decode-only contract).
- **SPEC-L** — new §14 Electrum native seed format (wordlist provenance, `SeedVersion` enum, HMAC-SHA512 prefix dispatch, 2FA refusal class).

## §0 Prerequisites

- v0.5.2 SHIPPED — `derive_slot::derive_bip32_from_entropy` available as the shared BIP-39 + BIP-32 derivation helper.
- ms-codec v0.1.0, mk-codec v0.2.1, md-codec v0.16.1 are the active codec library versions. Public surface verified via `design/agent-reports/spike-convert-v0_6_0-pre-spec.md`.

## §1 Node table

The v0.6/v0.7 conversion graph nodes:

| Node | Wire/in-memory shape | Annotations |
|------|----------------------|-------------|
| `phrase` | UTF-8 BIP-39 mnemonic | language-aware (12/15/18/21/24 words; 9 wordlists) |
| `entropy` | hex-encoded bytes | BIP-39-valid lengths only: 16/20/24/28/32 bytes |
| `xpub` | base58check BIP-32 extended pubkey | depth/chaincode-bearing |
| `xprv` | base58check BIP-32 extended privkey | depth/chaincode-bearing |
| `wif` | base58check Wallet Import Format | single privkey, ± compression flag, no chain code |
| `fingerprint` | 4-byte hex | derived from xpub or xprv |
| `path` | BIP-32 path string | informational; not accepted as a primary `--from` |
| `ms1` | codex32 (ms-codec) | tag = ENTR; carries entropy |
| `mk1` | codex32 (mk-codec) | carries xpub + origin metadata + policy_id_stubs |
| `bip38` (v0.7) | base58check BIP-38 encrypted WIF | non-EC-multiplied form; requires `--passphrase`; secret-bearing |
| `minikey` (v0.7) | Casascius mini-private-key | 22/26/30-char base58 string with SHA256 self-checksum; secret-bearing; decode-only |
| `electrum-phrase` (v0.7) | Electrum native seed phrase | own wordlist (NOT BIP-39); HMAC-SHA512 version-prefix dispatch; secret-bearing |
| `address` (v0.7) | Bitcoin address (bech32 / base58check) | hash of pubkey; one-way terminal |

**v0.7 NodeType `from_token` strings (locked per architect R1-I7):** `"bip38" → Bip38`, `"minikey" → MiniKey` (no hyphen, consistent with `mk1`/`ms1`), `"electrum-phrase" → ElectrumPhrase` (hyphen acceptable; multi-word with no ambiguity), `"address" → Address`.

**Deferred to v0.8 (not added in v0.7):** `Slip39Shares` (SLIP-39 group/member shares) — hand-roll required (no maintained Rust crate verified GREEN as of 2026-05-06); deferred to a focused v0.8 cycle OR closed wont-fix in favor of ms1-shares (the family's v0.2 share-encoding mechanism via codex32).

### Deferred nodes (not refused; awaiting upstream)

- `seed` (BIP-32 master seed): reserved.
- `raw_privkey` (32-byte hex secp256k1 scalar): reserved.
- `xprv`-via-ms1 / `seed`-via-ms1: gated on ms-codec v0.2 shipping `XPRV` / `SEED` tags.

These nodes are listed for forward-compatibility documentation; v0.6.0 does not accept or emit them. Their addition is an additive change for v0.7+.

### Excluded nodes (deliberate)

- `md1`: descriptor encoding. Descriptors are bundle artifacts, not single-key conversions. Use `mnemonic bundle --descriptor ...` instead.

## §2 Edge table (adjacency)

Bidirectional and one-way edges with required side-inputs:

| From | To | Required side-input | Mechanism |
|------|----|--------------------|-----------|
| `phrase` | `entropy` | `--language` (optional; default english) | BIP-39 wordlist reverse mapping |
| `entropy` | `phrase` | `--language` (optional; default english) | BIP-39 wordlist forward mapping |
| `phrase` | `xprv` / `xpub` / `fingerprint` | `--passphrase` (optional, default ""), `--path` OR (`--template` + optional `--account`) | parse phrase → entropy via `Mnemonic::parse_in(language, phrase)?.to_entropy()`, then `derive_slot::derive_bip32_from_entropy` (yields `DerivedAccount` with `account_xpriv` + `account_xpub`); equivalent to calling `derive::derive_full` |
| `entropy` | `xprv` / `xpub` / `fingerprint` | same as `phrase` | feed entropy bytes directly to `derive_slot::derive_bip32_from_entropy` (no parse step) |
| `xprv` | `xpub` | none | `Xpub::from_priv` (EC derive) |
| `xprv` | `fingerprint` | none | hash160 of pubkey |
| `xpub` | `fingerprint` | none | hash160 of pubkey |
| `xpub` | `xpub` (v0.6.1) | none (with `--xpub-prefix`: `--network` required) | Encoding-only normalization. Without `--xpub-prefix` (or with `--xpub-prefix xpub`), the input is decoded via §11 (accepts SLIP-0132) and re-emitted as the neutral `xpub`/`tpub`. With `--xpub-prefix <non-default>` per §11.a, the input is normalized then re-encoded with the requested SLIP-0132 prefix. Key material is unchanged; this is the round-trip symmetry primitive cited in §11.a. |
| `wif` | `xpub` (sentinel) | none | depth-0 sentinel xpub with zero chain code; matches `bundle.rs::resolve_slots` WIF behavior — stderr warning emitted |
| `phrase` / `entropy` | `wif` (v0.6.1) | explicit `--path` (any valid BIP-32 path; no depth assertion); `--passphrase` (optional, default ""; meaningful per §8 — this edge traverses PBKDF2); `--network` (optional, default mainnet — affects WIF version byte) | Phrase source: `Mnemonic::parse_in(language, phrase)?.to_entropy()` → shared entropy path. Entropy source: entropy bytes directly. Then: `Mnemonic::from_entropy_in(language, entropy).to_seed(passphrase)` → `Xpriv::new_master(network, seed)` → derive at `--path` → `bitcoin::PrivateKey { compressed: true, network: network.network_kind(), inner: derived_xpriv.private_key }` → `to_wif()`. The `compressed` flag MUST be `true` (BIP-32 §4 mandates compressed pubkeys for all derived keys; WIF compression follows the BIP-32 contract, not the network or input-WIF source flag). |
| `entropy` | `ms1` | none | `ms_codec::encode(Tag::ENTR, &Payload::Entr(bytes))` |
| `ms1` | `entropy` | none | `ms_codec::decode(s) -> (Tag, Payload)`; pattern-match `Payload::Entr(bytes)` |
| `mk1` | `xpub` (+ fingerprint + path as sub-outputs) | none | `mk_codec::decode(&[&str]) -> KeyCard`; `policy_id_stubs` ignored |
| `wif` | `bip38` (v0.7) | `--passphrase` (mandatory; NFC-normalized) | `bip38::EncryptWif::encrypt(wif, passphrase)` → BIP-38 non-EC-multiplied form (Scrypt n=16384, r=8, p=8); secret-bearing output |
| `bip38` | `wif` (v0.7) | `--passphrase` (mandatory; NFC-normalized) | `bip38::Decrypt::decrypt(bip38_str, passphrase)` → recovered WIF; mismatched passphrase emits `refusal_bip38_passphrase_mismatch` |
| `minikey` | `wif` (v0.7) | `--network` (optional; default mainnet) | Verify SHA256(mini_key + "?")[0] == 0x00; if checksum-ok, raw privkey = SHA256(mini_key); compressed=false (Casascius keys predate BIP-32 compressed-pubkey convention); `bitcoin::PrivateKey { compressed: false, network, inner: privkey_scalar }.to_wif()`. **Decode-only contract:** no reverse `(Wif, MiniKey)` edge — generating a valid mini-key requires brute-force search for the typo-checksum byte. |
| `electrum-phrase` | `entropy` (v0.7) | none | `electrum::validate_seed_version(phrase) -> SeedVersion`; if `Standard`/`Segwit`, `electrum::phrase_to_entropy(phrase, version) -> Vec<u8>` (own wordlist + HMAC-SHA512 verify); if `Standard2FA`/`Segwit2FA`, refuse via `refusal_electrum_2fa_unsupported`. |
| `entropy` | `electrum-phrase` (v0.7) | `--electrum-version <standard\|segwit>` (optional; default `standard`) | `electrum::entropy_to_phrase(entropy, version) -> String`; HMAC-SHA512(`"Seed version" \|\| candidate_phrase`) hex-prefix must match the requested version's discriminator (`01` for Standard, `100` for Segwit). 2FA versions (`101`/`102`) NOT exposed as encode targets. |
| `xpub` | `address` (v0.7) | `--path <BIP32-path>` (mandatory); `--script-type <p2wpkh\|p2sh-p2wpkh\|p2tr>` (mandatory unless inferable from `--template`); `--network` (optional; inferred from xpub prefix when absent) | `bitcoin::bip32::Xpub::derive_pub(secp, &path)` → child xpub → compressed pubkey → dispatch to `Address::p2wpkh` / `Address::p2shwpkh` / `Address::p2tr` per `--script-type`. **One-way:** address is a hash; no reverse edge. Missing `--path` → `refusal_address_no_path`. (v0.7 — see §10.a) |
| `phrase` | `electrum-phrase` (v0.7) | — | **REFUSE** (sibling-pivot): BIP-39 and Electrum native seeds are different artifact classes with different wordlists and validation rules; no meaningful direct conversion. Cross-format pivot via bundle, not single-format conversion. |
| `electrum-phrase` | `phrase` (v0.7) | — | **REFUSE** (sibling-pivot): same rationale as forward direction. |

### `phrase`/`entropy` → `wif` path requirement (v0.6.1)

The `phrase`/`entropy` → `wif` edge requires `--path` to be supplied. The toolkit does NOT auto-default a path from `--template`/`--account`. No depth assertion is made (BIP-32 depth is a counter, not a normative constraint); the user is responsible for supplying a path that produces a leaf privkey suitable for WIF serialization. Refusal stderr when `--path` is absent (byte-exact):

```
error: --to wif requires explicit --path; supply a BIP-32 path producing a leaf privkey (the toolkit does not auto-default a path from --template/--account).
```

Exit code: 2 (refusal class via `ToolkitError::ConvertRefusal`). NOT exit 1 (BadInput class) — this is a §3 refusal of an under-specified invocation, not a parse error of malformed input.

### Composite edges (graph traversal)

Edges not directly in the table are realized by graph traversal. Examples:
- `phrase → ms1` = `phrase → entropy → ms1`
- `entropy → xprv` and `entropy → xpub` are both produced by a single `derive_slot::derive_bip32_from_entropy` call (the returned `DerivedAccount` carries both `account_xpriv` and `account_xpub`).

The dispatcher walks the graph BFS from `--from` to each requested `--to` node, emitting in `--to` argument order.

### Secp context

Non-BIP-39 edges (`xprv → xpub`, `xprv → fingerprint`, `xpub → fingerprint`, `wif → xpub`-sentinel) instantiate `Secp256k1::new()` directly inside `cmd/convert.rs` (consistent with `bundle.rs:298` post-v0.5.1). They do NOT route through `derive_slot::derive_bip32_from_entropy`. Only the BIP-39-rooted edges (`phrase` / `entropy` → BIP-32 derivatives) call the shared helper.

## §3 Refusal taxonomy

Three classes; each refusal exits 2 with byte-exact stderr.

### §3.a One-way cryptographic barrier

Public material has no preimage for the secret. Edges:
- `xpub → xprv`, `xpub → entropy`, `xpub → phrase`, `xpub → wif`
- `mk1 → entropy`, `mk1 → phrase`, `mk1 → xprv`, `mk1 → wif`
- `fingerprint → *` (every node) — fingerprint is hash160 of the pubkey; the inverse direction recovers nothing
- `wif → entropy`, `wif → phrase`, `wif → xprv` — WIF is a single privkey scalar without BIP-32 chain code; cannot recover the BIP-39 entropy or BIP-32 master xpriv that derived it

Stderr (byte-exact):
```
error: --to <to_node> is cryptographically unrecoverable from --from <from_node> (one-way derivation barrier)
```

`<from_node>` and `<to_node>` are interpolated from the user's invocation (lowercase node name).

### §3.b Lossy compression barrier

PBKDF2 salt (passphrase) is unrecoverable from the master xpriv. The `xprv → seed` direction is impossible: the BIP-32 master xpriv is derived from the seed via HMAC-SHA512, but neither the seed nor the original passphrase is recoverable from the master. Currently moot: `seed` is a deferred node (§1). Reserved for v0.7+ when `seed` becomes a node.

Stderr template (reserved for v0.7+):
```
error: --to seed is unrecoverable from --from xprv (HMAC-SHA512 master derivation is one-way; PBKDF2 passphrase salt is also not stored)
```

### §3.c Type-class mismatch / cross-format pivot

Edges that are different artifact classes — these are bundle compositions, not single-format conversions:
- `ms1 → mk1`, `ms1 → md1` (where md1 is recognized as input)
- `mk1 → ms1`, `mk1 → md1`
- `md1 → ms1`, `md1 → mk1`

**Sibling-pivot stderr (byte-exact):**
```
error: --from <from_node> --to <to_node> is a sibling-format pivot, not a single-format conversion. Use 'mnemonic bundle' instead.
```

**`xpub → mk1` REFUSED in v0.6.0** — distinct refusal with a more specific message. mk1 cards bind xpubs to specific policies via `policy_id_stubs` (a non-empty `Vec<[u8; 4]>` from the descriptor's wallet policy ID). Encoding a standalone xpub to mk1 with a fabricated zero-stub produces a malformed-by-intent card. The workflow that needs xpub + descriptor → mk1 is `mnemonic bundle`. Spike memo: `design/agent-reports/spike-convert-v0_6_0-pre-spec.md`.

**`xpub → mk1` stderr (byte-exact):**
```
error: --to mk1 requires a policy descriptor binding (mk1 cards bind xpubs to specific policies via policy_id_stubs). Use 'mnemonic bundle --slot @0.xpub=... --template ...' to emit a complete bundle.
```

### §3.d v0.7 refusal-table completeness

The new v0.7 NodeTypes interact with existing edges through the catch-all refusal paths. Per architect R1-L11, the SPEC explicitly enumerates the new "obvious" refusal pairs so the code-reviewer can verify coverage:

| from | to | refusal class |
|---|---|---|
| `bip38` | `electrum-phrase`, `address`, `mk1`, `ms1`, `phrase`, `entropy`, `xpub`, `xprv`, `fingerprint` | one-way / sibling-pivot via `wif` intermediate (decrypt → wif → target) |
| `bip38` | `bip38` | identity-pivot refusal (same NodeType source and target) |
| `minikey` | anything except `wif` | sibling-pivot via `wif` intermediate (decode → wif → target) |
| `*` | `minikey` | one-way (mini-key generation requires brute-force search for typo-checksum byte) |
| `*` (any node where source is not `xpub`/`xprv`-derived) | `address` | composite via `xpub` intermediate; missing-arg refusal `refusal_address_no_path` if `--path` absent |
| `address` | `*` (every node) | one-way (address = hash160/sha256 of pubkey or scripthash) |
| `electrum-phrase` (2FA seed-versions `101`/`102`) | `entropy` | `refusal_electrum_2fa_unsupported` |
| `electrum-phrase` | `electrum-phrase` | identity-pivot refusal |
| `phrase` ↔ `electrum-phrase` | — | sibling-pivot refusal (different artifact classes) |

**`refusal_bip38_passphrase_mismatch` stderr (byte-exact):**
```
error: BIP-38 decryption failed: passphrase does not match the encrypted key (per BIP-38 §"Decryption" address-hash check).
```

**`refusal_address_no_path` stderr (byte-exact):**
```
error: --to address requires --path (xpub does not carry an origin path; supply BIP-32 derivation explicitly).
```

**`refusal_electrum_2fa_unsupported` stderr (byte-exact):**
```
error: Electrum 2FA seed (version 101 or 102) requires a second factor not present in the phrase alone; conversion not supported. Use Electrum directly for 2FA recovery.
```

**`refusal_electrum_phrase_pivot` stderr (byte-exact):**
```
error: --from phrase --to electrum-phrase (or reverse) is a sibling-format pivot, not a single-format conversion. BIP-39 and Electrum native seeds are different artifact classes.
```

**`refusal_minikey_one_way` stderr (byte-exact, used for both `--to minikey` and `minikey → non-wif` cascades):**
```
error: --to minikey is one-way (mini-key generation requires brute-force search for typo-checksum byte; no inverse derivation).
```

**`refusal_bip38_identity` stderr (byte-exact):**
```
error: --from bip38 --to bip38 is an identity pivot. To re-encrypt with a different passphrase, decrypt to wif then re-encrypt.
```

## §4 Specific refusal cases

- `wif → xpub --path m/...`: REFUSED. Chain code is destroyed in WIF serialization; derivation from a WIF is impossible. Stderr (byte-exact):
  ```
  error: --from wif does not retain a chain code; --path-driven derivation is impossible.
  ```
- Any `--to <node>` not reachable from the supplied `--from` via the edge table or any composite traversal: REFUSED with the appropriate refusal-class message from §3.

## §5 Grammar

```
mnemonic convert \
  --from <subkey>=<value> [--from <subkey>=<value> ...] \
  --to <subkey>[,<subkey> ...] \
  [--network <mainnet|testnet|signet|regtest>] \
  [--template <bip44|bip49|bip84|bip86|wsh-sortedmulti|...>] \
  [--path <bip32-path>] \
  [--language <english|...>] \
  [--passphrase <s>] \
  [--account <u32>] \
  [--fingerprint <8-hex>] \
  [--xpub-prefix <xpub|ypub|Ypub|zpub|Zpub>]   # v0.6.1 — see §11.a
  [--json]
```

`--from` is repeatable to assemble compound inputs (e.g., `--from xpub=... --from path=...` to bind metadata for mk1 round-trip — though `xpub → mk1` is refused in v0.6.0 per §3.c).

**v0.6 explicit constraint (single-from-value):** at most ONE primary value-bearing `--from` (any of `phrase`, `entropy`, `xpub`, `xprv`, `wif`, `ms1`, `mk1`). Additional `--from` flags supply only side-inputs (`fingerprint`, `path`). Multi-value-bearing input (e.g., two phrases) is reserved for a future grammar extension via `--slot @N` indexing.

### §5.a stdin convention

`--from <node>=-` reads `<node>`'s value from stdin, consistent with the existing `parse::read_phrase_input` convention used by `bundle` / `verify-bundle`. Applies to any node whose serialized form is a single line of UTF-8 text (`phrase`, `entropy` hex, `xpub`, `xprv`, `wif`, `ms1`, `mk1`). For `mk1` (multi-string codec output), stdin reads ALL whitespace-separated tokens on the input stream.

## §6 JSON envelope

Independent schema family from `BundleJson` (which is currently at `schema_version: "4"`). ConvertJson schema versions are their own sequence.

```json
{
  "schema_version": "1",
  "from_node": "phrase",
  "to": [
    {"node": "entropy", "value": "0000000000000000000000000000000000000000000000000000000000000000"},
    {"node": "xpub", "value": "xpub6CatWdiZ..."}
  ]
}
```

### §6.a `from_value` privacy policy (per architect r1 I-2)

`from_value` is OMITTED from the JSON envelope when `from_node` is secret-bearing (`phrase`, `entropy`, `xprv`, `wif`, `ms1`). Echoing the secret input back into a captured JSON output would propagate the secret into shell history, log captures, and any downstream tooling. For public `from_node` values (`xpub`, `mk1`, `fingerprint`), `from_value` IS included to support round-trip verification.

Public-node example:
```json
{
  "schema_version": "1",
  "from_node": "xpub",
  "from_value": "xpub6CatWdiZ...",
  "to": [
    {"node": "fingerprint", "value": "5436d724"}
  ]
}
```

### §6.b `to` array order

The array preserves the user's `--to` argument order (left to right). Compound `--to entropy,xpub,fingerprint` emits three entries in that exact order.

## §7 Side-channel hygiene

Secret-bearing outputs (`phrase`, `entropy`, `xprv`, `wif`, `ms1`) printed to stdout get a one-line stderr warning:

```
warning: secret material on stdout — consider redirecting (e.g., '> file.txt' or '| age -e ...')
```

This is a new convention in `convert`; the existing `bundle` subcommand emits secret-bearing ms1 strings to stdout WITHOUT this warning. The inconsistency is intentional for v0.6.0 (deliberate scope) and tracked at FOLLOWUP `secret-on-stdout-warning-bundle-retrofit` for the next bundle release. The `convert` subcommand is the natural place to introduce this convention because users invoke `convert` for ad-hoc one-shot operations where stdout-redirect-discipline is most likely to be overlooked.

## §8 `--passphrase` / `--language` scope

Per-edge meaningfulness:

| Side-input | Meaningful when |
|------------|-----------------|
| `--passphrase` | edge traverses PBKDF2: `phrase → xprv/xpub/fingerprint`, `entropy → xprv/xpub/fingerprint` |
| `--language` | `phrase` is `--from` or `--to` |
| `--network` | edge derives a BIP-32 xpub/xprv (network is encoded in the version bytes), OR emits `wif` (the WIF version byte is network-dependent), OR `--xpub-prefix` is non-default per §11.a (selects the SLIP-0132 mainnet/testnet swap target) |
| `--template` + `--account` | edge derives at a template path (substitutes for explicit `--path`) |
| `--path` | edge derives at a custom BIP-32 path (mutually exclusive with `--template`) |
| `--fingerprint` | side-input for compound `--from` invocations (e.g., assembling KeyCard inputs) |

Side-inputs that are not meaningful for the chosen edge are IGNORED (not refused). Refusing on irrelevant flags adds friction without preventing user error; ignoring matches the existing toolkit pattern (e.g., `bundle` ignores `--passphrase` for watch-only invocations).

**`--passphrase` warning policy (per architect r1 I-5):** when `--passphrase` is supplied but the chosen edge does NOT traverse PBKDF2 (e.g., `--from xpub --to fingerprint --passphrase ...`), the toolkit emits a one-line stderr warning:
```
warning: --passphrase ignored on this edge (not a PBKDF2-bearing conversion)
```
This is a higher-stakes side-input than the others (a user who believes a passphrase was applied may proceed with wrong assumptions about wallet recovery). All other ignored side-inputs are silent.

**`phrase`/`entropy` → `wif` PBKDF2 invariant (v0.6.1):** the v0.6.1 SPEC-A edge addition extends the PBKDF2-bearing target set. `convert.rs::run`'s `edge_uses_pbkdf2` predicate MUST include `Wif` in the matched set so that `--from phrase --to wif --passphrase x` does NOT spuriously emit the ignored-passphrase warning — PBKDF2 IS traversed (phrase → seed → master → derive at path). Normative invariant: `--passphrase` is meaningful for the v0.6.1-added `phrase/entropy → wif` edge.

## §9 Implementation hooks

Convert subcommand at `crates/mnemonic-toolkit/src/cmd/convert.rs`. Top-level dispatch:

1. Parse `--from` flags into a typed `FromInput { primary: PrimaryNode, side: SideInputs }` struct.
2. Parse `--to` into `Vec<TargetNode>` preserving argument order.
3. Validate the (from, to) pair against the edge-table adjacency. Refusal taxonomy emits §3 stderr verbatim.
4. Dispatch per edge:
   - BIP-39-rooted (`phrase` / `entropy` source): call `crate::derive_slot::derive_bip32_from_entropy` once; reuse the resulting `DerivedAccount` for any cascading `--to` requests.
   - Pure encode (`entropy → ms1`): call `ms_codec::encode(Tag::ENTR, &Payload::Entr(bytes))`.
   - Pure decode (`ms1 → entropy`, `mk1 → xpub+...`): call codec library `decode`, pattern-match outputs.
   - Pure BIP-32 (`xprv → xpub`, etc.): instantiate `Secp256k1::new()`, derive directly.
   - WIF: FIRST assert `--path` is NOT supplied; if present, refuse with §4 byte-exact stderr and exit 2 (chain code is destroyed in WIF serialization; derivation is impossible). On the happy path (no `--path`): parse via `bitcoin::PrivateKey::from_wif`; emit depth-0 sentinel xpub (matches `bundle.rs::resolve_slots` Wif branch around line 420 — depth=0, parent_fingerprint=default, child_number=Normal{0}, public_key=privkey.inner, chain_code=zero[32]).
5. Emit per `--json` flag: text-mode prints one line per `--to` node prefixed with the node name; `--json` mode emits the §6 envelope. Apply §7 stderr warning if any output is secret-bearing.

### Conversion-graph representation

Internal graph as a typed enum + adjacency `HashMap<(PrimaryNode, TargetNode), EdgeKind>`. `EdgeKind::Direct(fn)` for direct edges; `EdgeKind::Refusal(RefusalClass)` for refused edges; `EdgeKind::Composite(Vec<TargetNode>)` for traversal-based edges. Unit test asserts every `(from, to)` cell is either Direct, Composite, or Refusal — no holes (architect r1 I-2 partition).

## §10 v0.6 out-of-scope items (with v0.7 address-derivation extension)

> v0.7 amendment: §10.a now ships address derivation in scope; see §10.a for the full spec.

- `seed`, `raw_privkey`, `xprv`-via-ms1, `seed`-via-ms1 nodes (deferred pending ms-codec v0.2 — §1).
- Multi-value-bearing `--from` flags (single-from-value v0.6 constraint — §5). Reserved for future `--slot @N` indexing.
- Cross-format pivots (`ms1 ↔ mk1`, etc.) — `mnemonic bundle` is the composition operator.
- **Address derivation (xpub + path → bitcoin address) — SHIPPED in v0.7** per §2 edge `(Xpub, Address)` and the full grammar below. Read-only display only — does NOT extend to PSBT / signing flows (those remain out-of-scope per `bip174-psbt-signing` v1+).

### §10.a Address derivation grammar (v0.7)

The `(Xpub, Address)` edge derives a child key at a user-supplied BIP-32 path and renders it as a Bitcoin address per the requested script type.

**Inputs:**

- `--path <BIP32-path>` — **MANDATORY**. The toolkit does NOT auto-default a path from `--template`/`--account` for address derivation. Refusal stderr if absent: `refusal_address_no_path` (see §3.d). The `--chain receive|change` + `--address-index N` shorthand is deferred to v0.8 FOLLOWUPS as UX polish (architect R1-I6).
- `--script-type <p2wpkh|p2sh-p2wpkh|p2tr>` — **MANDATORY** unless inferable from `--template` (`wpkh` → `p2wpkh`; `sh-wpkh` → `p2sh-p2wpkh`; `tr` → `p2tr`). Explicit `--script-type` overrides any template-implied value.
- `--network <mainnet|testnet|signet|regtest>` — optional. When absent, network is inferred from the xpub version-byte prefix per BIP-32 (mainnet `0x0488B21E` / testnet `0x043587CF`) or from the SLIP-0132 prefix per §11 normalization.

**Algorithm:**

1. Normalize input xpub via `slip0132::normalize_xpub_prefix` (§11).
2. Parse the normalized xpub via `bitcoin::bip32::Xpub::from_str`.
3. Derive at `--path` via `Xpub::derive_pub(&secp, &path)` → child `Xpub`.
4. Extract compressed pubkey from child xpub.
5. Dispatch:
   - `p2wpkh` → `Address::p2wpkh(&compressed_pubkey, network)` (BIP-84 §"Test vectors": <https://github.com/bitcoin/bips/blob/master/bip-0084.mediawiki>).
   - `p2sh-p2wpkh` → `Address::p2shwpkh(&compressed_pubkey, network)` (BIP-49 §"Test vectors": <https://github.com/bitcoin/bips/blob/master/bip-0049.mediawiki>).
   - `p2tr` → `Address::p2tr(&secp, x_only_pubkey, None /* no script-path */, network)` (BIP-86 §"Test vectors": <https://github.com/bitcoin/bips/blob/master/bip-0086.mediawiki>).
6. Emit address string via `Address::to_string()`.

**One-way:** the reverse edge `(Address, *)` is REFUSED for every target node — addresses are hashes (`hash160` for P2WPKH/P2SH-P2WPKH; `sha256(taproot_output)` semantics for P2TR) and carry no preimage to the pubkey, let alone the xpub or upstream BIP-32 chain.

**Composite:** the edge `phrase → address` and `entropy → address` traverse `phrase/entropy → xpub → address` via the existing BIP-32 derivation pipeline. `--passphrase` is meaningful (PBKDF2-bearing per §8). `--path` is still mandatory at the address step — the toolkit does NOT collapse a single `--path` into both BIP-32 derivation and address-step derivation; the user supplies a path that derives directly to the leaf privkey/pubkey.

**Reference vectors pinned in tests:** at minimum one P2WPKH (BIP-84), one P2SH-P2WPKH (BIP-49), one P2TR (BIP-86) reference vector, byte-exact stderr-pinned in Phase 4 RED tests.

## §11 SLIP-0132 prefix-tolerant input (v0.6.1)

The toolkit's xpub-bearing inputs (`convert --from xpub=...`, `bundle --slot @0.xpub=...`, `verify-bundle --slot @0.xpub=...`) accept SLIP-0132 prefix variants in addition to the BIP-32 neutral `xpub`/`tpub`. On input, a non-neutral prefix is normalized to the neutral form via base58check-decode → version-byte swap → re-encode. The 78-byte raw buffer (4-byte version prefix + 74-byte payload of depth/parent_fingerprint/child_number/chain_code/pubkey) returned by `bitcoin::base58::decode_check` has the version-prefix swapped at offset `[0..4]`; the trailing 74-byte payload is byte-identical across SLIP-0132 variants of the same key. Normalization is encoding-only — no derivation, no key-material change. Implementation invariant: `raw.len() == 78`.

**Recognized prefixes (mainnet → swap to `xpub` `0x04 88 B2 1E`):**

- `ypub` (BIP-49 single-sig, `0x04 9D 7C B2`)
- `Ypub` (BIP-49 multisig P2SH-P2WSH, `0x02 95 B4 3F`)
- `zpub` (BIP-84 single-sig, `0x04 B2 47 46`)
- `Zpub` (BIP-84 multisig P2WSH, `0x02 AA 7E D3`)

**Recognized prefixes (testnet → swap to `tpub` `0x04 35 87 CF`):**

- `upub` (BIP-49 single-sig, `0x04 4A 52 62`)
- `Upub` (BIP-49 multisig, `0x02 42 89 EF`)
- `vpub` (BIP-84 single-sig, `0x04 5F 1C F6`)
- `Vpub` (BIP-84 multisig, `0x02 57 54 83`)

**Unknown prefix:** stderr `error: unknown extended-key version prefix: <hex>` — exit 1 (BadInput class). Not exit 2 (refusal class) — the input is malformed from the toolkit's perspective, not a policy-refused operation.

**Network cross-check:** the normalizer does NOT validate `--network` against the SLIP-0132 prefix's implied network. Users are responsible for network-consistent inputs; mismatch (e.g., `--network mainnet` with a `vpub` input that normalizes to `tpub`) produces a well-formed but network-inconsistent bundle, matching existing toolkit behavior for raw `tpub` supplied with `--network mainnet`. Not all xpub-flow paths route through `derive_slot::derive_bip32_from_entropy`'s downstream check; the policy is "user responsibility," not "caught downstream."

**Implementation hooks:** the normalizer is implemented in `src/slip0132.rs::normalize_xpub_prefix(s) -> Result<String, ToolkitError>` and called at every PRODUCTION `Xpub::from_str` site that consumes a user-supplied xpub:

- `convert.rs::compute_outputs` (Xpub-source branch, line ~515)
- `bundle.rs::resolve_slots` (template-mode Xpub branch, line ~327)
- `bundle.rs::bundle_run_unified_descriptor` (descriptor-mode Xpub branch, line ~853)
- `verify_bundle.rs`: NO `Xpub::from_str` call sites post-v0.5.1; coverage is transitive via `bundle::resolve_slots`.

**No normalizer call needed at:**

- `parse_descriptor.rs:946` (`bind_watch_only_singlesig`) — reachable only from `bind_descriptor_keys::830`, which is no longer called from any production path in `cmd/bundle.rs` after v0.5's `--xpub`/`--cosigner` flag deletion. Reached only by tests in `parse_descriptor.rs` (lines 1496+).
- `parse.rs:129` (`parse_cosigner_spec`) and `parse.rs:196` (`parse_cosigners_file`) — also dead post-v0.5: their CLI flag callers were removed in v0.5.1.
- `parse_descriptor.rs:1632` and `:1660` and the test fixtures at `:1702`/`:1705`/`:1708` — test bodies only; supply hand-crafted xpub strings, never user input.

**Output side:** see §11.a for `--to`-side SLIP-0132 emission grammar.

**Stderr info-line on actual normalization (v0.6.2).** When `normalize_xpub_prefix` actually fires (input was a SLIP-0132 variant, not already neutral), `mnemonic convert` and `mnemonic bundle` emit a stderr informational line of the form:

<!-- BEGIN: slip0132-info-line -->
```
info: normalized <variant> input to neutral <xpub|tpub> (encoding-only; no key change). Re-emit with --xpub-prefix <variant> if you need the SLIP-0132 form.
```
<!-- END: slip0132-info-line -->

`<variant>` is the input prefix preserving case (`ypub | Ypub | zpub | Zpub | upub | Upub | vpub | Vpub`); `<xpub|tpub>` is the BIP-32 neutral form for the variant's network class (mainnet → `xpub`; testnet → `tpub`). Suppressed when input is already neutral. Emitted regardless of `--json` and `--no-engraving-card`. See `SPEC_mnemonic_toolkit_v0_5.md` §5.5.a for the cross-cutting stderr ordering invariant (info-line precedes the engraving-card block; secret-on-stdout warning, when it fires, is last).

The HTML-comment markers `<!-- BEGIN: slip0132-info-line -->` / `<!-- END: slip0132-info-line -->` delimit the canonical info-line block for deterministic extraction by the Phase 7 carry-over test (`slip0132-info-line-spec-text-not-byte-pinned`): a unit test in `crates/mnemonic-toolkit/src/slip0132.rs` reads this SPEC via `include_str!`, slices between the markers, and asserts byte-equality with `render_slip0132_info_line(variant)` output. Closes the SPEC↔production drift hazard surfaced in v0.6.2 final review.

**v0.7 amendment — `mnemonic verify-bundle` SLIP-0132 input-normalization asymmetry (locked Option B per architect R1-I8).** `mnemonic verify-bundle` is structurally a checker that emits `VERIFIED` / `MISMATCH` status; SLIP-0132 input-normalization info notes are deliberately suppressed on this codepath to avoid breaking script callers that parse stderr line-by-line for status. Surfacing info-lines in `verify-bundle` would be an explicit UX policy change, not a bugfix. The 4 callsite-comments at `verify_bundle.rs:~209/~261/~337/~407` (each `// verify-bundle does not surface SLIP-0132 input-normalization signals.`) gain a one-line cross-pointer to this SPEC clause. Zero new emission code in v0.7.

Rationale (architect R1 audit, transcribed): Option A (surface info-line in verify-bundle) would thread `slip0132_signals` through `verify-bundle`'s render loop (~80 LOC); but `verify-bundle` output is consumed by scripts and humans checking for `VERIFIED`/`MISMATCH` signals; injecting info-lines into that path risks breaking callers that parse stderr line-by-line. Option B (document) has zero blast radius and is consistent with `verify-bundle`'s read-only checker semantics.

## §11.a `--xpub-prefix` modifier (v0.6.1)

When the convert invocation has `xpub` in `--to` (directly or via composite traversal — e.g., `phrase → xpub`), the optional `--xpub-prefix <variant>` flag controls the version-byte prefix of the emitted xpub:

| `--xpub-prefix` value | Mainnet swap | Testnet swap | Intent |
|---|---|---|---|
| `xpub` (default) | `0x04 88 B2 1E` | `0x04 35 87 CF` | BIP-32 neutral; default behavior |
| `ypub` | `0x04 9D 7C B2` | `0x04 4A 52 62` | BIP-49 single-sig (advisory) |
| `Ypub` | `0x02 95 B4 3F` | `0x02 42 89 EF` | BIP-49 multisig (advisory) |
| `zpub` | `0x04 B2 47 46` | `0x04 5F 1C F6` | BIP-84 single-sig (advisory) |
| `Zpub` | `0x02 AA 7E D3` | `0x02 57 54 83` | BIP-84 multisig (advisory) |

**5 flag values; network is selected by `--network`.** The flag value names the SLIP-0132 *semantic class* (BIP-49-single, BIP-49-multisig, BIP-84-single, BIP-84-multisig, neutral), not the specific prefix string. `--xpub-prefix ypub` emits `ypub` on mainnet and `upub` on testnet — selected via `--network`. There is no `--xpub-prefix upub` flag value (testnet variants are not exposed as flag values). The lowercase value names match the SLIP-0132 prefix character; uppercase `Y`/`Z` correspond to the multisig variants per the SLIP-0132 spec. The flag value `xpub` IS the default (omitting the flag emits BIP-32-neutral).

**`--network` required when `--xpub-prefix` is non-default.** When `--xpub-prefix` is anything other than `xpub`, `--network` MUST be supplied explicitly. Refusal stderr when `--network` is omitted (byte-exact):

```
error: --xpub-prefix <variant> requires explicit --network (cannot infer mainnet vs. testnet swap from defaults).
```

Exit code: 2 (refusal class via `ToolkitError::ConvertRefusal`). Eliminates an entire class of "testnet user omits `--network` and gets mainnet zpub" bugs. Default `--xpub-prefix xpub` continues to default `--network mainnet` per the existing convert behavior.

**No effect on non-xpub targets:** `--xpub-prefix` is silently ignored when the invocation has no xpub-typed target, consistent with §8's side-input ignore policy. Example: `convert --from phrase=... --to entropy --xpub-prefix zpub` emits entropy normally; the flag has no effect.

**`--passphrase` semantics on phrase-source edges through `--xpub-prefix`:** when the source is `phrase` or `entropy` and the target is `xpub` (with any `--xpub-prefix` value), the edge traverses PBKDF2 per §8. `--passphrase` is meaningful — non-empty passphrases produce distinct keys, and the resulting xpub (regardless of prefix swap) reflects the supplied passphrase.

**Round-trip property:** `convert --from xpub=<x> --to xpub --xpub-prefix zpub --network mainnet | mnemonic convert --from xpub=- --to xpub` emits `<x>` byte-for-byte (modulo trailing whitespace). The output zpub re-decodes to the same neutral xpub via §11; symmetry is exact for all SLIP-0132 prefix variants (mainnet + testnet).

## §12 BIP-38 encrypted WIF (v0.7)

The `(Wif, Bip38)` and `(Bip38, Wif)` edges implement the non-EC-multiplied form of BIP-38 (<https://github.com/bitcoin/bips/blob/master/bip-0038.mediawiki>) via the `bip38 v1.1.1` crate (<https://crates.io/crates/bip38>; Apache-2.0; verified GREEN per `design/agent-reports/spike-libaudit-bip38-electrum-2026-05-06.md` and Phase 1 source-level security review).

**Scrypt parameters (BIP-38 §"Encryption when EC multiply flag is not used"):** `n=16384`, `r=8`, `p=8`, derived-key length 64 bytes. Hardcoded inside the `bip38` crate; verified in Phase 1 security review.

**Passphrase normalization (BIP-38 §"Encryption when EC multiply flag is not used"):** Unicode NFC applied at both encrypt and decrypt entry points by the `bip38` crate.

**Inputs:**

- `--passphrase <s>` — **MANDATORY** for both `(Wif, Bip38)` and `(Bip38, Wif)`. Empty passphrase is permitted (BIP-38 spec does not forbid it). Stdin convention: `--passphrase=-` reads from stdin (consistent with §5.a).

**Algorithm (encrypt, `Wif → Bip38`):**

1. Parse input WIF via `bitcoin::PrivateKey::from_wif`.
2. Apply NFC normalization to passphrase.
3. Call `<&str as bip38::EncryptWif>::encrypt(wif_str, &passphrase) -> Result<String, bip38::Error>`.
4. Emit BIP-38 encrypted string (~58 chars; base58check; starts with `6P`).

**Algorithm (decrypt, `Bip38 → Wif`):**

1. Apply NFC normalization to passphrase.
2. Call `<&str as bip38::Decrypt>::decrypt(bip38_str, &passphrase) -> Result<(SecretKey, bool /* compressed */), bip38::Error>`.
3. Reconstruct `bitcoin::PrivateKey { compressed, network: <inferred from BIP-38 address-hash check>, inner: secret_key }`.
4. Emit WIF via `to_wif()`.

**Refusal taxonomy (v0.7):**

- `Bip38 → Bip38`: identity-pivot refusal — `refusal_bip38_identity` (see §3.d).
- `Bip38 → <non-Wif>`: composite via `Wif` intermediate; the dispatcher walks `Bip38 → Wif → <target>`. No special refusal (engages standard composite-edge resolution).
- BIP-38 EC-multiplied form (intermediate codes / passphrase codes — BIP-38 §"Encryption when EC multiply mode is used"): the `bip38` crate's `Decrypt` impl rejects EC-multiplied codes with a typed `bip38::Error` variant; dispatch maps to a clean error rather than silent mis-processing. v0.7 does NOT support EC-multiplied form on either encrypt or decrypt direction; FOLLOWUPS-tier deferral if user demand surfaces.
- Passphrase mismatch (decrypt direction): `bip38::Error::Pass` → `refusal_bip38_passphrase_mismatch` (see §3.d).

**Secret-bearing classification (architect R1-L12):** `Bip38` is added to the `NodeType::is_secret_bearing` arm — although the byte stream is encrypted, possession-with-passphrase yields the underlying privkey, so the §7 secret-on-stdout warning fires when `--to bip38` output is rendered to a terminal.

**Reference vectors:** BIP-38 §"Test vectors" — three non-EC-multiplied test vectors (no compression, with compression, and a test with passphrase containing non-ASCII). All three are pinned in Phase 1 RED tests.

### §12.b Composite `(Phrase|Entropy, Bip38)` passphrase semantics

For composite paths flowing `phrase → wif → bip38` (or `entropy → wif → bip38`), `--passphrase` serves a DUAL purpose:

1. **BIP-39 mnemonic extension** (input to PBKDF2 during seed derivation in `phrase → wif` step).
2. **BIP-38 encryption passphrase** (input to Scrypt during `wif → bip38` step).

This means a single `--passphrase` value composes BOTH the seed extension and the BIP-38 encryption. To use distinct values for each layer, invoke `convert` twice: first `phrase → wif --passphrase X` (or `--passphrase ""` for empty extension), then `wif → bip38 --passphrase Y`.

The `--bip38-passphrase` distinct-flag option is tracked as a v0.8 UX FOLLOWUP (`bip38-distinct-passphrase-flag`).

## §13 Casascius mini-private-key (v0.7)

The `(MiniKey, Wif)` edge decodes Casascius mini-private-key strings (the engraved format on physical Casascius bitcoin coins, ~2011-2013).

**Source:** Casascius mini private key spec (<https://en.bitcoin.it/wiki/Mini_private_key_format>).

**Format:** 22, 26, or 30 base58 characters; starts with `S`. Carries a single SHA256 self-checksum (no length-prefix or version byte).

**Self-checksum rule:** `SHA256(mini_key + "?")[0] == 0x00` — the first byte of the SHA256 hash of the mini-key string with the literal `?` byte appended must be zero. This is the typo-detection mechanism; valid mini-keys are mined by brute-force search until a candidate satisfies the checksum.

**Decode algorithm:**

1. Validate length is 22, 26, or 30 characters.
2. Validate self-checksum: `SHA256(mini_key.as_bytes() ++ b"?")[0] == 0x00` — refuse with `error: invalid Casascius mini-key checksum (SHA256(key + "?")[0] != 0x00)` on mismatch.
3. Compute raw privkey scalar: `SHA256(mini_key.as_bytes())` (32-byte output).
4. Wrap as `bitcoin::PrivateKey { compressed: false, network, inner: secp256k1::SecretKey::from_slice(&raw)? }`. **Compressed flag is `false`** — Casascius mini-keys predate BIP-32 compressed-pubkey convention; the canonical decode is uncompressed.
5. Emit WIF via `to_wif()`.

**Decode-only contract:** there is NO `(*, MiniKey)` edge. Generating a mini-key requires brute-force search until the self-checksum byte is zero (expected ~256 candidate strings per valid mini-key). This is a deliberate one-way refusal — `refusal_minikey_one_way` (see §3.d).

**Reference vectors:** at minimum one of each length class (22-char, 26-char, 30-char) pinned in Phase 2 RED tests, sourced from public references (cite source URL inline in test comments).

## §14 Electrum native seed format (v0.7)

The `(ElectrumPhrase, Entropy)` and `(Entropy, ElectrumPhrase)` edges implement Electrum's native seed-phrase format (NOT BIP-39 — Electrum uses its own wordlist and validation rule).

**Source:** Electrum's `electrum/mnemonic.py` (<TODO@phase-3-corpus-spike-resolves-this>; Phase 3 of v0.7 execution will pin the SHA + retrieval date for the wordlist + seed-version constants). Per architect R1-I5, Phase 3 executes a corpus spike against Electrum's `tests/test_wallet.py` before encode-direction lock.

**Wordlist provenance:** the embedded English wordlist (constant array, ~2KB, ~2048 entries) is sourced from Electrum's `electrum/wordlist/english.txt` (referenced from `electrum/mnemonic.py::ElectrumMnemonicEnglish`). Cite SHA + retrieval date inline in `crates/mnemonic-toolkit/src/electrum.rs` at module top.

**Seed version dispatch:**

```rust
enum SeedVersion {
    Standard,    // HMAC-SHA512("Seed version" || phrase) hex starts with "01"
    Segwit,      // HMAC-SHA512("Seed version" || phrase) hex starts with "100"
    Standard2FA, // HMAC-SHA512("Seed version" || phrase) hex starts with "101"
    Segwit2FA,   // HMAC-SHA512("Seed version" || phrase) hex starts with "102"
}
```

Validation function:

```rust
fn validate_seed_version(phrase: &str) -> Result<SeedVersion, ElectrumError>
```

Computes `HMAC-SHA512(key=b"Seed version", msg=phrase.as_bytes())` and dispatches on the lowercase hex prefix.

**Decode (`ElectrumPhrase → Entropy`):**

1. Call `validate_seed_version(phrase)`.
2. If `Standard2FA` or `Segwit2FA`, refuse with `refusal_electrum_2fa_unsupported` (§3.d).
3. Decode wordlist mapping: each whitespace-separated word → 11-bit index in the embedded wordlist; concatenate to form the entropy bigint; serialize to bytes.
4. Emit hex.

**Encode (`Entropy → ElectrumPhrase`):**

1. Take optional `--electrum-version <standard|segwit>` (default `standard`).
2. Encode entropy as a candidate phrase via wordlist forward mapping.
3. Verify `validate_seed_version(candidate) == requested_version`. If not, the entropy does NOT cleanly map to the requested seed-version (Electrum-style entropy must be "mined" by incrementing through candidates until the HMAC prefix matches); the toolkit's encode behavior in this case is left to Phase 3 spike-driven decision (lock once corpus reproduces).
4. Emit phrase string.

**Refusal class (2FA seeds):** seed-versions `101` (Standard2FA) and `102` (Segwit2FA) are explicitly refused on the decode direction with `refusal_electrum_2fa_unsupported` (§3.d). 2FA seeds require a second factor (Electrum's challenge-response with TrustedCoin) not present in the phrase alone; silently decoding them produces garbage entropy. 2FA versions are also NOT exposed as encode targets — `--electrum-version` enum is restricted to `standard|segwit`.

**Sibling-pivot refusals:**

- `Phrase → ElectrumPhrase` and `ElectrumPhrase → Phrase` are REFUSED via `refusal_electrum_phrase_pivot` (§3.d) — BIP-39 and Electrum native are different artifact classes.

**Reference vectors:** at minimum one Standard + one Segwit reference phrase from Electrum's `tests/test_wallet.py`, pinned in Phase 3 RED tests with cite-comment to source SHA. 2FA refusal stderr byte-pinned for both `101` and `102` versions.
