# cycle-5 PLAN R0 — round 1

- **Artifact under review:** `design/IMPLEMENTATION_PLAN_cycle5_snet_network_invariant.md`
- **Implements (GREEN) spec:** `design/BRAINSTORM_cycle5_snet_network_invariant.md` (0C/0I at spec-R0 round 2)
- **Source-of-truth SHA:** `origin/master` `ac4eead0` (toolkit `0.62.1`)
- **Date:** 2026-06-21
- **Reviewer mandate:** adversarial plan-doc R0 under the HARD pre-implementation gate (no code until 0C/0I). Verify the plan faithfully executes the GREEN spec against LIVE source; do not re-litigate the spec.

---

## Verification log (live source, all at `ac4eead0`)

Every cited edit site was re-grepped against `git show origin/master:<path>`. Results:

| Plan claim | Live source | Verdict |
|---|---|---|
| `error.rs:273` `#[allow(dead_code)] NetworkMismatch` | `:273` allow, `:274` variant, fields `xpub_network`/`expected` `:275-276` | ✅ |
| exit arm `:587` → 2 | `:587` `NetworkMismatch { .. } => 2` | ✅ |
| Display `:830`, detail_json `:913` | `:830-836` Display (`"xpub network {} does not match --network {}"`), `:913-919` detail_json | ✅ |
| `kind` `:656` → `"NetworkMismatch"` | `:656` | ✅ |
| unit test `:1013` asserts `exit_code()==2` only | `:1008-1019` constructs `{ xpub_network:"main", expected:"test" }`, asserts `==2`; NO Display assert | ✅ |
| ONLY readers of `xpub_network`/`expected` = Display+detail_json+def+test | grep confirms exactly 4 sites; `exit_code`/`kind` use `{ .. }` (rename-immune); NO `friendly.rs` reader | ✅ |
| `synthesize.rs:776-790` CosignerSpec predicate (`c.xpub.network != network.network_kind()`) | `:777-787` exactly | ✅ |
| H9 guard `import_wallet.rs:1192` `parsed.first()`; return `:1199`; `iter_mut()` rebind `:1204` | `:1192` `if let Some(first) = parsed.first()`, `:1199` `ImportWalletNetworkClassMismatch`, `:1204-1206` `for p in parsed.iter_mut() { p.network = rebound }` — the guard/rebind asymmetry is EXACTLY as described | ✅ |
| H9 per-entry emit `:1544` `network: network_human_name(p.network)` | `:1544` | ✅ |
| `ImportWalletNetworkClassMismatch` exit **1** (`error.rs:576`) | `:576` `=> 1` | ✅ |
| descriptor `coin_type_from_path` ≥2-component precondition `:199` | `:197-203` errors (`ImportWalletParse`) when `comps.len() < 2` | ✅ (see I1 — it ERRORS, not skips) |
| 7 import parsers: descriptor `:168`, specter `:370`, sparrow `:591`, bitcoin-core `:430`, bsms `:386`, coldcard-multisig `:679/:693`, electrum `:660`/`:886` | all present at cited lines | ✅ |
| convert xpub-prefix arm `:1100`, presence guard `:922` | `:1100-1112` arm (`apply_xpub_prefix` `:1110`), `:921-924` `refusal_xpub_prefix_no_network` | ✅ |
| convert wif→xpub `:1480`, sentinel `network: network.network_kind()` discards `pk.network` | `:1480-1491`, `pk = PrivateKey::from_wif`, `sentinel_xpub.network = network.network_kind()` | ✅ |
| export `:742` `cli_network_from_str(&envelope.bundle.network)` | `:742` exactly | ✅ |
| build-descriptor `:476` `args.network.unwrap_or(CliNetwork::Mainnet)` (L1) | `:476` exactly, in `emit_human` `:470` | ✅ |
| L3 truncation `coldcard.rs:237-241` `as u32`; legacy fallback `:266` `format!("m/{purpose}'/{coin_type}'/{raw_account}'")` in the `deriv_path_str_opt == None` arm | `:236-241` truncation, `:266` format inside `None =>` arm; per-bipN uses `Some(s)` arm (never interpolates `raw_account`) | ✅ |
| `NetworkKind::from(Network)`: Bitcoin→Main, all else→Test | bitcoin-0.32.8 `network.rs:50-57` exactly | ✅ |
| `PrivateKey.network: NetworkKind` (L11) | bitcoin-0.32.8 `key.rs:402` | ✅ |
| `DescriptorPublicKey::{Single, XPub(DescriptorXKey), MultiXPub(DescriptorMultiXKey)}`; `.xkey` field | miniscript-13.0.1 `key.rs:24/26/28`, `DescriptorXKey.xkey:66`, `DescriptorMultiXKey.xkey:100` | ✅ |
| version sites: `Cargo.toml` `0.62.1`; install.sh `:32`; both READMEs `<!-- toolkit-version: 0.62.1 -->`; `fuzz/Cargo.lock:575`; CHANGELOG | all present; CHANGELOG gate is tag-based (`changelog-check.yml` asserts a `[<ver>]` section) | ✅ |
| originless positive-control `cli_descriptor_concrete.rs:174` | `:174` — but it is an **export-wallet** test (`export_wallet_originless_concrete_still_accepted`), not import-wallet | ⚠ (see M1) |

