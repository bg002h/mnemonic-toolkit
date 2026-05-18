# Phase 0 R0 review — wallet-import v0.26.0

**Date:** 2026-05-18
**Reviewer:** opus `feature-dev:code-architect`
**Worktree:** `.claude/worktrees/wallet-import-export-multiformat-brainstorm`
**Recon under review:** `/tmp/phase-0-recon.md`

**Verdict:** YELLOW — proceed with §7.0 amendments **expanded** before Phase 1 first commit lands.

The recon report is technically sound on the gates it actually exercised, but it locked §0.7 to option (b) without surfacing two non-trivial edge cases that the SPEC's own test corpus (§10.2 testnet mainnet × 2 axis) will hit. Two additional plan-doc-vs-source defects went unnoticed in §0.6 (path shortcut) and §0.7 (3-way ambiguity). Net: 0 Critical, 2 Important, 3 Minor — fold both Important findings into the §7.0 commit and proceed.

## Critical

(none)

## Important

### I1 — §0.6 GO/NO-GO verified miniscript only, NOT the actual Phase 2 pipeline

**Site:** `/tmp/phase-0-recon.md:128-148` (§0.6 verification block); `crates/mnemonic-toolkit/src/cmd/export_wallet.rs:257-263` (the path the recon used); `crates/mnemonic-toolkit/src/parse_descriptor.rs:747-813` (the path Phase 2 SPEC §4.2 step 7 mandates).

**Defect:** The recon used `mnemonic export-wallet --descriptor "$DESC_LINE"` as the GO/NO-GO verification surface. That code path at `export_wallet.rs:257-263` calls only:

```rust
let d = MsDescriptor::<DescriptorPublicKey>::from_str(desc)?;
d.to_string()
```

— i.e., a thin `MsDescriptor::from_str` parse + re-render. This verifies rust-miniscript v13.0 accepts `sln:older(N)`, `tpub`, BIP-389 multipath, BIP-380 checksum — but it does NOT exercise:

1. `substitute_nums_sentinel` (irrelevant for BSMS, but normative pipeline step at `parse_descriptor.rs:757`).
2. `detect_bare_tr` (also irrelevant, but `parse_descriptor.rs:760-762`).
3. `lex_placeholders` (`parse_descriptor.rs:764`) — REQUIRES `@N` placeholder syntax. BSMS blob has concrete `[fp/path]xpub` keys, so Phase 2 must run `pipeline.rs::concrete_keys_to_placeholders` (plan §2.2) FIRST and THEN call `parse_descriptor`.
4. `resolve_placeholders` (`parse_descriptor.rs:765`) — validates dense `0..n` indexing, classifies Shared vs Divergent paths.
5. `substitute_synthetic` (`parse_descriptor.rs:776`) — substitutes BIP-32-valid synthetic xpubs.
6. `walk_root` → `walk_miniscript_node` (`parse_descriptor.rs:555-700`) — the AST → `md_codec::Node` translation.

The user's flagship blob `wsh(thresh(2, pkh(...), s:pk(...), sln:older(32768)))` decomposes (per miniscript) into `Wsh → Thresh{k:2, children: [Check(PkH), Swap(Check(PkK)), Swap(OrI(False, ZeroNotEqual(Older(32768))))]}`. I cross-checked the walker arms at `parse_descriptor.rs:555-700`: every required terminal (`PkH`, `PkK`, `Check`, `Swap`, `OrI`, `ZeroNotEqual`, `Older`, `False`, `Thresh`) is implemented. So the path SHOULD work — but it was not empirically verified.

**Severity:** Important (not Critical) because the walker arms exist; failure mode is low-probability but possible.

**Fold:** Run the full Phase 2 path empirically. Construct a `@N`-placeholder-form version of the BSMS descriptor + `(ParsedKey, ParsedFingerprint)` lists, then invoke `parse_descriptor::parse_descriptor` directly via a 20-LOC harness or via an existing test fixture that already exercises that pipeline. Document the result in the recon report addendum.

### I2 — §0.7 option (b) cannot disambiguate testnet from signet/regtest

**Sites:** `/tmp/phase-0-recon.md:158-185` (§0.7 lock + rationale); `crates/mnemonic-toolkit/src/network.rs:22-27` (`coin_type`); `design/SPEC_wallet_import_v0_26_0.md:317` (`network: bitcoin::Network`); `design/SPEC_wallet_import_v0_26_0.md:458-462` (test corpus testnet axis).

**Defect:** The recon's option (b) rationale picks `bitcoin::Network::Testnet` for `coin_type=1`, but `network.rs:22-27` documents that coin_type=1 maps to **three** CliNetwork variants (Testnet, Signet, Regtest). BIP-129 BSMS and Bitcoin Core `listdescriptors` don't distinguish — the blob is intrinsically ambiguous. Option (a) (xpub-prefix) has the same 3-way ambiguity (`tpub` is shared per `slip0132.rs:54-56`). Option (b) is not worse; the toolkit's choice of `Network::Testnet` as the canonical interpretation must be **explicit SPEC text**.

