# R0 Architect Review — technical-manual-symbol-pin-lint — Round 4

> Persisted verbatim from the opus `feature-dev:code-architect` agent
> (`agentId: a5957b9907db86e56`). Verified against source at toolkit `3d9d38e`.

## VERDICT: 0 Critical / 0 Important / 3 Minor — **GREEN**

**GREEN — cleared for implementation.** Both Round-3 findings (C1, I1) are closed with live evidence. The forward gate design (§2), nesting convention (§1), audit-all workflow (§3), and phasing (§6) are sound enough to begin P1. The three Minors are precision/reconciliation items the implementer should fold in P1 but none ships a wrong/vacuous anchor or breaks the gate.

### Critical
None.

### Important
None.

### Minor

**M1 — Recon generator's collision flag uses bare basename (`cycle-b-manifest3.py:88` `bn = os.path.basename(pp)`), diverging from the SPEC §2 gate predicate (path-suffix-across-repos).** Over-flags 4 rows §1 declares non-colliding: `cycle-b-audit-queue.tsv:801,813,823,824` flag `wallet_export/mod.rs` because basename `mod.rs` exists in multiple repos — yet §1 says "`wallet_export/mod.rs` is toolkit-unique → no collision" (Glob confirms the `/wallet_export/mod.rs` suffix is toolkit-only). Non-blocking: the generator is RECON, §1/§2 agree with each other, and both qualified/bare forms pass G2 (no wrong anchor ships). **Fix in P1:** implement the gate's collision predicate on **path-suffix** (per §2, NOT basename); optionally re-run the generator with the suffix predicate so recon TSV and gate agree.

**M2 — §1's collision characterization under-describes the live queue.** §1 says the genuine set is "all `derive.rs` in 31/33 + an ambiguous glossary `error.rs`" (=5); live count is **9** (`derive.rs`×4, `error.rs`×1, `wallet_export/mod.rs`×4 — the M1 over-flags). "~5-9" contains 9 so not false → Minor. Reconcile the prose count once M1's suffix predicate collapses the set to the intended 5.

**M3 — Two `mod.rs` cites in authoritative ch52 are `unresolved`, not false-resolved** (`cycle-b-audit-queue.tsv:378,395` — `mod.rs:27-31`, `mod.rs:29-37`, bare `mod.rs`, no parent dir). Correctly routed to audit (class `unresolved`); no wrong anchor ships. The P2 audit must supply the parent dir (e.g. `string_layer/mod.rs`) so G2 resolves — treat as path-incomplete, not merely line-stale.

### Verified clean

**R3 C1 — CLOSED (single unified 826-member audit input; autopin retired; fastpath ≠ skip).** `cycle-b-audit-queue.tsv` = 827 lines (826 data + header). `cycle-b-autopin.tsv` gone; generator removes it (`cycle-b-manifest3.py:120-121`). All members written to the one queue (`:115-119`); none `continue`d out. `fastpath` is a column value, never a skip; SPEC §2/§3/§3a/§5/§6 consistently name the single queue.

**R3 I1 — CLOSED (collision rule narrowed; authoritative chapters not forced to qualify).** Live collision-class rows = **9** (within "~5-9"). Predicate live: `cycle-b-manifest3.py:89` `collide = (bn in COLLIDING) and not explicit and (auth is None)`; `authoritative_repo()` returns a repo for 21/22/23/41/42/51/52/53/54, else None. **Authoritative-chapter false-negative = 0 in corpus:** spot-checked every bare colliding basename in 21/22/23/41/42/51/52/53/54 — each resolves to the chapter's OWN repo (ch51 `tlv.rs`→md, ch51/53 `error.rs`→md/ms, ch52/53 `consts.rs`→mk/ms). No authoritative cite resolves to a different repo. The §3a per-prose audit is the correctness backstop; G2 is only a hallucination/rename backstop. Narrowed rule misses no real wrong-repo case.

**Cumulative R1+R2 folds hold (spot-checked, live):**
- Six→seven sites all live-accurate at 3d9d38e: `AUTHORING.md:181` "Six checks" + `:185` "CI runs all six" (genuinely false — no technical-manual.yml); `Makefile:13`; `lint.sh:10-16`.
- R2 M1 synthesize boundaries — FOLDED with improved precision: `synthesize_descriptor` 228-322 (`derive_xpub_at_path` @323), `synthesize_multisig_full` @343, `synthesize_multisig_watch_only` @488, `synthesize_unified` @744; line 593 inside multisig_watch_only, intent `synthesize_unified`. Tightened, not drifted.
- G2 multi-segment / nesting convention — §1 4-row table + §2 per-segment `grep -wqF`; method anchors carry `T::method` (queue rows e.g. `tlv.rs:43`→`TlvSection::new_empty`, `error.rs:129`→`Error::fmt`).
- Comma-tails (R2 I1) routed to audit as `tail:*`; figures-cache (R2 I2) empirical in §3; no-auto-pin / SemVer no-bump-no-tag / no-lockstep re-confirmed (docs + docs-lint helper, byte-identical binary).

**Bottom line:** R3 C1 and R3 I1 closed with live evidence (826 unified rows; autopin removed; 9 narrowed collisions; authoritative chapters resolve to own repo; 0 false-negatives in corpus). No wrong/vacuous anchor ships under the gate design; the per-prose audit covers the residual; §2 path-suffix predicate is internally consistent with §1. The 3 Minors fold during P1. **GREEN — cleared for implementation. P1 may begin.**
