# Phase 3 R0 review — wallet-import v0.26.0

**Date:** 2026-05-18
**Reviewer:** opus architect
**Commit under review:** `e7a19f9` (`phase 3: Bitcoin Core listdescriptors parser + xprv refusal + 14 cells`)
**Worktree:** `.claude/worktrees/wallet-import-export-multiformat-brainstorm`

**Verdict:** YELLOW — 1 Critical, 2 Important, 4 Minor. Fold C1 + I1 + I2 before Phase 4 review.

The Phase 3 implementation is structurally clean. The `BitcoinCoreParser::parse` flow, sniff predicate, `--select-descriptor` filter helper, and aggregated dropped-fields NOTICE work as designed. C1 is a real watch-only-invariant weakness on testnet (`tprv` bypasses substring check); I2 is a Phase-4 dependency (canonicalize_bitcoin_core needs `wallet_name`).

## Critical

### C1 — xprv substring check misses `tprv`/`uprv`/`vprv`/`yprv`/`zprv` private-key prefixes (testnet/SLIP-132)

**Site:** `crates/mnemonic-toolkit/src/wallet_import/bitcoin_core.rs:192` — `if desc_with_csum.contains("xprv")`.

Bitcoin Core's `listdescriptors true` on a testnet/signet/regtest wallet emits `tprv`-prefixed extended private keys. SLIP-132 also defines `yprv`/`zprv`/`uprv`/`vprv`/`Yprv`/`Zprv`/`Uprv`/`Vprv` private-key prefix variants. None of these are caught by `contains("xprv")`.

A user running `bitcoin-cli -signet listdescriptors true` hits a `tprv`-bearing descriptor that bypasses the refusal check. The downstream `concrete_keys_to_placeholders` regex at `pipeline.rs:38` requires `[xtyzuvYZUV]pub` literal so the parse eventually fails with `"no [fp/path]xpub keys found in descriptor"` (exit 2) — but the user-facing template is misleading and the watch-only invariant is enforced by accident rather than by design.

**Fix:** Strip `#<csum>` first, then check for any `[xtuvyzYZUV]prv[A-HJ-NP-Za-km-z1-9]+` token. Apply the same template at `error.rs:557-562` (universal for both mainnet and testnet).

**SPEC dependency:** SPEC §5.2 step 2.a says "if `desc` contains `xprv`" — narrow literal. Amend SPEC OR fold implementation to broader match.

## Important

### I1 — xprv substring check has false-positive on BIP-380 checksum

**Site:** `bitcoin_core.rs:192` — substring check operates on desc INCLUDING the `#<csum>` trailer.

BIP-380 8-char checksum alphabet is `qpzry9x8gf2tvdw0s3jn54khce6mua7l` — all four chars `x`, `p`, `r`, `v` are in this set. Benign descriptor whose checksum happens to contain the 4-char substring `xprv` would be incorrectly refused. Probability ~5×10⁻⁶ per descriptor — rare but real false-positive class.

**Fix:** Strip the `#<csum>` trailer before the substring check. Pairs with C1 fix.

### I2 — `wallet_name` silently discarded; Phase 4 canonicalize requires it

**Site:** `bitcoin_core.rs:108-169` — `wallet_name` is never extracted from `obj`.

SPEC §7.3.2 line 272 says `wallet_name: preserved (metadata)` for canonicalize. Phase 4's semantic round-trip needs to re-emit `wallet_name` to match source blob. Phase 3 discards silently without filing FOLLOWUP.

**Fix:** Add `wallet_name: Option<String>` to `CoreSourceMetadata` (or top-level `ParsedImport`) and populate from `obj.get("wallet_name").and_then(Value::as_str).map(str::to_string)`.

## Minor

### M1 — `WalletFormatParser::parse` signature deviates from SPEC §8.1

`mod.rs:41` adds `stderr: &mut dyn Write`. SPEC §8.1 lines 311-312 shows only `blob`. Amend SPEC in lockstep or file FOLLOWUP.

### M2 — Dropped-fields NOTICE uses Rust Debug format `{:?}`

`bitcoin_core.rs:154-160` produces `dropped wallet-state fields ["timestamp", "next", "next_index"]: ...` — bracket+quote+comma-space is noisy. Suggested fix: `aggregate_dropped.join(", ")`. Pin format before Phase 4 canonicalize.

### M3 — `core_multipath_split_to_receive_change` cell name misleading

Cell asserts `bundles=1` (no split) but name says "split_to_receive_change". Implementation correctly preserves multipath intact per SPEC §5.2.b. Rename to `core_multipath_preserved_single_entry`.

### M4 — Multi-match active-receive/active-change scenario not test-covered

Cells §3.5 + §3.6 each cover single-match case. Multi-match path in `apply_select_descriptor` exercised structurally but not in integration cells.

## Per-design-note verdict

- **Sniff predicate conservative:** ACCEPT. All 5 SPEC §6.1 checks in correct order. Vendor-marker list matches SPEC byte-exact.
- **Dropped-fields aggregated:** ACCEPT with caveat M2 (Debug format).
- **ByIndex out-of-range exit-2:** ACCEPT (defensible). Tier-routing inconsistency between out-of-range-N (tier 2) and zero-match-active-* (tier 1). SPEC §5.3 silent on out-of-range. Document inline OR pick consistent tier.
- **Cross-entry coin-type heterogeneity allowed:** ACCEPT. Each ParsedImport carries own Network per SPEC §8.1.
- **No Phase 2 silent modifications:** VERIFIED. Only edit to `bsms.rs` is `source_metadata: None` initializer.

## Notable strengths

1. `extract_threshold` regex handles `thresh`/`multi`/`sortedmulti` correctly with `OnceLock` caching.
2. `apply_select_descriptor` cleanly separated for Phase 5 reuse.
3. Error-message prefix re-tagged `bsms:` → `bitcoin-core:` when reusing pipeline.
4. `parse_range_field` handles all four JSON shapes robustly.
5. Sniff unit tests comprehensive (6-case SPEC §6.1 matrix).

## Recommendation

Fold C1 + I1 + I2 before declaring Phase 3 GREEN. M1–M4 inline or FOLLOWUP at architect-reviewer discretion. Note: Phase 4 implementer is dispatched in parallel; folds will land after Phase 4 completion to avoid concurrent edits.
