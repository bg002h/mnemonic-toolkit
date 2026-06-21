# cycle-prep recon — 2026-06-21 — cycle-5 S-NET network-provenance invariant (H15, M13, M14, H9, L1, L2, L3, L10, L11)

**Origin/master SHA at recon time:** `ac4eead0`
**Local branch:** `feature/own-account-subset-search`
**Sync state:** 9 ahead / 17 behind origin/master (local branch is unrelated WIP; all verification done against `origin/master` bytes via `git show origin/master:<path>`).
**Untracked:** the bug-hunt report + the program plan + 8 prior cycle-prep recons + cycle1-4 design artifacts (none on origin/master yet — they are the in-flight bughunt program docs).

Source of findings: `design/agent-reports/constellation-bughunt-2026-06-20.md` (the report itself is untracked; its CITED toolkit source lives on `origin/master ac4eead0`). Workstream: `design/PLAN_constellation_bughunt_fix_program.md` §2 Tier 1 "S-NET", §4 "S-NET" workstream.

**Toolkit current version on origin/master:** `0.62.1`. Findings touch only `mnemonic-toolkit`; NOT a registry publish. Cycle-5 is a toolkit **MINOR** (new fail-closed rejections of previously-accepted corrupt input + the H9 legit-blob mislabel).

**Drift expectation:** the report's citations are snapshots from hunt-time HEAD `8967294d` (a feature branch); origin/master is `ac4eead0`. Several import-parser line numbers have DRIFTED by tens of lines (the `--format descriptor` parser landed v0.58.0 `1a0d0a9d`, plus the cycle-2 H10 export refusal `29b39723`). **No cited bug was structurally invalidated** — every decode site still derives network from coin-type / `--network` without an `xpub.network` cross-check, and all STILL-REPRODUCE.

---

## Per-finding verification

### H9 — `import-wallet --network` class-checks `first()` but rebinds ALL entries
- **WHAT:** the `--network` override resolves the coin-type class from `parsed.first()` only, then `iter_mut()`-rebinds `p.network` for **every** entry; a heterogeneous `[Bitcoin, Testnet]` Vec where `first()` matches the override passes the guard and all entries get silently relabeled. (The one LEGIT-INPUT wrong-label member of the cluster.)
- **Citations:**
  - `cmd/import_wallet.rs:1191-1209` (guard vs rebind) — **DRIFTED, STRUCTURALLY ACCURATE.** On origin/master the block is at **~1192-1209**: `if let Some(override_net) = args.network { if let Some(first) = parsed.first() { … parsed_coin_type … if override_net.coin_type() != parsed_coin_type { return …ImportWalletNetworkClassMismatch }; let rebound = …; for p in parsed.iter_mut() { p.network = rebound; } } }`. The guard reads `first()`; the write touches all. Confirmed verbatim.
  - `cmd/import_wallet.rs:1544` (per-entry emit) — **DRIFTED-by-~3.** `network: network_human_name(p.network)` is at **~1544** (the JSON emit struct `schema_version:"4" … network: network_human_name(p.network) … account: 0`). Confirmed.
  - `wallet_import/bitcoin_core.rs:444-450` (per-descriptor coin-type) — **ACCURATE (lightly drifted).** `coin_type_from_path(p, entry_idx)` collected per cosigner, then `for (i, ct) in coin_types.iter().enumerate().skip(1) { if *ct != first { return …all cosigners must share a coin-type } }` at ~440-450. NB this enforces **intra-descriptor** cosigner agreement; the **cross-entry** (across the `parsed` Vec) heterogeneity is what is unguarded.
- **STILL-REPRODUCES: YES.** Guard-on-`first()` / write-on-all asymmetry is present verbatim. Note the diff-oracle did NOT separately re-rate H9 (it is in the "network-provenance cluster" demoted set conceptually, but H9 is the legit-input member — worst case `[mainnet,testnet]+--network mainnet` presents a testnet descriptor as mainnet, inviting real funds to addresses with no spendable mainnet key). Keep as the highest-priority S-NET row.
- **Fix-site:** compute each entry's coin-type; refuse (or per-entry-gate) when entries span both coin-type classes — guard and rebind must operate on the SAME per-entry network, not `first()` for the guard and all for the write.

