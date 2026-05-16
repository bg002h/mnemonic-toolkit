# Dropdown enumeration reference {#dropdown-reference}

Curated cross-reference for every `Dropdown` / `NodeValueComposite`
/ `TaggedOrIndexed` enumeration in the GUI's schema. Source of
truth is `mnemonic-gui/src/schema/{mnemonic,md,ms,mk}.rs`. Use
this page when you need to know "what valid values does flag X
accept?" without scrolling to the per-flag chapter.

Convention: each row links to the **canonical** defining anchor
(`mnemonic bundle` is the canonical defining anchor when a
value-set is shared across several subcommands).

## `mnemonic` tab — Dropdowns

### `NETWORKS` (4 values) {#dropdown-networks}

Used by [`mnemonic bundle`](#mnemonic-bundle-network) /
[`verify-bundle`](#mnemonic-verify-bundle-network) /
[`convert`](#mnemonic-convert-network) /
[`export-wallet`](#mnemonic-export-wallet-network) /
[`derive-child`](#mnemonic-derive-child-network) /
[`md encode`](#md-encode-network) /
[`md verify`](#md-verify-network) /
[`md address`](#md-address-network) `--network`.

- [`mainnet`](#mnemonic-bundle-network-mainnet)
- [`testnet`](#mnemonic-bundle-network-testnet)
- [`signet`](#mnemonic-bundle-network-signet)
- [`regtest`](#mnemonic-bundle-network-regtest)

### `TEMPLATES` (10 values) {#dropdown-templates}

Used by [`mnemonic bundle`](#mnemonic-bundle-template) /
[`verify-bundle`](#mnemonic-verify-bundle-template) /
[`convert`](#mnemonic-convert-template) /
[`export-wallet`](#mnemonic-export-wallet-template) `--template`.

- [`bip44`](#mnemonic-bundle-template-bip44)
- [`bip49`](#mnemonic-bundle-template-bip49)
- [`bip84`](#mnemonic-bundle-template-bip84)
- [`bip86`](#mnemonic-bundle-template-bip86)
- [`wsh-multi`](#mnemonic-bundle-template-wsh-multi)
- [`wsh-sortedmulti`](#mnemonic-bundle-template-wsh-sortedmulti)
- [`sh-wsh-multi`](#mnemonic-bundle-template-sh-wsh-multi)
- [`sh-wsh-sortedmulti`](#mnemonic-bundle-template-sh-wsh-sortedmulti)
- [`tr-multi-a`](#mnemonic-bundle-template-tr-multi-a)
- [`tr-sortedmulti-a`](#mnemonic-bundle-template-tr-sortedmulti-a)

### `LANGUAGES` (`mnemonic` tab, 10 values, fused tokens) {#dropdown-languages-mnemonic}

Used by `mnemonic bundle` / `verify-bundle` / `convert` /
`export-wallet` / `derive-child` / `final-word` /
`seed-xor-split` / `seed-xor-combine` / `slip39-split` /
`slip39-combine` `--language`.

The `mnemonic`-tab tokens FUSE the qualifier:
`simplifiedchinese` / `traditionalchinese`. The `ms`-tab tokens
HYPHENATE: `chinese-simplified` / `chinese-traditional`. See
[`mnemonic bundle --language`](#mnemonic-bundle-language) for the
cross-tab divergence note.

- [`english`](#mnemonic-bundle-language-english)
- [`simplifiedchinese`](#mnemonic-bundle-language-simplifiedchinese)
- [`traditionalchinese`](#mnemonic-bundle-language-traditionalchinese)
- [`czech`](#mnemonic-bundle-language-czech)
- [`french`](#mnemonic-bundle-language-french)
- [`italian`](#mnemonic-bundle-language-italian)
- [`japanese`](#mnemonic-bundle-language-japanese)
- [`korean`](#mnemonic-bundle-language-korean)
- [`portuguese`](#mnemonic-bundle-language-portuguese)
- [`spanish`](#mnemonic-bundle-language-spanish)

### `MULTISIG_PATH_FAMILIES` (2 values) {#dropdown-multisig-path-families}

Used by [`mnemonic bundle`](#mnemonic-bundle-multisig-path-family)
/ [`verify-bundle`](#mnemonic-verify-bundle-multisig-path-family)
/ [`export-wallet`](#mnemonic-export-wallet-multisig-path-family)
`--multisig-path-family`. Two BIP-numbered multisig path
families.

- [`bip48`](#mnemonic-bundle-multisig-path-family-bip48)
- [`bip87`](#mnemonic-bundle-multisig-path-family-bip87)

### `EXPORT_FORMATS` (8 values) {#dropdown-export-formats}

Used by [`mnemonic export-wallet --format`](#mnemonic-export-wallet-format).

- [`bitcoin-core`](#mnemonic-export-wallet-format-bitcoin-core)
- [`bip388`](#mnemonic-export-wallet-format-bip388)
- [`coldcard`](#mnemonic-export-wallet-format-coldcard)
- [`jade`](#mnemonic-export-wallet-format-jade)
- [`sparrow`](#mnemonic-export-wallet-format-sparrow)
- [`specter`](#mnemonic-export-wallet-format-specter)
- [`electrum`](#mnemonic-export-wallet-format-electrum)
- [`green`](#mnemonic-export-wallet-format-green)

### `BIP85_APPLICATIONS` (9 values) {#dropdown-bip85-applications}

Used by [`mnemonic derive-child --application`](#mnemonic-derive-child-application).

- [`bip39`](#mnemonic-derive-child-application-bip39)
- [`hd-seed`](#mnemonic-derive-child-application-hd-seed)
- [`xprv`](#mnemonic-derive-child-application-xprv)
- [`hex`](#mnemonic-derive-child-application-hex)
- [`password-base64`](#mnemonic-derive-child-application-password-base64)
- [`password-base85`](#mnemonic-derive-child-application-password-base85)
- [`dice`](#mnemonic-derive-child-application-dice)
- [`rsa`](#mnemonic-derive-child-application-rsa)
- [`rsa-gpg`](#mnemonic-derive-child-application-rsa-gpg)

### `SCRIPT_TYPES` (3 values) {#dropdown-script-types}

Used by [`mnemonic convert --script-type`](#mnemonic-convert-script-type).

- [`p2wpkh`](#mnemonic-convert-script-type-p2wpkh)
- [`p2sh-p2wpkh`](#mnemonic-convert-script-type-p2sh-p2wpkh)
- [`p2tr`](#mnemonic-convert-script-type-p2tr)

### `ELECTRUM_VERSIONS` (2 values) {#dropdown-electrum-versions}

Used by [`mnemonic convert --electrum-version`](#mnemonic-convert-electrum-version).
The 2FA variants (`standard-2fa`, `segwit-2fa`) are not
supported and produce a specific 2FA-unsupported refusal.

- [`standard`](#mnemonic-convert-electrum-version-standard)
- [`segwit`](#mnemonic-convert-electrum-version-segwit)

### `SLIP39_TO_SHAPES` (2 values) {#dropdown-slip39-to-shapes}

Used by [`mnemonic slip39-combine --to`](#mnemonic-slip39-combine-to).
Selects the output shape for share-recovered material.

- [`entropy`](#mnemonic-slip39-combine-to-entropy)
- [`phrase`](#mnemonic-slip39-combine-to-phrase)

### `NODE_TYPES` (13 values; via `mnemonic convert --to` Dropdown) {#dropdown-node-types}

Used by [`mnemonic convert --to`](#mnemonic-convert-to) as a
plain Dropdown (one of 13 output-node tokens). The same constant
also backs the [`mnemonic convert --from`](#mnemonic-convert-from)
NodeValueComposite (see the next section).

- [`phrase`](#mnemonic-convert-to-phrase)
- [`entropy`](#mnemonic-convert-to-entropy)
- [`xpub`](#mnemonic-convert-to-xpub)
- [`xprv`](#mnemonic-convert-to-xprv)
- [`wif`](#mnemonic-convert-to-wif)
- [`fingerprint`](#mnemonic-convert-to-fingerprint)
- [`path`](#mnemonic-convert-to-path)
- [`ms1`](#mnemonic-convert-to-ms1)
- [`mk1`](#mnemonic-convert-to-mk1)
- [`bip38`](#mnemonic-convert-to-bip38)
- [`minikey`](#mnemonic-convert-to-minikey)
- [`electrum-phrase`](#mnemonic-convert-to-electrum-phrase)
- [`address`](#mnemonic-convert-to-address)

## `md` tab — Dropdowns

### `NETWORKS` (`md` tab, 4 values; same constant set as the `mnemonic` tab)

Used by [`md encode --network`](#md-encode-network),
[`md verify --network`](#md-verify-network), and
[`md address --network`](#md-address-network).

### `SCRIPT_CONTEXTS` (2 values) {#dropdown-script-contexts}

Used by [`md encode --context`](#md-encode-context) and
[`md compile --context`](#md-compile-context).

- [`tap`](#md-encode-context-tap)
- [`segwitv0`](#md-encode-context-segwitv0)

## `ms` tab — Dropdowns

### `LANG_MS` (10 values, hyphenated tokens) {#dropdown-lang-ms}

Used by [`ms encode --language`](#ms-encode-language) /
[`ms decode --language`](#ms-decode-language) /
[`ms verify --language`](#ms-verify-language).

Hyphenated Chinese tokens — divergent from the `mnemonic` tab's
fused tokens. See [§61 cross-tab
divergence](#ms-per-tab-reference).

- [`english`](#ms-encode-language-english)
- [`japanese`](#ms-encode-language-japanese)
- [`korean`](#ms-encode-language-korean)
- [`spanish`](#ms-encode-language-spanish)
- [`chinese-simplified`](#ms-encode-language-chinese-simplified)
- [`chinese-traditional`](#ms-encode-language-chinese-traditional)
- [`french`](#ms-encode-language-french)
- [`italian`](#ms-encode-language-italian)
- [`czech`](#ms-encode-language-czech)
- [`portuguese`](#ms-encode-language-portuguese)

## `mk` tab — Dropdowns

None. The `mk` CLI accepts no Dropdown-typed flags
(per the Phase 1 audit at plan §1.4 table and verified against
`schema/mk.rs`).

## `NodeValueComposite` widgets

The `--from <type>=<value>` / `--share <type>=<value>` family.
The GUI renders a Dropdown (the type-tag) plus a context-sensitive
value widget that swaps shape (Text / SecretLineEdit / Path)
based on the chosen tag. **Six** usages across the `mnemonic`
tab; none on the `md`, `ms`, or `mk` tabs.

- [`mnemonic convert --from`](#mnemonic-convert-from) — 13 tags (`NODE_TYPES` constant; same set as `--to` Dropdown).
- [`mnemonic derive-child --from`](#mnemonic-derive-child-from) — 2 tags: `xprv`, `phrase`.
- [`mnemonic slip39-split --from`](#mnemonic-slip39-split-from) — 2 tags: `phrase`, `entropy` (per `SLIP39_FROM_NODES`).
- [`mnemonic seed-xor-split --from`](#mnemonic-seed-xor-split-from) — 1 tag: `phrase` (per `PHRASE_ONLY` constant).
- [`mnemonic seed-xor-combine --share`](#mnemonic-seed-xor-combine-share) — 1 tag: `phrase` (same `PHRASE_ONLY`).
- [`mnemonic final-word --from`](#mnemonic-final-word-from) — 1 tag: `phrase` (same `PHRASE_ONLY`).

## `TaggedOrIndexed` widgets

One usage in the schema:

- [`mnemonic export-wallet --taproot-internal-key`](#mnemonic-export-wallet-taproot-internal-key)
  — tag `nums` (BIP-341 NUMS point) or indexed form `@N`
  (cosigner N's xpub as the Taproot internal key).
