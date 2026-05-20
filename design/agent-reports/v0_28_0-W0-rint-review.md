# Wave 0 integration — architect R-INT review

**Reviewer:** Opus 4.7 (inline architect-style review via feature-dev:code-architect dispatch; sub-agent had no Write tool — parent persisted this artifact)
**Branch:** `release/v0.28.0` @ `de3dc61`
**Cycle commits reviewed:** P0A (`74c6119`), P0B.2 (`5283d85`), P0D (`7281c46`), P0C (`de3dc61`); P0B.1 superseded-by-P0D, PR #35 closed
**Date:** 2026-05-19
**Purpose:** belt-and-suspenders integration check per P0D agent's recommendation; covers cross-PR interaction bugs the per-PR reviews could not see.

---

## Cross-PR interaction findings (Critical / Important / Minor)

### Critical: NONE
### Important: NONE

### Minor

**M1** — Doc-comment in `wallet_import/mod.rs:11-12` still references "v0.26.0 wallet-import formats" + `bitcoin_core (Phase 3)` (vestigial v0.26.0 phase-naming). Per-parser P{N}A sub-phases will naturally refresh as each new module lands. Not load-bearing for build/test gates.

**M2** — `cmd/import_wallet.rs:30-32` head doc-comment enumerates only "Bsms / BitcoinCore → dispatch to corresponding parser" for the auto-sniff outcome. Stale relative to the 10-variant `SniffOutcome` enum now in source. Refreshed during per-parser P{N}A phases.

**M3** — Same vestigial-v0.26.0 doc-comment pattern at `wallet_import/mod.rs:11-12` submodule-tree listing.

All three Minors are non-load-bearing doc-comment refreshes; tracked for natural cleanup during Wave-1 per-parser commits.

---

## Verification of cross-PR interactions

| # | Check | Status | Detail |
|---|---|---|---|
| 1 | P0D catch-all vs P0C stderr-template updates BOTH present at `cmd/import_wallet.rs:244-329` | ✓ PASS | P0C stderr templates at `:297-312` (8-format enumeration in `Ambiguous`/`NoMatch` arms); P0D `other => unreachable!` at `:325-329`. Disjoint structural sites within same `match`. |
| 2 | PossibleValuesParser 8-value list vs SniffOutcome 10 variants | ✓ PASS | 8 parser variants ↔ 8 PossibleValuesParser strings; Ambiguous + NoMatch are aggregate outcomes (not parsers). Alphabetical in both. |
| 3 | Skeleton helpers integration | ✓ PASS | 6 `canonicalize_<format>` skeletons at `roundtrip.rs:288-334`; alphabetical `use` block at `import_wallet.rs:64-68`; Site 6 dispatch `:536-548` calls matching names. |
| 4 | `match sniff_outcome` exhaustiveness over 10 variants | ✓ PASS | 4 explicit arms + P0D catch-all `other =>` = exhaustive. Catch-all is statically unreachable until per-parser P{N}A flips its sniff bool; `unreachable!` message cites SPEC §6.2 + P0D pre-stub for unmistakable cause attribution. |
| 5 | SPEC §6.1.1 cites `bitcoin_core.rs:81` | ✓ PASS | Verified at HEAD `de3dc61`: const at line 81, doc-comment range `:59-80`. |
| 6 | `design/v0_28_0-cycle-followups.md` "Open items: none yet" | ✓ PASS | Clean. No scope-creep from any Wave-0 sub-phase. |
| 7 | SniffOutcome enum order matches SPEC §6.2 verbatim | ✓ PASS | Source `:52-63` byte-identical to SPEC §6.2 `:127-138` order: `Ambiguous, BitcoinCore, Bsms, Coldcard, ColdcardMultisig, Electrum, Jade, NoMatch, Sparrow, Specter`. |
| 8 | Test coverage / duplication audit | ✓ PASS | Three layers (synthetic-dispatch unit / skeleton-shape unit / subprocess-panic integration) cover the surface from disjoint angles; no shadowing. |
| 9 | Wave-1 prerequisites | ✓ PASS | See table below. |

