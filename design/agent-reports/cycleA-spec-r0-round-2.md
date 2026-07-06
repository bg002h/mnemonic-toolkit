# R0 GATE REVIEW — SPEC_cycleA_descriptor_use_site_collapse.md — Round 2 (convergence)

**Reviewer:** opus architect. **Against:** `origin/master @ 8c8b9183`; sibling `descriptor-mnemonic`. Read-only, adversarial funds-safety posture. **Converging against:** `cycleA-spec-r0-round-1.md` (0C/2I). Persisted verbatim per CLAUDE.md.

---

## Independent verification (what re-checks out in rev-2)

**Both round-1 IMPORTANTs are resolved.**

**I-2 resolved by clean split.** Grepped the full spec for every Part-2 residue (`select-descriptor`, `1859`, `2265`, `checksum recompute`, `paired`, `GUI`, `wire-shape`, `merge`). Every hit is correctly framed as *out-of-cycle*: header §rationale, §7 "Moved to the pair-merge follow-up", §9 "No `--json` wire-shape change / No paired mnemonic-gui PR this cycle", §11 follow-up scope, §12 OQ1. **No orphaned in-scope reference survives** — no test expecting a merged `<0;1>` card, no `--select-descriptor`/`internal`-field edit listed as in-scope, no trap #1/#2/#3/#7 assigned to Cycle A (§7 correctly moves exactly those four out; all 10 traps accounted for, 6 in / 4 deferred). Interim limitation documented in **all three** required sites (manual §9, CHANGELOG §10, per-surface message §6). Follow-up `bitcoin-core-receive-change-pair-merge` filed with the full deferred scope including `internal: bool→Option<bool>`, both wire sites `:1859`+`:2265`, checksum recompute, GUI paired-PR, and the `:915` different-keys merge-negative-control (§11).

**Independent sanity-check of the split's core claim — both directions confirmed against source + fixtures:**
- `cli_import_wallet_bitcoin_core.rs:898` (`core_fixture_file_mainnet_receive_change_pair_parses`) — fixture `core-mainnet-receive-change-pair.json` carries `wpkh([b8688df1/84'/0'/0']xpub…/0/*)` (internal:false) + the SAME xpub `…/1/*` (internal:true). Traced both through the rev-2 regex (`parse_descriptor.rs:97-98`): `@0[fp]` matches, mpath needs `/<` (sees `/0`→no), wild needs `/*` (sees `/0`→no) → `match_end` after the bracket, residue `/0/*)` / `/1/*)`, next char `/` ∉ terminators → **both REJECT.** The test's `.success()`+`bundles=2` correctly flips to a reject. ✓ (spec §8 names this exactly).
- `:915` (`core_fixture_file_multipath_receive_change_pair_parses`) — fixture carries `wpkh([b8688df1/84'/0'/0']xpub…/<0;1>/*)` (FP_A, bip84) + `sh(wpkh([28645006/49'/0'/0']xpub…/<0;1>/*))` (FP_B, bip49) — **genuinely different keys.** Traced: mpath consumes `/<0;1>`, wild consumes `/*`, residue `)` → terminator → **both PASS Part 1 untouched.** `bundles=2 .success()` stays. ✓ (spec §8 pins it as the future merge-negative-control).

**I-1 fold complete** (§8). All five sub-parts present: (a) list marked NON-EXHAUSTIVE; (b) `grep -rn '/0/\*\|/1/\*' crates/mnemonic-toolkit/{src,tests}` + classify-every-hit mandated for the plan; (c) NO-WEAKENING rule explicit ("NEVER silently rewrite `/0/*`→`/<0;1>/*`… that would delete the regression this cycle proves"); (d) `:898` flip named (correctly re-targeted to *reject*, not the merge-era `bundles=1`); (e) `:915` stay-passing named. The migration cell list (a4/a5 convergence, sniff, coldcard BSMS blob, export refused, etc.) is materially broader than rev-1.

