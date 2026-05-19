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
toolkit's `bundle`/`verify-bundle`/`convert` triad. v0.26.0 supports
two on the import side:

- **BSMS Round-2 (BIP-129).** Plaintext multi-line shape emitted
  by Bitcoin coordinators (Coldcard, Specter, Bitcoin Core's
  miniscript-aware wallets, etc.) as the second round of the
  BIP-129 multisig setup protocol. The toolkit accepts both the
  full 6-line shape and a lenient 2-line excerpt.
- **Bitcoin Core `listdescriptors` JSON.** The RPC output of
  `bitcoin-cli listdescriptors`, carrying one or more descriptor
  entries plus wallet-state metadata (`active`, `internal`,
  `range`, `timestamp`).

Both formats are *watch-only by construction*: they carry xpubs
and origin paths but not secret material. (Bitcoin Core's
`listdescriptors true` variant CAN carry `xprv`; the toolkit
**refuses** that input — see [§3 below](#bitcoin-core-listdescriptors).)

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

**6-line shape** (full BIP-129 Round-2):

```text
BSMS 1.0
<TOKEN>
<descriptor>#<checksum>
<DERIVATION_PATH>
<FIRST_ADDRESS>
<SIGNATURE>
```

The toolkit's 6-line shape is a *lenient toolkit-specific
consolidation* of BIP-129's plaintext (4 lines: version, descriptor,
derivation_path, first_address) plus the BIP-129 *envelope*-side
HMAC token + signature flattened into the same blob. An importer
that doesn't decrypt the BIP-129 encryption envelope can still
preserve and audit those fields. The 4-line strict shape is also
accepted as a degenerate case.

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

v0.26.0 ships exactly two source formats. Recognized in the broader
ecosystem but NOT yet importable (each tracked by a queued FOLLOWUP
for v0.27+): Sparrow JSON (`wallet-import-sparrow`), Specter JSON
non-BSMS path (`wallet-import-specter`), Electrum wallet file
(`wallet-import-electrum`), Coldcard generic JSON
(`wallet-import-coldcard`) + multisig text
(`wallet-import-coldcard-multisig`), Blockstream Jade
`register_multisig` (`wallet-import-jade`), BSMS Round-1
token-only (`wallet-import-bsms-round-1`), and BSMS encrypted
envelopes per BIP-129 §5 (`wallet-import-bsms-encrypted`).

Workaround in v0.26.0: for Sparrow / Specter / Electrum / Coldcard
/ Jade, re-emit the watch-only descriptor via the source wallet's
Bitcoin-Core-compatible export path, then ingest the resulting
`listdescriptors` JSON. For encrypted BSMS envelopes, decrypt
out-of-band with the coordinator's key, then feed the plaintext
into `import-wallet`. BSMS Round-1 has no workaround until Round-1
parsing ships — the descriptor is not yet assembled at Round-1.

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
