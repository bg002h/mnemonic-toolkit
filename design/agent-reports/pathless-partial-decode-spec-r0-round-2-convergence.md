# Convergence R0 — SPEC_pathless_partial_decode.md (round 2, fold verification) — Fable

**Persisted verbatim per CLAUDE.md.** Verified vs descriptor-mnemonic `a39c9d9f` + mnemonic-toolkit `0a939443`; every folded claim re-checked against real code. VERDICT: **GREEN (0C / 0I)** — 1 trivial Minor. Sound to proceed to the implementation-plan phase.

## Fold-by-fold
**C-1 (three-layer plumbing + oracle-intact invariant) — CORRECT.** (a) Layer completeness re-traced: partial consumers reach decode ONLY via the three layers (md decode `decode.rs:11/:14`, md inspect `inspect.rs:13/:16` via `decode_md1_string`/`reassemble`; `decode_md1_string` self-dispatches chunk-forms into `reassemble(&[s])` `decode.rs:97-106`; toolkit inspect `cmd/inspect.rs:207`; verify-bundle `:2450`/`:3045`). No fourth layer. (b) Ordering byte-verified: `decode_payload` at `chunk.rs:381`, origin gate at `decode.rs:75`, content-id check at `chunk.rs:383-391` runs strictly AFTER → keeping :383-391 enforced under partial is necessary+sufficient to preserve the aliasing oracle. `encode_payload` (`encode.rs:100-110`) runs no origin gate → content-id check works on a partial descriptor. Header `:348-361` / index-gap `:363-372` cites exact. (c) "reassemble keeps the strict gate" contradiction gone (grep clean). (d) Both RED tests present.

**I-1 (verify-bundle site map) — CORRECT.** `:2450` = `reassemble(supplied)`, `:2846` = `match supplied_md_decoded.as_ref()` (consumption, NOT a decode), `:3045` = descriptor-flow `match reassemble(...)`. Compares exclude origin (`:2903-2923` tree/use_site/overrides + sorted pubkey multiset; `:3077` xpub bytes) → explicit gate cannot be delegated. `:388` re-verified as a separate reassemble routing into `verify_singlesig/multisig_template`, returns before the descriptor/policy gates; dead card fails `if let Ok(d)` → falls through to `:435-442` `--template is required` ModeViolation (fail-closed, exit≠0, never a pass, never partial). `restore.rs:316` strict twin. `:3193`/`:3403` bail-silently. Acceptance #2 rescoped with the `:388` strict carve-out; dead-template test present.

**I-2 → repair STAYS STRICT — CORRECT AND SAFE (adversarially checked).** md-cli `repair.rs:118/:124`: `decode_with_correction` error → `Ok(2)`; an intact dead card is BCH-clean but decode fails `MissingExplicitOrigin` → Err → exit 2, byte-identical to today. Toolkit `repair_via_md_codec` (`:1641`): same strict oracle; the v0.86.0 non-chunked demote (`:1660-1676`) lives ENTIRELY in the `Ok` arm — a dead card never reaches `Ok` under strict decode, so no Blessed-partial + no demote composition can occur. Round-1 enlargement concern GENUINELY DISSOLVED, not deferred with a hole (BCH candidates validated by strict full decode → a candidate resolving to a dead card stays pruned). De-scoped `repair-corrupted-pathless-card-partial` is a clean follow-up (no current code half-implements it). decode-exit-4 vs repair-exit-2 asymmetry pinned + coherent (repair's contract is "make the card decode"; it can't → un-repairable/exit-2 is honest).

**I-3 (sibling pin) — CORRECT.** SELF-pin only; md/ms/mk sibling pins UNCONDITIONALLY FROZEN incl. md-cli despite the md-cli publish, citing v0.75.0 post-tag revert. Matches the incident record + frozen-pin doctrine.

**Minors M1-M7 — ALL LANDED.** M-1 `result` `"ok"`/`"mismatch"` + `schema_version:"4"` at all 4 emit sites, mismatch>partial precedence stated. M-2 `Vec<u8>` (`Descriptor.n:u8`, `MissingExplicitOrigin{idx:u8}` error.rs:321-323). M-3 hand-built `serde_json::Map` wording. M-4 strict-consumer map complete + live (`verify.rs:18-23` decode before `:37-45` byte-compare; `bytecode.rs:10-13`; `word_card_adapter.rs:103`; `descriptor_intake.rs:224`; convert zero md1). M-5 BIP folded into leg 2 (placeholder `bip-mnemonic-descriptor.mediawiki:~218-227`). M-6 GUI informational FOLLOWUP. M-7 `identity.rs` comment P0 (WDT-id `:71-96` no origin dep).

**Auto-repair-non-firing — coherent.** Toolkit inspect trigger (`inspect.rs:123-136`) matches `MdCodec(_)`; once inspect opts into partial, a dead card returns `Ok` → trigger unreachable (clean fall-through). verify-bundle `:2451`/`:3120` likewise. No bad interaction with the v0.86.0 demote (Ok-arm-only).

## Findings
Critical: none. Important: none.
**Minor (1) — M-R2-1:** SPEC test line named `convert` as an auto-repair-non-firing surface, but `convert` has zero md1 intake (`convert.rs:1027-1035` matches only `MsCodec|MkCodec`). The "intact dead card in convert" test is unwritable → replace with `verify-bundle` (or drop). No funds/design impact. **[FOLDED — opus: line now reads `verify-bundle`/`inspect`.]**

## VERDICT: GREEN (0C / 0I)
All four majors folded correctly with no drift; every fold-introduced citation re-verified live. Sound to proceed to the implementation-plan phase.

---
**STATUS (opus, 2026-07-11):** SPEC R0 loop CONVERGED GREEN. M-R2-1 folded (one word). Next: user spec-review checkpoint → implementation-plan phase (own R0 gate) → single-implementer per-repo legs TDD.
