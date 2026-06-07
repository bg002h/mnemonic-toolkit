# R0 Architect Review — technical-manual CI workflow + api-harvest transcript cleanup — Round 2 (fold-confirmation)

> SPEC under review: `/scratch/code/shibboleth/mnemonic-toolkit/design/SPEC_technical_manual_ci_and_transcript_cleanup.md`
> Round 1 = GREEN (0C/0I) + 3 Minors. Folds M1 (broaden gap wording + rename id) and M2 (reword `*_BIN` comment) were applied. This round confirms the folds are clean. Reviewer had Read/Glob/Grep only; parent persists.

## Verdict: NOT GREEN — 0 Critical / 1 Important / 1 Minor

The M1 *id-rename* landed cleanly and consistently (old narrow id has zero occurrences; new id appears in exactly the two required sites). But the M1 *broadening* reached ship-plan step 5 and §2 only — it did **not** reach the YAML comment block, which still carries the pre-M1 narrow "absent sibling only" framing. The SPEC now contradicts itself on the precise point M1 was filed to fix. This is fold-introduced drift, which is exactly what the post-fold loop exists to catch.

## Critical
None.

## Important

**I1 — M1 broadening did not reach the YAML comment; step 5 ↔ YAML comment now contradict each other on the M1 point.**
The YAML comment (lines 181-182) reads: "symbol-ref-check enforces G1 ... + G2 (symbol existence) for **TOOLKIT refs**; G2 for refs whose file **lives only in an absent sibling** skips gracefully." This is the *pre-M1 narrow* framing. But:
- §2 line 110-111: "The same skip extends to a *non-authoritative-chapter* ref to **any file unresolvable in the present (toolkit) repo**."
- Step 5 lines 296-300 (the broadened text): the skip set explicitly includes "(ii) a *renamed toolkit file* cited by bare basename from a non-authoritative chapter."

A renamed-toolkit-file ref is (a) a toolkit-intended ref — so the comment's affirmative "G2 for TOOLKIT refs [enforced]" overclaims — and (b) its file does **not** "live only in an absent sibling," so the comment's skip clause fails to describe it, even though the Item-2a logic (`not auth and not qualified and ABSENT`, lines 140-142) skips it. The comment thus both overclaims enforcement and under-describes the skip set, in the same block whose FOLLOWUP id M1 renamed (line 183). M1's stated scope was to broaden "ship-plan step 5 **AND the YAML comment block**" — the id was updated in both, but the framing was broadened in only one.

Fix (one-line reword, lines 181-182): inherit §2's framing, e.g. "...+ G2 (symbol existence) for refs resolvable in the present (toolkit) repo; G2 for any ref whose file is unresolvable here — codec files in absent siblings, AND renamed toolkit files cited by bare basename from non-authoritative chapters — skips gracefully in bare CI (enforced by local `make lint`; see FOLLOWUP technical-manual-g2-uncovered-in-bare-ci)."

## Minor

**M1' — §2 line 99 under-specifies api-surface-coverage's source vs the M2 reword (comment is the correct side).**
The M2 reword (line 237) says api-surface-coverage "reads lib.rs/format.rs source directly." §2 line 99 says only "reads lib.rs." Verified against `docs/technical-manual/tests/api-surface-coverage.sh`: the script reads `lib.rs` for the three codecs (lines 189-210) **and** `src/format.rs` for the binary-only mnemonic-toolkit crate, which has no `lib.rs` (lines 54, 213-216). So the **M2 reword is factually correct and more precise**; §2 line 99 is the incomplete one. M2 did not introduce an error. Optional alignment: bump §2 line 99 to "lib.rs/format.rs" for internal consistency. Non-blocking (predates the fold; M2's scope was the binary-invocation clause).

## Fold confirmation

- **M1 — has-issue (partial).** Id rename landed perfectly: old `technical-manual-codec-g2-uncovered-in-ci` = 0 occurrences; new `technical-manual-g2-uncovered-in-bare-ci` used consistently at line 183 (YAML) and line 293 (step 5). Step 5 broadening (lines 296-300) is correct — enumerates (i)+(ii), cites R0-r1 M1, and the `ABSENT == []` local-catch claim matches Item-2a (lines 140-142). **But the broadening did not reach the YAML comment prose (181-182), which retains the narrow "absent sibling only" framing → see I1.**
- **M2 — landed correctly.** The `*_BIN=true` comment (lines 236-241) accurately states "the `make lint` path never INVOKES a binary," correctly attributes the source reads (api-surface-coverage reads `lib.rs`/`format.rs`; symbol-ref-check ignores the vars), and frames `=true` as belt-and-suspenders against the Makefile's default `cargo run`. Not overstating; verified against `api-surface-coverage.sh` (lines 33-37 ignore `*_BIN`; lines 189-219 read source). Accurate.

## Substance re-confirmation (unchanged by folds)
- Item-2a gate-fix logic (lines 132-167): untouched. Disposition (lines 7-10): untouched. Proof matrix §3 (lines 267-279): untouched. Triggers / tool-install (lines 186-256): untouched. The folds were wording-only as intended.

## Path to GREEN
Fold I1 (broaden the YAML comment 181-182 to match §2/step 5), optionally fold M1' (align §2 line 99), persist this review, then re-dispatch Round 3 to confirm the YAML comment now agrees with §2 and step 5 and that no further drift was introduced.

Relevant paths:
- `/scratch/code/shibboleth/mnemonic-toolkit/design/SPEC_technical_manual_ci_and_transcript_cleanup.md` (lines 181-182, 99, 293-300)
- `/scratch/code/shibboleth/mnemonic-toolkit/docs/technical-manual/tests/api-surface-coverage.sh` (lines 54, 189-219 — confirms M1' and M2)
