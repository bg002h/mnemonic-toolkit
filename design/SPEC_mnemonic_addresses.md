# SPEC — `mnemonic addresses` (batch watch-only address derivation)

**Repo:** mnemonic-toolkit. **Branch:** `mnemonic-addresses-subcommand` off `master` (@ `a9b30ac`).
**SemVer:** toolkit **0.37.11 → 0.38.0** (MINOR — new top-level subcommand, per the constellation convention: nostr→0.34.0, silent-payment→0.35.0, decode-address→0.36.0).
**Source ground truth verified @ `a9b30ac`.**

---

## §1. Context & motivation

Theme B piece #2 of the constellation feature survey ("see it / use it after you recover it"): after recovering a backup the user must reach for an external tool to list the wallet's addresses. `convert --to address` derives a **single** address (mandatory `--path` from master); `export-wallet --range` is only Bitcoin-Core `importdescriptors` metadata (no address loop). The real range-loop lives in `xpub-search`, where the renderer is a **private duplicate** of convert's.

This SPEC adds a dedicated **`mnemonic addresses`** subcommand — the watch-only complement to `export-wallet --range`, mirroring the just-shipped `mk address` (constellation consistency; user chose the dedicated-subcommand form). Read-only public derivation — **no signing, no private keys on stdout**.

It also dissolves the render-fn duplication (a low-risk pre-req refactor the recon surfaced).

---

## §2. Source ground truth (verified @ `a9b30ac`)

- **`src/cmd/convert.rs:357`** — `pub enum ScriptType { P2pkh, P2wpkh, P2shP2wpkh, P2tr }` + `as_str()` + `pub fn parse_script_type_arg(&str) -> Result<ScriptType, String>` (`:376`). **Reuse** — do NOT define a new enum.
- **`src/cmd/convert.rs:1593`** — `fn build_address_from_xpub<C: Verification>(secp, child: &bip32::Xpub, script_type: ScriptType, network: CliNetwork) -> String` (the four `Address::p2*` builders). **PRIVATE.**
- **`src/cmd/xpub_search/address_search.rs:35`** — `fn render_address<C: Verification>(...)` — a **byte-identical private duplicate** of `build_address_from_xpub` (its doc-comment says so). Dedup target.
- **`fn network_from_xpub(xpub) -> CliNetwork`** (`NetworkKind::Main→Mainnet`, `Test→Testnet`; signet/regtest collapse to Testnet) exists in **THREE** private copies: `convert.rs:1616`, `xpub_search/address_of_xpub.rs:359` (verbatim mirror), and is needed by `addresses`. **PRIVATE** in convert — must be lifted (§3.2), not just "called".
- **`src/derive_slot.rs:43`** — `pub(crate) fn derive_bip32_from_entropy(entropy, passphrase, language, network, template: CliTemplate, account: u32) -> Result<DerivedAccount, ToolkitError>` → derives the **account** key at `template.derivation_path(network, account)`. `DerivedAccount` has `account_xpub: Xpub`, `master_fingerprint`, `account_path`, zeroized `entropy` + mlock pin.
- **`src/template.rs:16`** — `pub enum CliTemplate { Bip44, Bip84, Bip49, Bip86, …multisig }`; `derivation_path(&self, network, account) -> DerivationPath` (`:76`) → `m/<purpose>'/<coin>'/<account>'` (coin 0'/1' from network). `script_type_from_template` (`convert.rs:393`) maps Bip44→P2pkh / Bip84→P2wpkh / Bip49→P2shP2wpkh / Bip86→P2tr.
- **`src/cmd/convert.rs:31`** — `pub enum NodeType { Phrase, Seedqr, Entropy, Xpub, Xprv, Wif, Fingerprint, Path, Ms1, Mk1, Bip38, MiniKey, ElectrumPhrase, Address }` — the `--from` node grammar (reuse the `xpub=`/`phrase=`/`entropy=`/`seedqr=` subset).
- **`src/main.rs:88`** — `enum Command { … Convert(cmd::convert::ConvertArgs), … }`; dispatch at `:150`. Add `Addresses(cmd::addresses::AddressesArgs)` arm + dispatch.
- **`CliNetwork`** (`src/network.rs`) `.network_kind()`/`.known_hrp()`; **`CliLanguage`** (`src/language.rs`); **`--passphrase`/`--passphrase-stdin`** pattern (`bundle.rs`, `convert.rs::read_stdin_passphrase`).

---

## §3. Design

### 3.1 `mnemonic addresses`

```
mnemonic addresses --from <SOURCE> --address-type <T> [--account <N>]
                   [--count <N> | --range <A,B>] [--chain <receive|change|both>]
                   [--network <NET>] [--passphrase <V> | --passphrase-stdin] [--json]
```

