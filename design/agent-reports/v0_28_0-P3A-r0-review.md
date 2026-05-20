# Phase P3A — architect R0 review

**Reviewer:** in-session architect-style self-review (Opus 4.7 main agent)
**Branch:** `v0.28.0/p3-coldcard`
**Files under review:**
- `crates/mnemonic-toolkit/src/wallet_import/coldcard.rs` (new, ~340 LOC including tests)
- `crates/mnemonic-toolkit/src/wallet_import/mod.rs` (+6 LOC: module declaration + forward-ref comment)
- `crates/mnemonic-toolkit/src/wallet_import/sniff.rs` (+1 LOC: ColdcardParser::sniff wire-up)
- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` (+10 LOC: auto-sniff-arm safety route)

**Source SHA verified against:** branch HEAD pre-commit
**Plan-doc:** `/home/bcg/.claude/plans/unified-meandering-sundae.md` Phase 3 P3A row + §S.3
**SPEC:** `design/SPEC_wallet_import_v0_28_0.md` §11.3 + §11.3.1

---

## Critical (correctness-blocking; would break P3B execution)

**None.** P3A delivers the skeleton-only contract:
- `ColdcardParser` struct + `WalletFormatParser` trait impl (`sniff` real, `parse` `unimplemented!`).
- `ColdcardSourceMetadata` + `ColdcardChain` + `ColdcardBip` type definitions per SPEC §11.3.
- 22 sniff unit tests covering SPEC §11.3 clauses + firmware-variance table positives + competing-vendor negatives.
- Sniff wiring at `sniff.rs:77` flips placeholder `false` → real call.
- Auto-sniff safety arm in `import_wallet.rs` routes `SniffOutcome::Coldcard` to existing P3C `unimplemented!` panic site (preserves clean failure mode).

No execution-blocker for P3B.

## Important (would block P3A merge)

**None.** The original prompt scheduled `ImportProvenance::Coldcard` enum addition for P3C, but P3A's sniff wiring makes `SniffOutcome::Coldcard` reachable at runtime — leaving the `unreachable!` catch-all in `import_wallet.rs` would yield a panic with a confusing message on any auto-sniffed Coldcard blob. The fold: add a one-line `SniffOutcome::Coldcard => "coldcard"` arm at the auto-sniff dispatch site, routing through the existing P3C `unimplemented!` arm (matches the explicit-format path's failure mode). This is documented in the file with a comment naming P3A as the introducer + P3C as the parse-impl wire-up site.

## Minor (fold inline or defer)

**M1 — `#[allow(dead_code)]` density at P3A is high (4 sites).** Three on the type-level skeleton (`ColdcardParser` struct, `ColdcardChain` enum, `ColdcardBip` enum, `ColdcardSourceMetadata` struct fields), justified because the constructor + field reads land at P3B. Acceptable per the v0.28.0 pre-stub pattern (P0A/P0C/P0D used the same discipline). No fold.

**M2 — `COLDCARD_PER_BIP_MARKERS` const string-array ordering.** Listed alphabetically (`bip44, bip48_1, bip48_2, bip49, bip84, bip86, xpub`) per the SPEC §11.3.1 reading-order convention. Not load-bearing for the sniff (the predicate is a logical OR), only for human-readable diff stability. Aligns with CLAUDE.md alphabetical-discipline guidance carried forward from `error.rs::ToolkitError`. No fold.

**M3 — Sniff cell coverage.** 22 cells span: 6 positive (one per firmware era from SPEC §11.3.1 table + bip48-only + legacy top-level-xpub-only), 7 negative-malformed (missing/wrong-shape fields per clause), 4 format-disambiguation (Bitcoin Core / Specter / Electrum / Jade negatives), 3 robustness (empty / invalid JSON / leading whitespace), 1 skeleton-panic guard, 1 provenance-struct-construction lock. Coverage matches plan-doc P3A `~120 tests` line-budget intent (LOC; cell-count is 22). No fold.

**M4 — `parse_skeleton_panics_at_p3a` test uses `#[should_panic(expected = "Phase P3B")]`.** This serves as the regression-guard against accidental impl-body-merge before P3B's review converges. Mirrors the v0.26.0/v0.27.0 pre-stub discipline. P3B's first commit will DELETE this test cell. No fold.

**M5 — `coldcard_metadata` accessor on `ImportProvenance` deliberately NOT added at P3A.** The accessor lands when the variant lands (P3C). At P3A, the type `ColdcardSourceMetadata` is publicly accessible to crate code via `coldcard::ColdcardSourceMetadata` but not via any provenance variant. This matches the v0.26.0 pattern where `CoreSourceMetadata` was defined as a struct before `ImportProvenance::BitcoinCore` was added. No fold.

## Verdict

**GREEN.** Proceeding to commit + P3B without further iteration. P3B's first R0 will verify the parse impl + canonicalize + ~5 fixtures.
