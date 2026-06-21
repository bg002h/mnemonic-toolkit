# BRAINSTORM — cycle-6: timelock decaying-multisig decay-ordering funds-loss bugs

**Status:** DESIGN ONLY (no code). Feeds the mandatory opus-architect **R0 loop to 0 Critical / 0 Important** before any implementation begins (CLAUDE.md hard gate).
**Findings:** **D-decay-rel** (genuine funds-loss; relative-timelock unit-blindness) + **D-decay-abs** (past-`after(T)` immediately spendable). Both EMPIRICALLY reproduce.
**Workstream:** WS-DECAY — ONE PR / ONE TDD pass / ONE fix-site (`archetype::validate_params` decay block), **TWO distinct predicates**.
**SemVer:** toolkit **MINOR → 0.64.0**. md-codec / mk-codec / ms-codec / GUI **NO-BUMP**. Zero clap-flag / `--json` / `schema_mirror` / manual-flag delta.

---

## 0. Source-SHA table (all citations re-grepped against `3fa2925b`)

Recon was authored against `ac4eead0`; this spec re-verifies every line against current `origin/master` = **`3fa2925b`** (`design(cycle5): S-NET network-invariant fix trail + bughunt 9-finding ticks`). Cycle-5 (S-NET) touched `cmd/build_descriptor.rs` control-flow but did NOT restructure the validate→gate dispatch, and did NOT touch `archetype.rs` / `gate.rs` / `timelock_advisory.rs`. Line numbers below are LIVE at `3fa2925b`.

