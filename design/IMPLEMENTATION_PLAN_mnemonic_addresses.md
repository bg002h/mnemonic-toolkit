# `mnemonic addresses` Implementation Plan

> Per-phase TDD (tests before impl); per-phase opus review to 0C/0I persisted to `design/agent-reports/`. Checkbox steps.

**Goal:** Add a `mnemonic addresses` subcommand that lists a wallet's receive/change addresses (batch, read-only) from an xpub or seed source, plus a pre-req dedup of the duplicated render/network helpers.

**Architecture:** New `src/address_render.rs` holds the two lifted `pub(crate)` helpers (`render_address_from_xpub`, `network_from_xpub`). New `src/cmd/addresses.rs` parses `--from` (reusing `parse_from_input`/`NodeType`), re-implements secret resolution over the reusable primitives, derives the account xpub (xpub direct, or seed→`ScriptType→CliTemplate`→`derive_bip32_from_entropy`), iterates `m/chain/index`, and renders. Wired into `main.rs`.

**Tech stack:** Rust, bitcoin 0.32, clap derive, serde_json. Spec: `design/SPEC_mnemonic_addresses.md` (R1 GREEN @ `a9b30ac`). SemVer 0.37.11 → 0.38.0.

**Verified APIs (@ a9b30ac):** `ScriptType`/`parse_script_type_arg` (convert.rs:357/376); `build_address_from_xpub` (convert.rs:1593) + duplicate `render_address` (address_search.rs:35); `network_from_xpub` private convert.rs:1616 + dup address_of_xpub.rs:359; `derive_bip32_from_entropy(entropy,passphrase,language,network,template,account) -> DerivedAccount{account_xpub}` (derive_slot.rs:43); `CliTemplate::derivation_path` (template.rs:76); `script_type_from_template` (convert.rs:393); `parse_from_input -> FromInput{node:NodeType,value:String}` (convert.rs:136, pub); `resolve_env_var_sentinel(value,flag_name)` (env_sentinel.rs:56, pub(crate)); `read_stdin_to_string`/`read_stdin_passphrase` (convert.rs:706/719, pub(crate)); `seedqr::decode` (pub); `main.rs` Command enum :88 / dispatch :147.

---

## File structure
- **Create** `src/address_render.rs` — the two lifted helpers + unit test. **BIN module** (it imports `cmd::convert::ScriptType` + `network::CliNetwork`, which are bin-only; C1).
- **Create** `src/cmd/addresses.rs` — `AddressesArgs`, `ChainSel`, `run`.
- **Modify** `src/main.rs` — add `mod address_render;` (alongside `mod cmd;` — NOT `lib.rs`, which has no `mod cmd`/`network`/`language`).
- **Modify** `src/cmd/mod.rs` — `pub mod addresses;`.
- **Modify** `src/cmd/convert.rs` — delete `build_address_from_xpub` + `network_from_xpub`, call shared.
- **Modify** `src/cmd/xpub_search/address_search.rs` + `address_of_xpub.rs` — delete dups, call shared.
- **Modify** `src/main.rs` — `Command::Addresses` arm + dispatch.
- **Create** `tests/cli_addresses.rs`.
- **Modify** `Cargo.toml` (0.38.0), `CHANGELOG.md`, both READMEs, `docs/manual/src/40-cli-reference/41-mnemonic.md`, `docs/manual/tests/cli-subcommands.list`, `mnemonic-gui/src/schema/mnemonic.rs` + pin.

---

## Phase 0 — dedup refactor

### Task 0.1: create `address_render.rs`, lift both helpers
**Files:** Create `src/address_render.rs`; Modify `src/main.rs` (add `mod address_render;` — C1: BIN module, not lib.rs).
- [ ] **Step 1 — write the module** (move bodies verbatim from convert.rs:1593 + :1616):
```rust
//! Shared address rendering + network inference for `convert`, `xpub-search`,
//! and `addresses` (de-duplicated — previously private copies in each).
use bitcoin::bip32::Xpub;
use bitcoin::secp256k1::{Secp256k1, Verification};
use bitcoin::{Address, NetworkKind};

use crate::cmd::convert::ScriptType;
use crate::network::CliNetwork;

/// Render an address string from a (derived) child xpub.
pub(crate) fn render_address_from_xpub<C: Verification>(
    secp: &Secp256k1<C>,
    child: &Xpub,
    script_type: ScriptType,
    network: CliNetwork,
) -> String {
    match script_type {
        ScriptType::P2pkh => Address::p2pkh(child.to_pub(), network.network_kind()).to_string(),
        ScriptType::P2wpkh => Address::p2wpkh(&child.to_pub(), network.known_hrp()).to_string(),
        ScriptType::P2shP2wpkh => {
            Address::p2shwpkh(&child.to_pub(), network.network_kind()).to_string()
        }
        ScriptType::P2tr => {
            Address::p2tr(secp, child.to_x_only_pub(), None, network.known_hrp()).to_string()
        }
    }
}

/// Infer `CliNetwork` from an xpub's version bytes (Main vs Test; signet/regtest
/// collapse to Testnet — not encoded in the prefix).
pub(crate) fn network_from_xpub(xpub: &Xpub) -> CliNetwork {
    match xpub.network {
        NetworkKind::Main => CliNetwork::Mainnet,
        NetworkKind::Test => CliNetwork::Testnet,
    }
}

#[cfg(test)]
mod tests { /* render four types vs a known child xpub; assert bc1q/bc1p/1/3 prefixes */ }
```
(Adjust `ScriptType`/`CliNetwork` import paths to the actual module locations during impl; `ScriptType` is `pub` in `cmd::convert`.)
- [ ] **Step 2** — add `mod address_render;` to **`src/main.rs`** (bin crate; C1). Build. **Step 3** — commit.

