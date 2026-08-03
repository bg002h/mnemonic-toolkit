# R0 round 3 — SPEC_verify_bundle_descriptor_exit_tier.md (GREEN)

Reviewer: fable (independent). Date: 2026-08-03. Persisted verbatim BEFORE the fold.

## VERDICT
GREEN (0 Critical / 0 Important) — every round-2 finding is genuinely resolved, the exit-4 meaning set is independently confirmed exhaustive, all seven site citations and every load-bearing claim check out at `4c89891a`, and the four residual items are Minor wording/citation precision that cannot change what an implementer does.

## ROUND-2 FINDINGS RESOLUTION
- **I-1 (six/five miscount) — fixed.** §3:43 now reads "SIX flip to exit 2; FIVE of those are REACHABLE (`:1514` flips but is dead)". Recounted independently: 6 "Becomes 2" rows, `grep -c "| yes"` = 5, `:1722` stays 4. §6 has exactly five reachable-site cells (T1-T5). Every other number consistent. The seven construction sites are exactly `verify_bundle.rs:1371, 1445, 1449, 1468, 1514, 1678, 1722`.
- **I-2 (`result: partial`) — fixed.** Both citations verified: `cli_verify_bundle_partial.rs:312-338` asserts `Some(4)` + `result: partial` on `verify-bundle --descriptor`; producer is the literal `"partial"` at `verify_bundle.rs:1808`, exiting via `Ok(if any_fail || partial { 4 } else { 0 })` at `:1837`. No contradiction with §3 or §7 — partial is precisely "a result with no self-oracle", which supports §7's principle.
- **I-3 (persistence) — fixed.** Both files exist; round-1 records the ordering violation at its top rather than hiding it.
- **MINORs** — `error.rs` rustdoc added to rewording scope; borrow phrasing corrected; §4 expanded; probe labels dropped. All fixed.

## NEW DRIFT FROM THIS FOLD
Two Minor, no Important. (1) §4:71's "(two cells on the `--descriptor` path)" — one asserts exit 4, the other asserts 0. (2) §9:127's "only `cli_cycleA_phase2_funds_proof.rs:228`" left unqualified; the grep also hits an inert doc comment at `cli_bundle_import_json.rs:909`. Neither can cause a wrong edit.

## EXIT-4 MEANING SET — EXHAUSTIVENESS
**{card mismatch, `result: partial`, `Bip388VerifyDistinctness`} is exhaustive.** Independent sweep: all `=> 4` arms in `error.rs` are `Bip388VerifyDistinctness`(:599), `BundleMismatch`(:607), `DescriptorReparseFailed`(:617), `ImportWalletSeedMismatch`(:630), `RestoreMismatch`(:645), `XpubSearchNoMatch`(:652), `XpubSearchPassphraseCandidatesExhausted`(:653), plus dynamic `RepairShortCircuit{exit_code}`(:644) constructed only with 5 (`repair.rs:1878`, sole site). Reachability from `descriptor_mode_verify_run`: `BundleMismatch` only inside `self_check_bundle`/`check_mk1_xpub_binding`, called exclusively from emit paths; the others live in other subcommands. Codec `From` impls route to exit 1/2/3 only. Non-error exit 4 is only `verify_emit_from_expected:1837` (`any_fail` or `partial`). Post-change the sole remaining exit-4 producer is the defensive `:1722`, structurally unreachable and barred from user-facing docs. Exit 5 (auto-repair short-circuit) can fire here but is not an exit-4 meaning and is correctly out of scope. **No fourth omission exists.**

## IMPLEMENTABILITY
Yes. All seven table line numbers exact at `4c89891a`; the mechanism (delete three `.map_err` re-wraps at `:1445/:1449/:1468`; construct `DescriptorParse` at `:1371/:1514/:1678`) is checkable line-by-line. `bundle.rs:318-321` already maps the same fs error to `DescriptorParse` with the byte-identical detail format `"--descriptor-file {}: {e}"`, so T5's both-surface parity falls out exactly. Six live probes re-run against a version-checked v0.96.0 binary: T3-shape → 4 ("unrecognized name 'wsh'"), T2-shape → 4 ("@1 not present"), bundle twin → 2, `--descriptor-file /nonexistent` → verify 4 / bundle 2, R-2 dead-site → 2 from the shared gate.

## CRITICAL
None.

## IMPORTANT
None.

## MINOR
1. §4 partial-file cell miscount — one descriptor-path cell asserts 4, the other asserts 0.
2. §9 "only `:228`" — add the inert doc-comment hit at `cli_bundle_import_json.rs:909`.
3. Unpathed technical-manual citations — add `docs/technical-manual/src/60-back-matter/` and `…/50-rust-api/` prefixes; the end-user manual tree has a *different* `67-troubleshooting.md`, so bare names invite a wrong-tree search.
4. Two off-by-one citations: `error.rs:125-128` for a rustdoc at `:124-126`; `:1423` for an identifier on `:1424`.

All four foldable in the implementing commit; none gates.

## PROOF OF WORK
(Full section in session transcript. 73 tool uses. Key: all 7 sites re-grepped exact; complete `=> 4` sweep of error.rs plus codec From impls and exit helpers; all cited test cells read individually; all doc citations verified incl. a full `docs/` sweep for `DescriptorReparseFailed`; GUI grep for `code(4)`/`Some(4)` = zero outside `render_exit_badge`; scripts/CI grep = zero; 6 live probes. Could not re-run T1/T4 — need fingerprint/xpub and WIF fixtures; round 2's persisted runtime verification covers them and their mechanism was verified at source.)
