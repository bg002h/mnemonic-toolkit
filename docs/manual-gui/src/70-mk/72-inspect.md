# `mk inspect` {#mk-inspect}

Decode one or more `mk1` strings and emit a structural commentary
that goes beyond [`mk decode`](#mk-decode)'s plain decode. Adds
the xpub-derived fingerprint (computed from the public key
itself, distinct from the engraved `--origin-fingerprint` which
is the **master** fingerprint), per-component derivation-path
breakdown, and per-chunk BCH-variant classification (`regular` or
`long`).

> **GUI form:** see [GUI Forms › mk › inspect](#gui-form-mk-inspect).

## `--json` {#mk-inspect-json}

Boolean. Emit structured JSON on stdout instead of the multi-line
text form. Default off.

The JSON object adds an `origin_path_components` array (one entry
per BIP-32 child number, each rendered as e.g. `"84h (hardened)"`)
and a `chunk_variants` array (one entry per supplied chunk).

## Positional `mk1-strings`

One or more `mk1` strings. **Repeating** positional. Required at
the runtime level — at least one string must be supplied. The
literal `-` causes the binary to read one string per line from
stdin until EOF. The GUI renders this as a multi-row text field;
the deep-link helper `?` icon scrolls here.

## Worked example

1. **mk** tab; pick **Inspect (structural commentary)**.
2. Paste both canonical `mk1` strings (one per row) into the
   `mk1-strings` field:

   ```text
   mk1qpydzkpqqsqupllwqr02m0h0qvzg3vs7zqsrqq4g4z52329g4z52329g4z52329g4z52329g4z52329g4z52329g4qpy6m8lr3sdrxkguwax
   mk1qpydzkppfdkdzdssxt9fh54wh8vsp2jdghv74kq2e9prxaxy2xnj2ng8vm68nf54c0vrdlfrgjzpd
   ```

3. Leave `--json` unchecked.
4. **Run** (no run-confirm modal — `mk inspect` operates on
   public material).

The output panel renders on stdout:

```{.text include="72-mk-inspect.out"}
xpub:                xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a
origin_fingerprint:  deadbeef
xpub_fingerprint:    ebc0ee0b
origin_path:         84'/0'/0'
  component[0]:       84h (hardened)
  component[1]:       0h (hardened)
  component[2]:       0h (hardened)
policy_id_stubs:     c0ffee00
chunks:              2
  chunk[0]:           long (BCH variant)
  chunk[1]:           regular (BCH variant)
```

Note the path string renders without the leading `m/` and uses
single-quote hardened markers; the per-component breakdown uses
the `h (hardened)` variant per
`bitcoin::bip32::ChildNumber::Hardened`. Note also that the two
chunks use distinct BCH variants — long for the leading chunk
(richer error-correction budget for the header), regular for the
trailing chunk.

## Refusals

| Trigger | Refusal |
|---|---|
| No positional `mk1-strings` provided AND stdin not used | clap-level `required` error |
| Any positional that does not parse as `mk1` | exit 2 with the matching `mk-codec` error (e.g. `error: invalid HRP …`, `error: invalid string length …`, `error: BCH uncorrectable …`) |
| One supplied mk1 has a `chunk_set_id` that disagrees with the others | exit 2 with `error: chunk_set_id mismatch` per `mk-codec` |
| `mk1` strings parse but the embedded version is newer than this build understands | exit 3 with `error: unsupported version …` per `mk-codec` |