### Task 0.2: rewire callers, delete dups
- [ ] Delete `build_address_from_xpub` (convert.rs:1593) + `network_from_xpub` (convert.rs:1616); update convert call-sites (:1291, :1343 render; :1342 network) to `crate::address_render::{render_address_from_xpub, network_from_xpub}`.
- [ ] Delete `render_address` (address_search.rs:35) + its caller (:87) → shared; delete `network_from_xpub` (address_of_xpub.rs:359) + caller → shared.
- [ ] Fix now-dead imports (`Address`, `Secp256k1`, `NetworkKind`) at vacated sites (clippy `-D warnings` will flag).
- [ ] **Run** the FULL suite — `cargo test -p mnemonic-toolkit --no-fail-fast` 0 failures (behavior unchanged); `clippy --all-targets -D warnings`. Commit.

### Task 0.3: Phase-0 review gate → persist → 0C/0I.

---

## Phase 1 — `AddressesArgs` + xpub-source happy path

### Task 1.1: args struct + wiring + xpub flow
**Files:** Create `src/cmd/addresses.rs`; Modify `cmd/mod.rs`, `main.rs`.
- [ ] **Step 1 — failing test** `tests/cli_addresses.rs`:
```rust
use assert_cmd::Command;
const ACCT_84: &str = "xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a";
#[test]
fn xpub_default_p2wpkh_count10() {
    let out = Command::cargo_bin("mnemonic").unwrap()
        .args(["addresses","--from",&format!("xpub={ACCT_84}"),"--address-type","p2wpkh"])
        .output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let s = String::from_utf8(out.stdout).unwrap();
    assert_eq!(s.lines().filter(|l| l.contains("bc1q")).count(), 10);
}
```
- [ ] **Step 2** — FAIL (no subcommand). **Step 3 — implement** `AddressesArgs`:
```rust
#[derive(clap::Args, Debug)]
pub struct AddressesArgs {
    /// xpub=<v> | phrase=<v> | entropy=<hex> | seedqr=<digits>. @env:VAR / - (stdin) for secret values.
    #[arg(long)]
    pub from: String,
    /// Address type (required): p2pkh|p2sh-p2wpkh|p2wpkh|p2tr.
    #[arg(long, value_parser = crate::cmd::convert::parse_script_type_arg)]
    pub address_type: crate::cmd::convert::ScriptType,
    /// Account index (seed sources only). Default 0.
    #[arg(long, default_value_t = 0)]
    pub account: u32,
    #[arg(long, conflicts_with = "range")]
    pub count: Option<u32>,
    #[arg(long, conflicts_with = "count")]
    pub range: Option<String>,
    #[arg(long, value_enum, default_value = "receive")]
    pub chain: ChainSel,
    #[arg(long, value_enum)]
    pub network: Option<crate::network::CliNetwork>,
    #[arg(long)]
    pub passphrase: Option<String>,
    #[arg(long, conflicts_with = "passphrase")]
    pub passphrase_stdin: bool,
    #[arg(long, value_enum)]
    pub language: Option<crate::language::CliLanguage>,
    #[arg(long)]
    pub json: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
#[clap(rename_all = "lower")]
pub enum ChainSel { Receive, Change, Both }
impl ChainSel { fn chains(self) -> &'static [u32] { match self { Self::Receive=>&[0], Self::Change=>&[1], Self::Both=>&[0,1] } } }
```
  Wire `Command::Addresses(cmd::addresses::AddressesArgs)` + dispatch `Command::Addresses(args) => cmd::addresses::run(args, stdin, stdout, stderr)`. **run signature mirrors the stdin-consuming non-repair subcommands** — `silent_payment::run`/`nostr::run` (`addresses` reads stdin for `--passphrase-stdin`/`from=-`, so a USABLE `stdin` is required, unlike `decode_address`'s unused `_stdin`): `pub fn run<R: Read, W: Write, E: Write>(args: &AddressesArgs, stdin: &mut R, stdout: &mut W, stderr: &mut E) -> Result<u8, ToolkitError>` — **`args` by-reference** (dispatch is `match &cli.command`), **`stdin` usable** (not `_stdin`), **returns `Result<u8>`** (the dispatch arm has NO `.map(|_| 0)`; return `Ok(0)`). NO `no_auto_repair` 5th arg; NO `is_json_mode` (that fn does not exist; JSON read via `args.json`). Dispatch arm: `Command::Addresses(args) => cmd::addresses::run(args, stdin, stdout, stderr),`. For Phase 1 implement ONLY the xpub branch: `parse_from_input(&args.from)`; if `node==Xpub` → `Xpub::from_str(&value).map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?` (M3); reject `--account != 0`/`--passphrase`/`--passphrase-stdin` (BadInput — don't apply to a bare xpub); resolve network (`crate::address_render::network_from_xpub` + `--network` kind guard); derive+render (default count 10, chain receive); emit text.
