# Cycle-prep STRICT-GATE recon — cycle-6: timelock decaying-multisig decay-ordering bugs

**Findings:** D-decay-rel (MED, genuine funds-loss/timelock) + D-decay-abs (LOW↓, timelock).
**Mode:** recon ONLY (pre-brainstorm). No brainstorm / spec / plan / code in this pass.

- **origin/master SHA:** `ac4eead002bb5bec1096427c4ebb26e0bfc07f6f`
  (`design(cycle4): codec funds-safety fix trail + bughunt H6/M4/M6 ticks`)
- **Bug-hunt report:** `design/agent-reports/constellation-bughunt-2026-06-20.md`
  (D-decay-rel row :836; D-decay-abs row :947; empirical-repro note :1155–1157).
- **Program plan:** `design/PLAN_constellation_bughunt_fix_program.md`
  (WS-DECAY = both findings, :216–217, :432, :617–:642; toolkit MINOR each; "pair").
- **Builder SPEC:** `design/SPEC_descriptor_builder_presets.md`
  (decay-ordering rule §3.1.3 = line 110; preset table line 153; negative-test line 171).
- **Builder version:** `mnemonic-toolkit/Cargo.toml:3` = `0.60.0` (master; cycle-5 in flight).
- **Cycle-5 drift caveat:** cycle-5 (S-NET) edits import/export/convert/build_descriptor +
  `error.rs`; it does **NOT** touch `descriptor_builder/archetype.rs` or `gate.rs`. No `error.rs`
  citation appears in THIS recon (the decay zone surfaces `Diagnostic`, exit-code 2, not a
  `ToolkitError` variant), so there is **nothing to re-verify post-cycle-5 here** — see Cross-cutting.

---

## Per-finding verification

### D-decay-rel — decay-ordering compares RAW BIP-68 operands without normalizing the bit-22 unit flag

**WHAT:** The decaying-multisig producer's only "tiers unlock progressively later" guard compares
raw `u32` `recovery_older` vs `older` (`recovery_older <= older`). It never normalizes the BIP-68
bit-22 (0x00400000) time-vs-block unit flag, so a 512-second-unit recovery operand whose RAW value
exceeds a block-height primary operand passes the guard while actually unlocking FIRST → the recovery
quorum is spendable before the primary timelock. Different operands ⇒ different miniscript ⇒
different P2WSH address (address/policy-affecting).

**Citations (each re-checked against `ac4eead0`):**

| Report claim | Live location | Verdict |
|---|---|---|
| `descriptor_builder/archetype.rs:305-317` (`validate_params`) — raw `older`/`recovery_older` compare, no unit normalization | `archetype.rs:305-317` — `if def.id=="decaying-multisig" { if let (Some(older),Some(recovery_older)) … if recovery_older <= older {` push `param_diag` | **ACCURATE** (exact lines, exact predicate) |
| guard lives in `validate_params`, the producer-level §3.1 check | `archetype.rs:244` `pub fn validate_params`, decay block at :301-317 | **ACCURATE** |
| compares "raw `u32`" Option values | `ArchetypeParams.older: Option<u32>` :24, `.recovery_older: Option<u32>` :25; clap `recovery_older: Option<u32>` `build_descriptor.rs:90` | **ACCURATE** |
| "512-sec unit flag" framing | bit-22 = 0x00400000 = SEQUENCE_LOCKTIME_TYPE_FLAG; SET ⇒ 512-second units (BIP-68, confirmed authoritative below) | **ACCURATE** |
| order of validation lets a CLEAN 512-sec operand through | `validate_params` runs FIRST (`build_descriptor.rs:278`, exit 2 on err) THEN `gate::validate_with_allow` (:287). The gate refuses only *masked* operands; a clean `0x400001` (1×512s) is `None` from `older_consensus_masked` ⇒ passes BOTH. | **ACCURATE — confirms reachability** |
| SPEC encodes the same unit-blind rule | `SPEC_descriptor_builder_presets.md:110` §3.1.3 "require `--recovery-older` > `--older`" — itself unit-blind | **ACCURATE** (SPEC + impl consistent; BOTH need the unit fix) |

