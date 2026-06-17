# cycle-prep recon — 2026-06-03 — all-single-sig-batch-emit (addresses + export-wallet)

**Origin/master SHA at recon time:** `eec0cb2`
**Local branch:** `master`
**Sync state:** up-to-date (0 ahead / 0 behind)
**Untracked:** pre-existing scratch only (`cycle-prep-recon-*.md`, `CONTINUITY.md`, `.claude/`, `feature-coverage-survey-*.md`, `stderr*.txt`).

Slug verified: **none exists** — NET-NEW feature (proposed slug `all-single-sig-batch-emit`). Feature recon, not citation re-verification. **Headline:** the two scoped halves differ sharply in difficulty — `addresses --all-script-types` is a near-mechanical port of an existing proven pattern; `export-wallet --all-single-sig` has a genuine per-format-artifact design fork.

---

## Feature recon — emit all 4 BIP single-sig types at once on `addresses` + `export-wallet`

User scope decision: **`addresses` + `export-wallet`** (bundle DESCOPED). The 4 BIP single-sig types: bip44/P2PKH · bip49/P2SH-P2WPKH · bip84/P2WPKH · bip86/P2TR.

### (A) `mnemonic addresses --all-script-types` — SMALL, low-risk (proven pattern exists)

- **`crates/mnemonic-toolkit/src/cmd/addresses.rs:35-36`** — `--address-type` is a REQUIRED single `ScriptType` (`value_parser = parse_script_type_arg`). — ACCURATE.
- **`ScriptType` → `CliTemplate` map at `addresses.rs:96-100`** — `P2pkh→Bip44, P2wpkh→Bip84, P2shP2wpkh→Bip49, P2tr→Bip86` (`template_for`). — ACCURATE. This is the per-type account-xpub derivation pivot.
- **Derive+emit path (`addresses.rs:~219-256`):** decode the `--from` source → `account_xpub` at `template_for(address_type)` → `for chain in chain.chains() { for index in indices { render_address_from_xpub(child, address_type, network) } }` → `emit_json(node, address_type, network, account_field, rows)` / `emit_text`. The `--json` row carries `"address_type": addr_type.as_str()` (`:348`). — ACCURATE.
- **THE PRECEDENT — `mnemonic nostr` ALREADY has `--all-script-types` (`cmd/nostr.rs:82-88,143`):** `--script-type` (single, `conflicts_with = "all_script_types"`) + `--all-script-types: bool`; `let types: Vec<ScriptType> = if args.all_script_types { <all 4> } else { vec![args.script_type] };` then loops. The GUI schema already mirrors this flag (`mnemonic-gui/src/schema/mnemonic.rs:2720`). — ACCURATE + load-bearing: the addresses `--all-script-types` is a near-verbatim port of this idiom (same flag name, same `conflicts_with`, same `Vec<ScriptType>` loop), so the **brainstorm/SPEC can cite `nostr.rs:82-88,143` as the reference implementation**.
- **Design for addresses (mechanical):** add `--all-script-types: bool` (`conflicts_with = "address_type"`); make `--address-type` non-required when `--all-script-types` set (clap: drop the required-ness or use a group). Loop the existing decode→template_for→render path over the 4 `ScriptType`s. Output: text = 4 labeled per-type sections; `--json` = an array of the existing per-type objects (each already keyed by `address_type`) — the row shape is unchanged, just repeated ×4. Shared-seed insight (below) applies: decode the source ONCE, re-derive the account xpub per type.
- **Action:** model on `nostr.rs:82-88,143`; cite SHA `eec0cb2`.

### (B) `mnemonic export-wallet --all-single-sig` — MEDIUM, has a per-format design fork

- **`export_wallet.rs:62-63`** — `--template: Option<CliTemplate>` (`conflicts_with = "descriptor"`); `CliTemplate` = {bip44/49/84/86 + multisig}. — ACCURATE. (Optional, because `--descriptor` is the alternative source.)
- **`export_wallet.rs:119-120`** — `--format: CliExportFormat` (default `bitcoin-core`). `CliExportFormat` (`:22-41`) = {bitcoin-core, bip388, coldcard, coldcard-multisig, jade, sparrow, specter, electrum, green, bsms}. — ACCURATE.
- **Per-format single-artifact emit (`export_wallet.rs:432,499-510` + `wallet_export/*.rs`):** `run()` resolves ONE descriptor (from `--template` or `--descriptor`) and dispatches to ONE per-format `Emitter` (BitcoinCoreEmitter / Bip388Emitter / ColdcardEmitter / …) producing ONE artifact to `--output` (default `-`). `format_bitcoin_core_importdescriptors` (`wallet_export/bitcoin_core.rs:42`) builds an `importdescriptors` JSON **array** via `into_single_descriptors()` (receive+change). — ACCURATE.
- **THE FORK (brainstorm must resolve):** "all 4 single-sig in one run" collides with the single-artifact-per-format model:
  - **bitcoin-core** (`importdescriptors`) is an ARRAY → naturally holds all 4 types (4×{receive,change} = 8 entries) in ONE valid artifact. Clean.
  - **bip388** wallet_policy — one policy per descriptor; "all 4" would be 4 policies (an array? bip388 is typically one policy). Needs a decision.
  - **single-wallet-file formats** (coldcard, sparrow, specter, electrum, green, jade) — each is ONE wallet config file by nature; 4 single-sig wallets do NOT fit one file. "All 4" here means 4 separate artifacts → requires a multi-output story (`--output` is a single path/`-`): 4 files (dir semantics?), or concatenated streams, or refuse.
  - **coldcard-multisig / bsms** are MULTISIG formats — irrelevant to single-sig "all 4" (should be refused under `--all-single-sig`).
