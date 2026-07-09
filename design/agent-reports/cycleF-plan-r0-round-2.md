# PLAN R0 review — ms1-repair-demote-to-candidate — round 2

**Verdict: NOT GREEN (0 Critical / 1 Important / 0 Minor)**
**Reviewer:** Fable architect (funds-weighted, read-only), per user directive. Plan rev-2 @ toolkit `c8d567bb`; verified vs source `b20e3ce7`, ms `c2fd4eb`, + a LIVE run of `target/debug/mnemonic`.
**Dispatched:** 2026-07-09 (Cycle F, plan-R0 loop round 2). Persisted verbatim per CLAUDE.md.

The I1 merge is structurally correct + removes the double-flip; §5.6/5.7/5.8 homed + implementable; M1-M4 folded cleanly; release ritual + G1-G9 complete. One factual error remains in the flip inventory — a wrong exit-code target Fable introduced in round 1, folded verbatim, now caught by RUNNING the binary.

## Convergence checks — all PASS except the one exit-code number
1. **Merge removes the transient — VERIFIED.** One P0 rewires the 2 verify-bundle ms1 sites to a direct pure `repair_card` call in the SAME phase that adds the advisory at the helper fall-through (`repair.rs:1701`). After the rewire, the only Ms1 helper callers are convert/inspect/xpub → advisory standalone-inline-only from the first commit; mk1/md1 verify-bundle helper callers (`:2122/:2339/:2360/:2974`) never fire the ms1-gated advisory (G4). Advisory MUST gate on `outcome.kind==Ms1` at `:1701` (mk1 partial-set also reaches that arm) — plan states it.
2a. **cell_27=exit 0 / cell_30=exit 0 — VERIFIED.** `synth_corrupted_bundle_json(17)` corrupts ms1[0] of the seed's OWN bundle; `@0.phrase` is that same seed → `expected.ms1[0]`=correct card; a single subst is uniquely BCH-correctable (t≤4) back to the original → corrected==expected → MATCH → ms1 checks pass → all-clean → exit 0. D20 short-circuit envelope ms1-unreachable (correct).
2b. **cell_19/cell_18b=exit 2 — WRONG, actual exit 1** → the Important.
3. **§5.6/5.7/5.8 homed + implementable — VERIFIED.** §5.6 CLI-level (not unit `indel_exit_code_precedence` @:2689); indel `recover_indel_card`→`Unique` subst_count 0 → `indel_exit_code(false,false,1)=5`, untouched by the substitution-arm-only demotion → keep-5 holds; multi-hit→Ambiguous→4. §5.7 ms1 Unverified→candidate_seen→exit 4 OR-fold. §5.8 direct call inside the pre-existing `if !no_auto_repair` guard; advisory suppressed via the gated helper call (`seed_intake.rs:180-186`); cell_28 extension valid.
4. **M1-M4 folded — VERIFIED.** M1 redaction `diff_byte_offset:None` + Zeroizing + §8.6 both-substring scan (codec already withholds input on error — reinforces achievability). M3 CHANGELOG head=`[0.80.0]` (CHANGELOG.md:9). M4/G9 ms-cli `RepairJson` @:204-210 D27 "Field order is part of the schema" (:200-203) → `verdict` REQUIRED at identical position. False doc-comments (`repair.rs:443-444`/:1145-1147) real.
5. **Release ritual + G1-G9 — COMPLETE** (4-site pin advance complete gate-set; `manual-gui.yml:165` excluded; re-vendor N/A; freebsd gate `@1.85.0`).

## IMPORTANT-1 — flip-inventory exit code for cell_19/cell_18b is wrong: actual is exit 1, not exit 2
**Evidence (LIVE run of current `target/debug/mnemonic`):**
```
$ MNEMONIC_FORCE_TTY=0 mnemonic convert --from ms1=<corrupted@17> --to phrase
error: ms1 codex32: invalid short checksum (50 chars; input withheld)   exit=1
$ MNEMONIC_FORCE_TTY=0 mnemonic inspect --ms1 <corrupted@17>
error: ms1 codex32: invalid short checksum (50 chars; input withheld)   exit=1
```
A single in-place substitution leaves length/HRP valid, fails ONLY the checksum → `ms_codec::Error::Codex32(_)` → `ms_codec_exit_code` maps `Codex32(_) => 1` (`error.rs:434-436`). The inventory rows (`cli_auto_repair.rs:52` cell_19, `:143` cell_18b) say **"exit 2 (original decode error surfaces)"** — the "original decode error surfaces" logic is right, the NUMBER is not: it is **1**, not 2. The demotion does not change this path (only stops the short-circuit) → exit 1 is also the post-cycle target. Origin: Fable's round-1 finding asserted "exit 2" without running it, folded verbatim; the existing TTY-negative cells 31/32/33 (`:611-658`) only assert `code(ne(5))`, so nothing pinned the true value. A TDD test written to the inventory would encode `.code(2)` and go red (or an implementer might "reconcile" the exit mapping — a real drift trap).
**Fix:** change cell_19 + cell_18b targets to **"exit 1 (`Codex32` invalid-checksum → `ms_codec_exit_code` ⇒ 1) + advisory — NOT 4"**; the §5.3 test-cell parenthetical (currently "exit 2") likewise → exit 1. cell_24 (`--json`) pins no exit (fine; surfaces exit 1 if asserted). cell_9=4 ✓, cell_27=0 ✓, cell_30=0 ✓ re-verified — no other number affected.

## Bottom line
Single Important — a one-cell number correction (2→1) with evidence given. No Critical/other-Important/Minor. Fold + re-dispatch round 3. (Reviewer flags this defect as its own round-1 carry-over — running the binary caught what reasoning about the codec taxonomy missed.)
