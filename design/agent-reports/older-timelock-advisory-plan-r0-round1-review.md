# Plan-R0 round 1 — architect review (verbatim) — PLAN_older_timelock_advisory.md

> Persisted verbatim per CLAUDE.md. Formal R0 gate on the IMPLEMENTATION PLAN (the spec already
> passed its own R0, GREEN). Dispatched via Agent tool (feature-dev:code-architect, inherited session
> default model Opus 4.8; body's self-attribution is the agent's own). Verdict: **does not pass** —
> 0 Critical, 1 Important (I-1), 4 Minor. All folded in the commit following this file (I-1 + m-2
> independently grep-verified: gate.rs test callers at 1146/1202/1263; dispatch at descriptor_intake.rs:140).
> Branch `older-timelock-advisory`, HEAD `6ace106`.

---

## Formal Plan-R0 Review — Consensus-masked `older()` Intake Advisory

**Artifact:** `design/PLAN_older_timelock_advisory.md`, branch `older-timelock-advisory` HEAD `6ace106`
**Date:** 2026-06-13

### Code Compilation Against Real APIs — all major claims verify

**Miniscript rev `95fdd1c`:** `Miniscript::iter()` → `Iter<'_,Pk,Ctx>` with `Item=&Miniscript`; `node.node: Terminal<Pk,Ctx>`; `Terminal::Older(lt)` with `lt: RelLockTime`; `lt.to_consensus_u32()` (`relative_locktime.rs:48`). `Wsh::as_inner() -> &Miniscript<Pk,Segwitv0>` (`segwitv0.rs:38`); `Sh::as_inner() -> &ShInner<Pk>` (`sh.rs:112`); `ShInner::Wsh(wsh)`. `Tr::leaves() -> TapTreeIter` (`tr/mod.rs:110`); `TapTreeIterItem::miniscript() -> &Arc<Miniscript<Pk,Tap>>` (`tr/taptree.rs:194`) — coerces `&Arc<T>→&T` at the call. Imports all valid; `DefiniteDescriptorKey: MiniscriptKey` (`key.rs:1455`); `miniscript::bitcoin::PublicKey` via `pub use bitcoin` (`lib.rs:92`).

**md-codec 0.35.3:** `Node{tag,body}` pub fields (`tree.rs:9`); `Tag: PartialEq` (`tag.rs:14`) so `node.tag == Tag::Older` compiles; `Body::{Timelock(u32),Children,Variable{k,children},Tr{is_nums,key_index,tree:Option<Box<Node>>}}` present; imports `md_codec::tag::Tag`, `md_codec::tree::{Body,Node}` valid; `md_codec::Descriptor.tree: Node` (`encode.rs:25`).

**Gate refactor (Task 2):** current arm `gate.rs:264` predicate identical to `older_consensus_masked`; format strings copied verbatim from `gate.rs:269-295`; characterization substrings present in current output (`:288-296`). `older_tree`/`field_diags` helpers exist (`gate.rs:930/942`).

### Hook Sites — spot-checked, accurate
`import_wallet.rs:1285` (`parsed: Vec<ParsedImport>`, `.descriptor: md_codec::Descriptor`); `bundle.rs:1662/1953` (both `descriptor: md_codec::Descriptor` in scope before `emit_unified`); `verify_bundle.rs:1028` (`descriptor` in scope before `verify_emit_from_expected`); `restore.rs:1291` (`parsed: MsDescriptor<DescriptorPublicKey>` post-fidelity-guard); `descriptor_intake.rs:140` dispatch (plan cited :141 — see m-2) + `parse_md1` hook `~:227`, literal funnel `~:290` (`stderr` already a param).

### Advisor folds — all verified
(a) Task 8 Step 3d documents `build_descriptor.rs:500/530` deliberate discard; safety verified (gate refuses masked older() at step 1 before cost preview). (b) `older_advisories_node` added Task 1, used Task 3. (c) Task 2 uses `older_tree`/`field_diags`. (d) Task 11 has clean-512s + `--json`-cleanliness + operand-keyed dedup. (e) advisory-blindness FOLLOWUP Task 12.

### Spec coverage — complete
§3.1→T1; §3.2→T1,T3; §3.3→T1; §4 (7 surfaces)→T4-10; §5→T1; §6→T1,2,3,4-10,11; §7→T2,12,13. No gap.

### IMPORTANT

**I-1: `cargo build --bin mnemonic` is insufficient to catch three `#[cfg(test)]` callers of `run_compare_cost` in `gate.rs` (Task 8, Step 4).** Test-module callers at `gate.rs:1146`, `:1202`, `:1263` need the new `stderr` arg but are invisible to `cargo build` (which skips `#[cfg(test)]`). The plan's Step 4 "`cargo build` to catch missed call sites" gives a false-green; they only fail at `cargo test`. Step 3d says "any TEST callers pass `&mut Vec::new()`" without enumerating them. **Fix:** add Step 3e enumerating `gate.rs:~1146/~1202/~1263` + verify `cargo test --bin mnemonic gate`; change Step 4 to run `cargo test --bin mnemonic`.

### MINOR
- **m-1:** Task 3 Step 2 stale note ("if the `md_codec::Descriptor` literal is wrong") — the test now uses `older_advisories_node(&root)` with a hand `Node`; no literal. Fix the note.
- **m-2:** Task 10 Step 3b cites dispatch `:141`; actual is `:140` (`DescriptorShape::Md1 => parse_md1(payload),`).
- **m-3:** `leaf.miniscript()` returns `&Arc<Miniscript<Pk,Tap>>`; passing to `older_advisories_ms` relies on `&Arc<T>→&T` deref coercion (compiles). Add a clarifying comment in the `Tr` arm.
- **m-4:** test var `s512` holds `0x0080_0064` (bit-22 CLEAR → Blocks, not 512s). Assertion correct; name misleading. Rename.

### Sequencing — sound
module → gate → adapters → surfaces → cross-surface → locksteps → verify; no forward dependency.

### Verdict
**Does not pass** — 0 Critical, 1 Important (I-1), 4 Minor. After adding Task 8 Step 3e (enumerate the three gate.rs test callers + `cargo test` verification) and the 4 minor fixes, the plan is API-verified, spec-complete, and executable.
