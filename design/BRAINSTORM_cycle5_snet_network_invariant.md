# BRAINSTORM — cycle-5 "S-NET" network-provenance invariant (DESIGN ONLY)

**Status:** brainstorm spec, decision-complete, R0-ready. NO code, NO implementation.
**Workstream:** S-NET (Tier 1, `design/PLAN_constellation_bughunt_fix_program.md` §2 + §4).
**Closes:** H15, M13, M14, H9, L1, L2, L3, L10, L11 (9 findings). Toolkit-only, MINOR, no registry publish.
**Next gate:** mandatory opus-architect **R0 loop → 0 Critical / 0 Important** BEFORE any implementation
(per CLAUDE.md "MANDATORY pre-implementation R0 gate"). This doc is the input to that loop.

---

## 0. Source-SHA table (all citations re-grepped against `origin/master` at write time)

| Anchor | File:symbol | origin/master line(s) | SHA |
|---|---|---|---|
| Dead error variant | `src/error.rs` `NetworkMismatch { xpub_network: &'static str, expected: &'static str }` | `273-277` (variant), `587` (exit_code→2), `656` (kind), `830/913/1013` (Display/json/test) | `ac4eead0` |
| Precedent predicate | `src/synthesize.rs` `CosignerSpec` network cross-check | `776-790` | `ac4eead0` |
| Helper home (module doc states §4.3 contract) | `src/network.rs` (`CliNetwork`, `network_kind()`, `human_name()`, `coin_type()`) | doc `1-4`; `network_kind` `30-35`; `human_name` `49-56` | `ac4eead0` |
| Shared decode chokepoint | `src/wallet_import/pipeline.rs` `finalize_slot_fields` | `109-123` | `ac4eead0` |
| H9 guard/rebind asymmetry | `src/cmd/import_wallet.rs` `--network` override block | `1191-1209` (guard on `parsed.first()`, write on `iter_mut()`) | `ac4eead0` |
| H9 per-entry emit | `src/cmd/import_wallet.rs` `network: network_human_name(p.network)` | `1544` | `ac4eead0` |
| Import: descriptor | `src/wallet_import/descriptor.rs` `network_from_origins` / `coin_type_from_path` / `build_slot_fields` | `168` / `199` / `152-163` | `ac4eead0` |
| Import: specter | `src/wallet_import/specter.rs` `network_from_origins` | `370` | `ac4eead0` |
| Import: sparrow | `src/wallet_import/sparrow.rs` `network_from_origins` | `591` | `ac4eead0` |
| Import: bitcoin-core | `src/wallet_import/bitcoin_core.rs` cross-cosigner coin-type + `match first` | `~440-452` | `ac4eead0` |
| Import: bsms (=L10) | `src/wallet_import/bsms.rs` `network_from_origins`/`coin_type_from_path` | `~386` | `ac4eead0` |
| Import: coldcard-multisig | `src/wallet_import/coldcard_multisig.rs` `network_from_path`/`coin_type_index` | `679` / `693` | `ac4eead0` |
| Import: electrum multisig (=L2) | `src/wallet_import/electrum.rs` `build_multisig_descriptor`, coin→net | `660`, `~711` | `ac4eead0` |
| Electrum single-sig predicate to mirror | `src/wallet_import/electrum.rs` `network_from_xpub_neutral` (`xpub`→Bitcoin / `tpub`→Testnet) | `886-896`, used `555` | `ac4eead0` |
| Export: from-import-json (=M13) | `src/cmd/export_wallet.rs` `run_from_import_json`, `cli_network_from_str(&envelope.bundle.network)` | `678`, `742` | `ac4eead0` |
| SLIP-132 re-emit overwrites version bytes | `src/slip0132.rs` `apply_xpub_prefix` / `swap_target_for` | `108-111` / `197-211` | `ac4eead0` |
| Convert: xpub-prefix (=M14) | `src/cmd/convert.rs` xpub-prefix arm; presence-only guard | `1100-1113`; `922-924` (`refusal_xpub_prefix_no_network`) | `ac4eead0` |
| Convert: wif→xpub (=L11) | `src/cmd/convert.rs` `Wif` arm; `network = args.network.unwrap_or(Mainnet)` | `1480-1491`; `1217` | `ac4eead0` |
| Build-descriptor display (=L1) | `src/cmd/build_descriptor.rs` `emit_human`, `args.network.unwrap_or(Mainnet)` → `derive_receive_addresses` | `470`, `476`, `480` | `ac4eead0` |
| L3 truncation | `src/wallet_import/coldcard.rs` `raw_account = … as u32`; legacy fmt | `237-241`; `266` | `ac4eead0` |
| Oracle harness | `tests/bitcoind_differential.rs` (CONNECT-ONLY, `#[ignore]`, `-chain=main` v27.0) | whole file | `ac4eead0` |

**origin/master tip = `ac4eead0`; toolkit version on origin/master = `0.62.1`.** Cycle-5 builds atop `ac4eead0`.

---

## 1. Finding summary — all 9 REPRODUCE on `ac4eead0`

| ID | Class (post-oracle) | One-line | Disposition |
|---|---|---|---|
| **H9** | A — legit-input wrong-label (highest priority) | `import-wallet --network` class-checks `parsed.first()` only, then `iter_mut()` rebinds **every** entry's `network`; a heterogeneous `[Bitcoin, Testnet]` Vec whose `first()` matches the override passes and all entries are silently relabeled. | **REJECT** cross-entry heterogeneity — **extend the `first()`-only class-check to ALL entries, reusing `ImportWalletNetworkClassMismatch` (exit 1)**; see §2.3.1 |
| **H15** | MED — corrupt-input-only (structural anchor) | 7 import parsers derive network from the BIP-48 coin-type child; `normalize_xpub_prefix` neutralizes the prefix but `Xpub.network` is never compared to the coin-type network → wrong-network accept on hand-edited blobs. | **REJECT** |
| **M13** | MED | `export-wallet --from-import-json` trusts the envelope `network` string; only the BIP-380 checksum is validated, not the descriptor's xpub network → mainnet-labeled file with testnet keys, then `apply_xpub_prefix` re-emits wrong version bytes. | **REJECT** |
| **M14** | MED | `convert --xpub-prefix` re-emits in the `--network` family; the only guard requires `--network` **present**, not **agreeing** with the xpub's own network. | **REJECT** |
| **L1** | LOW — DISPLAY-only | `build-descriptor` human view derives the preview address with `args.network.unwrap_or(Mainnet)`; canonical/bip388 deliverables are network-agnostic & correct, only the printed label/HRP is wrong. | **WARN/DIAGNOSE** |
| **L2** | LOW (∈H15 set) | Electrum **multisig** decides network from coin-type only; the **single-sig** path uses `network_from_xpub_neutral` (xpub-derived) — the two disagree. | **REJECT** (align multi with single) |
| **L3** | LOW — metadata-only (NOT a network bug) | Coldcard single-sig `account as u32` truncates a `>u32::MAX` JSON value → wrong **origin path** annotation; addresses correct. | **SPLIT-OUT ride-along** |
| **L10** | LOW (∈H15 set) | BSMS network from coin-type only, never cross-checked vs cosigner xpub version bytes. | **REJECT** |
| **L11** | LOW | `convert --from wif --to xpub` sets the sentinel xpub network from `--network` (default mainnet), discarding the WIF's embedded `pk.network`. | **REJECT** (cross-check `pk.network`) |

