# `mnemonic` vs `ms-cli` for secret cards

For ms1 (the secret card alone), both `mnemonic` and `ms` will do
the job. When does each fit?

## Use `ms` (the standalone CLI) when

- You want a *single-card* round-trip (encode a phrase to ms1, or
  decode an ms1 to a phrase) with no toolkit dependency.
- You're integrating ms1 into a non-toolkit pipeline — a backup
  script, a hardware-wallet firmware build, a third-party recovery
  tool.
- You want a debug-friendly inspect view (`ms inspect`) of the
  raw codex32 fields. The toolkit doesn't expose this.
- You need the canonical SHA-pinned v0.1 test-vector corpus
  (`ms vectors --pretty`).

## Use `mnemonic` (the toolkit) when

- You need ms1 alongside mk1 and md1 in a coherent bundle.
- You want the cross-binding `policy_id_stub` verification across
  the three cards.
- You want `mnemonic verify-bundle` to assert the ms1 entropy
  round-trips against the rest of the bundle.
- You want Bitcoin Core / BIP-388 / Sparrow / Specter wallet-export
  artifacts (`mnemonic export-wallet`).
- You want BIP-85 child derivations (`mnemonic derive-child`).
- You want the unified `--slot @N.<subkey>=<value>` input shape that
  composes with multisig.

## Side-by-side

| Capability | `ms encode` / `ms decode` | `mnemonic bundle` / `mnemonic convert` |
|---|---|---|
| BIP-39 phrase ↔ ms1 | yes | yes |
| Hex entropy ↔ ms1 | yes | yes (via `--slot @0.entropy=…`) |
| 10 BIP-39 wordlists | yes | yes |
| BCH error-position diagnostic | yes (decode + inspect) | yes (verify-bundle) |
| Cross-binding to mk1 + md1 | no | yes |
| Multisig (multi-source ms1) | no | yes |
| Watch-only wallet export | no | yes |
| BIP-85 derivation | no | yes |
| Stable API for non-toolkit pipelines | yes (smaller dep) | toolkit pulls in many crates |

## Practical takeaway

`ms` is the right tool for an ms1-only pipeline (e.g., a paper
wallet generator that only emits the secret card). `mnemonic` is
the right tool for end-user workflows that touch the bundle as a
whole.
