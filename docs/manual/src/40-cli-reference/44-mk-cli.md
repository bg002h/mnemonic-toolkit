# `mk` (mk-cli) reference

The standalone CLI for the mk1 format (mnemonic-key / mk-codec).
Eight subcommands. Most users will use `mnemonic bundle` and
`mnemonic verify-bundle` instead; `mk` is for direct key-card
inspection, mk1-plate recovery from an air-gapped machine without
shipping the secret-material code paths of the toolkit, or when
integrating mk1 into a non-toolkit pipeline.

`mk-cli` ships in the `bg002h/mnemonic-key` repo as a separate
binary alongside the `mk-codec` library; install with
`cargo install --git https://github.com/bg002h/mnemonic-key --tag mk-cli-v0.10.2 --bin mk`.

Every subcommand below accepts `--help` (`-h`) for inline help.

---

## `mk encode`

Encode an xpub plus origin metadata (master fingerprint + derivation
path) and one or more `policy_id_stub` values into one or more mk1
backup strings.

### Synopsis

```sh
mk encode --xpub <XPUB> --origin-path <PATH> [OPTIONS]
```

### Flags

| Flag | Purpose |
|---|---|
| `--xpub <XPUB>` | BIP-32 extended public key (xpub-prefixed string) |
| `--origin-fingerprint <HEX>` | 8-hex-char master fingerprint; mutually exclusive with `--privacy-preserving` |
| `--origin-path <PATH>` | derivation path (e.g., `m/84'/0'/0'`) |
| `--policy-id-stub <HEX>` | 8 hex chars (4 bytes) for one stub; repeatable |
| `--from-md1 <MD1-STRING>` | derive a stub from an md1 wallet-policy string; repeatable |
| `--privacy-preserving` | emit without master fingerprint; mutually exclusive with `--origin-fingerprint` |
| `--force-chunked` | force chunked output (reserved; codec auto-dispatches) |
| `--force-long-code` | force long-code BCH variant (reserved; codec auto-dispatches) |
| `--group-size <N>` | mstring display grouping: insert a separator every N characters in each emitted `mk1` string; `0` = unbroken (default 5). Display only — `--json` stays unbroken. Separator-stripping on intake means grouped/unbroken cards both re-ingest on `decode`/`verify`/etc. |
| `--separator <space\|hyphen\|comma>` | the grouping separator for `--group-size` (default `space`); keyword or the literal `-` / `,` or a space. |
| `--json` | emit a single JSON object on stdout |

At least one of `--policy-id-stub` or `--from-md1` is required (the
underlying `KeyCard.policy_id_stubs` slot must be non-empty).

### Worked example

```sh
mk encode \
  --xpub xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V \
  --origin-fingerprint 73c5da0a \
  --origin-path "m/84'/0'/0'" \
  --from-md1 md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np
```

### Output

Text mode emits one mk1 string per line, one per chunk. JSON mode emits

```json
{
  "schema_version": 1,
  "mk1_strings": ["mk1...", "mk1..."],
  "chunk_count": 2,
  "code_variant": "regular"
}
```

`code_variant` is `regular` or `long` per the BCH-code dispatch.

### SLIP-0132 prefix acceptance (`--xpub`)

`mk encode --xpub` (and `mk verify --xpub`) accept SLIP-0132
extended-public-key prefixes in addition to the canonical `xpub`:
mainnet `ypub`/`zpub` (single-sig) and `Ypub`/`Zpub` (BIP-48
multisig), plus their testnet counterparts `upub`/`vpub` and
`Upub`/`Vpub`. The prefix is normalized to the canonical `xpub`
(mainnet) or `tpub` (testnet) before encoding — the key material is
unchanged (same chain code, public key, depth, and parent
fingerprint); only the four version bytes are rewritten. A one-line
stderr note names the original prefix, e.g.:

```text
note: --xpub was a SLIP-0132 zpub (BIP-84 P2WPKH); normalized to canonical xpub — script type is conveyed by the origin path, not the key prefix
```

In mk1 the script type is conveyed by the card's origin path
(`m/49'/…` → P2SH-P2WPKH, `m/84'/…` → P2WPKH, `m/48'/…/1'|2'` →
BIP-48 multisig), not by the key prefix. When `--origin-path` is
supplied and the prefix's implied script type contradicts it (for
example a `zpub` with an `m/49'/…` path), `mk` refuses the input with
a `UsageError` (exit 64) and an actionable message naming the
expected path and the alternative prefix to use.

---

## `mk decode`

Reassemble + decode one or more mk1 strings into the underlying
xpub + origin + policy-id-stub fields.

### Synopsis

```sh
mk decode [OPTIONS] <MK1-STRING>...
```

