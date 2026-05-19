# Phase 0 recon — wallet-import v0.26.0

**Date:** 2026-05-18
**Worktree:** `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/wallet-import-export-multiformat-brainstorm/`
**Branch:** `worktree-wallet-import-export-multiformat-brainstorm`
**Predecessor:** toolkit master `7c1f874` (v0.25.1)
**Reconnaissance:** source analysis (feature-dev:code-explorer subagent) + runtime binary verification (controller).

## Verdict summary

| Gate | Status | One-line note |
|---|---|---|
| §0.1 lexsort empirical | YELLOW | Reversed slot order produces DIFFERENT `sortedmulti(...)` key argument order; toolkit preserves declaration order (no sort). Plan-doc §0.1 "byte-identical" expectation was WRONG. |
| §0.2 md_codec TLV shape | GREEN (plan-doc field-list incomplete) | NO `cosigner_order` field. `TlvSection` actually has 5 fields: `{use_site_path_overrides, fingerprints, pubkeys, origin_path_overrides, unknown}` — plan-doc §0.2 lists only 3. |
| §0.3 gui_schema secret flow | GREEN (plan-doc command wrong) | All 4 target flags (`--ms1`, `--passphrase`, `--bip38-passphrase`, `--share`) flow `secret: true`. Plan-doc `--classify-flags` flag does NOT exist; bare `gui-schema` is the correct entry. |
| §0.4 GUI run-confirm verbatim | GREEN | `mnemonic-gui/src/main.rs:686-688` renders argv verbatim via `ui.monospace(format!("  {}", tok))`. All `redact*` matches are in `persistence.rs` (on-disk redaction) — none touch argv display. |
| §0.5 v0.25.1 empty-string sentinel | GREEN | `cli_verify_bundle_watch_only::watch_only_empty_ms1_sentinel_marks_cosigner_skip_with_notice` passes. |
| §0.6 BSMS seed-case parse | **GREEN** (plan-doc command wrong) | User's flagship BSMS descriptor `wsh(thresh(2, pkh(...), s:pk(...), sln:older(32768)))` parses successfully via `export-wallet --descriptor`. Plan-doc's `convert --from descriptor=` is REJECTED — `descriptor` is not a valid NodeType. |
| §0.6.a paired checksum tamper | GREEN | Tampered `#zh0duts1` rejected with explicit `error: export-wallet --descriptor: invalid checksum zh0duts1; expected zh0duts0` (toolkit-side BIP-380 validation). |
| §0.7 slip0132 detect_network | GREEN | NO `detect_network` function. Decision locked: **option (b) — network-detect from `[fp/path]` coin-type**. Rationale below. |
| §0.8 similar crate workspace | GREEN | `grep` returns 0 matches. Phase 4 adds `similar = "2"` as new dep. |

**GO/NO-GO verdict: GO.** All 8 gates resolved. Two plan-doc command defects (R1) and one expectation inversion (R2) surfaced; foldable into §7.0 SPEC amendment commit.

**§0.7 decision locked: option (b)** — extract `coin_type` from the `[fp/path]` origin annotation (BIP-48 path component index 1: hardened `0'` → mainnet, hardened `1'` → testnet). Implementation lives in `wallet_import/bsms.rs` Phase 2; no change to `slip0132.rs`. Rationale: BSMS Round-2 mandates `[fp/purpose'/coin'/account'/script']` origin annotations per BIP-129 §4.1; coin-type is the authoritative network signal. The xpub-prefix approach (option (a)) suffers a `tpub`-stripping false-mainnet trap when users canonicalize prefixes.

---

## Per-gate detail

### §0.1 — Empirical lexsort

**Forward slot order (@0=b8688df1, @1=28645006, @2=5436d724):**
```
"desc": "wsh(sortedmulti(2,[b8688df1/...]xpub6FQya.../0/*,[28645006/...]xpub6DnEB.../0/*,[5436d724/...]xpub6Buxw.../0/*))#nwhu708u"
```

**Reversed slot order (@0=5436d724, @1=28645006, @2=b8688df1):**
```
"desc": "wsh(sortedmulti(2,[5436d724/...]xpub6Buxw.../0/*,[28645006/...]xpub6DnEB.../0/*,[b8688df1/...]xpub6FQya.../0/*))#80plltpl"
```

**Source confirmation:** `crates/mnemonic-toolkit/src/wallet_export/pipeline.rs:72-98` — `slots.iter()` declaration-order traversal; no sort. Bitcoin's `sortedmulti` performs key-sort at SIGNATURE-MATERIALIZATION time, not at descriptor construction. Toolkit wire form preserves user-supplied order.