### H15 — 7 import parsers derive network from coin-type, never cross-check xpub version bytes
- **WHAT:** every descriptor-bearing import parser computes network solely from the BIP-48 coin-type child; `normalize_xpub_prefix` only swaps/rejects SLIP-132 prefixes, so a canonical mainnet `xpub`/testnet `tpub` passes unchanged and `Xpub.network` is never compared to the coin-type-derived network → wrong-network accept on hand-edited foreign blobs. **Diff-oracle DEMOTED H15 → MEDIUM (corrupt-input-only)** — legit network-consistent round-trips are a clean-negative.
- **Citations (the "7 import parsers"):**
  - `wallet_import/descriptor.rs:168-213` — **DRIFTED.** `network_from_origins` at **~167-196** (`match first { 0=>Bitcoin, 1=>Testnet }`), `coin_type_from_path` at **~198-214**. No `xpub.network` compare. Confirmed.
  - `wallet_import/specter.rs:370-397` — **ACCURATE.** `network_from_origins` at **370**, `coin_type_from_path` at **399**. Same shape.
  - `wallet_import/sparrow.rs:591-633` — **ACCURATE.** `network_from_origins` at **591**, `coin_type_from_path` at **620**. Same shape.
  - `wallet_import/bitcoin_core.rs:430-475` — **ACCURATE (lightly drifted).** `bitcoin_core_network_from_descriptors` cross-cosigner coin-type agreement at ~440-450; `match first { 0=>Bitcoin … }` at ~452. No xpub.network compare.
  - `wallet_import/bsms.rs:386-430` — **STRUCTURALLY ACCURATE (also = L10).** network derived from coin-type via `network_from_origins`/`coin_type_from_path`; first-address handling at `:111/:134/:150/:182`. (Same finding as L10 — see L10.)
  - `wallet_import/coldcard_multisig.rs:678-707` — **ACCURATE.** `network_from_path` at **679** (`coin_type_index(path)` → `match {0=>Bitcoin…}`), `coin_type_index` at **693**. Cross-cosigner coin-type agreement already enforced at `:438-440`; `parse_heterogeneous_coin_type_rejected` test at `:1411`. No xpub.network compare.
  - `wallet_import/electrum.rs:698-716` — **DRIFTED (= L2).** multisig `build_multisig_descriptor` at **660**; coin-type→network `match first_coin { 0=>Bitcoin… }` at **~711**; single-sig path at `:555` DOES use `network_from_xpub_neutral` (the L2 single-vs-multi disagreement). (Same finding as L2 — see L2.)
  - `wallet_import/pipeline.rs:109-122` (`finalize_slot_fields`) — **ACCURATE.** **THE shared decode chokepoint** (see "shared invariant"): at **109-122** it calls `normalize_xpub_prefix(xpub_str)` then `Xpub::from_str(&neutral)` and returns the decoded `Xpub` (with `.network`) — but it receives no asserted/coin-type network to compare against.
  - `wallet_import/slip0132.rs:66-103` (`normalize_xpub_prefix` only rejects unknown prefixes) — **ACCURATE.** **66-103**: swaps SLIP-132 → canonical `xpub`/`tpub`, passes already-neutral through, errors only on unknown prefix. After this swap `Xpub.network` IS the true network (mainnet `xpub`→Main, testnet `tpub`→Test) — so the cross-check is feasible downstream.
  - `mod.rs:508-515` (`validate_watch_only_resolved`) — not re-read; the cited validator is a downstream consistency gate, not a network-derivation site (peripheral to the rule).
- **STILL-REPRODUCES: YES (corrupt-input-only per the diff-oracle, → MEDIUM).** Empirically proven: a mainnet xpub (`0488b21e`) on a `84'/1'/0'` path imports as `network=testnet` → witness program rendered `tb1q…` instead of `bc1q…`. This is the **structural anchor** of S-NET (the ONE rule lives here).
- **Fix-site:** after decoding each xpub, cross-check `xpub.network` against the coin-type-derived network and reject on mismatch. NB the bundle synthesis path already does this via `synthesize.rs` (`CosignerSpec`) — port the same invariant. One shared rule closes H15 + M13 + M14.

