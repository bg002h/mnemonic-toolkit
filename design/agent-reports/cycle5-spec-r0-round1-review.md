# cycle-5 "S-NET" network-provenance invariant — SPEC R0 review, ROUND 1

**Artifact under review:** `design/BRAINSTORM_cycle5_snet_network_invariant.md`
**Reviewer:** opus software architect (adversarial R0)
**origin/master SHA:** `ac4eead0` (toolkit `0.62.1`)
**Date:** 2026-06-21
**Gate:** mandatory pre-implementation R0 — NO code until 0 Critical / 0 Important.

All citations re-grepped against `git show origin/master:<path>` at review time; the spec's
claims were treated as hypotheses, not evidence. bitcoin crate = pinned `0.32.8` (Cargo.lock).

---

## Summary of verification (load-bearing claims that CHECK OUT)

These are confirmed against live source and are NOT findings — recorded so the next round
does not re-litigate them:

- **`ToolkitError::NetworkMismatch` is genuinely dead, exists for this rule.** `error.rs:273-277`
  variant with `#[allow(dead_code)]`, fields `xpub_network: &'static str` / `expected: &'static str`.
  `exit_code → 2` (`:587`), `kind → "NetworkMismatch"` (`:656`), Display (`:830`, message
  `"xpub network {} does not match --network {}"`), `detail_json` (`:913`), unit-test construction
  (`:1013`, `xpub_network:"main"`). `git grep ToolkitError::NetworkMismatch` returns ONLY the def +
  these 5 arms. Zero production construction. **CONFIRMED.** (Spec cites `273-277`; the actual variant
  body spans `273-277` with `NetworkMismatch {` at `274` — accurate.)
- **`CosignerSpec` precedent at `synthesize.rs:776-790`** with predicate `c.xpub.network != network.network_kind()`
  → `CosignerSpec { cosigner_idx, message }`. **CONFIRMED verbatim**, including the `human_name()`
  message form.
- **`network.rs` is the correct helper home.** Module doc line 1-4 states "§4.3 (network/xpub
  cross-check via Xpub::network field)". `CliNetwork::network_kind()` (`:30-35`) collapses
  Testnet|Signet|Regtest → `NetworkKind::Test`. `human_name()` (`:49-57`). `coin_type()` (`:22-27`).
  Test `network_kind_mainnet_vs_test` (`:82-86`) asserts the collapse. **CONFIRMED.**
- **`impl From<Network> for NetworkKind` exists** in bitcoin-0.32.8 `network.rs:50` and maps
  `Bitcoin→Main`, `Testnet|Testnet4|Signet|Regtest→Test`. **CONFIRMED — and crucially the match is
  exhaustive incl. `Testnet4`, so no panic risk.**
- **`PrivateKey.network: NetworkKind`** (bitcoin-0.32.8 `crypto/key.rs:402`) — the WIF (L11)
  extraction is type-sound. **CONFIRMED.**
- **2-way NetworkKind granularity is correct AND maximal (the crux, item 2).** bitcoin-0.32.8
  `bip32.rs:27` comment: *"Version bytes for extended public keys on **any of the testnet networks**."*
  There is exactly ONE testnet xpub version byte (`043587cf`) covering testnet/signet/regtest, and ONE
  mainnet (`0488b21e`). A 4-way Network check is **impossible at the xpub layer** — the xpub literally
  cannot encode signet-vs-testnet-vs-regtest. Address derivation depends only on NetworkKind + HRP;
  signet and testnet share `KnownHrp::Testnets` (`network.rs:43`, identical `tb1` addresses), and the
  sub-network HRP choice comes from the *CLI* `--network` (coin-type-1-bound, already enforced).
  **There is no case where Network-level matters but NetworkKind-level lets a wrong address through.**
  VERDICT: granularity is correct and complete.
- **SLIP-132 neutralization preserves NetworkKind (soundness of the cross-check).** `slip0132.rs::neutral_for`
  maps `ypub/zpub→xpub` (mainnet) and `upub/vpub→tpub` (testnet), so the post-`normalize_xpub_prefix`
  decoded `Xpub.network` faithfully reflects the original prefix's family. The cross-check therefore
  catches a mainnet `zpub` (or `xpub`) on a coin-type-1 path. **CONFIRMED.**
