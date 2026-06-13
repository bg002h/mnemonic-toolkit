# Consensus-masked `older()` Intake Advisory — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Emit a non-blocking, bit-31-aware stderr advisory on all 7 intake/round-trip surfaces when a descriptor's `older()` relative timelock is BIP-68 consensus-masked (weaker than its literal).

**Architecture:** One shared predicate `older_consensus_masked` (extracted from `descriptor_builder::gate`) in a new bin-crate module `timelock_advisory.rs`, plus two walk adapters — **Adapter A** (md_codec `Node` tree) and **Adapter B** (generic miniscript-AST core + `Descriptor` unwrap). Surfaces call a collector + `emit_advisories(&adv, stderr)`. `build-descriptor` keeps its hard refuse (the gate now sources its bit-math from the shared predicate; diagnostic byte-identical).

**Tech Stack:** Rust; `miniscript` (git rev `95fdd1c`), `md-codec` 0.35.3 (crates.io); tests via `assert_cmd` + `predicates` (integration) and `#[cfg(test)]` bin-crate unit tests.

**Spec:** `design/SPEC_older_timelock_advisory.md` (R0-GREEN, `design/agent-reports/older-timelock-advisory-r0-round{1,2,3}-review.md`). Source SHA `3235431`, branch `older-timelock-advisory`.

**Conventions:** TDD (failing test first). Stage paths explicitly (no `git add -A`). Bin-crate tests run with `cargo test --bin mnemonic`; integration tests with `cargo test --test <name>`. Commit message trailer: `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`. This plan-doc gets its OWN R0 gate (0C/0I) BEFORE Task 1.

---

## File Structure

**Create:**
- `crates/mnemonic-toolkit/src/timelock_advisory.rs` — predicate + two adapters + message + emit. One responsibility: detect & describe consensus-masked `older()`.

**Modify (source):**
- `crates/mnemonic-toolkit/src/main.rs` — add `mod timelock_advisory;`; thread `stderr` into the `CompareCost` dispatch (`:198`).
- `crates/mnemonic-toolkit/src/descriptor_builder/gate.rs` — `PolicyNode::Older` arm sources bit-math from the shared predicate (byte-identical diagnostic); reword stale `:262` comment.
- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` — hook after select (`~:1285`).
- `crates/mnemonic-toolkit/src/cmd/bundle.rs` — hooks at `~:1662` and `~:1953`.
- `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` — hook at `~:1028`.
- `crates/mnemonic-toolkit/src/cmd/restore.rs` — hook at `~:1291`.
- `crates/mnemonic-toolkit/src/cost/mod.rs` — add `stderr: &mut E` to `run_compare_cost`; hook after dispatch (`~:136`).
- `crates/mnemonic-toolkit/src/cmd/compare_cost.rs` — add `stderr: &mut E` to `run`; propagate.
- `crates/mnemonic-toolkit/src/cmd/export_wallet.rs` — hooks at `~:452` (`--descriptor`) and `~:721` (`--from-import-json`).
- `crates/mnemonic-toolkit/src/cmd/xpub_search/descriptor_intake.rs` — hook at `~:290` (literal funnel) and `~:227` (md1 funnel; `parse_md1` gains `stderr`).

**Modify (tests):** new `#[cfg(test)]` in `timelock_advisory.rs`; new cases in existing `tests/cli_*.rs` per surface (+ a new `tests/cli_older_advisory.rs` for cross-surface + A-raw-card).

**Modify (locksteps):**
- `docs/manual/src/40-cli-reference/41-mnemonic.md` — advisory prose paragraph x-ref'd from 7 sections.
- `design/FOLLOWUPS.md` — entry `intake-surfaces-accept-masked-older-no-advisory`: Where 4→7, mark RESOLVED.

---

## Task 1: New module — predicate, types, message, emit (+ unit tests)

**Files:**
- Create: `crates/mnemonic-toolkit/src/timelock_advisory.rs`
- Modify: `crates/mnemonic-toolkit/src/main.rs` (add `mod timelock_advisory;` near `:25` `mod secret_advisory;`)

