# `mk` (mk-cli) reference

The standalone CLI for the mk1 format (mnemonic-key / mk-codec).
Ten subcommands. Most users will use `mnemonic bundle` and
`mnemonic verify-bundle` instead; `mk` is for direct key-card
inspection, mk1-plate recovery from an air-gapped machine without
shipping the secret-material code paths of the toolkit, or when
integrating mk1 into a non-toolkit pipeline.

`mk-cli` ships in the `bg002h/mnemonic-key` repo as a separate
binary alongside the `mk-codec` library; install with
`cargo install --locked --git https://github.com/bg002h/mnemonic-key --tag mk-cli-v0.17.0 mk-cli`.

Every subcommand below accepts `--help` (`-h`) for inline help.

---

## `mk encode`

Encode an xpub plus origin metadata (master fingerprint + derivation
path) and one or more `policy_id_stub` values into one or more mk1
backup strings.

### Synopsis

```sh
mk encode --xpub <XPUB> --origin-path <PATH> (--origin-fingerprint <HEX> | --privacy-preserving) [OPTIONS]
```

### Flags

| Flag | Purpose |
|---|---|
| `--xpub <XPUB>` | BIP-32 extended public key (xpub-prefixed string) |
| `--origin-fingerprint <HEX>` | 8-hex-char master fingerprint; mutually exclusive with `--privacy-preserving`; a single-card encode needs one of the two (exit 64 otherwise); refused with `--keys`/`--in`, whose records carry their own |
| `--origin-path <PATH>` | derivation path (e.g., `m/84'/0'/0'`) |
| `--policy-id-stub <HEX>` | 8 hex chars (4 bytes) for one stub; repeatable |
| `--from-md1 <MD1-STRING>` | derive a stub from an md1 wallet-policy string; repeatable |
| `--from-md1-set <FILE>` | repeatable; read md1 strings from FILE (what `md encode --out` writes) and bind their stubs exactly as repeated `--from-md1` does; lines that are not md1 strings (the `chunk-set-id:` header, blanks, `#` comments) are skipped; a leading byte-order mark is removed and CRLF line ends are read as LF; any other carriage return or line-break character, or a byte-order mark after the first character, is refused (exit 2) naming the line; stubs bind in flag order `--policy-id-stub`, `--from-md1`, `--from-md1-set` |
| `--keys <FILE>` | mint ONE card per key record in FILE (`-` for stdin) instead of the single card described by `--xpub`/`--origin-path`; each record is BIP-380 origin notation `[fingerprint/path]xpub` on its own line; blanks and `#` comments ignored; a leading byte-order mark is removed and CRLF line ends are read as LF; a CR-only file, any other carriage return or line-break character, or a byte-order mark after the first character, is refused (exit 2) naming the line |
| `--in <FILE>` | read the key records from FILE (`-` for stdin) — the same reader `--keys` uses; the two are mutually exclusive |
| `--out <FILE>` | write the mk1 artifact to FILE, created `0600`, instead of stdout; the file holds exactly what stdout would (grouped under `--group-size`); overwrites an existing file (operator ruling 2026-08-26) |
| `--privacy-preserving` | emit without master fingerprint; mutually exclusive with `--origin-fingerprint`; a single-card encode needs one of the two (exit 64 otherwise) |
| `--force-chunked` | force chunked output (reserved; codec auto-dispatches) |
| `--force-long-code` | force long-code BCH variant (reserved; codec auto-dispatches) |
| `--chunk-set-id <HEX>` | pin the 20-bit `chunk_set_id` (`0x` prefix optional) instead of deriving it from the payload. Chunked output only. A value over 20 bits is refused, not truncated. Rarely needed: since mk-codec 0.5.0 the derived default is already deterministic, so re-encoding a card reproduces its strings byte for byte. |
| `--group-size <N>` | group each mk1 string on stdout every N characters, for hand engraving (default `0` = unbroken). `--out` receives the same bytes; `--json` stays unbroken. Separator-stripping on intake means grouped and unbroken cards both re-ingest on `decode`/`verify`/etc. |
| `--separator <space>` | the separator for `--group-size`: `space` (keyword, the default) or the literal `" "`. Whitespace only: `hyphen` and `comma` (and the literals `-` and `,`) are refused (exit 64); a card already grouped with them still decodes |
| `--json` | emit a single JSON object on stdout |