- [ ] **Step 4** — PASS. **Step 5** — commit.

### Task 1.2: address correctness (xpub) + network guard
- [ ] Tests: first address matches `convert --from xpub=ACCT_84 --to address --path m/0/0 --script-type p2wpkh` (independent oracle); all four `--address-type`s; `--network mainnet` on a test xpub → BadInput; `--account`/`--passphrase` on xpub → BadInput. Implement the xpub-branch derive loop (shared `render_address_from_xpub`, `Secp256k1::verification_only()`) + guards. PASS → commit.

### Task 1.3: Phase-1 review gate → persist → 0C/0I.

---

## Phase 2 — seed sources

### Task 2.1: phrase/entropy/seedqr → account xpub
- [ ] Tests: `--from phrase=<12-word>` `--address-type p2wpkh` `--account 0` → addresses == the xpub-source result for the same account xpub (cross-check); `entropy=<hex>` + `seedqr=<digits>` parity; `--account 1` differs; `--passphrase` changes them. **Implement** the seed branch:
```rust
// ScriptType → CliTemplate (inverse of script_type_from_template)
fn template_for(st: ScriptType) -> CliTemplate {
    match st { ScriptType::P2pkh=>CliTemplate::Bip44, ScriptType::P2wpkh=>CliTemplate::Bip84,
               ScriptType::P2shP2wpkh=>CliTemplate::Bip49, ScriptType::P2tr=>CliTemplate::Bip86 }
}
```
  Flow — **resolve the `Option` args to concrete values FIRST** (I2):
```rust
let language: CliLanguage = args.language.unwrap_or_default();      // English default
let network: CliNetwork = args.network.unwrap_or(CliNetwork::Mainnet); // seeds default mainnet
let passphrase: String = /* passphrase_stdin → read_stdin_passphrase(stdin)?; else args.passphrase.unwrap_or_default() */;
// entropy:
//   phrase=  → Mnemonic::parse_in(language.into(), &phrase).map_err(ToolkitError::Bip39)?.to_entropy()
//   entropy= → hex::decode(value)
//   seedqr=  → mnemonic_toolkit::seedqr::decode(&value).map_err(|e| crate::cmd::seedqr::map_seedqr_error(e,&action))? → phrase → to_entropy (M3)
let acct = derive_bip32_from_entropy(&entropy, &passphrase, language, network, template_for(args.address_type), args.account)?;
let account_xpub = acct.account_xpub;   // DerivedAccount.account_xpub (derive.rs:26)
```
  (`Mnemonic::parse_in` takes `(bip39::Language, &str)` — `CliLanguage: Into<bip39::Language>`; `passphrase`/`language`/`network` MUST be concrete, not `Option`, at the `derive_bip32_from_entropy` call.) PASS → commit.

### Task 2.2: Phase-2 review gate → persist → 0C/0I.

---

## Phase 3 — range/chain/json + ceiling guard