Use `-` as a positional argument to read one mk1 string per line
from stdin.

### Flags

| Flag | Purpose |
|---|---|
| `--json` | emit JSON output |

### Worked example

```sh
mk decode \
  mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4 \
  mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh
```

### Output

Text mode:

```text
xpub:                xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V
origin_fingerprint:  73c5da0a
origin_path:         m/84'/0'/0'
policy_id_stubs:     deadbeef
chunks:              2 (regular)
```

`origin_fingerprint` reads `(omitted, privacy-preserving mode)` for
cards encoded under `--privacy-preserving`.

JSON mode:

```json
{
  "schema_version": 1,
  "xpub": "xpub6...",
  "origin_fingerprint": "73c5da0a",
  "origin_path": "m/84'/0'/0'",
  "policy_id_stubs": ["deadbeef"],
  "chunks": 2,
  "code_variant": "regular"
}
```

`origin_fingerprint` is `null` for privacy-preserving cards.

---

## `mk inspect`

Decode + structural commentary: the xpub-derived fingerprint, each
derivation-path component spelled out (hardened vs. normal), per-chunk
BCH-code variant. v0.2 inspect is intentionally less rich than `md
inspect` because mk-codec's bytecode-layer surface is not yet public
(deferred to a v0.3 cycle alongside the public-bytecode-API
decision).

### Synopsis

```sh
mk inspect [OPTIONS] <MK1-STRING>...
```

### Flags

| Flag | Purpose |
|---|---|
| `--json` | emit JSON output |

### Output

Text mode adds the following fields beyond `mk decode` text output:

```text
xpub_fingerprint:    73c5da0a
  component[0]:       84h (hardened)
  component[1]:       0h (hardened)
  component[2]:       0h (hardened)
  chunk[0]:           regular (BCH variant)
  chunk[1]:           regular (BCH variant)
```

JSON mode adds `xpub_fingerprint`, `origin_path_components` (array of
strings), and `chunk_variants` (array of `"regular"|"long"`).

---

## `mk repair`

BCH error-correct one or more corrupted mk1 strings. Both BCH code
variants are supported: the regular `BCH(93,80,8)` code for data-parts
of 14–93 symbols (short mk1 chunks) and the long `BCH(108,93,8)` code
for data-parts of 96–108 symbols (the xpub-bearing first chunk of
typical mk1 emissions). Both correct up to four substitution errors
per chunk (singleton bound `t=4`).

`mk repair` is the per-codec sibling of toolkit's `mnemonic repair`
(see `41-mnemonic.md` `## mnemonic repair`). The two surfaces share
the same `RepairJson` envelope schema byte-exact (cross-CLI parser
reuse — D27 of the toolkit v0.22.x follow-ups cycle plan); the only
differences are that `mk repair` operates exclusively on the `mk` HRP
(no `--ms1`/`--mk1`/`--md1` selector flag) and emits no Levenshtein-1
"did you mean" suggestion on HRP mismatch (single-HRP context).

Note that `mk decode` already performs internal BCH correction within
the same `t=4` capacity during normal decode. `mk repair` is the
explicit-fix-with-report counterpart: it surfaces which character
positions were corrected, what the original symbols were, and what
the repaired symbols are — useful for recovery of a corroded engraving
(one or two letters unreadable), salvage of a hand-copied card with a
single typo, or sanity-checking a freshly engraved card against its
source bundle before committing to steel.

### Synopsis

```sh
mk repair [OPTIONS] [MK1_STRINGS]...
```

### Flags

| Flag | Purpose |
|---|---|
| `[MK1_STRINGS]...` | one or more mk1 strings to attempt to repair; use `-` to read one string per line from stdin |
| `--json` | emit a single JSON envelope on stdout instead of the text-form report; schema byte-matches `mnemonic repair --json`'s `RepairJson` shape |
| `--help` | print help |

### Exit codes

| Code | Meaning |
|---|---|
| `0` | all strings already valid (no repair applied; input echoed to stdout unchanged) |
| `5` | at least one string corrected (`REPAIR_APPLIED`); stdout = repair report + corrected strings |
| `2` | unrepairable (per-chunk `RepairError`; e.g. too many errors, HRP mismatch) |
| `1` | I/O error or other generic failure |

The exit-5 `REPAIR_APPLIED` code is consistent across all four CLIs
(`mnemonic`, `mk`, `ms`, `md`) per D26 of the v0.22.x follow-ups
cycle, so wrapper scripts can use a uniform `exit == 5` signal.

### Worked example

```sh
# A valid mk1 chunk with one character substituted at position 17:
mk repair mk1qprsqhpqqsqzcqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4
```

Stdout (the corrected string is on the LAST line; comment lines
describe the fix):