- [ ] **Step 1: Write the failing unit tests** (append `#[cfg(test)]` to the new file)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predicate_classifies_operands() {
        use TimelockMaskConsequence::*;
        use TimelockUnit::*;
        // masked-to-zero (stray high bit, low-16 == 0)
        assert_eq!(older_consensus_masked(65536), Some(Masked { effective: 0, unit: Blocks }));
        // stray bit 23 → consensus masks to low-16 value 100 (blocks)
        assert_eq!(older_consensus_masked(0x0080_0064), Some(Masked { effective: 100, unit: Blocks }));
        // bit-31 disable flag set
        assert_eq!(older_consensus_masked(0x8000_0001), Some(Bit31Disabled));
        // clean block values → None
        for n in [1u32, 2016, 52560, 65535] {
            assert_eq!(older_consensus_masked(n), None, "clean blocks {n}");
        }
        // clean 512-second-unit values (bit 22 set, nonzero low-16) → None
        for n in [0x0040_0001u32, 0x0040_FFFF] {
            assert_eq!(older_consensus_masked(n), None, "clean 512s {n:#x}");
        }
    }

    #[test]
    fn message_forms() {
        let masked = TimelockAdvisory { operand: 65536, consequence: TimelockMaskConsequence::Masked { effective: 0, unit: TimelockUnit::Blocks } };
        assert!(masked.message().contains("advisory: older(65536) is consensus-masked"));
        assert!(masked.message().contains("NO effective value"));
        let s512 = TimelockAdvisory { operand: 0x0080_0064, consequence: TimelockMaskConsequence::Masked { effective: 100, unit: TimelockUnit::Blocks } };
        assert!(s512.message().contains("effective value of 100 blocks"));
        let b31 = TimelockAdvisory { operand: 0x8000_0001, consequence: TimelockMaskConsequence::Bit31Disabled };
        assert!(b31.message().contains("bit-31 disable flag set"));
        assert!(b31.message().contains("no relative timelock at all"));
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test --bin mnemonic timelock_advisory::tests -- --nocapture`
Expected: FAIL — `cannot find ... older_consensus_masked` / module not declared.

- [ ] **Step 3: Write the module head + predicate + types + message + emit**

Create `crates/mnemonic-toolkit/src/timelock_advisory.rs` (place the `#[cfg(test)]` block from Step 1 at the bottom):

```rust
//! Non-blocking consensus-masked `older()` advisory (SPEC_older_timelock_advisory).
//!
//! Intake/round-trip surfaces accept BIP-68 consensus-masked relative timelocks
//! that `build-descriptor`'s authoring gate refuses. This module is THE single
//! source of the bit-math (`older_consensus_masked`, shared with the gate) plus
//! two walk adapters that collect non-blocking advisories.

use std::collections::BTreeSet;
use std::io::Write;

use md_codec::tag::Tag;
use md_codec::tree::{Body, Node};
use miniscript::descriptor::ShInner;
use miniscript::miniscript::decode::Terminal;
use miniscript::{Descriptor, Miniscript, MiniscriptKey, ScriptContext};

/// Unit of a BIP-68 relative timelock value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelockUnit {
    Blocks,
    Seconds512,
}

/// What consensus does to a footgun `older()` operand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelockMaskConsequence {
    /// Bit-31 disable flag set ⇒ CSV is a no-op (no timelock at all). Reachable
    /// from the gate's IR path and the A-raw-card path; never post-`from_str`.
    Bit31Disabled,
    /// Consensus masks the operand to `effective` in `unit`.
    Masked { effective: u16, unit: TimelockUnit },
}

/// A collected advisory: the literal operand + its consensus consequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimelockAdvisory {
    pub operand: u32,
    pub consequence: TimelockMaskConsequence,
}

/// THE shared predicate (SPEC §3.1). `Some` iff `n` is a BIP-68 footgun: a bit
/// outside {low-16 value, bit-22 type-flag} is set, OR the 16-bit value is zero.
/// `None` for clean operands (1..=65535 blocks, or 0x400001..=0x40FFFF 512-second
/// units). Mirrors `descriptor_builder::gate`'s former inline logic verbatim.
pub fn older_consensus_masked(n: u32) -> Option<TimelockMaskConsequence> {
    if (n & !0x0040_FFFFu32) != 0 || (n & 0x0000_FFFFu32) == 0 {
        if n & 0x8000_0000 != 0 {
            Some(TimelockMaskConsequence::Bit31Disabled)
        } else {
            let unit = if n & 0x0040_0000 != 0 {
                TimelockUnit::Seconds512
            } else {
                TimelockUnit::Blocks
            };
            Some(TimelockMaskConsequence::Masked {
                effective: (n & 0x0000_FFFF) as u16,
                unit,
            })
        }
    } else {
        None
    }
}

impl TimelockAdvisory {
    /// The stderr advisory line (SPEC §5). Two forms by consequence.
    pub fn message(&self) -> String {
        match self.consequence {
            TimelockMaskConsequence::Bit31Disabled => format!(
                "advisory: older({}) has the BIP-68 bit-31 disable flag set — consensus treats this \
                 CHECKSEQUENCEVERIFY as a no-op, so there is no relative timelock at all.",
                self.operand
            ),
            TimelockMaskConsequence::Masked { effective, unit } => {
                let unit_str = match unit {
                    TimelockUnit::Blocks => "blocks",
                    TimelockUnit::Seconds512 => "512-second units",
                };
                if effective == 0 {
                    format!(
                        "advisory: older({}) is consensus-masked — BIP-68 uses only the low 16 bits, \
                         so this relative timelock has NO effective value (0 {unit_str}); the literal \
                         overstates the lock.",
                        self.operand
                    )
                } else {
                    format!(
                        "advisory: older({}) is consensus-masked — BIP-68 uses only the low 16 bits, \
                         so this relative timelock has an effective value of {effective} {unit_str}; \
                         the literal overstates the lock.",
                        self.operand
                    )
                }
            }
        }
    }
}

/// Write each advisory's message to `stderr` (best-effort; mirrors `secret_advisory`).
pub fn emit_advisories<E: Write>(advisories: &[TimelockAdvisory], stderr: &mut E) {
    for a in advisories {
        let _ = writeln!(stderr, "{}", a.message());
    }
}

fn merge_deduped(advs: Vec<TimelockAdvisory>, seen: &mut BTreeSet<u32>, out: &mut Vec<TimelockAdvisory>) {
    for a in advs {
        if seen.insert(a.operand) {
            out.push(a);
        }
    }
}

// ---- Adapter B: miniscript AST (post-`from_str`; bit-31 unreachable) ---------

/// Generic core. Walks every sub-node of `ms` for `Terminal::Older` and collects
/// deduped advisories. Adapter B is ALWAYS post-`from_str`, so a bit-31 operand
/// cannot occur (the `debug_assert` documents the invariant; see SPEC §3.3).
pub fn older_advisories_ms<Pk: MiniscriptKey, Ctx: ScriptContext>(
    ms: &Miniscript<Pk, Ctx>,
) -> Vec<TimelockAdvisory> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for node in ms.iter() {
        if let Terminal::Older(lt) = &node.node {
            let n = lt.to_consensus_u32();
            if let Some(consequence) = older_consensus_masked(n) {
                debug_assert!(
                    consequence != TimelockMaskConsequence::Bit31Disabled,
                    "Adapter B is post-from_str; a bit-31 older() cannot parse"
                );
                if seen.insert(n) {
                    out.push(TimelockAdvisory { operand: n, consequence });
                }
            }
        }
    }
    out
}

/// Unwrap a parsed `Descriptor` to its inner miniscript(s) and collect advisories
/// (deduped across all leaves). Adapter B.
pub fn older_advisories_descriptor<Pk: MiniscriptKey>(d: &Descriptor<Pk>) -> Vec<TimelockAdvisory> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    match d {
        Descriptor::Wsh(wsh) => merge_deduped(older_advisories_ms(wsh.as_inner()), &mut seen, &mut out),
        Descriptor::Sh(sh) => {
            // Sh(Wsh) handled for completeness; no current surface produces sh(wsh) (R0-r3 m2).
            if let ShInner::Wsh(wsh) = sh.as_inner() {
                merge_deduped(older_advisories_ms(wsh.as_inner()), &mut seen, &mut out);
            }
        }
        Descriptor::Tr(tr) => {
            for leaf in tr.leaves() {
                merge_deduped(older_advisories_ms(leaf.miniscript()), &mut seen, &mut out);
            }
        }
        // Bare / Pkh / Wpkh carry no miniscript with older().
        _ => {}
    }
    out
}

// ---- Adapter A: md_codec Node tree (md1-card decode; bit-31 REACHABLE) --------

/// Walk an `md_codec` descriptor's node tree for `Tag::Older` + `Body::Timelock`.
/// Used by md1-card decode paths (A-raw-card: bit-31 REACHABLE → NO debug_assert)
/// AND by `parse_descriptor`-sourced MdDescriptors (A-post-from_str, bit-31-free).
pub fn older_advisories_tree(desc: &md_codec::Descriptor) -> Vec<TimelockAdvisory> {
    older_advisories_node(&desc.tree)
}

/// Walk a `Node` tree directly — unit-testable without constructing a full
/// `md_codec::Descriptor` (avoids `PathDecl`/`UseSitePath`/`TlvSection` field fragility).
pub(crate) fn older_advisories_node(root: &Node) -> Vec<TimelockAdvisory> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    walk_node(root, &mut seen, &mut out);
    out
}

fn walk_node(node: &Node, seen: &mut BTreeSet<u32>, out: &mut Vec<TimelockAdvisory>) {
    if node.tag == Tag::Older {
        if let Body::Timelock(n) = node.body {
            if let Some(consequence) = older_consensus_masked(n) {
                if seen.insert(n) {
                    out.push(TimelockAdvisory { operand: n, consequence });
                }
            }
        }
        return; // Older is a leaf
    }
    match &node.body {
        Body::Children(children) => children.iter().for_each(|c| walk_node(c, seen, out)),
        Body::Variable { children, .. } => children.iter().for_each(|c| walk_node(c, seen, out)),
        Body::Tr { tree, .. } => {
            if let Some(t) = tree {
                walk_node(t, seen, out);
            }
        }
        _ => {} // KeyArg / MultiKeys / Hash* / Timelock(After) / Empty — no Older children
    }
}
```

Then add to `crates/mnemonic-toolkit/src/main.rs` next to `mod secret_advisory;`:

```rust
mod timelock_advisory;
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test --bin mnemonic timelock_advisory::tests`
Expected: PASS (2 tests). Also `cargo build --bin mnemonic` clean (no unused-import warnings — the adapters are used in later tasks; if a `#[allow(dead_code)]` warning appears before Task 5, it is resolved by the surface hooks; do not silence it permanently).

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/src/timelock_advisory.rs crates/mnemonic-toolkit/src/main.rs
git commit -m "feat(timelock-advisory): shared older() mask predicate + two walk adapters

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: Gate sources bit-math from the shared predicate (byte-identical) + characterization test

**Files:**
- Modify: `crates/mnemonic-toolkit/src/descriptor_builder/gate.rs` (Older arm `~:257-296`; comment `:262`)
- Test: `crates/mnemonic-toolkit/src/descriptor_builder/gate.rs` (`#[cfg(test)]`, near existing `errs(...)` helper `~:704`)

- [ ] **Step 1: Write the failing characterization test** (in gate.rs `#[cfg(test)] mod tests`)

```rust
#[test]
fn gate_still_refuses_masked_older_byte_identical() {
    // build-descriptor must STILL refuse older(65536) with the exact pre-extraction
    // diagnostic (pins zero drift from sourcing the predicate from timelock_advisory).
    // Uses the EXISTING gate-test helpers `older_tree(n)` (gate.rs:930) + `field_diags`
    // (gate.rs:942); cf. the existing `rejects_masked_older_timelocks` test (gate.rs:953).
    let fd = field_diags(&older_tree(65536));
    let msg = fd.iter().map(|d| format!("{d}")).collect::<String>();
    assert!(msg.contains("older(N) encodes a BIP-68 relative timelock"));
    assert!(msg.contains("got 65536 (0x00010000)"));
    assert!(msg.contains("consensus would silently mask this to an effective value of 0 blocks"));
    // Bit-31 no-op branch (IR path reaches bit-31): older(0x80000090) must say "no-op", not a value.
    let nop = field_diags(&older_tree(0x8000_0090));
    let nopmsg = nop.iter().map(|d| format!("{d}")).collect::<String>();
    assert!(nopmsg.contains("no relative timelock at all"));
    assert!(!nopmsg.contains("effective value"));
}
```

> The `older_tree(n)` + `field_diags(...)` helpers and the `rejects_masked_older_timelocks` /
> `accepts_valid_older_block_and_time` tests already exist (`gate.rs:930/942/953/987`). This new test
> pins the EXACT emitted string so the Task-2 extraction is provably byte-identical. The substrings
> above are copied from the current `gate.rs:280-296` format.

- [ ] **Step 2: Run to verify it passes BEFORE the refactor (it is a characterization test of current behavior)**

Run: `cargo test --bin mnemonic gate_still_refuses_masked_older_byte_identical`
Expected: PASS against the CURRENT gate (the string already matches). This locks the baseline. (If it FAILS, fix the asserted substrings to match the current emitted diagnostic verbatim before refactoring.)

- [ ] **Step 3: Refactor the Older arm to source bit-math from the shared predicate**

In `gate.rs`, replace the `PolicyNode::Older(n)` arm body (`~:257-296`) so the predicate decision + the bit-31/unit/effective extraction come from `crate::timelock_advisory::older_consensus_masked`, while the emitted diagnostic STRING stays byte-identical:

```rust
PolicyNode::Older(n) => {
    // BIP-68 relative timelock: only the low 16 bits are the value and bit 22
    // (0x400000) selects 512-second units; consensus masks the operand to
    // 0x0040FFFF, so any other bit (incl. the bit-31 disable flag) is silently
    // dropped and a zero 16-bit value is a no-op lock. This is the JSON-IR
    // authoring gate (build-descriptor); intake/round-trip surfaces get a
    // non-blocking advisory instead (see `timelock_advisory`).
    if let Some(consequence) = crate::timelock_advisory::older_consensus_masked(*n) {
        use crate::timelock_advisory::{TimelockMaskConsequence, TimelockUnit};
        let consequence = match consequence {
            TimelockMaskConsequence::Bit31Disabled =>
                "the bit-31 disable flag is set, so consensus would treat this \
                 CHECKSEQUENCEVERIFY as a no-op — no relative timelock at all"
                    .to_string(),
            TimelockMaskConsequence::Masked { effective, unit } => {
                let unit = match unit {
                    TimelockUnit::Seconds512 => " (512-second units)",
                    TimelockUnit::Blocks => " blocks",
                };
                format!(
                    "consensus would silently mask this to an effective value of {}{}, \
                     weakening or nullifying the timelock",
                    effective, unit
                )
            }
        };
        out.push(field_diag(
            path,
            format!(
                "older(N) encodes a BIP-68 relative timelock: only the low 16 bits are \
                 the value, and bit 22 (0x400000) selects 512-second units. All other \
                 bits — including the bit-31 disable flag — must be clear, and the 16-bit \
                 value must be non-zero. got {n} (0x{n:08x}); {consequence}. Use 1..=65535 \
                 (blocks) or 0x400000|(1..=65535) (512-second units)."
            ),
        ));
    }
}
```

> The format strings above are copied VERBATIM from the current arm (`gate.rs:269-296`). Diff against the original to confirm byte-identity before/after. The only change is the SOURCE of the `if` decision + the `effective`/`unit` values (now from the shared predicate).

- [ ] **Step 4: Run the characterization test + the full gate test module**

Run: `cargo test --bin mnemonic gate`
Expected: PASS — `gate_still_refuses_masked_older_byte_identical` and all pre-existing gate tests (clean-values, refusal) still green.

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/src/descriptor_builder/gate.rs
git commit -m "refactor(gate): source older() bit-math from timelock_advisory (byte-identical diagnostic)

Reword stale :262 'engraving surface' comment; characterization test pins no drift.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: Adapter unit tests (tree bit-31-reachable + descriptor + dedup)

**Files:**
- Test: `crates/mnemonic-toolkit/src/timelock_advisory.rs` (`#[cfg(test)]`)

- [ ] **Step 1: Write the failing adapter tests**

```rust
#[test]
fn adapter_a_tree_reaches_bit31_and_dedups() {
    use md_codec::tag::Tag;
    use md_codec::tree::{Body, Node};
    // Hand-built tree: andor(<older(0x80000001)>, <older(65536)>, <older(65536)>)
    // mirrors a crafted md1 card (md_codec decode does NO operand validation).
    let older = |v: u32| Node { tag: Tag::Older, body: Body::Timelock(v) };
    let root = Node {
        tag: Tag::AndOr,
        body: Body::Children(vec![older(0x8000_0001), older(65536), older(65536)]),
    };
    // Walk the Node directly — no full md_codec::Descriptor literal needed (R0 advisor #2).
    let advs = older_advisories_node(&root);
    // bit-31 IS reachable on the A-raw-card path; 65536 deduped to one entry.
    assert_eq!(advs.len(), 2);
    assert!(advs.iter().any(|a| a.consequence == TimelockMaskConsequence::Bit31Disabled));
    assert!(advs.iter().any(|a| a.operand == 65536));
}

#[test]
fn adapter_b_descriptor_wsh_collects_masked_only() {
    use miniscript::Descriptor;
    use std::str::FromStr;
    // wsh(andor(pk(K0),older(65536),and_v(v:pk(K1),older(2016)))) — 65536 masked, 2016 clean.
    let k0 = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
    let k1 = "03f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9";
    let d = Descriptor::<miniscript::bitcoin::PublicKey>::from_str(&format!(
        "wsh(andor(pk({k0}),older(65536),and_v(v:pk({k1}),older(2016))))"
    )).expect("masked policy parses");
    let advs = older_advisories_descriptor(&d);
    assert_eq!(advs.len(), 1, "only older(65536) is masked; older(2016) is clean");
    assert_eq!(advs[0].operand, 65536);
}
```

> `adapter_a` walks a hand-built `Node` via `older_advisories_node` — no `md_codec::Descriptor` literal, so no `PathDecl`/`UseSitePath`/`TlvSection` construction. The Adapter-B test's key-type `bitcoin::PublicKey` avoids xpub/derivation; any `MiniscriptKey` works.

- [ ] **Step 2: Run to verify it fails/compiles-then-asserts**

Run: `cargo test --bin mnemonic timelock_advisory::tests::adapter`
Expected: initially may FAIL to compile if the `md_codec::Descriptor` literal is wrong — fix field construction until it compiles, then both assertions PASS.

- [ ] **Step 3: (no new impl — adapters already written in Task 1)**

If a test reveals a walk bug (e.g., missing a child-bearing `Body` variant), fix `walk_node`/`older_advisories_descriptor` minimally.

- [ ] **Step 4: Run to verify pass**

Run: `cargo test --bin mnemonic timelock_advisory::tests`
Expected: PASS (4 tests total).

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/src/timelock_advisory.rs
git commit -m "test(timelock-advisory): adapter A (bit-31 reachable) + adapter B + dedup

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Tasks 4–10: Per-surface hooks

> **Shared pattern.** Each surface, at the cited post-parse point, adds exactly:
> ```rust
> let _adv = crate::timelock_advisory::older_advisories_<tree|descriptor|ms>(<parsed>);
> crate::timelock_advisory::emit_advisories(&_adv, stderr);
> ```
> Adapter A surfaces pass an `&md_codec::Descriptor` (`older_advisories_tree`); Adapter B descriptor surfaces pass `&Descriptor<_>` (`older_advisories_descriptor`); compare-cost passes `&Miniscript` (`older_advisories_ms`).
> **Each surface test** uses `assert_cmd::Command::cargo_bin("mnemonic")`, modeled on the surface's existing `tests/cli_<surface>.rs`, asserting `stderr` **contains** `"advisory: older(65536) is consensus-masked"` AND exit success; plus a clean-input case asserting stderr does NOT contain `"advisory: older"`. The masked policy is `wsh(andor(pk(<K0>),older(65536),and_v(v:pk(<K1>),older(2016))))` (use the key format the existing test for that surface uses).

### Task 4: `import-wallet`

**Files:** Modify `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` (`~:1285`); Test: `crates/mnemonic-toolkit/tests/cli_import_wallet*.rs` (model an existing import test).

- [ ] **Step 1:** Failing integration test — import a (bitcoin-core/descriptor) wallet whose descriptor carries `older(65536)`; assert stderr contains the advisory + exit 0. (Model: `tests/cli_bundle_import_json.rs` or the closest `cli_import_*` test.)
- [ ] **Step 2:** Run → FAIL (no advisory yet).
- [ ] **Step 3:** Insert after the select line (`~:1285`, before `// Emit stdout.` `~:1287`):
```rust
    for p in &parsed {
        let adv = crate::timelock_advisory::older_advisories_tree(&p.descriptor);
        crate::timelock_advisory::emit_advisories(&adv, stderr);
    }
```
- [ ] **Step 4:** Run → PASS. Add + run the clean-input case (no advisory).
- [ ] **Step 5:** Commit (`feat(import-wallet): masked older() advisory`).

### Task 5: `bundle`

**Files:** Modify `crates/mnemonic-toolkit/src/cmd/bundle.rs` (`~:1662` and `~:1953`); Test: `crates/mnemonic-toolkit/tests/cli_bundle_full.rs` (+ `cli_bundle_import_json.rs` for the import-json site).

- [ ] **Step 1:** Failing tests: (a) `bundle --descriptor wsh(andor(...older(65536)...))` → stderr advisory; (b) `bundle --import-json <envelope with masked descriptor>` → stderr advisory. (Model: `cli_bundle_full.rs`, `cli_bundle_import_json.rs`, and the existing `cli_bundle_language_advisory.rs` for the stderr-assert idiom.)
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3:** Site 1 — after `emit_default_path_notice(...)` (`~:1662`), before `emit_unified(` (`~:1664`):
```rust
    let adv = crate::timelock_advisory::older_advisories_tree(&descriptor);
    crate::timelock_advisory::emit_advisories(&adv, stderr);
```
Site 2 — after `synthesize_descriptor(...)` (`~:1953`), before the `emit_unified` at `~:1978`:
```rust
    let adv = crate::timelock_advisory::older_advisories_tree(&descriptor);
    crate::timelock_advisory::emit_advisories(&adv, stderr);
```
- [ ] **Step 4:** Run → PASS (both sites). Clean-input case → no advisory.
- [ ] **Step 5:** Commit (`feat(bundle): masked older() advisory on --descriptor and --import-json`).

### Task 6: `verify-bundle --descriptor`

**Files:** Modify `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` (`~:1028`); Test: `crates/mnemonic-toolkit/tests/cli_bundle_multisig.rs` or `cli_verify_*`.

- [ ] **Step 1:** Failing test: `verify-bundle --descriptor wsh(andor(...older(65536)...)) --md1 ... --mk1 ...` → stderr advisory + `result: ok`. (Generate the matching md1/mk1 cards via `bundle` on the same masked descriptor in a setup step.)
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3:** After the `is_non_canonical` path-decl block (`~:1028`), before `verify_emit_from_expected(` (`~:1029`):
```rust
    let adv = crate::timelock_advisory::older_advisories_tree(&descriptor);
    crate::timelock_advisory::emit_advisories(&adv, stderr);
```
- [ ] **Step 4:** Run → PASS. Clean-input case → no advisory.
- [ ] **Step 5:** Commit (`feat(verify-bundle): masked older() advisory on --descriptor`).

### Task 7: `restore --md1`

**Files:** Modify `crates/mnemonic-toolkit/src/cmd/restore.rs` (`~:1291`); Test: `crates/mnemonic-toolkit/tests/cli_restore*.rs`.

- [ ] **Step 1:** Failing test: generate an md1 card from the masked descriptor (via `bundle`), then `restore --md1 <card> --format descriptor` → stderr advisory + exit 0, descriptor printed verbatim. (Model: `cli_restore_taproot.rs`.)
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3:** After the Display-fidelity guard (`~:1291`), before `derive_receive_addresses(` (`~:1292`):
```rust
    let adv = crate::timelock_advisory::older_advisories_descriptor(&parsed);
    crate::timelock_advisory::emit_advisories(&adv, stderr);
```
(`parsed: MsDescriptor<DescriptorPublicKey>`; Adapter B, fail-closed on bit-31 — `from_str` at `:1277` rejects bit-31/zero cards before this point.)
- [ ] **Step 4:** Run → PASS. Clean-input case → no advisory.
- [ ] **Step 5:** Commit (`feat(restore): masked older() advisory on --md1 round-trip`).

### Task 8: `compare-cost` (signature change + both invocations)

**Files:** Modify `crates/mnemonic-toolkit/src/cost/mod.rs` (`run_compare_cost` sig + hook `~:136`), `crates/mnemonic-toolkit/src/cmd/compare_cost.rs` (`run` sig), `crates/mnemonic-toolkit/src/main.rs` (`:198`); Test: `crates/mnemonic-toolkit/tests/cli_compare_cost.rs`.

- [ ] **Step 1:** Failing tests (both invocations): `compare-cost --descriptor wsh(andor(...older(65536)...))` AND `compare-cost --miniscript andor(...older(65536)...)` → each emits the advisory on stderr + exit 0. (Model: `cli_compare_cost.rs`.)
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3a:** Add `stderr` param to `run_compare_cost` (`cost/mod.rs:123`):
```rust
pub fn run_compare_cost<W: std::io::Write, E: std::io::Write>(
    args: &CompareCostArgs,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
```
Insert after the dispatch `match` closes (`~:136`, before `let original_input`):
```rust
    let adv = crate::timelock_advisory::older_advisories_ms(&translated.segv0);
    crate::timelock_advisory::emit_advisories(&adv, stderr);
```
**Step 3b:** Add `stderr` to `cmd::compare_cost::run` (`compare_cost.rs:67`) and forward it:
```rust
pub fn run<R: Read, W: Write, E: Write>(
    args: &CompareCostArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    // ... existing body, change the run_compare_cost call to:
    crate::cost::run_compare_cost(args, stdout, stderr)
}
```
**Step 3c:** `main.rs:198`:
```rust
Command::CompareCost(args) => cmd::compare_cost::run(args, stdin, stdout, stderr).map(|_| 0),
```
**Step 3d:** Update in-crate callers of `run_compare_cost` (search `run_compare_cost(`). TWO are
**production** call sites inside `build-descriptor` (`build_descriptor.rs:500` and `:530`
`cost_preview_value`). These get a **deliberately discarding** sink WITH a comment — the advisory is
unreachable there because `gate::validate_with_allow` (`build_descriptor.rs:287`/`:326`) hard-refuses
masked `older()` at field-validate (step 1) BEFORE cost preview (`:424`), so a masked operand never
reaches the cost pipeline (advisor must-fix #1):
```rust
    // build-descriptor's gate already refused masked older() upstream (validate_with_allow,
    // step-1 field-validate) before cost preview runs, so no advisory can fire here; discard.
    let mut _discard_advisory = std::io::sink();
    cost::run_compare_cost(&cost_args, /* stdout */ &mut buf, &mut _discard_advisory)?;
```
(Adjust to each call site's existing stdout target.) Any TEST callers of `run_compare_cost` /
`compare_cost::run` pass `&mut Vec::<u8>::new()` (or the test's captured stderr).
- [ ] **Step 4:** Run → PASS (both invocations). Clean-input case (both flags) → no advisory. Run `cargo build --bin mnemonic` to catch missed call sites.
- [ ] **Step 5:** Commit (`feat(compare-cost): masked older() advisory (--descriptor + --miniscript; threads stderr)`).

### Task 9: `export-wallet`

**Files:** Modify `crates/mnemonic-toolkit/src/cmd/export_wallet.rs` (`~:452` and `~:721`); Test: `crates/mnemonic-toolkit/tests/cli_export_wallet*.rs`.

- [ ] **Step 1:** Failing tests: (a) `export-wallet --descriptor wsh(andor(...older(65536)...)) --format descriptor` → stderr advisory; (b) the `--from-import-json` path with a masked descriptor → stderr advisory (fires even if a later refuse occurs). (Model: existing `cli_export_wallet*` tests.)
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3:** Site 1 — `--descriptor` path (`~:452`), after the `from_str` binds `d`, before `d.to_string()`:
```rust
        let adv = crate::timelock_advisory::older_advisories_descriptor(&d);
        crate::timelock_advisory::emit_advisories(&adv, stderr);
        d.to_string()
```
Site 2 — `run_from_import_json` after `script_type_from_descriptor(&parsed_ms)` (`~:721`), BEFORE the taproot refuse (`~:723`):
```rust
        let adv = crate::timelock_advisory::older_advisories_descriptor(&parsed_ms);
        crate::timelock_advisory::emit_advisories(&adv, stderr);
```
- [ ] **Step 4:** Run → PASS. Clean-input case → no advisory.
- [ ] **Step 5:** Commit (`feat(export-wallet): masked older() advisory on --descriptor and --from-import-json`).

### Task 10: `xpub-search` (both funnels; `parse_md1` gains stderr; A-raw-card bit-31 test)

**Files:** Modify `crates/mnemonic-toolkit/src/cmd/xpub_search/descriptor_intake.rs` (`~:290` literal funnel; `parse_md1` sig + `~:227`; `intake_from_shape` dispatch `:141`); Test: `crates/mnemonic-toolkit/tests/cli_xpub_search*.rs` + new A-raw-card case.

- [ ] **Step 1:** Failing tests: (a) `xpub-search --descriptor wsh(andor(...older(65536)...))` (literal funnel) → stderr advisory; (b) **A-raw-card wiring**: generate a real `md1` card from the masked descriptor via `bundle --descriptor wsh(andor(...older(65536)...))` (a setup step — `older(65536)` is bit-31-clear so it parses + encodes normally), feed that card to `xpub-search` (md1-card funnel) → stderr emits the `Masked` advisory + exit 0. This proves the md1-card→`older_advisories_tree` hook wiring. **Bit-31 reachability on this path is already proven by the Task-3 `adapter_a_tree_reaches_bit31_and_dedups` unit test** — a crafted bit-31 card is adversarial-only (no descriptor string yields it; advisor §3), so the fragile hand-crafted-card end-to-end cell is intentionally omitted.
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3a:** Literal funnel — after `from_str` binds `parsed` (`~:290`), before the cosigner loop (`~:291`):
```rust
    let adv = crate::timelock_advisory::older_advisories_descriptor(&parsed);
    crate::timelock_advisory::emit_advisories(&adv, stderr);
```
(`stderr: &mut impl Write` already a `parse_literal_xpub` param.)
**Step 3b:** Add `stderr` to `parse_md1`:
```rust
fn parse_md1(payload: &str, stderr: &mut impl std::io::Write) -> Result<DescriptorIntake, ToolkitError> {
```
and at the dispatch (`descriptor_intake.rs:141`): `DescriptorShape::Md1 => parse_md1(payload, stderr),`.
**Step 3c:** In `parse_md1`, after the `n == 0` guard (`~:227`), before the per-slot resolver (`~:228`):
```rust
    let adv = crate::timelock_advisory::older_advisories_tree(&desc);
    crate::timelock_advisory::emit_advisories(&adv, stderr);
```
(A-raw-card: bit-31 reachable; `older_advisories_tree` has NO debug_assert — correct.)
- [ ] **Step 4:** Run → PASS (both funnels; md1-card funnel emits the `Masked` advisory). Clean-input case → no advisory.
- [ ] **Step 5:** Commit (`feat(xpub-search): masked older() advisory on both descriptor and md1-card funnels`).

---

## Task 11: Cross-surface + non-blocking regression test

**Files:** Create `crates/mnemonic-toolkit/tests/cli_older_advisory.rs`

- [ ] **Step 1:** Add a focused test file asserting (via `assert_cmd`):
  - **Fires + non-blocking:** for the canonical masked policy, the advisory fires on at least
    `compare-cost`, `bundle`, `export-wallet` (representative of A/B), substring
    `"older(65536) is consensus-masked"`, and every surface still **exits 0**.
  - **Clean-512s false-positive guard (advisor #4):** a descriptor with a clean 512-second-unit
    timelock `older(4194305)` (`0x400001`) → stderr does NOT contain `"advisory: older"` (guards the
    most likely false-positive: a fat-fingered bit-22 mask).
  - **Dedup is operand-keyed (advisor #6):** `wsh(andor(pk(K0),older(65536),and_v(v:pk(K1),older(65536))))`
    (same literal twice) → the advisory line appears **exactly once**; but two DISTINCT masked literals
    `older(65536)` + `older(131072)` (both mask-to-0) → **two** advisory lines (distinct operands kept).
  - **`--json` stdout cleanliness (advisor #5):** for a `--json` invocation of `compare-cost`
    (and/or `export-wallet`/`bundle`), the advisory appears on **stderr** and the **stdout JSON does NOT
    contain** `"advisory: older"` (guards a future regression that inlines the advisory into stdout).
- [ ] **Step 2:** Run → expect PASS (hooks already in place from Tasks 4–10).
- [ ] **Step 3:** (no impl) — if any surface fails, fix that surface's hook.
- [ ] **Step 4:** `cargo test --test cli_older_advisory` → PASS.
- [ ] **Step 5:** Commit (`test: cross-surface masked-older() advisory + non-blocking + dedup`).

---

## Task 12: Locksteps — manual prose + FOLLOWUPS

**Files:** Modify `docs/manual/src/40-cli-reference/41-mnemonic.md`; `design/FOLLOWUPS.md`.

- [ ] **Step 1:** Add ONE shared "consensus-masked `older()` advisory" paragraph to `41-mnemonic.md` and cross-reference it from the 7 subcommand sections: `bundle` (`~:45`), `verify-bundle` (`~:518`), `export-wallet` (`~:686`), `restore` (`~:734`), `import-wallet` (`~:1019`), `xpub-search` (`~:3159`), `compare-cost` (`~:3693`). Paragraph text (no new flags — describe behavior):
> **Consensus-masked relative timelocks.** If a descriptor's `older(N)` value is BIP-68 consensus-masked (stray bits, or a zero 16-bit value such as `older(65536)`), this command prints a non-blocking advisory to stderr noting the effective (weaker) value. The command still succeeds — it never refuses to back up or inspect an already-deployed wallet.
- [ ] **Step 2:** Run the FULL manual lint (NOT just flag-coverage — v0.50.0 cspell lesson):
```bash
make -C docs/manual lint MNEMONIC_BIN=$(pwd)/target/debug/mnemonic MD_BIN=... MS_BIN=... MK_BIN=...
```
Expected: PASS (add any new vocabulary — e.g. `older`, `BIP-68` — to the manual `.cspell` allowlist if cspell flags it).
- [ ] **Step 3:** Update `design/FOLLOWUPS.md`: (a) entry `intake-surfaces-accept-masked-older-no-advisory` — extend the **Where** line (`:140`) from the 4 surfaces to all 7 (`bundle --descriptor`, `export-wallet --descriptor`, `import-wallet`, `xpub-search`, `verify-bundle --descriptor`, `restore --md1`, `compare-cost`), and mark **RESOLVED (this cycle)** with SPEC + R0-report + advisor references. (b) **File a new FOLLOWUP** `older-advisory-blindness-suppression` (advisor #7, tier deferred): the advisory fires on every intake of an already-known-masked deployed wallet, every surface, every run, unsuppressable → habituation/advisory-blindness risk. Future option: a `--quiet-advisories` flag (would be MINOR + schema_mirror + manual locksteps). Do NOT build now; record the rationale so it isn't re-discovered.
- [ ] **Step 4:** Re-run manual lint to confirm green.
- [ ] **Step 5:** Commit (`docs(manual,followups): masked older() advisory prose (7 sections) + resolve FOLLOWUP`).

---

## Task 13: Full verification + branch finish

- [ ] **Step 1:** `cargo test --bin mnemonic` (unit) and `cargo test` (all integration) → ALL PASS.
- [ ] **Step 2:** `cargo clippy --all-targets -- -D warnings` → clean (do NOT run `cargo fmt --all`; mlock.rs is fmt-exempt — see memory g6). If a fmt gate exists, run the project's fmt check command, NOT `cargo fmt --all`.
- [ ] **Step 3:** `cargo build --bin mnemonic` release-clean; manually run one surface to eyeball the advisory:
```bash
target/debug/mnemonic compare-cost --descriptor 'wsh(andor(pk(0279be...),older(65536),and_v(v:pk(03f9...),older(2016))))'
```
Expected: stderr shows the `older(65536)` advisory; stdout shows the cost table; exit 0.
- [ ] **Step 4:** Confirm SemVer PATCH: `git diff master --stat` shows no clap flag additions; `cargo test --test schema_mirror` (or the GUI schema test) unaffected. Bump the toolkit version (PATCH) per the repo's release ritual if this cycle ships standalone.
- [ ] **Step 5:** Per CLAUDE.md, run the per-cycle architect review on the implementation before tag (separate from this plan's R0). Then finish the branch (PR or merge per `superpowers:finishing-a-development-branch`).

---

## Self-Review (filled at write time)

**Spec coverage:** §3.1 predicate → Task 1; §3.1 module placement → Task 1 (main.rs); §3.2 Adapter A → Task 1 + Task 3; §3.2 Adapter B (generic core + descriptor unwrap + compare-cost ms) → Task 1 + Task 3 + Task 8; §3.3 three regimes / bit-31 → Task 1 (no assert on tree; assert on ms) + Task 10 (A-raw-card test); §4 all 7 surfaces → Tasks 4–10 (import, bundle×2, verify, restore, compare-cost×2, export×2, xpub×2); §5 message forms → Task 1; §6 tests (predicate table, gate-still-refuses, per-surface, A-raw-card, dedup) → Tasks 1,2,3,4–10,11; §7 PATCH/manual/FOLLOWUPS/gate-comment → Task 2 (comment) + Task 12 + Task 13. No gaps.

**Placeholder scan:** The only deferred specifics are (a) the exact `SpecDoc` JSON in Task 2 Step 1 and (b) per-surface argv — both explicitly directed to the existing test that already encodes the working shape (gate tests near `:990`; each `tests/cli_<surface>.rs`). These are "use the existing working fixture", not invented placeholders.

**Type consistency:** `older_consensus_masked(u32) -> Option<TimelockMaskConsequence>`; `TimelockMaskConsequence::{Bit31Disabled, Masked{effective:u16, unit:TimelockUnit}}`; `TimelockAdvisory{operand:u32, consequence}`; collectors `older_advisories_{tree(&md_codec::Descriptor), descriptor(&Descriptor<Pk>), ms(&Miniscript<Pk,Ctx>)} -> Vec<TimelockAdvisory>`; `emit_advisories(&[TimelockAdvisory], &mut E)`. Names consistent across all tasks.