### M13 — `export-wallet --from-import-json` trusts the envelope network string, no xpub cross-check
- **WHAT:** the `--from-import-json` export trusts the envelope's `network` string entirely; the only descriptor integrity check is its BIP-380 checksum, not its xpub network. A hand-edited `{"network":"mainnet", … "descriptor":"wpkh([fp/84'/1'/0']tpub…)"}` exports a mainnet-labeled file containing testnet keys; `apply_xpub_prefix` then overwrites the version bytes → wrong-network SLIP-132 re-emit.
- **Citations:**
  - `cmd/export_wallet.rs:711-712` (`network = cli_network_from_str(envelope.bundle.network)`) — **DRIFTED-by-~30.** On origin/master `let network = cli_network_from_str(&envelope.bundle.network)?;` is at **742** inside `run_from_import_json` (fn at 678). Confirmed verbatim.
  - `cmd/export_wallet.rs:824` — **DRIFTED** (consumed-network reference; the cross-check absence holds).
  - `wallet_import/json_envelope.rs:149-154,339-410,483-495` (only BIP-380 checksum validated) — not re-read line-by-line; the structural claim (checksum-only, no network cross-check) is consistent with the M13 mechanism. The brainstorm should re-grep these.
  - `slip0132.rs:108-111` (`apply_xpub_prefix` overwrites version bytes) — **ACCURATE.** `apply_xpub_prefix(xpub, variant, network)` at **108**, `raw[0..4].copy_from_slice(&swap_target_for(variant, network))` at **110** — version bytes overwritten from the CLI/envelope network, not the xpub's own.
- **STILL-REPRODUCES: YES.** Class-A (C→A) corrupt-input; MEDIUM.
- **Fix-site:** cross-check the descriptor/cards' `Xpub` NetworkKind against `envelope.bundle.network` and reject. (Lives in `cmd/export_wallet.rs:run_from_import_json` — shares `convert.rs`/`slip0132.rs` reject helper with M14/L11.)

### M14 — `convert --xpub-prefix` re-emits in `--network` family without checking the xpub's own network
- **WHAT:** `convert --from xpub=<testnet tpub> --to xpub --xpub-prefix zpub --network mainnet` emits a mainnet `zpub` whose decoded key is the testnet account key; the only guard requires `--network` to be **present**, not to **agree**.
- **Citations:**
  - `cmd/convert.rs:1100-1113` (`apply_xpub_prefix(&xpub, prefix, network)` with `network = args.network.unwrap_or(Mainnet)`) — **ACCURATE.** `if let Some(prefix) = args.xpub_prefix {` at **1100**; `let network = args.network.unwrap_or(CliNetwork::Mainnet);` at **1104**; `*value = apply_xpub_prefix(&xpub, prefix, network);` at **1109**. Confirmed verbatim.
  - `cmd/convert.rs:921-926` (guard checks presence not agreement) — **ACCURATE.** `if let Some(prefix) = args.xpub_prefix {` at **922** → `return Err(refusal_xpub_prefix_no_network())` at **924** (the refusal helper is defined at `:495`). Presence-only.
  - `slip0132.rs:197-211` (`swap_target_for` keys purely on the CLI arg) — **ACCURATE.** `fn swap_target_for(variant, network)` at **197**, `let mainnet = matches!(network, CliNetwork::Mainnet);` at **198** — purely CLI-network-keyed.
- **STILL-REPRODUCES: YES.** Class-A; MEDIUM.
- **Fix-site:** verify `xpub.network == args.network.network_kind()` before applying a non-default prefix; refuse on mismatch. (Lives in `cmd/convert.rs` xpub-prefix arm; shares `convert.rs` with L11.)

### L1 — `build-descriptor` human view derives first address with `--network`, no xpub cross-check (DISPLAY-only)
- **WHAT:** `emit_human()` uses `args.network.unwrap_or(Mainnet)` for `derive_receive_addresses`, never checking the descriptor xpubs' own network bytes → a testnet-`tpub` descriptor with no `--network` prints `first receive address (Mainnet): bc1…`. The canonical/bip388 deliverables are network-agnostic & correct; only the DISPLAY label/HRP is wrong.
- **Citations:**
  - `cmd/build_descriptor.rs:476-485` — **ACCURATE.** `fn emit_human` at **470**; `let network = args.network.unwrap_or(CliNetwork::Mainnet);` at **476**; `derive_receive_addresses(&vp.descriptor, 1, network.to_bitcoin_network())` at **480**. Confirmed.
