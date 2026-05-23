# v0.34.6 Signet/Regtest `--network` Override — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development / executing-plans. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Add `import-wallet --network <mainnet|testnet|signet|regtest>` to recover signet/regtest semantics from the coin-type-1→Testnet collapse (SPEC §4.2 step 8). The override re-binds the parsed network but ONLY within the parsed coin-type class; cross-class is refused.

**Architecture:** Optional `Option<CliNetwork>` clap flag (reuses the existing `CliNetwork` ValueEnum + GUI `NETWORKS` dropdown). After parse, if present, validate `override.coin_type() == parsed_coin_type` and re-bind every `ParsedImport.network`; mismatch → new typed error. Adds a `CliNetwork→bitcoin::Network` helper. TDD. **Paired GUI + manual lockstep** (new clap flag NAME).

**Tech Stack:** Rust; `src/network.rs`, `src/error.rs`, `src/cmd/import_wallet.rs`, new test `tests/cli_import_wallet_network_override.rs`; `docs/manual/src/40-cli-reference/41-mnemonic.md`; (paired) `mnemonic-gui/src/schema/mnemonic.rs`.

**Source SHA:** `d330240` (v0.34.5 tip). **SemVer:** PATCH (`v0.34.5 → v0.34.6`; additive flag). **Mandatory lockstep:** GUI `schema_mirror` + manual.

**Approved design:** user-approved 2026-05-22 ("Approve" — flag on import-wallet, 4-value reuse + cross-class guard). Recon: `design/cycle-prep-recon-batch-4features.md`.

**Verified facts (live source, SHA d330240):**
- `ParsedImport.network: bitcoin::Network` (`wallet_import/mod.rs:292`); parser produces only `Bitcoin` (coin-type-0) or `Testnet` (coin-type-1) per SPEC §4.2 step 8.
- `CliNetwork` (`network.rs:12`) derives `ValueEnum` `#[clap(rename_all="lower")]` → parses mainnet|testnet|signet|regtest; has `coin_type() -> u32` (Mainnet→0; Testnet/Signet/Regtest→1) + `human_name() -> &'static str` (NOT `as_str()`). **No `to_bitcoin_network()` yet** (add it).
- Initial parse: `let mut parsed: Vec<ParsedImport> = match format_str {…}` at `import_wallet.rs:1119-1135`. Override applies right after (the `mut` Vec), before seed overlay.
- Emit calls the free fn `network_human_name(p.network)` (`import_wallet.rs:2033-2041`) in the `BundleJson{…}` initializer (`:1440`); it handles ALL 4 networks + `_=>"unknown"` → override→Signet/Regtest labels correctly. JSON path: `v[0]["bundle"]["network"]` (envelope array `:1315`, `env.insert("bundle",…)` `:1691`).
- `ToolkitError` ImportWallet* variants (`error.rs:178-221`): alphabetical; new variant slots between `ImportWalletFormatMismatch` (184) and `ImportWalletParse` (196). exit_code arms at `error.rs:467-470` (FormatMismatch=1).
- Fixtures: `core-testnet-bip84.json` (coin-type-1/Testnet), `core-bip84-mainnet.json` (coin-type-0/Bitcoin) — both exist.
- GUI: `IMPORT_WALLET_FLAGS` (`mnemonic-gui/src/schema/mnemonic.rs:1655`); `NETWORKS` const (`:29`); import-wallet SubcommandSchema (`:2802`).
- Manual: import-wallet section at `41-mnemonic.md:680`; flag table ~701-742.

---

## Task 1: `CliNetwork::to_bitcoin_network` + `ToolkitError::ImportWalletNetworkClassMismatch`

**Files:** `src/network.rs`, `src/error.rs`.

- [ ] **Step 1: Add the helper** to `CliNetwork`'s `impl` block in `network.rs` (after `human_name`):