```text
# Repair report
#   mk1 chunk 0: 1 correction at position 17: 'z' -> '3'
mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4
```

Exit code: `5`.

### JSON output

`mk repair --json` byte-matches toolkit's `RepairJson` envelope
(`kind` is `"mk1"`):

```json
{
  "schema_version": "1",
  "kind": "mk1",
  "corrected_chunks": ["mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4"],
  "repairs": [
    {
      "chunk_index": 0,
      "original_chunk": "mk1qprsqhpqqsqzcqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4",
      "corrected_chunk": "mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4",
      "corrected_positions": [{"position": 17, "was": "z", "now": "3"}]
    }
  ]
}
```

### Stdin via `-`

Pass a single `-` token in place of any positional string to read mk1
strings from stdin, one per line:

```sh
printf '%s\n%s\n' "$BAD_MK1_FIRST" "$BAD_MK1_SECOND" | mk repair -
```

This composes with shell pipelines and is the recommended path when
the corrupted strings are large or already in a file.

### Per-chunk atomic semantics

When multiple mk1 strings are supplied (typical for chunked mk1
emissions of multi-chunk xpub material), if ANY chunk fails to repair
(more than four substitution errors), the WHOLE call fails with the
offending chunk index named. Partial repair of sibling chunks is NOT
returned — this avoids surfacing a half-fixed key-card that could
mislead the user into committing it.

### Regular vs long BCH variant