- **STILL-REPRODUCES: YES.** Class-A DISPLAY-only; LOW. **REJECT-vs-WARN candidate to SPLIT:** this is the one site where the real deliverable is correct and only the human-readable label is wrong. A hard-reject here would refuse a perfectly-fine descriptor whose ground-truth output is network-agnostic. The brainstorm should decide DIAGNOSE/WARN (infer display network from keys when `--network` omitted; warn on disagreement) rather than fail-closed — distinct disposition from the import/export hard-reject sites.
- **Fix-site:** walk `vp.descriptor.for_each_key`, read each `Xpub::network`; infer display network from keys when `--network` omitted, or warn on disagreement.

### L2 — Electrum multisig network from coin-type only
- **WHAT:** `build_multisig_descriptor` decides network solely from the BIP-48 coin-type child; the SLIP-132 xpub prefix is used only for `variant_class` and normalized away without asserting agreement. The **single-sig** path DOES derive network from the xpub prefix (`network_from_xpub_neutral`) — the two paths disagree.
- **Citations:**
  - `wallet_import/electrum.rs:698-718` (`build_multisig_descriptor`) — **DRIFTED, STRUCTURALLY ACCURATE.** `build_multisig_descriptor` at **660**; per-cosigner `variant_class` agreement at `:686-693`; per-cosigner `coin_type` agreement at `:699-705`; `match first_coin { 0=>Bitcoin, 1=>Testnet }` at **~711**. Single-sig contrast `network_from_xpub_neutral` at **555** (helper def **886**). Confirmed the single-vs-multi disagreement.
- **STILL-REPRODUCES: YES.** Class-A; LOW. (Also a member of the H15 7-parser set.)
- **Fix-site:** assert every cosigner's neutralized xpub network matches the coin-type-derived network; else `ImportWalletParse`/`NetworkMismatch` (mirror the single-sig `network_from_xpub_neutral` cross-check).

### L3 — Coldcard single-sig `account as u32` truncation bakes wrong origin (network-ADJACENT — SPLIT-OUT)
- **WHAT:** `account = obj["account"].as_u64().map(|n| n as u32).unwrap_or(0)` wraps a `>u32::MAX` JSON value silently; in the legacy top-level-xpub fallback it is interpolated into `format!("m/{purpose}'/{coin_type}'/{raw_account}'")`, producing a wrong **origin path** annotation.
- **Citations:**
  - `wallet_import/coldcard.rs:237-241` — **ACCURATE.** `let raw_account = obj.get("account").and_then(|v| v.as_u64()).map(|n| n as u32).unwrap_or(0);` at **237-241**; legacy fallback `format!("m/{purpose}'/{coin_type}'/{raw_account}'")` at **266**. Confirmed verbatim.
- **STILL-REPRODUCES: YES (mechanism), but DEMOTED to metadata-only by the diff-oracle** — `as u32` account truncation makes origin metadata wrong, **addresses correct**.
- **DISPOSITION: SPLIT OUT of the network invariant.** L3 is an **account-index** truncation in the origin path, NOT an xpub-network agreement bug. It shares the `coldcard.rs` file zone with the S-NET parser (which is why the plan rebases it onto S-NET as a "file zone" rider), but the FIX is unrelated to `assert_network_agrees` — it is a `u64→u32` bound check (`reject/saturate an account > u32::MAX`). The brainstorm should treat L3 as a **co-located ride-along** (cheap to land on the same branch since the file is already open) but NOT as a clause of the shared network helper. It is also the lowest-value item (metadata-only, addresses correct).

### L10 — BSMS network from coin-type only
- **WHAT:** distinct BSMS-parser instance of the Electrum coin-type pattern — network from BIP-48 coin-type child, never cross-checked against cosigner xpub version bytes.
- **Citations:**
  - `wallet_import/bsms.rs:386-413` (`network_from_origins`/`coin_type_from_path`) — **STRUCTURALLY ACCURATE.** bsms.rs follows the same `network_from_origins`/`coin_type_from_path` idiom as descriptor/specter/sparrow (grep confirms the shared helper shape); first-address handling at `:111/:134/:150/:182` (cited `:249,297` for first-address — DRIFTED). The network-derivation mechanism and the missing xpub.network cross-check are present.
  - `wallet_import/bsms.rs:249,297` (first-address) — **DRIFTED** (first-address verification now wired through `derive_first_address` at `:273-275`); peripheral to the network rule.
- **STILL-REPRODUCES: YES.** Class-A; LOW. (Member of the H15 7-parser set.)
- **Fix-site:** assert xpub network consistent across cosigners and with coin-type; else `ImportWalletParse`/`NetworkMismatch`.

