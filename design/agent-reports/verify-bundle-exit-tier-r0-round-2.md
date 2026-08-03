# R0 round 2 — SPEC_verify_bundle_descriptor_exit_tier.md (convergence)

Reviewer: fable (independent). Date: 2026-08-03. Persisted verbatim, BEFORE the fold.

## VERDICT
NOT GREEN — C-1/I-1/I-2 are genuinely resolved and all six repros are runtime-confirmed, but the folds introduced two Important spec-text defects (§3's "six are reachable" contradicts the SPEC's own table, which has five; §5's newly-enumerated exit-4 meaning set omits the v0.88.0 `result: partial` class, which demonstrably fires on the descriptor-mode verify path), plus the round-1 review is not persisted to `design/agent-reports/`.

## C-1 / I-1 / I-2 RESOLUTION

**C-1 — genuinely fixed.** Independently re-verified at source. `parse_descriptor` (`parse_descriptor.rs:933-1012`): every fallible step — `expand_literal_double_star` (:946), `substitute_nums_sentinel` (:954), `detect_bare_tr` (:957), `lex_placeholders` (:963), `resolve_placeholders` (:964), `substitute_synthetic` (:975), `MsDescriptor::from_str` (:976), `walk_root` (:978, consumes the `key_map` from synthetic substitution, not caller keys) — reads only `input`. Caller `keys`/`fingerprints` touch only infallible collect/sort/assign at :980-1003. Additionally checked determinism: `substitute_synthetic` uses `BTreeMap` + regex `replace_all` (:479-519) — no HashMap-ordering hazard. `descriptor_str` is a non-`mut` binding after :1412; probe (:1467) and final parse (:1721) see the identical string. §3's restructure honestly reflects this, and the NON-NEGOTIABLE doc bar is present (:59).

**I-1 — genuinely fixed in structure, but the replacement enumeration has a new gap (see NEW DEFECTS).** `Bip388VerifyDistinctness` verified: variant `error.rs:24`, `exit_code` 4 at `:599`, fired pre-card at `verify_bundle.rs:1429-1431` and :1717-1718. §1 fact 2 carries the qualification; §7 distinguishes §4.11.c coherently; §8 lists it out of scope; T7 pins it.

**I-2 — genuinely fixed.** §6 now has one RED-first cell per reachable flipped site plus parity and no-collateral cells. Every repro runtime-verified against a freshly built v0.96.0 binary: all five site repros are RED today at exit 4 with the exact claimed messages, and the emit-surface twins exit 2. A half-fix can no longer ship green.

## NEW DEFECTS FROM THE FOLD

1. **§3 line 43: "All seven sites are input/usage-stage, and six are reachable" — the SPEC's own table has FIVE reachable sites** (`:1371`, `:1445`, `:1449`, `:1468`, `:1678`; `:1514` dead, `:1722` unreachable; `grep -c "| yes"` = 5), and §6 has exactly five site cells (T1-T5; T6 is parity). "Six" conflates "sites that flip to 2" (six, including dead `:1514`) with "reachable" (five). IMPORTANT — this count is the load-bearing inventory I-2's remedy hangs on.

2. **§5's migration note enumerates the post-change exit-4 meaning set as "the cards mismatched, or BIP-388 key-distinctness failed" — omitting `result: partial`**, the v0.88.0 pathless/dead-card partial-decode verdict, which fires on this exact path: `tests/cli_verify_bundle_partial.rs:312-338` runs `verify-bundle --descriptor` + elided md1 and asserts `Some(4)` + `result: partial`; producer at `verify_bundle.rs:1808` inside `verify_emit_from_expected`. The GUI gloss the SPEC cites lists partial as a distinct exit-4 meaning. Shipping the note as written would recreate the §1-fact-3 defect shape. IMPORTANT.

## CRITICAL
None.

## IMPORTANT
1. §3 "six are reachable" vs. the table's five — internal contradiction on the mandatory-test-cell count.
2. §5 exit-4 meaning set omits `result: partial`. Fix: add the partial class; no new test needed — `cli_verify_bundle_partial.rs` already pins it.
3. (Process) **Round-1 review not persisted** to `design/agent-reports/`. Project rule requires verbatim persistence BEFORE the fold.

## MINOR
- `error.rs:125-128` — `DescriptorReparseFailed`'s rustdoc still says "corrupted JSON, manual edit, upstream library version mismatch. Exit 4 (BundleMismatch tier)" — the disproven artifact-provenance framing. Add to the defensive-arm rewording scope.
- §3 table / §7 reference probe labels P2/P3/P4/P6/P7 and P1/P2 defined nowhere in the SPEC. Define or drop.
- §3: "borrowed at exactly two later points" — also borrowed at :1422, :1423, :1445. The load-bearing invariant holds; the phrasing overclaims.
- §9: `"re-parse failed"` grep also hits `cli_bundle_import_json.rs:909` (doc comment). A *different* surviving message `"--import-json: descriptor re-parse failed:"` exists at `bundle.rs:2109` (exit 2) — worth a word so an implementer doesn't fix the wrong one.
- §4's no-collateral enumeration misses `cli_verify_bundle_partial.rs` and `cli_verify_bundle_md1_template.rs:323/:627` — all genuine result-tier cases, untouched.
- Future-option (non-blocking): keys only populate infallible TLV fields, so the second parse at :1721 could one day be eliminated (parse once, attach keys), retiring the defensive arm structurally. FOLLOWUP at most.

## §6 TEST-SURFACE VERIFICATION
Built `target/release/mnemonic` from master @ `4c89891a` (0.96.0). Minted a real 2-of-2 bundle, harvested cards, ran each repro with the full card set. Sanity: correct descriptor → `result: ok`, exit 0.
- **T1 (:1445)** `wsh(multi(2,@0/<0h;1h>/*,@1/<0;1>/*))` → exit 4, "@0 multipath alternative `0h` is hardened…". Flips correctly; native lexer message survives wrapper deletion.
- **T2 (:1449)** `wsh(multi(2,@0/<0;1>/*,@2/<0;1>/*))` → exit 4, "@1 not present; placeholders must be dense 0..n".
- **T3 (:1468)** `wsh(pk(@0/<0;1>/*),pk(@1/<0;1>/*))` → exit 4, "unrecognized name 'wsh'". Short-circuit proves :1468 not :1722.
- **T4 (:1678)** `--slot @0.wif=<valid WIF>` → exit 4, ":1678 string verbatim".
- **T5 (:1371)** `--descriptor-file /nonexistent` → verify exit 4; bundle exit 2. Asymmetry live.
- **T6 (parity)** same T1 descriptor on `bundle` → exit 2, native message, no prefix.
- **T7/T8 inputs verified**; R-2 dead-site probe: n=3 with one slot → exit 2 from the shared gate; `:1514` confirmed dead.

## `:1722` DISPOSITION — INDEPENDENT JUDGMENT
Retention is right. Alternatives: (a) map to exit 2 — dishonest, since a failure there can only mean internal divergence and "fix your input" is the wrong advice; (b) panic — uncontrolled (exit 101) in a funds-verification tool, against project precedent; (c) typed defensive arm with honest internal-invariant wording — the SPEC's choice, matching `repair.rs:900-918`'s `PostCorrectionDecodeFailed`, ~4 lines. The vacuity failure-mode this project keeps hitting is contrived *tests* and *documentation* for dead arms manufacturing false liveness — T8 and the R-3 doc bar close both. `ExportWalletFormatStub` precedent for a retained construction-site-free variant is real (`error.rs:141-147`, `#[allow(dead_code)]`, zero construction sites). Keeping the variant avoids a needless enum API break.

## PROOF OF WORK
(Full section in session transcript. Key: `grep -c "| yes"` on the §3 table = 5; `find` for a persisted round-1 report = none; exactly 7 construction sites; zero exit-4 branching in scripts/.github/Makefile; zero `code(4)`/`Some(4)` in mnemonic-gui; full `code(4)|Some(4)` sweep identified `cli_verify_bundle_partial.rs` and `cli_verify_bundle_md1_template.rs` beyond §4's two; live probes for sanity/T1-T6/R-2.)