**Empirical anchor (H15):** a mainnet xpub (`0488b21e`) on a `84'/1'/0'` path imports as `network=testnet` and renders `tb1q…` instead of `bc1q…`. The diff-oracle rated legit network-consistent round-trips a clean negative → H15 is corrupt-input-only (MEDIUM); H9 is the one legit-input member (worst case `[mainnet,testnet] + --network mainnet` presents a testnet descriptor as mainnet, inviting funds to addresses with no spendable mainnet key).

---

## 2. The shared invariant — design

### 2.1 The one rule

> **A decoded xpub's NetworkKind (Main vs Test) MUST agree with the asserted network / coin-type. Otherwise REJECT (fail-closed).**

Granularity is **`bitcoin::NetworkKind` (Main / Test), NOT 4-way** (mainnet/testnet/signet/regtest). This is the correct and only feasible granularity: **BIP-32 xpub version bytes encode only two families** — `0488b21e` (`xpub`, mainnet → `Main`) and `043587cf` (`tpub`, all of testnet/signet/regtest → `Test`). A signet or regtest descriptor is carried by a `tpub`; there is no signet-specific or regtest-specific version byte. So `tpub` covering all non-mainnet is exactly `NetworkKind::Test`, and `CliNetwork::network_kind()` already collapses `Testnet|Signet|Regtest → Test` (`network.rs:30-35`, asserted by `network_kind_mainnet_vs_test` test). The `CosignerSpec` precedent (`synthesize.rs:776-790`) cross-checks at exactly this `NetworkKind` granularity. **Confirmed: NetworkKind-granular matches both Bitcoin's xpub version semantics and the in-repo precedent.** (Coin-type 1 likewise covers testnet/signet/regtest, so coin-type→NetworkKind and xpub→NetworkKind are the same 2-way partition — they are directly comparable.)

### 2.2 The shared helper

**Decision (recommended lean):** ONE fail-closed helper in `src/network.rs` (the module whose doc already states the §4.3 "network/xpub cross-check via `Xpub::network`" contract). It takes the **already-extracted `NetworkKind`s**, not the `Xpub` — keeping it dependency-free, trivially unit-testable, and reusable by the WIF path (which has a `PrivateKey::network` not an `Xpub`). Callers extract `xpub.network` (a `NetworkKind`) at the site.

```rust
// src/network.rs — fulfils the module's already-stated §4.3 contract.
// PRECONDITION (caller-side): only invoke once an `asserted` network exists.
// Originless / no-coin-type inputs have NO asserted side — the caller MUST skip
// the call entirely (preserves the current accept; see the no-op precondition
// note above). The helper itself is unconditional: given two NetworkKinds it
// compares them; the skip-when-no-asserted-network discipline lives at the call site.
pub(crate) fn assert_network_agrees(
    decoded: bitcoin::NetworkKind,      // the artifact's own network (xpub.network OR pk.network)
    asserted: bitcoin::NetworkKind,     // coin-type-derived OR --network-derived
    context: NetworkContext,            // small enum identifying the site for the message
) -> Result<(), ToolkitError> {
    if decoded != asserted {
        return Err(ToolkitError::NetworkMismatch {
            decoded_network: network_kind_name(decoded),   // "mainnet" | "testnet"
            expected_network: network_kind_name(asserted),
            context,                                        // see §2.3 field-type decision
        });
    }
    Ok(())
}
```

- **Takes `NetworkKind`, not `Xpub`** — recommended. Reasons: (a) the WIF case (L11) carries `pk.network: NetworkKind`, no xpub; one helper serves both. (b) Import parsers already hold both the coin-type-derived `bitcoin::Network` and the decoded `Xpub`; mapping `Xpub::network` (already `NetworkKind`) and `bitcoin::Network → NetworkKind` (via `NetworkKind::from(network)`) at the call site is one line each. (c) No `Xpub` clone / decode coupling in the helper.
- **`bitcoin::Network → NetworkKind`** at import sites: `bitcoin::NetworkKind::from(net)` (mainnet→Main; testnet/signet/regtest→Test). Verified this exists in the `bitcoin` crate (`From<Network> for NetworkKind`). At convert/build sites the asserted side is a `CliNetwork`, use the existing `CliNetwork::network_kind()`.
- **`network_kind_name(NetworkKind) -> &'static str`** — a new tiny const-fn in `network.rs`: `Main => "mainnet"`, `Test => "testnet"`. Static, so it satisfies the `&'static str` field type for the two network-name fields (see §2.3). It deliberately renders all non-mainnet as `"testnet"` because the cross-check cannot distinguish signet/regtest from testnet — and saying "testnet" for a `tpub` is correct at the NetworkKind layer. (The error message wording in §2.3 makes the family-collapse explicit so a signet user is not confused.)

**Precondition — the helper (and every new cross-check) is a NO-OP when there is NO asserted network.** The cross-check has two sides: the decoded artifact network and the *asserted* network. **When the asserted side is absent, the check MUST be skipped entirely (not errored).** The reachable absent-asserted case is an **originless / no-coin-type input**: a bare `tpub`/`xpub` descriptor with no `[fp/path]` origin (or an origin shorter than 2 path components) has no coin-type to derive — `coin_type_from_path` requires ≥2 components (e.g. `cli_descriptor_concrete.rs:174` carries `wpkh(tpubD…/0/*)` with no origin). Such inputs are accepted today; the cross-check MUST preserve that accept (skip the assertion), else we introduce a NEW availability / over-reject bug — exactly the funds-hole class the L1 WARN-not-reject disposition is careful to avoid. This is an **explicit precondition of `assert_network_agrees`'s callers** (the helper is only invoked once a coin-type-derived network exists; the per-parser call site guards on "did we derive a coin-type network?" before calling). It is scoped to the import parsers and M13 (export-from-import-json); M14/L11/build-L1 always have an asserted side (`--network` / `pk.network`) and are unaffected. A positive control (§7) proves an originless `tpub` descriptor still imports/parses unchanged.

