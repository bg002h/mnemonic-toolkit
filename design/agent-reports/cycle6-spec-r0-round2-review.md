# Cycle-6 brainstorm spec — R0 review, round 2

- **Spec under review:** `design/BRAINSTORM_cycle6_timelock_decay.md` (post round-1 fold)
- **Cluster:** timelock decaying-multisig decay-ordering funds-loss (D-decay-rel cross-unit BIP-68 `older()` blindness; D-decay-abs past `after(T)` immediately-spendable).
- **origin/master SHA:** `3fa2925b02a0d29846e823a1bfd4cec44256f70b` (toolkit 0.63.0, post-cycle-5/S-NET)
- **Date:** 2026-06-21
- **Reviewer:** opus software architect (independent; round-1 findings treated as hypotheses, folds re-verified against live source, NOT against the spec's self-assessment)

Round 1 was **1C / 2I (RED)**. This round verifies the three folds (C1, I1, I2) against live `3fa2925b` source and checks that the folds introduced no new drift. Every coupled-site row in §7 was independently re-grepped and traced through the predicate + dispatch order.

---

## Fold verification (against live source)

### C1 (self-contradictory positive-control value) — RESOLVED

**Round-1 defect:** the spec recommended `--after 900000000` as the canon/positive-control "future" value, but that value is REFUSED by its own D-decay-abs predicate (`900_000_000 > 500_000_000` ⇒ time-classified, but `< ABS_TIME_PAST_FLOOR = 1_750_000_000` ⇒ past ⇒ refused; it is 1998-07-09).

**Fold verified:**
- `grep '900000000'` over the spec returns exactly THREE hits (`:205`, `:233`, `:266`), and ALL THREE are inside explicit "Why NOT `900000000`" / "NOT a valid control" rejection notes. No surviving recommendation.
- The canon/positive-control value is now `--after 4000000` consistently: R7 (`:233`), §4.2 (`:204`), the §0 table (`:29`, `:35`-`:37`), and the §7 abs control rows (`:258`, `:263`-`:264`).
- The RED value is `--after 500000` (`:257`, `:262`); the mutation base is `--after 4000001`/`--after 500001` handled at row #8 (`:279`).
- **Arithmetic independently recomputed** under the spec's floors `{ABS_HEIGHT_PAST_FLOOR=900_000, ABS_TIME_PAST_FLOOR=1_750_000_000}` and the BIP-65 `500_000_000` split:
  - `4000000` → `< 500_000_000` ⇒ height; `> 900_000` ⇒ FUTURE → **builds** ✓ (valid positive control)
  - `4000001` → height, future → **builds**, `≠ 4000000` ⇒ descriptor differs ✓ (valid mutation base)
  - `500000` / `500001` → height, `< 900_000` ⇒ PAST → **refused** ✓ (valid RED)
  - `900000000` → time, `< 1_750_000_000` ⇒ PAST → **refused** ✓ (correctly rejected as a control)
  - Datetimes confirmed: `900000000`=1998-07-09, `1_750_000_000`=2025-06-15 — both match the spec's stated dates.
- The "same height axis as `500000`, no axis flip, minimal golden churn" rationale (R7) holds: `4000000` stays height-classified like `500000`.

C1 is fully resolved and applied uniformly.

### I1 (in-crate `validate_params_decay_ordering` RED) — RESOLVED

**Round-1 defect:** `archetype.rs:743` `validate_params_decay_ordering` consumes `fixture_params("decaying-multisig")` which sets `after: Some(500000)` (`:542`), loops `recovery_older ∈ {1000, 999}`, asserts `diags.len() == 1`. Under the abs predicate the past `500000` adds a SECOND diag ⇒ `len() == 2` ⇒ RED. Not in the round-0 "three sites" budget.

**Fold verified against live source:**
- `archetype.rs:542` `after: Some(500000)` confirmed in `fixture_params` (with `older: Some(1000)`).
- `archetype.rs:743` test confirmed: loops `[1000, 999]`, mutates `recovery_older`, asserts `assert_eq!(diags.len(), 1)` + message contains `--recovery-older` and `--older`.
- Spec now enumerates this as inventory **row #2** (`:273`) AND in the §0 table (`:33`-`:34`), prescribing `after: Some(4000000)`, with the load-bearing analysis spelled out (future `after` ⇒ abs predicate silent ⇒ only the rel diag fires for `recovery_older ∈ {1000,999}` ⇒ `len()==1` holds).
- Predicate trace independently confirmed: for both `recovery_older` values, `u1==u2` (Blocks) and `v2 <= v1` (1000≤1000 equal; 999≤1000) ⇒ exactly the rel diag fires; with `after=4000000` (height, future) the abs predicate is silent ⇒ `len()==1`. Correct.

I1 is resolved; the site is enumerated and the fix is precise.

### I2 (CLI `repeated_keys` localization RED) — RESOLVED

**Round-1 defect:** `cli_build_descriptor.rs:~1020` (`repeated_keys` block) carries `--after 500000`; under the abs fix `validate_params` short-circuits exit 2 BEFORE the gate, so `diagnostics[0].kind` becomes `"param"`/`node_path "params"` instead of `"repeated_keys"`/`"root.andor[2]"` ⇒ both `assert_eq!` break. Not enumerated.

**Fold verified against live source:**
- The block at `cli_build_descriptor.rs:996-1031` confirmed: `--older 1000 / --recovery-older 2000` (correctly ordered same-unit), `--final-key K3` (dup with `--recovery-key K3`), `--after 500000` (`:1020`), `--json`, asserts `.code(2)` + `d["kind"]=="repeated_keys"` + `d["node_path"]=="root.andor[2]"` + `flag` absent.
- Dispatch order confirmed at `build_descriptor.rs:278-287`: `validate_params` (`:278`) → `return Ok(2)` (`:280`) runs BEFORE `gate::validate_with_allow` (`:287`). So a past-`after` param diag genuinely pre-empts the gate. `param_diag` (`:326`) sets `node_path:"params"`, `kind:Param` — confirming the broken-assertion mechanism exactly.
- Spec now enumerates this as inventory **row #9** (`:280`) AND in the §0 table (`:32`), prescribing `--after 4000000` with the full short-circuit analysis. Correct.

I2 is resolved; the site is enumerated and the fix is precise.

---

## Coupled-site table — completeness + correctness audit (independently re-grepped @ `3fa2925b`)

I ran `git grep -n '500000|500001'` over `crates/mnemonic-toolkit/src` + `tests` at `3fa2925b`. Every decay-relevant hit is accounted for in the spec's §7 table:

| Live hit @ `3fa2925b` | Spec row | Classification | Independent verdict |
|---|---|---|---|
| `archetype.rs:33` doc comment | #1 | MIGRATE | CORRECT — pure prose, stale-doc hygiene; still feeds `--spec-schema`/manual metadata. |
| `archetype.rs:542` `after: Some(500000)` | #2 | MIGRATE | CORRECT (I1) — load-bearing; `len()==1` assertion depends on it. |
| `mod.rs:66` canon string golden | #3 | MIGRATE | CORRECT — `older(1000)/older(2000)` (rel passes) + `after(500000)`; migrating keeps a genuine positive control. |
| `decaying-multisig.json:19` | #4 | MIGRATE | CORRECT — source spec the goldens derive from. |
| `decaying-multisig.descriptor:1` (`#llvl05j9`) | #5 (+checksum regen) | MIGRATE | CORRECT — byte-exact golden ends `after(500000)))))#llvl05j9`; checksum MUST be re-derived (explicitly noted). |
| `decaying-multisig.bip388:2` | #6 | MIGRATE | CORRECT — `description_template` embeds `after(500000)`; distinct template-string edit. |
| `cli_build_descriptor.rs:82` canon `preset_args` | #7 | MIGRATE | CORRECT — drives the `.descriptor`/`.bip388`/`.json` goldens in lockstep. |
| `cli_build_descriptor.rs:683` mutation `("--after","500001")` | #8 | MIGRATE | CORRECT — harness asserts `.success()`; past `500001`→exit 2 would break it; future base `4000001` preserves the mutation intent. |
| `cli_build_descriptor.rs:1020` repeated_keys | #9 | MIGRATE | CORRECT (I2) — short-circuit breaks `repeated_keys` assertion. |
| `cli_build_descriptor.rs:571` decay-neg `2000/2000` | `:570` row | MIGRATE (cleanliness) | CORRECT — assertions are stderr-substring only so not strictly RED, but the abs predicate would add a second diag; migration isolates the rel path. Honest "not strictly RED" labelling. |
| `ir.rs:390` `After(500000).render()` | U1 | UNAFFECTED | CORRECT — pure IR render unit test; no `validate_params`/archetype path. |
| `cli_compare_cost.rs:1123` `after(1500000000)` | U2 | UNAFFECTED | CORRECT — raw `--miniscript` string; the file has ZERO `decaying`/`build-descriptor`/`archetype`/`validate_params` refs (`git grep -c` = 0). |

**One extra `500000` hit found that the table omits — correctly omits:** `tests/fixtures/wallet_import/specter-descriptor-with-checksum.json:3` `"blockheight": 500000`. This is a Specter *wallet-import* metadata field on a `wpkh` single-sig descriptor (no `after()` anywhere), routed through `wallet_import/specter.rs` / `cli_import_wallet_specter.rs` — entirely outside the decaying-multisig/`validate_params` path. NOT a decay coupled-site; the spec's omission is correct, not a miss.

**Verdict on the table:** COMPLETE and CORRECTLY CLASSIFIED. Every MIGRATE site genuinely needs migration (abs predicate fires on its past baseline, or it is a canon golden whose emitted miniscript changes, or — for `:570`/`#1` — a hygiene/doc tick), and both UNAFFECTED sites are genuinely outside the `validate_params` path (verified by tracing, not by trusting the label). No missed or mis-classified site.

---

## No-new-drift scan

- **"three coupled sites" leftover:** every occurrence (`:268`, `:277`, `:285`, `:287`) explicitly frames "three" as the *rejected recon round-0 budget* ("budgeted only", "EXCEEDS the … budget"), never as current truth. The live count is stated as NINE MIGRATE + TWO UNAFFECTED with an honest tenth-`:570`-or-hygiene counting note. Internally consistent.
- **Self-contradictory RED/control values:** none survive. `4000000`=future/control, `500000`=past/RED, `4000001`=future/mutation, `900000000`=explicitly-rejected. The §7 D-decay-rel table (`:246`-`:250`) controls are sound: cross-unit `145`/`4194305` REFUSED (units differ — `4194305=0x40_0001`=Seconds512/value-1 confirmed against the live `0x0040_0000` bit constant and the `older_consensus_masked`→`None` clean-operand tests); both-512s `0x40_0001`/`0x40_0002` BUILDS (same unit, strictly later). The `:247` "Today" cell ("exit 0 … actually refused today too") is muddled prose but self-corrects in the same parenthesis and does not affect the after-fix column or any fixture — Minor at most, see M1 below.
- **Resolved-decisions table (R1-R8) vs folded body:** consistent. R7 (`:233`) now reads `4000000`, matching §4.2/§7. R3/R4 (Param exit-2; `older_unit_value` helper) unchanged and still match §4.1/§4.2. R5 (keep FOLLOWUP open, narrow resolution) matches §8 (`:295`). R2/R6 (static floors, no oracle row) unchanged.
- **Core fix design unchanged + still sound (re-verified against live source):**
  - cross-unit REFUSE + same-unit strict value-ordering (R1) — sound; `--spec` bypass of `validate_params` confirmed at `build_descriptor.rs` (archetype arm calls `validate_params`; the `read_spec` arm does not), so mixed-unit authoring remains expressible.
  - static-floor abs future-ness (R2) — floors arithmetically monotone-safe (only false-negative, never false-positive on a legitimate future locktime); `4000000` clears `900_000`, legitimate forward-dated heights never refused.
  - `older_unit_value(n) -> (TimelockUnit, u16)` helper (R4) — JUSTIFIED: `timelock_advisory.rs` `TimelockUnit{Blocks,Seconds512}` exists; `older_consensus_masked` returns `None` for clean operands (live tests `[1,2016,52560,65535]` + `[0x40_0001,0x40_FFFF]` ⇒ `None`), so it cannot classify a clean operand's unit; inline `n & 0x0040_0000` / `n & 0x0000_FFFF` matches the live bit constants.
  - `DiagnosticKind::Param` exit-2 (R3) — `Param` variant exists (`gate.rs`, `as_str "param"`); `param_diag` sets `flag: Some(..)` provenance; no new `ToolkitError` variant ⇒ CLAUDE.md alphabetical rule N/A. SPEC §3.3 note confirmed at `SPEC_descriptor_builder_presets.md:119`; SPEC §3.1 item 3 unit-blind rule confirmed at `:110` (lockstep update flagged).
  - SemVer MINOR → 0.64.0 (master 0.63.0 confirmed), md/mk/ms NO-BUMP, GUI NO-BUMP (zero clap-flag/`schema_mirror`/dropdown/`--json`-shape/manual-flag delta), oracle N/A (`bitcoind_differential.rs` never consumes build-descriptor) — all unchanged and confirmed.
- **§0 citation re-verification:** the citations the folds depend on are LIVE-accurate at `3fa2925b` — `archetype.rs:305-317` raw compare, `:542` fixture, `:743` test, `mod.rs:66` golden, `build_descriptor.rs:278/287` dispatch, `gate.rs` After arm range-only, FOLLOWUP slug header `:219`, SPEC `:110`/`:119`.

---

## Minor (non-blocking; documentation polish only)

### M1 — §7 D-decay-rel row `:247` "Today" cell is internally muddled (cosmetic)
The `rel-same-unit-mis-ordered` row (`--older 2000 / --recovery-older 1000`) lists "Today: exit 0 (raw `1000 <= 2000`… actually refused today too — keep as a unit-aware regression guard)." The current code `recovery_older <= older` ⇒ `1000 <= 2000` ⇒ TRUE ⇒ refused exit 2 TODAY. The cell's leading "exit 0" is wrong and self-corrects in the same parenthesis. This is a test labelled "RED-ish" that is actually already-GREEN-as-a-regression-guard; the after-fix column (exit 2) and all fixtures are correct, so nothing breaks. Recommend tightening the cell to "exit 2 today (raw `1000<=2000`); kept as a unit-aware regression guard" for implementer clarity. Informational — does not affect any GREEN/RED path.

### M2 — carry-over informational notes M3/M4/M5 from round 1 remain optional
Round-1 M3 (`param_diag` carries structured `flag`, so `--json` shape gains a `flag` key — already true and desirable), M4 (gate's `MixedTimelock` guard does not overlap the cross-unit decay case — predicate-1 is genuinely additive), and M5 (decay-before-mask message precedence is fail-closed) were all optional documentation polish and remain so. The spec's §4.1 belt-and-suspenders note already covers the M5 substance. No blocker.

---

## Verdict

**R0 ROUND 2: 0C / 0I** — GREEN.

All three round-1 findings (C1 self-contradictory control value; I1 in-crate `validate_params_decay_ordering` RED; I2 CLI `repeated_keys` RED) are RESOLVED and verified against live `3fa2925b` source, not against the spec's self-assessment. The coupled-site migration table is COMPLETE (every live `500000`/`500001` decay hit enumerated; the one extra `specter` hit correctly omitted as a non-decay wallet-import field) and CORRECTLY CLASSIFIED (each MIGRATE traced to a real RED/golden-change; each UNAFFECTED traced to a non-`validate_params` path). No new drift: the rejected "three sites" budget is consistently framed as historical, no surviving self-contradictory control value, the resolved-decisions table matches the folded body, and the core fix design (cross-unit refuse, static-floor abs future-ness, `older_unit_value` helper, `DiagnosticKind::Param` exit-2, MINOR 0.64.0, no lockstep, oracle N/A) is unchanged and re-confirmed sound. The two Minors (M1 cosmetic table cell, M2 carry-over informational notes) are non-blocking documentation polish.

The brainstorm spec is R0-GREEN and may proceed to the SPEC stage (which itself carries its own mandatory R0 loop before any implementation).
