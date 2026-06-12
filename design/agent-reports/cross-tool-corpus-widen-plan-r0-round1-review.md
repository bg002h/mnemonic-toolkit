# R0 Review — cross-tool corpus widen (PLAN) — Round 1
Reviewer: Fable 5, 2026-06-12. Verified against mnemonic-toolkit origin/master 1f0eb74. The workspace `md` is source-identical to the CI-pinned `descriptor-mnemonic-md-cli-v0.6.2` tag (diff over md-codec/src + md-cli/src is empty; post-tag commits are fuzz/test-only), so probes are valid against the CI baseline.

## Verdict: GREEN (0C/0I)

## Critical
None.
## Important
None.
## Minor
- **M1 — §7 "R0 questions" stale (says "9 rows", asks tr-NUMS/hashlock questions §2 already settled).** Plan-doc hygiene; §2 table + documented drops are the unambiguous contract. [FOLDED: §7 rewritten to record answers.]
- **M2 — two cheap probe-verified omissions: `and_b`+`a:` and `t:or_c`.** Both run through both built binaries and MATCH:
  `wsh(and_b(pk(@0/<0;1>/*),a:pk(@1/<0;1>/*)))` → 3b7827a56ada29a201a2d77c00c99748 / aa203f1ea88576a5b918dbacbc4a8ad5
  `wsh(t:or_c(pk(@0/<0;1>/*),v:pk(@1/<0;1>/*)))` → 552313aac0e243d3fd4175d8b084d1c3 / 5671479bfe0c9e0346d426494a1ea544
  (shared `[73c5da0a/48'/0'/0'/2']`, `--path m/48'/0'/0'/2'`). Free given the probe evidence — fold (9 rows). [FOLDED.]
- **M3 — pin the full 64-hex sha256 literal** `0000000000000000000000000000000000000000000000000000000000000001` (probe-matched). A wrong-length literal would land ToolError → caught loudly at impl. [FOLDED.]
- **M4 (optional) — pin the corpus count** (`assert_eq!(entries.len(), N)`) so an accidentally deleted row is loud. [FOLDED.]

## Notes
- Independently re-ran 4 of the 7 rows (andor-hashlock, sh-wsh-sortedmulti, thresh-2of2, and_v-older) through built `mnemonic` + `md` 0.6.2 — all MATCH.
- (1) Row set + drops right. n≥3 drop verified-real (md-cli depth==path-depth gate + only two depth-4 consts; matches the harness's own [m5] comment). tr-NUMS drop correct — shipping without resolving internal-key-spelling parity manufactures a spurious Diverge; belongs with GAP-1. sortedmulti sole-child-only correct (GAP-3 owns the combinator form).
- (2) Spurious-match limit accepted (inherent to any differential; harness has proven sensitivity via the Check-PkK catch). Shared `md inspect` does NOT collapse signal — each tool encodes independently; inspect only reads ids. Anti-vacuity: a broken row returns None → ToolError → fails TWICE (failures push at :396-400 + `n_both_error==0 && n_tool_error==0` at :429-434). No false pass possible.
- (3) Entry construction internally consistent for all rows (bracket path == md --path, the [I2] invariant). One-key rows use @0 only (the loop handles it).
- (4) NO-BUMP confirmed: one test file, no clap surface, cross-tool-differential.yml triggers on the test path (:23,:28) and pins md-cli-v0.6.2 (:46).
- (5) Found-divergence contingency sound (matches v0.55.0 precedent: characterize → temporary documented Diverge → fix → flip to Match; never re-label to green the suite).
- Pre-existing drive-by: cross-tool-differential.yml header comment (:9-12) still says "pinning the known wsh(pk)-shape divergence as Diverge" — resolved at v0.55.0; optional comment refresh.

**Gate: GREEN — implementation may begin (fold M1-M4).**
