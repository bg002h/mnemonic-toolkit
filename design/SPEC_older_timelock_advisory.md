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

Extract the funds-safety bit-math out of `gate.rs`'s Older arm into a **new library-root module**
`crates/mnemonic-toolkit/src/timelock_advisory.rs` (NOT inside `descriptor_builder/` — `cmd/`
surfaces must not depend on the builder module; layering call from the architect, folded):

```rust
pub enum TimelockMaskConsequence {
    /// Bit-31 disable flag set ⇒ CSV is a no-op (NO timelock at all).
    /// Reachable ONLY from the gate's IR path (raw u32 pre-`from_str`); see §3.3.
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

- **Adapter A — md_codec Node-tree walk.** Recurse `md_codec::Descriptor.tree`
  (public field, `md-codec encode.rs:25`; `Node { tag, body }` with `Body::Children` /
  `Variable{children}` / `Tr{tree}` recursion, `tree.rs:9-52`), matching
  `Tag::Older` + `Body::Timelock(u32)` (`tree.rs:49,169`). Serves every surface that holds an
  `MdDescriptor`.
- **Adapter B — miniscript-AST walk.** Iterate a parsed
  `miniscript::Descriptor<DescriptorPublicKey>`, matching `Terminal::Older(lt)` and recovering the
  raw operand via `lt.to_consensus_u32()` (miniscript rev `95fdd1c`
  `primitives/relative_locktime.rs:48` returns the inner `Sequence`'s u32 verbatim). Serves
  surfaces that hold only a miniscript descriptor.

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
  (`md-codec tree.rs:169-172`). A crafted md1 card can carry a bit-31-set or zero `older()`
  straight to Adapter A, bypassing miniscript. Confirmed A-raw-card surface: `xpub-search`'s md1
  funnel (`parse_md1`, `descriptor_intake.rs:140-215`). `verify-bundle --md1` card-only is an R0
  item (§8.5).

**Consequence for the advisory:** it MUST handle **both** `TimelockMaskConsequence` variants. A
`debug_assert!(consequence != Bit31Disabled)` is valid ONLY at Adapter-B and A-post-from_str call
sites; **A-raw-card call sites must emit the `Bit31Disabled` message for real.** The gate's IR path
(`validate_fields` on raw `PolicyNode::Older(u32)` before step-2 `from_str`, `gate.rs:161-178`)
also reaches bit-31 — which is why the shared predicate keeps the variant.

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
| 5 | `compare-cost` | `miniscript::Descriptor` via `cost/strip.rs:21` | B |
| 6 | `export-wallet --descriptor` | `miniscript::Descriptor` via `export_wallet.rs:452` / `566` / `715` | B |
| 7 | `xpub-search` | literal-xpub / BIP-388 funnel → `miniscript::Descriptor` (`descriptor_intake.rs:289`); **md1-card funnel** → `MdDescriptor` (`parse_md1`, `descriptor_intake.rs:140-215`) | B + A |

**Scope note (architect C2 disposition):** `xpub-search` is the weakest funds-safety case (it
mines xpubs, not a backup artifact). It is **kept in scope** — consistent with the user's
"advisory everywhere" decision and ~free once both adapters exist (the md1 funnel uses Adapter A;
the literal funnel uses Adapter B).

**Bit-31/zero regime per surface (§3.3):** rows 1–3 are A-post-from_str (bit-31 unreachable);
rows 4–6 are Adapter B (bit-31 unreachable; restore is additionally fail-closed); row 7's md1
funnel is **A-raw-card** (bit-31/zero REACHABLE — must handle the `Bit31Disabled` message), its
literal funnel is Adapter B. `verify-bundle --md1` card-only reachability is R0 item §8.5.

**Emit discipline.** Each surface calls the appropriate adapter at its existing post-parse success
point and writes the deduped advisories to its own `E: Write` stderr **before** printing/engraving
its stdout result. `import-wallet`'s eight formats should collapse to the fewest emit sites the
code allows (single funnel preferred); the implementation plan enumerates exact emit sites with
grep-verified line numbers and the R0 gate confirms all 7 are covered.

**Walk-input mechanism (deferred to the implementation plan + R0):** Adapter-A surfaces hold an
`MdDescriptor`; they may either (i) walk `MdDescriptor.tree` directly, or (ii) have
`parse_descriptor` compute advisories once internally (on its `ms_desc` at `parse_descriptor.rs:780`
via Adapter B) and return them — trading a ~14-caller return-type change for a single computation
site. The plan picks the minimal-churn option; both satisfy the §6 per-surface tests identically.

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
  proof).
- **A-raw-card bit-31 / zero cell** (§3.3): construct an md1 card carrying `older(0x80000001)`
  (bit-31) and one carrying `older(0)` — either via `md_codec` encode of a hand-built
  `Body::Timelock` tree, or by direct bit-encode — feed each to `xpub-search`'s md1 funnel; assert
  the `Bit31Disabled` (resp. `Masked{0}`) advisory fires AND the surface still exits 0. This is the
  cell that proves the §3.3 correction (bit-31 reachable past miniscript's filter); RED-provable by
  reverting the advisory to Masked-only / by re-adding the `debug_assert` at the A-raw-card site.

## §7 SemVer & locksteps

- **PATCH** — advisory-only, zero clap delta → **no GUI `schema_mirror` impact, no manual
  flag-row.** (`schema_mirror` gates clap flag-NAME parity only; nothing here adds/renames a flag.)
- **Manual** (CLAUDE.md mirror invariant): add an advisory-behavior prose paragraph under
  `docs/manual/src/40-cli-reference/` (the `bundle` + `restore` sections) describing the
  non-blocking masked-`older()` advisory. **Run the FULL manual lint**, not just flag-coverage
  (`make -C docs/manual lint MNEMONIC_BIN=…`) — the v0.50.0 cspell lesson (a new manual section
  fails CI's cspell pass even when flag-coverage is clean).
- **FOLLOWUPS** (`FOLLOWUPS.md`): rides the implementing commit — extend the Where list
  (`:140`) from 4 → 7 surfaces, and mark the entry RESOLVED.
- **Gate comment reword** (architect m1): `gate.rs:262`'s *"on an engraving surface a
  silently-weakened timelock is a funds-safety bug"* now overstates the gate's coverage (only
  `build-descriptor` is gated; `bundle` is the real engraving surface and is advisory-only).
  Reword to describe the gate as the JSON-IR authoring gate.
- No md-codec change (the wire correctly round-trips the literal — the advisory is
  presentation-layer). No sibling-codec companions.

## §8 Open items for the formal R0 gate

R0 must converge to 0 Critical / 0 Important before any code, and explicitly adjudicate:
1. **Walk-input mechanism** (§4): `MdDescriptor.tree` direct-walk vs `parse_descriptor`
   return-type extension — pick the minimal-churn option and confirm it covers all Adapter-A
   surfaces.
2. **Emit-site minimization** for `import-wallet`'s 8 formats (single funnel vs per-format).
3. **Final advisory wording** (§5) — confirm it names literal + effective + unit and is
   unambiguous; confirm the `debug_assert!` documenting bit-31 unreachability (§3.3).
4. Confirm the §6 characterization test asserts the **exact** current gate diagnostic string
   (no drift) and is RED-provable by perturbing the extracted predicate.
5. **A-raw-card surface set** (§3.3): confirm the complete set of surfaces that walk a tree from
   `md_codec::decode` of a raw card (bypassing `from_str`, so bit-31/zero reachable).
   `xpub-search`'s md1 funnel is confirmed; trace whether any `verify-bundle --md1` card-only path
   (no `--descriptor`) independently walks the decoded card's tree for `older()` — if so it is
   A-raw-card (needs bit-31 handling + a test cell); if it only checks card identity / never
   surfaces the card's `older()`, document it explicitly out of scope.

## Non-goals

- Blocking/refusing on ANY intake surface (decided §2).
- `after()` advisories (§5).
- A `bundle` override flag (would be MINOR; rejected §2).
- Preset `--older`/`--recovery-older` blocks-vs-512s tightening — separate FOLLOWUP
  `archetype-older-blocks-flag-accepts-time-units` (`FOLLOWUPS.md`, gate-SPEC M1).
- Any md-codec wire change.