**Multi-cosigner coin-type heterogeneity:** SPEC §4.2 currently produces a single `network: bitcoin::Network` field. If cosigner 0 has `m/48'/0'/...` (mainnet) and cosigner 1 has `m/48'/1'/...` (testnet), the SPEC has no rule. Lock it.

**Fold:** Expand §7.0.a SPEC text per the recommendation block below.

## Minor

### M1 — Plan-doc §2.2 adapter return signature vs SPEC §4.2 step 7 wording

Cosmetic: SPEC §4.2 step 7 uses `parsed_keys, parsed_fingerprints` (plural snake_case); plan-doc §2.2 returns `Vec<ParsedKey>, Vec<ParsedFingerprint>`. Optional plan-doc-amendment commit; not blocking.

### M2 — Plan-doc §0.7 option (a) prose "BIP-43 coin-type" intermediate hop is misleading

`slip0132.rs::normalize_xpub_prefix` doesn't map SLIP-132 variants to BIP-43 coin-type; it maps prefix bytes to neutral xpub/tpub. The recon correctly rejected option (a) for the `tpub`-stripping false-mainnet trap. Cosmetic plan-doc cleanup.

### M3 — §0.4 / §0.5 well-anchored; §0.6 over-confident given the path-shortcut

§0.4 and §0.5 anchored to source + existing test passes. §0.6 should have run the actual Phase 2 path end-to-end. I1 fold path is sufficient. Surface as feedback for future Phase 0 R0 reconnaissance patterns.

## Plan-doc command defects (R1 from recon) — confirmed

All 3 defects in the recon are real:
- §0.3: `gui-schema --classify-flags` — flag does not exist; only `--classify-descriptor` exists per `gui_schema.rs:1262`.
- §0.6: `convert --from descriptor=` — `descriptor` is not a `NodeType`.
- §0.6.a: same.

Spot-checked Phase 1 §1.3, Phase 2 §2.4, Phase 6 §6.13 — all buildable. No additional command defects discovered.

## §7.0 amendment list — fold §7.0.f INTO the single §7.0 commit

Project convention `[[feedback-r0-must-read-source-off-by-n]]` + v0.21.0 / v0.25.0 precedents: end-of-phase folds go inline. One commit titled `design: pre-cycle SPEC + BRAINSTORM amendments for wallet-import v0.26.0` carries all 6 fold items (§7.0.a-f).

## Expanded §7.0 amendment list (RECOMMENDED FOR PHASE 1 FIRST COMMIT)

Pre-existing §7.0.a-e stand as written, but **expand §7.0.a** per I2; **add §7.0.f** per R1+R2+R3.

The expanded §7.0.a normative text for SPEC §4.2 step 8 (proposed):

> **8. Network detection from origin paths.** Extract the `coin_type` child number (hardened path component at index 1) from the first parsed cosigner's `[fp/path]` origin annotation. Map: `0'` → `bitcoin::Network::Bitcoin`; `1'` → `bitcoin::Network::Testnet`. Signet and regtest are not distinguishable from testnet via origin-path inspection in either BIP-129 BSMS or Bitcoin Core `listdescriptors` — both use coin-type `1`. Wallets intrinsically running on signet/regtest are imported as testnet; users must apply `--network signet|regtest` post-import via a downstream subcommand if signet/regtest semantics are required. FOLLOWUP: `wallet-import-signet-regtest-disambiguation`. Cosigner-to-cosigner coin-type mismatch (e.g., cosigner 0 has `m/48'/0'/...`, cosigner 1 has `m/48'/1'/...`) → exit 2 `ImportWalletParse` per §2.3.

## Notable strengths

1. **Source-grounded findings.** R1+R2+R3 all trace to actual source with file:line citations per `[[feedback-r0-must-read-source-off-by-n]]`.
2. **Empirical lexsort confirmation is load-bearing.** §0.1 empirical test (forward vs reverse → DIFFERENT outputs) directly informs Phase 5 §5.8 seed-overlay design.
3. **Decision-locking discipline at §0.7.** Recon committed to option (b) with a clear rationale rather than punting forward.
4. **No SPEC violations escalated.** Recon correctly distinguishes "plan-doc inaccuracy about codebase" from "SPEC inaccuracy" and chooses the right fold target for each.

## Recommendation

**Proceed to Phase 1 first commit with the expanded §7.0 amendment list (§7.0.a-f).** Fold I1 + I2 inline.

**Optionally add §0.6.b sub-task to Phase 0** per I1: a 20-LOC harness exercising the full Phase 2 pipeline end-to-end against the user's BSMS blob. If skipped, accept the residual Phase 2 §2.4 first-cell risk explicitly.

---

**Verdict reaffirmed: YELLOW.** Fold I1 + I2; proceed to Phase 1 §7.0 commit. No Critical findings; cycle is unblocked.
