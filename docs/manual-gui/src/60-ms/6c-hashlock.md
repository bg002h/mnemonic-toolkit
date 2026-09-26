# `ms hashlock` {#ms-hashlock}

\index{ms hashlock}Turn a **hashlock phrase** (or an existing 32-byte
preimage) into the `hash:` record a hash-locked spend path commits to,
and back the **preimage** up as an `ms1` plate string. The digest
goes to [`md compose`](#md-compose) as a `<kind>=<digest>` path term;
the preimage is the secret that later unlocks that path.

The form handles a secret, and treats it as one:

- The phrase, [`--hex`](#ms-hashlock-hex) and the `ms1` positional
  are secret fields: masked and never saved, and a run with one of
  them filled asks for confirmation.
- On Linux each goes to `ms` over standard input, never on the
  command line (see [Secret channels](#secret-channels)): the phrase
  as `--hashlock-phrase-stdin`, `--hex` as `--hex -`, the plate as a
  positional `-`. The phrase is sent **byte for byte** — edge spaces
  included — because the preimage is a hash of exactly those bytes.
- stdout carries only the public `hash:` record; the engraving card on
  stderr carries the preimage. `--json` changes that (see
  [`--json`](#ms-hashlock-json)).

**Exactly one source.** The `ms1` positional, the phrase, `--hex`,
[`--in`](#ms-hashlock-in) and [`--random`](#ms-hashlock-random) are
alternatives. While none is filled all are marked **Required**; once
one is, every source after it in that order greys out
(`form::conditional::ms_hashlock`).

**Run waits for a `--kind`.** The [`--kind`](#ms-hashlock-kind)
dropdown opens on *(choose)*, and Run and both Copy buttons stay
disabled with *"choose a --kind first: without one, stdout carries the
sha256 record whatever your wallet's kind is"* until you pick one.

> **GUI form:** see [GUI Forms › ms › hashlock](#gui-form-ms-hashlock).

## Outline {#ms-hashlock-outline}

- [`--hashlock-phrase`](#ms-hashlock-hashlock-phrase) — the hashlock phrase, byte for byte (secret)
- [`--hashlock-phrase-stdin`](#ms-hashlock-hashlock-phrase-stdin) — GUI-managed: the channel the phrase travels on
- [`--kind`](#ms-hashlock-kind) — which hash the script commits to (Run waits for it)
- [`--hex`](#ms-hashlock-hex) — an existing 32-byte preimage as 64 hex characters (secret)
- [`--in`](#ms-hashlock-in) — read the preimage plate (`ms1`) from a file
- [`--random`](#ms-hashlock-random) — 32 random bytes; requires `--out`
- [`--method`](#ms-hashlock-method) — phrase → preimage method (phrase source only)
- [`--out`](#ms-hashlock-out) — write the preimage `ms1` to a file, owner-only
- [`--json`](#ms-hashlock-json) — one JSON object on stdout, which then carries the secret
- [`--no-engraving-card`](#ms-hashlock-no-engraving-card) — suppress the stderr card
- [`--emit-record`](#ms-hashlock-emit-record) — also print a `phrase:` record on the card (phrase source only)
- [`--group-size`](#ms-hashlock-group-size) — group the `ms1` on the card every N characters
- [`--separator`](#ms-hashlock-separator) — card separator (`space`)
- [`--phrase-looks-like-digest-ok`](#ms-hashlock-phrase-looks-like-digest-ok) — accept a phrase that looks like a hex digest

## `--hashlock-phrase` {#ms-hashlock-hashlock-phrase}

Secret field. The hashlock phrase, **never trimmed**. Type the value,
or `@env:VAR` to have the GUI read it from its own environment; a
value that is `-`, or that reads like `@env…` after trimming and
case-folding, is refused (see [Secret channels](#secret-channels)).
With the default [`--method`](#ms-hashlock-method) the preimage is
`PBKDF2-HMAC-SHA256(phrase, salt "ms-hashlock-v1", 100000
iterations)`.

## `--hashlock-phrase-stdin` {#ms-hashlock-hashlock-phrase-stdin}

Boolean, rendered **disabled**: the GUI sets it itself when it sends
the phrase over standard input, and you never tick it.

## `--kind` {#ms-hashlock-kind}

Dropdown. Which hash the **script** commits to. It has no default on
purpose: without `--kind`, `ms` still writes a `sha256` record to
stdout, and a pipe that consumes it would pack a sha256 payload for a
wallet whose kind may be another.

### Outline {#ms-hashlock-kind-outline}

- [`(choose)`](#ms-hashlock-kind-)
- [`sha256`](#ms-hashlock-kind-sha256)
- [`hash256`](#ms-hashlock-kind-hash256)
- [`ripemd160`](#ms-hashlock-kind-ripemd160)
- [`hash160`](#ms-hashlock-kind-hash160)
- [`all kinds — lookup only`](#ms-hashlock-kind-all-kinds-lookup-only)

### `(choose)` {#ms-hashlock-kind-}

The initial state. Nothing is emitted; Run and Copy stay disabled.

### `sha256` {#ms-hashlock-kind-sha256}

`--kind sha256`: stdout is `hash:<64 hex>`.

### `hash256` {#ms-hashlock-kind-hash256}

`--kind hash256` (double SHA-256).

### `ripemd160` {#ms-hashlock-kind-ripemd160}

`--kind ripemd160`: a 40-hex digest.

### `hash160` {#ms-hashlock-kind-hash160}

`--kind hash160` (RIPEMD-160 of SHA-256): stdout is
`hash:hash160:<40 hex>`.

### `all kinds — lookup only` {#ms-hashlock-kind-all-kinds-lookup-only}

Emits no `--kind` flag, so `ms` lists the digest under every kind on
stderr — for matching an older plate against a descriptor. Run is
allowed, and the form shows the note *"stdout is the sha256 record;
your wallet's kind may differ"*.

## `--hex` {#ms-hashlock-hex}

Secret field. An existing preimage: exactly 32 bytes as 64 hex
characters. Sent as `--hex -` over standard input on Linux. `ms`
warns that the first spend publishes these bytes, so they must not be
anything else's secret.

## `--in` {#ms-hashlock-in}

Path widget. Read the preimage plate's `ms1` string from FILE — the
way to re-derive the digest from a plate without typing it.

## `--random` {#ms-hashlock-random}

Boolean. 32 bytes from the OS random source. **Requires**
[`--out`](#ms-hashlock-out): the GUI marks `--out` Required while
`--random` is on (`ms`: *"--random needs --out FILE: a preimage that
reaches no file is data loss"*, exit 64).

## `--method` {#ms-hashlock-method}

Dropdown; default `hardened`. The phrase → preimage method. Enabled
only when the phrase is the source.

### Outline {#ms-hashlock-method-outline}

- [`hardened`](#ms-hashlock-method-hardened)
- [`sha256`](#ms-hashlock-method-sha256)

### `hardened` {#ms-hashlock-method-hardened}

PBKDF2-HMAC-SHA256, salt `ms-hashlock-v1`, 100 000 iterations — the
default.

### `sha256` {#ms-hashlock-method-sha256}

One SHA-256 of the phrase bytes: the brainwallet construction, much
cheaper to guess.

## `--out` {#ms-hashlock-out}

Path widget. Write the preimage `ms1` string to FILE, owner-only. It
never suppresses stdout.

## `--json` {#ms-hashlock-json}

Boolean. One JSON object on stdout in place of the `hash:` line. That
object **carries the secret**; the form shows the note *"--json:
stdout then carries the secret"* while it is ticked.

## `--no-engraving-card` {#ms-hashlock-no-engraving-card}

Boolean. Suppress the engraving card on stderr (which carries the
preimage).

## `--emit-record` {#ms-hashlock-emit-record}

Boolean. Also print a `phrase:` record on the card, for
`me sysw pack --pack-preimage`. Enabled only when the phrase is the
source. The record carries the phrase, so it goes on the card, never
on stdout.

## `--group-size` {#ms-hashlock-group-size}

Number widget, `0..=65535`, default `5`. Group the `ms1` on the card
every N characters (`0` = no grouping).

## `--separator` {#ms-hashlock-separator}

Dropdown with one value, `space` (the default).

### `space` {#ms-hashlock-separator-space}

ASCII space between groups.

## `--phrase-looks-like-digest-ok` {#ms-hashlock-phrase-looks-like-digest-ok}

Boolean. A phrase of exactly 40 or 64 hex characters is probably a
digest pasted into the wrong field: hashing it would commit the wallet
to the ASCII text of the digest. `ms` stops on such a phrase (exit 1,
*"that phrase is 64 hex characters, the width of a digest …"*) unless
this box is ticked.

## Positional `ms1`

Secret field. A preimage-kind `ms1` string (a preimage plate), to
re-derive the digest from it. Sent as a positional `-` over standard
input on Linux. It is first in the source order, so filling it greys
every other source.

## Worked example — a phrase to a sha256 digest

1. **ms** tab; pick **Hashlock (phrase/preimage -> hashlock digest)**.
2. Type the phrase into `--hashlock-phrase` (here the demo phrase
   `correct horse battery staple`, which you must never use).
3. `--kind`: `sha256`.
4. **Run**, and confirm in the dialog, whose **Secrets:** line reads
   `--hashlock-phrase ← stdin via --hashlock-phrase-stdin + '\r\n' (typed)`.

```text
argv: ms hashlock --hashlock-phrase-stdin --kind sha256
exit: 0
stdout:
hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
stderr:
THIS CARD CARRIES THE PREIMAGE -- the secret. stdout carries only the public digest.
digest:          3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
for md compose:  --wrapper wsh --path <your other paths> --path keyless,sha256=3cf5d421… --experimental --md-only
preimage (ms1):  ms10h ashsq 0p7ja f9gsj jpkjv ll2l2 74w8a 388xg qzlew p73sc ptwxg tjugs pvs8t klufg 89hqj
…
source:          phrase (stdin)
```

The `for md compose:` line is the [`md compose --path`](#md-compose-path)
term to paste. Engrave the preimage `ms1`; the digest is public.

## Refusals

| Trigger | Refusal |
|---|---|
| `--kind` left at *(choose)* | GUI: Run and Copy disabled, with the reason as their tooltip |
| No source | `ms`: *"no source given; exactly one source: …"*, exit 64 (the GUI marks every source Required first) |
| More than one source | the GUI greys every source after the first filled one |
| `--random` without `--out` | `ms`, exit 64 (the GUI marks `--out` Required) |
| `--method` or `--emit-record` with a non-phrase source | `ms`, exit 64 (the GUI greys both) |
| A phrase of 40 or 64 hex characters | `ms`, exit 1, unless `--phrase-looks-like-digest-ok` |
| A secret field holding `-`, a lookalike, or a value ending in CR/LF | GUI refusal before Run; see [Secret channels](#secret-channels-refusals) |
| macOS / Windows: a phrase starting with `-` | GUI `value-starts-with-dash`: there the phrase goes on the command line, and `--hashlock-phrase=VALUE` is not byte-exact for `ms` |
