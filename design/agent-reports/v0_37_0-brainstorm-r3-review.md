# R3 ARCHITECT RE-REVIEW — `BRAINSTORM_v0_37_0_from_import_json_template_reemit.md`

**Round:** R3 (post-R2-fold convergence check)
**Date:** 2026-05-24
**Reviewer:** feature-dev:code-reviewer (opus), continuation of R0/R1/R2
**Spec SHA basis:** `36e6bfa`
**Verdict:** GREEN (0 Critical / 0 Important)

Files re-read: `cmd/export_wallet.rs` (both dispatch blocks, EmitInputs `:446`/`:661`, taproot wall `:629-639`, account guard `:554`, `CliExportFormat` `:22`, `conflicts_with_all` `:171`), `wallet_export/{mod,sparrow,coldcard,electrum,jade,bip388}.rs`, `template.rs`, `docs/manual/src/45-foreign-formats.md` (`:313-314`, `:347`, `:352-357`), `40-cli-reference/41-mnemonic.md:669`.

## Fold verification of the two R2 items
- **I-R2-1 (M-R1-a un-propagated to §6 Phase 2) — RESOLVED, exact.** Spec `:146` now reads "the `45:347` prose (leave the `45:352-357` taproot round-trip note unchanged — §2.4 keeps taproot walled, M-R1-a)". Whole-spec grep for `45:347|45:352-357|45:353` matches only `:110` (§3) and `:146` (§6), both with `45:352-357` exclusively in a "leave unchanged" clause. §3↔§6 contradiction gone.
- **M-R2-a (§2.6 dispatch-block labeling) — RESOLVED, exact.** Both `:493-511` (direct `run`) and `:713-735` (`run_from_import_json`) confirmed emit-dispatch `ColdcardMultisig` arms with `_ => Err` at `:510`/`:730`; collect_missing dispatches `:469`/`:687` carry no guard.

## Whole-spec convergence pass
All load-bearing citations re-grepped against source at `36e6bfa`; every one matches (parsed_ms `:613`, taproot `:629-639`, threshold `:659`, template:None `:666`, threshold_user_supplied `:671`, direct `:454`/`:435`, account `:554`, conflicts `:171`, CliExportFormat `:22`, CliTemplate `:15` 10-variants-no-bare-sh-multi, mod.rs script-type arms, bip388 `:33`, sparrow `:42/:43/:125/:137`, manual `45:313-314/:347/:352-357`, `41-mnemonic.md:669`).

**§1 vs §2.3 line divergence is intentional, not drift.** §1 cites the refusal *message-string* line (sparrow `:106`, coldcard `:113`, jade `:38`, electrum `:54`); §2.3 cites the `.ok_or_else` *predicate* line (sparrow `:104`, coldcard `:111`, jade `:36`, electrum `:52`). Both accurate locators into the same refusal block; off-by-2 is the message-vs-predicate distinction. Verified all eight.

Internal-consistency spot-checks all pass: §0 contract ↔ §5.1 matrix ↔ §5.3 byte-equality; §2.6 succeed/refuse split; §2.3 partition = `.ok_or_else`-refusers (green reads `script_type` only, bip388 excluded); §2.4 taproot wall precedes the `:661` EmitInputs build so `Tr(_)` arm is genuinely defensive.

## CRITICAL / IMPORTANT
None.

## MINOR
- **M-R3-a** (carried, non-blocking): §5.1 test anchors `:841`/`:611`/`:892`/`:722` point at the `fn` line rather than the `#[test]` attribute one line above. Valid locators; cosmetic.

## VERDICT
**VERDICT: GREEN (0C/0I)** — both R2 folds confirmed exact; convergence pass found no remaining Critical/Important inaccuracy, contradiction, citation drift, or fold-introduced error. The spec has converged and clears the mandatory pre-implementation R0 gate.
