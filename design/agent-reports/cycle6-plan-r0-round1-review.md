# Cycle-6 IMPLEMENTATION PLAN — R0 review, round 1

- **Plan-doc under review:** `design/IMPLEMENTATION_PLAN_cycle6_timelock_decay.md`
- **Implements (GREEN spec, 0C/0I @ spec-R0 round 2):** `design/BRAINSTORM_cycle6_timelock_decay.md`
- **Cluster:** timelock decaying-multisig decay-ordering funds-loss (D-decay-rel cross-unit BIP-68 `older()` blindness; D-decay-abs past `after(T)` immediately-spendable).
- **origin/master SHA:** `3fa2925b02a0d29846e823a1bfd4cec44256f70b` (toolkit 0.63.0, post-cycle-5/S-NET)
- **Date:** 2026-06-21
- **Reviewer:** opus software architect (independent; plan claims treated as hypotheses, every edit-site / line / predicate re-verified against live `3fa2925b` source — the plan's claims are NOT evidence)

The plan executes the R0-GREEN brainstorm spec faithfully. I independently re-grepped every cited edit site, recomputed both predicates' arithmetic against the live bit-math, re-grepped the complete `500000`/`500001` coupled-site inventory, traced the validate→gate dispatch order, and confirmed the version sites against the release ritual. **No Critical and no Important findings.** Five Minors (one operator drift from the GREEN spec, one non-existent enum-variant name typo, one already-GREEN test mislabelled RED, one omitted version-site that self-heals, one imprecise `cost/enumerate.rs:74` citation) — all behaviorally inert against every fixture and none blocks GREEN, but each is worth a one-line reconciliation so the implementer pins to the reviewed spec.

---

## Edit-site verification (independently re-grepped @ `3fa2925b`)

| Plan claim | Live evidence | Verdict |
|---|---|---|
| `archetype.rs:306-317` raw `recovery_older <= older` compare in `validate_params` decay arm | `:305` `if def.id == "decaying-multisig"`, `:306` `if let (Some(older), Some(recovery_older))`, `:307` `if recovery_older <= older`, push through `:315` | ACCURATE (the block is `:305-317`; the plan's "306-317" names the body — fine) |
| Flag-name constants `RECOVERY_OLDER`/`OLDER`/`AFTER` (used by the new `param_diag` calls) | `:96` `OLDER`, `:97` `RECOVERY_OLDER`, `:98` `AFTER` (all `&str`) | ACCURATE — the predicate code's `RECOVERY_OLDER`/`AFTER` symbols exist |
| `param_diag` (`:326`) → `Diagnostic{node_path:"params", kind:Param, flag:Some(..)}` → exit 2 | confirmed `:326-333`; sets `flag: Some(flag.to_string())` | ACCURATE — provenance preserved; no new variant |
| `DiagnosticKind::Param` exists (exit 2, no new variant/`ToolkitError`) | `gate.rs:90` enum, `:97` `Param`, `:123` `as_str => "param"` | ACCURATE |
| `validate_params`→gate dispatch: producer FIRST `return Ok(2)`, gate SECOND | `build_descriptor.rs:278` `validate_params`, `:280` `return Ok(2)`, `:287` `gate::validate_with_allow` | ACCURATE — confirms exit-2 + I2 short-circuit mechanism |
| `timelock_advisory::older_consensus_masked` returns `None` for CLEAN operands; `TimelockUnit{Blocks,Seconds512}` | `timelock_advisory.rs:18-22` enum; `:48` predicate `(n & !0x0040_FFFF)!=0 || (n & 0xFFFF)==0`; tests `:249-254` prove `[1,2016,52560,65535]`+`[0x40_0001,0x40_FFFF]`⇒`None` | ACCURATE — `older_unit_value` helper genuinely justified (the existing fn cannot classify a clean operand's unit) |
| `cost/enumerate.rs:74` BIP-65 `500_000_000` height/time split | `:72/:74` are DOC COMMENTS referencing `500_000_000`/`N<500_000_000`; the literal is used in computation `:238-248`; NO named const `LOCKTIME_THRESHOLD` anywhere in the crate | INACCURATE-as-"reuse" — see Minor M5 (the predicate must hardcode `500_000_000` inline, exactly as the spec §4.2 code does; there is no reusable named constant at `:74`) |
| canon `preset_args` `--after 500000` @ `cli_build_descriptor.rs:81-82` | `:81` `"--after"`, `:82` `"500000"` | ACCURATE |
| `:570-571` decay-neg `2000/2000` + `--after 500000` | `:546` `preset_decay_ordering_violation_exit_2`, `:570/:571` `--after`/`500000`, stderr asserts `--recovery-older`+`--older` | ACCURATE |
| `:683` mutation `("decaying-multisig","--after","500001")` asserts `.success()` then `assert_ne!` | `:683` row; harness `:680-708` `.success()` + descriptor-differs | ACCURATE |
| `:1019-1020` `repeated_keys` localization `--after 500000` asserts `.code(2)`+`kind=="repeated_keys"`+`node_path=="root.andor[2]"` | block `:996-1031`; `:1019` `"--after"`, `:1020` `"500000"`; asserts confirmed | ACCURATE |
| `archetype.rs:33` doc "block HEIGHT `after(500000)`" | `:33` confirmed | ACCURATE |
| `archetype.rs:542` `fixture_params` `after: Some(500000)` (+ `older: Some(1000)`) | `:542` confirmed; `:743` `validate_params_decay_ordering` loops `[1000,999]`, asserts `diags.len()==1` | ACCURATE |
| `mod.rs:66` canon descriptor STRING golden `after(500000)` | `:66` `wsh(andor(...after(500000)...))` | ACCURATE |
| `.json:19` `"after": 500000`; `.bip388:2` template `after(500000)`; `.descriptor:1` ends `after(500000)))))#llvl05j9` | all three confirmed; `.descriptor` DOES carry checksum `#llvl05j9` | ACCURATE — checksum-regen flag is correct |
| UNAFFECTED `ir.rs:390` (`After(500000).render()`), `cli_compare_cost.rs:1123` (`after(1500000000)` raw `--miniscript`), specter `blockheight:500000` | `ir.rs:390` pure render; `cli_compare_cost.rs:1123` raw policy string (file has 0 `decaying`/`build-descriptor`/`archetype`/`validate_params` refs); `specter-descriptor-with-checksum.json:3` `"blockheight":500000` on a `wpkh` single-sig import (no `after()`) | ACCURATE — all three genuinely non-`validate_params` paths |
| Version sites: `Cargo.toml:3`, `README.md:13`, `crates/.../README.md:9`, `scripts/install.sh:32`, `fuzz/Cargo.lock:575`, `CHANGELOG.md` all at 0.63.0 | all confirmed; `readme_version_current.rs` enforces BOTH README markers | ACCURATE (but see Minor M4 — root `Cargo.lock:727` also pins 0.63.0 and is not in the explicit list) |
| FOLLOWUP `archetype-older-blocks-flag-accepts-time-units` @ `FOLLOWUPS.md:219` | `:219` header confirmed | ACCURATE |
| Bughunt report decay rows | `constellation-bughunt-2026-06-20.md:842` (D-decay-rel), `:953` (D-decay-abs) | ACCURATE (table rows, not checkboxes — "tick" = status annotation w/ fixing commit) |
| No new clap flag (`--older`/`--recovery-older`/`--after` pre-exist) ⇒ no schema_mirror/manual/dropdown/codec leg | `build_descriptor.rs:85/90/95` `#[arg(long, requires="archetype")] Option<u32>` | ACCURATE |

---

## Critical

None.

---

## Important

None.

---

## Minor

### M1 — Plan abs-predicate operator is `<=`; the R0-GREEN spec is `<` (drift from the reviewed artifact)
**Where:** plan `:62` ("if height ... and `after <= ABS_HEIGHT_PAST_FLOOR`") and `:64` ("if time ... and `after <= ABS_TIME_PAST_FLOOR`") vs spec §4.2 code `:182-183` `if is_height { after < ABS_HEIGHT_PAST_FLOOR } else { after < ABS_TIME_PAST_FLOOR }` (strict `<`).
**Impact:** The ONLY values whose classification differs between `<` and `<=` are exactly `after == 900_000` and `after == 1_750_000_000`. No fixture, RED test, positive control, or mutation uses either exact value (canon = `4000000`, RED = `500000`/`500001`, mutation = `4000001`, both-512 controls are `older`-side). So neither operator breaks any test, and both directions are fail-closed (refusing the floor value `900_000`, a long-mined past height, is harmless; accepting it is also harmless). **Required reconciliation:** the plan must EXECUTE the GREEN spec — write `<` to match spec §4.2, OR add an explicit one-line note that the implementer deliberately tightens to `<=` (and why). Do not let the implementer pick silently; the spec is the reviewed contract. (Classified Minor only because it is behaviorally inert against every test asset and both choices are fail-closed; flagged because plan-vs-spec operator drift in a funds-safety predicate is exactly the "folds introduce drift" class the loop guards against.)

### M2 — Plan writes `Time512`; the live enum variant is `Seconds512` (non-existent name would not compile)
**Where:** plan `:29` (`{ Time512 } else { Blocks }`) and `:42` (RED test `4194305 → (Time512,1)`). Live `timelock_advisory.rs:21` is `Seconds512`; spec §4.1/§0 correctly use `Seconds512`.
**Impact:** purely a plan-prose typo — `TimelockUnit::Time512` does not exist; `cargo` would reject it, and the implementer reading the spec's `older_unit_value` body (which uses `Seconds512`) would write the correct name. **Required fix:** s/`Time512`/`Seconds512`/ in the plan (lines 29, 42) so the plan reads correctly and the Phase-1 RED-test expectation names the real variant.

### M3 — Plan lists `rel-same-unit-mis-order` (`--older 2000 --recovery-older 1000`) as a RED test; it is ALREADY refused today (a regression guard, not RED-first)
**Where:** plan `:68` ("a same-unit mis-order (`--older 2000 --recovery-older 1000`) → exit 2").
**Evidence:** today's raw `recovery_older <= older` ⇒ `1000 <= 2000` ⇒ TRUE ⇒ already exit 2. After the fix the unit-aware `v2(1000) <= v1(2000)` also refuses ⇒ exit 2. So this case does NOT transition RED→GREEN; it is an already-GREEN unit-aware regression guard. The spec §7 table (`:247`) labels it honestly ("RED-ish ... actually refused today too — keep as a unit-aware regression guard"; the round-2 spec-R0 M1 noted the muddled cell); the PLAN dropped that caveat. **Required fix:** annotate the plan's RED list so the implementer does not expect a RED-first failure for this row — label it "already exit-2 today; retained as a unit-aware regression guard." The genuine RED-first negatives are the cross-unit case (`145`/`4194305` — verified accepted today, exit 0) and the abs-past case (`--after 500000` — verified accepted today, exit 0). Both are non-vacuous.

### M4 — Root workspace `Cargo.lock:727` (toolkit `version = "0.63.0"`) is not in the Phase-4 version-site list
**Where:** plan `:111-112` lists "Cargo.toml + BOTH READMEs + `scripts/install.sh` self-pin + `fuzz/Cargo.lock` + CHANGELOG" — omits the root `Cargo.lock` (which pins the toolkit's own package version `:727`).
**Impact:** the root `Cargo.lock` auto-regenerates on the first `cargo build`/`cargo test` after the `Cargo.toml` bump, and Phase 4 runs the full suite, so it self-heals mechanically (unlike `fuzz/Cargo.lock`, the separately-resolved lockfile that the release-ritual memory flags as silent-drift-prone and which the plan correctly DOES list). **Recommended:** add a one-line note "(root `Cargo.lock` auto-regenerates on the Phase-4 build; verify it shows `0.64.0` before tag)" so the implementer confirms rather than assumes.

### M5 — `cost/enumerate.rs:74` is cited as a "reuse" source for `500_000_000`, but it is a doc comment, not a named constant
**Where:** plan `:35` ("reuse `cost/enumerate.rs:74`") and the spec §0/§4.2.
**Evidence:** `cost/enumerate.rs:72/74` are doc comments mentioning `N<500_000_000`/`N≥500_000_000`; there is NO `const LOCKTIME_THRESHOLD` in the crate (grep is empty); `gate.rs:1041` uses the literal `500_000_000` inline too. The spec §4.2 code correctly hardcodes `let is_height = after < 500_000_000;` (a bare literal). **Required reconciliation:** the plan's "reuse" wording implies a named constant to import; there is none. The implementer should hardcode the BIP-65 literal `500_000_000` inline (matching the spec code and the existing `gate.rs:1041`/`enumerate.rs` convention), NOT search for a non-existent const. Reword "reuse `cost/enumerate.rs:74`" to "BIP-65 literal `500_000_000`, consistent with the doc-comment split at `cost/enumerate.rs:72-74` and the inline use at `gate.rs:1041`."

---

## Verdicts on the prompt's load-bearing questions

**(a) Are both predicates correct? — YES.**

- **D-decay-rel (cross-unit + order):** `older_unit_value(n)` = `(Seconds512 if n&0x0040_0000 else Blocks, (n&0xFFFF) as u16)` matches the live bit-math in `older_consensus_masked` verbatim. The predicate runs on CLEAN operands: clean values (`145`, `0x40_0001`) make `older_consensus_masked`⇒`None`, so the gate Older arm (`gate.rs:270` `if let Some(consequence) = older_consensus_masked(*n)`) does NOT refuse them — they pass the gate AND reach `validate_params:305`. So the new predicate is the FIRST and only check that sees clean cross-unit pairs. Masked operands (bit-31/stray-bits/zero-value) are still independently refused by the gate's Older arm regardless of what the decay check does (belt-and-suspenders; fail-closed). Cross-unit refuse (units differ ⇒ un-orderable offline) + same-unit `v_r <= v_p` (low-16 value, consensus-effective) are both correct. The recon repro `--older 145`(Blocks) / `--recovery-older 4194305=0x40_0001`(Seconds512,v=1) is refused (units differ) — the funds-loss case.

- **D-decay-abs (monotone-safe):** independently recomputed under `{ABS_HEIGHT_PAST_FLOOR=900_000, ABS_TIME_PAST_FLOOR=1_750_000_000}` + the BIP-65 `500_000_000` split: `500000`/`500001`→height-PAST→refused (catches the bug); `4000000`/`4000001`→height-FUTURE→builds (positive control + mutation hold); `900000000`→time-PAST→refused (correctly NOT a control; it is 1998-07-09); `2_000_000_000`/`1_750_000_001`→time-FUTURE→builds. The floors sit BELOW current chain state (mainnet tip ≈ 910k mid-2026 > `900_000`; `1_750_000_000` = 2025-06-15, already elapsed), so the check is structurally **only ever false-negative, never false-positive on a legitimately-future locktime**. A legit future `after` is always either a height `> 900_000` or a time `> 1_750_000_000` ⇒ never refused. Monotone-safe confirmed. (The one operator nit M1 — `<` vs `<=` — touches only the exact floor value and is fail-closed either way.)

**(b) Is the coupled-site migration complete (any missed/mis-classified)? — YES, complete; none missed, none mis-classified.**

`git grep -n '500000|500001'` over `crates/mnemonic-toolkit/{src,tests}` @ `3fa2925b` returns exactly: `archetype.rs:33`, `archetype.rs:542`, `ir.rs:390`, `mod.rs:66`, `cli_build_descriptor.rs:82`, `:571`, `:683`, `:1020`, `cli_compare_cost.rs:1123`, `.bip388:2`, `.descriptor:1`(#llvl05j9), `.json:19`, plus the specter wallet-import `blockheight:500000`. The plan's 10 MIGRATE rows (`:33`, `:542`, `mod.rs:66`, `.json`, `.descriptor`+checksum, `.bip388`, `cli:81-82`, `cli:570-571`, `cli:683`→`4000001`, `cli:1019-1020`) cover every `validate_params`-reachable hit; the 3 UNAFFECTED (`ir.rs:390` pure render, `cli_compare_cost.rs:1123` raw `--miniscript`, specter `blockheight`) are each independently traced to a non-`validate_params` path. The `:542` fixture migration is load-bearing (keeps `:743` `diags.len()==1`); the `:683` mutation→`4000001` (preserves `.success()` + `assert_ne!`); the `:1019-1020`→`4000000` (lets `validate_params` pass so the gate's `repeated_keys` path runs). The `.descriptor` checksum `#llvl05j9` regen is correctly flagged (the fixture carries it; a stale checksum fails parse). All exact line numbers verified against live source.

**(c) Are all RED tests genuinely RED-first? — MOSTLY YES (two genuinely RED; one mislabelled — see M3).**

- `rel-cross-unit` (`--older 145` / `--recovery-older 4194305`): TODAY raw `4194305 <= 145` is FALSE ⇒ NOT refused ⇒ exit 0 ACCEPTED, mis-ordered descriptor EMITTED. Genuinely RED. ✓
- `abs-past` (`--after 500000`): TODAY in-range `[1,0x7FFF_FFFF]` ⇒ gate passes ⇒ exit 0 (corroborated by the existing `:683` `.success()` on `500001`). Genuinely RED. ✓
- `rel-same-unit-mis-order` (`--older 2000` / `--recovery-older 1000`): ALREADY refused today (raw `1000<=2000`) ⇒ NOT RED-first; it is an already-GREEN unit-aware regression guard. The plan mislabels it RED (M3). Non-vacuous, but not a RED→GREEN transition.

Positive controls all genuinely build: same-unit ordered (`1000/2000` blocks), both-512s (`4194305/4194306` = Seconds512 v1<v2), future `after(4000000)`. No vacuous test.

**Phase ordering (P2+P3 must land together) — CORRECT.** The canon truly uses past `after(500000)` (canon `preset_args:82`, `mod.rs:66` golden string, all three fixtures, the in-crate `:743` fixture). The abs predicate (P2) refuses `500000`, so committing P2 without the P3 migration would turn every canon golden build RED (`preset_descriptor_goldens`, `preset_bip388_goldens`, `mod.rs::decaying_multisig`, `validate_params_decay_ordering`). They MUST land together. ✓

**SemVer / version sites — CORRECT (one omission, M4).** master = 0.63.0 ⇒ MINOR → 0.64.0 (tightening the producer's accepted input set is behavioural). No new clap flag (the three timelock flags pre-exist) ⇒ zero schema_mirror flag-name delta, zero dropdown delta, zero manual flag-row delta, zero `--json` wire-shape change; md/mk/ms NO-BUMP; bitcoind oracle N/A (never consumes build-descriptor). Version sites enumerated (Cargo.toml + both READMEs + install.sh + fuzz/Cargo.lock + CHANGELOG) match the release ritual; the root `Cargo.lock` self-heals on build (M4).

---

## Verdict

**PLAN R0 ROUND 1: 0C / 0I — GREEN (0C/0I).**

The plan faithfully executes the R0-GREEN brainstorm spec. Every cited edit site exists at the cited line(s) @ `3fa2925b`; both predicates are arithmetically correct (rel cross-unit-refuse + same-unit strict-order on clean operands; abs monotone-safe static floors that catch `after(500000)` yet never refuse a legit future); the coupled-site migration is complete and correctly classified (10 MIGRATE + 3 UNAFFECTED, exact lines, checksum-regen flagged); the genuine RED-first negatives (cross-unit, abs-past) are non-vacuous; the P2+P3 co-landing requirement is correct; SemVer/version-sites are complete. The five Minors (M1 `<=`-vs-`<` operator drift from the spec; M2 `Time512`→`Seconds512` typo; M3 mislabelled already-GREEN regression guard; M4 omitted self-healing root `Cargo.lock`; M5 imprecise `cost/enumerate.rs:74` "reuse" citation) are all behaviorally inert against every test asset and do NOT block GREEN — but folding M1/M2/M3/M5 before implementation will keep the implementer pinned to the reviewed spec and avoid a non-compiling variant name. Per the reviewer-loop discipline, fold the Minors (optional) and re-dispatch, or proceed to a single-implementer TDD pass in a worktree off `origin/master`, followed by the mandatory whole-diff adversarial execution review.
