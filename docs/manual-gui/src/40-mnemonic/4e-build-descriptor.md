# `mnemonic build-descriptor` {#mnemonic-build-descriptor}

Build a **validated `wsh(...)` descriptor** + its **BIP-388
wallet-policy** from a **versioned JSON policy-tree spec**, or from a
**curated archetype preset**. The spec is a fragment-level miniscript
tree (keys, multisig, timelocks, hashlocks, combinators); the engine
renders it to a concrete multipath descriptor and runs it through a
**fail-closed validation gate** before emitting anything. Like
[`restore`](#mnemonic-restore) / [`export-wallet`](#mnemonic-export-wallet),
build-descriptor is **watch-only-out** — it takes cosigner **xpub**s
and NEVER accepts secret material (an `xprv` / WIF in a key node is
refused, exit 2, without ever echoing the key). It does not sign.

\index{mnemonic build-descriptor}

:::danger
The worked example uses placeholder cosigner xpubs. build-descriptor is
watch-only by construction — it accepts only public xpub material and
refuses any `xprv` / WIF key node, so there is no secret-bearing field
here. The descriptors it emits are **vault policies**; review the
generated tree (`--emit-spec`) before engraving or funding any wallet
built from a hand-authored policy.
:::

## Outline {#mnemonic-build-descriptor-outline}

- [`--spec`](#mnemonic-build-descriptor-spec) — the JSON node-tree spec (file path or `-` for stdin)
- [`--spec-schema`](#mnemonic-build-descriptor-spec-schema) — dump the node-tree grammar JSON and exit
- [`--archetype`](#mnemonic-build-descriptor-archetype) — build from a curated preset instead of a `--spec` tree
- [`--key`](#mnemonic-build-descriptor-key) — primary-path cosigner key (repeat per cosigner; argv order preserved)
- [`--threshold`](#mnemonic-build-descriptor-threshold) — primary quorum k
- [`--recovery-key`](#mnemonic-build-descriptor-recovery-key) — recovery-path cosigner key (repeating)
- [`--recovery-threshold`](#mnemonic-build-descriptor-recovery-threshold) — recovery quorum k
- [`--final-key`](#mnemonic-build-descriptor-final-key) — last-resort key (`decaying-multisig` tier 3)
- [`--older`](#mnemonic-build-descriptor-older) — relative timelock (blocks) gating the recovery path
- [`--recovery-older`](#mnemonic-build-descriptor-recovery-older) — `decaying-multisig` tier-2 relative timelock (> `--older`)
- [`--after`](#mnemonic-build-descriptor-after) — `decaying-multisig` tier-3 absolute locktime
- [`--hash`](#mnemonic-build-descriptor-hash) — SHA-256 digest (64 hex) for `hashlock-gated`
- [`--emit-spec`](#mnemonic-build-descriptor-emit-spec) — print the lowered + gate-validated spec JSON instead of building
- [`--allow`](#mnemonic-build-descriptor-allow) — reviewed opt-out of ONE funds-safety sanity rule per occurrence (repeatable)
- [`--format`](#mnemonic-build-descriptor-format) — emit a single bare artifact (`descriptor` / `bip388`)
- [`--network`](#mnemonic-build-descriptor-network) — Bitcoin network (default `mainnet`; human-view rendering only)
- [`--json`](#mnemonic-build-descriptor-json) — emit a structured JSON envelope for the GUI

## `--spec` {#mnemonic-build-descriptor-spec}

The JSON node-tree spec — a file path, or `-` for stdin. A versioned
document `{"schema_version": 1, "wrapper": "wsh", "root": <node>}`
(`deny_unknown_fields`). Each `<node>` is an externally-tagged object
(`pk` / `pkh` / `multi` / `sortedmulti` / `older` / `after` / hashlocks
/ `and_v` / `or_d` / `or_i` / `or_b` / `andor` / `thresh` / `wrap`).
Conflicts with `--archetype`. If omitted, stdin is read when it is not
a TTY. The GUI renders this as a Path widget; `stdio_sentinel: true`.

## `--spec-schema` {#mnemonic-build-descriptor-spec-schema}

Dump the versioned node-tree grammar JSON (the schema the GUI + presets
consume) and exit; ignores all other inputs. Since v0.51.0 it also
carries an `archetypes` section: per-preset parameter field-specs. The
GUI renders this as a Boolean toggle (it is an action flag).

## `--archetype` {#mnemonic-build-descriptor-archetype}

Build from a curated archetype preset instead of a `--spec` node-tree
(conflicts with `--spec`). Five curated vault shapes over the same
engine — no JSON authoring; parameters come from the preset flags
below, and the lowered tree flows through the SAME validation gate. The
Dropdown's default value is the empty sentinel `(none)` (no preset).
The GUI renders this flag with a `?` help-icon.

### Outline {#mnemonic-build-descriptor-archetype-outline}

- [`(none)`](#mnemonic-build-descriptor-archetype-)
- [`decaying-multisig`](#mnemonic-build-descriptor-archetype-decaying-multisig)
- [`hashlock-gated`](#mnemonic-build-descriptor-archetype-hashlock-gated)
- [`kofn-recovery`](#mnemonic-build-descriptor-archetype-kofn-recovery)
- [`simple-timelocked-inheritance`](#mnemonic-build-descriptor-archetype-simple-timelocked-inheritance)
- [`tiered-recovery`](#mnemonic-build-descriptor-archetype-tiered-recovery)

### `(none)` {#mnemonic-build-descriptor-archetype-}

The empty-string sentinel — no archetype preset selected. This is the
Dropdown's default; in this state the form expects a `--spec` tree
rather than the per-preset parameter flags.

### `decaying-multisig` {#mnemonic-build-descriptor-archetype-decaying-multisig}

`andor(multi(k1,T1…), older(N1), andor(multi(k2,T2…), older(N2),
and_v(v:pk(F), after(T))))` — a quorum decays through a recovery quorum
to a final key. Parameters: `--key` ×n (≥2), `--threshold`, `--older`,
`--recovery-key` ×n (≥2), `--recovery-threshold`, `--recovery-older`
(> `--older`), `--final-key` ×1, `--after`. Here `--older` is the
**tier-1** timelock.

### `hashlock-gated` {#mnemonic-build-descriptor-archetype-hashlock-gated}

`andor(pk(A), sha256(H), and_v(v:pk(B), older(N)))` — primary key +
SHA-256 preimage; recovery key after `N` blocks. Parameters: `--key`
×1, `--hash`, `--recovery-key` ×1, `--older`.

### `kofn-recovery` {#mnemonic-build-descriptor-archetype-kofn-recovery}

`or_d(multi(k,K…), and_v(v:pk(R), older(N)))` — k-of-n multisig; a
single recovery key after `N` blocks. Parameters: `--key` ×n (≥2),
`--threshold`, `--recovery-key` ×1, `--older`.

### `simple-timelocked-inheritance` {#mnemonic-build-descriptor-archetype-simple-timelocked-inheritance}

`or_d(pk(P), and_v(v:pkh(H), older(N)))` — owner spends anytime; heir
after `N` blocks. Parameters: `--key` ×1, `--recovery-key` ×1,
`--older`.

### `tiered-recovery` {#mnemonic-build-descriptor-archetype-tiered-recovery}

`or_i(sortedmulti(k1,P…), and_v(v:older(N), thresh(k2, pk, s:pk…)))` —
a primary sorted multisig OR a timelocked recovery threshold of
distinct keys. Parameters: `--key` ×n (≥2), `--threshold`, `--older`,
`--recovery-key` ×n (≥2), `--recovery-threshold`.

## `--key` {#mnemonic-build-descriptor-key}

Primary-path cosigner key (`[fp/path]xpub…`); repeat per cosigner.
**Argv order is preserved into the quorum** (even `sortedmulti`'s
descriptor string keeps authored order; sorting is script-time). The
GUI renders this as a Text widget with `repeating: true`. Watch-only —
an `xprv` / WIF here is refused (exit 2).

## `--threshold` {#mnemonic-build-descriptor-threshold}

Primary quorum threshold k. Range 1..20. The GUI renders this as a
Number widget; no `?` help-icon.

## `--recovery-key` {#mnemonic-build-descriptor-recovery-key}

Recovery-path cosigner key; repeat per cosigner (argv order preserved).
The GUI renders this as a Text widget with `repeating: true`.

## `--recovery-threshold` {#mnemonic-build-descriptor-recovery-threshold}

Recovery quorum threshold k. Range 1..20. The GUI renders this as a
Number widget.

## `--final-key` {#mnemonic-build-descriptor-final-key}

Last-resort key (`decaying-multisig` tier 3). The GUI renders this as a
Text widget.

## `--older` {#mnemonic-build-descriptor-older}

Relative timelock (blocks) gating the recovery path
(`decaying-multisig`: the tier-1 timelock). Only the low 16 bits carry
the value and bit 22 selects 512-second units, so the accepted domain
is `1..=65535` (blocks) or `0x400000|(1..=65535)` (512-second units);
any other bit set or a zero 16-bit value is rejected (consensus would
silently mask it). The GUI renders this as a Number widget.

## `--recovery-older` {#mnemonic-build-descriptor-recovery-older}

`decaying-multisig` tier-2 relative timelock; must be **greater than**
`--older` (tiers must unlock progressively later). The GUI renders this
as a Number widget.

## `--after` {#mnemonic-build-descriptor-after}

`decaying-multisig` tier-3 absolute locktime (block height, or unix
time past the BIP-65 threshold). Range `1 ≤ N ≤ 0x7fffffff`. The GUI
renders this as a Number widget.

## `--hash` {#mnemonic-build-descriptor-hash}

SHA-256 digest (64 hex chars) for `hashlock-gated`. The GUI renders
this as a Text widget.

## `--emit-spec` {#mnemonic-build-descriptor-emit-spec}

Boolean. Print the lowered + gate-validated node-tree spec JSON instead
of building — review it, edit it, feed it back via `--spec`. Conflicts
with `--format` / `--json`; `--network` is accepted and ignored. The
gate still runs: an invalid preset emits diagnostics, never a spec.
Records NO allowance in the spec document (replaying without `--allow`
correctly refuses). The GUI renders this as a Boolean toggle.

## `--allow` {#mnemonic-build-descriptor-allow}

Reviewed opt-out of ONE funds-safety sanity rule per occurrence
(repeatable) — a deliberate, reviewed act, never silent. Every rule
that **actually fired** is named in an unmissable stderr warning (all
output modes); a requested-but-not-fired allowance gets a `did not
fire` nudge. The cost preview is unavailable on a sanity-overridden
descriptor. The GUI renders this as a Dropdown with `repeating: true`
and a `?` help-icon.

### Outline {#mnemonic-build-descriptor-allow-outline}

- [`malleable`](#mnemonic-build-descriptor-allow-malleable)
- [`mixed-timelock`](#mnemonic-build-descriptor-allow-mixed-timelock)
- [`repeated-keys`](#mnemonic-build-descriptor-allow-repeated-keys)
- [`resource-limit`](#mnemonic-build-descriptor-allow-resource-limit)
- [`sigless-branch`](#mnemonic-build-descriptor-allow-sigless-branch)

### `malleable` {#mnemonic-build-descriptor-allow-malleable}

Waive the `malleable` sanity rule (a non-canonical witness path). Funds
remain spendable; the witness is just not malleation-resistant.

### `mixed-timelock` {#mnemonic-build-descriptor-allow-mixed-timelock}

Waive the `mixed_timelock` rule (an unspendable mixed height/time path
— the "wrong timelock loses money" guard). Only waive after confirming
the path is intentional.

### `repeated-keys` {#mnemonic-build-descriptor-allow-repeated-keys}

Waive the `repeated_keys` rule (the same key in two branches — e.g. a
"degrading threshold"). The BIP-388 output emits duplicate `keys_info`
entries (no dedup); hardware-signer behavior on duplicate keys is
signer-defined.

### `resource-limit` {#mnemonic-build-descriptor-allow-resource-limit}

Waive the `resource_limit` rule (a tree near the script resource
bounds). Confirm the spend witness fits consensus limits.

### `sigless-branch` {#mnemonic-build-descriptor-allow-sigless-branch}

Waive the `sigless_branch` rule (an anyone-can-spend path). **The most
dangerous opt-out** — a sigless branch means funds can be swept by any
party that can satisfy the non-signature conditions.

## `--format` {#mnemonic-build-descriptor-format}

Emit a single bare artifact instead of the rich human view. Dropdown;
two values. Omit `--format` for the human view (descriptor + first
receive address + cost table). Overridden by `--json`. The GUI renders
this flag with a `?` help-icon.

### Outline {#mnemonic-build-descriptor-format-outline}

- [`descriptor`](#mnemonic-build-descriptor-format-descriptor)
- [`bip388`](#mnemonic-build-descriptor-format-bip388)

### `descriptor` {#mnemonic-build-descriptor-format-descriptor}

The concrete `wsh(M)#checksum` descriptor string.

### `bip388` {#mnemonic-build-descriptor-format-bip388}

The BIP-388 wallet-policy JSON.

## `--network` {#mnemonic-build-descriptor-network}

Bitcoin network. Default `mainnet`. Used only for the human-view
first-receive-address rendering; the descriptor / bip388 / cost output
is network-agnostic (the xpubs carry the network). Dropdown; same 4
values as [`bundle --network`](#mnemonic-bundle-network). The GUI
renders this flag with a `?` help-icon.

### Outline {#mnemonic-build-descriptor-network-outline}

- [`mainnet`](#mnemonic-build-descriptor-network-mainnet)
- [`testnet`](#mnemonic-build-descriptor-network-testnet)
- [`signet`](#mnemonic-build-descriptor-network-signet)
- [`regtest`](#mnemonic-build-descriptor-network-regtest)

### `mainnet` {#mnemonic-build-descriptor-network-mainnet}

See [`bundle --network mainnet`](#mnemonic-bundle-network-mainnet).

### `testnet` {#mnemonic-build-descriptor-network-testnet}

See [`bundle --network testnet`](#mnemonic-bundle-network-testnet).

### `signet` {#mnemonic-build-descriptor-network-signet}

See [`bundle --network signet`](#mnemonic-bundle-network-signet).

### `regtest` {#mnemonic-build-descriptor-network-regtest}

See [`bundle --network regtest`](#mnemonic-bundle-network-regtest).

## `--json` {#mnemonic-build-descriptor-json}

Emit a structured JSON envelope `{descriptor, bip388, cost,
diagnostics}` for the GUI (the `cost` field is the embedded
`compare-cost --json` object). On a gate failure: `{diagnostics:
[{node_path, kind, message}]}` with exit 2; in preset mode each
diagnostic may additionally carry `flag`. The GUI renders this as a
Boolean toggle.

## Worked example — k-of-n recovery vault via an archetype preset

1. Switch to **mnemonic** tab; pick **Build Descriptor
   (vault policy)** in the subcommand selector.
2. Set `--archetype` to `kofn-recovery`.
3. Add three `--key` rows (the primary 2-of-3 quorum) and set
   `--threshold` to `2`. Add one `--recovery-key` row and set
   `--older` to `52560` (~1 year of blocks).
4. Set `--format` to `descriptor`. Click **Run** (no run-confirm modal
   — every field is public xpub material).

The output panel renders the concrete descriptor on stdout:

```text
wsh(or_d(multi(2,…/<0;1>/*,…),and_v(v:pk(…),older(52560))))#<checksum>
```

To review the generated tree before building, set `--emit-spec` instead
of `--format`; the lowered spec JSON prints to stdout (feed it back via
`--spec` to build).

## Refusals

| Trigger | Refusal |
|---|---|
| an `xprv` / WIF in a key node | exit 2 — watch-only screen; the key is never echoed |
| `--spec` AND `--archetype` | clap-level `conflicts_with` |
| a parameter not belonging to the chosen archetype, or a missing required one | `param`-kind diagnostic naming the flag (exit 2) |
| `--recovery-older` ≤ `--older` | `param`-kind diagnostic (tiers must unlock progressively later) |
| a fired sanity rule without a matching `--allow` | node-addressed diagnostic naming the rule + token (exit 2) |
| `--older` with any disallowed BIP-68 bit set or a zero value | gate refusal (consensus would silently weaken the timelock) |
| a tree exceeding the build-time complexity cap | refused so the cost preview always renders |

## Advisories

build-descriptor is watch-only by construction — no flag carries secret
material, so there are no argv-leakage advisories. When `--allow`
waives a rule that actually fired, an unmissable stderr warning names
the overridden rule; `--json` adds `"allowed_rules_fired": [...]` to
the success envelope.

## See also

- [`mnemonic build-descriptor` (CLI manual)](#mnemonic-build-descriptor)
  — the full node-tree grammar, the archetype parameter tables, the
  validation gate stages, and the reviewed sanity opt-out semantics.
- [`mnemonic export-wallet`](#mnemonic-export-wallet) — the descriptor
  build-descriptor emits round-trips via `export-wallet --descriptor
  <D> --format bip388`.