```rust
    /// The `bitcoin::Network` for this CLI network (1:1 mapping).
    pub fn to_bitcoin_network(self) -> bitcoin::Network {
        match self {
            CliNetwork::Mainnet => bitcoin::Network::Bitcoin,
            CliNetwork::Testnet => bitcoin::Network::Testnet,
            CliNetwork::Signet => bitcoin::Network::Signet,
            CliNetwork::Regtest => bitcoin::Network::Regtest,
        }
    }
```
(`use bitcoin::Network` is not needed — fully-qualified `bitcoin::Network` per the existing file style; confirm `bitcoin` is in scope at impl.)

- [ ] **Step 2: Add the error variant** in `error.rs`, alphabetically between `ImportWalletFormatMismatch { … }` and `ImportWalletParse(String)`:

```rust
    /// `import-wallet --network <X>` requested a network in a different
    /// coin-type class than the imported blob's coin-type-derived network.
    /// The blob's xpub prefix is coin-type-bound (coin-type-1 ↔
    /// testnet/signet/regtest; coin-type-0 ↔ mainnet), so cross-class
    /// re-binding would contradict the key material. (exit 1)
    ImportWalletNetworkClassMismatch {
        requested: String,
        parsed_coin_type: u32,
    },
```

- [ ] **Step 3: Add the match arms** for the new variant in `error.rs`. **VERIFIED (R0):** the rendering mechanism is `pub fn message(&self) -> String` (`error.rs:547`) — arms return a `String` via `format!(…)` (NOT a `Display` `write!(f,…)`; `Display` @`error.rs:753-755` wraps `message()` with an `error: {}` prefix). Insert each arm alphabetically (between the `ImportWalletFormatMismatch` and `ImportWalletParse` arms in each block):
  - **`message()`** (insert between `error.rs:664`/`:665`, mirroring the FormatMismatch `format!` arm at `:662-664`):
```rust
            ToolkitError::ImportWalletNetworkClassMismatch { requested, parsed_coin_type } => format!(
                "import-wallet: --network {requested} is incompatible with the imported \
                 wallet's coin-type-{parsed_coin_type} network. The blob's xpub prefix is \
                 coin-type-bound (coin-type-1 ↔ testnet/signet/regtest; coin-type-0 ↔ mainnet); \
                 omit --network to use the coin-type-derived network."
            ),
```
  - **`exit_code()`** (insert between `:468`/`:469`): `ToolkitError::ImportWalletNetworkClassMismatch { .. } => 1,`
  - **`kind()`** (insert between `:524`/`:525`): `ToolkitError::ImportWalletNetworkClassMismatch { .. } => "ImportWalletNetworkClassMismatch",`

- [ ] **Step 4: Build check.** `cargo build -p mnemonic-toolkit` → compiles (exhaustive matches satisfied).

- [ ] **Step 5: Commit.**
```bash
git add crates/mnemonic-toolkit/src/network.rs crates/mnemonic-toolkit/src/error.rs
git commit -m "feat(network,error): CliNetwork::to_bitcoin_network + ImportWalletNetworkClassMismatch variant"
```

---

## Task 2: `--network` flag + override application (TDD)

**Files:** `src/cmd/import_wallet.rs`; new `tests/cli_import_wallet_network_override.rs`.

- [ ] **Step 1: Write the failing tests** — new file `tests/cli_import_wallet_network_override.rs`:

