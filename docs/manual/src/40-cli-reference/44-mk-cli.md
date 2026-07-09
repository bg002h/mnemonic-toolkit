# `mk` (mk-cli) reference

The standalone CLI for the mk1 format (mnemonic-key / mk-codec).
Eight subcommands. Most users will use `mnemonic bundle` and
`mnemonic verify-bundle` instead; `mk` is for direct key-card
inspection, mk1-plate recovery from an air-gapped machine without
shipping the secret-material code paths of the toolkit, or when
integrating mk1 into a non-toolkit pipeline.

`mk-cli` ships in the `bg002h/mnemonic-key` repo as a separate
binary alongside the `mk-codec` library; install with
`cargo install --git https://github.com/bg002h/mnemonic-key --tag mk-cli-v0.12.0 --bin mk`.

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
  --policy-id-stub deadbeef
```

(`--from-md1 <MD1-STRING>` is the alternative to `--policy-id-stub`: it
derives the stub from a wallet-policy md1 string instead of taking the
4-byte stub literally.)

### Output

Text mode emits one mk1 string per line, one per chunk. JSON mode emits
a `{ "schema_version", "mk1_strings", "chunk_count", "code_variant" }`
object (the per-call `mk1_strings` values are elided here — `mk encode`
randomizes the chunk-set identifier, so they differ run to run):

```json
{
  "schema_version": 1,
  "mk1_strings": ["mk1...", "mk1..."],
  "chunk_count": 2,
  "code_variant": "long"
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
  mk1qp0wrvpqqsqaatd7aaeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8q82lnyqx86wgywhq \
  mk1qp0wrvpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6n0sh92dmhwpm2qxcz3xrx
```

### Output

Text mode:

```{.text include="44-mk-decode-text.out"}
```

`origin_fingerprint` reads `(omitted, privacy-preserving mode)` for
cards encoded under `--privacy-preserving`.

JSON mode:

```{.json include="44-mk-decode-json.out"}
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

Text mode emits the full decode fields plus the structural commentary
(`xpub_fingerprint` — the fingerprint *of the card's xpub*, distinct
from the master `origin_fingerprint`; the spelled-out path components;
and the per-chunk BCH variant):

```{.text include="44-mk-inspect-text.out"}
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
| `5` | at least one string corrected (`REPAIR_APPLIED`); stdout = repair report + corrected strings. If the supplied strings are an INCOMPLETE `chunk_set_id` group (e.g. a single chunk of a multi-chunk card), stderr additionally carries an `UNVERIFIED` advisory — see [Set-level re-verify](#set-level-re-verify) below |
| `2` | unrepairable (per-chunk `RepairError`; e.g. too many errors, HRP mismatch) **or** a COMPLETE `chunk_set_id` group whose correction fails cross-chunk reassembly (`SetReassemblyMismatch` — the per-chunk correction aliased to a different, wrong card; see [Set-level re-verify](#set-level-re-verify) below) |
| `1` | I/O error or other generic failure |

**Asymmetry vs. the toolkit (read before relying on `exit == 5`
uniformly):** this standalone `mk repair` binary reports exit `5` for
ANY correction — including an INCOMPLETE `chunk_set_id` group, where it
adds the `UNVERIFIED` advisory above but does NOT change the exit code.
`mnemonic repair --mk1` (the toolkit's own copy of this same engine)
instead **demotes** that identical incomplete-group case to exit `4`
`VERIFY-ME` (see `41-mnemonic.md`'s [mk1 set-level
re-verify](#mnemonic-repair-mk1-set-level-reverify)) — the two surfaces
diverge on this one case; both share every other exit code. The
principled rule across all four CLIs (D26 of the v0.22.x follow-ups
cycle, refined by Cycle E + Cycle F): exit-5 `REPAIR_APPLIED` means a
correction is **verified now** (a complete, cleanly-reassembling
`chunk_set_id` group) **or verifiable-by-reassembly later** (this
binary's own incomplete-group case, above) — never "an oracle verified
it" standing alone. Exit-4 `VERIFY-ME` means a substitution correction
that spent the checksum's error-detection budget and has **no
self-oracle** — always true for `ms1` (see `43-ms.md`), and true for an
incomplete `mk1` group specifically in `mnemonic repair --mk1`.

### Set-level re-verify {#set-level-re-verify}

BCH correction is a best-fit operation: it returns the codeword within
Hamming distance 4 of the corrupted input, and for a genuine ≤4-error
corruption that is provably the originally-encoded chunk. Beyond that
bound (5 or more substitution errors in one chunk), a correction can
still *succeed* — the corrected chunk passes its own BCH check — while
actually **aliasing to a different, valid-but-wrong codeword** rather
than recovering the original. The corrected chunk alone cannot tell the
two cases apart; only reassembling the FULL card (checking the
cross-chunk hash that ties every chunk of a card together) can.

An empirically measured rate for this failure mode — a 5-substitution
corruption of the regular-code chunk aliasing to a different, valid
codeword — is on the order of **7.2 × 10⁻⁵** (a 95% Clopper-Pearson
upper confidence bound, measured by a seeded, reproducible harness in
the toolkit test suite; not a theoretical estimate). Small, but not
zero, and disproportionate in consequence: an aliased "correction"
looks exactly like a confident fix.

`mk repair` re-verifies every full `chunk_set_id` group it can before
reporting a confident fix:

- **A COMPLETE group (every chunk of the card supplied) reassembles
  cleanly** — reported as repaired, exit `5`, as before.
- **A COMPLETE group's correction FAILS reassembly** — the per-chunk
  correction has aliased to a different card. `mk repair` REJECTS it
  outright: exit `2` (`SetReassemblyMismatch`), no corrected string is
  printed, no partial output. This is a breaking exit-code change from
  pre-Cycle-E behavior, where such a miscorrection could be reported as
  a confident fix.
- **An INCOMPLETE group is supplied** (a single chunk of a multi-chunk
  card, or otherwise fewer chunks than the card's `total_chunks` — the
  documented per-plate recovery workflow in the worked example below) —
  reassembly cannot be checked, because the other chunks aren't
  present. `mk repair` still corrects and reports the chunk (exit `5`,
  unchanged), but adds a loud `UNVERIFIED` advisory on stderr instructing
  the operator to reassemble the full card before trusting the
  correction.

BIP-93 itself recommends confirming a corrected codex32 string before
relying on it; this advisory operationalizes that recommendation for
the one case `mk repair` cannot verify on its own. When in doubt,
decode the full card (`mk decode` with every chunk of the set) — full
reassembly's cross-chunk hash check is the authoritative confirmation
that a correction recovered the true original card, not merely *a*
valid one.

### Worked example

```sh
# A valid mk1 chunk with one character substituted at position 17.
# This is ONE chunk of a 2-chunk mk1 card — the documented single-plate
# recovery workflow (only this plate is legible; the sibling chunk is
# not being supplied) — so the group is INCOMPLETE and the correction
# below is an UNVERIFIED candidate per Set-level re-verify above.
mk repair mk1qp0wrvpqqsqaatd7aqeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8q82lnyqx86wgywhq
```

Stdout (the corrected string is on the LAST line; comment lines
describe the fix):

```{.text include="44-mk-repair-text.out"}
```

Stderr (the `warning:` line is the [Set-level re-verify](#set-level-re-verify)
advisory — this single chunk cannot be reassembled against its sibling,
so reassemble the full card with `mk decode` before trusting the fix):

```{.text include="44-mk-repair-text.err"}
```

Exit code: `5`.

### JSON output

`mk repair --json` byte-matches toolkit's `RepairJson` envelope
(`kind` is `"mk1"`). The `UNVERIFIED` advisory (same wording as the text
mode above) is still emitted on stderr — `--json` only changes stdout:

```{.json include="44-mk-repair-json.out"}
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
  mk1qp0wrvpqqsqaatd7aaeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8q82lnyqx86wgywhq \
  mk1qp0wrvpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6n0sh92dmhwpm2qxcz3xrx
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
  mk1qp0wrvpqqsqaatd7aaeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8q82lnyqx86wgywhq \
  mk1qp0wrvpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6n0sh92dmhwpm2qxcz3xrx \
  --count 3
```

### Output

Text mode (receive chain):

```{.text include="44-mk-address-text.out"}
```

With `--chain both`, rows are grouped by chain (`receive` then `change`).

JSON mode (here with `--count 2`):

```{.json include="44-mk-address-json.out"}
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
  mk1qp0wrvpqqsqaatd7aaeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8q82lnyqx86wgywhq \
  mk1qp0wrvpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6n0sh92dmhwpm2qxcz3xrx \
  --path m/0/5
```

### Output

Text mode:

```{.text include="44-mk-derive-text.out"}
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

## `mk gen-man` (v0.11.0) {#mk-gen-man}

Emit roff man pages for the whole `mk` CLI tree into a directory. The pages are
generated directly from the compiled clap `Command` tree (`clap_mangen`), so
they are binary-faithful by construction. One page per (nested) subcommand is
written, hyphen-joined: `mk.1` (root), `mk-encode.1`, `mk-decode.1`, and so on.
`scripts/install.sh` invokes this after `cargo install` to drop pages into the
user manpath (no sudo).

### Synopsis

```sh
mk gen-man --out <DIR>
```

### Flags

| Flag | Meaning |
|---|---|
| `--out <DIR>` (required) | Directory to write the `*.1` man pages into (created if absent). |
| `--help` | Print help and exit. |

### Exit codes

| Condition | Exit |
|---|---|
| success | `0` |
| output-dir create / write I/O error | `1` |
