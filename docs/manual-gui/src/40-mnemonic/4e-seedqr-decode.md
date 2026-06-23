# `mnemonic seedqr-decode` {#mnemonic-seedqr-decode}

Decode a SeedQR numeric payload back to its BIP-39 phrase. The inverse
of [`mnemonic seedqr-encode`](#mnemonic-seedqr-encode). The GUI exposes
the toolkit's `seedqr decode` sub-subcommand as a flat **SeedQR Decode**
form on the `mnemonic` tab. The recovered phrase is emitted on stdout
(or a JSON envelope when `--json-out` is set).

:::danger
The worked example in this chapter uses an all-zero SeedQR payload that
decodes to the canonical all-`abandon` test vector. **Never engrave or
fund** any wallet recovered from a demonstration payload. A real SeedQR
payload IS secret-equivalent to the seed it encodes; the value pasted
into `--from seedqr=` (and the recovered phrase on stdout) is master
key material. The run-confirm modal redacts the secret-bearing argv
token as a fixed `••••` sentinel (see [§14 Defense 2](#secret-handling));
the cold/airgapped operational practice remains good hygiene for every
secret-bearing decode.
:::

## Outline {#mnemonic-seedqr-decode-outline}

- [`--from`](#mnemonic-seedqr-decode-from) — the SeedQR payload to decode (`seedqr=<value-or->`; canonical input)
- [`--variant`](#mnemonic-seedqr-decode-variant) — how to interpret the payload (`standard` default, or `compact`)
- [`--digits`](#mnemonic-seedqr-decode-digits) — DEPRECATED digit-string input (Standard only; mutually exclusive with `--from`)
- [`--json-out`](#mnemonic-seedqr-decode-json-out) — write a versioned JSON envelope to PATH instead of plain stdout

## `--from` {#mnemonic-seedqr-decode-from}

The canonical SeedQR input (v0.31.6+). The GUI renders this as a
NodeValueComposite field: a node-type selector (fixed to `seedqr` —
only that node type is accepted) plus a value editor. Grammar
`seedqr=<value>` inline or `seedqr=-` to route the value through stdin.

Under `--variant standard` (default) the value is a numeric digit
string (48 / 60 / 72 / 84 / 96 ASCII digits — 12 / 15 / 18 / 21 /
24-word phrases). Under `--variant compact` the value is lowercase hex
of the raw BIP-39 entropy bytes (32 hex chars = 12-word; 64 hex chars =
24-word). The value editor renders as a masked `SecretLineEdit`; the
composite paste-warn, argv-mask, and run-confirm protections key on the
secret-bearing `seedqr` node type.

### `seedqr` {#mnemonic-seedqr-decode-from-seedqr}

The only accepted node type. Carries the SeedQR payload — decimal
digits under Standard, lowercase entropy hex under Compact. The
equivalent Standard conversion is also reachable through
[`mnemonic convert`](#mnemonic-convert) `--from seedqr=<digits> --to
phrase`; the dedicated `seedqr-decode` form additionally exposes the
Compact variant and the JSON envelope.

## `--variant` {#mnemonic-seedqr-decode-variant}

Dropdown. How to interpret the `--from seedqr=` value (default
`standard`). Two allowed values. The `?` help-icon deep-links here.

### Outline {#mnemonic-seedqr-decode-variant-outline}

- [`standard`](#mnemonic-seedqr-decode-variant-standard)
- [`compact`](#mnemonic-seedqr-decode-variant-compact)

### `standard` {#mnemonic-seedqr-decode-variant-standard}

Standard SeedQR (default). Interprets the value as a decimal digit
string, 4 digits per BIP-39 word index. Word counts 12 / 15 / 18 / 21
/ 24 (48 / 60 / 72 / 84 / 96 digits) are all supported.

### `compact` {#mnemonic-seedqr-decode-variant-compact}

CompactSeedQR. Interprets the value as lowercase hex of the raw BIP-39
entropy bytes (16 bytes = 12-word; 32 bytes = 24-word). Only 12- and
24-word payloads are valid for compact. To decode a scanned binary-mode
QR, hex-encode the scanned bytes first and paste the hex into the value
editor.

## `--digits` {#mnemonic-seedqr-decode-digits}

**DEPRECATED** (v0.31.6). The original digit-string input (Standard
variant only). Still accepted, but the toolkit emits a stderr
deprecation notice directing to `--from seedqr=`. The GUI renders this
as a masked `SecretLineEdit` (the digit string is secret-bearing) but
the field is retained only for backward compatibility — prefer the
`--from seedqr=` composite. Mutually exclusive with `--from` (clap-level
`conflicts_with`; exit 64). Exactly one of `--from seedqr=` or
`--digits` is required.

## `--json-out` {#mnemonic-seedqr-decode-json-out}

Path widget. When set, the toolkit writes a versioned JSON envelope to
the given path instead of emitting the recovered phrase on stdout. The
OS file picker is not yet wired (FOLLOWUP `gui-file-picker-affordance`);
the field accepts a path string. The written file holds the recovered
seed phrase — emit it only to encrypted, access-controlled storage.

## Worked example — Standard decode

1. Switch to the **mnemonic** tab; pick **SeedQR Decode** in the
   subcommand selector.
2. Leave `--variant` at its seeded default `standard`.
3. In the `--from` field, keep the node selector at `seedqr` and paste
   a 48-digit Standard payload (this all-zero-but-last payload decodes
   to the canonical test phrase):

   ```text
   000000000000000000000000000000000000000000000003
   ```

4. The `Preview:` line resembles:

   ```text
   mnemonic seedqr decode --from "seedqr=000000000000000000000000000000000000000000000003"
   ```

5. Click **Run**; redact-confirm in the modal.

The output panel renders the recovered 12-word BIP-39 phrase on stdout.

\index{mnemonic seedqr-decode}