**Implication for Phase 2:** The `wallet_import` BSMS parser's `@N` placeholder adapter MUST preserve declaration order from the BSMS blob's key list. Round-trip byte-identical equality will fail for ANY blob whose cosigner order differs from toolkit's slot-index order; semantic round-trip + unified-diff WARNING (per SPEC §7.2) is the load-bearing path, not a corner case.

### §0.2 — md_codec Descriptor + TlvSection field shape

**`Descriptor`** (`descriptor-mnemonic/crates/md-codec/src/encode.rs:17-28`):
```rust
pub struct Descriptor {
    pub n: u8,
    pub path_decl: PathDecl,
    pub use_site_path: UseSitePath,
    pub tree: Node,
    pub tlv: TlvSection,
}
```
Plan-doc §0.2 enumeration exact match. **No `cosigner_order` field.**

**`TlvSection`** (`descriptor-mnemonic/crates/md-codec/src/tlv.rs:24-38`):
```rust
pub struct TlvSection {
    pub use_site_path_overrides: Option<Vec<(u8, UseSitePath)>>,
    pub fingerprints: Option<Vec<(u8, [u8; 4])>>,
    pub pubkeys: Option<Vec<(u8, [u8; 65])>>,
    pub origin_path_overrides: Option<Vec<(u8, OriginPath)>>,
    pub unknown: Vec<(u8, Vec<u8>, usize)>,
}
```

**Plan-doc §0.2 OMISSION:** Plan-doc enumerates `{fingerprints, pubkeys, use_site_path_overrides}` but does NOT list `origin_path_overrides` (TLV tag `0x03`, added in md-codec v0.13 §3.2) or `unknown` (forward-compat preservation). For wallet-import, `origin_path_overrides` matters: BSMS cosigners with non-default BIP-48 account paths may surface per-`@N` overrides that the toolkit must preserve through the round-trip. **Phase 2 R0 must verify pipeline.rs handles `origin_path_overrides` correctly.**

### §0.3 — gui_schema secret flow

**Plan-doc command defect:** `gui-schema --classify-flags` is REJECTED — the only `--classify-*` flag is `--classify-descriptor` (v0.20.0 F2). Bare `gui-schema` is the correct entry; emits the full schema JSON envelope on stdout.

**Actual output** (`./target/release/mnemonic gui-schema | jq -c '.subcommands[] | {name, secret_flags: [.flags[] | select(.secret==true) | .name]}'`):

```
{"name":"bundle","secret_flags":["--passphrase","--passphrase-stdin"]}
{"name":"convert","secret_flags":["--bip38-passphrase","--bip38-passphrase-stdin","--passphrase","--passphrase-stdin"]}
{"name":"derive-child","secret_flags":["--passphrase","--passphrase-stdin"]}
{"name":"export-wallet","secret_flags":[]}
{"name":"final-word","secret_flags":[]}
{"name":"inspect","secret_flags":["--ms1"]}
{"name":"repair","secret_flags":["--ms1"]}
{"name":"seed-xor-combine","secret_flags":["--share"]}
{"name":"seed-xor-split","secret_flags":[]}
{"name":"slip39-combine","secret_flags":["--passphrase","--passphrase-stdin","--share"]}
{"name":"slip39-split","secret_flags":["--passphrase","--passphrase-stdin"]}
{"name":"verify-bundle","secret_flags":["--ms1","--passphrase","--passphrase-stdin"]}
```

All 4 target flags (`--ms1`, `--passphrase`, `--bip38-passphrase`, `--share`) flow as `secret: true` across the expected subcommand surfaces. Phase 6 SubcommandSchema entry for `import-wallet` must extend this set (no new pattern; consumed by existing `mnemonic-gui` `secret_flag_keys()` machinery).

### §0.4 — GUI run-confirm-modal verbatim

`/scratch/code/shibboleth/mnemonic-gui/src/main.rs:680-700`:

```rust
.show(ctx, |ui| {
    ui.label(secrets::RUN_CONFIRM_MODAL_PREFIX);
    ui.separator();
    ui.label("Argv:");
    for tok in &argv {
        ui.monospace(format!("  {}", tok));
    }
    ...
});
```

**Verbatim render confirmed.** All `redact*` / `mask` matches are in `persistence.rs` (on-disk JSON state redaction) or `form/secret_widget.rs` (TextEdit input-masking — separate from argv display). No argv-side redaction exists in v0.10.0.

The `@env:VAR` sentinel architecture is the correct v0.11.0 design — argv contains `--ms1 @env:MNEMONIC_MS1_0`, the modal renders the sentinel string, and secret material never crosses the argv boundary. v0.12.0's argv-redaction is defense-in-depth, not a blocker.

### §0.5 — v0.25.1 empty-string sentinel

```
cargo test -p mnemonic-toolkit --release --test cli_verify_bundle_watch_only -- empty_ms1
...
test watch_only_empty_ms1_sentinel_marks_cosigner_skip_with_notice ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out
```

