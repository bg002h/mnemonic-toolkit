# `md compose` {#md-compose}

\index{md compose}Lower an **ordered list of spend paths** to a
BIP-388 wallet-policy template by fixed rules. This is the CLI behind
the Wallet Policy composer, and the opposite of
[`md compile`](#md-compile): no search and no cost model, so every
implementation writes the same template from the same paths. The
result is a **keyless** template (placeholders `@0`, `@1`, … with
their default origins), ready for [`md encode`](#md-encode) or
[`md descriptor --template`](#md-descriptor-template).

The form carries no secret input: `md compose` reads spend-path
descriptions and hash digests, never keys or preimages. The
run-confirm modal does not fire.

Two input modes, one of which you must choose: a list of
[`--path`](#md-compose-path) rows **or** one
[`--preset`](#md-compose-preset). The conditional-visibility engine
(`form::conditional::md_compose`) marks both **Required** while
neither has a value, and greys out the other once you fill one (the
CLI refuses the pair: `error: the argument '--path <PATH>' cannot be
used with '--preset <PRESET>'`, exit 2).
[`--unspendable`](#md-compose-unspendable) is greyed unless
[`--wrapper`](#md-compose-wrapper) is `tr`, because md refuses it for
the other wrappers (exit 1).

> **GUI form:** see [GUI Forms › md › compose](#gui-form-md-compose).

## Outline {#md-compose-outline}

- [`--wrapper`](#md-compose-wrapper) — script wrapper: `tr`, `wsh`, `sh-wsh` or `sh` (required)
- [`--path`](#md-compose-path) — one spend path; repeating, in the order you list them (XOR with `--preset`)
- [`--preset`](#md-compose-preset) — one of the six named archetypes (XOR with `--path`)
- [`--experimental`](#md-compose-experimental) — admit key-less paths and unsorted-where-sorted-was-legal, with a warning
- [`--json`](#md-compose-json) — emit the template, slot map and marks as JSON
- [`--unspendable`](#md-compose-unspendable) — which unspendable taproot internal key to use (`tr` only)
- [`--md-only`](#md-compose-md-only) — compose even when every known coordinator refuses the policy

## `--wrapper` {#md-compose-wrapper}

Dropdown, **required** (clap-required; the form opens on `tr`). The
script wrapper the paths are lowered into.

### Outline {#md-compose-wrapper-outline}

- [`tr`](#md-compose-wrapper-tr)
- [`wsh`](#md-compose-wrapper-wsh)
- [`sh-wsh`](#md-compose-wrapper-sh-wsh)
- [`sh`](#md-compose-wrapper-sh)

### `tr` {#md-compose-wrapper-tr}

Taproot. Paths become `multi_a` / script leaves; when no path supplies
a real internal key, md uses an unspendable one (see
[`--unspendable`](#md-compose-unspendable)). Default origins use the
BIP-48 script type `3'` (`@0/48'/0'/0'/3'/<0;1>/*`).

### `wsh` {#md-compose-wrapper-wsh}

Native segwit v0 script hash, BIP-48 script type `2'`. A single
`2of3` path lowers to `sortedmulti`.

### `sh-wsh` {#md-compose-wrapper-sh-wsh}

The `wsh` script nested in P2SH, for coordinators that need a legacy
address format.

### `sh` {#md-compose-wrapper-sh}

Legacy P2SH.

## `--path` {#md-compose-path}

Repeating Text widget, one spend path per row, **in the order you
list them** — the order is meaningful, and the GUI passes the rows in
that order (`--path 2of3 --path 1of1,older=26280`). The grammar is
`<k>of<n>[,older=N|older=Nu|after=H|after=Tt][,<hash>=HEX][,unsorted]`,
or `keyless,<hash>=HEX[,older=..|after=..]` for a path with no key,
where `<hash>` is `sha256` / `hash256` (64 hex) or `ripemd160` /
`hash160` (40 hex), at most one per path. A key-less path needs
[`--experimental`](#md-compose-experimental). A digest for a `<hash>=`
term comes from [`ms hashlock`](#ms-hashlock), whose engraving card
prints a ready-made `--path keyless,<kind>=<digest>` line.

## `--preset` {#md-compose-preset}

Text widget. One of the six named archetypes,
`<name>[,<k>of<n>]*[,<param>=<value>]*`, for example
`kofn-recovery,2of3,older=26280`. Mutually exclusive with
[`--path`](#md-compose-path).

## `--experimental` {#md-compose-experimental}

Boolean. Admit key-less paths and `unsorted` where sorted was legal.
md prints `warning: EXPERIMENTAL: …` naming each such path. Without
it, a key-less path is refused (`md: this policy needs
--experimental`, exit 1).

## `--json` {#md-compose-json}

Boolean. Emit one JSON object on stdout: the origin-less `template`,
the inline-origin `template_with_origins`, the `slots` map, the
taproot `internal_key_path`, the `experimental` marks, and with
`--preset` the resolved preset.

## `--unspendable` {#md-compose-unspendable}

Dropdown. Which unspendable taproot internal key to use when no spend
path supplies a real one. **Greyed unless `--wrapper` is `tr`**; md
refuses it under `wsh`, `sh` and `sh-wsh` (`md: --unspendable:
--wrapper wsh has no taproot internal key to choose`, exit 1).

### Outline {#md-compose-unspendable-outline}

- [`(none)`](#md-compose-unspendable-)
- [`nums`](#md-compose-unspendable-nums)
- [`liana`](#md-compose-unspendable-liana)

### `(none)` {#md-compose-unspendable-}

The default: the flag is left off argv and md uses its own default,
the `nums` key.

### `nums` {#md-compose-unspendable-nums}

The BIP-341 H-point, which is also what omitting the flag produces.

### `liana` {#md-compose-unspendable-liana}

Liana's own derived unspendable key, which Liana requires in order to
import the wallet.

## `--md-only` {#md-compose-md-only}

Boolean. After composing, md prints a coordinator table on stderr
(`note: coordinators for this template …`). When **every** coordinator
it knows refuses the policy at every verified version, md refuses to
compose (`md: no wallet coordinator md knows imports this policy …
Pass --md-only to compose it anyway.`, exit 1). `--md-only` composes it
anyway: md can rebuild such a wallet from its card, but md cannot
sign, so you need some other way to spend from it.

## Worked example — a 2-of-3 multisig

1. **md** tab; pick **Compose (spend paths -> wallet policy)**.
2. `--wrapper`: `wsh`.
3. Add a `--path` row: `2of3`.
4. **Run** (no confirm dialog; nothing here is secret).

```text
argv: md compose --wrapper wsh --path 2of3
exit: 0
stdout:
wsh(sortedmulti(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*))
stderr:
note: coordinators for this template (verified versions only; newer: unmeasured):
  Liana 8.0-15.0: refuses (no locked path)
  Nunchuk 2.1.1: unproven (a template has no keys, so no import can be claimed)
  Bitcoin Core 24.2-31.1: unproven (a template has no keys, so no import can be claimed)
note: stdout is a keyless descriptor template (no keys)
```

Adding a second row `1of1,older=26280` under `--wrapper tr` gives a
taproot policy with the 2-of-3 as one leaf and a single key after
26280 blocks as another; the preset
`kofn-recovery,2of3,older=26280` under `wsh` gives the same shape as
an `or_d` script.

## Refusals

| Trigger | Refusal |
|---|---|
| Neither `--path` nor `--preset` | clap, exit 2 (the GUI marks both Required first) |
| Both `--path` and `--preset` | clap `cannot be used with`, exit 2 (the GUI greys the other) |
| `--unspendable` with a wrapper other than `tr` | md, exit 1 (the GUI greys the dropdown) |
| A key-less path without `--experimental` | `md: this policy needs --experimental`, exit 1 |
| Every known coordinator refuses the policy | md, exit 1, until `--md-only` |