- **`--from <SOURCE>`** (required) — the `xpub=`/`phrase=`/`entropy=`/`seedqr=` subset of convert's `--from` grammar. **Parsing reuses `parse_from_input`/`NodeType` (`convert.rs:136/31`, both public) — which only SPLITS `<node>=<value>`.** Convert's resolution machinery (`resolve_env_sentinels`/`needs_env_sentinel_resolution`, `convert.rs:1689/1668`) is **private + typed on `ConvertArgs`** → NOT reusable. So `addresses::run` re-implements the resolution loop over the reusable primitives: `env_sentinel::resolve_env_var_sentinel` per secret-bearing value (`@env:VAR`), `read_stdin_to_string` (`convert.rs:706`, `pub(crate)`) for a `-` value with the single-stdin mutual-exclusion check, and `seedqr::decode` for `seedqr=` (decode to a phrase, then the phrase flow). `xprv=`/`mk1=`/multisig sources are out of scope (§6).
- **`--address-type <T>`** (required) — `p2pkh|p2sh-p2wpkh|p2wpkh|p2tr` (reuse `ScriptType` + `parse_script_type_arg`; clap `value_parser`). Drives BOTH the derivation purpose (seed sources) AND the render type. *(The toolkit's older `convert --to address` uses `--script-type`/`--template`; `xpub-search` uses `--address-type`. This subcommand aligns with `xpub-search` + `mk address` — the pre-existing `--script-type`/`--address-type` split is left as-is, not widened.)*
- **Source resolution:**
  - **`xpub=`** → the value IS the account xpub. `--account`/`--passphrase` are rejected (they don't apply to a bare xpub) → `ToolkitError::BadInput`. Network from `network_from_xpub`, overridable by `--network` with a **main-vs-test kind-agreement guard** (mismatch → `BadInput`). **(M2: this guard is STRICTER than `convert`'s xpub→address edge (`convert.rs:1342`), which silently honors a mismatched `--network` — no in-repo precedent; it follows the mk-cli pattern. Deliberate footgun-prevention.)**
  - **`phrase=`/`entropy=`/`seedqr=`** → map `ScriptType → CliTemplate` (P2pkh→Bip44, P2wpkh→Bip84, P2shP2wpkh→Bip49, P2tr→Bip86) and `derive_bip32_from_entropy(entropy, passphrase, language, network, template, account)` → `DerivedAccount.account_xpub` at `m/<purpose>'/<coin>'/<account>'`. `--account` default 0; `--network` default mainnet; BIP-39 `--passphrase`/`--passphrase-stdin`; `seedqr=` decodes to a phrase first (reuse `seedqr::decode`).
- **Range:** `--count N` (default 10) → `0..N`; `--range A,B` → `A..=B` (mutually exclusive). **BIP-32 normal-index ceiling guard validated BEFORE allocating** — reject with `BadInput` (NOT a `from_normal_idx(...).unwrap()` panic; the exact class fixed in mk-cli v0.6.0). **Exact inequality (M4):** `--count N` → highest index `N-1`, valid iff `N ≤ 2^31` (so `--count 2147483648` is VALID — highest index 2^31−1; `--count 2147483649` rejected). `--range A,B` → require `A ≤ B` **and** `B ≤ 2^31−1` (i.e. `B < 2^31`).
- **Chain:** `--chain receive|change|both` (default receive = chain 0; change = 1; both = 0 then 1).
- **Derivation:** for each selected chain `c`, each index `i`: `account_xpub.derive_pub(secp, m/c/i)` → render via the shared helper (§3.2). `secp` = `Secp256k1::verification_only()`.
- **Output (text):** when `both`, group by chain header; rows `  <index>  <address>`.
- **Output (`--json`):** `{ "schema_version": "1", "source": "<node>", "address_type": "p2wpkh", "network": "mainnet", "account": 0, "addresses": [ {"chain":0,"index":0,"address":"bc1q…"}, … ] }` (string `schema_version` — toolkit house style, unlike mk's integer). `account` omitted/null for the xpub source.
- **Non-English advisory does NOT fire** — addresses are DERIVED keys (the BIP-39 language is already applied), not a language-losing re-encodable backup form. Consistent with the v0.37.11 boundary (derived targets don't fire). No `language::non_english_seed_advisory` call here.
- **Secret hygiene:** seed sources go through the zeroize/mlock-pinned `DerivedAccount`; only PUBLIC addresses reach stdout (no xpub/xprv on stdout unless… none). `--passphrase` inline → the existing argv-leak stderr advisory.

### 3.2 Pre-req dedup refactor

New shared module **`src/address_render.rs`** holding TWO lifted `pub(crate)` fns (both currently private + duplicated):

1. `pub(crate) fn render_address_from_xpub<C: Verification>(secp, child: &Xpub, script_type: ScriptType, network: CliNetwork) -> String` — lifted from `build_address_from_xpub` (`convert.rs:1593`). Rewire: `convert.rs:1291`/`:1343` (delete the private fn); `xpub_search/address_search.rs:35` (delete the duplicate `render_address`, `:87` call-site); new `addresses.rs`.
2. `pub(crate) fn network_from_xpub(xpub: &Xpub) -> CliNetwork` — lifted from `convert.rs:1616`. Rewire: `convert.rs` call-sites; `xpub_search/address_of_xpub.rs:359` (delete its verbatim copy); new `addresses.rs`.

Pure move + re-point; behavior byte-identical (covered by existing convert + xpub-search tests + the new addresses tests). Watch for now-dead imports (`Address`, `Secp256k1`, `bitcoin::NetworkKind`) at the vacated sites.

---

## §4. SemVer + lockstep

- **toolkit 0.37.11 → 0.38.0** (MINOR new subcommand). `Cargo.lock` re-resolve; CHANGELOG `[0.38.0]`; both README `<!-- toolkit-version -->` markers (gated by `readme_version_current.rs`); README subcommand count (twenty → twenty-one) in BOTH `README.md:14` and `crates/mnemonic-toolkit/README.md:10` (M3: the crate README's status line is already stale at "v0.36.x" — refresh it while there; the count is NOT gated, manual discipline).
- **GUI schema-mirror** — `mnemonic-gui/src/schema/mnemonic.rs`: new `addresses` `SubcommandSchema` + flags (`--from` composite, `--address-type`/`--chain`/`--network` dropdowns, `--account`/`--count`/`--range`, `--passphrase`/`--passphrase-stdin`, `--json`); bump the toolkit pin. The `schema_mirror` flag-NAME gate fires on the new flags.
- **Manual** — `docs/manual/src/40-cli-reference/41-mnemonic.md`: new `mnemonic addresses` section (every flag) + add to `docs/manual/tests/cli-subcommands.list`. `--json` wire-shape is NOT schema-gated, but the new clap flags ARE (GUI + manual).
- **`mnemonic gui-schema`** auto-reflects the new subcommand (reflective like mk); the GUI mirror is hand-maintained.

---

## §5. Test plan (per-phase TDD)

**Dedup refactor (Phase 0):** existing convert + xpub-search suites stay green after lifting the helper (regression guard); a focused unit test on `render_address_from_xpub` (four types vs known child).

**`mnemonic addresses` (integration):**
1. xpub source, default (count 10, receive, heuristic-free explicit `--address-type p2wpkh`) → 10 `bc1q…`.
2. Address correctness — first N addresses match `convert --to address … --path m/0/i` (independent in-toolkit oracle) for all four `--address-type`s.
3. phrase source + `--address-type p2wpkh` + `--account 0` → derives `m/84'/0'/0'` account → addresses match a known vector; `--passphrase` changes them; `entropy=`/`seedqr=` parity.
4. `--account 1` derives a different account (addresses differ from account 0).
5. `--count`/`--range` (incl. `A>B` → BadInput; conflict → clap error); **BIP-32 ceiling** — `--count 2147483649` → BadInput and `--range 0,2147483648` → BadInput (CLI, reject before allocating, NOT panic); the `2^31` accept boundary is a **unit test** on `resolve_indices` (`Some(2147483648)` → Ok, `Some(2147483649)` → Err) — NOT a CLI run (it would eagerly build an 8 GB index Vec).
6. `--chain receive|change|both` (chain indices + ordering).
7. Network — xpub-inferred; `--network regtest` → `bcrt1…` (test-kind xpub); `--network mainnet` on a test xpub → BadInput (kind mismatch); seed source + `--network testnet` → `tb1…`.
8. xpub source + `--account`/`--passphrase` → BadInput (don't apply).
9. `--json` shape (`addresses[]` of `{chain,index,address}`, string `schema_version`, valid JSON).
10. No non-English advisory fires (french phrase + `--address-type` → stderr has no advisory).
11. Multisig/unsupported `--from` (`xprv=`/`mk1=`) → refused (out of scope).
12. **Secret channels (I2):** `--from phrase=@env:VAR` resolves from the env var; `--from phrase=-` reads from stdin (and a second stdin consumer, e.g. `--passphrase-stdin` + `phrase=-`, → BadInput single-stdin guard); `--from seedqr=<digits>` parity with the equivalent `phrase=`.

**Lockstep:** `gui-schema` includes `addresses`; manual flag-coverage passes.

---

## §6. Non-goals / boundaries

- **No signing / no private keys on stdout** (firm boundary).
- **Single-sig only** — multisig descriptor address ranges out (use `export-wallet`/descriptor tooling); `--from` restricted to `xpub`/`phrase`/`entropy`/`seedqr`.
- **Account-level derivation only** — `m/<purpose>'/<coin>'/<account>'` then `m/chain/index`; no arbitrary `--path` (that's `convert --to address`'s single-leaf job).
- **No `--gap-limit`** — deterministic enumeration, not used-address scanning (that's `xpub-search`).

---

## §7. Open questions for R0
1. `--address-type` required vs an origin-path-style default — SPEC requires it (xpub carries no purpose; seeds need it to pick the template). Confirm.
2. xpub-source `--account`/`--passphrase` → hard `BadInput` vs silent-ignore — SPEC picks hard error. Confirm.
3. JSON `account` field for the xpub source — omit vs null. SPEC: omit. Confirm.
4. Shared-helper module name/location (`src/address_render.rs`) — confirm vs folding into `derive_slot.rs`.