At least one of `--policy-id-stub` or `--from-md1` is required (the
underlying `KeyCard.policy_id_stubs` slot must be non-empty).

A depth-0 xpub (`--origin-path m`, or a key record with no path) must be a valid BIP 32 master: its parent
fingerprint must be `00000000`, and a declared origin fingerprint must be the key's own. At depth 1 a declared
origin fingerprint must be the xpub's parent fingerprint. `mk encode` refuses any of these with exit 2 and mints
nothing.

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

Text mode prints each mk1 string once, on stdout: one per line, one per
chunk, unbroken by default and grouped under `--group-size`. stderr
carries only the watch-only note; with `--keys` and two or more cards
it is preceded by one `card <i> of <n>: lines <a>-<b>` line per card. JSON
mode emits a `{ "schema_version", "mk1_strings", "chunk_count",
"code_variant", "origin_fingerprint", "origin_path" }` object (the
`mk1_strings` values are elided here; the chunk-set identifier is
derived from the card's content, so the same inputs print the same
strings every run):

```json
{
  "schema_version": 1,
  "mk1_strings": ["mk1...", "mk1..."],
  "chunk_count": 2,
  "code_variant": "long",
  "origin_fingerprint": "73c5da0a",
  "origin_path": "84'/0'/0'"
}
```

`origin_fingerprint` is `null` for a `--privacy-preserving` card.

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
an `InvalidInput` error (exit 2) and an actionable message naming the
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
| `--in <FILE>` | read mk1 strings from FILE, one per line, instead of (or as well as) the positional arguments. Display separators are stripped, so a card typed back from its grouped engraving form re-ingests |
| `--json` | emit JSON output |

### Worked example

```sh
mk decode \
  mk1qpu9e2pqqsqaatd7aaeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qyle5zxl62t28hpv \
  mk1qpu9e2pp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6n0sh92dm9ygtac94unyvx
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

A card that breaks one of the key-origin rules in `mk encode` (minted before mk-codec 0.6.0) still reads: `mk decode`, `mk inspect`, `mk address`
and `mk derive` print it as read and add a `warning: this card is invalid (...)` line on stderr.

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
| `--in <FILE>` | read mk1 strings from FILE, one per line, instead of (or as well as) the positional arguments. Display separators are stripped, so a card typed back from its grouped engraving form re-ingests |
| `--json` | emit JSON output |

### Output

Text mode emits the full decode fields plus the structural commentary
(`xpub_fingerprint` — the fingerprint *of the card's xpub*, distinct
from the master `origin_fingerprint`; the spelled-out path components;
the per-chunk BCH variant; and, for a chunked card, a last
`chunk_set_id:` line giving the id stamped in its headers):

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
(see `41-mnemonic.md` `## mnemonic repair`). The two surfaces share the
`RepairJson` envelope's fields (cross-CLI parser reuse — D27 of the
toolkit v0.22.x follow-ups cycle plan). Since Cycle F, toolkit's
`RepairJson` is a strict **superset**: it carries an extra `verdict`
field (inserted after `kind`) that the NO-BUMP `mk repair` envelope does
not; a shared-field parser still reads both. Beyond that, `mk repair`
operates exclusively on the `mk` HRP (no `--ms1`/`--mk1`/`--md1`
selector flag) and emits no Levenshtein-1 "did you mean" suggestion on
HRP mismatch (single-HRP context).

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
| `--in <FILE>` | read mk1 strings from FILE, one per line, instead of (or as well as) the positional arguments. Display separators are stripped, so a card typed back from its grouped engraving form re-ingests |
| `--json` | emit a single JSON envelope on stdout instead of the text-form report; shares the `RepairJson` fields with `mnemonic repair --json` (whose toolkit envelope is a superset since Cycle F — see above) |
| `--help` | print help |

### Exit codes