**Insertion strategy per parser (lower-churn lean):** do **NOT** thread the expected network through `pipeline::finalize_slot_fields` (that touches every caller's signature and the chokepoint deliberately has no asserted network). Instead, each parser already computes its coin-type-derived network (via `network_from_origins` / `network_from_path` / `coin_type_*`) and already holds the decoded slot `Xpub`s (returned by `finalize_slot_fields`). **Call `assert_network_agrees` once per parser, iterating the resolved slots' `xpub.network` against the coin-type-derived network — but ONLY when a coin-type network was actually derivable** (per the no-op precondition above; originless / sub-2-component-origin inputs skip the call), immediately after the coin-type network is resolved. This localizes the new edit to each parser's existing network-resolution function and leaves the shared chokepoint untouched.

### 2.3 The `NetworkMismatch` variant — field-type decision

**Today (`error.rs:273-277`):**
```rust
#[allow(dead_code)]
NetworkMismatch {
    xpub_network: &'static str,
    expected:     &'static str,
}
```
Zero construction sites (confirmed: `git grep ToolkitError::NetworkMismatch` returns only the def + the 5 match arms at `587/656/830/913/1013`). It is dead, declared exactly for this rule. `exit_code` already maps it to `2` (`:587`) and `kind` to `"NetworkMismatch"` (`:656`).

**The gotcha:** the fields are `&'static str`. Dynamic names (`format!("{:?}", xpub.network)`) are `String`. Two ways to wire it.

**DECISION (recommended): keep both network-name fields `&'static str`, fed by the new `network_kind_name()` const-fn; ADD one `context: NetworkContext` field (a `&'static str` site label).** Rationale: the cross-check only ever produces two network names ("mainnet" / "testnet") — both static. There is **no need to widen to `String`**; the const route (gotcha option (ii)) is cleaner, avoids an allocation per error, and keeps the variant cheap. The only dynamic-seeming thing is *which site* triggered, which is itself a closed set → a `&'static str` (or a tiny `enum NetworkContext`). Recommended exact shape:

```rust
#[allow(dead_code)]  // REMOVE this attribute when wired (Phase 1 emits it).
NetworkMismatch {
    decoded_network:  &'static str,   // "mainnet" | "testnet"  (was `xpub_network`)
    expected_network: &'static str,   // "mainnet" | "testnet"  (was `expected`)
    context:          &'static str,   // e.g. "import-wallet: specter cosigner xpub"
}
```
*(Rename `xpub_network`→`decoded_network` / `expected`→`expected_network` so the field names also cover the WIF case, where the "decoded" network is a WIF's, not an xpub's. Add `context` for actionable messages. **R0-RATIFIED (round 1, M-4): take the rename** — `&'static str` is confirmed sufficient (the cross-check only ever yields two static names, fed by `network_kind_name()`), and the rename genuinely covers the WIF case where `xpub_network` is a misnomer; cost is the three mechanical arm edits already enumerated (Display `:830`, `detail_json` `:913`, unit test `:1013`). This is no longer an open lean.)*

- **Display (`error.rs:830`)** updated to (keeping the family-collapse explicit):
  `format!("network mismatch: {context}: key encodes {decoded_network} but {expected_network} was asserted (xpub version bytes distinguish mainnet vs testnet/signet/regtest only)")`
  **(M-6, no-op note):** the variant is dead, so this Display rewrite changes no observed behavior — NO test asserts the old `"xpub network {} does not match --network {}"` string (the `:1013` unit test asserts only `exit_code()==2`, not Display). The wording change does not matter to any gate; recorded purely so the plan-doc isn't surprised.
- **`detail_json` (`:913`)** updated to emit `{"decoded_network","expected_network","context"}`.
- **`exit_code` (`:587`) stays `2`** — user-input / funds-safety reject class. **Verify at R0 it is currently unreachable** (it is — zero construction sites; the `:1013` unit test constructs the variant directly and asserts `exit_code()==2`, but no production path builds it). Update that unit test's field names if renamed.
- **Alphabetical ordering (CLAUDE.md):** `NetworkMismatch` already sits between `MultisigConfig` and `NostrKeyParse` — alphabetical. The field-type edit and the new `context` field do **not** move the variant; ordering is preserved. The `Display`/`detail_json`/`exit_code`/`kind` match arms keep their positions. **No re-sort needed.** (CLAUDE.md's alphabetical rule applies to *new* variants and new exhaustive match blocks; we add neither — we only edit the existing variant's fields, in place.)

### 2.3.1 Two coexisting exit codes in the import block — INTENTIONAL (the H9 axis vs the H15 axis)

The import path now carries **two distinct network checks on two different axes**, which deliberately use two different variants and exit codes:

- **Axis 1 — `--network`-flag-vs-blob-coin-type CLASS check (H9).** This asks: does the user-supplied `--network` override agree, **per entry**, with each parsed entry's coin-type class (coin-type-0 vs coin-type-1)? H9's fix is to **extend the existing `first()`-only class-check (`import_wallet.rs:1192`, which returns `ImportWalletNetworkClassMismatch` at `:1199`) to ALL parsed entries** — the exact same condition the adjacent sibling already refuses, just applied per-entry instead of `first()`-only. It therefore **reuses `ImportWalletNetworkClassMismatch` → exit 1** (`error.rs:576`). Using the same variant/exit as the immediately-adjacent sibling refusal is the consistent fix; minting a different variant/exit for "the same condition, one more entry" would be the inconsistency.
- **Axis 2 — xpub-version-bytes-vs-coin-type check (H15/M13/M14/L2/L10/L11 + convert-xpub-prefix + convert-wif + export-from-import-json).** This asks: does a **decoded xpub's own version-byte NetworkKind** agree with the coin-type-derived (or `--network`/`pk.network`-derived) asserted network? This is the formerly-dead **`NetworkMismatch` → exit 2** (`error.rs:587`), wired here for the first time. It is a *different question* from the class check: a blob can pass the `--network` class check yet still carry an xpub whose version bytes contradict its own coin-type path (the H15 hand-edited-blob case).

**Both refusals living in the import flow with different exit codes is intentional and documented**: exit 1 = "your `--network` flag disagrees with the blob's coin-type class" (`ImportWalletNetworkClassMismatch`, the H9 axis); exit 2 = "a decoded key's version bytes disagree with its asserted network" (`NetworkMismatch`, the H15 axis). They are two distinct conditions, not two spellings of one. The plan-doc and tests assert each axis against its own variant/exit (§7).

### 2.4 Precedent — keep `synthesize.rs` `CosignerSpec` as-is

The bundle synthesis path already enforces the rule via `CosignerSpec { cosigner_idx, message }` (`synthesize.rs:776-790`). **Do NOT churn it** — it works, is tested, and changing it to `NetworkMismatch` is pure risk for zero benefit. S-NET wires the dead `NetworkMismatch` for the import/export/convert/build sites (its intended home) and leaves the bundle path on `CosignerSpec`. The two coexist: `CosignerSpec` is cosigner-index-scoped (bundle context); `NetworkMismatch` is the generic decode-site reject.

---

## 3. Fix-site map — per-site disposition (reject vs warn), file:symbol:line

| # | Site | File:symbol (origin/master) | Asserted-network source today | Disposition | Helper call |
|---|---|---|---|---|---|
| 1 | import: descriptor | `wallet_import/descriptor.rs` `network_from_origins:168` (+ slots from `build_slot_fields:152`→`finalize_slot_fields`) | coin-type only | **REJECT** | per-slot `assert_network_agrees(slot.xpub.network, NetworkKind::from(coin_type_net), "import: descriptor")` |
| 2 | import: specter | `wallet_import/specter.rs` `network_from_origins:370` | coin-type only | **REJECT** | same idiom |
| 3 | import: sparrow | `wallet_import/sparrow.rs` `network_from_origins:591` | coin-type only | **REJECT** | same idiom |
| 4 | import: bitcoin-core | `wallet_import/bitcoin_core.rs` per-descriptor coin-type `~440-452` | coin-type only | **REJECT** | per-cosigner cross-check after coin-type resolves |
| 5 | import: bsms (=L10) | `wallet_import/bsms.rs` `network_from_origins:~386` | coin-type only | **REJECT** | same idiom |
| 6 | import: coldcard-multisig | `wallet_import/coldcard_multisig.rs` `network_from_path:679`/`coin_type_index:693` | coin-type only | **REJECT** | per-cosigner cross-check |
| 7 | import: electrum **multisig** (=L2) | `wallet_import/electrum.rs` `build_multisig_descriptor:660`, coin→net `~711` | coin-type (multi) vs xpub (single, `network_from_xpub_neutral:886`) — **disagree** | **REJECT**; mirror the single-sig `network_from_xpub_neutral` cross-check per cosigner so multi and single use the same predicate |
| H9 | import: **cross-entry** rebind | `cmd/import_wallet.rs` override block `1191-1209` (class-check `parsed.first()` at `:1192` → returns `ImportWalletNetworkClassMismatch` at `:1199`); emit `:1544` | `parsed.first()` coin-type for guard, `iter_mut()` rebind-all | **REJECT** (exit **1**, **reuse `ImportWalletNetworkClassMismatch`** — the adjacent sibling) | **Extend the existing `first()`-only class-check to ALL parsed entries**, reusing `ImportWalletNetworkClassMismatch`. Compute coin-type **per entry**; if ANY entry's coin-type class disagrees with the requested/resolved override network → return `ImportWalletNetworkClassMismatch` (exit 1). The guard and the rebind must read the SAME per-entry network (not `first()` for guard + all for write). The override may only rebind WITHIN a homogeneous coin-type class. **NOT** `NetworkMismatch`/exit 2 — see §2.3.1 for why H9 stays on exit 1. |
| 8 | shared chokepoint | `wallet_import/pipeline.rs` `finalize_slot_fields:109` | decodes `Xpub` (`.network` available); receives no asserted network | **NO new param** — leave untouched; cross-check at each parser (lower-churn lean §2.2) |
| 9 | export: from-import-json (=M13) | `cmd/export_wallet.rs` `run_from_import_json:678`; `cli_network_from_str(&envelope.bundle.network):742`; re-emit `src/slip0132.rs::apply_xpub_prefix:108` | envelope JSON `network` string | **REJECT** | decode each descriptor/card xpub; `assert_network_agrees(xpub.network, network.network_kind(), "export: from-import-json")` BEFORE any `apply_xpub_prefix` |
| 10 | convert: xpub-prefix (=M14) | `cmd/convert.rs` xpub-prefix arm `1100-1113`; presence-only guard `922-924` | `--network` (default Mainnet) | **REJECT** | inside the arm, after decoding each output `Xpub`: `assert_network_agrees(xpub.network, network.network_kind(), "convert: --xpub-prefix")` before `apply_xpub_prefix` |
| 11 | convert: wif→xpub (=L11) | `cmd/convert.rs` `Wif` arm `1480-1491`; `network` from `args.network.unwrap_or(Mainnet):1217` | `--network` (discards `pk.network`) | **REJECT** on disagreement (see §5) | `assert_network_agrees(pk.network, network.network_kind(), "convert: wif→xpub")` — generalized "asserted-source network" |
| 12 | build-descriptor human view (=L1) | `cmd/build_descriptor.rs` `emit_human:470`; `args.network.unwrap_or(Mainnet):476` → `derive_receive_addresses:480` | `--network` (default Mainnet) | **WARN/DIAGNOSE** (see §4) | NO helper-reject; infer display network from descriptor keys when `--network` omitted; emit a stderr WARN when supplied `--network` disagrees with the keys |
| L3 | import: coldcard single-sig account | `wallet_import/coldcard.rs:237-241` `as u32`; origin fmt `:266` | n/a — account-index, not network | **SPLIT-OUT ride-along** (see §5) | NOT a network check — a `u64→u32` bound check |

**Justification of dispositions:**
- **Hard-REJECT for 1-11 + H9:** these accept or re-emit a wrong-network artifact today. The toolkit's threat model is steel-engraved cold-storage backups — a wrong-network card silently directs funds to addresses with no spendable key on the asserted chain. Fail-closed is the program's §1 invariant and the report's explicit recommendation ("standardize on a fail-closed network rule"). The cost of a false-reject (a legitimately network-consistent blob refused) is **proven zero by a committed FULL-suite sweep** — a green `cargo test -p mnemonic-toolkit` (the WHOLE package, per the project's full-package-R0 discipline, MEMORY `feedback_r0_review_run_full_package_suite`) is the zero-false-reject proof, NOT a targeted-test claim. The existing fixtures are network-consistent (coin-type-1 paths pair with `tpub`, coin-type-0 with `xpub`), and the **originless / no-coin-type case is handled by the no-op precondition (§2.2)** so an originless `tpub` descriptor is NOT over-rejected. The per-site positive controls (§7) plus the full-suite green together prove consistent inputs pass unchanged.
- **WARN for L1 only:** `build-descriptor`'s canonical descriptor and bip388 outputs are **network-agnostic and byte-correct**; only the optional human-readable first-address preview label/HRP is affected. A hard-reject would refuse a perfectly valid descriptor over a *display preview*. Correct behavior: when `--network` is omitted, **infer** the display network from the descriptor's own xpub version bytes (so the preview is right by default); when `--network` is supplied and disagrees with the keys, **emit a stderr WARN** and still render (the user explicitly asked for that network's preview). This is the single disposition split R0 must ratify.

**Edit-cluster count:** 11 decode/assertion sites collapse to ~7-9 edit clusters: descriptor/specter/sparrow/bsms share the `network_from_origins` idiom; bitcoin-core and coldcard-multisig are per-cosigner variants; electrum is the outlier (mirror its own single-sig predicate); plus 3 convert/export + 1 build-descriptor WARN + H9's cross-entry gate + the helper + the variant edit. Estimate ~250-450 LOC incl. tests.

---

## 4. L1 — WARN/DIAGNOSE design (the one disposition split)

`emit_human` (`build_descriptor.rs:470`) derives the preview with `args.network.unwrap_or(Mainnet)`. New behavior:

1. Walk `vp.descriptor` keys (`for_each_key`-style over `&DescriptorPublicKey`) and infer a single display `NetworkKind`. `DescriptorPublicKey` is an enum — handle each variant:
   - **`XPub(DescriptorXKey<Xpub>)`** → read `.xkey.network` (a `NetworkKind`).
   - **`MultiXPub(DescriptorMultiXKey<Xpub>)`** → read `.xkey.network`.
   - **`Single(SinglePub)`** (a raw single pubkey leaf) → **no network is encoded** (a bare pubkey carries no version bytes); contributes **nothing** to the inference (skip it).
   Fold the network-bearing keys to a single `NetworkKind` (all xpub keys share a family by construction; if they somehow disagree that is a pre-existing malformed descriptor — out of S-NET scope, leave to existing parse errors). **If NO key encodes a network** (an all-`Single` descriptor) → inference yields "unknown"; **keep the `--network` default and emit NO warning** (there is nothing to disagree with — the L1 WARN is only meaningful when the keys assert a family).
2. **`--network` omitted:** if the keys inferred a `NetworkKind`, use it to pick the preview's `bitcoin::Network` (map Test→Testnet for HRP purposes; the preview HRP `tb1…`/`bc1…` is now correct by default), no warning. If inference yielded "unknown" (all-`Single`), fall back to the current `unwrap_or(Mainnet)` default (unchanged), no warning.
3. **`--network` supplied AND the keys encode a KNOWN network that disagrees:** print one stderr line, e.g.
   `warning: build-descriptor: --network mainnet disagrees with descriptor keys (testnet); preview shown for --network (deliverable descriptor is network-agnostic)`, then render the preview per `--network` as today (the user explicitly chose it). **Exit 0** — the deliverable is correct.
4. **`--network` supplied AND agrees:** unchanged.

This is the ONLY non-reject site. It must NOT call `assert_network_agrees` (that returns `Err`); it does an inline compare + WARN. Rationale recorded in §3.

---

## 5. L3 split-out + WIF (L11) special case

### 5.1 L3 — fold as a co-located ride-along (recommended)

L3 is a `u64→u32` account-index truncation (`coldcard.rs:237-241`, `raw_account = obj["account"].as_u64().map(|n| n as u32)`) that bakes a wrong **origin path**, **not** an xpub-network bug. **The truncation only MANIFESTS in the legacy top-level-xpub fallback branch** (`coldcard.rs:266`, the `deriv_path_str_opt == None` arm that formats `m/{purpose}'/{coin_type}'/{raw_account}'`). The per-bipN sub-object path uses the sub-object's own `deriv` string and never interpolates `raw_account`, so the RED test MUST target a **legacy top-level-xpub** Coldcard blob (cf. fixture `coldcard-mk1-legacy-bip84-mainnet.json`) — a per-bipN fixture would render the test VACUOUS (truncation unobservable). The diff-oracle confirms **addresses are correct**; only origin metadata is wrong. It shares the `coldcard.rs` file zone with no S-NET parser edit (the multisig variant is `coldcard_multisig.rs`; single-sig is `coldcard.rs`), so the "same file open" argument is weaker than the recon implies.

**DECISION: FOLD it as a ride-along, but as a SEPARATE sub-item with its own test — NOT a clause of `assert_network_agrees`.** Reasons: (a) it is cheap (a single bound check: reject — or, lean below, error — when `obj["account"].as_u64() > u32::MAX`); (b) it is in the same import-parser family the cycle is already touching and reviewing; (c) deferring it to a future "fidelity cycle" risks it being forgotten (metadata-only items are exactly the ones that rot in FOLLOWUPS). **Fix lean: REJECT (error) on `account > u32::MAX`, not saturate** — a backup tool must never silently rewrite a user's account index; an out-of-range account is corrupt input → `ImportWalletParse` (exit 2). If R0 prefers minimal blast radius, defer L3 to a fidelity cycle and file a FOLLOWUP; recommended is fold-with-its-own-test. Either way it is firewalled from the network helper.

### 5.2 L11 — WIF carries its own network byte; same helper, generalized

A WIF (`PrivateKey::from_wif`) carries `pk.network: NetworkKind` directly — not an xpub. The `Wif` arm (`convert.rs:1480-1491`) builds a sentinel xpub with `network: network.network_kind()` (from `--network`), discarding `pk.network`.

**DECISION: SAME helper, generalized to "asserted-source network".** `assert_network_agrees` already takes two `NetworkKind`s (§2.2) and is agnostic about whether the "decoded" side came from an xpub or a WIF. The `Wif` arm calls `assert_network_agrees(pk.network, network.network_kind(), "convert: wif→xpub")` and rejects on disagreement. This is why the variant fields are renamed `decoded_network`/`expected_network` (not `xpub_network`) in §2.3 — the helper is xpub-agnostic. No sibling check needed. (The sentinel already emits a "zeroed chain code; not BIP-32 derivable" warning at `:971-974`, so the blast radius was limited — but a testnet WIF emitting a mainnet `xpub…` is still a wrong-network artifact and warrants the reject; the user can pass `--network testnet` to get the `tpub…` they intended.)

---

## 6. SemVer + lockstep + oracle-gate

### 6.1 SemVer — toolkit MINOR `0.62.1 → 0.63.0`

New fail-closed rejections of previously-**accepted** corrupt input (H15/M13/M14/L2/L10/L11) and the H9 cross-entry refusal are observable behavior changes; pre-1.0, behavior-breaking ⇒ MINOR. L1 (WARN) and L3 (account reject) ride in the same MINOR. **Single MINOR cycle.**

**Version-collision note (recon-flagged):** the paused `feature/own-account-subset-search` cycle (halted at plan-R0-GREEN per MEMORY) also plans `0.63.0`. **First-to-ship claims `0.63.0`; the other renumbers.** S-NET should claim `0.63.0` when it ships; **do NOT touch the `feature/own-account-subset-search` branch** — leave its renumber to whoever ships second. Record as a coordination note only.

### 6.2 Lockstep / wire surface — NONE required (explicit)

Every fix is an internal `assert_network_agrees`-style reject (or an L1 stderr WARN, or an L3 bound check) inserted into existing decode paths. **No new clap flag, no new/renamed subcommand, no new dropdown value.** The only error-surface change is wiring the **already-declared** `NetworkMismatch` variant (a new error *variant* is not a clap surface).

**`--json` error-shape nuance (honest accounting):** wiring `NetworkMismatch` and renaming its fields DOES add/rename keys in that variant's `detail_json` error-wire output (`error.rs:913`: `"xpub_network"→"decoded_network"`, plus the new `"context"` key). So it is **not** true that there is "no `--json` wire change at all" — there is an error-envelope `--json` delta for this one formerly-dead variant. **However:** (a) this is NOT a `schema_mirror` trigger and NOT a manual trigger — those gates cover clap flag-NAMES + dropdown VALUES, not `--json` error-envelope shape (per CLAUDE.md); and (b) the variant was dead (zero construction sites), so **no existing consumer can currently observe these keys** → practical blast radius is zero. We state this explicitly rather than claiming a blanket "no wire-shape change." Therefore:

- **GUI `schema_mirror` (flag-NAME parity): NOT triggered.** Per CLAUDE.md, `schema_mirror` gates the clap **flag-name set + dropdown value enums**. S-NET adds/removes/renames none. (It does NOT gate `--json` wire-shape at all — so even the `NetworkMismatch.detail_json` key delta above is outside its scope.) → **No `mnemonic-gui/src/schema/mnemonic.rs` edit, no paired GUI PR.**
- **Manual `docs/manual/src/40-cli-reference/`: NOT triggered.** The mirror invariant fires on flag/option/subcommand add/remove/rename. None here. *(Optional, gate-NOT-mandated nicety: a one-line note that import/convert/export now refuse network-mismatched blobs. Recommend deferring even that to avoid scope creep — record as an optional FOLLOWUP.)*
- **Sibling-codec FOLLOWUP companions: NONE.** All sites are toolkit-internal; no md/mk/ms codec API touched. The network-family relatives **L8** (`canonical_origin.rs`) and **D-mdcli-coin** (`md-cli/src/parse/path.rs:25-34`) live in OTHER repos and are explicitly OUT of S-NET per the program plan — do NOT pull them in.

**Confirmed: toolkit MINOR `0.63.0`, zero lockstep, zero sibling publish.**

### 6.3 The Class-A differential-oracle gate

The program plan (§3 Phase 0) makes `tests/bitcoind_differential.rs` (AGREE/DISAGREE vs Core `deriveaddresses`) a HARD merge precondition for **address-affecting** changes. S-NET is class-A but **only ADDS rejections — it never changes a derived address for valid input.** The gate is satisfied as follows:

1. **Valid-input round-trips stay byte-identical AGREE.** Every existing shape in the harness (`wpkh`, `tr(NUMS,sortedmulti_a)`, etc.) is network-consistent (mainnet `xpub…`, `-chain=main`). The positive controls (§7) prove `assert_network_agrees` is a no-op for them — so the oracle's existing AGREE rows stay green, byte-for-byte. **This is the core gate claim: S-NET cannot regress any address the oracle covers, because it only rejects on inconsistent input the oracle never feeds.**
2. **The new rejects are on invalid/corrupt input the oracle doesn't cover.** A mismatched-network blob has no "correct address" for the oracle to compare — Core would reject a mainnet xpub on regtest anyway. So the DISAGREE corpus rows assert `exit≠0` from the toolkit (a CLI-exit assertion), not an address compare. These belong in the CLI/unit suites (§7), not the bitcoind oracle proper.
3. **Harness note:** `bitcoind_differential.rs` is CONNECT-ONLY, `#[ignore]`-by-default, `-chain=main` v27.0, env-gated (`MNEMONIC_BIN`/`BITCOINCLI_BIN`/`BITCOIND_DATADIR`/`BITCOIND_RPCPORT`). The implementation/review **runs it if a local bitcoind is feasible** (`cargo test -p mnemonic-toolkit --test bitcoind_differential -- --ignored --nocapture`); the gate is "existing AGREE rows remain green after S-NET." Because S-NET adds no new derived address, **no new oracle rows are strictly required** — but the plan invites a "network-provenance matrix" advisory corpus: a coin-type/xpub-AGREE row (passes, correct HRP) + a DISAGREE row (exit≠0). Those DISAGREE rows live in the CLI suite (the oracle can't derive a "correct" address for corrupt input); the AGREE rows are already covered by the existing mainnet shapes. **Recommended: add the DISAGREE-asserts to the CLI/unit suites (§7) and rely on the existing oracle AGREE rows for the no-regression proof — do not bloat the bitcoind harness with rows it structurally can't oracle.**

---

## 7. Tests (TDD, RED-first) — per finding + positive controls

Each finding gets (a) a RED test that fails today (mismatched-network input silently accepted/mislabeled) and passes after, and (b) a positive control proving a network-CONSISTENT input still succeeds unchanged (the funds-safety guard against over-rejection). The GREEN reject is **typed by axis** (§2.3.1): the xpub-version-vs-coin-type axis (H15/M13/M14/L2/L10/L11) asserts `NetworkMismatch` / **exit 2**; the `--network`-vs-coin-type-class axis (**H9**) asserts `ImportWalletNetworkClassMismatch` / **exit 1**; L3 asserts `ImportWalletParse` / exit 2 (account-range, not network); L1 is a WARN at exit 0.

| ID | RED test (fails today) | GREEN assertion | Positive control |
|---|---|---|---|
| **H15** | `import-wallet --format descriptor` with mainnet `xpub` on a `84'/1'/0'` path | exit 2, `kind=NetworkMismatch`, stderr names mainnet-vs-testnet | mainnet `xpub` on `84'/0'/0'` → exit 0, `network:"mainnet"`, `bc1q…` preview unchanged |
| **H9** | `import-wallet` of a 2-entry mixed blob `[Bitcoin, Testnet]` with `--network bitcoin` (first=Bitcoin passes the old `first()`-only check; the Testnet entry is now caught per-entry) | exit **1**, `kind=ImportWalletNetworkClassMismatch` — cross-entry heterogeneity refused by the per-entry class-check (axis 1, §2.3.1); NO silent relabel of the Testnet entry | **same-class** homogeneous `[Bitcoin, Bitcoin] + --network bitcoin` → exit 0, both `network:"mainnet"` (all entries same class, agrees with `--network` → the per-entry extension does NOT over-reject a homogeneous valid blob) |
| **M13** | `export-wallet --from-import-json` envelope `{"network":"mainnet", "descriptor":"wpkh([../84'/1'/0']tpub…)"}` | exit 2, `NetworkMismatch` BEFORE any `apply_xpub_prefix` re-emit | consistent envelope `{"network":"mainnet", …xpub…}` → exits 0, byte-identical output to today |
| **M14** | `convert --from xpub=<tpub> --to xpub --xpub-prefix zpub --network mainnet` | exit 2, `NetworkMismatch` | `--from xpub=<xpub> … --xpub-prefix zpub --network mainnet` → exit 0, valid mainnet `zpub` (unchanged) |
| **L1** | `build-descriptor` of a `tpub` descriptor with `--network mainnet` | exit **0** + stderr WARN line (deliverable unchanged); and with `--network` OMITTED the preview HRP is `tb1…` (inferred), not `bc1…` | `tpub` descriptor with `--network testnet` (agree) → exit 0, NO warning, `tb1…` preview; mainnet descriptor no `--network` → `bc1…` |
| **L2** | electrum **multisig** import with a `tpub` cosigner on coin-type-0 path | exit 2, `NetworkMismatch` (multi now matches single-sig behavior) | consistent electrum multisig (all `xpub` on coin-type-0) → exit 0, unchanged; and the existing single-sig consistent case still passes |
| **L3** | **legacy top-level-xpub** coldcard single-sig JSON (NO per-bipN `deriv` field, so `deriv_path_str_opt == None`, cf. `coldcard-mk1-legacy-bip84-mainnet.json`) with `"account": 4294967296` (`u32::MAX + 1`) — this is the ONLY branch where `raw_account` is interpolated into the origin (`coldcard.rs:266`, `format!("m/{purpose}'/{coin_type}'/{raw_account}'")`); a per-bipN blob uses the sub-object's own `deriv` and never touches `raw_account`, so a per-bipN RED test would be VACUOUS | exit 2, `ImportWalletParse` (account out of range) — NOT a wrapped origin | legacy top-level-xpub blob with `"account": 5` → exit 0, origin `m/84'/0'/5'` unchanged |
| **L10** | BSMS import with a `tpub` cosigner on coin-type-0 path | exit 2, `NetworkMismatch` | consistent BSMS round-trip → exit 0, unchanged |
| **L11** | `convert --from wif=<testnet WIF> --to xpub --network mainnet` | exit 2, `NetworkMismatch` (was: silent mainnet `xpub…`) | `--from wif=<testnet WIF> --to xpub --network testnet` → exit 0, `tpub…` sentinel (the WIF's true network), existing "not BIP-32 derivable" warning still present |

**Helper unit tests (`network.rs`):** `assert_network_agrees(Main, Main)`→Ok; `(Test, Test)`→Ok; `(Main, Test)`→`NetworkMismatch{decoded="mainnet",expected="testnet",…}`; `(Test, Main)`→symmetric. `network_kind_name(Main)=="mainnet"`, `(Test)=="testnet"`. Confirm `NetworkKind::from(Network::Signet)==Test` etc.

**Positive-control discipline:** every reject site MUST ship at least one consistent-input control that exits 0 unchanged. This is the funds-safety guard: an over-broad reject that refuses legitimate testnet/signet/regtest wallets would be a *new* availability bug. The controls are non-negotiable.

**Committed FULL-suite sweep (the zero-false-reject proof):** the implementation/review MUST run the WHOLE package suite — `cargo test -p mnemonic-toolkit` (not a targeted `--test` subset) — and it MUST be green post-S-NET. Per the project's full-package-R0 discipline (MEMORY `feedback_r0_review_run_full_package_suite`), a stale or network-consistent fixture elsewhere in the package is the exact class of regression a targeted run misses; the full-suite green is the no-over-rejection proof, NOT the per-site controls alone.

**Originless / no-coin-type positive control:** an originless `tpub` descriptor (e.g. `wpkh(tpubD…/0/*)` with NO `[fp/path]` origin, cf. `cli_descriptor_concrete.rs:174`) → exit 0, imports/parses unchanged. Proves the no-op precondition (§2.2): with no asserted coin-type the cross-check is SKIPPED, so the originless input is not over-rejected. This is a required control, not optional.

**Differential oracle:** existing `bitcoind_differential.rs` AGREE rows must remain green post-S-NET (run if local bitcoind available) — the no-regression proof per §6.3. No new oracle rows strictly required.

---

## 8. FOLLOWUP slugs (to file in `design/FOLLOWUPS.md` at ship)

- `snet-network-provenance-invariant` — the canonical S-NET record: one fail-closed `assert_network_agrees` helper wiring the formerly-dead `NetworkMismatch`, closing H15/M13/M14/H9/L1/L2/L3/L10/L11; toolkit MINOR 0.63.0; status flipped to `resolved` in the shipping commit.
- `snet-l1-build-descriptor-display-warn-not-reject` — record the disposition split (WARN, infer-from-keys when `--network` omitted) so a future "harden everything" pass doesn't accidentally convert it to a reject.
- `snet-l3-coldcard-account-u32-bound` — if L3 is DEFERRED (R0 may prefer firewalling it), this carries the `u64→u32` account bound-check; if folded, file as `resolved` in the same commit.
- `snet-optional-manual-note-network-refusal` — optional, gate-NOT-mandated one-line manual note that import/convert/export now refuse network-mismatched blobs. Deferred by default.
- Cross-reference (do NOT action here): network-family relatives **L8** (`canonical_origin.rs`, toolkit) and **D-mdcli-coin** (`md-cli/src/parse/path.rs`, sibling repo) — out of S-NET scope per the program plan; named for the reviewer's situational awareness.

---

## 9. Resolved decisions (no open questions — leans recorded)

| # | Decision point | Resolved lean | Rationale |
|---|---|---|---|
| 1 | Helper home | `src/network.rs` `assert_network_agrees` | module doc already states the §4.3 cross-check contract |
| 2 | Helper takes Xpub or NetworkKind | **NetworkKind** (decoded + asserted) | dependency-free, unit-testable, serves the WIF case too |
| 3 | Per-parser insertion vs thread-through-chokepoint | **per-parser call after coin-type resolves**; leave `finalize_slot_fields` signature untouched | lower-churn; chokepoint deliberately has no asserted network |
| 4 | `NetworkMismatch` field types (the `&'static str` gotcha) | **keep `&'static str`**, fed by new const-fn `network_kind_name()`; ADD `context: &'static str`; **R0-RATIFIED rename** `xpub_network`→`decoded_network`, `expected`→`expected_network` (no longer an open lean) | the cross-check only yields 2 static names ("mainnet"/"testnet"); no `String` widening needed; rename covers the WIF case |
| 5 | Variant ordering / re-sort | **none needed** — edit in place; variant stays between `MultisigConfig`/`NostrKeyParse` (alphabetical); match arms unmoved | CLAUDE.md alphabetical rule is for *new* variants / *new* match blocks; we add neither |
| 6 | exit code | **2** (already mapped at `error.rs:587`); remove `#[allow(dead_code)]` when wired | user-input / funds-safety reject class |
| 7 | NetworkKind granularity | **Main/Test (2-way), NOT 4-way** | xpub version bytes distinguish only `xpub`(Main)/`tpub`(Test); coin-type-1 = testnet+signet+regtest; matches `network_kind()` + `CosignerSpec` precedent |
| 8 | Per-site disposition | imports (1-7) + H9 + convert (10,11) + export (9) = **REJECT**; build-descriptor L1 = **WARN/DIAGNOSE** | reject sites emit/accept wrong-network artifacts; L1 deliverable is network-agnostic & correct (display-only) |
| 9 | H9 fix shape + variant/exit | **extend the existing `first()`-only class-check (`import_wallet.rs:1192/1199`) to ALL parsed entries, REUSING `ImportWalletNetworkClassMismatch` → exit 1**; compute coin-type per entry; guard+rebind read the SAME per-entry network; reject cross-entry heterogeneity | H9 is the SAME condition as the adjacent sibling refusal, applied per-entry not `first()`-only → use the same variant/exit; NOT `NetworkMismatch`/exit 2 (that is the distinct xpub-version axis, §2.3.1) |
| 9a | Two exit codes in the import block | **INTENTIONAL: exit 1 (`ImportWalletNetworkClassMismatch`, H9 `--network`-vs-coin-type-class axis) + exit 2 (`NetworkMismatch`, H15 xpub-version-vs-coin-type axis) coexist** (§2.3.1) | two distinct conditions, not two spellings of one; documented as such |
| 9b | Originless / no-coin-type input | **NO-OP — skip the cross-check (preserve current accept)** when no asserted coin-type is derivable (originless / sub-2-component origin, e.g. `cli_descriptor_concrete.rs:174`); positive control required | else a legitimate originless `tpub` descriptor is over-rejected — a NEW availability/funds bug |
| 9c | Zero-false-reject proof | **committed FULL-suite sweep** `cargo test -p mnemonic-toolkit` (whole package) green, NOT a targeted-test claim | project full-package-R0 discipline; targeted runs miss stale/consistent fixtures elsewhere |
| 10 | L3 fold vs defer | **FOLD as a co-located ride-along, separate sub-item + own test; REJECT account > u32::MAX (don't saturate)** | cheap, same parser family, metadata-only items rot if deferred; never silently rewrite a user's account index |
| 11 | WIF (L11) same helper or sibling | **same helper, generalized "asserted-source network"** (uses `pk.network`) | helper is xpub-agnostic by design (decision #2/#4) |
| 12 | `CosignerSpec` precedent | **leave `synthesize.rs:776-790` on `CosignerSpec`**; do not migrate to `NetworkMismatch` | works + tested; migrating is pure risk |
| 13 | SemVer | toolkit **MINOR 0.63.0**; first-to-ship claims it (own-account cycle renumbers) | behavior-breaking pre-1.0 = MINOR |
| 14 | Lockstep | **NONE** — no GUI schema_mirror, no manual leg, no sibling-codec companion | no clap flag/subcommand/dropdown change; a new error variant is not a clap surface. (The one `--json` delta is `NetworkMismatch.detail_json` keys on a formerly-dead variant — outside `schema_mirror`/manual scope, zero observable blast radius; §6.2 states this honestly rather than claiming "no wire change at all") |
| 15 | Oracle gate | satisfied by **existing AGREE rows staying byte-identical**; DISAGREE asserts live in CLI/unit suites (oracle can't derive a "correct" address for corrupt input) | S-NET only adds rejections; changes no derived address for valid input |
| 16 | Manual optional note | **deferred** (optional FOLLOWUP) | not gate-mandated; avoid scope creep |

---

## 10. Mandatory R0 gate

Per CLAUDE.md ("MANDATORY pre-implementation R0 gate — NO code before GREEN (0C/0I)"): this brainstorm spec MUST pass an opus-architect **R0 review loop to 0 Critical / 0 Important** before the plan-doc is written, and the plan-doc MUST independently pass its own R0 loop before any code. Reviewer-loop continues after every fold (folds can introduce drift); each round's full review persists verbatim to `design/agent-reports/` BEFORE the fold-and-commit step. The two highest-stakes ratification points for R0: **(i)** the `NetworkMismatch` field-type decision (§2.3 — **R0-RATIFIED round 1, M-4**: keep `&'static str` + the rename) and **(ii)** the L1 WARN-vs-REJECT disposition split (§4) plus the L3 fold-vs-defer call (§5.1).

**R0 round-1 folds applied (this revision):** I1 — H9 reconciled to **`ImportWalletNetworkClassMismatch` / exit 1** (extend the existing `first()`-only class-check to all entries; the two import-block exit codes coexist by design, §2.3.1); I2 — H9 positive control corrected to the same-class `[Bitcoin, Bitcoin] + --network bitcoin → exit 0`, RED test corrected to the mixed `[Bitcoin, Testnet]` blob; I3 — originless / no-coin-type no-op precondition specified (§2.2) with a positive control, plus a committed FULL-package-suite sweep as the zero-false-reject proof (§3/§7). Minors M-1…M-6 folded.

Implementation is a SINGLE subagent per phase (TDD, RED-first), and the class-A differential-oracle no-regression proof (§6.3) is a hard merge precondition.
