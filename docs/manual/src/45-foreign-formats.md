# Foreign wallet formats {#foreign-wallet-formats}

A reference for the third-party wallet-export formats the toolkit
imports via `mnemonic import-wallet`. Each format described here
is a *foreign blob* — a wire-shape emitted by a wallet other than
this toolkit, that v0.26.0+ knows how to ingest, parse, and round-
trip back to its canonical form.

The chapter complements
[`mnemonic import-wallet`](#mnemonic-import-wallet) (subcommand
reference) and
[`mnemonic export-wallet`](#mnemonic-export-wallet) (the watch-
only emit side, which targets a partially overlapping but distinct
format set).

## Overview

A *foreign format* is any wire shape that originates outside this
toolkit's `bundle`/`verify-bundle`/`convert` triad. v0.28.0 supports
eight on the import side:

- **BSMS Round-2 (BIP-129).** Plaintext multi-line shape emitted
  by Bitcoin coordinators (Coldcard, Specter, Bitcoin Core's
  miniscript-aware wallets, etc.) as the second round of the
  BIP-129 multisig setup protocol. The toolkit accepts the
  BIP-129-canonical 4-line shape, the toolkit's lenient 6-line
  consolidation, and a 2-line excerpt.
- **Bitcoin Core `listdescriptors` JSON.** The RPC output of
  `bitcoin-cli listdescriptors`, carrying one or more descriptor
  entries plus wallet-state metadata (`active`, `internal`,
  `range`, `timestamp`).
- **Sparrow Wallet JSON.** Sparrow Desktop's wallet-export JSON
  (`policyType` + `defaultPolicy.miniscript.script` + `keystores[]`).
- **Specter-DIY JSON.** Specter Desktop's bare wallet-export JSON
  (`blockheight` + `descriptor` + `devices[]`).
- **Coldcard single-sig `wallet.json`.** Coq's "Export Wallet"
  single-sig artifact (per-BIP derivation blocks: `bip44` / `bip49`
  / `bip84` / `bip86`).
- **Coldcard multisig text file.** Coldcard's "Multisig Setup file"
  text format (`Name:` / `Policy:` / `Format:` / `Derivation:` +
  per-cosigner `<XFP>: <xpub>` lines).
- **Blockstream Jade JSON.** Jade's `get_registered_multisig` RPC
  reply (top-level `multisig_file` field whose value is a Coldcard-
  multisig-text body).
- **Electrum 4.x wallet file.** Electrum's on-disk JSON
  (`seed_version` + `wallet_type` + `keystore` or `x1/`...`xN/`
  per-cosigner blocks).