| Code | Meaning |
|---|---|
| `0` | all strings already valid (no repair applied; input echoed to stdout unchanged) |
| `16` | at least one string corrected (`REPAIR_APPLIED`), and the corrected group decodes; stdout = repair report + corrected strings. If the supplied strings are an INCOMPLETE `chunk_set_id` group (e.g. a single chunk of a multi-chunk card: fewer chunks than the set holds, each index given once and inside the set), stderr additionally carries a `warning:` line saying the correction is verifiable when the rest of the set is reassembled, and telling you to run `mk decode` with every chunk — see [Set-level re-verify](#set-level-re-verify) below. A clean incomplete group is echoed at `0`. Short groups under different `chunk_set_id`s that declare one size and together cover every index of it are not incomplete: they are refused (`2`) |
| `18` | `16 + 2`: a corrected group that decodes to a card that then fails validation (`KeyCard::validate`). It is printed, with an error line; a clean card that fails validation is printed at `2`. (`mnemonic repair --mk1` does not run this check yet; see [Appendix J](#appendix-j-exit-codes).) |
| `22` | `16 + 4 + 2`: a corrected string that is not a card — a corrected single (not chunked) string that does not decode, or a corrected string whose chunk header does not parse as an mk1 chunk. It is still printed, with an error line telling you not to trust it. (A corrected single string that decodes, `20`, cannot occur in mk: a single string cannot hold a 65-byte xpub.) |
| `2` | unrepairable (per-chunk `RepairError`; e.g. too many errors, HRP mismatch); a clean string `mk decode` rejects, or a clean string whose chunk header does not parse; a COMPLETE `chunk_set_id` group whose correction fails cross-chunk reassembly (`SetReassemblyMismatch` — the per-chunk correction aliased to a different, wrong card; see [Set-level re-verify](#set-level-re-verify) below); **or** a malformed group: an index given twice, an index outside the set, more chunks than the set declares, or chunks that disagree on the set's size; **or** short groups under different `chunk_set_id`s that declare one size and together cover every index of it (a suspected miscorrection of a set id, or chunks of two cards mixed). Nothing is printed, and the reason is on stderr |
| `32` | a file or stream could not be read or written |
| `64` | the command line is wrong; nothing ran |

**The same codes as the toolkit.** `mk repair` and `mnemonic repair
--mk1` give the same code, and print the same strings, on mk-codec's
negative test vectors N6 to N21, each clean and with one substitution
(the toolkit's tests compare the two when `MK_BIN` names an `mk`
build). Bit 16 means a correction
was applied: it is **verified now** (a complete, cleanly-reassembling
`chunk_set_id` group) or **verifiable by reassembly later** (an
incomplete group, whose `warning:` line tells you to reassemble the
full card; short groups under different set ids that together cover
every index of one size are refused, `2`, instead). A corrected single (not chunked) string that does not decode
is printed at `22`; a chunked set of one follows the complete-set rule
(`2`, nothing printed). See
[Appendix J](#appendix-j-exit-codes).

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
  cleanly** — reported as repaired, exit `16`.
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
  present. `mk repair` still corrects and reports the chunk (exit `16`:
  the rest of the set verifies it when reassembled), and adds a `warning:`
  line on stderr: the correction is verifiable when the rest of the set is
  reassembled, so run `mk decode` with every chunk of the card to confirm
  it. `--json` marks such a group `"incomplete": true`.
- **The chunks are not one card's set** (an index given twice, an index
  outside the set, more chunks than the set declares, or chunks that
  disagree on its size): exit `2`, nothing printed, the reason on
  stderr. Such a group is not an incomplete set.
- **Short groups under different `chunk_set_id`s cover one set**: two
  or more incomplete groups that declare the same size and whose
  indices together cover every index of it (overlap allowed) are a
  suspected miscorrection of a set id, or chunks of two cards mixed
  (`md` refuses chunks of two cards the same way): exit `2`, nothing
  printed, clean or corrected. Groups that declare different sizes never
  combine, and a complete card in the same batch never counts: a whole
  card beside a lone chunk of another card is a card plus an incomplete
  set.

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
# below is verifiable only by reassembly, per Set-level re-verify above.
mk repair mk1qpu9e2pqqsqaatd7aqeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qyle5zxl62t28hpv
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

Exit code: `16`.

### JSON output

`mk repair --json` emits the shared `RepairJson` fields (`kind` is
`"mk1"`); toolkit's `mnemonic repair --mk1 --json` is a superset of this
since Cycle F (it adds a `verdict` field after `kind` that this NO-BUMP
envelope omits — shared-field-parser compatible). Two fields follow `kind`
and are absent unless they apply: `"not_a_card": {"kind": "<error
kind>", "message": "<reason>"}` at exit `22`, and `"incomplete": true`
for an incomplete group (exit `0` or `16`). The incomplete-set `warning:`
line (same wording as the text mode above) is still emitted on
stderr — `--json` only changes stdout:

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
| `--policy-id-stub <HEX>` | assert decoded stubs match this set; repeatable, in any order (a stderr note says when the card carries them in another order) |
| `--from-md1 <MD1-STRING>` | derive expected stub from md1 string; repeatable, in any order (a stderr note says when the card carries them in another order) |
| `--in <FILE>` | read mk1 strings from FILE, one per line, instead of (or as well as) the positional arguments. Display separators are stripped, so a card typed back from its grouped engraving form re-ingests |
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
  mk1qpu9e2pqqsqaatd7aaeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qyle5zxl62t28hpv \
  mk1qpu9e2pp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6n0sh92dm9ygtac94unyvx
```

Exits 0 with `OK: mk1 string(s) decode cleanly ...` on success. Exits 2 when the card breaks one of the key-origin rules in `mk encode` (kind
`MasterParentFingerprintNonzero`, `MasterFingerprintMismatch` or `OriginParentFingerprintMismatch`), before any expected-\* comparison. Exits 1 with a
`ContentMismatch` error envelope when an expected-* flag disagrees with the decoded value. A card that
decoded only after BCH correction adds 16 (17 for a mismatch on a corrected card), and the OK line reads
`decode after BCH correction` instead of `decode cleanly`, and
the `--json` report carries an additive `"corrected": true|false` field.
For a chunked card the `--json` envelope also carries `chunk_set_id`: `declared`, `derived`,
`matches`, and `bundle_stamped` (true when the stamped id is the bundle id `mnemonic bundle` stamps).

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
that addresses are derived relative to the card's xpub). A card that holds one fixed key (depth 0 with an all-zero chain code — how the toolkit writes a WIF key) is **refused** before anything else is checked (exit 2): it has no child keys that any wallet watches. The refusal prints the `mnemonic convert` command that renders the key's own address; a key in a multisig wallet takes its addresses from the md1 policy card with `md address`. Multisig-cosigner cards
(`m/48'`/`m/87'`) are **refused** — single-key addresses would not match the
wallet; derive its addresses with `md address <MD1>... --from-mk1-file FILE` (the md1 policy card, and every cosigner's mk1 card in FILE; for testnet cards add `--network testnet`, `--network signet` or `--network regtest`) instead.

### Synopsis

```sh
mk address [OPTIONS] <MK1-STRING>...
```

Use `-` as a positional argument to read one mk1 string per line from stdin.

### Flags

| Flag | Purpose |
|---|---|
| `--address-type <TYPE>` | `p2pkh` \| `p2sh-p2wpkh` \| `p2wpkh` \| `p2tr`; defaults to the account-depth purpose heuristic |
| `--count <N>` | number of addresses per chain, starting at index 0 (default 10, max 1000000); conflicts with `--range` |
| `--range <A,B>` | inclusive index range `A..=B`, at most 1000000 indices; conflicts with `--count` |
| `--chain <WHICH>` | `receive` (default) \| `change` \| `both` |
| `--network <NET>` | `mainnet` \| `testnet` \| `signet` \| `regtest`; defaults to the xpub's version bytes and must agree with its network kind |
| `--in <FILE>` | read mk1 strings from FILE, one per line, instead of (or as well as) the positional arguments. Display separators are stripped, so a card typed back from its grouped engraving form re-ingests |
| `--json` | emit JSON output |

### Worked example

```sh
mk address \
  mk1qpu9e2pqqsqaatd7aaeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qyle5zxl62t28hpv \
  mk1qpu9e2pp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6n0sh92dm9ygtac94unyvx \
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
rejected. A `--path` with no steps (`''`, `m`, `m/`) is rejected (the card's own key is `mk decode`'s output), and a fixed-key card (see `mk address`) is refused. The emitted `child_xpub` is composable — pipe it back into `mk encode`.
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
| `--path <REL>` | relative derivation path, unhardened only, at least one step (e.g. `m/0/5`) |
| `--index <N>` | single external-chain index — sugar for `--path m/0/<N>` |
| `--in <FILE>` | read mk1 strings from FILE, one per line, instead of (or as well as) the positional arguments. Display separators are stripped, so a card typed back from its grouped engraving form re-ingests |
| `--json` | emit JSON output |

### Worked example

```sh
mk derive \
  mk1qpu9e2pqqsqaatd7aaeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qyle5zxl62t28hpv \
  mk1qpu9e2pp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6n0sh92dm9ygtac94unyvx \
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
`UsageError`, `InvalidInput`, `ContentMismatch`, `Mismatch`, `Refused`,
`IoError`, `Internal`, `MdCodec` (`--from-md1` parse failures),
`SetReassemblyMismatch`, and `FutureFormat` (reserved for v0.3+ when
mk-codec distinguishes future-version strings from currently-unsupported
ones). The envelope's `exit_code` is the process's exit status.
The `details` field is kind-specific (e.g., `ContentMismatch` carries
`{field, expected, actual}`).

## Exit codes

`mk` uses the constellation codes of [Appendix J](#appendix-j-exit-codes);
codes 1 to 63 are sums of the bits below.

| Code | Meaning |
|---|---|
| 0 | Success. |
| 1 | A check said no: `mk verify` content mismatch with an expected-* flag, a key that is not a cosigner of the policy, an origin that contradicts its policy, a `--keys` batch membership failure. |
| 2 | An input is invalid: an mk1 format violation (every `Error::*` kind, including `UnsupportedVersion` for a newer format) in any verb, an md1 that does not decode, a bad stub, fingerprint, origin path or SLIP-132 xpub, a bad key record, an empty named input (an empty `--in` file or empty stdin), input that is not UTF-8, a fixed-key card given to `derive` or `address`, a multisig card given to `address`, a card that fails validation (`decode`, `inspect`, `address` and `derive` still print it, with a warning), or a repair that fails reassembly or whose chunks are not one card's set. A bad `--xpub` is not echoed: the error names the flag and the version bytes, and says when they belong to a private key. |
| 8 | Refused: an output file that is also an input, or a card of the other network given to `address`. |
| 16 | A BCH correction was applied and the corrected value used: `mk repair`, and also `decode`, `verify`, `inspect`, `address` and `derive` on a damaged card (re-engrave it). |
| 18 | `16 + 2`: a corrected card that is still invalid for the verb, for example one that fails validation (`mk repair` prints it with an error line). |
| 22 | `16 + 4 + 2`: `mk repair` corrected a string that is not a card — a single (not chunked) string that does not decode, or a chunk whose header does not parse; it is printed. |
| 32 | A file, stdin or stdout failed: an unreadable `--in FILE` or `--keys FILE`, an unwritable `--out`. |
| 64 | The command line is wrong (unrecognized flag, missing required argument, a bad setting, no input named at all: no positional, no `--in`, no `-`); nothing ran. |
| 101 | An internal error. |

**Per-verb exit status.** Every `mk` verb's `-h` and `--help` ends with one `Exit status: ...` line
that names the codes that verb can give (for example `mk repair --help`); the root
`--help` ends with the shared legend of [Appendix J](#appendix-j-exit-codes).

A reader that closes the pipe early (`mk address … | head -3`) only stops the output: the exit code is the verb's own. Any other write error on stdout (a full disk, `> /dev/full`), from a verb or `--help`, prints `mk: cannot write the output: <error>` on stderr and adds 32 to the verb's code (0 becomes 32, 16 becomes 48, a `verify` mismatch that prints its report, 1, becomes 33); a verb that fails before writing anything keeps its code (an invalid input stays 2); 64 and 101 are kept.

## `mk gui-schema`

Emit a machine-readable JSON description of `mk`'s flag surface, which the
`mnemonic-gui` overlay uses to build and drift-check its per-subcommand
forms. It takes no flags; the output is for programs, not for reading.

```sh
mk gui-schema | jq '.subcommands[].name'
```

---

## `mk gen-man` (v0.11.0) {#mk-gen-man}

Emit roff man pages for the whole `mk` CLI tree into a directory. The pages are
generated directly from the compiled clap `Command` tree (`clap_mangen`), so
they are binary-faithful by construction. One page per (nested) subcommand is
written, hyphen-joined: `mk.1` (root), `mk-encode.1`, `mk-decode.1`, and so on.
`scripts/install.sh` invokes this after installing the binary to drop pages into the
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
| output-dir create / write I/O error | `32` |