**STILL-REPRODUCES: YES (empirically, by construction).** Reproduction (report :1157):
`--older 145` (=145 blocks, bit-22 CLEAR, ≈24 h) and `--recovery-older 4194305`
(=`0x400001`, bit-22 SET, value 1 ⇒ 1×512 s ≈ 8.5 min). Raw compare
`4194305 > 145` PASSES the guard; the recovery tier unlocks ~8.5 min vs the primary's ~24 h →
**recovery quorum spendable BEFORE the primary timelock**. `0x400001` is NOT consensus-masked, so
the gate also passes → the mis-ordered `wsh(andor(multi(…),older(145),andor(multi(…),older(4194305),…)))`
is EMITTED. The existing negative test `cli_build_descriptor.rs:546 preset_decay_ordering_violation_exit_2`
uses raw-equal `2000`/`2000` (same unit) — it does NOT cover the cross-unit case, so the gap is live.

**Fix-site:** `archetype.rs:301-317` (`validate_params` decay block). The fix must compare
*normalized* durations, not raw operands. Reuse vector already in-tree:
`timelock_advisory::TimelockUnit{Blocks,Seconds512}` + `older_consensus_masked`
(`timelock_advisory.rs:19,47`) classify the unit via `n & 0x0040_0000`; and miniscript's
`RelLockTime::is_height_locked()/is_time_locked()` (used at `gate.rs:626-629`). SPEC §3.1.3 (:110)
must update in lockstep with the predicate.

### D-decay-abs — tier-3 absolute `after(T)` never validated against the decay invariant

**WHAT:** The decaying-multisig's tier-3 absolute `after(T)` (block height OR unix time per BIP-65)
is gate-validated only for the range `[1, 0x7FFFFFFF]` — there is NO check that `T` is in the future
or that tier-3 actually unlocks LAST. A past block-height/time makes the last-resort `final_key`
immediately spendable, collapsing the decay ladder.

**Citations (each re-checked against `ac4eead0`):**

| Report claim | Live location | Verdict |
|---|---|---|
| `descriptor_builder/archetype.rs:305-317` — `after` not in the decay-ordering rule | `validate_params` decay block :301-317 checks ONLY `recovery_older`/`older`; `after` is absent from every ordering check (the `supplied`/arity loop :249-299 only counts it) | **ACCURATE** (the absence is the bug) |
| `gate.rs:306-324` — `after` field-validation only ranges, no decay invariant | `gate.rs:306-324` `PolicyNode::After(n)`: refuses `n==0` (:307-311) and `n>0x7FFF_FFFF` (:312-323); no future-ness, no tier-position check | **ACCURATE** (exact lines, exact constants) |
| `after` is BIP-65 absolute, height-vs-time split | `after_tree` test uses `500_000_000` as the height/time boundary (`gate.rs:1041`); `cost/enumerate.rs:72-74` defines `<500_000_000` = block-height, `≥500_000_000` = MTP-time | **ACCURATE** (matches BIP-65 LOCKTIME_THRESHOLD) |
| canon uses a block-HEIGHT `after(500000)` | `archetype.rs:33-34` doc; fixture param `after: Some(500000)` (`:542`) | **ACCURATE** |

**STILL-REPRODUCES: YES (by construction — no future-ness validation exists).** `--after 500000`
(a height long since passed on mainnet) is range-valid (`1 ≤ N ≤ 0x7FFFFFFF`), passes the gate, and
emits `…and_v(v:pk(final_key),after(500000))` — i.e. the last-resort key is spendable now.
The existing test `cli_build_descriptor.rs:683` even mutates `--after` to `500001` for golden
non-vacuity, confirming arbitrary past heights are accepted. No `after`-decay validation anywhere.

