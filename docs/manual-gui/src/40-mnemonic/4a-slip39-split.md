# `mnemonic slip39-split` {#mnemonic-slip39-split}

Trezor-compatible SLIP-0039 K-of-N threshold share splitter.
Splits a master secret (BIP-39 phrase or hex entropy) into a
two-layer K-of-N share scheme: G groups, each with M_i members
of which T_i must combine to reconstruct the group share, and the
group layer has its own threshold K-of-G. Any K group-shares
combine to recover the master.

This is the GUI's most flexible share-splitting subcommand:
unlike `seed-xor` (N-of-N, no fault tolerance), SLIP-39 supports
arbitrary K-of-N with multi-group hierarchy. Bit-identical to
the `python-shamir-mnemonic` reference at `17fcce14` per
v0.13.0's cross-implementation smoke test.

:::danger
The master secret AND every emitted share are secret-class
material. The §14 Defense 2 cold-node operational warning
applies. The output panel renders all shares on stdout — every
share that appears on screen is exposed to the same
screen-observation threats as the master.
:::

## Outline {#mnemonic-slip39-split-outline}

- [`--from`](#mnemonic-slip39-split-from) — master secret (required; `phrase=<value-or->` or `entropy=<hex-or->`)
- [`--passphrase`](#mnemonic-slip39-split-passphrase) — SLIP-39 passphrase (XOR with `--passphrase-stdin`; **NOT** the BIP-39 passphrase)
- [`--passphrase-stdin`](#mnemonic-slip39-split-passphrase-stdin) — read `--passphrase` from stdin
- [`--group-threshold`](#mnemonic-slip39-split-group-threshold) — K of the group layer (required, 1..16)
- [`--group`](#mnemonic-slip39-split-group) — per-group `<member_count>,<member_threshold>` (required, repeating)
- [`--iteration-exponent`](#mnemonic-slip39-split-iteration-exponent) — Iteration exponent E (default 0; G9 advisory at E ≥ 5)
- [`--language`](#mnemonic-slip39-split-language) — BIP-39 wordlist (input-side; ignored for `entropy=`)
- [`--json-out`](#mnemonic-slip39-split-json-out) — write JSON envelope to PATH (side-effect)

## `--from` {#mnemonic-slip39-split-from}

The master secret. NodeValueComposite with two valid nodes:
`phrase` (BIP-39 mnemonic; 12 / 15 / 18 / 21 / 24 words) or
`entropy` (raw hex bytes; 16 / 20 / 24 / 28 / 32 bytes).
Required. Schema-`secret: false` but value-dependent (both nodes
are secret-class).

Suffix `=-` reads the value from stdin. Single-stdin-per-
invocation: at most one stdin consumer across `--from`,
`--passphrase-stdin`, and any future stdin-form input — refused
with `slip39: at most one stdin consumer per invocation (across --share, --from, and --passphrase-stdin)`.

### Outline {#mnemonic-slip39-split-from-outline}

- [`phrase`](#mnemonic-slip39-split-from-phrase)
- [`entropy`](#mnemonic-slip39-split-from-entropy)

### `phrase` {#mnemonic-slip39-split-from-phrase}

A BIP-39 mnemonic phrase (12 / 15 / 18 / 21 / 24 words). The
`--language` flag picks the wordlist (default `english`).
Refused with `slip39 split: input phrase must be 12/15/18/21/24
words; got <N>` for off-spec lengths.

### `entropy` {#mnemonic-slip39-split-from-entropy}

Raw hex entropy bytes (16 / 20 / 24 / 28 / 32 bytes; 32-64 hex
chars). The `--language` flag is ignored under entropy mode (the
conditional-visibility engine hides it). Refused with `slip39
split: entropy hex must decode to 16/20/24/28/32 bytes; got <N>
bytes` for off-spec lengths.

## `--passphrase` {#mnemonic-slip39-split-passphrase}

The **SLIP-39 passphrase** (NOT the BIP-39 passphrase — these
are mechanically distinct cryptographic channels even though the
flag name is shared between subcommands). Schema-`secret: true`.
XOR with `--passphrase-stdin` (the conditional-visibility engine
disables one when the other has a value).

The same SLIP-39 passphrase MUST be supplied at combine time;
without it, the reconstruction recovers a *different* master
(SLIP-39's plausible-deniability feature).

## `--passphrase-stdin` {#mnemonic-slip39-split-passphrase-stdin}

Boolean. Read SLIP-39 passphrase from stdin (raw, NULL-byte
preserving). Schema-`secret: true`. XOR with `--passphrase`.
Single-stdin-per-invocation: mutually exclusive with `--from
<node>=-`.

## `--group-threshold` {#mnemonic-slip39-split-group-threshold}

K of the group layer. Number widget; range 1..16. Required. For
K-of-G, set this to K. Constraint: `1 ≤ K ≤ G` where G = the
`--group` occurrence count.

## `--group` {#mnemonic-slip39-split-group}

Per-group `<member_count>,<member_threshold>` specification.
Repeating; one occurrence per group (so G groups total → G
`--group` rows). Each row's grammar: a comma-separated pair of
positive integers, e.g. `2,2` (2 members, 2-of-2 threshold) or
`5,3` (5 members, 3-of-5 threshold).

Group constraints (per SLIP-39 spec): `member_count ≤ 16`;
`member_threshold ≤ member_count`; the degenerate `1,1` group
shape is refused with the byte-exact `slip39 split: 1-of-1 group offers no recovery benefit; use --group N,T with N >= 2 (toolkit policy); got group <g_idx>=1,1`
(per `crates/mnemonic-toolkit/src/cmd/slip39.rs:320-322` +
mapped variant at `:654-656`).

The GUI renders this as a multi-row text-field repeating widget.
Each row's value is the `M,T` literal string.

## `--iteration-exponent` {#mnemonic-slip39-split-iteration-exponent}

PBKDF2 iteration exponent E. Number widget; range 0..15
(library-enforced). Default 0. Each unit increase doubles the
key-stretching iteration count: E=0 is the SLIP-39 reference
default; E=1 doubles; E=15 is 2^15 = 32768× the reference rate.

**Advisory at E ≥ 5** per the toolkit's G9 stderr advisory (see
the byte-exact text in the Advisories table below). Per source
(`crates/mnemonic-toolkit/src/cmd/slip39.rs:480-484`), E=5 yields
sub-second to multi-second split+combine performance; **E ≥ 10
may exceed 30s on weak hardware**. The SLIP-0039 spec gives no
recommended values; Trezor's reference uses E=1 (20000 iters) as
default. Use higher E values only for adversarial threat models
that justify the slower combine path.

## `--language` {#mnemonic-slip39-split-language}

BIP-39 wordlist for parsing `--from phrase=<value>`. Default
`english`. **Ignored under `--from entropy=<hex>` mode** (the
conditional-visibility engine hides this flag when `--from` is
`entropy`). Same 10 values as
[`mnemonic bundle --language`](#mnemonic-bundle-language).

Note: the SLIP-39 share output is a SLIP-39 mnemonic (1024-word
SLIP-39 wordlist), not a BIP-39 mnemonic — `--language` selects
only the **input-phrase** parser, not the output encoding.

### Outline {#mnemonic-slip39-split-language-outline}

- [`english`](#mnemonic-slip39-split-language-english)
- [`simplifiedchinese`](#mnemonic-slip39-split-language-simplifiedchinese)
- [`traditionalchinese`](#mnemonic-slip39-split-language-traditionalchinese)
- [`czech`](#mnemonic-slip39-split-language-czech)
- [`french`](#mnemonic-slip39-split-language-french)
- [`italian`](#mnemonic-slip39-split-language-italian)
- [`japanese`](#mnemonic-slip39-split-language-japanese)
- [`korean`](#mnemonic-slip39-split-language-korean)
- [`portuguese`](#mnemonic-slip39-split-language-portuguese)
- [`spanish`](#mnemonic-slip39-split-language-spanish)

### `english` {#mnemonic-slip39-split-language-english}

See [`mnemonic bundle --language english`](#mnemonic-bundle-language-english).

### `simplifiedchinese` {#mnemonic-slip39-split-language-simplifiedchinese}

See [`mnemonic bundle --language simplifiedchinese`](#mnemonic-bundle-language-simplifiedchinese).

### `traditionalchinese` {#mnemonic-slip39-split-language-traditionalchinese}

See [`mnemonic bundle --language traditionalchinese`](#mnemonic-bundle-language-traditionalchinese).

### `czech` {#mnemonic-slip39-split-language-czech}

See [`mnemonic bundle --language czech`](#mnemonic-bundle-language-czech).

### `french` {#mnemonic-slip39-split-language-french}

See [`mnemonic bundle --language french`](#mnemonic-bundle-language-french).

### `italian` {#mnemonic-slip39-split-language-italian}

See [`mnemonic bundle --language italian`](#mnemonic-bundle-language-italian).

### `japanese` {#mnemonic-slip39-split-language-japanese}

See [`mnemonic bundle --language japanese`](#mnemonic-bundle-language-japanese).

### `korean` {#mnemonic-slip39-split-language-korean}

See [`mnemonic bundle --language korean`](#mnemonic-bundle-language-korean).

### `portuguese` {#mnemonic-slip39-split-language-portuguese}

See [`mnemonic bundle --language portuguese`](#mnemonic-bundle-language-portuguese).

### `spanish` {#mnemonic-slip39-split-language-spanish}

See [`mnemonic bundle --language spanish`](#mnemonic-bundle-language-spanish).

## `--json-out` {#mnemonic-slip39-split-json-out}

Optional. Writes a versioned JSON envelope to PATH in addition to
plain shares on stdout. Schema includes `schema_version`,
`group_threshold`, `groups[]` (with per-group share arrays),
`iteration_exponent`, and metadata fields.

World-readable-path advisory on Unix systems with default umask.

## Worked example — 2-of-3 single-group split

1. **mnemonic** tab; pick **SLIP-39 Split (K-of-N share splitter)**.
2. `--from`: pick `phrase`; paste the canonical master.
3. `--group-threshold`: `1` (single-group setup → group threshold = 1).
4. `--group`: add one row with value `3,2` (3 members, 2-of-3
   threshold).
5. Leave `--passphrase` empty (default empty SLIP-39 passphrase).
6. Leave `--iteration-exponent` at default 0.
7. Click **Run**. The run-confirm modal appears (`--from phrase=`
   is secret-class). Click **Run** in the modal.

The output panel renders 3 SLIP-39 share mnemonics on stdout, one
per line. Each share is a 20-word SLIP-39 mnemonic (using the
SLIP-39 wordlist, NOT the BIP-39 wordlist). Any 2 of the 3 shares
combined via `mnemonic slip39-combine` reconstruct the master.

## Refusals

| Trigger | Refusal |
|---|---|
| `--from <node>=` other than `phrase=` or `entropy=` | `slip39 split: --from only accepts phrase=<value-or-> or entropy=<hex-or->; got <N>=` |
| Master phrase word count not in {12, 15, 18, 21, 24} | `slip39 split: input phrase must be 12/15/18/21/24 words; got <N>` |
| Master entropy hex bytes not in {16, 20, 24, 28, 32} | `slip39 split: entropy hex must decode to 16/20/24/28/32 bytes; got <N> bytes` |
| Multiple stdin consumers | `slip39: at most one stdin consumer per invocation (across --share, --from, and --passphrase-stdin)` |
| Degenerate `--group 1,1` | `slip39 split: 1-of-1 group offers no recovery benefit; use --group N,T with N >= 2 (toolkit policy); got group <g_idx>=1,1` (byte-exact per `cmd/slip39.rs:320-322`) |
| `--group-threshold > G` (where G = `--group` count) | SLIP-39 library refusal |
| `--passphrase` AND `--passphrase-stdin` | clap-level `conflicts_with` |

## Advisories

| Trigger | Stderr advisory |
|---|---|
| Inline `--from <node>=<value>` | `warning: secret material on argv (--from <node>=) — pipe via --from <node>=- to avoid /proc/$PID/cmdline exposure` |
| Inline `--passphrase <value>` | `warning: secret material on argv (--passphrase) — pipe via --passphrase-stdin to avoid /proc/$PID/cmdline exposure` |
| `--iteration-exponent ≥ 5` | byte-exact per `cmd/slip39.rs:480-484`: `warning: --iteration-exponent E=<E> yields <iters> × PBKDF2-HMAC-SHA-256 iterations; split + combine performance may be observably slow (sub-second to multi-second). Trezor's reference uses E=1 (20000 iters) as default; the SLIP-0039 spec gives no recommended values. E >= 10 may exceed 30s on weak hardware.` |