## Verification of Wave-1 prerequisites

| Prerequisite | Verified? | Status |
|---|---|---|
| `WalletFormatParser` trait exists + unchanged | YES | `wallet_import/mod.rs:38-47`; both required associated functions present |
| 6 new `SniffOutcome` variants present | YES | `sniff.rs:56-62`: Coldcard / ColdcardMultisig / Electrum / Jade / Sparrow / Specter, alphabetical |
| 6 new `ImportProvenance` variants — NOT yet added (per-parser scope) | YES (per plan-doc) | `mod.rs:63-71` still 2 variants; new variants land per-parser as per §B.2 #2 |
| `cmd/import_wallet.rs` dispatch sites pre-stubbed with `unimplemented!()` | YES | Site 2 supplied-format + Site 4 parser-invocation both have 6 phase-tagged `unimplemented!()` arms |
| `wallet_import/roundtrip.rs` skeleton `canonicalize_<format>` helpers exist | YES | 6 helpers; each returns `Err(ToolkitError::BadInput("…not yet implemented; <format> ingest lands in Phase P{N}B"))` |
| `wallet_import/bitcoin_core.rs` VENDOR_MARKER_KEYS at 13 entries | YES | 5 v0.26.0 originals + 8 v0.28.0 P0A additions |

## Net Wave-0 changeset audit

14 files changed; +1941/-80 LOC. All deltas accounted for; no anomalies. Per-PR detail in `design/agent-reports/v0_28_0-P0{A-r{0,1,2}, B2-r0, C-r0, D-r0}-review.md`.

## Cross-cutting integration risks (audit)

1. **P0B.1 supersession via P0D** — PR #35 closed; P0D landed alone and re-did the 4-existing-variant alphabetical reorder as part of its 10-variant final form. RISK: NONE.
2. **ImportProvenance reorder vs Wave-1 variant additions** — alphabetical anchor locked at P0B.2; per-parser inserts at known alphabetical positions. RISK: NONE.
3. **Catch-all `other =>` may shadow legitimate per-parser arms if Wave-1 forgets to remove** — `unreachable!` message cites SPEC §6.2 + P0D pre-stub explicitly, making the cause unmistakable. RISK: LOW.
4. **Wave-1 fan-out concurrent edits to `wallet_import/sniff.rs` at the 8-bool block** — Each P{N}A flips ONE alphabetically-positioned single-line edit; disjoint lines merge cleanly. RISK: NONE.
5. **Wave-1 fan-out concurrent edits to `cmd/import_wallet.rs` at Sites 2, 4, 6, 7** — Each P{N}C flips ONE format's arm at each site, alphabetically positioned. RISK: NONE.
6. **`wallet_import/mod.rs` submodule list** — alphabetical inserts of `coldcard, coldcard_multisig, electrum, jade, sparrow, specter` between existing modules. RISK: LOW (small file region, mechanical merge).

---

## Overall verdict

**GREEN.** Wave 0 integration is structurally sound. 9 cross-PR interaction checks PASS, all 6 Wave-1 prerequisites met, no scope-creep recorded, no conflict between P0C stderr-templates and P0D catch-all (disjoint structural sites), exhaustive match preserved over 10-variant `SniffOutcome` via `other => unreachable!`, alphabetical discipline enforced uniformly (CLAUDE.md ToolkitError rule extended to ImportProvenance + SniffOutcome + VENDOR_MARKER_KEYS + dispatch arms at all 8 sites).

**Recommendation: PROCEED to Wave 1 fan-out.** 10-way parallel per-parser sub-phase dispatch is unblocked. Lockless-update engineering at plan-doc §B.2 #5 + #6 verified in source.

---

**File-paths referenced (absolute):**
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/wallet_import/sniff.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/wallet_import/mod.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/wallet_import/bitcoin_core.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/wallet_import/roundtrip.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/import_wallet.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/tests/cli_import_wallet_p0c_dispatch.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/design/SPEC_wallet_import_v0_28_0.md`
- `/scratch/code/shibboleth/mnemonic-toolkit/design/v0_28_0-cycle-followups.md`
