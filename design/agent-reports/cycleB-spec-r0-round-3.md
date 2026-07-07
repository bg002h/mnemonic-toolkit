# SPEC R0 review — bitcoin-core-receive-change-pair-merge — round 3

**Verdict:** GREEN (0C / 0I)
**Reviewer:** opus architect, SHA d9063523
**Dispatched:** 2026-07-06 (Cycle B, SPEC R0 loop round 3 — convergence check on rev-3, script-path `tr` scope lock). Persisted verbatim per CLAUDE.md. m3 folded post-GREEN (non-gating copy-edit) before opening the plan.

## Critical (must fix before implementation)
- none

## Important (must fix before implementation)
- none

The design is fully converged. The script-path `tr` scope is now LOCKED (floor-reject) and reconciled across every scope-defining site; §12's "no open design decisions remain" is now TRUE (the decision is *made*, not deferred). The guard matrix remains funds-sound, all oracles are non-tautological, and all source citations verify against `d9063523`.

## Minor (fold if cheap — recommended before impl for full internal consistency; NOT a gate)

- **[m3] §4.3 line 150 — one residual stale-prose site the I-a reconciliation missed** — `SPEC §4.3:149-150`. The construction rule still says replace "for **EVERY** key expression (all cosigners for multisig; **the internal key AND every leaf key for `tr`**)…". "Every leaf key for `tr`" describes a **multi-leaf (script-path) `tr`** construction — but script-path `tr` is now OUT of scope → floor-rejects at §4.2 cond. 1/cond. 7 and can **never reach** §4.3's construction. So the clause is unreachable dead-text and, read in isolation, implies §4.3 *constructs a merged script-path `tr`* — contradicting the locked scope. **Zero funds impact** (the guard is the authoritative gate; §4.3 only runs on entries that already passed it), but it's the 5th prose site that should match the lock. **Fix (delete 6 words):** change to "(all cosigners for multisig; the single key-path key for a single-key bip86 `tr`)". This is a copy-edit, not a design change — I do not hold GREEN on it, but recommend folding it so an implementer isn't nudged to build dead multi-leaf-`tr` replacement logic.

## Citation verification (rev-3 deltas)
- §3 row `BitcoinCoreParser::parse` — "aggregate-dropped build loop `:185-197`; NOTICE emit `:198-205`; per-entry loop `:207-211`; pre-pass inserts after the NOTICE emit (`:205`), before `:207`": **ACCURATE** (verified: build loop 185-197, `if !aggregate_dropped.is_empty()` writeln block 198-205, `for … enumerate()` at 209, `parse_entry` push 210). m2 fully resolved.
- §4.1 placement "after the aggregate-dropped-fields NOTICE emit (`bitcoin_core.rs:198-205`; see §4.5) and before the ParsedImport loop (`:207`)": **ACCURATE** — the anchor is now tight and the ordering intent (NOTICE on original array, then merge) is unambiguous.
- §0 non-goal 4 label "#10 (multisig) / #15 (tr)": **ACCURATE** (§8 item 10 = `core_multisig_receive_change_pair_merges`; item 15 = the two `tr` tests). m1 fully resolved.
- taproot fixture `core-bip86-mainnet.json` = `tr(…/<0;1>/*)`: **ACCURATE** (unchanged).
- All other anchors (`:361`, `:326-327`, `:411`/`:427`, `:158-177`, five checksum copies, `:203-213` floor, `:1859`/`:2265`, `:926`/`:1108`/`:952` tests, `:355`, `:534`): **ACCURATE** (unchanged from rounds 1-2, re-confirmed).