```rust
//! v0.34.6 — `import-wallet --network` signet/regtest disambiguation override.
//! Closes `wallet-import-signet-regtest-disambiguation`.
use assert_cmd::Command;

const FIXTURE_BASE: &str = "tests/fixtures/wallet_import";

fn run_import(fixture: &str, network: Option<&str>) -> std::process::Output {
    let path = std::path::PathBuf::from(FIXTURE_BASE).join(fixture);
    let p = path.to_str().unwrap().to_string();
    let mut args: Vec<String> =
        ["import-wallet", "--format", "bitcoin-core", "--blob", &p, "--json"]
            .iter().map(|s| s.to_string()).collect();
    if let Some(n) = network {
        args.push("--network".to_string());
        args.push(n.to_string());
    }
    Command::cargo_bin("mnemonic").unwrap().args(&args).output().expect("spawn")
}

fn bundle_network(out: &std::process::Output) -> String {
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    v[0]["bundle"]["network"].as_str().expect("network field").to_string()
}

#[test]
fn testnet_blob_default_network_is_testnet() {
    let out = run_import("core-testnet-bip84.json", None);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(bundle_network(&out), "testnet");
}

#[test]
fn testnet_blob_override_to_signet() {
    let out = run_import("core-testnet-bip84.json", Some("signet"));
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(bundle_network(&out), "signet");
}

#[test]
fn testnet_blob_override_to_regtest() {
    let out = run_import("core-testnet-bip84.json", Some("regtest"));
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(bundle_network(&out), "regtest");
}

#[test]
fn testnet_blob_override_to_mainnet_refused() {
    let out = run_import("core-testnet-bip84.json", Some("mainnet"));
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("coin-type"), "expected coin-type-class mismatch; got: {stderr}");
}

#[test]
fn mainnet_blob_override_to_signet_refused() {
    let out = run_import("core-bip84-mainnet.json", Some("signet"));
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn mainnet_blob_override_to_mainnet_noop_ok() {
    let out = run_import("core-bip84-mainnet.json", Some("mainnet"));
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(bundle_network(&out), "mainnet");
}
```