- **Recommended export-wallet scoping (for the brainstorm):** restrict `--all-single-sig` to the array-capable formats — **bitcoin-core (definitely), bip388 (if a 4-policy array is acceptable)** — and REFUSE it for the single-wallet-file + multisig formats with a clear error ("…--all-single-sig emits 4 descriptors; use --format bitcoin-core, or run per --template for single-wallet formats"). This keeps the cycle bounded; a later FOLLOWUP can add multi-file output for the single-wallet formats if demanded.
- **Action:** the SPEC must enumerate the per-format behavior table + the refuse-set; cite `export_wallet.rs:62-63,119-120,432,499-510`, `wallet_export/bitcoin_core.rs:42`, `CliExportFormat` `:22-41`, SHA `eec0cb2`.

### Shared-secret insight (both halves)
Across the 4 types the BIP-39 entropy/seed is IDENTICAL; only the account xpub + descriptor differ per purpose (44'/49'/84'/86'). So decode the `--from`/seed source ONCE, then re-derive the per-type account xpub. (For `addresses` this is the loop body; for `export-wallet` it's 4 descriptors from one resolved seed/root.)

---

## Cross-cutting observations

1. **No slug exists** — net-new; no citation drift. File `all-single-sig-batch-emit` if not implemented immediately.
2. **`addresses` is de-risked by the `nostr --all-script-types` precedent** — same flag name, conflict semantics, and `Vec<ScriptType>` loop already shipped (nostr, v0.34.x) + already in the GUI schema. The addresses half is a port, not a design.
3. **`export-wallet` is the real work** — the per-format artifact model forces a scoping decision (array-capable formats vs single-wallet formats). This asymmetry suggests a possible split (see scope).
4. **Lockstep is now GUARDED** — this is a clap flag change → `schema_mirror` (GUI `ADDRESSES_FLAGS` gains `--all-script-types`; the export-wallet flag-set gains `--all-single-sig`) + manual `41-mnemonic.md` + GUI picker. The GUI is freshly pinned at toolkit v0.41.0 (this session, v0.22.0) and the new `tests/pin_coherence.rs` guard enforces the Cargo/pinned-upstream lockstep — so a paired GUI cycle (GUI v0.23.0, pin → the new toolkit version) lands the schema flags; the lagging schema_mirror gate fires at that pin bump.
5. **`--all-script-types` naming consistency:** `addresses` should reuse `--all-script-types` (matches `nostr`). For `export-wallet`, "single-sig" is template-framed (bip44/49/84/86), so `--all-single-sig` reads better than `--all-script-types` — but the brainstorm should weigh naming consistency (both are "the 4 single-sig types").

---

## Recommended brainstorm-session scope

- **SemVer:** new flags on existing subcommands (additive). Per the cycle-prep rule that's PATCH, but it's a user-facing batch capability → recommend **MINOR (toolkit v0.42.0)**; PATCH (v0.41.1) is defensible. Brainstorm/architect to confirm. (The `nostr --all-script-types` precedent shipped within a feature MINOR.)
- **Decomposition — two viable shapes; brainstorm to pick:**
  - **(a) One cycle, both commands, export-wallet RESTRICTED:** `addresses --all-script-types` (mechanical, mirror nostr) + `export-wallet --all-single-sig` limited to bitcoin-core (+ maybe bip388) with a refuse-set for single-wallet/multisig formats. Bounded; ~150-250 LOC + tests. **Recommended** — delivers both halves the user asked for without the multi-file rabbit hole.
  - **(b) Split:** ship `addresses --all-script-types` first (small, clean), then `export-wallet` as its own cycle (where the per-format multi-artifact question gets full treatment). Lower risk per cycle; two lockstep rounds.
- **Lockstep (mandatory):** GUI `schema_mirror` (ADDRESSES_FLAGS + export-wallet flags — paired GUI v0.23.0 pinning the new toolkit tag), manual `docs/manual/src/40-cli-reference/41-mnemonic.md`, GUI dropdown/flag. No sibling-codec change.
- **Reference implementations to cite:** `cmd/nostr.rs:82-88,143` (the `--all-script-types` idiom), `cmd/addresses.rs:96-100` (`template_for`), `wallet_export/bitcoin_core.rs:42` (the array-capable emit). Source SHA `eec0cb2`.
- Mandatory opus R0 on the brainstorm spec + plan + per-phase + end-of-cycle (0C/0I before code; re-dispatch after every fold). The brainstorm's first decision is the export-wallet per-format behavior table (the only genuinely-open design question; addresses is settled by the nostr precedent).
