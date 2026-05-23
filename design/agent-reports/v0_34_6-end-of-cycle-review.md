# v0.34.6 — End-of-cycle architect review (opus) — MANDATORY pre-tag gate (toolkit side)

**Date:** 2026-05-22
**Cycle:** mnemonic-toolkit v0.34.6 — `import-wallet --network` signet/regtest disambiguation
**Branch:** `v0.34.6-signet-network-override`
**Reviewer:** opus (feature-dev:code-reviewer), end-of-cycle gate
**Scope reviewed:** full toolkit cycle diff `/tmp/v0_34_6_cycle.diff` (commits `c3544bf`..`c42451c`) + live source

---

## Critical
(none)

## Important
(none)

## Minor (both non-actionable observations — no fix required)

- **M1 — `parsed.first()` empty-Vec guard silently no-ops the override** (`import_wallet.rs:1151`). Unreachable — every parser arm returns a non-empty `Vec<ParsedImport>` or propagates a parse error first. No fix.
- **M2 — `parsed_coin_type` read from `parsed.first()` assuming cosigner homogeneity** (`import_wallet.rs:1152`). Correct given the parser invariant (a single blob is single-network by construction); rebind-to-all matches. Documented assumption, not a defect.

---

## Verification summary (all confirmed correct)

1. **Override + guard** (`import_wallet.rs:1146-1165`): runs only when `args.network` is `Some`; `parsed_coin_type` derived Bitcoin→0 else→1 (parser yields only Bitcoin/Testnet across all 8 arms); guard `override_net.coin_type() != parsed_coin_type` refuses cross-class; rebind applies to ALL cosigners via `iter_mut`. Placed BEFORE the select-descriptor shadow + emit (`:1470`) so it propagates. Standalone Round-1 path returns before the parse → override never runs there.
2. **6 tests non-vacuous**: JSON path `v[0]["bundle"]["network"]` matches assembly; fixtures verified (`core-testnet-bip84.json` tpub/84'/1' = Testnet; `core-bip84-mainnet.json` xpub/84'/0' = Bitcoin). All distinct behaviors exercised; all passing.
3. **Error variant + 3 arms** (`error.rs`): `ImportWalletNetworkClassMismatch { requested, parsed_coin_type }` alphabetically placed in all three blocks; exit_code=1; kind matches; message via `format!` references "coin-type" (test assertion) + non-leaky (network name + integer only). Constructed at override site → no dead-code.
4. **`CliNetwork::to_bitcoin_network`** (`network.rs:59-66`): all 4 mapped correctly.
5. **No over-redaction / semantic break**: `network_human_name` emits "signet"/"regtest" (not "unknown"); `--network` is NON-secret; absent-flag path byte-identical.
6. **Manual + cspell**: `--network` row in import-wallet flag table; `bcrt` added to `.cspell.json`; lint 6/6.
7. **Version consistency**: Cargo.toml=0.34.6, Cargo.lock=0.34.6, install.sh self-pin=v0.34.6, CHANGELOG=[0.34.6]. Aligned.
8. **FOLLOWUP closure** accurate. **OUTSTANDING PAIRED OBLIGATION:** toolkit `gui-schema` now emits `--network` on import-wallet → the GUI `schema_mirror` test will fail until the paired Task 5 lands `--network` (Dropdown `NETWORKS`) in `mnemonic-gui/src/schema/mnemonic.rs` + pin bump v0.34.2→v0.34.6. MUST ship as the paired GUI step.
9. **Scope discipline**: only feature + error + helper + tests + manual + cspell + version + docs; alphabetical ToolkitError ordering preserved; clippy `--all-targets -D warnings` clean.

VERDICT: GREEN (0C/0I)

---

## Fold disposition (controller)
GREEN (0C/0I) → gate satisfied. No folds (M1/M2 are non-actionable observations with "no fix required"). Toolkit side ready to ship (GATED — cadence requires user go-ahead for cycle 3); the paired GUI schema-mirror lockstep (Task 5) follows the toolkit tag.