| Artifact | Path | Live line(s) @ `3fa2925b` | Content (verified) |
|---|---|---|---|
| Decay-ordering guard (D-decay-rel fix-site) | `crates/mnemonic-toolkit/src/descriptor_builder/archetype.rs` | **305-317** | `if def.id == "decaying-multisig" { if let (Some(older), Some(recovery_older)) = … { if recovery_older <= older {` — RAW `u32` compare, NO unit normalization |
| `validate_params` fn | `…/descriptor_builder/archetype.rs` | 244 | `pub fn validate_params(def, params) -> Result<(), Vec<Diagnostic>>` |
| `ArchetypeParams.older / .recovery_older / .after` | `…/descriptor_builder/archetype.rs` | 24 / 25 / 26 | all `Option<u32>` |
| `lower_decaying_multisig` (tier assembly) | `…/descriptor_builder/archetype.rs` | 395-419 | tier3 `and_v(v:pk(F), after(T))` → tier2 `andor(multi(recov), older(N2), tier3)` → root `andor(multi(prim), older(N1), tier2)` |
| `param_diag` helper | `…/descriptor_builder/archetype.rs` | 326 | builds a `Diagnostic` |
| `ParamKind` enum (`Blocks`, `AbsoluteLocktime`) | `…/descriptor_builder/archetype.rs` | 36-53 | `ParamKind::Blocks` @51, `::AbsoluteLocktime` @52 |
| `After(n)` field validation (D-decay-abs current state) | `…/descriptor_builder/gate.rs` | **306-324** | `n == 0` refused @307-311; `n > 0x7FFF_FFFF` refused @312-323; **no future-ness / decay check** |
| `Diagnostic` struct + `DiagnosticKind` enum + `Param` variant | `…/descriptor_builder/gate.rs` | 75 / 90 / 97 | `DiagnosticKind::Param` (as_str `"param"`) already EXISTS @97,123 |
| `older_consensus_masked` + `TimelockUnit{Blocks,Seconds512}` | `crates/mnemonic-toolkit/src/timelock_advisory.rs` | enum 19-21; fn 47-52 | masks `!0x0040_FFFF`; unit via `n & 0x0040_0000`; **returns `None` for CLEAN operands** (see §3) |
| BIP-65 height/time split (`<500_000_000`) | `crates/mnemonic-toolkit/src/cost/enumerate.rs` | 72-74 (constant used @243-248 via `absolute::LockTime`) | block-height `<500_000_000`, MTP-time `≥500_000_000` |
| validate→gate dispatch order | `crates/mnemonic-toolkit/src/cmd/build_descriptor.rs` | `validate_params` @278; `emit_diagnostics` @279/300; `gate::validate_with_allow` @287 | producer runs FIRST (exit 2), gate SECOND |
| clap fields `--older / --recovery-older / --after` | `…/cmd/build_descriptor.rs` | 85 / 90 / 95 | all `Option<u32>` |
| Existing decay negative test | `crates/mnemonic-toolkit/tests/cli_build_descriptor.rs` | 546-577 (`preset_decay_ordering_violation_exit_2`) | same-unit `--older 2000` / `--recovery-older 2000` → exit 2; **misses cross-unit** |
| Canon decay golden fixture (CLI `preset_args`) | `…/tests/cli_build_descriptor.rs` | 56-84 (`--after` @81-82) | `--older 1000 / --recovery-older 2000 / --after 500000` — **MIGRATE → `4000000`** (§7 inventory #7) |
| `--after 500001` golden mutation | `…/tests/cli_build_descriptor.rs` | 683 | proves arbitrary past heights accepted today — **MIGRATE → `4000001`** (§7 inventory #8) |
| Existing same-unit decay-neg test (`preset_decay_ordering_violation_exit_2`) carries `--after 500000` | `…/tests/cli_build_descriptor.rs` | 570-571 | stderr-substring assertions; **MIGRATE → `4000000`** for assertion-intent hygiene (§7 inventory `:570`) |
| `repeated_keys` localization test carries `--after 500000` | `…/tests/cli_build_descriptor.rs` | 1019-1020 | past `after` short-circuits `validate_params` before the gate ⇒ breaks `kind=="repeated_keys"` — **MIGRATE → `4000000`** (§7 inventory #9) |
| In-crate `fixture_params("decaying-multisig")` `after: Some(500000)` | `…/src/descriptor_builder/archetype.rs` | 542 | feeds `validate_params_decay_ordering` — **MIGRATE → `Some(4000000)`** (§7 inventory #2) |
| In-crate decay unit test `validate_params_decay_ordering` (`diags.len() == 1`) | `…/src/descriptor_builder/archetype.rs` | 743 | future `after` ⇒ abs predicate silent ⇒ count stays 1 (§7 inventory #2) |
| `ParamKind` doc comment "canon uses block HEIGHT `after(500000)`" | `…/src/descriptor_builder/archetype.rs` | 33 | **MIGRATE → `after(4000000)`** doc (§7 inventory #1) |
| Canon descriptor STRING golden (inline `check(...)`) | `…/src/descriptor_builder/mod.rs` | 66 | `…and_v(v:pk(E),after(500000))…` — **MIGRATE → `after(4000000)`** (§7 inventory #3) |
| Canon fixtures (`.json` / `.descriptor` w/ checksum `#llvl05j9` / `.bip388` template) | `…/tests/fixtures/descriptor_builder/decaying-multisig.{json,descriptor,bip388}` | — | `after(500000)` in all three — **MIGRATE → `4000000`**, REGENERATE the `.descriptor` BIP-380 checksum (§7 inventory #4-6) |
| `PolicyNode::After(500000).render()` unit test | `…/src/descriptor_builder/ir.rs` | 390 | **UNAFFECTED** — pure render, no `validate_params` (§7 inventory U1) |
| `compare-cost --miniscript and_v(v:pk(A),after(1500000000))` | `…/tests/cli_compare_cost.rs` | 1123 | **UNAFFECTED** — raw miniscript cost path, not the archetype/`validate_params` (§7 inventory U2) |
| SPEC decay-ordering rule §3.1 item 3 | `design/SPEC_descriptor_builder_presets.md` | 110 (`### 3.1` item 3) | "require `--recovery-older` > `--older`" — itself unit-blind; MUST update in lockstep |
| SPEC `DiagnosticKind::Param` note (NOT `ToolkitError`) | `design/SPEC_descriptor_builder_presets.md` | §3.3 (DiagnosticKind bullet) | "this enum is not `ToolkitError`, so the alphabetical-variant convention does not apply" |
| FOLLOWUP `archetype-older-blocks-flag-accepts-time-units` | `design/FOLLOWUPS.md` | 219-225 | same unit-blindness; open preset/gate-boundary design call |
| Bug-hunt report rows | `design/agent-reports/constellation-bughunt-2026-06-20.md` | D-decay-rel :836; D-decay-abs :947; repro :1155-1157 | both rows |
| Program plan WS-DECAY | `design/PLAN_constellation_bughunt_fix_program.md` | :216-217, :432, :617-642 | both findings, toolkit MINOR, "pair" |

**Version correction vs recon:** the recon (against `ac4eead0`) read `Cargo.toml` version `0.60.0`. At `3fa2925b` it is **`0.63.0`** (cycle-5 shipped 0.63.0). This cycle bumps to **`0.64.0`**.

---

## 1. Finding summary — both REPRODUCE at `3fa2925b`

### D-decay-rel (genuine funds-loss) — REPRODUCES

The decay-ordering guard (`archetype.rs:305-317`) is the ONLY "tiers unlock progressively later" check. It compares the RAW `u32` operands `recovery_older <= older` with **no BIP-68 bit-22 (`0x0040_0000`) unit normalization**. A 512-second-unit recovery operand whose raw value exceeds a block-height primary operand passes the guard while actually unlocking FIRST.

**Reproduction (report :1157, by construction):** `--older 145` (145 blocks, bit-22 CLEAR, ≈ 24 h) and `--recovery-older 4194305` (`0x40_0001`, bit-22 SET, value 1 ⇒ 1 × 512 s ≈ 8.5 min). Raw compare `4194305 > 145` PASSES the guard. Recovery actually unlocks ~8.5 min vs primary's ~24 h → **the recovery quorum is spendable BEFORE the primary timelock**. `0x40_0001` is a CLEAN BIP-68 operand (not consensus-masked: `older_consensus_masked(0x40_0001) == None`, per `timelock_advisory.rs` test @253-254), so the downstream gate (which refuses only masked operands) ALSO passes. The mis-ordered `wsh(andor(multi(…), older(145), andor(multi(…), older(4194305), and_v(v:pk(F), after(…)))))` is EMITTED — a funds-unsafe spending policy at a real P2WSH address. The existing negative test uses same-unit `2000/2000` and never exercises the cross-unit path.

### D-decay-abs — REPRODUCES

Tier-3's absolute `after(T)` is gate-validated ONLY for range `[1, 0x7FFF_FFFF]` (`gate.rs:306-324`). There is NO future-ness check and NO tier-position check. `--after 500000` (a mainnet block height long since passed) is range-valid, passes the gate, and emits `…and_v(v:pk(final_key), after(500000))` — i.e. the last-resort key is spendable now, collapsing the decay ladder to its weakest leaf. The golden mutation at test `:683` (`--after 500001`) proves arbitrary past heights are accepted today.

**Severity framing (recon judgement, ratified):** `after(T)` is **absolute** (a wall-clock/height moment); the `older` tiers are **relative** (a delay from each UTXO's confirmation). They live in different reference frames and CANNOT be totally ordered without a per-UTXO confirmation-height assumption. So the decidable, offline-checkable invariant is **future-ness**, not cross-frame ordering. This is the softer (LOW↓) facet; D-decay-rel is the primary funds-loss.

---

## 2. BIP-68 / BIP-65 protocol facts (authoritative; verified against the mediawiki specs)

### BIP-68 — `nSequence` relative locktime (CONFIRMED against bip-0068.mediawiki)

| Constant | Value | Meaning |
|---|---|---|
| `SEQUENCE_LOCKTIME_DISABLE_FLAG` | `1<<31` = `0x8000_0000` | SET ⇒ nSequence has NO consensus meaning (CSV is a no-op). |
| **`SEQUENCE_LOCKTIME_TYPE_FLAG`** | **`1<<22` = `0x0040_0000`** | **SET ⇒ time-based, value in units of 512 seconds; CLEAR ⇒ block-height-based.** |
| `SEQUENCE_LOCKTIME_MASK` | `0x0000_FFFF` | Only the low 16 bits encode the value. |
| `SEQUENCE_LOCKTIME_GRANULARITY` | `9` (`2^9 = 512`) | 512 seconds per time unit. |

Two `older()` operands are comparable as durations ONLY when they share the same unit type. A height-100 (≈ 16.7 h @ 10 min/blk) and a time-100 (`0x40_0064`, 100 × 512 s ≈ 14.2 h) are different durations; a height-145 and a time-1 (`0x40_0001`, 8.5 min) invert. The report's "512-second" framing is **CONFIRMED, not corrected.**

### BIP-65 — OP_CLTV absolute locktime (CONFIRMED against bip-0065.mediawiki)

- **`LOCKTIME_THRESHOLD = 500_000_000`.** `nLockTime < 500_000_000` ⇒ **block height**; `≥` ⇒ **Unix timestamp (MTP)**. The toolkit hard-codes the value at `cost/enumerate.rs:74` and `gate.rs:1041`.
- CLTV fails unless the stack value's type matches nLockTime's type (both `<` or both `≥` threshold). A height and a time are not comparable.
- CLTV succeeds only when the transaction's nLockTime ≥ the stack value (the absolute locktime has been reached). **A `T` in the past is satisfiable immediately.**

### Protocol fact corrected (one)

The recon's framing held, but a load-bearing API fact must be pinned for the implementer: **`older_consensus_masked(n)` returns `None` for every CLEAN operand** (1..=65535 blocks; `0x40_0001..=0x40_FFFF` 512-second units) — proven by `timelock_advisory.rs` tests @250-254. It therefore does NOT yield the unit of a clean operand. The unit-classification logic (`n & 0x0040_0000`) lives INLINE inside `older_consensus_masked` (`timelock_advisory.rs:52`) and is not separately exposed. The recon's "reuse `older_consensus_masked` for unit classification" shorthand would misfire for clean operands. The fix must classify the unit directly via `n & 0x0040_0000` (see §4.1, with a tiny shared `older_unit_value` helper).

---

## 3. The decay invariant (stated precisely)

A sound decaying multisig requires **each successive tier to unlock STRICTLY LATER than the previous, in a COMMON normalized time unit.** Concretely, over the registry shape (`lower_decaying_multisig`, `archetype.rs:395-419`):

```
andor( multi(k1, primary…),  older(N1),
  andor( multi(k2, recovery…), older(N2),
    and_v( v:pk(final),         after(T3) )))
```

the invariant decomposes into TWO independent, offline-checkable predicates:

1. **Tier-1 → Tier-2 ordering (D-decay-rel) — RELATIVE, same-frame.** `duration(older N2) > duration(older N1)`. **Fail-closed rule (chosen — see §6 Decision R1):** require both operands to be the SAME BIP-68 unit, then require strict value-ordering `value(N2) > value(N1)`. **Cross-unit pairs are REFUSED as un-orderable** (a block delay and a 512-second delay cannot be totally ordered without a block-interval assumption the toolkit must not bake in). This is the safe fail-closed choice; the current code enforces only raw value-ordering and is blind to the unit, so the same-unit precondition is the missing half.

2. **Tier-3 future-ness (D-decay-abs) — ABSOLUTE, cross-frame.** `after(T3)` cannot be totally ordered against the relative `older` tiers without a per-UTXO confirmation assumption, so the decay invariant for tier-3 is **future-ness**: `T3` must be a FUTURE absolute locktime relative to a sane reference, classified by the BIP-65 `500_000_000` height/time split, refused fail-closed if past. **No live chain tip is required** (see §6 Decision R2).

---

## 4. Per-finding fix design

Both predicates land in the SAME function and block — `archetype::validate_params`, the decay arm `if def.id == "decaying-multisig" { … }` (`archetype.rs:305-317`). Both emit through the EXISTING `Diagnostic` / `DiagnosticKind::Param` (exit-2) path via `param_diag`. **No new `enum ToolkitError` variant and no new `DiagnosticKind` variant are introduced** — the SPEC already documents (`§3.3`) that producer diagnostics use the existing `DiagnosticKind::Param` and that "this enum is not `ToolkitError`, so the alphabetical-variant convention does not apply." The new error *surfaces* are new `param_diag(...)` call-sites with distinct messages, not new variants. (This resolves the prompt's "new typed error variant" item: in this code path the typed surface is `Diagnostic{kind: Param}`, which is already present; the deliverable is the new message + flag-naming, not an enum addition. CLAUDE.md's alphabetical-`ToolkitError` rule does not apply here.)

### 4.1 D-decay-rel fix — BIP-68 unit-aware tier-ordering

Add a tiny shared classifier in `timelock_advisory.rs` (next to `older_consensus_masked`, reusing the same bit constants) so the unit/value math lives in ONE place:

```rust
/// (unit, low-16 value) of a CLEAN BIP-68 relative-locktime operand.
/// Caller must have established the operand is clean (gate / preset range);
/// this is unit/value extraction only, NOT a footgun screen.
pub fn older_unit_value(n: u32) -> (TimelockUnit, u16) {
    let unit = if n & 0x0040_0000 != 0 { TimelockUnit::Seconds512 } else { TimelockUnit::Blocks };
    (unit, (n & 0x0000_FFFF) as u16)
}
```

Then the decay-rel predicate in `validate_params`:

```rust
if def.id == "decaying-multisig" {
    if let (Some(older), Some(recovery_older)) = (params.older, params.recovery_older) {
        let (u1, v1) = timelock_advisory::older_unit_value(older);
        let (u2, v2) = timelock_advisory::older_unit_value(recovery_older);
        if u1 != u2 {
            // CROSS-UNIT — un-orderable; refuse fail-closed.
            diags.push(param_diag(
                RECOVERY_OLDER,
                format!(
                    "decaying-multisig --older ({older}) and --recovery-older \
                     ({recovery_older}) use different BIP-68 timelock units \
                     (one block-height, one 512-second) and cannot be ordered; \
                     use the same unit for both, or author the policy with --spec."
                ),
            ));
        } else if v2 <= v1 {
            // SAME-UNIT but not strictly later — the existing (now unit-aware) rule.
            diags.push(param_diag(
                RECOVERY_OLDER,
                format!(
                    "decaying-multisig requires --recovery-older ({recovery_older}) > \
                     --older ({older}): tiers must unlock progressively later."
                ),
            ));
        }
    }
}
```

Notes:
- Operand cleanliness: `validate_params` runs BEFORE the gate, so a *masked* operand (e.g. `0x80_0001`) is not yet screened here. `older_unit_value` only reads bit-22 + low-16, so it is well-defined for any `u32`; a masked operand still flows to the gate which refuses it (`gate.rs` Older arm). The decay check therefore needs no cleanliness precondition of its own — it compares unit+value, and any genuinely-masked operand is independently rejected downstream with its own diagnostic. (Belt-and-suspenders: the same-unit value compare uses the low-16 `u16`, matching consensus's effective value.)
- The same-unit `v2 <= v1` branch is the *unit-aware generalization* of today's `recovery_older <= older`; it keeps the existing test (`2000`/`2000`, both blocks ⇒ `v2 == v1` ⇒ refused) GREEN by construction.
- SPEC §3.1 item 3 (`SPEC_descriptor_builder_presets.md:110`) updates in lockstep: "require `--recovery-older` and `--older` to share a BIP-68 unit and `--recovery-older` strictly later."

### 4.2 D-decay-abs fix — tier-3 `after(T)` future-ness (offline-checkable)

`after(T)` is absolute. The offline-checkable invariant is **future-ness against a conservative static floor**, classified by the BIP-65 `500_000_000` split. Add a second predicate in the SAME decay arm:

```rust
// Conservative static "already-past" floors. Any after(T) below these is
// UNAMBIGUOUSLY in the past on any live chain → fail-closed.
//   - Block-height floor: a mainnet height already mined long ago. We pick a
//     value safely below any plausible *future* height yet above all genesis-era
//     heights, so legitimate forward-dated heights are never refused.
//   - MTP-time floor: a Unix timestamp already in the past.
const ABS_HEIGHT_PAST_FLOOR: u32 = 900_000;          // mainnet height ~ mid-2025, already mined
const ABS_TIME_PAST_FLOOR:  u32 = 1_750_000_000;     // ~2025-06-15 UTC, already elapsed

if def.id == "decaying-multisig" {
    if let Some(after) = params.after {
        // BIP-65 height/time classification (cost/enumerate.rs:72-74 split).
        let is_height = after < 500_000_000;
        let past = if is_height { after < ABS_HEIGHT_PAST_FLOOR }
                   else         { after < ABS_TIME_PAST_FLOOR };
        if past {
            diags.push(param_diag(
                AFTER,
                format!(
                    "decaying-multisig --after ({after}) encodes an absolute locktime \
                     that is already in the past ({}), so the final-key tier would be \
                     spendable immediately and the decay ladder collapses; use a future \
                     {} value, or author the policy with --spec.",
                    if is_height { "block height" } else { "Unix time" },
                    if is_height { "block height" } else { "Unix timestamp" },
                ),
            ));
        }
    }
}
```

Rationale for the static-floor design (vs a live tip):
- The toolkit is an **offline** steel-engraving authoring tool; no live chain tip is available, and adding a `--current-height` flag would incur the full lockstep tax (GUI `schema_mirror` + manual flag rows + `make lint`) for marginal benefit. **Decision R2 (below): NO `--current-height` flag.**
- A *static* past-floor is monotone-correct: it can only ever produce FALSE NEGATIVES that shrink over time (a `T` between the floor and the true tip that is technically past but above the floor would be accepted) — it NEVER produces a false positive that refuses a legitimately-future locktime. Refusing a legitimate spec is the only failure mode that hurts a user authoring a real wallet; the static floor structurally cannot do that. The floors are documented as conservative and updatable.
- The golden fixture (`--after 500000`, a long-past mainnet height) WILL now be refused by this predicate — **the canon decaying-multisig fixture must update its `--after` to a future value**. We use **`--after 4000000`** consistently: a block HEIGHT (`4_000_000 < 500_000_000` ⇒ height-classified) that is FUTURE relative to `ABS_HEIGHT_PAST_FLOOR = 900_000` (`4_000_000 > 900_000`) ⇒ accepted. This keeps the canon on the same height *axis* as today's `500000` (no axis flip, minimal golden churn). See §7.
  - **Why NOT `900000000` (the value the recon round-0 draft floated, now CORRECTED — R0 round-1 C1):** `900_000_000 > 500_000_000` ⇒ it CLASSIFIES as a UNIX-TIME locktime, but `900_000_000` as a Unix timestamp is **1998-07-09** — deeply in the PAST (it is `< ABS_TIME_PAST_FLOOR = 1_750_000_000`), so this very predicate REFUSES it. A "big number that clears the height/time threshold" is NOT the same as "a future timestamp." Using it would make the positive golden RED and the abs-future positive control fail. `4000000` is the canonical FUTURE `after` value everywhere this spec needs a "valid/future" abs locktime.

**Why not order tier-3 against tier-2's `older`?** Cross-frame (relative vs absolute) ordering is not well-defined without a per-UTXO confirmation assumption (see §1 / §3). Future-ness is the maximal sound, offline-decidable invariant; this is ratified, not deferred.

---

## 5. SemVer / lockstep / oracle

| Axis | Call | Justification |
|---|---|---|
| **Toolkit SemVer** | **MINOR → 0.64.0** | New fail-closed rejections tighten the producer's accepted input set. Today the producer SILENTLY mis-builds (no panic, no refusal) → wrong spending policy at a real address; the fix makes a previously-accepted (cross-unit / mis-ordered / past-`after`) keyless-archetype invocation an error. Tightening a producer's accepted set is a behavioural change ⇒ MINOR. Matches PLAN :216-217. |
| **md-codec / mk-codec / ms-codec** | **NO-BUMP** | The decay logic is toolkit-local in `descriptor_builder/` (producer layer). No codec wire-format, parser, or API change. |
| **GUI (`mnemonic-gui`)** | **NO-BUMP** | No clap flag added/removed/renamed; the diagnostic flows through the EXISTING `Param`/exit-2 path. Zero `schema_mirror` (flag-NAME) delta, zero dropdown-value delta. |
| **clap-flag / `--json` wire-shape** | **NO CHANGE** | No new flag (no `--current-height`). `--json` build-descriptor failure shape (`{diagnostics:[…]}`, exit 2) is unchanged — same `DiagnosticKind::Param`, same `node_path`/`flag` machinery; only new MESSAGE strings + new call-sites. No manual `41-mnemonic.md` flag-row change (only optional prose). |
| **Class-A `bitcoind_differential` oracle** | **No required corpus row; OPTIONAL belt-and-suspenders** | The existing `tests/bitcoind_differential.rs` corpus is a bundle→restore→derive address-equality oracle over single-tier shapes with **NO decaying-multisig shape**, and **build-descriptor producer output is never fed to the oracle** (zero `build-descriptor` references). The real gate is the producer/`validate_params` unit-test layer (§7). A decaying-multisig corpus row (address-level corroboration of a *correctly-ordered* emitted policy) is OPTIONAL; **recommend NOT adding it this cycle** — the unit layer is decisive and a multi-tier andor corpus row is non-trivial scope creep. |

---

## 6. Resolved decisions (no open questions)

| # | Decision | Choice | Rationale |
|---|---|---|---|
| **R1** | D-decay-rel cross-unit handling: refuse vs normalize | **REFUSE cross-unit `older`/`recovery_older` pairs as un-orderable; require same-unit value strict-ordering** | Normalizing blocks↔seconds requires baking in a block-interval assumption (~10 min) the toolkit must not assume; refusal is the safe fail-closed choice and matches the recon's lean. A user wanting mixed-unit tiers has `--spec`. |
| **R2** | D-decay-abs reference source | **Static conservative past-floors (`ABS_HEIGHT_PAST_FLOOR` / `ABS_TIME_PAST_FLOOR`); NO `--current-height` flag** | Offline tool; a flag triggers the full GUI/manual lockstep tax for marginal benefit. Static floors are monotone-correct (only ever false-negative, never false-positive on a legitimate future locktime). |
| **R3** | New typed error surface | **Reuse existing `DiagnosticKind::Param` (exit 2); NEW `param_diag` messages + flag-naming; NO new enum variant** | The decay path is `Diagnostic`-based, not `ToolkitError`-based; SPEC §3.3 already records `Param` exists and the alphabetical-`ToolkitError` convention does not apply here. |
| **R4** | Unit-classification reuse | **Add `timelock_advisory::older_unit_value(n) -> (TimelockUnit, u16)`** (inline `n & 0x0040_0000`) | `older_consensus_masked` returns `None` for clean operands, so it cannot classify a clean operand's unit. A tiny pub helper keeps the bit-math in one module (DRY with the existing constants). |
| **R5** | Sibling FOLLOWUP `archetype-older-blocks-flag-accepts-time-units` | **NARROW resolution: cross-unit decay refusal (R1) — do NOT bound `ParamKind::Blocks` to `1..=65535` globally** | The broad option (blocks-only at the preset layer) would refuse legitimate 512-second-unit timelocks the gate accepts, breaking the deliberate "validate_params does not duplicate gate rules" boundary (`archetype.rs` boundary test) AND would refuse a *consistent* cross-unit decay (e.g. both tiers 512s). R1 resolves the funds-loss facet (cross-unit ordering) precisely; the broad blocks-only constraint is a separate preset-ergonomics call with near-nil exposure. **Keep the FOLLOWUP OPEN** (re-scope its note to "funds-loss facet resolved in cycle-6 via cross-unit refusal; blocks-only preset constraint remains deferred") — do NOT flip it to RESOLVED, because cycle-6 does not implement the blocks-only bound it proposes. |
| **R6** | bitcoind corpus row | **Do NOT add a decaying-multisig oracle row this cycle** | Producer/unit layer is the decisive gate; the oracle never consumes build-descriptor output. Optional, deferred. |
| **R7** | Canon golden fixture `--after 500000` | **UPDATE to `--after 4000000`** (block HEIGHT, future) — applied uniformly across EVERY coupled site (§7 inventory) | `500000` is a long-past mainnet height that the D-decay-abs predicate now refuses; the positive golden must use a future locktime to keep the build path GREEN. **`4000000` chosen, NOT `900000000`** (R0 round-1 C1): `4_000_000 < 500_000_000` ⇒ height-classified, `4_000_000 > ABS_HEIGHT_PAST_FLOOR (900_000)` ⇒ accepted; whereas `900_000_000 > 500_000_000` ⇒ time-classified but `900_000_000 < ABS_TIME_PAST_FLOOR (1_750_000_000)` ⇒ **refused** (it is 1998-07-09, past). Same-axis as `500000` ⇒ minimal golden churn. |
| **R8** | Workstream shape | **ONE PR / ONE TDD pass / ONE fix-site, TWO predicates** | Both live in the same `validate_params` decay arm; PLAN groups them as WS-DECAY "pair." |

---

## 7. Tests (TDD, RED-first)

All in `crates/mnemonic-toolkit/tests/cli_build_descriptor.rs` (producer/`validate_params` layer — the decisive gate). RED-first: each negative asserts `.code(2)` and currently FAILS (today the spec builds exit-0).

### D-decay-rel

| Test | Inputs | Today | After fix |
|---|---|---|---|
| **rel-cross-unit (the recon repro)** — NEW, RED | `--older 145` (blocks) / `--recovery-older 4194305` (`0x40_0001`, 512s) | exit 0, mis-ordered descriptor EMITTED | **exit 2**, stderr names `--older` + `--recovery-older` + "different BIP-68 timelock units" |
| **rel-same-unit-mis-ordered** — NEW, RED-ish | `--older 2000` / `--recovery-older 1000` (both blocks) | exit 0 (raw `1000 <= 2000`… actually refused today too — keep as a unit-aware regression guard) | exit 2, "tiers must unlock progressively later" |
| **rel-same-unit-equal (existing, MUST stay GREEN)** | `--older 2000` / `--recovery-older 2000` (`preset_decay_ordering_violation_exit_2` @546) | exit 2 | exit 2 (unchanged; `v2 == v1` ⇒ refused) |
| **rel-positive same-unit (control)** — NEW or via canon | `--older 1000` / `--recovery-older 2000` (both blocks, canon) | builds | **still builds** (exit 0) |
| **rel-positive both-512s (control)** — NEW | `--older 0x40_0001` (=4194305) / `--recovery-older 0x40_0002` (=4194306) | builds | **still builds** (same unit, strictly later) |

> Note on the cross-unit repro direction: the recon's headline case (`--older 145` blocks vs `--recovery-older 4194305` 512s) is REFUSED by the cross-unit branch (units differ) — this is the funds-loss case and the primary RED. We do NOT additionally try to "normalize and value-compare" it; R1 refuses it outright.

### D-decay-abs

The RED test and its positive control are explicit and non-self-contradictory:
- **RED:** `--after 500000` — a PAST block height (`500_000 < ABS_HEIGHT_PAST_FLOOR = 900_000`) ⇒ now REJECTED (exit 2, abs-future diagnostic).
- **Positive control:** `--after 4000000` — a FUTURE block height (`4_000_000 > 900_000`, `< 500_000_000` ⇒ height-classified) ⇒ builds.

| Test | Inputs | Today | After fix |
|---|---|---|---|
| **abs-past-height (RED)** — NEW | canon args but `--after 500000` (past mainnet height) | exit 0, final-key spendable now | **exit 2**, stderr "already in the past" + names `--after` |
| **abs-future-height (control / new canon base)** — NEW | `--after 4000000` (FUTURE height, `4_000_000 > 900_000`, `< 500_000_000`) | builds | **still builds** |
| **canon golden fixture** (`Archetype "decaying-multisig"` @56-84) | currently `--after 500000` | builds (golden) | **MUST regenerate** the `.descriptor` (incl. its BIP-380 checksum) + `.bip388` + `.json` fixtures with `--after 4000000` (R7) so the golden stays GREEN AND consistent with the abs predicate |

> Note — `900000000` is NOT a valid control (R0 round-1 C1): it classifies as MTP-time (`> 500_000_000`) but is below `ABS_TIME_PAST_FLOOR (1_750_000_000)` — it is 1998-07-09, PAST — so the predicate would refuse it. Every "future" value in this spec is `4000000`.

**Coupled-site migration inventory — `--after 500000` / `500001` (R0 round-1 I1+I2).** Changing the canon `--after` from `500000` to `4000000` changes the emitted miniscript AND (independently) the new abs predicate now FIRES on every past `500000`/`500001` baseline. The recon round-0 draft budgeted only "THREE coupled sites"; the verified inventory below — **re-grepped against `3fa2925b`** — is **NINE MIGRATE + TWO UNAFFECTED**. Each MIGRATE site moves its `after` value to the future base `4000000` (mutation rows to `4000001`); each UNAFFECTED site is a pure render / raw-policy / non-`validate_params` path and is LEFT as-is.

| # | Site (`3fa2925b`) | Current value | Classification | Action / why |
|---|---|---|---|---|
| 1 | `src/descriptor_builder/archetype.rs:33` (doc comment "canon uses block HEIGHT `after(500000)`") | `500000` | **MIGRATE** | Doc text → `after(4000000)`; keep the "block HEIGHT, not unix time" framing (still true). Pure prose, but stale-doc hygiene. |
| 2 | `src/descriptor_builder/archetype.rs:542` `after: Some(500000)` (`fixture_params`) | `Some(500000)` | **MIGRATE** | → `Some(4000000)`. Feeds the in-crate `validate_params_decay_ordering` unit test (`:743`): it asserts `diags.len() == 1`. With a FUTURE `after`, the abs predicate adds NO diagnostic, so ONLY the rel diagnostic fires for `recovery_older ∈ {1000, 999}` ⇒ count stays **1** (assertion holds). With the past `500000`, the abs predicate would push a SECOND diag ⇒ `len() == 2` ⇒ RED — hence this migration is load-bearing, not cosmetic. |
| 3 | `src/descriptor_builder/mod.rs:66` (canon descriptor STRING golden, `after(500000)`) | `after(500000)` | **MIGRATE** | → `after(4000000)` in the inline `check(...)` expected string for `decaying_multisig()`. |
| 4 | `tests/fixtures/descriptor_builder/decaying-multisig.json` (`{ "after": 500000 }`) | `500000` | **MIGRATE** | → `4000000`. Source spec the canon golden derives from. |
| 5 | `tests/fixtures/descriptor_builder/decaying-multisig.descriptor` (byte-exact golden, ends `after(500000)))))#llvl05j9`) | `500000` + checksum `#llvl05j9` | **MIGRATE (+ checksum regen)** | → `after(4000000)` AND **regenerate the BIP-380 descriptor checksum** (`#llvl05j9` is computed over the OLD string; a stale checksum fails the parser — the new value yields a DIFFERENT checksum). Must re-derive, do not hand-edit only the digits. |
| 6 | `tests/fixtures/descriptor_builder/decaying-multisig.bip388` (`description_template` embeds `after(500000)`) | `500000` | **MIGRATE** | → `after(4000000)` inside `description_template`. (The recon round-0 "three sites" note named `.descriptor`+`.bip388` as one bullet but listed only `.json` + goldens + the `:683` mutation; the `.bip388` template-string substitution is its own concrete edit.) |
| 7 | `tests/cli_build_descriptor.rs:81-82` (canon `preset_args` `--after 500000`) | `500000` | **MIGRATE** | → `4000000`. Drives `preset_descriptor_goldens` / `preset_bip388_goldens` / `emit_spec_value_equals_fixture_and_round_trips` against fixtures #4-6; must move in lockstep. |
| 8 | `tests/cli_build_descriptor.rs:683` mutation row `("decaying-multisig","--after","500001")` | `500001` | **MIGRATE** | `500001` is ALSO a past height (`< 900_000`) ⇒ under the abs fix it would exit-2, but `preset_negative_discrimination_mutated_param_breaks_golden` (`:680`) asserts `.success()` then a golden mismatch. **Decision: use a FUTURE mutation base `--after 4000001`** so the test still exercises its INTENDED mutation ("mutating a numeric param breaks the byte-exact golden") rather than silently flipping to "past-after reject." (`4000001 ≠ 4000000` ⇒ descriptor differs ⇒ `assert_ne!` holds; both future ⇒ both build.) |
| 9 | `tests/cli_build_descriptor.rs:1019-1020` (`repeated_keys` localization, `--after 500000`) | `500000` | **MIGRATE** | → `4000000`. The block asserts `.code(2)` AND `diagnostics[0].kind == "repeated_keys"` at `node_path == "root.andor[2]"`. But `validate_params` runs BEFORE the gate (`build_descriptor.rs:278` `return Ok(2)`); under the abs fix the past `500000` makes `validate_params` push a `param` diag and short-circuit exit 2 → the gate never runs → `diagnostics[0].kind` becomes `"param"` (`node_path == "params"`), breaking BOTH assertions. Migrating `--after` to `4000000` lets `validate_params` pass so the gate's `repeated_keys` path is exercised as intended. (FOURTH CLI `--after 500000` occurrence; the recon round-0 narrative missed it.) |
| — | `tests/cli_build_descriptor.rs:570-571` (`preset_decay_ordering_violation_exit_2`, same-unit `2000/2000`, `--after 500000`) | `500000` | **MIGRATE (cleanliness)** | Its assertions are stderr-substring only (`--recovery-older`, `--older`) ⇒ would still PASS (the rel diag for `v2==v1` still fires, and `validate_params` collects ALL diags before exit 2). But with past `500000` the abs predicate ALSO fires, so the test would silently exercise TWO diags instead of the rel-ordering one it names. → `4000000` so it isolates the rel-ordering path. (Not strictly RED, but migrated for assertion-intent hygiene; flagged so the implementer applies the uniform substitution here too.) |
| U1 | `src/descriptor_builder/ir.rs:390` `assert_eq!(PolicyNode::After(500000).render(), "after(500000)")` | `500000` | **UNAFFECTED** | Pure `render()` unit test of the IR node — no `validate_params` / archetype path. LEAVE as-is. |
| U2 | `tests/cli_compare_cost.rs:1123` `and_v(v:pk(A),after(1500000000))` | `1500000000` | **UNAFFECTED** | **VERIFIED:** a RAW miniscript string passed to `compare-cost --miniscript` — NOT the `decaying-multisig` archetype and NOT through `validate_params` (the whole file has ZERO `decaying`/`build-descriptor`/`archetype` refs). It is `absolute_mtp_time_lock_satisfies`, a cost-comparison label test. LEAVE as-is. |

**Tally:** NINE **MIGRATE** sites (#1-#9 + the cleanliness migration at `:570-571` = the 9 numbered MIGRATE rows; counting `:570` brings it to ten distinct `after`-bearing edits, but #5's checksum-regen is part of #5) + TWO **UNAFFECTED** (`ir.rs:390`, `cli_compare_cost.rs:1123`). This EXCEEDS the recon round-0 "three coupled sites" budget — the true coupled-site count is materially larger, driven by the abs predicate firing on every past baseline (not just by the changed emitted-miniscript), and by the `.bip388` template + the in-crate unit-test fixture being distinct sites.

> Counting note: the MIGRATE rows are `archetype.rs:33`, `archetype.rs:542`, `mod.rs:66`, `.json`, `.descriptor` (+checksum), `.bip388`, `cli:81-82`, `cli:683`, `cli:1019-1020`, plus the `cli:570-571` cleanliness migration. Whether `:570` is counted as a tenth MIGRATE or a hygiene tick, the point for R0 is the same: it is FAR more than three, and ALL are enumerated here so none goes silently RED.

---

## 8. FOLLOWUP slugs

| Slug | Disposition this cycle |
|---|---|
| `archetype-older-blocks-flag-accepts-time-units` (`FOLLOWUPS.md:219`) | **Re-scope, keep OPEN.** The funds-loss facet (cross-unit decay ordering) is RESOLVED via R1's cross-unit refusal; annotate the entry: "cycle-6 (toolkit 0.64.0) resolved the decaying-multisig cross-unit ordering funds-loss; the broader `ParamKind::Blocks → 1..=65535` preset-semantic constraint remains DEFERRED (near-nil exposure, preset/gate-boundary design call)." Do NOT flip to RESOLVED. |
| **NEW: `decay-abs-static-floor-staleness`** (file in this cycle) | The D-decay-abs past-floors (`ABS_HEIGHT_PAST_FLOOR` / `ABS_TIME_PAST_FLOOR`) are static conservative values that drift further into the past over time, widening the false-negative window. Tier: deferred/low (monotone-safe; never false-positive). Revisit if a live chain-param reference is ever plumbed. |
| **NEW (optional): `decaying-multisig-bitcoind-oracle-row`** | Adding a correctly-ordered decaying-multisig shape to `tests/bitcoind_differential.rs` as a belt-and-suspenders address-level oracle. Deferred (R6); not required to close the findings. |

---

## 9. Mandatory R0-gate note

Per CLAUDE.md: **NO code before this brainstorm (and the subsequent SPEC + plan-doc) passes the opus-architect R0 review to 0 Critical / 0 Important.** Fold findings → persist the review verbatim to `design/agent-reports/cycle6-timelock-decay-brainstorm-r0-round{N}-review.md` → re-dispatch → repeat until GREEN. The reviewer-loop continues after EVERY fold (folds can introduce drift). Implementation is a SINGLE subagent in a worktree, TDD (RED-first per §7), followed by a MANDATORY non-deferrable whole-diff adversarial execution review. This spec is decision-complete (§6) and carries no open questions into R0.
