# SPEC — consensus-masked `older()` intake advisory

**Source SHA:** `3235431` (origin/master == HEAD at write time; sync clean). All file:line
citations below were grep-verified against this tree; re-grep on any later rebase.
**Cycle:** resolves FOLLOWUP `intake-surfaces-accept-masked-older-no-advisory`
(`design/FOLLOWUPS.md:137-148`, surfaced v0.53.9 R0-r1 M5).
**SemVer:** PATCH (new non-blocking stderr advisory lines only; zero clap delta).
**Scope:** toolkit-only. No md-codec / sibling-codec / GUI code changes.
**Predecessor:** `design/SPEC_older_timelock_mask_gate.md` (v0.53.9) — built the *authoring* gate
this cycle extends to *intake* surfaces.
**Pre-R0 direction consult:** architect review folded (this session); verdict YELLOW →
the three findings (C1 bit-31 reachability, C2 walk-layer bifurcation, C3 bundle --import-json)
are resolved in §3–§4 below. The mandatory **formal R0 gate runs on THIS spec** (0C/0I before any code).

---

## §1 Problem

`build-descriptor` (the JSON-IR authoring surface) hard-refuses BIP-68 consensus-masked
`older()` values via the v0.53.9 gate (`crates/mnemonic-toolkit/src/descriptor_builder/gate.rs:257`
Older arm; predicate `gate.rs:264`). Seven *intake / round-trip* surfaces accept the same
masked values **silently** — correct (must never refuse to back up / recover an already-deployed
wallet; that strands real funds) but with **no advisory** that the timelock is weaker than its
literal suggests.

**Verified consensus semantics** (rust-bitcoin `0.32.8`, the pinned consensus-equivalent stack;
`src/blockdata/transaction.rs`):
- A CSV operand is a relative timelock iff bit 31 is clear (`LOCK_TIME_DISABLE_FLAG_MASK =
  0x80000000`, `:354`; `is_relative_lock_time()` `:393`).
- Units: bit 22 (`LOCK_TYPE_MASK = 0x00400000`, `:356`) selects 512-second intervals; else blocks
  (`is_height_locked()` `:399`).
- **Value = the low 16 bits only** (`to_relative_lock_time()` uses `low_u16()` `:481-500`,
  commented *"BIP-68 only uses the low 16 bits for relative lock value"*).

Therefore `older(65536)` (`0x0001_0000`): bit-31 clear (active lock), bit-22 clear (blocks),
value `0x0001_0000 as u16 = 0` → **a 0-block relative lock = trivially satisfied = funds
spendable immediately.** The author's intended ~65 536-block (~455-day) lock is silently
nullified. The gate predicate `(n & !0x0040_FFFF) != 0 || (n & 0x0000_FFFF) == 0` (`gate.rs:264`)
catches both footgun classes: stray bits consensus would mask away, and a zero 16-bit value.

## §2 Decision (user-approved)

**Advisory-only on all seven surfaces.** No intake surface refuses. `build-descriptor` keeps its
existing hard refuse unchanged. The advisory is a non-blocking **stderr** line. Rationale: the
FOLLOWUP's R0-vetted position is that blocking intake would strand recovery of a deployed wallet
(`FOLLOWUPS.md:141`); `bundle` is dual-natured (engraving AND deployed-wallet backup) and cannot
distinguish the two at the descriptor-string boundary, so a loud-but-non-blocking advisory is the
correct tool. (Considered and rejected: a `bundle` refuse-unless-`--allow` flag — it adds a clap
flag → MINOR + schema_mirror + manual + GUI locksteps, and can block legitimate mid-recovery
backup.)

## §3 Architecture — one predicate, two walk adapters

### §3.1 Shared predicate (the single source of bit-math truth)

