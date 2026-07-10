# SPEC — F3 network fail-open: assert_network_agrees on the five unguarded edges

**Source SHA:** toolkit `713484c3` (master; v0.82.0 line + g4a fix). All citations re-grepped live at this SHA.
**Finding:** constellation-eval 2026-07-06 **F3** (IMPORTANT), `design/agent-reports/constellation-eval-2026-07-06.md:83-99`. Eval remediation "Cycle D".
**SemVer:** MINOR `v0.83.0` (toolkit-only). A previously-accepted wrong-network input now fails closed — behavior change, funds-safety. md/mk/ms NO-BUMP; no new clap flag → no `schema_mirror` lockstep; manual prose lockstep only (§5).

**R0 status:** SPEC-R0 round 1 (Fable) = NOT GREEN → 2 Important + 3 Minor, ALL FOLDED here (round 2 re-dispatch pending). Review `design/agent-reports/cycleH-spec-r0-round-1.md`. Folds: **+E4** (`export-wallet` slot path — a live-reproduced worse edge the round-1 sweep missed, firing on the DEFAULT `--network`; my "export_wallet.rs:793 already-guarded" was false-safe — only the `--from-import-json` arm is guarded); **§2 sweep rewritten** as an intake-arm audit with the 8 benign hits pre-adjudicated (the round-1 grep couldn't even find E3/E4); +2 test cells; +corpus-lockstep reminder; +1 out-of-scope FOLLOWUP note.

## 0. Problem

**Five** toolkit edges render/derive/re-emit from a key whose network is inferable from its version bytes, but use the asserted `--network` (explicit OR clap-default) DIRECTLY without checking it agrees with the key — so a `tpub`/`tprv` + a contradicting network silently yields a wrong-network address/derivation/SLIP-0132-re-encoding from a testnet key (exit 0, no warning), invisible to a standard mainnet restore. The adjacent WIF→xpub edge (`convert.rs:1554`), the `addresses` command (`addresses.rs:183-198`), and `export-wallet`'s OWN `--from-import-json` arm (`export_wallet.rs:792-799`) already fail closed on the identical input via `assert_network_agrees` — this SPEC brings the five laggards to parity.

`assert_network_agrees(decoded: NetworkKind, asserted: NetworkKind, context) -> Result<(), ToolkitError>` (`network.rs:94`) returns the EXISTING `ToolkitError::NetworkMismatch { decoded_network, expected_network, context }` on mismatch. **Precondition (its doc, `network.rs:88-93`):** callers MUST skip the call entirely when there is NO asserted network (so an originless `tpub` is not over-rejected) — i.e. guard ONLY the explicit-`--network` arm; the inference arm stays unguarded. `NetworkKind` granularity is Main-vs-Test (signet/regtest = Test), which is exactly the version-byte agreement level (tpub/vpub/upub all Test).

## 1. The five edges (fix = mirror the `addresses.rs` / WIF→xpub / `--from-import-json` prior-art)

### E1 — `convert` xpub→address (`crates/mnemonic-toolkit/src/cmd/convert.rs:1524-1526`)
Current (unguarded):
```rust
let net = args
    .network
    .unwrap_or_else(|| crate::address_render::network_from_xpub(&xpub));
```
Fix:
```rust
let net = match args.network {
    Some(n) => {
        crate::network::assert_network_agrees(xpub.network, n.network_kind(), "convert: xpub→address")?;
        n
    }
    None => crate::address_render::network_from_xpub(&xpub),
};
```

### E2 — `xpub-search address-of-xpub` (`crates/mnemonic-toolkit/src/cmd/xpub_search/address_of_xpub.rs:215-217`)
Byte-identical `args.network.unwrap_or_else(|| network_from_xpub(&xpub))` pattern → the same `match` guard, context `"xpub-search address-of-xpub"`. (`xpub` is in scope; confirm the local binding name at impl.)

### E3 — `silent-payment` xprv/tprv master (`crates/mnemonic-toolkit/src/cmd/silent_payment.rs:124-135`)
`args.network` here is a NON-optional `CliNetwork` (always asserted). The ms1/phrase/entropy input branches build the master via `Xpriv::new_master(network.network_kind(), seed)` — network-agnostic seed, NO contradiction possible, NO guard. But the **xprv/tprv branch** (`:125` `if let Ok(xpriv) = Xpriv::from_str(s)`) returns the parsed master WITHOUT checking its embedded network against `args.network`, which then drives `coin_type()` (`:250`) + `sp_hrp` (`:255`). A `tprv` + `--network mainnet` derives at mainnet coin-type and emits a mainnet `sp` address from a testnet key. Fix: inside that branch, BEFORE `return Ok(xpriv)`:
```rust
crate::network::assert_network_agrees(xpriv.network, network.network_kind(), "silent-payment: xprv/tprv master")?;
```
(Place after the existing passphrase-ignored warning, or before it — order is cosmetic; the assert should gate the return.)

### E4 — `export-wallet` `--template`/`--slot` path (`crates/mnemonic-toolkit/src/cmd/export_wallet.rs`) — R0-Important-1, live-reproduced
`export-wallet --network` is `#[arg(long, default_value = "mainnet")]` (`:226-227`) — so this fires on the DEFAULT, no explicit flag needed. The `--template --slot @N.xpub=` path resolves slots via `bundle::resolve_slots` (`:542`, → `resolved_slots_ref` `:625/:676`) and builds `EmitInputs { network: args.network…, resolved_slots: resolved_slots_ref, … }` (`:674`) with NO network check; the electrum/coldcard emitters then REWRITE each slot's xpub version bytes from `inputs.network` (`electrum.rs:110/:162` → `render_slip132_xpub` → `apply_xpub_prefix(xpub, variant, network)` `:217`; `coldcard.rs:187-196`). **Live at `713484c3`:** `export-wallet --template bip84 --slot @0.xpub=tpub… --format electrum` (no `--network`) → emits a mainnet `zpub…` at exit 0 — a wrong-network mint, worse than E1-E3. The sibling `--from-import-json` arm already guards this exact hazard (`:792-799`, "cycle-5 S-NET M13"); the template/`--slot` arm was never brought to parity.

Fix — mirror `:792-799` over the resolved slots, placed after `resolved_slots_ref` is bound and BEFORE `EmitInputs` (`~:672`):
```rust
for slot in resolved_slots_ref {
    crate::network::assert_network_agrees(
        slot.xpub.network,
        args.network.network_kind(),
        "export: --template/--slot",
    )?;
}
```
- **`--descriptor` concrete-keys arm:** its `resolved_slots` is EMPTY (concrete keys are not `@N` slots), so the loop is a safe no-op and the SLIP-0132 slot-rewrite never fires on it — placing the guard over `resolved_slots_ref` covers the template/`--slot` arm and is inert for `--descriptor` (no over-rejection). The impl MUST confirm this at the site (grep that the electrum/coldcard/descriptor emitters read version-byte-rewritable keys ONLY from `inputs.resolved_slots`, not from the concrete descriptor string); if a concrete `--descriptor` path DOES reach `apply_xpub_prefix` on an embedded xpub, guard those too. Document the adjudication in the impl.
- **Default-vs-explicit wrinkle:** guarding a clap-default value means `export --template bip84 --slot @0.xpub=tpub…` (no `--network`) now REFUSES instead of minting a mainnet zpub — the CORRECT funds-safe outcome (an export file must commit to one network; `bundle` requires `--network` and fail-closes at `synthesize.rs:1019-1031` — the precedent). Users export a testnet wallet with explicit `--network testnet`.
- **Minor-C (fold): also check `slot.master_xpub`.** `coldcard.rs:220` emits `inputs.master_xpub_at_0` verbatim; a testnet `@0.master_xpub=` + mainnet slot yields an internally-inconsistent coldcard JSON. Extend the loop one line: `if let Some(mx) = slot.master_xpub { assert_network_agrees(mx.network, args.network.network_kind(), "export: --template/--slot master_xpub")?; }` (`master_xpub: Option<Xpub>`, `Xpub: Copy` per `export_wallet.rs:672`).

### E5 — `export-wallet --descriptor --format bsms` 4-line first-address mint (`crates/mnemonic-toolkit/src/wallet_export/bsms.rs`) — R0-round-2-Important-A, live-reproduced
The E4 slot-guard is INERT for the `--descriptor` arm (empty slots), but BSMS 4-line derives line 4 DIRECTLY from the parsed descriptor, not from slots — so a concrete `tpub` descriptor + default mainnet mints a mainnet line-4 address from the testnet key. **Live at `713484c3`:** `export-wallet --descriptor "wpkh([00000000/84h/1h/0h]tpub…/0/*)" --format bsms` (no `--network`, default `--bsms-form 4-line`) → line 4 = `bc1q…` mainnet address, exit 0. Chain: `bsms.rs:113 derive_first_address(&parsed, network_to_bitcoin(inputs.network))` → `derive_address.rs:62-64 definite.address(network)` (no version-byte cross-check).

Fix — guard at the mint site, in `BsmsEmitter::emit`'s `FourLine` arm (`bsms.rs:~100-114`), AFTER `parsed` is built and BEFORE `derive_first_address`: walk `parsed.for_each_key` and assert each **extended** key's network agrees with `inputs.network.network_kind()`, using the vendored fork's per-key helper `DescriptorPublicKey::xkey_network(&self) -> Option<NetworkKind>` (`vendor/miniscript/src/descriptor/key.rs:1043-1049`) — it returns `Some` for BOTH `XPub` AND `MultiXPub` (BIP-389 `<0;1>/*`) and `None` for raw-hex `Single`, i.e. EXACTLY the skip semantics (do NOT hand-match `DescriptorPublicKey::XPub` only — that leaves the BIP-129-canonical multipath `<0;1>/*` mint live, R0-round-3-Important-D). `for_each_key`'s closure returns `bool`, so capture the first mismatch and produce the canonical error after the walk:
```rust
let mut mismatch: Option<bitcoin::NetworkKind> = None;
parsed.for_each_key(|k| {
    if let Some(decoded) = k.xkey_network() {
        if decoded != inputs.network.network_kind() { mismatch = Some(decoded); return false; }
    }
    true
});
if let Some(decoded) = mismatch {
    crate::network::assert_network_agrees(decoded, inputs.network.network_kind(), "export: bsms first-address")?;
}
```
(The whole-descriptor `Descriptor::xkey_network() -> XKeyNetwork` `mod.rs:1004-1028` is an alternative, but its `XKeyNetwork::Mixed` arm would need an explicit refusal; the per-key form above handles Mixed for free and is preferred.)
- Placing it in `BsmsEmitter::emit` FourLine covers BOTH the `--descriptor` arm (the mint) AND the template/`--slot` arm (redundant no-op — E4 already guaranteed the slots, hence the synthesized `canonical_descriptor`, agree). One site, uniform.
- **Do NOT guard the whole `--descriptor` arm:** `--format descriptor`/`bitcoin-core`/`bip388` re-emit the descriptor VERBATIM (zero `inputs.network` consumption — verified `descriptor.rs`, `bitcoin_core.rs`, `bip388.rs:46-52`), so refusing a tpub-descriptor passthrough at default mainnet would over-reject legitimate canonicalization. The mint is BSMS-4-line-specific.
- **BSMS 2-line** has no line-4 → no address mint → adjudicated ACCEPTED (line 2 is the verbatim descriptor; a network-label mismatch is a diagnosis papercut, not a mint).

## 2. Completeness sweep (impl MUST do) — an intake-arm audit, NOT a single grep (R0-Important-2)
The round-1 grep (`network.*unwrap_or_else`) found neither E3 (non-`Option` `args.network`) nor E4 (export) and flag-stormed on 8 benign hits — it can't see the shape it guards. Instead, **audit every site where an asserted network (explicit `--network` OR a clap `default_value` network) is used to render / derive / re-emit against version-byte-bearing input** (`Xpub`/`Xpriv`/WIF/`mk1`/SLIP-0132) WITHOUT reaching `assert_network_agrees`, per command intake arm. **The known set is E1-E5** (E5 = the `export-wallet --descriptor --format bsms` 4-line first-address mint, which the E4 slot-guard does NOT cover — do not treat `--descriptor` as hazard-free after E4). If the audit finds a 6th, FLAG it (surface for a scope decision — do not silently widen).

**Minor-B (fold): restore/bundle bare-xpub asymmetry (out of scope, file a FOLLOWUP — see §3).** `restore.rs:2152` (`let _ = network; // informational`) is adjudicated BENIGN here (the md1 card is a network-less raw-65-byte authority; the search binds key BYTES; worst case is contradiction-not-diagnosed, not a key-network override), but note it is ASYMMETRIC with `bundle`'s `synthesize.rs:1019-1031`, which refuses the identical `s.xpub.network != network.network_kind()` shape — and restore's `--network` is DEFAULTED (`:1376/:3056`) then re-mints via `xpub_from_65_bytes`. Distinct diagnosis class (no wrong mint), so out of this cycle.

**Pre-adjudicated ALREADY-GUARDED (leave):** `convert.rs:1144` + `:1554`, `wallet_import/pipeline.rs:156` (`assert_slots_network_agrees`), `export_wallet.rs:792-799` (`--from-import-json` arm ONLY — the template/`--slot` arm is E4), `addresses.rs:183-194`, `synthesize.rs:1019-1031` (`bundle`).

**Pre-adjudicated BENIGN (network-agnostic or emission-only — leave, do NOT guard; guarding over-rejects):**
- `addresses.rs:199/:237` — seed / Electrum-phrase arms (no embedded version bytes; master minted AT `--network`).
- `convert.rs:1257` — feeds seed arms + the WIF arm already guarded at `:1554`.
- `restore.rs:451/:812/:1376/:3056` — seed inputs; the keyed-md1 arm stores **raw 65-byte keys** (xpubs minted at `--network` via `xpub_from_65_bytes`, no embedded bytes to contradict); bare-xpub cosigners explicitly adjudicated in-code at `restore.rs:2152` (`let _ = network; // network kind is informational`).
- `derive_child.rs:214` — BIP-85: emission-only network; the master's network affects no derivation byte (in-code comment `:191-194`). Out of F3's class; guarding would over-reject.

## 3. Out of scope
- **GUI amplification / dropdown default** (F3 "amplified by the GUI"; the `--network` dropdown seeds mainnet with `default_value:None`, so untouched GUI `convert` passes explicit `--network mainnet` and now fails closed for testnet keys). The toolkit fix CLOSES the funds risk (fail-closed) regardless of the GUI; the GUI change (default `--network` to inference, an F6-adjacent `default_value` fix) is a **companion** tracked in `mnemonic-gui` — file a FOLLOWUP/companion note, do NOT fold a GUI change into this toolkit cycle. Note: after this ships, a GUI on a bumped pin will surface an error instead of a wrong address for the testnet+default-mainnet case — the CORRECT funds-safe outcome (louder, not wrong).
- F2/F5/F6/M-series — separate cycles.
- No new error variant (reuse `NetworkMismatch`), no new flag, no wire/JSON shape change.
- **xpub-search *search*-family diagnostic (R0-Minor-3, out of scope → file a FOLLOWUP `xpub-search-network-contradiction-not-diagnosed`):** `path-of-xpub` / `passphrase-of-xpub` / `passphrase-search` with a `tpub` target + `--network mainnet` derive mainnet candidates against a testnet target → exhaust + report NOT-FOUND instead of diagnosing the network contradiction. Wrong-diagnosis papercut, NO wrong-address mint (distinct from E1-E5's mint class) → file, do not fold into this cycle.
- **restore/verify-bundle bare-xpub `--cosigner` network contradiction (R0-round-2-Minor-B, out of scope → file a FOLLOWUP `restore-cosigner-bare-xpub-network-contradiction-not-diagnosed`):** `restore.rs:2152` accepts a `--cosigner` bare-xpub whose network contradicts the DEFAULTED `--network` without diagnosis (asymmetric with `bundle`'s `synthesize.rs:1019-1031` refusal), then re-mints via `xpub_from_65_bytes`. Contradiction-not-diagnosed, NO wrong mint (the md1 authority is network-less) → file, do not fold.

## 4. TDD (tests before impl; per edge, RED-proven)
`tests/cli_network_fail_open.rs` (new) — public never-fund test vectors:
- **E1a** `convert --from xpub --to address --path <p> --script-type <s> --network mainnet <TPUB>` → exit = `NetworkMismatch` code, stderr names decoded=test/expected=main; NO address on stdout. (Reproduces the eval's `tpub`+`--network mainnet` → `bc1q…` exit-0 case, now closed.)
- **E1b** same but `--network testnet` (agrees) → exit 0, renders the testnet address.
- **E1c** same but NO `--network` (inference) → exit 0, renders the inferred (testnet) address. (Guards against over-rejection / the originless arm.)
- **E1d** mainnet `xpub` + `--network mainnet` → exit 0 (agreement, no regression).
- **E2a/E2b** `xpub-search address-of-xpub` tpub + `--network mainnet` → mismatch; + agreeing/None control.
- **E3a** `silent-payment --network mainnet` with a `tprv` master input → mismatch; **E3b** matching `--network testnet` → ok; **E3c** an ms1/phrase seed input + any `--network` → ok (network-agnostic, NOT over-rejected — the load-bearing non-regression for the seed path).
- **E4a** `export-wallet --template bip84 --slot @0.xpub=<TPUB> --format electrum` with NO `--network` (default mainnet) → mismatch, NO zpub minted (reproduces the R0-Important-1 live case, now closed). **E4b** same + explicit `--network testnet` → exit 0, emits the testnet `vpub`/coin-type-1 encoding. **E4c** a mainnet `xpub` slot + default `--network` → exit 0 (no regression). **E4d** `--descriptor` concrete-key path with `--format descriptor` (verbatim passthrough) → exit 0 (guard inert; the concrete descriptor still exports — pins that E4/E5 do NOT over-reject descriptor canonicalization).
- **E5a** `export-wallet --descriptor "<tpub-desc>" --format bsms` (defaults: mainnet, 4-line) → `NetworkMismatch` exit 2, NO address line (R0-round-2-Important-A, now closed). **E5b** same + `--network testnet` → exit 0, line 4 = `tb1…`. **E5c** `--format bsms --bsms-form 2-line` + tpub descriptor → exit 0 (no line-4 mint; verbatim descriptor — adjudicated accepted). **E5d** a hex-pubkey-only descriptor + `--format bsms` 4-line → exit 0 (Single keys carry no version bytes — the skip-arm non-regression pin). **E5e** a **multipath** `<0;1>/*` tpub descriptor + `--format bsms` defaults → `NetworkMismatch` exit 2, NO address (R0-round-3-Important-D — the `MultiXPub` variant; today mints exit 0; the `xkey_network()` helper closes it); + the agreeing `--network testnet` multipath control → exit 0 (line 3 `/0/*,/1/*`).
- **Minor-1 cells:** an E2 cell with an **mk1 card** target (pins that the guard covers mk1-decoded xpubs via `resolve_target_xpub`); a `--network signet` + `<TPUB>` **ACCEPT** cell on E1 (pins the 2-way `NetworkKind` semantics — signet==Test==tpub's kind → renders at test coin-type — against future over-tightening).
- Assert the exact `NetworkMismatch` exit code = **2** (verified `error.rs:621`; still re-grep at impl in case of drift).

## 5. Manual lockstep (prose only — no flag change)
Mirror the `addresses.rs` refusal wording into the manual `docs/manual/src/40-cli-reference/41-mnemonic.md` sections for `convert` (xpub→address), `xpub-search address-of-xpub`, and `silent-payment`: note that an explicit `--network` disagreeing with the key's version bytes now refuses (fail-closed) instead of rendering a wrong-network address. Re-grep the exact anchors at impl. R0 confirmed 0 hits for `tpub`/`tprv` + `--network mainnet` across `docs/manual/` and `.examples-build/`, so no golden `.out` transcript exercises a now-refused path (none to regen for a diff). **Corpus-lockstep reminder (Minor-2 / Cycle-A gotcha):** `examples.yml` re-runs `.examples-build/gen.sh` on ANY `crates/` change and the corpus self-pin FATALs on version drift — so the v0.83.0 release step MUST bump gen.sh's version pins + regen `.examples-build/Examples.md` (a pure version-string diff is expected, no transcript content change) EVEN THOUGH no example hits the refused path. No `docs/manual/src/40-cli-reference/` flag-table change (no flag added/removed).

## 6. Guard-rails
- **G1** Guard ONLY the explicit-`--network` arm (respect the `assert_network_agrees` precondition: skip when no asserted network) — the inference arm and originless inputs stay unguarded (no over-rejection).
- **G2** Reuse `ToolkitError::NetworkMismatch` + `assert_network_agrees`; NO new error variant, NO new match arm ordering churn.
- **G3** Compare against the KEY's embedded network kind (`xpub.network` / `xpriv.network`), mirroring `addresses.rs:186`'s `n.network_kind() != xpub.network` — NOT a re-inference that could differ.
- **G4** Seed/phrase/ms1/entropy silent-payment inputs stay unguarded (network-agnostic; E3c is the regression pin).
- **G5** No clap-surface change → no `schema_mirror`; toolkit-only MINOR; md/mk/ms NO-BUMP; no re-vendor; no sibling-pin change.