The v0.25.1 empty-string `--ms1 ""` watch-only sentinel is active. Phase 1 §1.5 cell (`env_var_empty_string_preserves_v0_25_1_sentinel`) builds on this: setting `MNEMONIC_MS1_0=""` should resolve through the new sentinel resolver to `""` and trigger the same code path.

### §0.6 — BSMS seed-case parse — GO/NO-GO GATE

**Plan-doc command defect:** `convert --from descriptor=<line2>` fails with:
```
error: invalid value 'descriptor=...' for '--from <FROM>': unknown --from node "descriptor"; expected one of: phrase, entropy, xpub, xprv, wif, fingerprint, path, ms1, mk1, bip38, minikey, electrum-phrase, address
```
`descriptor` is not a `NodeType` (`convert.rs:66-83`). Plan-doc references a surface that never existed.

**Correct verification surface:** `mnemonic export-wallet --descriptor '<line2>'`. Result:
```bash
./target/release/mnemonic export-wallet --descriptor "$DESC_LINE" --network testnet --format bitcoin-core
```
Exit 0. Stdout = Bitcoin Core `importdescriptors` JSON array with 2 entries (receive `/0/*` + change `/1/*` from multipath `<0;1>/*` expansion), each re-checksummed (`#ld0r6z6d` receive, `#7qqz5h4m` change).

**rust-miniscript v13.0 acceptance confirmed for all 4 BSMS syntax features:**
1. `sln:` compound wrapper — `Swap+OrI(0,_)+ZeroNotEqual` per `miniscript/mod.rs:1030-1044`. Accepted.
2. `tpub` testnet keys — `DescriptorPublicKey::XPub` accepts mainnet `xpub` + testnet `tpub` version bytes.
3. BIP-389 multipath `<0;1>/*` — `DescriptorMultiXKey` (`descriptor/key.rs:97-106`) decodes; toolkit auto-splits into receive+change pair.
4. BIP-380 checksum `#zh0duts0` — validated by `Descriptor::from_str` AND a toolkit-side BIP-380 check at `export-wallet`'s entry.

**GO.** Cycle proceeds. No upstream miniscript bump needed.

### §0.6.a — Paired checksum tamper

```bash
TAMPERED=$(echo "$DESC_LINE" | sed 's/#zh0duts0/#zh0duts1/')
./target/release/mnemonic export-wallet --descriptor "$TAMPERED" --network testnet --format bitcoin-core
```
Stderr: `error: export-wallet --descriptor: invalid checksum zh0duts1; expected zh0duts0` (toolkit-side check fires before miniscript parse). Exit code = error path. BIP-380 checksum validation is load-bearing today.

### §0.7 — slip0132 detect_network

`grep -n "pub fn\|pub(crate) fn" crates/mnemonic-toolkit/src/slip0132.rs`:
```
31:    pub fn is_default(self) -> bool {
38:pub fn parse_xpub_prefix_arg(s: &str) -> Result<XpubPrefix, String> {
66:pub(crate) fn normalize_xpub_prefix(
106:pub(crate) fn apply_xpub_prefix(
122:pub(crate) fn neutral_for(variant: &'static str) -> &'static str {
134:pub(crate) fn render_slip0132_info_line(variant: &'static str) -> String {
```

**No `detect_network` function.** Plan-doc §0.7 prediction confirmed.

**Phase 2 §2.3 decision LOCKED: option (b).** Extract network from `[fp/path]` BIP-48 coin-type:
- Path `m/48'/0'/account'/script'` → `coin_type == 0` → `bitcoin::Network::Bitcoin` (mainnet).
- Path `m/48'/1'/account'/script'` → `coin_type == 1` → `bitcoin::Network::Testnet`.
- BIP-44 / BIP-49 / BIP-84 / BIP-86 use the same coin-type slot (index 1 after purpose).

Rationale:
- BSMS Round-2 mandates `[fp/path]` origin annotations per BIP-129 §4.1 — coin-type is always present.
- Bitcoin Core `listdescriptors` also always emits origin annotations.
- Avoids the `tpub` → user-canonicalized-`xpub` false-mainnet trap that option (a) (xpub-prefix-based) suffers.
- Implementation ~5 LOC inside `wallet_import/bsms.rs` and `wallet_import/bitcoin_core.rs`; no `slip0132.rs` change.
- Trade-off: SLIP-132 prefixes (ypub/zpub/upub/vpub) are not consulted; network is derived from path alone. SLIP-132 prefixes remain handled by existing `slip0132.rs::normalize_xpub_prefix` for the xpub-string canonicalization step (orthogonal concern).