**Net:** every load-bearing line citation in the plan resolves correctly against `ac4eead0`. The architecture (one `NetworkKind`-pair helper, dead-variant wiring, two-axis separation, L1-WARN/L3-reject firewall) is sound and faithfully executes the GREEN spec. Findings below are about *test-construction guidance gaps*, not architectural defects.

---

## Critical

**(none.)**

The funds-safety crux — two-axis separation, over-rejection guard, fail-closed disposition — is correct and live-source-verified (see the explicit yes/no at the end). No finding rises to Critical.

---

## Important

**(none.)**

Two candidate Importants were investigated and **dismissed** as not-defects after live-source verification; recorded here so the next round sees the reasoning:

- **(dismissed) "H9 RED may be unreachable."** The H9 bug needs a SINGLE parser to emit a `Vec<ParsedImport>` with `[Bitcoin, Testnet]` heterogeneity. Verified: of the 9 parsers, ALL of `descriptor`/`specter`/`sparrow`/`coldcard`/`jade` return `Ok(vec![ParsedImport{..}])` — single-entry. **Only `bitcoin-core`** loops (`bitcoin_core.rs:208-210` `out.push(parse_entry(i, …))`) and each `parse_entry` independently derives its own network → a 2-descriptor bitcoin-core blob (descriptor[0] coin-type-0, descriptor[1] coin-type-1) IS a reachable `[Bitcoin, Testnet]` Vec. The existing override test harness already hardcodes `--format bitcoin-core` (`cli_import_wallet_network_override.rs:14`). So the H9 RED is reachable and genuinely RED-first — but the plan never NAMES the format, which would let an implementer pick a single-entry format and write a vacuous test. This is a real guidance gap → downgraded to **Minor M2** (the spec/plan are correct, just under-specified; not a wrong instruction).
- **(dismissed) "originless no-op precondition is mis-anchored for the import path."** The plan/spec anchor the no-op precondition on `coin_type_from_path` "needs ≥2 components" and cite `cli_descriptor_concrete.rs:174`. Live: in the descriptor *import* parser, `network_from_origins` returns an **`Err(ImportWalletParse)`** both when `origins.is_empty()` (`:171-174`) and when `coin_type_from_path` sees `<2` components (`:199-203`). So an originless descriptor is NOT silently accepted by *import* today — it already errors. The genuine no-op surface is **export-wallet** (the `cli_descriptor_concrete.rs:174` control is an export test) and any decode site that reaches a bare xpub with no coin-type. The architecture is still correct (skip the cross-check when no asserted network exists), and the full-suite sweep will catch any regression regardless of where the no-op bites — so this is not a defect, but the anchoring prose is imprecise → **Minor M1**. It does not change any edit and does not weaken the over-rejection guard.

---

## Minor