- [ ] **Step 2: Run — verify RED** (the `--network` flag doesn't exist yet → clap errors / unknown arg).

Run: `cargo test -p mnemonic-toolkit --test cli_import_wallet_network_override`
Expected: FAIL (currently `--network` is an unrecognized argument).

- [ ] **Step 3: Add the flag** to `ImportWalletArgs` in `import_wallet.rs` (after the `--decrypt-password-stdin` field block, ~L253) + add `use crate::network::CliNetwork;` to the file's imports:

```rust
    /// v0.34.6: re-bind the imported network to disambiguate signet/regtest
    /// from the coin-type-1→testnet collapse (SPEC §4.2 step 8). Honored ONLY
    /// within the parsed coin-type class (testnet ↔ {testnet,signet,regtest};
    /// mainnet ↔ mainnet); cross-class is refused. Absent = use the
    /// coin-type-derived network. Closes `wallet-import-signet-regtest-disambiguation`.
    #[arg(long, value_name = "NETWORK")]
    pub network: Option<CliNetwork>,
```

- [ ] **Step 4: Apply the override** in `run()` immediately after the initial parse (`import_wallet.rs:1135`, after the `};` closing the `let mut parsed = match format_str {…}`):

```rust
    // v0.34.6: `--network` override (signet/regtest disambiguation). The
    // override must stay WITHIN the parsed coin-type class — the blob's xpub
    // prefix is coin-type-bound. Closes `wallet-import-signet-regtest-disambiguation`.
    if let Some(override_net) = args.network {
        if let Some(first) = parsed.first() {
            let parsed_coin_type: u32 =
                if first.network == bitcoin::Network::Bitcoin { 0 } else { 1 };
            if override_net.coin_type() != parsed_coin_type {
                return Err(ToolkitError::ImportWalletNetworkClassMismatch {
                    requested: override_net.human_name().to_string(),
                    parsed_coin_type,
                });
            }
            let rebound = override_net.to_bitcoin_network();
            for p in parsed.iter_mut() {
                p.network = rebound;
            }
        }
    }
```
(`CliNetwork::coin_type()` returns the BIP-32 coin-type as the same integer type used here — confirm `coin_type()`'s return type and adjust the `u32` annotation / comparison to match at impl.)

- [ ] **Step 5: Run — verify GREEN.**

Run: `cargo test -p mnemonic-toolkit --test cli_import_wallet_network_override`
Expected: all 6 cells pass.

- [ ] **Step 6: Commit.**
```bash
git add crates/mnemonic-toolkit/src/cmd/import_wallet.rs crates/mnemonic-toolkit/tests/cli_import_wallet_network_override.rs
git commit -m "feat(import-wallet): --network signet/regtest disambiguation override (coin-type-class guarded)"
```

---

## Task 3: Manual lockstep

**Files:** `docs/manual/src/40-cli-reference/41-mnemonic.md` (import-wallet section ~680).

- [ ] **Step 1: Add the `--network` row** to the import-wallet flag table + a short paragraph after it:

```
| `--network <mainnet\|testnet\|signet\|regtest>` | (v0.34.6) re-bind the imported network to disambiguate **signet/regtest** from the coin-type-1→testnet collapse (BIP-129 BSMS + Bitcoin Core `listdescriptors` use coin-type `1` for testnet/signet/regtest alike). Honored ONLY within the parsed coin-type class (testnet ↔ {testnet, signet, regtest}; mainnet ↔ mainnet) — a cross-class request (e.g. `--network mainnet` on a testnet-coin-type blob) is refused (exit 1). Absent = use the coin-type-derived network. Note: signet shares testnet's address params (`tb1…`), so `testnet→signet` changes only the network label; `testnet→regtest` changes the HRP to `bcrt1…` |
```

- [ ] **Step 2: Manual lint.** `make -C docs/manual lint MNEMONIC_BIN=$PWD/target/debug/mnemonic MD_BIN=md MS_BIN=ms MK_BIN=mk` → 6/6 OK (flag-coverage now REQUIRES `--network` to be documented — this row satisfies it).

- [ ] **Step 3: Commit.**
```bash
git add docs/manual/src/40-cli-reference/41-mnemonic.md
git commit -m "docs(manual): import-wallet --network signet/regtest override flag row"
```

---

## Task 4: Close FOLLOWUP + release artifacts + ship toolkit

**Files:** `design/FOLLOWUPS.md`, `Cargo.toml`, `Cargo.lock`, `scripts/install.sh`, `CHANGELOG.md`.

- [ ] **Step 1: Close `wallet-import-signet-regtest-disambiguation`** Status → resolved:
```
- **Status:** resolved — v0.34.6. Added `import-wallet --network <mainnet|testnet|signet|regtest>` (option (a), the FOLLOWUP's primary suggestion). Re-binds `ParsedImport.network` post-parse, guarded to the parsed coin-type class (testnet↔{testnet,signet,regtest}; mainnet↔mainnet); cross-class → `ImportWalletNetworkClassMismatch` (exit 1). New `CliNetwork::to_bitcoin_network` helper. 6 cells in `tests/cli_import_wallet_network_override.rs`. Paired GUI schema-mirror (`--network` Dropdown(NETWORKS) on import-wallet) + manual. Closed via cycle-prep recon (SHA `d330240`).
```

- [ ] **Step 2: Version + lock** `Cargo.toml` 0.34.5→0.34.6; `cargo build -p mnemonic-toolkit`; confirm `Cargo.lock` = 0.34.6.
- [ ] **Step 3: install.sh self-pin** v0.34.5→v0.34.6.
- [ ] **Step 4: CHANGELOG** `[0.34.6]`:
```
## mnemonic-toolkit [0.34.6] — 2026-05-22

**SemVer-PATCH — `import-wallet --network` signet/regtest disambiguation.** New `import-wallet --network <mainnet|testnet|signet|regtest>` re-binds the imported network to recover signet/regtest semantics from the coin-type-1→testnet collapse (BIP-129 BSMS + Bitcoin Core `listdescriptors` use coin-type `1` for testnet/signet/regtest alike, so v0.26.0 collapsed all three to testnet). The override is honored only WITHIN the parsed coin-type class (testnet ↔ {testnet,signet,regtest}; mainnet ↔ mainnet); a cross-class request is refused (`ImportWalletNetworkClassMismatch`, exit 1) because the blob's xpub prefix is coin-type-bound. Adds `CliNetwork::to_bitcoin_network`. Paired GUI schema-mirror lockstep + manual. Closes `wallet-import-signet-regtest-disambiguation`.
```

- [ ] **Step 5: Full regression + clippy + manual lint.** `cargo test -p mnemonic-toolkit` green; `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` clean; manual lint 6/6 (with `--network` documented).
- [ ] **Step 6: Commit release artifacts** (incl. plan-doc + R0 review).
- [ ] **Step 7: End-of-cycle opus review → GREEN (0C/0I)**; persist to `design/agent-reports/v0_34_6-end-of-cycle-review.md`.
- [ ] **Step 8: Ship toolkit (GATED — user go-ahead per cadence)** — merge→master (ff), push, tag `mnemonic-toolkit-v0.34.6`, GH release. install-pin-check passes (self-pin bumped).

---

## Task 5: Paired GUI schema-mirror lockstep

**Files:** `mnemonic-gui/src/schema/mnemonic.rs`, `mnemonic-gui/pinned-upstream.toml`, `mnemonic-gui/Cargo.toml`, `mnemonic-gui/CHANGELOG.md`.

- [ ] **Step 1: Add `--network`** to `IMPORT_WALLET_FLAGS` (`mnemonic-gui/src/schema/mnemonic.rs:1655`), canonical position, with ALL required `FlagSchema` fields (mirror the `--format` entry's shape): `name: "--network"`, `kind: FlagKind::Dropdown(NETWORKS)` (reusing the `NETWORKS` const at `:29`), `required: false`, `repeating: false`, `help: "(v0.34.6) re-bind the imported network to disambiguate signet/regtest from the coin-type-1→testnet collapse; honored only within the parsed coin-type class."`, `secret: false`, `default_value: None` (clap `Option<CliNetwork>` has no default), `global: false`.
- [ ] **Step 2: Bump toolkit pin** `pinned-upstream.toml:22` + `Cargo.toml:42` git-dep tag **`mnemonic-toolkit-v0.34.2` → `-v0.34.6`** (lockstep, both). **NOTE (R0):** the GUI pin is currently at **v0.34.2** (v0.34.3/v0.34.4/v0.34.5 were toolkit-only, no CLI-surface change → no GUI lockstep), so the only schema-mirror delta to backfill is `--network` on import-wallet — no cumulative backfill of other flags is required.
- [ ] **Step 3: Bump GUI version** `mnemonic-gui/Cargo.toml` (current 0.19.1 → 0.19.2) + `cargo build --lib` (regen lock).
- [ ] **Step 4: GUI CHANGELOG** `[0.19.2]` — schema-mirror lockstep for toolkit v0.34.6 `import-wallet --network`.
- [ ] **Step 5: Schema-mirror gates** with `MNEMONIC_BIN` = the local v0.34.6 binary: `cargo test --test schema_mirror --test schema_mirror_secret_drift --test gui_schema_conditional_drift --test secret_taxonomy_pin` → green; then full GUI suite + clippy.
- [ ] **Step 6: Commit + ship GUI** — branch→commit→merge→master→push→tag `mnemonic-gui-v0.19.2`→GH release. (Toolkit tag must exist first so the git+tag pin resolves.)

---

## Self-review (writing-plans)

- **Spec coverage:** flag + override + cross-class guard + error + helper + 6 tests + manual + GUI mirror + FOLLOWUP close. ✓
- **No placeholders:** flag decl, override block, error variant + arms, helper, all 6 test cells written verbatim; fixtures verified to exist. ✓
- **Type consistency (R0-verified):** `CliNetwork::coin_type() -> u32` + `human_name() -> &'static str` exist (NOT `as_str()`); new `to_bitcoin_network`; `ParsedImport.network: bitcoin::Network`; `network_human_name` (free fn @:2033) covers all 4; `error.rs::message()` returns `String` via `format!`. ✓
- **SemVer/lockstep:** PATCH; new flag NAME → GUI `schema_mirror` (Task 5) + manual (Task 3) mandatory. ✓
- **Risk:** low-medium — additive flag + guarded re-bind; the load-bearing `network_human_name`-handles-all-4 risk is cleared; main care is the error.rs match-arm structure (read before editing) + the GUI pin-resolves-after-toolkit-tag ordering.
