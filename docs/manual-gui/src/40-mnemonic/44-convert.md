# `mnemonic convert` {#mnemonic-convert}

Single-format conversion across the m-format constellation's
13-node typed graph: `phrase`, `entropy`, `xpub`, `xprv`, `wif`,
`fingerprint`, `path`, `ms1`, `mk1`, `bip38`, `minikey`,
`electrum-phrase`, `address`. One source via `--from <node>=<value>`;
one or more destinations via `--to <node>` (repeating). The CLI
walks the typed-graph edges to produce each requested output.

This is the GUI's most flexible subcommand: it covers every
single-format pivot (`phrase` → `ms1`, `entropy` → `mk1`, etc.)
that does not require the bundle's slot machinery. For
multi-output bundle assembly use [`mnemonic bundle`](#mnemonic-bundle)
instead; for share-splitting use the slip39 / seed-xor families.

:::danger
Several `--from` nodes are **secret-bearing**: `phrase`,
`entropy`, `xprv`, `wif`, `ms1`, `bip38`, `electrum-phrase`. The
canonical worked examples below use the all-`abandon` BIP-39 test
vector. **Never engrave or fund** any wallet derived from this
phrase. The [§14 Defense 2](#secret-handling) cold-node operational
warning applies whenever `--from` carries a secret-class node OR
when `--passphrase` / `--bip38-passphrase` is non-empty.
:::

> **GUI form:** see [GUI Forms › mnemonic › convert](#gui-form-mnemonic-convert).

## Outline {#mnemonic-convert-outline}

- [`--group-size`](#mnemonic-convert-group-size) — mstring display grouping width when emitting an `ms1`/`mk1` card (default 5)
- [`--separator`](#mnemonic-convert-separator) — grouping separator for `--group-size` (default `space`)
- [`--from`](#mnemonic-convert-from) — source node `<name>=<value>` (required; secrecy is value-dependent)
- [`--to`](#mnemonic-convert-to) — destination node (required, repeating)
- [`--network`](#mnemonic-convert-network) — Bitcoin network
- [`--template`](#mnemonic-convert-template) — template (when `--to` involves derivation)
- [`--path`](#mnemonic-convert-path) — explicit BIP-32 derivation path
- [`--language`](#mnemonic-convert-language) — BIP-39 wordlist (default `english`)
- [`--passphrase`](#mnemonic-convert-passphrase) — BIP-39 PBKDF2 passphrase (XOR with `--passphrase-stdin`)
- [`--bip38-passphrase`](#mnemonic-convert-bip38-passphrase) — BIP-38 Scrypt passphrase (XOR with `--bip38-passphrase-stdin`)
- [`--bip38-passphrase-stdin`](#mnemonic-convert-bip38-passphrase-stdin) — read `--bip38-passphrase` from stdin
- [`--passphrase-stdin`](#mnemonic-convert-passphrase-stdin) — read `--passphrase` from stdin
- [`--account`](#mnemonic-convert-account) — BIP-32 account index
- [`--fingerprint`](#mnemonic-convert-fingerprint) — master fingerprint (8 hex chars)
- [`--xpub-prefix`](#mnemonic-convert-xpub-prefix) — SLIP-0132 prefix override for `--to xpub`
- [`--electrum-version`](#mnemonic-convert-electrum-version) — Electrum seed-version selector
- [`--electrum-language`](#mnemonic-convert-electrum-language) — Electrum wordlist (distinct from `--language`)
- [`--script-type`](#mnemonic-convert-script-type) — script-type selector for `(Xpub, Address)` derivation
- [`--json`](#mnemonic-convert-json) — emit JSON-shaped output

## `--group-size` {#mnemonic-convert-group-size}

mstring display grouping. When `convert` emits an `ms1` / `mk1` card
(`--to ms1` / `--to mk1`), inserts a separator every N characters to
ease engraving + read-aloud verification. `0` = unbroken; default `5`.
Display only — `--json` output stays unbroken. The same flag (with
`--separator`) is also accepted on
[`bundle`](#mnemonic-bundle-group-size). Has no effect for non-card
`--to` targets (e.g. `--to address`).

The GUI renders this as a Number widget; no `?` help-icon.

## `--separator` {#mnemonic-convert-separator}

The grouping separator inserted by `--group-size`. Default `space`.
Dropdown; same three values as
[`bundle --separator`](#mnemonic-bundle-separator). The GUI renders
this flag with a `?` help-icon.

### Outline {#mnemonic-convert-separator-outline}

- [`space`](#mnemonic-convert-separator-space)
- [`hyphen`](#mnemonic-convert-separator-hyphen)
- [`comma`](#mnemonic-convert-separator-comma)

### `space` {#mnemonic-convert-separator-space}

See [`bundle --separator space`](#mnemonic-bundle-separator-space).

### `hyphen` {#mnemonic-convert-separator-hyphen}

See [`bundle --separator hyphen`](#mnemonic-bundle-separator-hyphen).

### `comma` {#mnemonic-convert-separator-comma}

See [`bundle --separator comma`](#mnemonic-bundle-separator-comma).

## `--from` {#mnemonic-convert-from}

The source node, in `<node>=<value>` form. Required. The GUI
renders this as a NodeValueComposite widget (a Dropdown selecting
the node + a text field for the value). Schema-`secret: false`
but secrecy is value-dependent: when the chosen node is in
`NodeType::is_secret_bearing()`'s true-arm
(`crates/mnemonic-toolkit/src/cmd/convert.rs:85-95`), the value
field renders as a `SecretLineEdit` and the run-confirm modal
fires.

Suffix `=-` reads the value from stdin. All 13 nodes accept
`=-` syntactically, but the GUI's secret widget treatment fires
only for nodes in `is_argv_secret_bearing()` (the 7 secret-class
nodes plus `minikey` per `cmd/convert.rs:107-109`). Public
nodes accept `=-` as a plain stdin pipe with no secret-widget
masking and no argv-leakage advisory.

### Outline {#mnemonic-convert-from-outline}

- [`phrase`](#mnemonic-convert-from-phrase)
- [`seedqr`](#mnemonic-convert-from-seedqr)
- [`entropy`](#mnemonic-convert-from-entropy)
- [`xpub`](#mnemonic-convert-from-xpub)
- [`xprv`](#mnemonic-convert-from-xprv)
- [`wif`](#mnemonic-convert-from-wif)
- [`fingerprint`](#mnemonic-convert-from-fingerprint)
- [`path`](#mnemonic-convert-from-path)
- [`ms1`](#mnemonic-convert-from-ms1)
- [`mk1`](#mnemonic-convert-from-mk1)
- [`bip38`](#mnemonic-convert-from-bip38)
- [`minikey`](#mnemonic-convert-from-minikey)
- [`electrum-phrase`](#mnemonic-convert-from-electrum-phrase)
- [`address`](#mnemonic-convert-from-address)

### `phrase` {#mnemonic-convert-from-phrase}

A BIP-39 mnemonic phrase. Secret-bearing. Length must be one of
12 / 15 / 18 / 21 / 24 words, each in the `--language` wordlist;
checksum must validate.

### `seedqr` {#mnemonic-convert-from-seedqr}

A SeedQR digit-string (input-only, v0.31.6+). Secret-bearing.
`seedqr=<digits>` decodes a 48 / 60 / 72 / 84 / 96-digit SeedQR string
to a BIP-39 phrase, then projects to any phrase-reachable target.
`seedqr` is a valid `--from` node only — it is NOT a valid `--to`
target (use the `mnemonic seedqr encode` subcommand to emit a
SeedQR digit-string). The same conversion is reachable via
`mnemonic seedqr decode --from seedqr=<digits>`.

### `entropy` {#mnemonic-convert-from-entropy}

Raw BIP-39 entropy as hex bytes. Secret-bearing. Length must be
one of 16 / 20 / 24 / 28 / 32 bytes (32-64 hex chars).

### `xpub` {#mnemonic-convert-from-xpub}

A BIP-32 extended public key. Public material; not secret. Use
when you have an xpub and want to derive an address, fingerprint,
or `mk1` representation.

### `xprv` {#mnemonic-convert-from-xprv}

A BIP-32 extended private key. Secret-bearing. Use when you
have an xprv and want to derive an xpub, wif, address, etc.

### `wif` {#mnemonic-convert-from-wif}

A WIF-encoded single private key. Secret-bearing. Use for
single-key conversions (no BIP-32 derivation tree).

### `fingerprint` {#mnemonic-convert-from-fingerprint}

An 8-hex-character master fingerprint. Public. Used as input on
certain edges (e.g., `mk` reconstruction from xpub plus
fingerprint plus path).

### `path` {#mnemonic-convert-from-path}

A BIP-32 derivation path string (e.g., `m/84'/0'/0'`). Public.

### `ms1` {#mnemonic-convert-from-ms1}

An `ms1` seed-secret card. Secret-bearing. Use to recover a
phrase, entropy, mk1, md1, or address from an engraved ms1.

### `mk1` {#mnemonic-convert-from-mk1}

An `mk1` master-key card. Public. Use to derive xpub +
fingerprint + path metadata, or to re-emit the mk1 in a different
SLIP-0132 prefix.

### `bip38` {#mnemonic-convert-from-bip38}

A BIP-38 encrypted private key. Secret-bearing. Requires
`--bip38-passphrase` to decrypt; the result is a `wif` or
derived form.

### `minikey` {#mnemonic-convert-from-minikey}

A Casascius mini-key (1990s-era short-form private key, distinct
from `bip38`). Secret-bearing. The argv-leakage advisory uses
the wider `is_argv_secret_bearing()` predicate that includes
`minikey` (per `cmd/convert.rs:107-109`); the narrower
`is_secret_bearing()` predicate that gates other secret machinery
does NOT include `minikey` (a deliberate v0.9.0 split documented
at the toolkit's `convert-minikey-stdout-redaction` FOLLOWUP).

### `electrum-phrase` {#mnemonic-convert-from-electrum-phrase}

An Electrum seed phrase (distinct from BIP-39 — uses the Electrum
wordlist + seed-version-prefix scheme). Secret-bearing. Pair with
`--electrum-version` and `--electrum-language` to disambiguate.

### `address` {#mnemonic-convert-from-address}

A Bitcoin address. Public. Useful only as a sanity-check pivot
(e.g., re-derive the address from a fingerprint+path+xpub combo
and assert byte-identical output).

## `--to` {#mnemonic-convert-to}

The destination node. Required, repeating. Same 13-node value-set
as `--from` but rendered as a Dropdown widget (no value field —
the value is computed from the source). For multi-output, add
multiple `--to` rows; the assembled argv emits one `--to <node>`
per row.

### Outline {#mnemonic-convert-to-outline}

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

### `phrase` {#mnemonic-convert-to-phrase}

Emit a BIP-39 phrase. Refusal: `--to phrase` from a
non-entropy-bearing source is a one-way derivation barrier
(e.g., `--from xpub --to phrase`).

### `entropy` {#mnemonic-convert-to-entropy}

Emit raw BIP-39 entropy hex.

### `xpub` {#mnemonic-convert-to-xpub}

Emit a BIP-32 extended public key. Optionally pair with
`--xpub-prefix` (zpub, ypub, etc.) per SLIP-0132.

### `xprv` {#mnemonic-convert-to-xprv}

Emit a BIP-32 extended private key.

### `wif` {#mnemonic-convert-to-wif}

Emit a WIF single private key.

### `fingerprint` {#mnemonic-convert-to-fingerprint}

Emit an 8-hex-character master fingerprint.

### `path` {#mnemonic-convert-to-path}

Emit a BIP-32 derivation path.

### `ms1` {#mnemonic-convert-to-ms1}

Emit an `ms1` seed-secret card.

### `mk1` {#mnemonic-convert-to-mk1}

Emit an `mk1` master-key card. Refusal: `--from xpub --to mk1`
is refused outright with the byte-exact message at
`crates/mnemonic-toolkit/src/cmd/convert.rs::refusal_xpub_to_mk1`:
`--to mk1 requires a policy descriptor binding (mk1 cards bind
xpubs to specific policies via policy_id_stubs). Use 'mnemonic
bundle --slot @0.xpub=... --template ...' to emit a complete
bundle.` Supplying `--fingerprint` and `--path` does NOT unblock
this refusal; the operation requires a full descriptor binding
that only `mnemonic bundle` provides.

### `bip38` {#mnemonic-convert-to-bip38}

Emit a BIP-38 encrypted private key. Requires
`--bip38-passphrase` for the encryption.

### `minikey` {#mnemonic-convert-to-minikey}

Emit a Casascius mini-key. Note: emitting a minikey from a
random secret is non-trivial (the minikey format requires a
specific entropy preprocessing) — the convert subcommand
supports the round-trip but production-grade minikey *generation*
is outside scope.

### `electrum-phrase` {#mnemonic-convert-to-electrum-phrase}

Emit an Electrum seed phrase.

### `address` {#mnemonic-convert-to-address}

Emit a Bitcoin address. Refusal: `--to address` from a phrase or
entropy without `--script-type` is refused (the script type
disambiguates which SegWit / Taproot derivation path applies).

## `--network` {#mnemonic-convert-network}

Same 4 values as [`mnemonic bundle --network`](#mnemonic-bundle-network).
Optional; defaults vary by `--to` node (mainnet for most;
inferred from the source for some pivots).

### Outline {#mnemonic-convert-network-outline}

- [`mainnet`](#mnemonic-convert-network-mainnet)
- [`testnet`](#mnemonic-convert-network-testnet)
- [`signet`](#mnemonic-convert-network-signet)
- [`regtest`](#mnemonic-convert-network-regtest)

### `mainnet` {#mnemonic-convert-network-mainnet}

See [`mnemonic bundle --network mainnet`](#mnemonic-bundle-network-mainnet).

### `testnet` {#mnemonic-convert-network-testnet}

See [`mnemonic bundle --network testnet`](#mnemonic-bundle-network-testnet).

### `signet` {#mnemonic-convert-network-signet}

See [`mnemonic bundle --network signet`](#mnemonic-bundle-network-signet).

### `regtest` {#mnemonic-convert-network-regtest}

See [`mnemonic bundle --network regtest`](#mnemonic-bundle-network-regtest).

## `--template` {#mnemonic-convert-template}

Same 10 values as [`mnemonic bundle --template`](#mnemonic-bundle-template).
Used when `--to` involves derivation (e.g., `--to address` or
`--to xpub`) to disambiguate the BIP-32 path family.

### Outline {#mnemonic-convert-template-outline}

- [`bip44`](#mnemonic-convert-template-bip44)
- [`bip49`](#mnemonic-convert-template-bip49)
- [`bip84`](#mnemonic-convert-template-bip84)
- [`bip86`](#mnemonic-convert-template-bip86)
- [`wsh-multi`](#mnemonic-convert-template-wsh-multi)
- [`wsh-sortedmulti`](#mnemonic-convert-template-wsh-sortedmulti)
- [`sh-wsh-multi`](#mnemonic-convert-template-sh-wsh-multi)
- [`sh-wsh-sortedmulti`](#mnemonic-convert-template-sh-wsh-sortedmulti)
- [`tr-multi-a`](#mnemonic-convert-template-tr-multi-a)
- [`tr-sortedmulti-a`](#mnemonic-convert-template-tr-sortedmulti-a)

### `bip44` {#mnemonic-convert-template-bip44}

See [`mnemonic bundle --template bip44`](#mnemonic-bundle-template-bip44).

### `bip49` {#mnemonic-convert-template-bip49}

See [`mnemonic bundle --template bip49`](#mnemonic-bundle-template-bip49).

### `bip84` {#mnemonic-convert-template-bip84}

See [`mnemonic bundle --template bip84`](#mnemonic-bundle-template-bip84).

### `bip86` {#mnemonic-convert-template-bip86}

See [`mnemonic bundle --template bip86`](#mnemonic-bundle-template-bip86).

### `wsh-multi` {#mnemonic-convert-template-wsh-multi}

See [`mnemonic bundle --template wsh-multi`](#mnemonic-bundle-template-wsh-multi).

### `wsh-sortedmulti` {#mnemonic-convert-template-wsh-sortedmulti}

See [`mnemonic bundle --template wsh-sortedmulti`](#mnemonic-bundle-template-wsh-sortedmulti).

### `sh-wsh-multi` {#mnemonic-convert-template-sh-wsh-multi}

See [`mnemonic bundle --template sh-wsh-multi`](#mnemonic-bundle-template-sh-wsh-multi).

### `sh-wsh-sortedmulti` {#mnemonic-convert-template-sh-wsh-sortedmulti}

See [`mnemonic bundle --template sh-wsh-sortedmulti`](#mnemonic-bundle-template-sh-wsh-sortedmulti).

### `tr-multi-a` {#mnemonic-convert-template-tr-multi-a}

See [`mnemonic bundle --template tr-multi-a`](#mnemonic-bundle-template-tr-multi-a).

### `tr-sortedmulti-a` {#mnemonic-convert-template-tr-sortedmulti-a}

See [`mnemonic bundle --template tr-sortedmulti-a`](#mnemonic-bundle-template-tr-sortedmulti-a).

## `--path` {#mnemonic-convert-path}

An explicit BIP-32 derivation path string (e.g., `m/84'/0'/0'`).
Overrides the path derived from `--template` + `--account` +
`--script-type`. Use when the destination edge needs a non-default
path (e.g., for cosigner-derivation pivots).

## `--language` {#mnemonic-convert-language}

BIP-39 wordlist for parsing `--from phrase=` input AND for
emitting `--to phrase` output. Same 10 values as
[`mnemonic bundle --language`](#mnemonic-bundle-language).

### Outline {#mnemonic-convert-language-outline}

- [`english`](#mnemonic-convert-language-english)
- [`simplifiedchinese`](#mnemonic-convert-language-simplifiedchinese)
- [`traditionalchinese`](#mnemonic-convert-language-traditionalchinese)
- [`czech`](#mnemonic-convert-language-czech)
- [`french`](#mnemonic-convert-language-french)
- [`italian`](#mnemonic-convert-language-italian)
- [`japanese`](#mnemonic-convert-language-japanese)
- [`korean`](#mnemonic-convert-language-korean)
- [`portuguese`](#mnemonic-convert-language-portuguese)
- [`spanish`](#mnemonic-convert-language-spanish)

### `english` {#mnemonic-convert-language-english}

See [`mnemonic bundle --language english`](#mnemonic-bundle-language-english).

### `simplifiedchinese` {#mnemonic-convert-language-simplifiedchinese}

See [`mnemonic bundle --language simplifiedchinese`](#mnemonic-bundle-language-simplifiedchinese).

### `traditionalchinese` {#mnemonic-convert-language-traditionalchinese}

See [`mnemonic bundle --language traditionalchinese`](#mnemonic-bundle-language-traditionalchinese).

### `czech` {#mnemonic-convert-language-czech}

See [`mnemonic bundle --language czech`](#mnemonic-bundle-language-czech).

### `french` {#mnemonic-convert-language-french}

See [`mnemonic bundle --language french`](#mnemonic-bundle-language-french).

### `italian` {#mnemonic-convert-language-italian}

See [`mnemonic bundle --language italian`](#mnemonic-bundle-language-italian).

### `japanese` {#mnemonic-convert-language-japanese}

See [`mnemonic bundle --language japanese`](#mnemonic-bundle-language-japanese).

### `korean` {#mnemonic-convert-language-korean}

See [`mnemonic bundle --language korean`](#mnemonic-bundle-language-korean).

### `portuguese` {#mnemonic-convert-language-portuguese}

See [`mnemonic bundle --language portuguese`](#mnemonic-bundle-language-portuguese).

### `spanish` {#mnemonic-convert-language-spanish}

See [`mnemonic bundle --language spanish`](#mnemonic-bundle-language-spanish).

## `--passphrase` {#mnemonic-convert-passphrase}

The BIP-39 PBKDF2 passphrase (the "25th word"). Schema-`secret: true`.
XOR with `--passphrase-stdin` (the conditional-visibility engine
disables one when the other has a value).

## `--bip38-passphrase` {#mnemonic-convert-bip38-passphrase}

The BIP-38 Scrypt passphrase. **Distinct from `--passphrase`** —
they are two different cryptographic channels and a v0.8 BREAKING
change separated them. Use when `--from bip38` (decryption) or
`--to bip38` (encryption). Schema-`secret: true`. XOR with
`--bip38-passphrase-stdin`.

## `--bip38-passphrase-stdin` {#mnemonic-convert-bip38-passphrase-stdin}

Boolean. Read `--bip38-passphrase` from stdin (raw, NULL-byte
preserving). Closes the BIP-38 V3 spec NULL-byte argv gap.
Schema-`secret: true`. XOR with `--bip38-passphrase`.

## `--passphrase-stdin` {#mnemonic-convert-passphrase-stdin}

Boolean. Read `--passphrase` from stdin (raw, NULL-byte
preserving). Schema-`secret: true`. XOR with `--passphrase`.

## `--account` {#mnemonic-convert-account}

BIP-32 account index (default 0). Range 0..2_147_483_647.

## `--fingerprint` {#mnemonic-convert-fingerprint}

A master fingerprint (8 hex chars). Used as input on certain
edges where the destination needs origin metadata that the
source does not supply (e.g., `--from xpub --to mk1` requires
`--fingerprint` and `--path` to construct the mk1 origin block).

## `--xpub-prefix` {#mnemonic-convert-xpub-prefix}

SLIP-0132 prefix override for `--to xpub`. Allowed values
include the BIP-44 family (`xpub`, `ypub`, `zpub`, `Ypub`,
`Zpub`) and the testnet equivalents (`tpub`, `upub`, `vpub`,
`Upub`, `Vpub`). When omitted, defaults to the prefix that
matches the active `--template` + `--network`.

## `--electrum-version` {#mnemonic-convert-electrum-version}

Dropdown. Electrum seed-version selector for the
`(Entropy, ElectrumPhrase)` edge. Two allowed values; **2FA
versions are explicitly refused** (per
`crates/mnemonic-toolkit/src/cmd/convert.rs:272-286`'s
`parse_electrum_version_arg`).

### Outline {#mnemonic-convert-electrum-version-outline}

- [`standard`](#mnemonic-convert-electrum-version-standard)
- [`segwit`](#mnemonic-convert-electrum-version-segwit)

### `standard` {#mnemonic-convert-electrum-version-standard}

Electrum's "standard" seed-version (legacy P2PKH derivation).

### `segwit` {#mnemonic-convert-electrum-version-segwit}

Electrum's "segwit" seed-version (P2WPKH derivation).

## `--electrum-language` {#mnemonic-convert-electrum-language}

Electrum-specific wordlist (distinct from `--language` which is
BIP-39's set). Accepts `english` plus 4 non-English options. Used
only with `--from electrum-phrase` or `--to electrum-phrase`.

## `--script-type` {#mnemonic-convert-script-type}

Dropdown. Script-type selector for `(Xpub, Address)` derivation
— disambiguates which SegWit / Taproot family the address belongs
to. Required when `--to address` and the source does not
unambiguously imply the script type.

### Outline {#mnemonic-convert-script-type-outline}

- [`p2wpkh`](#mnemonic-convert-script-type-p2wpkh)
- [`p2sh-p2wpkh`](#mnemonic-convert-script-type-p2sh-p2wpkh)
- [`p2tr`](#mnemonic-convert-script-type-p2tr)

### `p2wpkh` {#mnemonic-convert-script-type-p2wpkh}

Native SegWit P2WPKH (BIP-84 family). Address prefix `bc1q` on
mainnet.

### `p2sh-p2wpkh` {#mnemonic-convert-script-type-p2sh-p2wpkh}

Nested P2SH-P2WPKH (BIP-49 family). Address prefix `3` on
mainnet.

### `p2tr` {#mnemonic-convert-script-type-p2tr}

Taproot P2TR (BIP-86 family). Address prefix `bc1p` on mainnet.

## `--json` {#mnemonic-convert-json}

Boolean. Emit each `--to` output as a JSON envelope instead of
plain stdout. The envelope shape is `{"node": "<name>", "value":
"<...>"}` per output, joined as a JSON array when multiple
`--to` rows are present.

## Worked example — phrase → ms1

This is the same conversion shown in chapter 32's tour
walkthrough.

1. **mnemonic** tab; pick **Convert (between formats)**.
2. `--from`: pick `phrase` from the node Dropdown; paste the
   canonical phrase into the value field:

   ```text
   abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
   ```

3. `--to`: pick `ms1`.
4. Leave all other flags empty.
5. Click **Run**. The run-confirm modal appears (the `phrase`
   node is secret-class). Click **Run** in the modal.

Output panel stdout:

```{.text include="44-convert-phrase-to-ms1.out"}
ms1: ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq 34v7f
```

The output is the canonical `ms1` for the all-`abandon` vector,
emitted with the `ms1:` label prefix and the default 5-character
display grouping (drop the grouping with `--group-size 0`). The
underlying unbroken string `ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f`
is byte-identical to the bundle output and to the
`docs/manual/src/40-cli-reference/43-ms.md:40` reference.

## Worked example — multi-output (xpub → fingerprint + path + address)

1. `--from`: `xpub` with the canonical mainnet BIP-84 xpub
   (extract from `mk decode <canonical mk1>`).
2. `--to`: add three rows: `fingerprint`, `path`, `address`.
3. `--script-type`: `p2wpkh`.
4. **Run** (no modal — xpub source is public).

Output panel stdout (one line per `--to` row in declaration order):

```text
73c5da0a
m/84'/0'/0'
bc1q...
```

## Refusals

The `convert` command's edge graph has many one-way barriers and
sibling-format pivots that cannot be performed as single-format
conversions. The toolkit emits specific refusal classes:

| Trigger | Refusal class |
|---|---|
| `--to <X>` from a `--from <Y>` where the edge is cryptographically one-way | `--to X is cryptographically unrecoverable from --from Y (one-way derivation barrier)` (per `cmd/convert.rs::refusal_one_way`) |
| `--from <X> --to <Y>` is a sibling-format pivot (multi-output bundle) | `--from X --to Y is a sibling-format pivot, not a single-format conversion. Use 'mnemonic bundle' instead.` (per `refusal_sibling_pivot`) |
| `--from xpub --to mk1` (any arguments) | `--to mk1 requires a policy descriptor binding (mk1 cards bind xpubs to specific policies via policy_id_stubs). Use 'mnemonic bundle --slot @0.xpub=... --template ...' to emit a complete bundle.` (byte-exact per `refusal_xpub_to_mk1`) |
| `--to address` from `phrase`/`entropy` without `--script-type` | `refusal_address_no_script_type` — directs the user to supply `--script-type p2wpkh` / `p2sh-p2wpkh` / `p2tr` |
| `--electrum-version standard-2fa` (or `segwit-2fa` / `101` / `102`) | `electrum 2FA seed-versions are not supported` (per `convert.rs:272-286`) |
| `--passphrase` AND `--passphrase-stdin` | clap-level `conflicts_with` |
| `--bip38-passphrase` AND `--bip38-passphrase-stdin` | clap-level `conflicts_with` |
| `--from <node>=<empty>` | `--from <name> value is empty; supply a non-empty value (or '-' to read from stdin)` |

The full edge-by-edge refusal matrix is documented in the SPEC's
typed-graph appendix; the CLI manual at
`docs/manual/src/40-cli-reference/41-mnemonic.md` `convert`
section cross-references each edge.

## Advisories

All `warning:` strings below are byte-exact mirrors of
`crates/mnemonic-toolkit/src/secret_advisory.rs::secret_in_argv_warning`'s
`warning: secret material on argv ({flag}) — pipe via {alternative} to avoid /proc/$PID/cmdline exposure`
format.

| Trigger | Stderr advisory |
|---|---|
| Inline `--from <secret-class>=<value>` | `warning: secret material on argv (--from <name>=) — pipe via --from <name>=- to avoid /proc/$PID/cmdline exposure` |
| Inline `--passphrase <value>` | `warning: secret material on argv (--passphrase) — pipe via --passphrase-stdin to avoid /proc/$PID/cmdline exposure` |
| Inline `--bip38-passphrase <value>` | `warning: secret material on argv (--bip38-passphrase) — pipe via --bip38-passphrase-stdin to avoid /proc/$PID/cmdline exposure` |
| `--from minikey=<value>` (the wider `is_argv_secret_bearing` predicate fires here, even though `is_secret_bearing` returns false) | argv-leakage advisory in the same `pipe via` format as the secret-class rows above |