Each input is auto-classified by data-part length: data-parts of
14–93 symbols use the regular `BCH(93,80,8)` code; data-parts of
96–108 symbols use the long `BCH(108,93,8)` code. The decoded code
variant is surfaced as `code` in the JSON envelope's per-repair
detail (when present in mk-codec's `DecodedString`). Mixed-variant
inputs in the same call are supported — each chunk is decoded
independently.

---

## `mk verify`

Verify mk1 strings decode cleanly (BCH check + structural validity)
and optionally cross-check against expected field values.

### Synopsis

```sh
mk verify [OPTIONS] <MK1-STRING>...
```

### Flags

| Flag | Purpose |
|---|---|
| `--xpub <EXPECTED-XPUB>` | assert decoded xpub equals this |
| `--origin-fingerprint <EXPECTED-HEX>` | assert decoded fingerprint equals this |
| `--origin-path <EXPECTED-PATH>` | assert decoded path equals this |
| `--policy-id-stub <HEX>` | assert decoded stubs match this set; repeatable, order-sensitive |
| `--from-md1 <MD1-STRING>` | derive expected stub from md1 string; repeatable, order-sensitive |
| `--json` | emit a JSON envelope on stdout |

Without any expected-* flags, `mk verify` performs BCH-checksum and
structural validation only. With expected flags, it additionally
asserts content equality on each supplied field.

`--xpub` accepts SLIP-0132 prefixes (ypub/zpub/Ypub/Zpub and the
testnet upub/vpub/Upub/Vpub) on the same terms as `mk encode` — see
[SLIP-0132 prefix acceptance](#slip-0132-prefix-acceptance---xpub)
above: the expected xpub is normalized to canonical xpub/tpub before
comparison, a stderr note names the original prefix, and a
prefix↔origin-path script-type mismatch is refused.

### Worked example

```sh
mk verify \
  --xpub xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V \
  --origin-fingerprint 73c5da0a \
  --origin-path "m/84'/0'/0'" \
  mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4 \
  mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh
```

Exits 0 with `OK: mk1 string(s) decode cleanly ...` on success, exits
4 with a `ContentMismatch` error envelope when an expected-* flag
disagrees with the decoded value.

---

## `mk vectors`

Print the SHA-pinned mk-codec v0.1 test-vector corpus as JSON.

### Synopsis

```sh
mk vectors [OPTIONS]
```

### Flags

| Flag | Purpose |
|---|---|
| `--pretty` | indent the JSON output for human readability (also applies to the per-fixture files written under `--out`) |
| `--out <DIR>` | write one `<name>.json` per fixture into `<DIR>` instead of emitting to stdout |

The corpus is `include_str!`-baked into the binary at build time, so
`cargo install`-style installs are fully self-contained: `mk vectors`
runs from any working directory without a fixture-path dependency.

### Worked example

```sh
mk vectors --out /tmp/mk-vectors
# (stderr) wrote N vector file(s) to /tmp/mk-vectors
```

---

## `mk address`

Render the receive/change addresses controlled by a card's xpub. Read-only
public derivation — no private keys, no signing.

The address type is inferred from the origin-path purpose **at canonical
single-sig account depth** (`m/44'`→`p2pkh`, `49'`→`p2sh-p2wpkh`, `84'`→`p2wpkh`,
`86'`→`p2tr`) and is overridable with `--address-type`. A card whose origin is
not at account depth requires the explicit flag (and prints a stderr advisory
that addresses are derived relative to the card's xpub). Multisig-cosigner cards
(`m/48'`/`m/87'`) are **refused** — single-key addresses would not match the
wallet; use descriptor tooling instead.

### Synopsis

```sh
mk address [OPTIONS] <MK1-STRING>...
```

Use `-` as a positional argument to read one mk1 string per line from stdin.

### Flags

| Flag | Purpose |
|---|---|
| `--address-type <TYPE>` | `p2pkh` \| `p2sh-p2wpkh` \| `p2wpkh` \| `p2tr`; defaults to the account-depth purpose heuristic |
| `--count <N>` | number of addresses per chain, starting at index 0 (default 10); conflicts with `--range` |
| `--range <A,B>` | inclusive index range `A..=B`; conflicts with `--count` |
| `--chain <WHICH>` | `receive` (default) \| `change` \| `both` |
| `--network <NET>` | `mainnet` \| `testnet` \| `signet` \| `regtest`; defaults to the xpub's version bytes and must agree with its network kind |
| `--json` | emit JSON output |

### Worked example

```sh
mk address \
  mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4 \
  mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh \
  --count 3
```

### Output

Text mode (receive chain):

```text
  0  bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
  1  bc1qnjg0jd8228aq7egyzacy8cys3knf9xvrerkf9g
  2  bc1qp59yckz4ae5c4efgw2s5wfyvrz0ala7rgvuz8z
```

With `--chain both`, rows are grouped by chain (`receive` then `change`).

JSON mode:

```json
{
  "schema_version": 1,
  "xpub": "xpub6...",
  "origin_path": "m/84'/0'/0'",
  "address_type": "p2wpkh",
  "network": "mainnet",
  "addresses": [
    { "chain": 0, "index": 0, "address": "bc1q..." },
    { "chain": 0, "index": 1, "address": "bc1q..." }
  ]
}
```

---

## `mk derive`

Derive a child xpub at a relative path from the card's xpub. An xpub can only
derive **unhardened** children (it has no private key); hardened components are
rejected. The emitted `child_xpub` is composable — pipe it back into `mk encode`.
Read-only; no signing.

### Synopsis

```sh
mk derive [OPTIONS] <MK1-STRING>...
```

Exactly one of `--path` / `--index` is required. Use `-` as a positional to read
mk1 strings from stdin.

### Flags

| Flag | Purpose |
|---|---|
| `--path <REL>` | relative derivation path, unhardened only (e.g. `m/0/5`) |
| `--index <N>` | single external-chain index — sugar for `--path m/0/<N>` |
| `--json` | emit JSON output |

### Worked example

```sh
mk derive \
  mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4 \
  mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh \
  --path m/0/5
```

### Output

Text mode:

```text
parent_xpub:          xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V
parent_origin_path:   m/84'/0'/0'
relative_path:        m/0/5
child_xpub:           xpub6FrCS2gWHvogpRtNtG8qfs9QjHU9qVPL418V59XRrBQaJ5byrvkSYSwdbMsiBCeRM8U4tiDSHu13W7jNRSZs9bnmW7gDbjgB1NHY3aoNx5X
child_fingerprint:    98442048
depth:                5
network:              mainnet
```

JSON mode emits the same fields as a `{ "schema_version": 1, … }` object.

---

## Error envelope

In `--json` mode, errors are emitted as a structured envelope:

```json
{
  "schema_version": 1,
  "error": {
    "kind": "<error-kind-name>",
    "message": "<human-readable>",
    "exit_code": 2,
    "details": { }
  }
}
```

Error `kind` values map 1:1 to mk-codec's `Error` enum variants
(e.g., `InvalidHrp`, `MixedCase`, `BchUncorrectable`,
`ChunkSetIdMismatch`, `PathTooDeep`) plus mk-cli-only kinds:
`UsageError`, `ContentMismatch`, `IoError`, `MdCodec` (`--from-md1`
parse failures), and `FutureFormat` (reserved for v0.3+ when mk-codec
distinguishes future-version strings from currently-unsupported ones).
The `details` field is kind-specific (e.g., `ContentMismatch` carries
`{field, expected, actual}`).

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Success. |
| 2 | mk1 format violation; codec rejected the input. Maps to `Error::*` kinds except `UnsupportedVersion`. |
| 3 | FutureFormat — string is well-formed but its declared version is newer than this tool. Maps to `Error::UnsupportedVersion`. |
| 4 | Verify content mismatch (only `mk verify` with expected-* flags emits this). |
| 64 | CLI usage error per clap convention (unrecognized flag, missing required argument, etc.). |
