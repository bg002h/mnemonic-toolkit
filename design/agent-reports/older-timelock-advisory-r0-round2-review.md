# R0 round 2 — architect review (verbatim) — SPEC_older_timelock_advisory.md

> Persisted verbatim per CLAUDE.md. Dispatched via Agent tool (feature-dev:code-architect,
> inherited session default model Opus 4.8; the body's self-attribution line is the agent's own).
> Verdict: **YELLOW** (0 Critical, 2 Important, 1 Minor). Source SHA `3235431` / md-codec 0.35.3.
> All three findings folded in commit following this file.

---

## R0 REVIEW — SPEC_older_timelock_advisory.md — Round 2

**Reviewer:** Claude Sonnet 4.6 (Fable 5)
**Source SHA verified:** `3235431` (spec self-declares); md-codec 0.35.3 from `/home/bcg/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/md-codec-0.35.3/src/`
**Date:** 2026-06-13

---

## Critical

None found.

---

## Important

### I1 — Adapter B input-type mismatch for `compare-cost`: §3.2 specifies `Descriptor<DescriptorPublicKey>` but `compare-cost` holds `Miniscript<DefiniteDescriptorKey, _>`

**Evidence:**

`cost/translate.rs:19-23` shows `Translated.segv0: Miniscript<DefiniteDescriptorKey, Segwitv0>` and `Translated.tap: Miniscript<DefiniteDescriptorKey, Tap>`. `cost/mod.rs:128-136` shows both dispatch arms produce a `Translated`, never a `Descriptor<DescriptorPublicKey>`.

All other Adapter B surfaces hold `Descriptor<DescriptorPublicKey>` (or its alias):
- `restore --md1`: `restore.rs:833,1277` → `MsDescriptor::<DescriptorPublicKey>::from_str`
- `export-wallet`: `export_wallet.rs:452,566,715` → `MsDescriptor::<DescriptorPublicKey>::from_str`
- `xpub-search` literal funnel: `descriptor_intake.rs:289` → `MsDescriptor::<DescriptorPublicKey>::from_str`

§3.2 says "Adapter B — Iterate a parsed `miniscript::Descriptor<DescriptorPublicKey>`, matching `Terminal::Older(lt)`". This description fits the three surfaces above but NOT `compare-cost`. The `compare-cost` surfaces hold raw `Miniscript<DefiniteDescriptorKey, _>` — the inner AST without the descriptor wrapper. Walking `Terminal::Older` on that type requires a different entry point than walking it on a `Descriptor<DescriptorPublicKey>`.

An implementer following §3.2 literally would write `older_advisories_descriptor(d: &Descriptor<DescriptorPublicKey>)` and find that `compare-cost`'s `Translated` cannot be passed to it. They would need to also write `older_advisories_miniscript(ms: &Miniscript<DefiniteDescriptorKey, Segwitv0>)` (and a Tap variant). The spec must acknowledge this bifurcation.