## I-a reconciliation — site-by-site consistency check (the round-3 focus)
Script-path `tr` must read "floor-reject / does-not-merge / out-of-scope" with **no residual "merges" claim** anywhere. Verified across all sites:
- **§0 scope table (`:23`):** "Single-key bip86 `tr` IN scope (merges like `wpkh`); **Script-path `tr` … OUT of scope → floor-reject**." ✓ CONSISTENT.
- **§0 non-goal 3 (`:28`):** script-path `tr` added to the no-merge list ("these DO NOT merge — they reject"). ✓ CONSISTENT.
- **§4.2 cond. 1 (`:120-121`):** "Covers … single-key `tr` (key-path bip86). **Script-path `tr` (tapscript leaves) → floor-reject.**" ✓ CONSISTENT.
- **§4.2 cond. 7 (`:139-141`):** "single-key `tr` … merges like `wpkh` (one key, one step). A **script-path `tr` … is OUT of scope → floor-reject**." ✓ CONSISTENT.
- **§8.15 (`:277-283`):** first test `core_tr_bip86_receive_change_pair_merges` (mandatory, §8.4 oracle); second test `core_tr_scriptpath_pair_does_not_merge` — "OUT of scope: the guard does NOT merge it; it falls to the floor reject (exit 2). **LOCKED behavior (§0 / §4.2 cond. 7), NOT contingent on fixture feasibility.**" ✓ CONSISTENT — the round-2 "decided in P-planning" escape hatch is GONE.
- **§12 item 4 (`:346-347`):** "LOCKED: single-key bip86 `tr` merge mandatory; script-path `tr` OUT of scope → floor-reject. No open decision." ✓ CONSISTENT.
- **§12 intro (`:337-338`):** taproot scope added to the locked list; "**No open design decisions remain.**" ✓ **NOW TRUE** — every prior "R0 to decide" item (range, mechanism, checksum, message, taproot scope) is locked.
- **Residual (m3, `§4.3:150`):** the construction-rule parenthetical "every leaf key for `tr`" — the sole site still *implying* a multi-leaf `tr` is constructed/merged. Flagged Minor above; unreachable, zero funds impact.

**Net: §0 ↔ §4.2 ↔ §8.15 ↔ §12 are mutually consistent; §12's "no open decisions remain" is TRUE.** The only place script-path `tr` is even implicitly "constructed" is the dead §4.3 parenthetical (m3), which cannot execute.

## Notes
- **Full-fold audit (rounds 1-2, 14 findings): all resolved.** I1 (explicit merge-provenance flag, shape-detection forbidden, §8.13 select assertions) · I2 (all-keys structural replacement + original-anchored address oracle both chains) · I3/I-a (taproot scoped: single-key merges, script-path floor-rejects) · I4 (grouping key excludes final step, positional) · I5 (range union/never-blocks, all decisions locked) · M1-M9 (checksum count/6th copy, manual `:1404-1405`+block, non-goal caveat, hardened/wildcard-hardness, parse_bool_field, dropped-fields NOTICE ordering, `both` token, remove-both-insert-one, input-checksum validation). No fold introduced new drift beyond the single m3 copy-edit.
- **Guard-matrix funds-safety re-affirmed:** any pair passing conditions 1-7 is byte-identical except an unhardened fixed final step over positionally-identical keys/template — provably one wallet's two chains, whose `<recv;chg>` expands element-wise to exactly the two originals. No wrong-merge can pass. The §8.4/§8.10/§8.15 original-anchored oracles are the backstop against a misfired all-keys replacement (which verify-bundle alone would false-pass — the Cycle A C1 class).
- **Plan-phase (NOT SPEC-gate) carry-forwards for the implementer** — flagged so they aren't lost, none blocking:
  1. The pre-pass must reliably **classify key-path vs script-path `tr`** (e.g. via the parsed key structure / presence of a tap tree, since it operates on raw JSON before `parse_entry`) so a script-path `tr` deterministically fails the mergeable grouping and floor-rejects (enforces the §4.2 cond. 1/7 lock). `core_tr_scriptpath_pair_does_not_merge` (§8.15) is the gate on this.
  2. Confirm the §8.4/§8.10/§8.15 oracle can obtain the **merged descriptor string** (e.g. `original_descriptor` in `--json`, or a verify-bundle/inspect round-trip) so the address comparison is built in its non-tautological form and cannot silently regress.
  3. The merged JSON entry must carry `active = A.active || B.active` as a written field (parse_entry reads `active` from JSON while `internal` is threaded by param — §5); ensure the pre-pass writes it.
- **Disposition:** SPEC is R0-GREEN (0C/0I). Recommend folding m3 (6-word copy-edit) in the same pass that opens implementation — it needs no re-review. Proceed to the IMPLEMENTATION_PLAN, which itself must pass its own opus R0 loop to 0C/0I before any implementer dispatch (CLAUDE.md), then per-phase TDD + the mandatory post-impl whole-diff review, weighted toward the merge-guard + the original-anchored address oracles.