**All MINORs folded:**
- **M-1** — D1 follow-up carries explicit funds framing + CHANGELOG residual disclosure (§4, §10, §11). ✓
- **M-2** — sparrow/coldcard/electrum/coldcard_multisig sweep restated with citations; sparrow descriptor-passthrough caveat present (§2). Verified: `sparrow.rs` synthesizes `…/<0;1>/*` (L380-393) but has a real `has_tr && !has_at_placeholder` descriptor-passthrough branch (L58, L321-343) → confirm-at-impl is the right disposition. ✓
- **M-3** — verify-path reject is `DescriptorReparseFailed{detail}`; confirmed against `verify_bundle.rs:1375` (`lex_placeholders(…).map_err(|e| ToolkitError::DescriptorReparseFailed { detail: e.message() })`); trap-#9 test asserts that shape (§4, §8). ✓
- **M-4** — `#` reword to "never *directly* follows a placeholder" present and accurate (§2), correctly superseding rev-1's over-broad claim. ✓
- **M-5** — citation nit fixed: "validator body ~`:77-110`; placement comment `:121-127`" matches md-cli `template.rs`. ✓
- **M-6** — bare-unbracketed-origin `@0/48h/0h/0h/<0;1>/*` negative test present (§5, §8); traced → residue `/48h…`, rejects. ✓
- **M-7** — verify false-pass mechanism reworked: encode can no longer *build* a `/0/*` bundle, so the test (a) verifies a `/0/*` descriptor vs any card asserting reparse-reject, and/or (b) loads a pre-generated wrong-card fixture (§8). ✓

**Residual correctness (unchanged Part 1) holds.** The §5 residue block is byte-logically identical to the R0-round-1-verified md-cli mirror (`template.rs:128-137`): same terminator set `) , }` + whitespace + EOS, `if let Some(next)` gives EOS-passes-free. Placement (after validator `:146-178`, before push `:183`) preserves the H13 typed-error-first ordering. Both residue directions (pre-mp `@0/0/<0;1>/*`, post-mp `@0/<0;1>/0/*`) re-traced → reject. No rev-2 edit weakened the floor.

## CRITICAL
**None.**

## IMPORTANT
**None.** Both round-1 IMPORTANTs are resolved; the split introduced no in-scope orphan and no cross-section contradiction. §3 / §7 / §9 all consistently reflect Part-1+Part-3-only.

## MINOR
- **M-8 (new, fold-introduced cosmetic drift).** §8 labels the migration paragraph "Migration — §9 is NON-EXHAUSTIVE", but the migration cell list lives in §8; rev-2's §9 is "Lockstep ripples." Stale cross-reference carried from the round-1 fold instruction. Content unambiguous + correctly placed — purely a wrong section number. Fold: drop the `§9`.
- **M-9 (carry-forward, non-blocking).** Three confirmations deferred to plan/impl MUST be discharged there: (i) OQ1's grep-sweep proof that no surviving Cycle-A test implies a merge round-trip; (ii) the sparrow descriptor-passthrough branch can never forward a fixed use-site step; (iii) BSMS `wallet_import/bsms.rs` `/**` residue handling + `wallet_export/bsms.rs:159-161` self-round-trip reject. Plan-phase; flagged so the plan-doc R0 verifies they land.

## VERDICT
**GREEN (0C/0I).** Both round-1 IMPORTANTs resolved: I-2 by a clean Part-2 split with no orphaned in-scope reference and the interim Core hard-fail documented across manual + CHANGELOG + follow-up (with the `:915` different-keys negative-control preserved); I-1 by the non-exhaustive grep-mandate + NO-WEAKENING rule + the `:898`→reject / `:915`→pass classification, both independently re-traced against source and fixtures. The Part 1 residue floor is unchanged, fail-closed, and correct. The two MINORs do not block. Cleared to proceed to the implementation plan-doc R0 loop.