**Fix:** Update §3.2's Adapter B description to note that the actual walk input type differs by surface: for `export-wallet`, `restore --md1`, and `xpub-search`'s literal funnel the walker is called on `Descriptor<DescriptorPublicKey>` (extract the inner miniscript via pattern-matching `Wsh(inner)`, `ShInner::Wsh(inner)`, or `Tr`'s leaf scripts as appropriate); for `compare-cost` it is called directly on `Miniscript<DefiniteDescriptorKey, Segwitv0>` (and `Tap`). The implementation plan must specify which function signature Adapter B exposes — either a generic over `miniscript::Miniscript<_, _>` and a descriptor unwrapper, or two separate entry points.

---

### I2 — §6 test coverage gap: `compare-cost --miniscript` path has no explicit test cell after the I2 fold

**Evidence:**

§4 row 5 states "BOTH must fire the advisory" for `compare-cost`'s `--descriptor` and `--miniscript` paths, added by the R1 I2 fold. §6 states "Per-surface integration cells (one per the 7 surfaces)" — only one cell per surface row.

For `compare-cost`, the `--descriptor` path uses a `wsh(...)` descriptor string as input. The `--miniscript` path uses the bare inner miniscript string (e.g., `andor(pk(K0),older(65536),and_v(v:pk(K1),older(2016)))`) as input. These are distinct invocations with different argv. A single test cell using the canonical `wsh(andor(...))` form would test only the `--descriptor` path and leave the `--miniscript` path uncovered.

The I2 fold correctly added `--miniscript` to the §4 coverage table but did not extend §6 to require a second `compare-cost` test cell. With "one per 7 surfaces", an implementer writing exactly what §6 says would test `--descriptor` for surface #5 and miss `--miniscript`.

**Fix:** §6's per-surface integration cell description for `compare-cost` should explicitly say: "the `compare-cost` cell covers BOTH paths — `--descriptor wsh(andor(...))` and `--miniscript andor(...)` — as two invocations within the same test function (or as sibling test cells)."

---

## Minor

### m1 — `cost/mod.rs` dispatch range: spec says `:128-135`, closing brace is at `:136`

**Evidence:** `cost/mod.rs:128-136` shows the `match &args.input { ... }` block closes at line 136 (the `};`). The spec (§4 row 5) cites "dispatch at `cost/mod.rs:128-135`". The range 128-135 ends one line before the closing brace.

**Fix:** Change the citation to `cost/mod.rs:128-136`.

---

## Fold verification — round-1 findings

### I1 (md-codec citations) — VERIFIED CORRECT

All md-codec 0.35.3 citations confirmed against the registry source `md-codec-0.35.3/src/tree.rs`:
- `Node` at `:9`; `Body` at `:18`; `Children(Vec<Node>)` at `:20`; `Variable{children}` at `:24`; `Tr{tree}` at `:49`; `Timelock(u32)` at `:70`; Older decode arm at `:293-295` (`let v = r.read_bits(32)? as u32; Body::Timelock(v)` — no operand validation). `encode.rs:25`: `pub tree: Node,`. Cargo.lock confirms `md-codec` resolves to `0.35.3` from registry (root `Cargo.toml` `[patch.crates-io]` only patches `miniscript`). Local git checkout and sibling working tree correctly excluded.

### I2 (compare-cost --miniscript path) — FOLD VERIFIED (but exposed new I1 above)

The fold correctly added `cost/translate.rs:82/84` and `cost/mod.rs:128-135`. `translate.rs:81-82` parses `segv0` via `Miniscript::from_str`; `:83-84` parses `tap`. `--descriptor` path at `strip.rs:21` is `Descriptor::<DescriptorPublicKey>::from_str(input)`. Both go through miniscript's `TryFrom<Sequence>` (bit-31 unreachable). Correct on coverage; exposed the Adapter B type mismatch.

### I3 (walk-input mechanism) — FOLD VERIFIED

§4 states option (i) direct `MdDescriptor.tree` walk. `parse_descriptor.rs:748-752` returns `MdDescriptor`; `encode.rs:25` `pub tree: Node` is public. All Adapter-A sites can pass `.tree` to the walker; return type unchanged. CORRECT.

### I4 (manual) — FOLD VERIFIED

§7 names `docs/manual/src/40-cli-reference/41-mnemonic.md`. File exists (3931+ lines). All 7 sections present: `bundle`:45, `verify-bundle`:518, `export-wallet`:686, `restore`:734, `import-wallet`:1019, `xpub-search`:3159, `compare-cost`:3693.

### m1/m2/m3/m4/m5 — ALL FOLD VERIFIED

m1 derives `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` (debug_assert compiles). m2 §8.5 closed (verify-bundle --md1 OUT of scope). m3 file name named. m4 BIN-crate-root placement confirmed (`main.rs:12` `mod descriptor_builder;` bin-only, NOT in lib.rs; both gate.rs + cmd/ import `crate::timelock_advisory`). m5 `older(0)` → `Masked{0,Blocks}` classification correct.

---

## Additional correctness checks (new hunt)

- §3.3 bit-math re-confirmed against `gate.rs:264` (predicate), `:269` (Bit31Disabled branch), `:274-285` (Masked branch). Consequence computation extractable without drift.
- §3.2 dedup by operand value — correct for a user-facing advisory.
- §4 `descriptor_intake.rs:140-215` — loose but non-wrong (dispatch ~131, parse_md1 ~210).
- §5 `effective==0` wording deferred to impl — no conflict with gate's wording.
- §7 gate comment reword: `gate.rs:262` confirmed.
- PATCH SemVer — zero clap delta, no schema_mirror. CORRECT.
- §6 A-raw-card test feasibility: `tree.rs::write_node` `Body::Timelock(v) => w.write_bits(u64::from(*v), 32)` writes raw u32 without validation → bit-31 card constructible.

---

## Verdict

**YELLOW** — 0 Critical, 2 Important, 1 Minor. The gate does NOT pass at GREEN.

**I1** is the central new finding introduced by the R1 I2 fold: `compare-cost`'s parsed form (`Miniscript<DefiniteDescriptorKey, _>`) is a different type than the other Adapter B surfaces (`Descriptor<DescriptorPublicKey>`). The §3.2 Adapter B description is wrong for `compare-cost`. The implementation plan cannot be written cleanly until the spec resolves the Adapter B type signature.

**I2** is a consequence of I1 + the I2 fold: §6 says "one per the 7 surfaces" but `compare-cost` now has two invocation paths that must both be tested.

**Required fold actions before re-dispatch:**

1. **[I1]** In §3.2, extend the Adapter B description to state the input-type bifurcation: (a) `Descriptor<DescriptorPublicKey>` surfaces extract the inner miniscript; (b) `compare-cost` calls the core on `Miniscript<DefiniteDescriptorKey, Segwitv0>` (and `Tap`) directly. Specify either one generic function or two entry points.

2. **[I2]** In §6, the `compare-cost` integration cell must cover TWO invocations: `--descriptor wsh(andor(...))` AND `--miniscript andor(...)`.

3. **[m1]** Change `cost/mod.rs:128-135` to `cost/mod.rs:128-136`.