- **M1 — originless no-op control is an export-wallet test, and the import parsers already error on originless.** §2.2 / Phase-2 prose imply the import parsers currently *accept* originless input and must be made to *skip* the new check to preserve that accept. Live source shows the import descriptor parser already `Err`s on `origins.is_empty()` (`descriptor.rs:171`) and on `<2`-component paths (`:199`). The real no-op surface for the import leg is "a coin-type WAS derivable but we only call the helper once it is" — i.e., the guard is "did we resolve a coin-type network?", which the plan already states (§2.2 insertion strategy). **Fix:** in Phase 2 / Phase 5, note that the originless *positive control* lives on the **export-wallet** path (`cli_descriptor_concrete.rs:174` is `export_wallet_originless_concrete_still_accepted`), and that for the import parsers the no-op is automatically satisfied because the helper is only called after a coin-type network resolves (originless import already errors pre-S-NET, unchanged). Keep the export-wallet originless control as the required no-op proof. No edit-site change.

- **M2 — H9 RED test must use `--format bitcoin-core` (the only multi-entry producer); name it.** The plan's H9 RED is "a 2-entry mixed blob `[Bitcoin, Testnet]`" but does not state the format. Verified only `bitcoin-core` emits a multi-`ParsedImport` Vec (`bitcoin_core.rs:208-210`); the other 8 parsers return a single-element `vec![…]`, so a mixed-network RED is structurally impossible for them. **Fix:** Phase 2 should specify the H9 RED uses `--format bitcoin-core` with a 2-descriptor blob (descriptor[0] on `…/0'/…`, descriptor[1] on `…/1'/…`) so `first()==Bitcoin` passes the old check and the testnet entry is caught per-entry. Otherwise an implementer could write a single-entry RED that never exercises the `iter_mut()` rebind bug (vacuous). The homogeneous positive control `[Bitcoin, Bitcoin]` must likewise be a 2-descriptor bitcoin-core blob.

- **M3 — L3 RED needs a real legacy-fallback fixture; confirm `select_dominant_bip` reaches the `None` arm.** The plan correctly requires a *legacy top-level-xpub* coldcard blob (`deriv_path_str_opt == None`) so `raw_account` is interpolated at `coldcard.rs:266`. Verified the `None =>` arm is the only `raw_account` interpolation site and per-bipN uses `Some(s)`. **Fix (guidance):** the RED must use a fixture that drives `select_dominant_bip` to return `deriv_path_str_opt == None` (top-level `xpub` + `account`, no per-bipN sub-object with a `deriv` string — cf. the named `coldcard-mk1-legacy-bip84-mainnet.json` pattern). The implementer should assert at write-time that the RED is RED (the plan already mandates RED-first); flag that a per-bipN fixture renders it vacuous (the plan says this — just make the fixture-construction step explicit so the bound check is provably exercised).

- **M4 — prompt-cited `coldcard.rs:268` is off-by-2; live `format!` is `:266`.** The review prompt cites the L3 legacy fallback at `coldcard.rs:268`; the spec cites `:266`. Live source: the `format!("m/{purpose}'/{coin_type}'/{raw_account}'")` is at `:266`. No action needed (the plan mandates re-grep at write time, and the spec line is correct), recorded for the audit trail.