Extract the funds-safety bit-math out of `gate.rs`'s Older arm into a **new bin-crate-root module**
`crates/mnemonic-toolkit/src/timelock_advisory.rs`, declared `mod timelock_advisory;` in
`src/main.rs` (R0-r1 m4 — both consumers are bin-crate modules: `descriptor_builder/gate.rs`
(`main.rs:12`) and the `cmd/` surfaces (`main.rs:6`); `descriptor_builder` is bin-only, NOT in
`lib.rs`, so the shared module lives in the bin crate and both import it via `crate::timelock_advisory`.
**No `lib.rs` change.** Placed at crate root, NOT inside `descriptor_builder/`, so `cmd/` surfaces
do not depend on the builder module — architect layering call.):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]   // PartialEq/Eq required for the §3.3 debug_assert (R0-r1 m1)
pub enum TimelockMaskConsequence {
    /// Bit-31 disable flag set ⇒ CSV is a no-op (NO timelock at all).
    /// Reachable ONLY from the gate's IR path and the A-raw-card path (§3.3); never post-`from_str`.
    Bit31Disabled,
    /// Consensus masks the operand to `effective` in `unit`; the literal overstates it.
    Masked { effective: u16, unit: TimelockUnit },   // unit: Blocks | Seconds512
}

/// `Some(consequence)` iff `n` is a BIP-68 footgun (stray masked bits OR zero 16-bit value).
/// `None` for clean operands (1..=65535 blocks, or 0x400001..=0x40FFFF 512-second units).
pub fn older_consensus_masked(n: u32) -> Option<TimelockMaskConsequence>;
```

The predicate body is exactly `gate.rs:264`'s condition; the consequence computation is exactly
`gate.rs:269-285`'s bit-31 / unit / `n & 0xFFFF` logic. **Message strings stay in the callers**
(the gate keeps its verbatim diagnostic; the advisory builds its own text) so the gate's
user-facing output is byte-identical post-extraction (pinned by the §6 characterization test).

### §3.2 Two walk adapters (both call the predicate, both dedupe by operand)

The surfaces bifurcate by which parsed form they hold (architect C2, verified — there is **no**
single universal walk input):

- **Adapter A — md_codec Node-tree walk.** Recurse `md_codec::Descriptor.tree` (public field,
  **md-codec 0.35.3** — crates.io, the toolkit's pinned dep, `encode.rs:25`; `Node { tag, body }`
  `tree.rs:9`, `enum Body` `tree.rs:18` with `Children(Vec<Node>)` `:20` / `Variable{children}`
  `:24` / `Tr{tree}` `:49` recursion), matching `Tag::Older` + `Body::Timelock(u32)` (`tree.rs:70`).
  Serves every surface that holds an `MdDescriptor`. (All `tree.rs` lines are md-codec **0.35.3**;
  the stale local git checkout `fe1531b` and the drifted sibling working tree differ — cite 0.35.3,
  the published source the toolkit actually compiles.)
- **Adapter B — miniscript-AST walk.** Core: a generic
  `older_advisories_ms<Pk: MiniscriptKey, Ctx: ScriptContext>(ms: &Miniscript<Pk, Ctx>)` that
  recurses the miniscript AST, matches `Terminal::Older(lt)`, and recovers the raw operand via
  `lt.to_consensus_u32()` (miniscript rev `95fdd1c` `primitives/relative_locktime.rs:48` returns
  the inner `Sequence`'s u32 verbatim). The generic core is needed because Adapter-B surfaces hold
  **two different parsed types** (R0-r2 I1):
  - `export-wallet`, `restore --md1`, `xpub-search` literal funnel hold a
    `Descriptor<DescriptorPublicKey>` → a thin `older_advisories_descriptor(&Descriptor<Pk>)`
    unwraps the inner miniscript(s) (`Wsh` via `Wsh::as_inner()` / `Tr` leaves via `TapTree::leaves()`;
    `Sh(Wsh)` handled for completeness though no current surface produces it — R0-r3 m2) and calls
    the core.
  - `compare-cost` holds a `Translated` (`cost/translate.rs:19`) carrying
    `segv0: Miniscript<DefiniteDescriptorKey, Segwitv0>` (`:22`) and `tap:` (`:23`). BOTH
    `--descriptor` and `--miniscript` produce one `Translated` (dispatch `cost/mod.rs:128-136`), so
    a SINGLE hook on `translated.segv0` after the dispatch covers both input paths — it calls the
    core directly (no descriptor unwrap). Walking only `segv0` is sufficient even for single-leaf
    `tr(IK,M)` `--descriptor` input: `strip.rs` reverse-projects the Tap leaf into `segv0` via a
    `from_str` round-trip (`cost/strip.rs:125`) preserving the `older()` operand values; `tap` holds
    structurally identical `older()` nodes (only the key type differs) and dedup-by-value means a
    second walk adds nothing (R0-r3 m1).

Both adapters return `Vec<TimelockAdvisory>` deduped by operand value (architect I6: value-keyed,
not node-path-keyed — the user needs to know the value is masked, not which node it sits in).

### §3.3 Bit-31 / zero reachability — three regimes (architect C1, REVISED post-write)

`older()` operands reach the adapters by three routes with **different validation**. The original
"unreachable on intake" claim (justified via miniscript `from_str`) holds for two of them but
**fails for raw md1-card decode**, which bypasses miniscript:

- **Adapter B (miniscript `from_str`) — UNREACHABLE.** `RelLockTime`'s `TryFrom<Sequence>`
  (rev `95fdd1c` `relative_locktime.rs:70-80`) returns `Err` unless `seq.is_relative_lock_time()
  && seq != Sequence::ZERO`, so bit-31-set and zero `older()` fail to parse. Surfaces:
  `compare-cost`, `export-wallet`, `restore --md1`, `xpub-search` literal/BIP-388 funnel.
- **Adapter A built inside `parse_descriptor` (A-post-from_str) — UNREACHABLE.** `import-wallet`,
  `bundle`, `verify-bundle --descriptor` build the md_codec tree *after* `parse_descriptor` runs
  `from_str` (`parse_descriptor.rs:780`), so the tree is already bit-31/zero-free.
- **Adapter A from a raw md1 card (A-raw-card) — REACHABLE.** `md_codec` decode performs **NO
  operand validation**: `read_node` reads a raw 32-bit value —
  `Tag::After | Tag::Older => { let v = r.read_bits(32)? as u32; Body::Timelock(v) }`
  (**md-codec 0.35.3** `tree.rs:293-295`). A crafted md1 card can carry a bit-31-set or zero
  `older()` straight to Adapter A, bypassing miniscript. **Sole confirmed A-raw-card surface:**
  `xpub-search`'s md1 funnel (`parse_md1`, `descriptor_intake.rs:140-215`). (`verify-bundle --md1`
  card-only was traced and ruled OUT of scope — see §8.5, resolved R0-r1.)

**Consequence for the advisory:** it MUST handle **both** `TimelockMaskConsequence` variants. A
`debug_assert!(consequence != Bit31Disabled)` is valid ONLY at Adapter-B and A-post-from_str call
sites; **A-raw-card call sites must emit the `Bit31Disabled` message for real.** The gate's IR path
(`validate_fields` on raw `PolicyNode::Older(u32)` before step-2 `from_str`, `gate.rs:161-178`)
also reaches bit-31 — which is why the shared predicate keeps the variant.

`older(0)` (R0-r1 m5): bit-31 clear, low-16 zero → classified `Masked { effective: 0, unit:
Blocks }` (predicate's `(n & 0xFFFF) == 0` clause), **not** `Bit31Disabled`. It is rejected by
`TryFrom<Sequence>`'s `seq != Sequence::ZERO` check on the Adapter-B and A-post-from_str paths
(unreachable there) but reaches Adapter A via a raw md1 card → emits the `Masked{0}` "no effective
relative timelock" message (§5).

`restore --md1` is **fail-closed** on bit-31/zero: it reconstructs a descriptor *string* from the
card and re-parses via `from_str` (`restore.rs:833`), so a bit-31/zero card yields
`older(2147483649)` / `older(0)` → `from_str` rejects → restore errors with
*"--md1 reconstructed descriptor parse: …"* BEFORE any advisory. Not advised, by design.

## §4 Surfaces & coverage

Seven surfaces (FOLLOWUP Where-list extended from 4 → 7 as the implementing-commit lockstep,
§7). Parse-family grep-verified @ `3235431`:

| # | Surface | Parsed form held | Adapter |
|---|---------|------------------|---------|
| 1 | `import-wallet` (all 8 formats) | `MdDescriptor` via `parse_descriptor::parse_descriptor` (`wallet_import/`: `bitcoin_core.rs:278`, `pipeline.rs:322`, `bsms.rs:227`, `sparrow.rs:419`, `specter.rs:234`, `coldcard.rs:308`, `coldcard_multisig.rs:463`, `electrum.rs:377`) | A |
| 2 | `bundle` (`--descriptor`, `--descriptor-file`, `--import-json`) | `MdDescriptor` via `parse_descriptor` (`bundle.rs:1228`, `1603`, `1936` — the `1936` site resolves architect C3: `--import-json` IS covered) | A |
| 3 | `verify-bundle --descriptor` | `MdDescriptor` via `parse_descriptor` (`verify_bundle.rs:709`, `1017`) | A |
| 4 | `restore --md1` | `miniscript::Descriptor` re-parsed from the reconstructed descriptor string (`restore.rs:833`, `1277`) | B |
| 5 | `compare-cost` (`--descriptor` AND `--miniscript`) | both paths produce one `Translated` (`--descriptor`→`cost/strip.rs:21`, `--miniscript`→`cost/translate.rs:82`/`84`; dispatch `cost/mod.rs:128-136`). ONE hook on `translated.segv0: Miniscript<DefiniteDescriptorKey, Segwitv0>` after the dispatch covers BOTH paths (R0-r2 I1/m1). | B (core, on `Miniscript`) |
| 6 | `export-wallet --descriptor` | `miniscript::Descriptor` via `export_wallet.rs:452` / `566` / `715` | B |
| 7 | `xpub-search` | literal-xpub / BIP-388 funnel → `miniscript::Descriptor` (`descriptor_intake.rs:289`); **md1-card funnel** → `MdDescriptor` (`parse_md1`, `descriptor_intake.rs:140-215`) | B + A |

**Scope note (architect C2 disposition):** `xpub-search` is the weakest funds-safety case (it
mines xpubs, not a backup artifact). It is **kept in scope** — consistent with the user's
"advisory everywhere" decision and ~free once both adapters exist (the md1 funnel uses Adapter A;
the literal funnel uses Adapter B).

**Bit-31/zero regime per surface (§3.3):** rows 1–3 are A-post-from_str (bit-31 unreachable);
rows 4–6 are Adapter B (bit-31 unreachable; restore is additionally fail-closed); row 7's md1
funnel is **A-raw-card** (bit-31/zero REACHABLE — must handle the `Bit31Disabled` message), its
literal funnel is Adapter B. (`verify-bundle --md1` card-only is OUT of scope — §8.5, resolved R0-r1.)

**Emit discipline.** Each surface calls the appropriate adapter at its existing post-parse success
point and writes the deduped advisories to its own `E: Write` stderr **before** printing/engraving
its stdout result. `import-wallet`'s eight formats should collapse to the fewest emit sites the
code allows (single funnel preferred); the implementation plan enumerates exact emit sites with
grep-verified line numbers and the R0 gate confirms all 7 are covered.

**Walk-input mechanism — DECIDED (R0-r1 I3): option (i), direct `MdDescriptor.tree` walk.**
Each Adapter-A surface calls the `timelock_advisory` md_codec-tree walker (e.g.
`older_advisories_tree(&descriptor.tree)`) on the `MdDescriptor` it already holds, right after its
existing `parse_descriptor` call. **`parse_descriptor`'s return type is UNCHANGED** — Adapter A
lives entirely in `timelock_advisory.rs`. Rejected: option (ii) (extend `parse_descriptor` to
return `(MdDescriptor, Vec<…>)`) — it churns all `parse_descriptor` callers across `wallet_import/*`
+ `bundle.rs` + `verify_bundle.rs` for no functional gain. Both options pass the §6 tests
identically; (i) is minimal-churn.

## §5 Behavior

- **Trigger:** `older_consensus_masked(n).is_some()` for any `older()` operand in the policy.
- **Message — two forms** (which one is reachable depends on the regime, §3.3):
  - `Masked { effective, unit }` (all regimes): `advisory: older(<N>) is consensus-masked —
    BIP-68 uses only the low 16 bits, so this relative timelock has an effective value of
    <effective> <blocks|512-second units>; the literal <N> overstates the lock.` (For
    `effective == 0`: phrase as "no effective relative timelock".)
  - `Bit31Disabled` (A-raw-card only): `advisory: older(<N>) has the BIP-68 bit-31 disable flag
    set — consensus treats this CHECKSEQUENCEVERIFY as a no-op, so there is no relative timelock at
    all.`
  - Exact wording finalized at implementation; each must name the literal `N` and the consequence.
- **Dedup:** one line per distinct operand value (§3.2).
- **Stream/placement:** stderr only, **including `restore --md1`** (architect I4: inline in the
  stdout restore document would corrupt downstream parsing of that artifact).
- **`after()`:** no advisory (architect I5, verified): absolute locktimes are fail-closed at
  `from_str` and the gate's `After` arm (`gate.rs:298`) is range-only (`n==0`, `n>0x7FFF_FFFF`),
  with no BIP-68-style silent mask.

## §6 Testing

- **Predicate unit table** (`timelock_advisory.rs` tests): `older(65536)`→`Masked{0,Blocks}`;
  `older(0x800000|100)`→`Masked{100,Blocks}` (bit-23 stray); `older(0x400000|100)` =
  `older(4194404)`→`None` (clean 512s, value 100); known-clean set from `gate.rs:990`
  (`1`, `65535`, `52560`, `0x0040_0001`, `0x0040_FFFF`)→all `None`.
- **Gate-still-refuses characterization test** (TDD-first; architect m3, load-bearing): assert
  `build-descriptor` still refuses a masked `older()` with the **byte-identical** pre-extraction
  diagnostic — pins zero behavior drift from the predicate extraction.
- **Per-surface integration cells** (one per the 7 surfaces): advisory fires on the canonical
  masked policy `wsh(andor(pk(K0),older(65536),and_v(v:pk(K1),older(2016))))` (the recon probe
  shape — `older(65536)` masked, `older(2016)` clean → exactly one deduped advisory line); silent
  on a fully-clean descriptor; surface still exits 0 / engraves / round-trips (non-blocking
  proof). **`compare-cost`'s cell exercises BOTH invocations** (R0-r2 I2):
  `--descriptor wsh(andor(pk(K0),older(65536),and_v(v:pk(K1),older(2016))))` AND the bare
  `--miniscript andor(pk(K0),older(65536),and_v(v:pk(K1),older(2016)))` — both must emit the
  advisory (the single `translated` hook serves both, but each argv path is tested).
- **A-raw-card bit-31 reachability** (§3.3): proven by a **module unit test** walking a hand-built
  `Node` tree (`older_advisories_node`) containing `older(0x80000001)` (bit-31) + duplicate
  `older(65536)` → asserts `Bit31Disabled` is produced and dedup holds. The **xpub-search md1-funnel
  integration cell** then uses a *real* `older(65536)` card (built via the normal
  descriptor→`parse_descriptor`→encode pipeline; `older(65536)` is bit-31-clear so it encodes
  normally) to prove the md1-card→`older_advisories_tree` **hook wiring** emits the `Masked` advisory.
  A crafted bit-31 *card* end-to-end cell is intentionally omitted (adversarial-only — no descriptor
  string yields a bit-31 operand; advisor §3). RED-provable by reverting the tree walk to Masked-only.
- **False-positive + cleanliness guards** (advisor): a **clean-512s-unit** integration cell
  (`older(4194305)` = `0x400001` → advisory SILENT) guards the likeliest false-positive (a
  mis-set bit-22 mask); a **`--json` stdout-cleanliness** cell asserts the advisory stays on stderr
  and never leaks into the JSON stdout payload; the **dedup** assertion is operand-keyed (same literal
  twice → one line; two distinct masked literals → two lines).

## §7 SemVer & locksteps

- **PATCH** — advisory-only, zero clap delta → **no GUI `schema_mirror` impact, no manual
  flag-row.** (`schema_mirror` gates clap flag-NAME parity only; nothing here adds/renames a flag.)
- **Manual** (CLAUDE.md mirror invariant) — R0-r1 I4: add the advisory-behavior prose to
  `docs/manual/src/40-cli-reference/41-mnemonic.md`. Preferred shape: ONE shared "consensus-masked
  `older()` advisory" paragraph, cross-referenced from each of the **seven** affected subcommand
  sections — `bundle`, `restore`, `import-wallet`, `export-wallet`, `verify-bundle`, `compare-cost`,
  `xpub-search` — so a user of any surface finds it (not just bundle/restore). **Run the FULL manual
  lint**, not just flag-coverage (`make -C docs/manual lint MNEMONIC_BIN=…`) — the v0.50.0 cspell
  lesson (a new manual section fails CI's cspell pass even when flag-coverage is clean).
- **FOLLOWUPS** (`FOLLOWUPS.md`): rides the implementing commit — extend the Where list
  (`:140`) from 4 → 7 surfaces, and mark the entry RESOLVED.
- **Gate comment reword** (architect direction-consult m1): `gate.rs:262`'s *"on an engraving surface a
  silently-weakened timelock is a funds-safety bug"* reads as if the gate *should* cover every
  engraving surface. Reword to state the intended design plainly: this is the **JSON-IR authoring
  gate** (refuse-on-author); intake/round-trip surfaces get a non-blocking advisory (advise-on-intake).
  The split is deliberate, **not** a historical coverage gap being patched (advisor #8).
- **New FOLLOWUP** `older-advisory-blindness-suppression` (advisor #7, deferred): the advisory fires
  on every intake of an already-known-masked deployed wallet, every surface, every run, unsuppressable
  → advisory-blindness risk. Not built this cycle; a future `--quiet-advisories` would be MINOR +
  schema_mirror/manual locksteps. Recorded so it isn't re-discovered.
- No md-codec change (the wire correctly round-trips the literal — the advisory is
  presentation-layer). No sibling-codec companions.

## §8 R0 dispositions & items for the implementation plan

**Resolved in R0 round 1** (folded into the spec body; full review at
`design/agent-reports/older-timelock-advisory-r0-round1-review.md`):
- **§8.1 Walk-input mechanism** — DECIDED: option (i) direct `MdDescriptor.tree` walk;
  `parse_descriptor` unchanged (§4, I3).
- **§8.5 A-raw-card surface set** — DECIDED: the **sole** A-raw-card surface is `xpub-search`'s
  md1 funnel. `verify-bundle --md1` card-only (template mode) goes `run_multisig` →
  `emit_md1_checks` (decode / wallet_policy / xpub-match only; never traverses the policy tree for
  `older()`) → **OUT of scope** (R0-r1 m2). No 8th intake surface found (`inspect`, `convert`,
  `repair`, `addresses`, `decode-address` all examined; none surface `older()`).
- **I1/I2 citation + coverage fixes** folded (md-codec → 0.35.3 lines; `compare-cost --miniscript`
  second path added). m1/m3/m4/m5 folded.

**Carries to the implementation plan** (the plan-doc gets its own R0):
- **Emit-site minimization** for `import-wallet`'s 8 formats (prefer a single funnel emit over
  per-format). (`compare-cost`'s two paths are already resolved to ONE hook on `translated` —
  §4 row 5, R0-r2.)
- **Final advisory wording** (§5) — confirm each form names literal + effective + unit and is
  unambiguous; place the `debug_assert!(consequence != Bit31Disabled)` ONLY at Adapter-B /
  A-post-from_str call sites (NOT the A-raw-card site).
- **Characterization test** (§6) must assert the **exact** current gate diagnostic string (no
  drift) and be RED-provable by perturbing the extracted predicate.

## Non-goals

- Blocking/refusing on ANY intake surface (decided §2).
- `after()` advisories (§5).
- A `bundle` override flag (would be MINOR; rejected §2).
- Preset `--older`/`--recovery-older` blocks-vs-512s tightening — separate FOLLOWUP
  `archetype-older-blocks-flag-accepts-time-units` (`FOLLOWUPS.md`, gate-SPEC M1).
- Any md-codec wire change.
