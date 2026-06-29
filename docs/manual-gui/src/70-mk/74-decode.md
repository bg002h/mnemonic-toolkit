# `mk decode` {#mk-decode}

Reassemble one or more `mk1` strings and decode them to xpub +
origin metadata. The inverse of [`mk encode`](#mk-encode). For a
richer view (per-component path breakdown, xpub-derived
fingerprint, per-chunk BCH variant) use
[`mk inspect`](#mk-inspect).

> **GUI form:** see [GUI Forms › mk › decode](#gui-form-mk-decode).

## `--json` {#mk-decode-json}

Boolean. Emit a structured JSON object on stdout (fields:
`schema_version`, `xpub`, `origin_fingerprint`, `origin_path`,
`policy_id_stubs`, `chunks`, `code_variant`) instead of multi-line
labeled text. Default off.

## Positional `mk1-strings`

One or more `mk1` strings. **Repeating** positional. The literal
`-` causes the binary to read one string per line from stdin
until EOF. The GUI renders this as a multi-row text field.

## Worked example

1. **mk** tab; pick **Decode (mk1 → xpub)**.
2. Paste both canonical `mk1` strings into the `mk1-strings`
   field:

   ```text
   mk1qpydzkpqqsqupllwqr02m0h0qvzg3vs7zqsrqq4g4z52329g4z52329g4z52329g4z52329g4z52329g4z52329g4qpy6m8lr3sdrxkguwax
   mk1qpydzkppfdkdzdssxt9fh54wh8vsp2jdghv74kq2e9prxaxy2xnj2ng8vm68nf54c0vrdlfrgjzpd
   ```

3. Leave `--json` unchecked.
4. **Run**. No run-confirm modal.

The output panel emits on stdout:

```{.text include="74-mk-decode.out"}
xpub:                xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a
origin_fingerprint:  deadbeef
origin_path:         84'/0'/0'
policy_id_stubs:     c0ffee00
chunks:              2 (long)
```

The path renders without a leading `m/`. The `chunks: 2 (long)`
line reports the variant of the **first** chunk only; for
per-chunk classification use [`mk inspect`](#mk-inspect). When
the originating mk1 was encoded with
[`--privacy-preserving`](#mk-encode-privacy-preserving), the
`origin_fingerprint:` line reads
`(omitted, privacy-preserving mode)` instead of a hex value.

## Refusals

| Trigger | Refusal |
|---|---|
| No positional `mk1-strings` provided AND stdin not used | clap-level `required` error |
| Any positional that does not parse as `mk1` | exit 2 with the matching `mk-codec` error (e.g. `error: invalid HRP …`, `error: invalid string length …`, `error: BCH uncorrectable …`) |
| Supplied chunks have inconsistent `chunk_set_id` headers | exit 2 with `error: chunk_set_id mismatch` per `mk-codec` |
| `mk1` strings parse but the embedded version is newer than this build understands | exit 3 with `error: unsupported version …` per `mk-codec` |
| `mk1` chunk payloads have malformed padding or reserved-bits-set | exit 2 with `error: malformed payload padding` / `error: reserved bits set` per `mk-codec` |