- **M5 — Display-string rewrite is behaviorally safe (confirms the spec's M-6 claim).** Verified no test asserts the current Display string `"xpub network {} does not match --network {}"` (`error.rs:834`); the only `xpub_network`/`expected` readers are Display, detail_json, the def, and the `exit_code()==2` unit test. The `cli_import_wallet_envelope_v0_27_0.rs:155` hit is a test *description label*, not the error string. The variant rename + Display rewrite are compile-forced and test-safe. No action; recorded as positive confirmation of the spec's M-6.

---

## Cross-checks against the prompt's 8 required verifications

1. **Edit sites exist as cited:** ✅ all verified (table above). Only drift is the prompt's own `:268` vs live `:266` (M4) and the originless-control being an export test (M1) — neither is a plan error.
2. **Two-axis separation:** ✅ **preserved and correct.** Axis-1 (H9) → extend `first()`→all entries, **reuse `ImportWalletNetworkClassMismatch`, exit 1** (`error.rs:576`). Axis-2 (H15 et al.) → new `NetworkMismatch`, **exit 2** (`error.rs:587`). The plan §"Phase 2" and the spec §2.3.1 keep them STRICTLY separate; no site routes H9 to NetworkMismatch/exit 2. Exit codes verified against live `:576`/`:587`.
3. **Over-rejection / no-op + full-suite:** ✅ guarded. The helper is a no-op when no network is asserted (caller-side skip; §2.2). The plan commits a FULL `cargo test -p mnemonic-toolkit` package sweep (Phase 5) as the zero-false-reject proof, plus per-site positive controls and the originless control. The three named legitimate cases (WIF testnet→testnet, originless tpub via export, consistent tpub-on-coin-type-1) are all covered and none would be newly rejected. The only imprecision is *where* the originless no-op bites for the import leg (M1) — does not weaken the guard.
4. **TDD integrity per finding:** ✅ all 9 are genuinely RED-first and non-vacuous, with two construction caveats: H9 RED must be `--format bitcoin-core` multi-entry (M2) and L3 RED must hit the legacy `None` fallback (M3). L1 asserts a stderr WARNING at exit 0 (not a reject) — verified the plan requires `assert on stderr, NOT a non-zero exit`. H9 RED is a genuine `[Bitcoin, Testnet]` blob; positive control is genuine same-class `[Bitcoin, Bitcoin]`.
5. **L3 ride-along:** ✅ in-scope and correct. Reject on `account > u32::MAX` (not saturate), `ImportWalletParse`/exit 2, firewalled from the network helper, its own test. Not scope creep — same parser family, metadata-only items rot if deferred; the spec offers a defer-with-FOLLOWUP fallback if the reviewer prefers minimal blast radius. Folding is the reasonable lean.
6. **WIF (L11):** ✅ sound. The plan extracts the WIF's OWN `pk.network` (`PrivateKey.network: NetworkKind`, verified bitcoin-0.32.8 `key.rs:402`) — NOT a BIP-32 version — and passes it as `decoded`. The current code discards it (`sentinel_xpub.network = network.network_kind()`); the fix asserts before building the sentinel.
7. **Variant rename ripple:** ✅ complete. The `xpub_network→decoded_network` / `expected→expected_network` rename + new `context` field compile-forces exactly the 3 reader arms the plan enumerates (Display `:830`, detail_json `:913`, unit test `:1013`); `exit_code`/`kind` use `{ .. }` and are rename-immune; no hidden reader (grep-confirmed). `network_kind_name()` const-fn feeds the `&'static str` fields. No site reads the old names beyond the enumerated 4.
8. **SemVer/lockstep/oracle/version-sites:** ✅ MINOR `0.63.0` correct (new fail-closed rejections of previously-accepted input = behavior-breaking pre-1.0). No clap flag / `--json`-flag / dropdown change → no `schema_mirror`, no manual leg, no sibling companion. The `detail_json` error-shape delta is honestly noted as outside `schema_mirror` scope (which gates flag-NAMES + dropdown VALUES, not `--json` error envelope) — confirmed correct per CLAUDE.md. Oracle gate (add-only-rejection, existing AGREE rows stay byte-identical, DISAGREE asserts in CLI/unit suites) is sound. Version sites complete: `Cargo.toml` + both READMEs + `scripts/install.sh` + `fuzz/Cargo.lock` + `CHANGELOG.md` (tag-gated). The `0.63.0` collision with the paused `feature/own-account-subset-search` branch is correctly flagged as first-to-ship-claims-it / do-not-touch.

---

## Verdict

All 5 Minors are guidance/precision refinements (test-construction explicitness, control-anchoring prose, an off-by-2 in the *prompt* not the plan). None blocks implementation under the R0 gate: the architecture is sound, every load-bearing citation resolves against `ac4eead0`, the two-axis separation and over-rejection guard are correct and live-verified, and all 9 RED tests are genuinely RED-first and non-vacuous (M2/M3 sharpen *how* to construct two of them, but the plan's assertions are already correct).

**PLAN R0 ROUND 1: 0C / 0I — GREEN (0C/0I).**

Recommend the implementer fold M1–M3 into the Phase-2/Phase-4/Phase-5 prose at write time (name `bitcoin-core` for the H9 RED; assert the L3 fixture drives the `None` legacy arm; keep the originless control on the export-wallet leg) — these are pre-coding clarifications, not gate blockers. M4/M5 are audit-trail confirmations requiring no change.