### Task 3.1: `--count`/`--range` (ceiling), `--chain`, `--json`
- [ ] Tests (mirror mk + SPEC §5.5/§5.6/§5.9): count default 10 / explicit; `--range A,B` inclusive; `A>B` → BadInput; `2147483649` → BadInput, `--range 0,2147483648` → BadInput (CLI — reject BEFORE allocating); **the `2^31` boundary is a UNIT test only (I1 — a CLI `--count 2147483648` would eagerly build an 8 GB Vec / 2.1B derivations → OOM): `assert!(resolve_indices(Some(2_147_483_648), None).is_ok()); assert!(resolve_indices(Some(2_147_483_649), None).is_err());`**; `--chain receive|change|both` (ordering); `--json` shape `{schema_version:"1", source, address_type, network, account, addresses:[{chain,index,address}]}`. **Implement** `resolve_indices` (BIP-32 ceiling, mirror mk-cli):
```rust
fn resolve_indices(count: Option<u32>, range: Option<&str>) -> Result<Vec<u32>, ToolkitError> {
    const MAX_PLUS1: u32 = 1u32 << 31; // valid indices 0..=2^31-1
    match (count, range) {
        (Some(_), Some(_)) => unreachable!("clap conflicts_with"),
        (Some(c), None) => { if c > MAX_PLUS1 { return Err(bad("--count exceeds 2147483648")); } Ok((0..c).collect()) }
        (None, Some(r)) => { /* split ',', parse a,b; a<=b; b < MAX_PLUS1; (a..=b).collect() */ }
        (None, None) => Ok((0..10).collect()),
    }
}
```
  Loop guards each `index` via `ChildNumber::from_normal_idx(index).map_err(..)?` (defense-in-depth). JSON via `serde_json::json!`. PASS → commit.

### Task 3.2: Phase-3 review gate → persist → 0C/0I.

---

## Phase 4 — secret channels + advisory-non-fire

### Task 4.1: `@env:` / stdin / argv-advisory / advisory-non-fire
- [ ] Tests (SPEC §5.10/§5.12): `--from phrase=@env:VAR` (set env) resolves; `--from phrase=-` (stdin) works; `--passphrase-stdin` + `phrase=-` → BadInput (single-stdin); inline `--from phrase=<secret>` (or `--passphrase <v>`) → stderr argv-leak advisory (M4); a french phrase + `--address-type` → stderr has NO non-English advisory (derived target). **Implement** the resolution loop in `run` (mirror convert.rs:790-835): effective passphrase (stdin vs inline), value resolution (`-`→`read_stdin_to_string` with single-stdin guard; else `resolve_env_var_sentinel(value,"--from")`), seedqr decode; emit `crate::secret_advisory::secret_in_argv_warning(stderr, "--from", "@env:VAR or -")` when a secret-bearing value arrives inline (mirror convert's emission). Confirm NO `non_english_seed_advisory` call. PASS → commit.

### Task 4.2: Phase-4 review gate → persist → 0C/0I.

---

## Phase 5 — version + CHANGELOG + README + end-of-cycle + ship

### Task 5.1: bump + docs
- [ ] `Cargo.toml` 0.37.11→0.38.0; `Cargo.lock` re-resolve; CHANGELOG `[0.38.0]`; both README version markers → 0.38.0 + count twenty→twenty-one (refresh the crate README's stale "v0.36.x" status line too). `cargo test readme_version_current` passes.

### Task 5.2: full gate + end-of-cycle R0
- [ ] `cargo test -p mnemonic-toolkit --no-fail-fast` 0 failures; `clippy --all-targets -D warnings`; (toolkit has NO fmt gate). Dispatch end-of-cycle opus R0 over `master..HEAD`; persist; fold to 0C/0I (re-dispatch per fold).

### Task 5.3: ship
- [ ] Clean tree; `git fetch`; confirm `master==origin/master`; ff-merge `master`; push; tag `mnemonic-toolkit-v0.38.0`; push tag.

---

## Phase 6 — lockstep

### Task 6.1: manual
- [ ] `docs/manual/src/40-cli-reference/41-mnemonic.md`: new `mnemonic addresses` section (every flag, real worked-example output); add `mnemonic addresses` to `docs/manual/tests/cli-subcommands.list`. Run flag-coverage (`MNEMONIC_BIN=<v0.38.0>`).

### Task 6.2: GUI schema-mirror (paired PR on mnemonic-gui)
- [ ] `mnemonic-gui/src/schema/mnemonic.rs`: add `addresses` `SubcommandSchema` (flags: `--from` composite, `--address-type`/`--chain`/`--network` dropdowns, `--account`/`--count`/`--range`/`--passphrase`/`--passphrase-stdin`/`--language`/`--json`); bump the toolkit pin. Run `schema_mirror` against the v0.38.0 binary → 0 drift.

---

## Self-review
- Spec coverage: §5 cells 1-12 map to Phase 1-4 tasks ✓. Dedup §3.2 = Phase 0 ✓. Lockstep §4 = Phase 5/6 ✓.
- No placeholders for load-bearing code (address_render, AddressesArgs, resolve_indices, template_for) ✓.
- Type consistency: `ScriptType`/`CliNetwork`/`ChainSel`/`render_address_from_xpub`/`network_from_xpub` names consistent ✓.
- Ceiling: `--count N` valid iff N ≤ 2^31 (`MAX_PLUS1` = 2^31; reject `c > MAX_PLUS1`) ✓.
