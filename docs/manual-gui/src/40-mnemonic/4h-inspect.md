# `mnemonic inspect` {#mnemonic-inspect}

Describe the contents of an m-format card (`ms1` / `mk1` / `md1`)
without performing any conversion. The GUI exposes the toolkit's
`inspect` subcommand as a flat **Inspect** form on the `mnemonic` tab.
At least one of `--ms1` / `--mk1` / `--md1` is required; the three may
be combined (one HRP per card). Output is a text-form report on stdout,
or a single JSON envelope when `--json` is set.

Per card kind the report carries:

- **`ms1`** — tag (`entr` for entropy-only; `mnem` for a
  language-tagged card), payload kind, byte length, bit strength
  (= 8 × bytes). The entropy hex is suppressed by default; opt in with
  [`--reveal-secret`](#mnemonic-inspect-reveal-secret). `mnem`-kind
  cards also report the stored wordlist language.
- **`mk1`** — policy-id-stub count, origin fingerprint (or `<absent>`
  for a privacy-preserving emission), origin path, xpub.
- **`md1`** — the keyless BIP-388 `@N` wallet-policy template (the full
  miniscript expression with `@N` key placeholders, rendered identically
  to `md decode`), placeholder count, root-tree tag (`Wpkh` / `Tr` /
  `Wsh` / …), wallet-policy-mode flag, path-decl shape (`Shared` vs
  `Divergent`).

:::danger
The worked example uses a canonical zero-entropy `ms1` card. **Never
engrave or fund** any wallet derived from a demonstration card. An
`ms1` card is master key material; the `--ms1` field is secret-bearing
and [`--reveal-secret`](#mnemonic-inspect-reveal-secret) prints the raw
entropy hex on stdout. The run-confirm modal redacts the secret-bearing
`--ms1` argv token as a fixed `••••` sentinel (see [§14 Defense
2](#secret-handling)). `mk1` and `md1` carry no secret material.
:::

## Outline {#mnemonic-inspect-outline}

- [`--ms1`](#mnemonic-inspect-ms1) — single `ms1` chunk to inspect (`-` reads from stdin)
- [`--mk1`](#mnemonic-inspect-mk1) — one or more `mk1` chunks (repeating; `-` reads from stdin)
- [`--md1`](#mnemonic-inspect-md1) — one or more `md1` chunks (repeating; `-` reads from stdin)
- [`--json`](#mnemonic-inspect-json) — emit a single JSON envelope instead of the text report
- [`--reveal-secret`](#mnemonic-inspect-reveal-secret) — reveal the `ms1` entropy hex on stdout (no effect for `mk1` / `md1`)

## `--ms1` {#mnemonic-inspect-ms1}

Text field. A single `ms1` chunk to inspect. Use `-` to read one chunk
from stdin. Combinable with `--mk1` / `--md1` (the toolkit's D35 rule
requires at least one card across the three flags). The GUI renders the
value editor as a masked `SecretLineEdit` because an `ms1` card is
secret-bearing; pasting a real card triggers the run-confirm modal.

## `--mk1` {#mnemonic-inspect-mk1}

Text field, repeating. One or more `mk1` chunks to inspect. The GUI
renders this as a repeating row with a **+ Add mk1** button — one chunk
per row. Use `-` on a single occurrence to read chunks from stdin (one
per line). Combinable with `--ms1` / `--md1`. `mk1` is public (an xpub
card); the value editors are not masked.

## `--md1` {#mnemonic-inspect-md1}

Text field, repeating. One or more `md1` chunks to inspect, rendered as
a repeating row with a **+ Add md1** button. Use `-` on a single
occurrence to read chunks from stdin (one per line). Combinable with
`--ms1` / `--mk1`. `md1` is public (a descriptor card); the value
editors are not masked.

## `--json` {#mnemonic-inspect-json}

Boolean flag. When set, emits a single JSON envelope on stdout instead
of the text-form report. The envelope carries a top-level
`schema_version: "1"` field followed by the kind-specific fields.

## `--reveal-secret` {#mnemonic-inspect-reveal-secret}

Boolean flag (checkbox). When set, the `ms1` report includes the raw
entropy hex on stdout; the default suppresses it (the summary stays at
length / bit-strength). No effect for `mk1` / `md1`, which carry no
secret material.

The flag itself is presence-only and is **not** classified
secret-bearing by the schema (it carries no value) — but the *output*
it unlocks is secret. The run-confirm modal still fires on the
underlying `--ms1` input; treat any revealed entropy hex on stdout as
master key material and redirect it to encrypted storage rather than
leaving it in the output panel.

## Worked example — inspect an `ms1` card

1. Switch to the **mnemonic** tab; pick **Inspect** in the subcommand
   selector.
2. Paste a zero-entropy `ms1` card into the `--ms1` masked field:

   ```text
   ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
   ```

3. Leave `--reveal-secret` unchecked (so the entropy hex stays
   suppressed).
4. The `Preview:` line resembles:

   ```text
   mnemonic inspect --ms1 ••••
   ```

5. Click **Run**; redact-confirm in the modal.

The output panel renders the text-form `ms1` report on stdout (tag,
payload kind, byte length, bit strength) with the entropy hex
suppressed. Tick `--reveal-secret` only on an airgapped machine when you
genuinely need the raw entropy.

\index{mnemonic inspect}