Drives **§7.0.a SPEC amendment**: SPEC §4.2 step 8 must be rewritten from "Network detection via `slip0132.rs::detect_network`" to "Network detection from the `[fp/path]` BIP-48 coin-type child number on the first parsed key origin annotation (hardened `0'` → mainnet, `1'` → testnet)."

### §0.8 — similar crate workspace

```bash
grep -rn '^similar\b\|"similar"' Cargo.toml crates/*/Cargo.toml
```
Exit 1, 0 matches. `similar` not in workspace today. Phase 4 §4.2 adds `similar = "2"` to `crates/mnemonic-toolkit/Cargo.toml` (toolkit-crate-level dep, not workspace-shared per BRAINSTORM — narrow scope).

---

## Surprises / risks to fold

### R1 — Plan-doc command defects (3 instances) — CRITICAL pre-Phase-1 amendment

Three plan-doc §0.x verification commands reference CLI surfaces that don't exist today:
1. **§0.3:** `gui-schema --classify-flags` — flag doesn't exist (only `--classify-descriptor`). Correct: bare `gui-schema`.
2. **§0.6:** `convert --from descriptor=<line2>` — `descriptor` is not a `NodeType`. Correct: `export-wallet --descriptor '<line2>'`.
3. **§0.6.a:** Same as §0.6 (uses the same wrong entry point).

These are recon-instruction defects, NOT cycle blockers. The underlying questions (does the gui-schema flow secret-true; does miniscript parse the BSMS descriptor) resolve correctly via the right commands.

**Fold:** Add §0.x command-correction items to Phase 1's §7.0 SPEC-amendment commit, OR amend the plan-doc inline. Pick the latter (plan-doc lives in the worktree; SPEC has its own normative role).

### R2 — Plan-doc §0.1 expected-result inverted

Plan-doc §0.1: "Expected per architect-review: byte-identical output (no toolkit-side lexsort)." This is WRONG. Toolkit preserves declaration order; reversed slot order produces a different `sortedmulti(...)` key sequence. The premise that "no toolkit-side lexsort means byte-identical" conflates two concepts: toolkit doesn't sort, BUT toolkit preserves what you gave it, so different input → different output.

**Fold:** Amend plan-doc §0.1 expectation: "outputs DIFFER in `sortedmulti(...)` argument order, confirming declaration-order preservation. Import-side adapter must mirror this discipline."

### R3 — TlvSection has `origin_path_overrides` (and `unknown`) not listed in plan-doc §0.2

Plan-doc §0.2 enumerates `{fingerprints, pubkeys, use_site_path_overrides}`. Actual: also `origin_path_overrides` (BIP-48 non-default account paths surface here) and `unknown` (forward-compat preservation).

**Fold:** Phase 2 R0 checklist item — confirm `wallet_import/pipeline.rs` populates `origin_path_overrides` correctly when BSMS cosigners use non-default account paths (e.g., account index ≠ 0).

### R4 — Empirical confirmation of `sortedmulti` order preservation strengthens Phase 5 design

The §0.1 finding informs Phase 5 sniff + seed-overlay design: when the user supplies `--ms1 <S>` overlay, the toolkit must map `<S>` to the cosigner whose xpub matches the derived xpub at the BLOB's declared path — NOT to a lexsort position. Phase 5 §5.8 already calls this out (`apply_seed_overlay` matches via `derive_xpub_at_path(&entropy, &cosigner.path)`); no change required, but the empirical confirmation supports the design.

---

## Plan-doc-vs-SPEC amendment decision

R1 + R2 + R3 are **plan-doc inaccuracies** about the existing codebase, not SPEC inaccuracies. SPEC §4.2 step 8 (the slip0132 network-detection) IS a SPEC inaccuracy and is correctly captured in the §7.0.a SPEC amendment.

Recommendation:
- §7.0.a unchanged — still locks SPEC §4.2 step 8 to option (b) (the coin-type extraction approach).
- §7.0.b through §7.0.e unchanged — SPEC + BRAINSTORM amendments per plan-doc §7.0.
- **NEW §7.0.f** (optional, additive) — amend plan-doc §0.1, §0.2, §0.3, §0.6, §0.6.a inaccuracies discovered at Phase 0 execution. Either fold into the §7.0 documentation-only commit OR file as a separate sibling commit titled `design: plan-doc Phase 0 inaccuracies discovered at execution`.

The user has asked to commit freely and defer merging. Folding §7.0.f into the §7.0 commit keeps the artifact trail tight; alternative split is also fine.

---

**Status: GREEN — GO.**

Phase 0 closes successfully. §0.6 GO/NO-GO gate is **GO**. The cycle proceeds to Phase 1 (documentation-only §7.0 amendment commit), then Phase 1 §1.1 code work.

Next step: dispatch opus `feature-dev:code-architect` R0 on this recon report.
