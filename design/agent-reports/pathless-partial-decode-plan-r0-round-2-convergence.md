# R0 CONVERGENCE — IMPLEMENTATION_PLAN_pathless_partial_decode.md (round 2, fold verification) — Fable

**Persisted verbatim per CLAUDE.md.** Verified vs descriptor-mnemonic `a39c9d9f` (md-codec 0.41.0/md-cli 0.12.0 live) + mnemonic-toolkit `8e240d31` (Cargo.toml:34 pins md-codec 0.40.0). Every folded claim re-checked against source. VERDICT: **GREEN (0C / 0I / 2M record-only)**.

## Fold verification
**C-1 → P2.0/P2.1 split — CORRECT, coherent.** (a) Silent-rider removed: prerequisite cycle owns the 0.40→0.41.x boundary (F-A1 arm exists only in live 0.41.0 `canonical_origin.rs:64-71`, absent from pinned/vendored 0.40.0); P2.1's 0.41.x→0.42.0 carries only partial-decode delta. (b) Out-of-plan confirmed. (c) `synthesize.rs:359-368` do-not-fix note preserved + correct (no `Sh` arm). (d) Every `0.40` in the plan coherent; zero leftover direct-bump text; P2.0 scope matches `FOLLOWUPS.md:4845` (a)-(e) + `:4836` 6 comment sites (`error.rs:343-349`, `gui_schema.rs:1317-1320` re-verified exact); FOLLOWUP ownership unambiguous (both sh(wpkh) slugs → prerequisite cycle; `pathless-wallet-backup-partial-decode` → here). Version arithmetic coherent (md-codec 0.41→0.42, md-cli 0.12→0.13, toolkit MINOR after prerequisite bump).

**I-1 — CORRECT + buildable.** (a) Unconditional placement (before `validate.rs:222-224` early-return, or separate validator). (b) Distinct-variant room confirmed (`EmptyTlvEntry{tag}` at `error.rs:146`; Error insertion-ordered); forbidden `if !allow {validate()?}` shape named. (c) 3 RED tests present. Layer composition verified: origin gate fires ONLY in `decode_payload` (`decode.rs:75`); `reassemble` (`chunk.rs:381`) + `decode_md1_string` (`decode.rs:103/:105`) delegate via `?` → match-and-swallow-only-`MissingExplicitOrigin` at the single gate site composes cleanly; content-id check (`chunk.rs:384-391`) runs `encode_payload` (no origin gate) → invariant feasible on a partial descriptor; expand-side convergence in P0.3 (`canonicalize.rs:465` `sparse_lookup(...).is_none()` conjunct). Non-breaking re-confirmed (all vector `origin_path_overrides` null; 12 `MissingExplicitOrigin` pins). Downstream: empty-override wire → `Err` at P2.2 → mismatch, never partial.

**I-2 — CORRECT.** Chunk-form fixtures pinned (P2.2 note + P2.3 + RED-proof #6); `:207` no-change stated; FOLLOWUP filed. Fact re-confirmed: toolkit `inspect.rs:207` = `reassemble` only (dispatcher `decode_md1_string` unused there); all verify-bundle supplied intakes (`:388/:2450/:3045`) also `reassemble` → constraint consistent. Site map complete (`:2844/:3063` expected-bundle mint-fresh; `:3193/:3403` strict-and-bail).

**Minors:** M-1 genuine verdict-selection (`mismatch>partial>ok`, guards OK verdict only); tr-multi-a/tr-sortedmulti-a note correct (tr-taptree→None; wsh-sortedmulti→Some(48'/0'/0'/2')). M-2 `compute_wallet_policy_id(...)?` at `inspect.rs:20` exact; 3 pid outputs enumerated. M-3 parity=#6; auto-repair non-firing on both triggers (`inspect.rs:123-136`, `verify_bundle.rs:2451-2461/:3119-3127`). M-4 cites fixed. install.sh md-cli sibling pin frozen `md-cli-v0.11.2` while 0.12.0 current — re-confirmed.

## Findings
Critical: none. Important: none.
**Minor (record-only):**
- **M-R2-A** — `SPEC_pathless_partial_decode.md:73` still reads "pin-bump md-codec (0.40.0 → new)", superseded by the P2.0/P2.1 split. Acceptance (§Acceptance 1-9) has no 0.40 text → unaffected; add a one-line SPEC annotation so a future reader can't re-introduce the direct bump. **[FOLDED — opus.]**
- **M-R2-B** — trivial cite skews ≤2 lines (`template.rs:37-41` variant at :42; `inspect.rs:62-65` println at :61-64; `cli_inspect.rs:246` fn / :248 call). No action.

## VERDICT: GREEN (0C / 0I)
All round-1 folds correct, complete, no new drift. Sound to begin implementation: **P0 (md-codec) may start now, in parallel with the separate sh(wpkh) re-pin prerequisite cycle**; P2 gated on that cycle (enforced by the plan's P2.0 precondition).

---
**STATUS (opus, 2026-07-11):** Plan R0 loop CONVERGED GREEN. M-R2-A folded (SPEC annotation). Implementation CLEARED — Track A (partial-decode P0/P1, descriptor-mnemonic) + Track B (sh(wpkh) re-pin cycle, toolkit) run in parallel; P2 last.
