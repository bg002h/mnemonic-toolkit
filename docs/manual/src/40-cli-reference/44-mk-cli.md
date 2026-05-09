# `mk` (mk-cli) reference

The standalone CLI for the mk1 format (mnemonic-key / mk-codec).
Five subcommands. Most users will use `mnemonic bundle` and
`mnemonic verify-bundle` instead; `mk` is for direct key-card
inspection, mk1-plate recovery from an air-gapped machine without
shipping the secret-material code paths of the toolkit, or when
integrating mk1 into a non-toolkit pipeline.

`mk-cli` ships in the `bg002h/mnemonic-key` repo as a separate
binary alongside the `mk-codec` library; install with
`cargo install --git https://github.com/bg002h/mnemonic-key --tag mk-cli-v0.2.0 --bin mk`.

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
| `--json` | emit a single JSON object on stdout |

At least one of `--policy-id-stub` or `--from-md1` is required (the
underlying `KeyCard.policy_id_stubs` slot must be non-empty).

### Worked example

```sh
mk encode \
  --xpub xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V \
  --origin-fingerprint 73c5da0a \
  --origin-path "m/84'/0'/0'" \
  --from-md1 md1zsxdspqqqpm6jzzqqvqz6qu79mg9p2sgfff6p2eph8wftp5uf6gqnlgzqqqnymv0
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
| `--pretty` | indent the JSON output for human readability (ignored when `--out` is set) |
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