- **H9 is strictly override-scoped.** Without `--network`, `emit_json_envelope` (`import_wallet.rs:1505+`)
  emits `network: network_human_name(p.network)` PER ENTRY — a `[Bitcoin, Testnet]` blob is per-entry
  correct. The bug lives ONLY in the `--network` override block (`:1191-1209`): guard reads
  `parsed.first()`, write loops `parsed.iter_mut()` rebinding ALL. **CONFIRMED verbatim.**
- **bitcoin-core is the multi-entry parser** (`bitcoin_core.rs:208-211` pushes one `ParsedImport` per
  descriptor, each with its own `network` via `network_from_origins(&origins, idx)`), so the
  heterogeneous `[Bitcoin, Testnet]` Vec is reachable. **CONFIRMED.**
- **Oracle gate argument is sound (item 8).** `bitcoind_differential.rs` is `-chain=main`, all-mainnet
  xpubs, `#[ignore]`/env-gated, `deriveaddresses` corroboration. S-NET adds only rejections on
  mismatched input the harness never feeds; existing AGREE rows stay byte-identical; DISAGREE asserts
  (CLI exit≠0) structurally cannot be oracled (Core rejects a mainnet xpub off-main anyway). The "no
  new oracle rows required; DISAGREE asserts live in CLI/unit suites" disposition is **correct**.
- **Existing fixtures are network-consistent**, so the new reject is plausibly a no-op for the suite:
  testnet/coin-type-1 paths pair with `tpub` (`cli_descriptor_concrete.rs:4`,
  `cli_import_wallet_bitcoin_core.rs:29`); mainnet/coin-type-0 with `xpub`. (But see I3 — the suite was
  NOT exhaustively swept; the spec asserts zero false-reject without enumerating the at-risk tests.)

---

## Critical

**(none)**

The spec is structurally sound: the invariant is correct, the granularity is provably maximal, the
dead variant exists for exactly this purpose, the helper home is right, the H9 layer-analysis is
correct, and the SemVer/lockstep/oracle reasoning holds. No funds-safety hole, no
implementation-blocking architectural error. The findings below are precision/consistency gaps that
must be closed before plan-doc, but none is a design-invalidating Critical.

---

## Important