All eight formats are *watch-only by construction*: they carry xpubs
and origin paths but not secret material. (Bitcoin Core's
`listdescriptors true` variant CAN carry `xprv`; the toolkit
**refuses** that input — see [§3 below](#bitcoin-core-listdescriptors).
Electrum's encrypted wallet files and 2fa/imported wallet variants
are also refused — see [§9 below](#electrum-wallet-file).)

Importing a foreign blob yields an m-format `bundle` whose
cosigners are populated from the blob's xpubs. The `ms1` slot is
the watch-only sentinel `""` until the user re-attaches secret
material via the `--ms1` / `--slot @N.phrase=` seed-overlay flags
(see [`import-wallet` seed overlay](#mnemonic-import-wallet-seed-overlay)).

## BSMS Round-2 (BIP-129) {#bsms-round-2}

BSMS — *Bitcoin Secure Multisig Setup* — is the BIP-129 multisig
coordinator protocol. The toolkit consumes its *Round-2* artifact:
a coordinator-emitted plaintext blob carrying the assembled
descriptor + a per-cosigner audit envelope (token, signature, first-
address verification value).

### Accepted shapes

**2-line shape** (lenient):

```text
BSMS 1.0
<descriptor>#<checksum>
```

Surfaces from minimal coordinators (some web tools, hand-rolled
fixtures) and from the kickoff seed-case for v0.26.0 — the flagship
decaying-multisig descriptor `wsh(thresh(...))`. The toolkit
accepts this excerpt and emits stderr `warning: import-wallet: bsms:
2-line excerpt; full BIP-129 Round-2 carries token + signature + first-address verification fields; accepting reduced form`.

**4-line shape** (BIP-129-canonical Round-2; v0.28.0):

```text
BSMS 1.0
<descriptor>#<checksum>
<path-restrictions>
<first-address>
```

This is the BIP-129 §Specification *Round 2* on-disk plaintext shape:
version header, descriptor (with BIP-380 `#checksum`), path-
restrictions, and the wallet's first address at `/0/0` (derived via
`crate::derive_address::derive_first_address`). The BIP-129 audit
envelope's token + HMAC + signature travel *out-of-band* with the
coordinator and are NOT in the plaintext blob. Line 3's path-
restrictions string emits `/0/*,/1/*` for canonical multipath
descriptors (`<0;1>/*` cosigner keys), `/0/*` for single-receive-
branch descriptors, or `No path restrictions` otherwise (per SPEC
§3.5.1). v0.28.0 parses this shape natively (no fallback). When the
parser falls through the 4-line arm to the legacy 6-line arm, a
stderr DEPRECATION notice fires.

**6-line shape** (legacy toolkit consolidation; deprecated):

```text
BSMS 1.0
<TOKEN>
<descriptor>#<checksum>
<DERIVATION_PATH>
<FIRST_ADDRESS>
<SIGNATURE>
```

A *lenient toolkit-specific consolidation* of BIP-129's plaintext
(4 lines: version, token, descriptor, derivation_path) plus the
BIP-129 *envelope*-side first-address verification value + signature
flattened into the same blob. An importer that doesn't decrypt the
BIP-129 encryption envelope can still preserve and audit those
fields. This shape was added before v0.28.0 promoted the 4-line
canonical parser to first-class status; it remains supported but
emits a stderr DEPRECATION notice.

### Where it comes from

Coordinators that emit BSMS Round-2 include Coldcard ("Multisig
Wallets > Make Multisig Wallet" on Mk4 / Q firmware emits a BSMS
blob alongside the multisig text file), Specter Desktop ("New
Multisig Wallet > Export setup"), and miniscript-patched Bitcoin
Core forks. Hand-rolled fixtures and test vectors (such as the
v0.26.0 kickoff `wsh(thresh(2, pk(@0), s:pk(@1), sln:older(32768)))`
decaying-multisig descriptor) typically use the 2-line shape.

### Audit fields and the round-trip drop

BIP-129's audit envelope (`token`, `signature`, `first_address`,
`derivation_path`) is *coordinator-output-side metadata*. It cannot
be regenerated from a bundle alone: the HMAC token requires the
coordinator's keying material, and the signature would need to be
re-signed by the same key.

Consequently, the **round-trip discipline** for BSMS is
*descriptor-only* (SPEC §7.3.1):

1. Importing a 6-line BSMS Round-2 → bundle DROPS audit metadata
   from the canonicalize comparison.
2. Re-exporting the bundle → BSMS Round-2 emits a 2-line shape
   (no synthesis of fresh token/signature/first-address).
3. The `--json` envelope's `bsms_audit` field preserves the
   original audit metadata for the user to re-attach manually.

A `--coordinator-key <FILE>` flag enabling re-signed Round-2
export is queued as FOLLOWUP `bsms-audit-field-regeneration`.

In v0.26.0, BSMS *re-emission* via `export-wallet --format bsms`
is unimplemented (FOLLOWUP `wallet-export-bsms-emitter`). When the
emitter is absent, the `--json` envelope reports the discriminator
`roundtrip.status: "blocked_no_emitter"` to indicate that the
round-trip discipline is not yet evaluable for this blob — not
that the blob is malformed.

### Signature verification — deferred

`<SIGNATURE>` is preserved verbatim in
`ParsedImport.bsms_audit.signature` and reflected to the `--json`
envelope, but **v0.26.0 does not verify it**. The user receives a
stderr WARNING noting that the signature is present but not
verified in v0.26.0 — see FOLLOWUP `bsms-verify-signatures`.
First-address verification (deriving the descriptor at the declared
path and comparing against the FIRST_ADDRESS field) is similarly
deferred to FOLLOWUP `bsms-first-address-verify` (v0.27+).

## Bitcoin Core `listdescriptors` {#bitcoin-core-listdescriptors}

Bitcoin Core's `listdescriptors` RPC emits a JSON envelope
describing the descriptors registered in the active wallet. The
toolkit ingests both the canonical wrapper shape and a bare-array
shape some older Core clients emit.

### Accepted shape

Top-level JSON wrap:

```json
{
  "wallet_name": "<name>",
  "descriptors": [
    {
      "desc": "wsh(sortedmulti(2,[fp1/48h/0h/0h/2h]xpub.../<0;1>/*,[fp2/48h/0h/0h/2h]xpub.../<0;1>/*))#abcdefgh",
      "timestamp": 1700000000,
      "active": true,
      "internal": false,
      "range": [0, 999],
      "next": 0,
      "next_index": 0
    }
  ]
}
```

### `listdescriptors` vs `listdescriptors true`

The Bitcoin Core RPC distinguishes two invocations:

- `bitcoin-cli listdescriptors` — emits xpub-only descriptors.
  This is the toolkit-supported shape.
- `bitcoin-cli listdescriptors true` — emits the **private**
  variant carrying `xprv` extended *private* keys in place of
  xpubs. The `true` argument tells Core to include private
  material in the export.

The toolkit **refuses** the second variant. Any descriptor entry
whose `desc` field contains `xprv` triggers `error: import-wallet:
bitcoin-core: xprv-bearing descriptor refused; re-run
\`bitcoin-cli listdescriptors\` without \`true\` to get xpub-only
output` (exit 2). Rationale: an extended private key is the full
spending key material for a BIP-32 subtree — accepting it would
silently elevate an "import" subcommand into a secret-material
ingest path, violating the watch-only invariant the wallet-
import surface enforces (SPEC §8.2).

### Per-entry metadata fields

| Field | Round-trip discipline |
|---|---|
| `desc` | preserved (canonicalized via `MsDescriptor::from_str` + re-checksum) |
| `wallet_name` | preserved (top-level metadata) |
| `active` | preserved (drives `--select-descriptor active-receive` / `active-change` filtering, SPEC §5.3) |
| `internal` | preserved (drives `active-change` selector) |
| `range` | preserved (byte-equality on the `[start, end]` pair) |
| `timestamp` | DROPPED on round-trip; stderr NOTICE on input presence |
| `next` | DROPPED on round-trip; stderr NOTICE on input presence |
| `next_index` | DROPPED on round-trip; stderr NOTICE on input presence |

The three dropped fields are *wallet-state* (the user's current
scan position, the next derivation index, the timestamp at which
the descriptor was added), not *key-state*. They cannot be
faithfully reconstructed from a bundle alone, so the toolkit
deliberately discards them to keep the canonicalize comparison
sound. A stderr NOTICE fires when any are present in the input:
`notice: import-wallet: bitcoin-core: dropped wallet-state fields
<fields>: not preserved in bundle output (key-state only)`.

### `--select-descriptor` filtering

A Bitcoin Core wallet typically exports four descriptor entries
(BIP-84 receive + change, plus sometimes BIP-86 receive + change).
The `--select-descriptor` flag (`all` / integer / `active-receive`
/ `active-change`) selects which entries become bundles; see
[`import-wallet`'s flag reference](#mnemonic-import-wallet) for
the full enumeration. Under `--format bsms`, any non-default value
emits stderr NOTICE (BSMS Round-2 carries a single descriptor) and
is treated as `all`.

## Sparrow Wallet (`--format sparrow`) {#sparrow-wallet}

Sparrow Desktop emits a wallet-export JSON whose distinctive shape
carries `policyType` (`"SINGLE"` or `"MULTI"`), `scriptType`
(`"P2WPKH"` / `"P2WSH"` / `"P2SH-P2WPKH"` / …),
`defaultPolicy.miniscript.script`, and a `keystores[]` array (per
SPEC §11.1). Sparrow stores the wallet script as a miniscript
fragment with `@N/**` placeholders inside `defaultPolicy.miniscript.script`;
the parser re-wraps it as `wsh(...)` / `sh(wsh(...))` per the script
type and ties cosigner xpubs from `keystores[i]`.

### Sniff signature

A blob is recognized as Sparrow iff its top-level JSON object carries
ALL of: `policyType` ∈ {`"SINGLE"`, `"MULTI"`}, `scriptType` (string),
`defaultPolicy.miniscript.script` (nested string), and a non-empty
`keystores` array. The vendor-marker quartet disambiguates Sparrow
from Bitcoin Core / Specter / other JSON formats; sniff is positive-
marker-based with no false-positive co-fire risk against the other
v0.28.0 parsers.

### CLI invocation

```sh
mnemonic import-wallet --format sparrow \
  --blob tests/fixtures/wallet_import/sparrow-singlesig-p2wpkh.json
```

The toolkit synthesizes a watch-only bundle whose `mk1` slot carries
the single cosigner xpub and whose `md1` slot carries the descriptor
`wpkh([5436d724/84'/0'/0']xpub6Bner3L3.../<0;1>/*)`. For the multisig
fixture `sparrow-multisig-2of3-p2wsh-sortedmulti.json`, three
cosigners populate `mk1` and the descriptor wraps `sortedmulti(2,...)`.

### Provenance metadata

The `--json` envelope's `bundle.import_provenance.sparrow` field
preserves Sparrow-specific metadata that doesn't ride the bundle
itself (per SPEC §11.1):

| Field | Source | Note |
|---|---|---|
| `label` | top-level `name` (or `label`) | preserved verbatim |
| `policy_type` | `policyType` | `"Single"` / `"Multi"` |
| `script_type` | `scriptType` | verbatim — `"P2WPKH"` / `"P2WSH"` / … |
| `dropped_fields` | runtime | Sparrow fields not lifted into the bundle (analogous to Bitcoin Core's drop list) |

### Round-trip example

```sh
# Import → JSON envelope
mnemonic import-wallet --format sparrow \
  --blob sparrow-singlesig-p2wpkh.json --json > envelope.json

# Re-emit via export-wallet (v0.37.0+: --template is auto-derived from
# the envelope descriptor; do NOT pass --template here, it conflicts)
mnemonic export-wallet --from-import-json envelope.json \
  --format sparrow > sparrow_re.json

# Compare under per-format canonicalize (semantic round-trip)
diff <(jq -S . sparrow-singlesig-p2wpkh.json) \
     <(jq -S . sparrow_re.json)
```

> **Note on the diff.** As written above, the `diff` is **empty** —
> the recipe round-trips the wallet name verbatim through the envelope
> (the v0.37.8 universal source-name lift carries
> `sparrow_source_metadata.label` back into the re-emitted `name` /
> `label` fields). To override the lifted name on re-emit, pass
> `--wallet-name "<chosen-name>"` explicitly: the CLI flag always
> beats the envelope-lifted name. The same lift covers Specter, Jade,
> Electrum, Bitcoin Core, and Coldcard-multisig sources — Specter no
> longer refuses `--from-import-json` for missing `--wallet-name` when
> the envelope carries a liftable name. Closes FOLLOWUP
> `sparrow-from-import-json-wallet-name-preservation`.

### Taproot import (shipped v0.31.1 + v0.31.2) {#taproot-import-shipped-v0311}

Sparrow's emit side ships taproot wallets in two shapes:

- **Taproot MULTISIG** (`tr-multi-a` / `tr-sortedmulti-a` per
  `wallet_export/sparrow.rs:215-219`) → *descriptor-passthrough*:
  concrete `[fp/path]xpub` keys embedded in
  `defaultPolicy.miniscript.script` directly (no `@N/**` placeholders).
- **Taproot SINGLESIG** (Bip86 per `wallet_export/sparrow.rs:195`) →
  *template-mode*: standard `@0/**` placeholder (e.g. `tr(@0/**)`).

**v0.31.1+ Cycle 8** shipped the descriptor-passthrough import side
via a path-split at `wallet_import/sparrow.rs::parse` Step 6:
descriptor-passthrough shape (`tr(` AND no `@0/**` placeholder)
bypasses Step 5 substitution and feeds `script_template` directly
through the existing `concrete_keys_to_placeholders` →
`parse_descriptor` pipeline. Closes FOLLOWUP
`sparrow-taproot-descriptor-passthrough-import-support`.

**v0.31.2+ Cycle 9** collapsed the narrow refusal for taproot
SINGLESIG template-mode by routing `tr(@0/**)` through the standard
Step 5 substitution branch. The resulting
`tr([fp/86'/0'/0']xpub.../<0;1>/*)` descriptor is accepted cleanly by
the existing pipeline. Closes FOLLOWUP
`sparrow-taproot-singlesig-template-mode-import`.

The direct export-wallet path requires a recognized `--template` (no
descriptor-passthrough); taproot-multisig emit is supported via
`--template tr-multi-a` / `tr-sortedmulti-a`. **On the `--from-import-json`
path (v0.37.0+) the `--template` is auto-derived from the envelope's
descriptor**, so you omit it (passing `--template` there is a clap
conflict). Import auto-sniff still fires (sniff is `policyType`-based, not
script-content-based).

**Round-trip note:** import → JSON envelope works for both taproot
shapes; re-emission via `export-wallet --from-import-json` is gated on
the orthogonal FOLLOWUP `wallet-import-taproot-internal-key` (envelope
wire-shape doesn't yet surface NUMS-vs-raw-xonly internal-key
designation). To re-emit, use `--format <emitter> --descriptor <body>`
directly.

## Specter-DIY (`--format specter`) {#specter-diy}

Specter Desktop emits a bare wallet-export JSON whose distinctive
shape carries a top-level `blockheight` integer alongside `label`,
`descriptor`, and `devices[]` (per SPEC §11.2). The `blockheight`
field is the load-bearing sniff marker — no other v0.28.0 format
carries it at JSON top level.

### Sniff signature

A blob is recognized as Specter iff its top-level JSON object carries
ALL of: `label` (string), `blockheight` (integer), `descriptor`
(string), and `devices` (array). The `blockheight` integer is the
strongest discriminator (Sparrow doesn't carry it; Bitcoin Core
doesn't carry it).

### CLI invocation

```sh
mnemonic import-wallet --format specter \
  --blob tests/fixtures/wallet_import/specter-singlesig-p2wpkh.json
```

The parser extracts `descriptor` verbatim (preserving its `#<checksum>`
trailer) and lifts `label` into the bundle as the wallet name.
`devices[]` becomes per-cosigner provenance hints (vendor strings
like `"coldcard"`, `"trezor"`, `"unknown"`) — informational only,
not load-bearing for descriptor parse.

### Provenance metadata

The `--json` envelope's `bundle.import_provenance.specter` field
preserves Specter-specific metadata:

| Field | Source | Note |
|---|---|---|
| `label` | top-level `label` | preserved verbatim |
| `blockheight` | top-level `blockheight` | u64 — wallet's import block height |
| `devices` | top-level `devices[]` | `Vec<{device_type, label}>` |
| `dropped_fields` | runtime | any Specter fields not lifted into the bundle |

### Round-trip example

```sh
mnemonic import-wallet --format specter \
  --blob specter-singlesig-p2wpkh.json --json > envelope.json
mnemonic export-wallet --from-import-json envelope.json \
  --format specter --wallet-name "Specter re-export" > specter_re.json
```

`blockheight` is preserved in the provenance metadata but DROPPED on
the canonicalize-side comparison (it's wallet-state, not key-state —
analogous to Bitcoin Core's `timestamp` field).

## Coldcard single-sig wallet.json (`--format coldcard`) {#coldcard-singlesig}

Coldcard hardware-wallet firmware (Mk1 through Q) exports a single-sig
wallet manifest as a JSON file carrying per-BIP derivation blocks
(`bip44` / `bip49` / `bip84` / `bip86`) alongside a top-level `chain`
field (`"BTC"` mainnet or `"XTN"` testnet) and master fingerprint
`xfp` (per SPEC §11.3).

### Sniff signature

A blob is recognized as Coldcard single-sig iff its top-level JSON
object carries ALL of:

- `chain` ∈ {`"BTC"`, `"XTN"`}
- `xfp` (8-char uppercase hex string)
- At-least-one-of: `xpub`, `bip44`, `bip49`, `bip84`, `bip86`,
  `bip48_1`, `bip48_2`

The disjunction in the third clause absorbs firmware variance —
different Coldcard firmware eras emit different combinations of
per-BIP derivation blocks:

| Firmware era | Emits | Discriminator |
|---|---|---|
| Mk1/Mk2 (pre-2022) | top-level `xpub` only | legacy `xpub` field |
| Mk3 (2022+) | `bip44` / `bip49` / `bip84` blocks | per-bipN sub-objects |
| Mk4 (2023+) | + `bip86` block (taproot) | adds `bip86` |
| Q (2024+) | + `bip48_1` / `bip48_2` (multisig hints) | adds `bip48_*` |

### Dominant-BIP selection

Coldcard typically exports several per-BIP blocks side-by-side. The
parser picks ONE dominant block per the heuristic at SPEC §11.3.1:
`bip86` (taproot) > `bip84` (P2WPKH) > `bip49` (P2SH-P2WPKH) > `bip44`
(P2PKH); fall back to legacy top-level `xpub` with SLIP-132-prefix
inference (`zpub` → BIP-84, `ypub` → BIP-49, `xpub` → BIP-44). The
`bip48_*` multisig-hint blocks are IGNORED by the single-sig parser
(use [Coldcard multisig text](#coldcard-multisig) instead).

### CLI invocation

```sh
mnemonic import-wallet --format coldcard \
  --blob tests/fixtures/wallet_import/coldcard-singlesig-bip84-mainnet.json
```

For the BIP-84 mainnet fixture, the result is a watch-only bundle
whose descriptor is `wpkh([b8688df1/84'/0'/0']xpub6FQya7zGhR9.../<0;1>/*)`
with `mk1` carrying the lone cosigner xpub.

### Provenance metadata

The `--json` envelope's `bundle.import_provenance.coldcard` field
preserves Coldcard-specific metadata:

| Field | Source | Note |
|---|---|---|
| `chain` | top-level `chain` | `Btc` (mainnet) / `Xtn` (testnet) |
| `xfp` | top-level `xfp` | 4-byte master fingerprint |
| `bip_derivation` | dominant block | `Bip44` / `Bip49` / `Bip84` / `Bip86` |
| `raw_account` | dominant block | `account` integer (typically 0) |
| `dropped_fields` | runtime | per-BIP blocks not selected as dominant |

### Round-trip example

```sh
mnemonic import-wallet --format coldcard \
  --blob coldcard-singlesig-bip84-mainnet.json --json > envelope.json
mnemonic export-wallet --from-import-json envelope.json \
  --format coldcard > coldcard_re.json   # v0.37.0+: --template auto-derived
```

### Deferral — legacy Mk1/Mk2 xpub-prefix inference

Coldcard Mk1/Mk2 firmware emits only the legacy top-level `xpub`
field (no per-BIP blocks). v0.28.0's parser handles this case by
inferring the dominant BIP from the xpub's SLIP-132 prefix (zpub →
BIP-84, ypub → BIP-49, xpub → BIP-44), but the inference is heuristic
and not all legacy variants are covered. Cycle-FOLLOWUP
`wallet-import-coldcard-legacy-mk1-mk2-xpub-prefix-inference-edge-cases`
tracks edge cases as users surface them.

## Coldcard multisig text (`--format coldcard-multisig`) {#coldcard-multisig}

Coldcard's "Multisig Setup File" emit is a **text** format (not JSON,
distinct from the single-sig wallet.json above). The file shape is
line-oriented (per SPEC §11.4):

```text
Name: <wallet-name>
Policy: <K>-of-<N>
Format: <script-type>          # P2WSH, P2SH-P2WSH, or P2SH
Derivation: m/...
[XFP: <hex>]                   # optional header (firmware-variant)

<xfp1>: <xpub1>
<xfp2>: <xpub2>
...
```

### Sniff signature

A blob is recognized as Coldcard-multisig iff its first three non-
blank lines (in order) match `Name: <...>`, `Policy: <K>-of-<N>`,
`Format: <...>`. The `XFP: <hex>` header line is optional —
firmware variants disagree on its presence.

### CLI invocation

```sh
mnemonic import-wallet --format coldcard-multisig \
  --blob tests/fixtures/wallet_import/coldcard-ms-2of3-p2wsh-with-xfp.txt
```

For the 2-of-3 P2WSH fixture, the parser synthesizes
`wsh(sortedmulti(2, [34a3a4f1/48'/0'/0'/2']xpub6FQya..., ...))` and
populates `mk1` with all three cosigner xpubs.

### XFP-header policy (5-row truth table)

The `XFP:` header line interacts with the computed fingerprint (from
the master xpub or per-cosigner xpubs) per the table at SPEC §11.4.1.
The header-matches-computed case is the silent-pass row; the
header-disagrees case emits a stderr WARNING but proceeds with the
header value as authoritative:

```text
warning: import-wallet: coldcard-multisig: xfp header `XFP: <hex>`
  disagrees with computed fingerprint `<hex>` from cosigner xpub;
  using blob-supplied header value as authoritative
```

### Provenance metadata

The `--json` envelope's `bundle.import_provenance.coldcard_multisig`
field preserves the multisig metadata:

| Field | Source | Note |
|---|---|---|
| `name` | `Name:` line | wallet name |
| `policy` | `Policy:` line | `(k, n)` u8 pair |
| `script_format` | `Format:` line | `P2wsh` / `P2shP2wsh` / `P2sh` |
| `xfp_was_blob_supplied` | header present | `true` if `XFP:` line in blob |
| `xfp_header_disagreed` | computed compare | `true` if WARNING fired |
| `dropped_fields` | runtime | non-load-bearing lines |

### Round-trip example

```sh
mnemonic import-wallet --format coldcard-multisig \
  --blob coldcard-ms-2of3-p2wsh-with-xfp.txt --json > envelope.json
mnemonic export-wallet --from-import-json envelope.json \
  --format coldcard > coldcard_ms_re.txt   # v0.37.0+: template + threshold
                                           # both derived from the envelope
diff coldcard-ms-2of3-p2wsh-with-xfp.txt coldcard_ms_re.txt
```

> **Note on the diff.** The `diff` is **non-empty** by design:
> Coldcard-multisig re-emit strips fixture comment lines and writes a
> normalized header order. Semantic equivalence (xpubs / derivation /
> threshold / policy) is preserved. The `Name:` header is also
> preserved verbatim through the envelope (v0.37.8 universal
> source-name lift carries `coldcard_multisig_source_metadata.name`
> back into the re-emit; pre-v0.37.8 the name was replaced by the
> `imported-descriptor` placeholder).
>
> **Format-name parity (v0.28.4+).** Both `--format coldcard` and
> `--format coldcard-multisig` are accepted on the **export** side
> (v0.28.4 closed the prior asymmetry). The two values produce
> identical output for multisig templates; `coldcard-multisig`
> additionally refuses singlesig templates (`bip44`/`bip49`/`bip84`)
> with a pointer to `--format coldcard`. The recipe above uses
> `--format coldcard` for backward compatibility with v0.28.0–v0.28.3
> readers; `--format coldcard-multisig` is equivalent here (on the
> `--from-import-json` path the template + threshold are auto-derived
> from the envelope, v0.37.0+; on the direct `--template` path it would
> be `--format coldcard-multisig --template wsh-sortedmulti --threshold 2`).

## Blockstream Jade (`--format jade`) {#jade-multisig}

Blockstream's Jade hardware wallet exposes multisig wallet registration
via the `register_multisig` RPC; the inverse `get_registered_multisig`
RPC reply carries a top-level `multisig_file` field whose value is the
**Coldcard-multisig text body** verbatim (per SPEC §11.5).

### Sniff signature

A blob is recognized as Jade iff its top-level JSON object carries a
`multisig_file` field whose value is a non-empty string. The field
name `multisig_file` is the distinctive marker — no other format
uses it.

### Parse contract

The parser extracts the `multisig_file` string and delegates to the
Coldcard-multisig text parser ([§7 above](#coldcard-multisig)). The
provenance is annotated as Jade rather than Coldcard so the round-trip
emit re-wraps the body in Jade's JSON envelope.

### CLI invocation

```sh
mnemonic import-wallet --format jade \
  --blob tests/fixtures/wallet_import/jade-multisig-2of3-p2wsh.json
```

The Jade JSON wrapper is:

```json
{
  "id": "jade-test-request-001",
  "multisig_name": "TestMs2of3",
  "multisig_file": "Name: TestMs2of3\nPolicy: 2 of 3\nDerivation: ...\n..."
}
```

The parser extracts `multisig_file`, delegates parse to
`coldcard_multisig::parse_text()`, and tags the resulting bundle's
provenance as Jade with the inner Coldcard-multisig metadata embedded.

### Provenance metadata

```rust
pub(crate) struct JadeSourceMetadata {
    pub coldcard_compat: ColdcardMultisigSourceMetadata,
    pub jade_specific_fields: Vec<String>,  // empty in v0.28.0
}
```

The `jade_specific_fields` field is reserved for future Jade-only
metadata once the SeedQR variant ships (see deferral below).

### Round-trip example

```sh
mnemonic import-wallet --format jade \
  --blob jade-multisig-2of3-p2wsh.json --json > envelope.json
mnemonic export-wallet --from-import-json envelope.json \
  --format jade > jade_re.json   # v0.37.0+: template + threshold auto-derived
```

### SeedQR (Jade + SeedSigner + others)

SeedQR is an open spec originated by SeedSigner; Blockstream Jade and
several other wallets (Coldcard, Cobo, Krux) adopted it. Because SeedQR
encodes a BIP-39 seed (not a wallet policy), it does NOT round-trip
through `mnemonic import-wallet` — instead, decode the SeedQR payload
to a phrase via `mnemonic seedqr decode`, then feed the phrase into
`mnemonic bundle` or any other downstream subcommand.

See [`mnemonic seedqr`](40-cli-reference/41-mnemonic.md#mnemonic-seedqr)
for the encode/decode subsurface (v0.30.0+).

## Electrum 4.x wallet file (`--format electrum`) {#electrum-wallet-file}

Electrum 4.x stores wallets as Python-dict-serialized JSON on disk.
The toolkit imports singlesig and `<k>of<n>` multisig variants; 2fa,
imported, and encrypted variants are refused with format-specific
stderr errors (per SPEC §11.6).

> **Disambiguation:** `--format electrum` (this section) is the
> *Electrum wallet file* parser. It is **distinct** from
> [`mnemonic electrum {encode,decode}`](#mnemonic-electrum) which is
> Electrum's native *seed format* codec (BIP-39-alternative entropy
> serialization). See SPEC §1.4.

### Sniff signature

A blob is recognized as Electrum iff its top-level JSON object carries
ALL of:

- `seed_version` (integer ∈ {11..71}; current FINAL_SEED_VERSION is 71)
- `wallet_type` (string ∈ {`"standard"`, `"<k>of<n>"`, `"2fa"`,
  `"imported"`})

The `wallet_type` value-set follows Electrum's `electrum/util.py::multisig_type`
regex `(\d+)of(\d+)` for multisig — values like `"2of3"`, `"3of5"` are
matched, NOT the literal string `"multisig"`.

### Parse contract

| `wallet_type` | Action |
|---|---|
| `"standard"` | Singlesig parse: extract `keystore.xpub` + `keystore.derivation`. Compute descriptor via standard BIP-84/49/44 wrapping based on the xpub's SLIP-132 prefix. |
| `<k>of<n>` (regex `(\d+)of(\d+)`) | Multisig parse: iterate `x1/`, `x2/`, … sub-objects; extract per-cosigner xpub + derivation. Synthesize `wsh(sortedmulti(K, ...))`. |
| `"2fa"` | **REFUSE** — TrustedCoin two-factor wallet; not reconstructible from xpubs alone. |
| `"imported"` | **REFUSE** — "imported addresses" wallet has no derivation chain. |

Encrypted wallets (`use_encryption: true` + base64-encrypted sensitive
fields) are imported as **watch-only** at v0.30.1+. Per Electrum's
`electrum/keystore.py`, the field-level encryption protects only the
seed-material fields (`keystore.seed` / `keystore.xprv` / `keystore.passphrase`
/ `keystore.keypairs`). The watch-only fields (`keystore.xpub`,
`keystore.derivation`, `keystore.root_fingerprint`, `keystore.label`) are
plaintext under both encrypted and unencrypted wallets. The toolkit reads
only the watch-only fields and emits a stderr NOTICE advisory describing
the passthrough semantic; the encrypted seed/xprv/passphrase/keypairs
fields are ignored. To extract the encrypted seed, use `electrum
--decrypt-wallet` out-of-band then re-import the plaintext wallet.

### Refusal stderr templates

```text
error: import-wallet: electrum: 2fa wallets require TrustedCoin
  two-factor restoration; ingest not supported

error: import-wallet: electrum: imported-addresses wallets have no
  derivation chain to reconstruct; ingest not supported
```

### Encrypted-wallet NOTICE advisory (v0.30.1+)

```text
notice: import-wallet: electrum: wallet is encrypted (use_encryption=true);
  importing watch-only material only (encrypted seed/xprv/passphrase/keypairs
  fields ignored). To extract the encrypted seed, use 'electrum
  --decrypt-wallet' out-of-band then re-import the plaintext wallet.
```

### CLI invocation

```sh
mnemonic import-wallet --format electrum \
  --blob tests/fixtures/wallet_import/electrum-standard-bip84-mainnet.json
```

Singlesig BIP-84 yields `wpkh([5436d724/84'/0'/0']zpub6qTB.../<0;1>/*)`.
The multisig fixture `electrum-multisig-2of3-wsh.json` (with
`wallet_type: "2of3"`) yields `wsh(sortedmulti(2, [b8688df1/48'/0'/0'/2']Zpub.../<0;1>/*, …))`.

### Provenance metadata

```rust
pub(crate) struct ElectrumSourceMetadata {
    pub seed_version: u64,
    pub wallet_type: ElectrumWalletType,    // Standard | Multisig { k, n }
    pub wallet_name: Option<String>,
    pub dropped_fields: Vec<String>,
}
```

Refused variants (`2fa` / `imported` / encrypted) do not produce a
`ParsedImport` and therefore have no provenance.

### Round-trip example

```sh
mnemonic import-wallet --format electrum \
  --blob electrum-standard-bip84-mainnet.json --json > envelope.json
mnemonic export-wallet --from-import-json envelope.json \
  --format electrum > electrum_re.json   # v0.37.0+: --template auto-derived
```

### Deferrals

- ~~**Encrypted wallet files** — refused at sniff/parse time~~ — imported
  as watch-only at v0.30.1+ with a stderr NOTICE advisory (the parser
  reads only plaintext xpub/derivation/fingerprint/label; encrypted
  seed/xprv/passphrase/keypairs fields are ignored). To extract the
  encrypted seed, decrypt out-of-band via `electrum --decrypt-wallet`,
  then re-feed the plaintext blob. See the §"Wallet-type classification"
  block above for the advisory text.
- **Whole-file storage encryption** (Format B; version-byte + AES-CBC + MAC)
  — out of scope (FOLLOWUP `wallet-import-electrum-encrypted-storage-format-b`).
- **Pre-4.x legacy `wallet_type` values** (`"old"`, `"xpub"`, `"bip44"`)
  — rejected at sniff time. Open the wallet in Electrum 4.x first
  (auto-upgrade rewrites `wallet_type` to `"standard"`), then export
  for import.
- **2fa and imported-addresses wallets** — refused by design (not
  reconstructible from xpubs alone).

## Commented descriptor (`--format descriptor`) {#commented-descriptor}

`--format descriptor` (v0.58.0) reads a watch-only concrete descriptor
from a plain-text file, tolerating leading `#`-comment lines and blank
lines. It is the import counterpart to `export-wallet --format descriptor`
and `--format green`: a green export is just two `#`-comment lines plus a
descriptor, so `--format descriptor` re-imports it directly, and the same
door accepts any hand-written or foreign descriptor that arrives as text.

### Accepted shape

One descriptor line (after `#`-comment and blank lines are stripped), with
inline `[fp/path]xpub` key origins:

```text
# Blockstream Green — Watch-only import (singlesig)
# Help: https://help.blockstream.com/...
wpkh([5436d724/84'/0'/0']xpub6Bner.../<0;1>/*)#00lx6ere
```

- **Singlesig and multisig.** Unlike `export-wallet --format green` (which
  is singlesig-only, because Blockstream Green's multisig is
  server-mediated), the descriptor *import* accepts a multisig
  `wsh(sortedmulti(...))` too — a descriptor string carries the threshold
  and all cosigners.
- **Checksum is tolerant.** The BIP-380 `#checksum` is validated if present
  (a wrong checksum is refused) and tolerated if absent — matching
  `bundle --descriptor`.
- **Watch-only out.** The result is a watch-only bundle (no secret
  material); the network is inferred from the BIP-48 coin-type in the key
  origins.

### Explicit-only (no auto-sniff)

A bare descriptor is too generic to auto-detect safely, so `--format
descriptor` is **required** — it is never selected by the sniffer (a file
with no `--format` and no recognized header is reported as "could not
detect format", exactly as before). This mirrors the encrypted-BSMS
"explicit `--format` required" rule.

### Refusals

- A file with **no** descriptor line (only comments/blanks) → refused.
- A file with **two or more** descriptor lines → refused (supply one).
- A **wrong** BIP-380 checksum → refused.

## Round-trip discipline {#foreign-formats-roundtrip}

For every imported blob `B`, the toolkit runs:

```text
let bundle  = mnemonic import-wallet --blob B;
let blob_re = mnemonic export-wallet --format F < bundle;
assert canonicalize(B) == canonicalize(blob_re);
```

The comparison distinguishes two outcomes:

- **`byte_exact: true`** — `B` and `blob_re` are byte-identical.
  The strongest possible round-trip guarantee.
- **`semantic_match: true, byte_exact: false`** — the
  canonicalize-per-format normalization (re-checksum + drop audit
  fields + re-render via `MsDescriptor::from_str().to_string()`)
  yields equal output, but the original input had cosmetic drift
  (whitespace, key ordering, dropped wallet-state fields).

In default mode, a non-byte-exact / semantic match prints
`warning: import-wallet: roundtrip not byte-exact; semantic
equivalent; diff below` to stderr followed by a unified diff
(RFC 5261 format). In `--json` mode the diff goes only in the
envelope's `roundtrip.diff` field.

The `roundtrip.status` discriminator (`"ok"` /
`"blocked_no_emitter"` / `"canonicalize_failed"`) tells
downstream consumers whether the round-trip is *evaluable* at
all (SPEC §7.4).

### Why we drop audit fields

The drop is a *correctness* choice, not a *minimization* one. BSMS
audit fields (`token`, `signature`, `first_address`,
`derivation_path`) are coordinator-output-only — not derivable
from bundle state without the coordinator's HMAC keying material.
Bitcoin Core `timestamp` / `next` / `next_index` are wallet-state,
not key-state; recovering them from a bundle would require either
timestamping the bundle (breaks determinism) or reading the chain
tip (introduces network dependency). Both classes are preserved in
the `--json` envelope's `bsms_audit` / `source_metadata` field so
the user can re-attach them externally if desired.

## What's NOT supported {#foreign-formats-not-supported}

v0.28.0 ships eight source formats. Recognized in the broader
ecosystem but NOT yet importable (each tracked by a queued
FOLLOWUP):

- **BSMS Round-1 token-only** (`wallet-import-bsms-round-1`) — the
  pre-descriptor handshake. No workaround until Round-1 parsing
  ships; the descriptor is not yet assembled at Round-1.
- ~~**BSMS encrypted envelopes** per BIP-129 §5~~ — shipped in v0.31.0
  via `--bsms-encryption-token <FILE|->` (PBKDF2-SHA512 + AES-256-CTR +
  HMAC-SHA256 per BIP-129 §Encryption; STANDARD + EXTENDED token widths).
  Encrypted blobs lack the `BSMS 1.0` header so `--format bsms` is
  REQUIRED. MAC verify failure → exit 2 (typed `BsmsMacMismatch`).
  Encrypted Round-1 decrypt-then-verify shipped in v0.31.2/v0.32.1;
  per-Signer TOKEN variants (repeatable `--bsms-encryption-token`)
  shipped in v0.32.2. **Cross-implementation validated** against (a)
  BIP-129 Test Vector 3 (`crates/mnemonic-toolkit/src/bsms_crypto.rs`
  unit tests) AND (b) the independent Coinkite Python reference
  (`coinkite/bsms-bitcoin-secure-multisig-setup`, pinned SHA
  `c30abe3a`) via vendored cross-impl fixtures — see
  `crates/mnemonic-toolkit/tests/external/README.md` for the regen
  recipe. Both Round-1 (STANDARD) + Round-2 (EXTENDED) directions
  cross-validated.
- ~~**Sparrow taproot descriptor-passthrough**~~ — shipped in v0.31.1
  via the Step 6 path-split at `wallet_import/sparrow.rs` (taproot
  MULTISIG branch). Taproot SINGLESIG template-mode (Bip86
  `tr(@0/**)`) shipped in v0.31.2 by collapsing the narrow refusal
  into the general substitution path. See
  [§Taproot import](#taproot-import-shipped-v0311) above.
- ~~**Jade SeedQR variant**~~ — shipped in v0.30.0 as a vendor-neutral subsurface. See [`mnemonic seedqr`](40-cli-reference/41-mnemonic.md#mnemonic-seedqr).
- ~~**Electrum encrypted wallet files**~~ — shipped in v0.30.1 as
  watch-only passthrough (parses plaintext xpub/derivation/etc., ignores
  encrypted seed/xprv/passphrase/keypairs, emits stderr NOTICE advisory).
  Whole-file Format-B encryption (`wallet-import-electrum-encrypted-storage-format-b`)
  remains deferred — see [§9 above](#electrum-wallet-file).
- **Electrum pre-4.x legacy `wallet_type` values**
  (`wallet-import-electrum-pre-4x-legacy-types`) — see [§9 above](#electrum-wallet-file)
  deferral note.

## Normative references {#foreign-formats-references}

- **BIP-129** — *Bitcoin Secure Multi-Sig Setup (BSMS)*. Defines
  Round-1 and Round-2 wire shapes and the HMAC token / signature
  envelope. <https://github.com/bitcoin/bips/blob/master/bip-0129.mediawiki>
- **BIP-380** — *Output Script Descriptors General Operation*.
  Defines the descriptor checksum the toolkit re-computes during
  canonicalization. <https://github.com/bitcoin/bips/blob/master/bip-0380.mediawiki>
- **BIP-389** — *Multipath descriptor expressions*. Defines the
  `<0;1>/*` multipath shape Bitcoin Core 25+ emits by default in
  `listdescriptors` output. <https://github.com/bitcoin/bips/blob/master/bip-0389.mediawiki>
- **rust-miniscript** — the library that backs the toolkit's
  descriptor parsing, canonicalization, and re-rendering paths.
  <https://github.com/rust-bitcoin/rust-miniscript>

For the toolkit-internal wire formats (`ms1`, `mk1`, `md1`), see
the [BCH codex32 primer](#bch-codex32-primer) and the
[descriptors primer](#descriptors-primer).