### L11 — `convert --from wif --to xpub` uses `--network`, ignores WIF's embedded network
- **WHAT:** the sentinel xpub's network is set from `--network` (default mainnet), discarding the parsed `pk.network`; a testnet WIF → `xpub…` (mainnet) instead of `tpub…`. Sentinel is flagged non-derivable; blast radius limited.
- **Citations:**
  - `cmd/convert.rs:1480-1491` (Wif arm) — **DRIFTED, STRUCTURALLY ACCURATE.** The `Wif => {` arm is at **1480**: `let pk = PrivateKey::from_wif(value)…; let sentinel_xpub = bip32::Xpub { network: network.network_kind(), …, chain_code: ChainCode::from([0u8;32]) };` at **1484-1491**. `network` is the function-level `args.network.unwrap_or(Mainnet)`.
  - `cmd/convert.rs:1217` (`network = args.network.unwrap_or(Mainnet)`) — **ACCURATE.** `let network = args.network.unwrap_or(CliNetwork::Mainnet);` at **1217**. The Wif arm reads this; `pk.network` is discarded.
- **STILL-REPRODUCES: YES.** Class-A; LOW. The `wif → xpub` already emits a "zeroed chain code; not BIP-32 derivable" sentinel WARNING at `:971-974` — so this sentinel is already flagged non-derivable (limited blast radius).
- **Fix-site:** derive network from `pk.network` (the WIF's embedded network), or error on `pk.network` vs `--network` disagreement, in the Wif arm. Shares `convert.rs` with M14.

---

## The shared invariant

### (a) The dead `ToolkitError::NetworkMismatch` variant — CONFIRMED DEAD
- `crates/mnemonic-toolkit/src/error.rs:274` — the variant exists, **annotated `#[allow(dead_code)]`**:
  ```rust
  #[allow(dead_code)]
  NetworkMismatch {
      xpub_network: &'static str,
      expected: &'static str,
  },
  ```
- **Construction sites: ZERO.** `git grep 'ToolkitError::NetworkMismatch' origin/master -- crates/mnemonic-toolkit/src` returns only the variant def + 5 match arms in `error.rs` (`exit_code`→`2` at `:587`; `kind`→`"NetworkMismatch"` at `:656`; three `Display` arms at `:830/:913/:1013`). No code path ever builds it. **CONFIRMED unused — exists for exactly this rule.** (The report's Wave-B appendix explicitly REFUTED "NetworkMismatch is dead → no cross-check anywhere": the bundle path DOES cross-check, but via `CosignerSpec`, not `NetworkMismatch`. The variant is genuinely dead; the import/export/convert paths are the real gap.)
- **GOTCHA for the brainstorm (load-bearing):** the variant's fields are **`&'static str`** (`xpub_network`, `expected`). Dynamic network names (e.g. `format!("{:?}", xpub.network)`, `network.human_name()`) are `String`/`&str`, NOT `&'static str`. Wiring this variant either (i) requires the fields to be changed to `String` (then re-sort the variant + all 5 match arms — alphabetical-by-variant-name per CLAUDE.md is already satisfied; just widen the field types), or (ii) map every (xpub_network, expected) pair to a `&'static str` constant set. Decide in the spec. The exit code is already `2` (user-input class) — appropriate for a corrupt-input reject.

### (b) The `CosignerSpec` precedent in bundle synthesis — CONFIRMED, template to port
- `crates/mnemonic-toolkit/src/synthesize.rs:776-790` (cited as `:771-783` — **DRIFTED-by-~5**). The exact precedent predicate:
  ```rust
  // 2. SPEC §4.3 per-cosigner network/xpub cross-check.
  for (i, c) in cosigners.iter().enumerate() {
      if c.xpub.network != network.network_kind() {
          return Err(ToolkitError::CosignerSpec {
              cosigner_idx: i,
              message: format!(
                  "xpub network {:?} does not match --network {}",
                  c.xpub.network,
                  network.human_name()
              ),
          });
      }
  }
  ```
- **The reusable predicate is: `xpub.network != asserted_network.network_kind()` → reject.** (`xpub.network` is `bitcoin::NetworkKind`; `CliNetwork::network_kind()` maps Mainnet→`Main`, else→`Test`.) This is **NetworkKind-granular (Main/Test), NOT 4-way (mainnet/testnet/signet/regtest)** — signet/regtest all collapse to `Test`, matching coin-type-1. That is the correct granularity for the cross-check (xpub version bytes only distinguish Main vs Test).
- **NOTE the precedent uses `CosignerSpec` (with `cosigner_idx`), NOT `NetworkMismatch`.** The brainstorm must pick: reuse `CosignerSpec`'s pattern with the dead `NetworkMismatch` variant (intended home), or reuse `CosignerSpec` directly. The cleaner choice (and what the plan §2.1 + §4 intend) is to **wire `NetworkMismatch`** for the import/export/convert sites, keeping the `synthesize.rs` `CosignerSpec` cross-check as-is (it already works; don't churn it).

### (c) Proposed shared-helper shape
A single fail-closed helper, e.g.:
```rust
// network.rs (already houses CliNetwork + NetworkKind mapping + a doc-stated "xpub-version cross-check")
pub(crate) fn assert_network_agrees(
    xpub_network: bitcoin::NetworkKind,
    expected: bitcoin::NetworkKind,
) -> Result<(), ToolkitError> {
    if xpub_network != expected {
        return Err(ToolkitError::NetworkMismatch { /* &'static-str or String fields */ });
    }
    Ok(())
}
```
- `network.rs` already exists and its module doc says it "Realizes … §4.3 (network/xpub cross-check via Xpub::network field)" — so this helper has a natural home and a stated contract.
- **ONE shared helper called at N sites** is the right structure (the plan calls S-NET "ONE rule, applied at every decode site"). For the **import** sites, the cleanest insertion is **inside the network-derivation functions** (each parser's `network_from_origins`/`network_from_path` already has both the coin-type-derived network AND the decoded xpubs in scope, or one level up where slots are resolved). `pipeline::finalize_slot_fields` decodes the `Xpub` but does NOT receive the asserted network — so it is the right place to surface `xpub.network` but the cross-check itself must be threaded where the coin-type network is computed. The brainstorm must decide: thread the expected-network into `finalize_slot_fields` (touches every caller's signature), OR call `assert_network_agrees` once per parser after `network_from_origins` resolves, iterating the slots' `xpub.network`. The latter is lower-churn.

---

## Fix-site map (enumerated decode sites needing the invariant)

| # | Site | File:symbol (origin/master) | Network source today | Disposition |
|---|------|------------------------------|----------------------|-------------|
| 1 | import: descriptor | `wallet_import/descriptor.rs` `network_from_origins` (~167) + `build_slot_fields` (155) → `pipeline::finalize_slot_fields` (109) | coin-type only | **REJECT** on mismatch |
| 2 | import: specter | `wallet_import/specter.rs` `network_from_origins` (370) | coin-type only | **REJECT** |
| 3 | import: sparrow | `wallet_import/sparrow.rs` `network_from_origins` (591) | coin-type only | **REJECT** |
| 4 | import: bitcoin-core (= H9 trigger) | `wallet_import/bitcoin_core.rs` per-descriptor coin-type (440) + `cmd/import_wallet.rs:1192-1209` cross-entry rebind | coin-type per-entry + `--network` rebind-all | **REJECT** cross-entry heterogeneity; per-entry xpub.network cross-check |
| 5 | import: bsms (= L10) | `wallet_import/bsms.rs` `network_from_origins` (~386) | coin-type only | **REJECT** |
| 6 | import: coldcard-multisig | `wallet_import/coldcard_multisig.rs` `network_from_path` (679) / `coin_type_index` (693) | coin-type only | **REJECT** |
| 7 | import: electrum multisig (= L2) | `wallet_import/electrum.rs` `build_multisig_descriptor` (660), coin→net (~711); single-sig `network_from_xpub_neutral` (555/886) | coin-type (multi) vs xpub (single) — disagree | **REJECT**; align multi with single |
| — | import: coldcard single-sig account (= L3) | `wallet_import/coldcard.rs:237-241` `as u32`; origin fmt `:266` | n/a (account-index, not network) | **SPLIT OUT** — `u64→u32` bound check, metadata-only |
| 8 | shared decode chokepoint | `wallet_import/pipeline.rs:finalize_slot_fields` (109) | decodes `Xpub` (`.network` available) but receives no asserted network | thread expected-network OR cross-check at each caller |
| 9 | export: from-import-json (= M13) | `cmd/export_wallet.rs:run_from_import_json` (678); `cli_network_from_str(envelope.bundle.network)` (742); `slip0132::apply_xpub_prefix` (108) | envelope JSON string | **REJECT** on descriptor-xpub vs envelope-network mismatch |
| 10 | convert: xpub-prefix (= M14) | `cmd/convert.rs` xpub-prefix arm (1100-1109); guard (922-924 presence-only); `slip0132::swap_target_for` (197) | `--network` (default Mainnet) | **REJECT** on xpub.network vs `--network` mismatch |
| 11 | convert: wif→xpub (= L11) | `cmd/convert.rs` Wif arm (1480-1491); `network` from `args.network.unwrap_or(Mainnet)` (1217) | `--network` | **REJECT** OR derive from `pk.network` (already-flagged non-derivable sentinel) |
| 12 | build-descriptor human view (= L1) | `cmd/build_descriptor.rs:emit_human` (470); `args.network.unwrap_or(Mainnet)` (476) → `derive_receive_addresses` (480) | `--network` (default Mainnet) | **WARN/DIAGNOSE** — DISPLAY-only, deliverable correct (split disposition) |
| — | error variant | `error.rs:274` `NetworkMismatch` (dead, `&'static str` fields) | — | **WIRE** (widen fields or const-map) |
| — | helper home | `network.rs` (`CliNetwork` + doc-stated §4.3 cross-check) | — | new `assert_network_agrees` |

**Net distinct decode/assertion sites needing the rule: 11** (7 import parsers — but #1-#7 collapse to fewer code edits because descriptor/specter/sparrow/bsms share the `network_from_origins` idiom and routes #1-#6 through `pipeline::finalize_slot_fields`; electrum is the outlier — + 3 convert/export sites #9-#11 + 1 build-descriptor display site #12) **+ L3 split out** (account-index, not network) **+ the error-variant wiring + the shared helper**. Reasonable estimate: ~7-9 edit clusters behind one shared helper.

---

## Cross-cutting

- **SemVer: toolkit MINOR (0.62.1 → 0.63.0).** New fail-closed rejections of previously-ACCEPTED corrupt input (H15/M13/M14/L2/L10/L11) are behavior changes; the H9 legit-blob case changes the network LABEL emitted for a heterogeneous blob (previously silently mislabeled → now refused/correctly-labeled). Pre-1.0, behavior-breaking ⇒ MINOR. (The plan's per-finding column also marks every S-NET item "toolkit MINOR" for the structural items / "toolkit PATCH" for the L-tier; bundled into one MINOR cycle.) **Version-bump collision note:** the own-account cycle (paused at plan-R0-GREEN, branch `feature/own-account-subset-search`) also wants 0.63.0 per MEMORY — whichever ships first claims 0.63.0; the other renumbers. Recon-only flag.
- **Lockstep / wire surface: NONE required.** This is purely internal validation — **no new clap flag, no new subcommand, no `--json` wire-shape change.** Confirmed: every fix is an `assert_network_agrees`-style reject inserted into existing decode paths; the only new error variant is the already-declared (dead) `NetworkMismatch`. Therefore:
  - **GUI `schema_mirror` (flag-NAME parity): NOT triggered** — no clap flag add/remove/rename. (A new *error variant* is not a clap surface.)
  - **Manual `docs/manual/src/40-cli-reference/`: NOT triggered** — no CLI flag/option/subcommand change. (Optional: a one-line note that import/convert/export now refuse network-mismatched blobs — nice-to-have, not gate-mandated.)
  - **Sibling-codec FOLLOWUP companions: NONE** — all sites are toolkit-internal; no md/mk/ms codec API change. (The network-family relatives L8 `canonical_origin.rs` and `D-mdcli-coin` `md-cli/parse/path.rs` live in OTHER repos and are explicitly OUT of S-NET — they "coordinate but live in other repos/files" per the plan; do NOT pull them into this toolkit cycle.)
- **Reject-vs-warn per site:**
  - **HARD-REJECT (fail-closed, exit≠0):** all import parsers (#1-#7), export from-import-json (#9), convert xpub-prefix (#10), convert wif→xpub (#11). These accept/emit a wrong-network artifact today; the steel-backup threat model warrants fail-closed (the plan §1 invariant clause + the report's "fail-closed network rule" recommendation).
  - **WARN/DIAGNOSE (not hard-reject):** build-descriptor human view (#12, L1) — DISPLAY-only; the canonical/bip388 deliverables are network-agnostic & correct. Refusing here would block a valid descriptor whose ground-truth output is fine. Infer display network from keys when `--network` omitted; warn on disagreement. **This is the one disposition split the brainstorm must ratify.**
- **Split-outs:**
  - **L3 — SPLIT OUT of the network helper.** It is an account-index `u64→u32` truncation in the ORIGIN path, not an xpub-network bug; diff-oracle DEMOTED it to metadata-only (addresses correct). Keep it as a co-located ride-along on the `coldcard.rs`-open branch (a cheap `reject account > u32::MAX`), but it is NOT a clause of `assert_network_agrees`. Lowest value of the nine; could even be deferred.
  - **L1 — disposition split (warn not reject), not a workstream split** — stays in S-NET but with the DISPLAY-only disposition above.
- **STILL-REPRODUCES robustness:** none of cycles 1-4's shipped commits altered the network-derivation logic. The closest toucher is cycle-2 H10 (`29b39723`, export_wallet refuses unsorted multi) — orthogonal to network. The `--format descriptor` parser (`1a0d0a9d`, v0.58.0) is relatively new but uses the same coin-type-only idiom. **All nine findings reproduce on `ac4eead0`.**
- **Oracle-gate (from the program plan §3, Phase 0):** S-NET is a **class-A** workstream → its plan-doc MUST add the "network-provenance matrix" corpus rows to `tests/bitcoind_differential.rs` (per parser: a coin-type/xpub-AGREE row that passes + derives correct HRP, and a DISAGREE row that asserts exit≠0). The default-CI anti-vacuity leg must gate before merge. This is a hard precondition the brainstorm must name.

---

## Recommended brainstorm-session scope

**ONE FORMAL workstream — "S-NET" — toolkit MINOR (0.63.0), no lockstep.** Single fail-closed rule "decoded xpub network (NetworkKind Main/Test) MUST agree with the asserted network / coin-type, else REJECT" ported from the `synthesize.rs:776-790` `CosignerSpec` precedent and wiring the dead `error.rs:274 NetworkMismatch` variant via one `network.rs::assert_network_agrees` helper.

Scope, in dependency order:
1. **Decide the `NetworkMismatch` field types** (`&'static str` → `String`, or const-map) — load-bearing; blocks every call site.
2. **One shared helper** in `network.rs` (it already has the doc-stated §4.3 contract).
3. **7 import parsers** (#1-#7) — hard-reject; most route through `pipeline::finalize_slot_fields`, so surface `xpub.network` there or cross-check per-parser after `network_from_origins`. Electrum multisig is the outlier (align with its own single-sig `network_from_xpub_neutral`). **H9** is the cross-ENTRY (Vec-level) heterogeneity gate in `import_wallet.rs:1192-1209` — fix the guard-on-`first()`/write-on-all asymmetry.
4. **3 convert/export sites** (M13 export-from-import-json #9, M14 convert-xpub-prefix #10, L11 convert-wif→xpub #11) — hard-reject; share `convert.rs`/`slip0132.rs`.
5. **L1 build-descriptor** (#12) — **WARN/DIAGNOSE only** (display label), not hard-reject — ratify this disposition.
6. **L3 — SPLIT OUT** as a cheap co-located `u64→u32` account bound check (metadata-only, lowest value; defer-able).
7. **Oracle rows:** add the network-provenance AGREE/DISAGREE matrix to `tests/bitcoind_differential.rs` (Phase-0 class-A gate) + default-CI anti-vacuity leg.

**Sizing:** moderate — one rule but ~7-9 edit clusters across `wallet_import/*` + `cmd/{convert,export_wallet,build_descriptor}.rs` + `error.rs` + `network.rs` + oracle rows. The 7 import parsers are the bulk; they share an idiom so the per-parser delta is small once the helper exists. Estimate ~250-450 LOC incl. tests.

**Mandatory next gate (NOT in this recon):** brainstorm spec → opus R0 → plan-doc → opus R0, each converging to **0 Critical / 0 Important** BEFORE any code. The two ratification points for R0: (i) `NetworkMismatch` field-type decision, (ii) L1 warn-vs-reject disposition + L3 split-out. Single-subagent-per-phase TDD; the class-A oracle gate is a hard merge precondition.