**Severity nuance (recon judgement):** "past `after`" is a softer funds-loss than D-decay-rel.
`after(T)` is **absolute** (a wall-clock/height moment) while the `older` tiers are **relative** (a
delay measured from each UTXO's confirmation). They live in **different reference frames**, so a
"strictly later than tier-2's `older`" comparison is NOT well-defined in general (a relative delay
and an absolute moment can't be totally ordered without a confirmation-height assumption). The
**defensible, decidable** invariant the fix CAN enforce is **future-ness**: tier-3 `after(T)` must be
a FUTURE absolute locktime relative to a sane reference (current chain tip / now), refusing a past T
fail-closed. This is the LOW↓ facet; it is real but its scope is "refuse an obviously-past absolute
locktime in the keyless authoring path", not a full cross-frame ordering proof.

**Fix-site:** `archetype.rs` `validate_params` decay block (same function as D-decay-rel) for the
future-ness predicate; the BIP-65 height/time classification (`<500_000_000`) is already in-tree
(`cost/enumerate.rs:72-74`). A "current tip / current time" reference is an authoring-time input the
brainstorm must source (chain param / `--current-height` style, or a conservative static floor).

---

## The decay invariant + BIP-68 / BIP-65 protocol facts (authoritative)

### BIP-68 nSequence relative-locktime bit layout (CONFIRMED against bip-0068.mediawiki)

| Constant | Value | Meaning |
|---|---|---|
| `SEQUENCE_LOCKTIME_DISABLE_FLAG` | `1 << 31` = `0x80000000` | SET ⇒ nSequence has NO consensus meaning (CSV is a no-op). |
| **`SEQUENCE_LOCKTIME_TYPE_FLAG`** | **`1 << 22` = `0x00400000`** | **SET ⇒ time-based, value in units of 512 seconds; CLEAR ⇒ block-height-based.** |
| `SEQUENCE_LOCKTIME_MASK` | `0x0000FFFF` | Only the low 16 bits encode the value. |
| `SEQUENCE_LOCKTIME_GRANULARITY` | `9` (`2^9 = 512`) | Time granularity = 512 seconds per unit. |

**Report's "512-second" framing = CONFIRMED, not corrected.** Bit 22 SET ⇒ time-based in 512-second
units; CLEAR ⇒ block-height. Comparing two `older()` operands' RAW low-16 values without first
checking they share the same unit type is the bug: a height-based 100 (≈16.7 h @10 min/blk) and a
time-based 100 (`0x400064`, 100×512 s ≈ 14.2 h) are *different durations*, and a height-based 100 vs
a time-based 1 (`0x400001`, 8.5 min) invert. The in-tree predicate already mirrors these exact
constants: `older_consensus_masked` masks with `!0x0040_FFFF` and reads the unit via `n & 0x0040_0000`
(`timelock_advisory.rs:48,52`).

### BIP-65 OP_CLTV absolute-locktime semantics (CONFIRMED against bip-0065.mediawiki)

- **`LOCKTIME_THRESHOLD = 500000000`** (5×10⁸). `nLockTime < LOCKTIME_THRESHOLD` ⇒ **block height**;
  `≥` ⇒ **Unix timestamp**. (BIP-65 names the constant; the toolkit hard-codes the value at
  `cost/enumerate.rs:74` and `gate.rs:1041` — verified in-tree.)
- **Apples-to-apples type rule:** CLTV fails the script unless the stack value's type matches the
  transaction's nLockTime type (both `<` threshold or both `≥`). Mixed types ⇒ script failure.
- **Verify rule:** CLTV succeeds only when the transaction's nLockTime ≥ the stack value (the absolute
  locktime has been reached). A `T` in the past is therefore satisfiable immediately.

### The decay invariant the fix must enforce

A sound decaying multisig requires **each successive tier to unlock STRICTLY LATER than the previous,
in a COMMON normalized time unit.** Concretely, for the registry shape
`andor(multi(k1,T1…), older(N1), andor(multi(k2,T2…), older(N2), and_v(v:pk(F), after(T3))))`
(`SPEC §6` / `archetype.rs:395-419 lower_decaying_multisig`):

1. **Tier-1 → Tier-2 (D-decay-rel):** `duration(older N2) > duration(older N1)` where `duration`
   normalizes the BIP-68 unit (blocks → wall-clock via a block-interval assumption, or — the
   simplest fail-closed rule — **require both operands to be the SAME unit type AND value-ordered**:
   refuse cross-unit pairs, then `value(N2) > value(N1)`). The current code enforces only the raw
   value-ordering and is blind to the unit, so the SAME-UNIT precondition is the missing half.
2. **Tier-3 (D-decay-abs):** `after(T3)` is **absolute**, not relative — it cannot be totally ordered
   against the relative `older` tiers without a confirmation-height assumption. The decidable
   invariant is **future-ness**: `T3` must be a future block-height/time (per the BIP-65 threshold
   classification) relative to a sane reference, refused fail-closed if past.

**Where the builder constructs/orders the tiers:** `lower_decaying_multisig`
(`archetype.rs:395-419`) — tier3 (`and_v(v:pk(F), after(T))`) built first, then tier2
(`andor(multi(recovery), older(N2), tier3)`), then root
(`andor(multi(primary), older(N1), tier2)`). The ORDERING guard that must enforce the invariant is in
`validate_params` (`archetype.rs:301-317`), which runs BEFORE lowering and before the gate.

---

## Cross-cutting

**SemVer:** **toolkit MINOR** (matches PLAN :216-217). Both findings add a NEW fail-closed rejection
(exit 2 `Param` diagnostic) for specs that build SILENTLY today — i.e. they tighten the accepted
input set. Today the producer **silently mis-builds** (no panic, no refusal) and emits a
funds-unsafe descriptor; the fix makes a previously-accepted (mis-ordered/cross-unit/past-`after`)
keyless-archetype invocation an error. Tightening a producer's accepted set is a behavioural change
warranting a MINOR. No new public API surface beyond possibly one new `--current-height`-style input
(brainstorm decision) — IF such a flag is added it becomes a clap-flag delta (see lockstep).

**Address/policy-affecting → oracle-gate (Class-A bitcoind_differential):**
A mis-ordered or cross-unit decay spec produces a DIFFERENT miniscript (different `older`/`after`
operands ⇒ different witness script ⇒ different P2WSH address), so the wrong policy *would* be
oracle-detectable in principle. **HOWEVER** — recon caveat — the existing
`tests/bitcoind_differential.rs` corpus (`corpus()` :90-153) is a **bundle→restore→derive**
address-equality oracle over 12 single-tier shapes (`wpkh`, `pkh`, `wsh-(sorted)multi`,
`wsh-timelocked older(144)`, `wsh-thresh`, `tr-nums-*`). It has **NO decaying-multisig / multi-tier
andor shape**, and **build-descriptor producer output is never fed to the oracle** (grep: zero
`build-descriptor` references in the oracle). So the oracle gate does NOT today exercise this fix;
the right gate is the **producer/`validate_params` unit test layer** (`cli_build_descriptor.rs`
preset goldens + the existing `preset_decay_ordering_violation_exit_2` extended for the cross-unit
and past-`after` cases). The brainstorm should decide whether to ADD a decaying-multisig shape to the
bitcoind corpus as a belt-and-suspenders oracle row (optional; address-level corroboration of the
emitted policy), but it is not strictly required to close these findings.

**Clap-flag / `--json` lockstep:** The core fix (a stricter `validate_params` predicate) adds NO clap
flag and NO `--json` wire-shape change → **zero GUI `schema_mirror` impact, zero manual-flag-row
impact**. The diagnostic flows through the EXISTING `Param`/exit-2 path (`emit_diagnostics`,
`build_descriptor.rs:279`). **EXCEPTION:** if the brainstorm chooses to source a "current
height/time" reference for D-decay-abs via a NEW flag (e.g. `--current-height`), THAT triggers the
full lockstep tax — GUI `mnemonic-gui/src/schema/mnemonic.rs` `BUILD_DESCRIPTOR_FLAGS` (flag-name
parity), `docs/manual/src/40-cli-reference/41-mnemonic.md` flag rows + `make lint` coverage gate.
Flag a non-flag design (static conservative floor / chain-param) to avoid this; the brainstorm must
make the call.

**Shared fix vs separate:** **ONE shared fix-site, ONE workstream (WS-DECAY), but TWO distinct
predicates.** Both live in the same `validate_params` decay block (`archetype.rs:301-317`) and the
PLAN groups them as WS-DECAY "pair" (:216-217, :432). They are NOT the same predicate, though:
D-decay-rel = a *relative* tier-ordering check with BIP-68 unit-normalization (refuse cross-unit /
require same-unit value-ordering); D-decay-abs = an *absolute* tier-3 future-ness check with BIP-65
height/time classification. Build them as two checks in one PR / one TDD pass. **Sibling FOLLOWUP:**
`archetype-older-blocks-flag-accepts-time-units` (`FOLLOWUPS.md:220-226`) already documents the SAME
unit-blindness on `--older`/`--recovery-older` as a deferred item, and explicitly flags a **design
call** about the `validate_params`-vs-gate boundary ("the gate legitimately accepts 512s-unit
encodings, so a blocks-only constraint is a preset-semantic, not a gate rule — decide where it
belongs"). D-decay-rel is the funds-loss escalation of that entry; the brainstorm should fold both —
deciding whether the fix is "refuse cross-unit ordering" (narrow) or "bound `ParamKind::Blocks` to
`1..=65535` blocks-only at the preset layer" (the FOLLOWUP's broader proposal, which would resolve
both at once). Flip `archetype-older-blocks-flag-accepts-time-units` status in the shipping commit if
the broader option is taken.

**Post-cycle-5 re-verification:** **NONE required for this recon.** Every cited file is in the decay
zone (`descriptor_builder/archetype.rs`, `gate.rs`, `timelock_advisory.rs`, `cost/enumerate.rs`,
`cmd/build_descriptor.rs`, the two SPEC/PLAN docs) — none touched by cycle-5's
import/export/convert/build_descriptor-control-flow/`error.rs` edits. No `error.rs` line is cited
(the decay path uses `Diagnostic`/exit-2, not a `ToolkitError` variant). The ONLY cycle-5 overlap is
`cmd/build_descriptor.rs` — but the cited lines (:90 the clap field, :278-287 the
validate_params→gate dispatch order) are the preset *dispatch*, which cycle-5 (S-NET, network-string
agreement) does not restructure. **Re-confirm `build_descriptor.rs:278-287` line numbers after
cycle-5 merges** (low risk; structure-stable but a courtesy re-grep).

---

## Recommended brainstorm-session scope

1. **WS-DECAY, one PR / one TDD pass, two predicates in `archetype::validate_params`:**
   - **D-decay-rel:** BIP-68-unit-aware tier-ordering. Decide the rule — recommended fail-closed:
     **refuse cross-unit `older`/`recovery_older` pairs, then require same-unit value-ordering**
     (`recovery_older` strictly later than `older`). Reuse `timelock_advisory::TimelockUnit` /
     `n & 0x0040_0000` for unit classification. Update SPEC §3.1.3 (`:110`) in lockstep.
   - **D-decay-abs:** tier-3 `after(T)` future-ness. Decide the reference source (static conservative
     floor / chain-param — NO new flag preferred to dodge the lockstep tax) and use the BIP-65
     `<500_000_000` height/time split (`cost/enumerate.rs:72-74`). Refuse a past `T` fail-closed.
2. **Fold the sibling FOLLOWUP** `archetype-older-blocks-flag-accepts-time-units` (`FOLLOWUPS.md:220-226`)
   — same unit-blindness, deferred, has an open design call on the preset/gate boundary. Decide narrow
   (cross-unit ordering only) vs broad (`ParamKind::Blocks` → blocks-only `1..=65535`). Flip its
   status in the shipping commit if resolved.
3. **Tests:** extend `cli_build_descriptor.rs:546 preset_decay_ordering_violation_exit_2` for the
   cross-unit reproduction (`--older 145` / `--recovery-older 4194305` → exit 2) + a past-`after`
   case; keep the existing same-unit case. Producer/`validate_params` unit layer is the right gate
   (the bitcoind oracle does NOT exercise build-descriptor output — adding a decaying-multisig corpus
   row is OPTIONAL belt-and-suspenders, a brainstorm call).
4. **Lockstep:** none for the core fix (no clap/`--json` delta). ONLY if a new `--current-height`-style
   flag is chosen → GUI `schema_mirror` + manual flag rows + `make lint`. Recommend avoiding it.
5. **SemVer:** toolkit MINOR. md-codec / mk-codec / ms-codec NO-BUMP (decay logic is toolkit-local
   in `descriptor_builder/`). GUI NO-BUMP unless a flag is added.
6. **Gate discipline:** mandatory R0 on brainstorm + plan (0C/0I) before any code; single-subagent
   TDD; mandatory whole-diff post-impl review. Per CLAUDE.md.