### I1 — H9 error-variant / exit-code is unspecified and CONFLICTS with the adjacent existing refusal (exit 1 vs spec's exit 2)
**Evidence:** The existing `--network` override cross-class refusal uses
`ToolkitError::ImportWalletNetworkClassMismatch` → **exit 1** (`error.rs:576`), proven by
`cli_import_wallet_network_override.rs:67` (`testnet_blob_override_to_mainnet_refused` asserts
`status.code() == Some(1)`). The spec's H9 RED test (§7, row H9) asserts the heterogeneous-blob case
returns **exit 2** and implies `kind=NetworkMismatch` (the §7 prose lists every reject as "exit 2,
kind=NetworkMismatch" except where noted). But the H9 fix sits in the SAME override block
(`:1191-1209`) immediately adjacent to the existing exit-1 cross-class refusal, and the two conditions
are siblings (both "the asserted/override network is incompatible with the blob"). Shipping a
cross-ENTRY heterogeneity reject at exit 2 / `NetworkMismatch` while the cross-CLASS override reject
stays exit 1 / `ImportWalletNetworkClassMismatch` is an unjustified inconsistency the spec never
addresses (Decision-table row #9 specifies the *shape* of the fix but is silent on variant/exit).
**Required fix:** §2.3/§3/§7 must pin H9's exact variant + exit and justify it against the adjacent
`ImportWalletNetworkClassMismatch`. Either (a) H9 reuses `ImportWalletNetworkClassMismatch` (exit 1)
for consistency with the override family — in which case §7's H9 row must change to exit 1 and drop the
`NetworkMismatch` kind; or (b) H9 uses `NetworkMismatch` (exit 2) and the spec explicitly argues why
the heterogeneous-blob case is a different error class than the cross-class case (and accepts two
different exits for two refusals in the same code block). The current spec asserts exit 2 in §7
without reconciling it with the live exit-1 sibling — a plan-doc written from this spec would ship an
inconsistency.

### I2 — H9 positive control `[Bitcoin] + --network signet → exit 0` is FACTUALLY WRONG (contradicts live behavior)
**Evidence:** §7 row H9, positive-control column: *"`[Bitcoin] + --network signet` (same class) → exit 0"*.
This is false. A `[Bitcoin]` blob is coin-type-0 (`override.coin_type()==0`); `--network signet` is
coin-type-1 (`CliNetwork::Signet.coin_type()==1`, `network.rs:24`). That is a CROSS-class override,
which the existing code REFUSES — proven by `cli_import_wallet_network_override.rs:76-79`
(`mainnet_blob_override_to_signet_refused` asserts exit 1). The spec's positive control would FAIL
(it exits 1, not 0). A plan-doc lifting this control verbatim would write a test that does not pass,
or — worse — "fix" the production code to make a coin-type-0→signet override succeed, silently
regressing the live cross-class guard.
**Required fix:** Replace the bogus control. The valid same-class controls are: `[Bitcoin, Bitcoin] +
--network mainnet → exit 0` (already listed) and `[Testnet] + --network signet → exit 0` (testnet
coin-type-1 → signet coin-type-1, same class — matches the live `testnet_blob_override_to_signet`
passing test at `:51-55`). The spec confuses mainnet/coin-type-0 with the testnet/coin-type-1 class.

### I3 — "zero false-reject" is asserted, not proven: the existing suite was not swept, and originless / no-coin-type inputs are unhandled
**Evidence:** §3 ("The cost of a false-reject … is zero because the positive controls prove consistent
inputs pass unchanged") and §7 ("every reject site MUST ship at least one consistent-input control")
guarantee a per-site control but do NOT guarantee the *existing* suite stays green. Two concrete
gaps:
  (a) **No enumeration of at-risk existing tests.** coin-type-1 paths appear across many test files
  (`cli_import_wallet_bitcoin_core.rs`, `cli_descriptor_concrete.rs`, `cli_export_wallet_*`,
  `cli_convert_address.rs`, etc.). Spot-checks show they ARE network-consistent (tpub on `…/1'/…`),
  so the reject is *probably* a no-op — but the spec ASSERTS this without enumerating the at-risk
  set. Per the project's full-suite-R0 discipline (MEMORY `feedback_r0_review_run_full_package_suite`),
  the plan-doc must run the FULL `cargo test -p mnemonic-toolkit` suite and the spec should commit to
  that as the no-over-rejection proof, not just the new per-site controls.
  (b) **Originless / coin-type-absent descriptors.** `cli_descriptor_concrete.rs:174`
  (`wpkh(tpubD…/0/*)`) and similar carry NO `[fp/path]` origin → there is no coin-type to assert
  against. The spec's import cross-check idiom is "iterate slots' `xpub.network` against the
  coin-type-derived network", but `coin_type_from_path` REQUIRES ≥2 path components
  (`descriptor.rs:202`, errors otherwise) — an originless or short-path key has no asserted side. The
  spec is silent on this: the cross-check MUST be SKIPPED (not errored) when no coin-type is derivable,
  else a legitimate originless `tpub` descriptor is over-rejected (a NEW availability bug — exactly
  the funds-hole class the WARN-not-reject L1 disposition was careful to avoid). (M14/L11/build-L1 are
  unaffected — they assert against `--network`/`pk.network`, both always present — so this is scoped to
  the import parsers and M13.)
**Required fix:** (1) §3/§7 must commit the FULL-suite-green requirement as the no-over-rejection proof
and name the no-coin-type case; (2) §2.2/§3 must specify that the import cross-check is a no-op when
the parser has no coin-type-derived network (originless / sub-2-component origin), with a positive
control (originless tpub descriptor → still accepted). Without this the "zero false-reject" claim is
unsubstantiated for a reachable legitimate input.

---

## Minor

### M-1 — `slip0132.rs` is mis-pathed in two places
The source-SHA table (line 30) and the fix-site map (§3 rows 9/10) cite
`src/wallet_import/slip0132.rs`. The actual path is **`src/slip0132.rs`** (crate root) — confirmed by
`git grep fn apply_xpub_prefix` → `crates/mnemonic-toolkit/src/slip0132.rs:108`. Line numbers
(108/197) and bodies are correct; only the module prefix is wrong. Per the project's "grep-verified at
write time" discipline this is real citation drift. Fix the path before plan-doc lifts it.

### M-2 — L3 RED test must hit the legacy fallback branch, not the per-bipN branch
§7 row L3 uses `"account": 4294967296`. But the truncation (`coldcard.rs:237-241` `as u32`) only
*manifests* in the legacy top-level-xpub fallback (`:266`, `format!("m/{purpose}'/{coin_type}'/{raw_account}'")`),
reached ONLY when `deriv_path_str_opt == None` (no per-bipN `deriv` field). The per-bipN sub-objects
use the sub-object's own `deriv` string and never interpolate `raw_account`. The RED test fixture must
therefore be a *legacy top-level-xpub* Coldcard blob (cf. existing
`coldcard-mk1-legacy-bip84-mainnet.json`), not a per-bipN one, or the test is vacuous (the truncation
is silently unobservable). Spec should name the fixture shape. Also confirm exit/kind: §7 says
`ImportWalletParse` (exit 2) which is consistent with §5.1's REJECT lean — fine, but the firewall from
`assert_network_agrees` (Decision #10) must be explicit in the plan (separate function, separate test;
NOT a clause of the network helper) — the spec states this; carry it forward.

### M-3 — variant-rename ripple is under-counted
§2.3 proposes renaming `xpub_network→decoded_network` / `expected→expected_network` and adding
`context`. The spec lists Display (`:830`), `detail_json` (`:913`), and the unit test (`:1013`) as the
edit sites. That is the complete set on `error.rs` (verified: `git grep` shows exactly the def + 5
arms, of which `exit_code`/`kind` use `{ .. }` and need no field edit). The rename itself is clean.
BUT the `detail_json` JSON KEY change (`"xpub_network"→"decoded_network"`, add `"context"`) is a
`--json` error-output wire-shape change. §6.2 claims "no `--json` wire-shape change." The error
`detail_json` shape is technically a JSON wire surface (any consumer parsing `mnemonic … --json`
error envelopes sees the new keys). It is NOT clap-flag schema (so `schema_mirror` genuinely isn't
triggered — that gate is flag-NAME only, correctly per CLAUDE.md), and the variant was dead so no
existing consumer parses it today — so the practical blast radius is zero. Still, §6.2's blanket "no
`--json` wire-shape change" is slightly overstated; tighten to "no clap-schema change; the only
`--json` delta is the formerly-unreachable `NetworkMismatch.detail_json` keys, which no consumer can
currently observe." Cosmetic, but the project tracks wire-shape claims carefully.

### M-4 — Decision #4 keeps `&'static str` (sufficient) but the rename note hedges
Item 4 (sufficiency of `&'static str`): CONFIRMED sufficient. Every call site's asserted/decoded
network resolves to exactly one of two static names via `network_kind_name()`; no site needs a dynamic
coin-type number or xpub prefix IN the message (the `context: &'static str` site-label is a closed
set; the Display string interpolates only the two static names + the static context). No site
requires `String`. The §2.3 parenthetical "if R0 prefers the minimal footprint, keep the original two
field names" is fine — but R0 should RATIFY one shape so the plan-doc isn't left with an open lean.
**Recommendation (ratified):** take the rename (`decoded_network`/`expected_network` + `context`) — it
genuinely covers the WIF case where "xpub_network" is a misnomer, and the cost is the three mechanical
arm edits already enumerated. Not a finding; recorded as the R0 ratification of ratification-point (i).

### M-5 — L1 "infer from keys" feasibility is real but under-specified for `DescriptorPublicKey` variants
§4 step 1 says "Walk `vp.descriptor` keys (`for_each_key`-style), read each `Xpub::network`."
`vp.descriptor` is `MsDescriptor<DescriptorPublicKey>` (`gate.rs:39`). miniscript's `for_each_key`
exposes `&DescriptorPublicKey`, which is an enum (`Single` / `XPub(DescriptorXKey<Xpub>)` /
`MultiXPub(...)`). Reading `.network` requires matching the `XPub`/`MultiXPub` arms (`.xkey.network`)
and deciding what to do for a `Single` raw-pubkey leaf (no network — skip). The spec hand-waves this
as "read each `Xpub::network`." Feasible (the keys DO carry network), but the plan-doc must specify the
`DescriptorPublicKey` arm handling and the all-`Single` fallback (where no network can be inferred →
keep the `--network` default, no warning). Not a blocker; a precision gap for the plan.

### M-6 — Display message wording will change for the existing (dead) variant
The current Display (`error.rs:830`) is `"xpub network {} does not match --network {}"`. §2.3 rewrites
it to the family-collapse-explicit form. This is fine (the variant was dead, no test asserts the old
string except the exit-code test at `:1013` which doesn't check Display). Just confirm in the plan that
no test asserts the OLD Display string (verified: only `exit_code()` is asserted at `:1013`). Carry as
a no-op note.

---

## Item-by-item disposition of the review checklist

1. **Citations re-verified.** All load-bearing citations CHECK OUT except the `slip0132.rs` path
   (M-1, minor). `NetworkMismatch` dead at `:273-277` ✓, exit-code arm `:587` ✓, `CosignerSpec`
   `:776-790` ✓, `From<Network> for NetworkKind` ✓, all 11 fix sites located (import parsers
   specter:370/sparrow:591/bsms:386/coldcard_multisig:679/693/electrum:660, convert xpub-prefix
   :1100-1113 + guard :922-924, wif :1480-1491 / :1217, export :742, build :470/476/480) ✓. The
   spec's line numbers are accurate to ±3 throughout.
2. **2-way NetworkKind granularity — CORRECT AND MAXIMAL.** Authoritatively confirmed (bip32.rs:27 +
   the exhaustive `From<Network>` map + the shared-HRP collapse). A 4-way check is impossible at the
   xpub layer; no wrong address slips through a 2-way check. No finding.
3. **H9 per-entry closure — the LAYER analysis is correct; the variant/exit and one positive control
   are WRONG.** The fix is at the right layer (override block, not `first()`), and the spec correctly
   identifies guard+rebind must read the same per-entry network. But I1 (variant/exit unspecified +
   conflicting with the live exit-1 sibling) and I2 (bogus `[Bitcoin]+signet` control) must be fixed.
   Net: the mixed-blob case IS closed by an override-block per-entry gate; the spec's *test
   specification* for it is defective.
4. **`&'static str` sufficiency — CONFIRMED sufficient (M-4).** No site needs dynamic context.
5. **Per-site dispositions — sound.** L1=WARN is correct (deliverable network-agnostic, only the
   preview HRP; verified `build_descriptor.rs` canonical/bip388 outputs don't consume `--network`).
   Hard-REJECT for import/convert/export is correct fail-closed. Over-rejection risk is REAL but
   narrow (I3: originless/no-coin-type imports) and must be firewalled. Positive controls exist per
   site in §7 but I2/I3 expose two defective/missing controls.
6. **L3 fold — correct to REJECT (not saturate), correctly firewalled (separate sub-item + own test,
   Decision #10).** M-2: the RED test must target the legacy fallback branch or it's vacuous. Not
   scope creep — it rides the same parser family and is cheap; the firewall is explicit.
7. **WIF (L11) — sound.** `PrivateKey.network: NetworkKind` (key.rs:402) feeds `assert_network_agrees`
   directly; the generalized "asserted-source network" helper handles it; the sentinel-xpub
   over-emit is the correct thing to reject. No finding.
8. **SemVer/lockstep/oracle — MINOR 0.63.0 correct; no schema_mirror/manual/companion (M-3 nuance on
   the `detail_json` wire claim); oracle-gate argument sound.** The version-collision note with
   `feature/own-account-subset-search` is correctly handled (first-to-ship claims 0.63.0).
9. **TDD integrity — each finding has a RED + a positive control, but two are defective.** H9's
   positive control is factually wrong (I2); L3's RED is vacuous unless it hits the legacy branch
   (M-2). The originless positive control is missing (I3). The helper unit tests (§7) are sound.

---

## Verdict

Criticals = 0; Importants = 3 (I1, I2, I3); Minors = 6 (M-1…M-6).

**R0 ROUND 1: 0C / 3I — RED.**

Three Important findings block the gate:
- **I1** — H9's error variant + exit code is unspecified and conflicts with the adjacent live
  `ImportWalletNetworkClassMismatch` (exit 1) vs the spec's asserted exit 2.
- **I2** — H9 positive control `[Bitcoin] + --network signet → exit 0` is factually wrong (it exits 1
  under live cross-class refusal); confuses the coin-type-0 and coin-type-1 classes.
- **I3** — "zero false-reject" is asserted not proven: no full-suite sweep committed, and the
  originless / no-coin-type import case is unhandled (over-rejection risk for a legitimate input).

Fix all three (and the 6 Minors, especially M-1 path drift and M-2 vacuous-test), persist this review,
re-dispatch round 2. Per CLAUDE.md the reviewer-loop continues after every fold.
