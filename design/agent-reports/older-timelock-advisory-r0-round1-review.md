# R0 round 1 — architect review (verbatim) — SPEC_older_timelock_advisory.md

> Persisted verbatim per CLAUDE.md ("persist the full review-agent output before applying folds").
> Dispatched via Agent tool (feature-dev:code-architect, inherited session default model Opus 4.8;
> the review body's self-attribution line is the agent's own and is left as written).
> Verdict: **YELLOW** (0 Critical, 3 Important, 5 Minor). Source SHA `3235431`.

---

## R0 REVIEW — SPEC_older_timelock_advisory.md — Round 1

**Reviewer:** Claude Sonnet 4.6 (Fable 5 per standing preferences)
**Source SHA verified:** `3235431`
**Date:** 2026-06-12

---

## Critical

None found.

---

## Important

### I1 — Citation Drift: `tree.rs:169-172` does NOT contain the Older decode arm

The spec (§3.3) cites "md-codec tree.rs:169-172" as the location of `Tag::After | Tag::Older => { let v = r.read_bits(32)? as u32; Body::Timelock(v) }`.

Grep-verified against `/scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec/src/tree.rs`:

- Lines 169-172 contain Hash160Body WRITE code (`for byte in h { w.write_bits(u64::from(*byte), 8); }`).
- The Older/After READ decode is at lines **293-295**: `Tag::After | Tag::Older => { let v = r.read_bits(32)? as u32; Body::Timelock(v) }`.

This is an Important finding because the spec's citation underpins the entire §3.3 "A-raw-card REACHABLE" argument: if the wrong lines are cited, reviewers cannot verify the claim that md-codec performs no operand validation on decode. The actual code at 293-295 does confirm no validation — `r.read_bits(32)? as u32` with no bounds check — but the spec must cite the correct lines.

**Fix:** In §3.3 (and anywhere the decode arm is cited), replace `tree.rs:169-172` with `tree.rs:293-295`.

---

### I2 — `compare-cost --miniscript` input path uncited: spec covers only `--descriptor`

The spec's §4 table for surface 5 (`compare-cost`) cites only `cost/strip.rs:21` (the `--descriptor` path). `compare-cost` has TWO parse entry points:

- `--descriptor` → `strip::translate_descriptor` → `Descriptor::<DescriptorPublicKey>::from_str(input)` at `cost/strip.rs:21`
- `--miniscript` → `translate::translate_miniscript` → `Miniscript::from_str(&segv0_input)` at `cost/translate.rs:82` and `Miniscript::from_str(&tap_input)` at `cost/translate.rs:84`

Both paths go through miniscript's `TryFrom<Sequence>` validation (Adapter B, bit-31 unreachable), so both need the advisory to fire. The `run_compare_cost` function at `cost/mod.rs:123-135` dispatches between them based on `args.input`. If a masked `older()` is supplied via `--miniscript`, no advisory fires under the current spec. This is a coverage gap.

The spec's §4 cite of only `cost/strip.rs:21` is incomplete for `compare-cost`. The implementation plan must add the advisory call at or after the `translate_miniscript` path at `cost/mod.rs:130` as well.

**Fix:** Update the §4 table row for `compare-cost` to cite BOTH `cost/strip.rs:21` AND `cost/translate.rs:82` (or the dispatch site `cost/mod.rs:130`). Note that Adapter B applies to both. The implementation adds the advisory walk on the Translated result (which holds the parsed miniscript), not at the from_str site.

Note: Both paths are Adapter B (bit-31 unreachable), so no change to the regime table is needed — only the citation and implementation coverage need updating.

---

### I3 — Walk-input mechanism for Adapter A deferred but must be decided before implementation

The spec §4 explicitly defers the walk-input mechanism choice (direct `MdDescriptor.tree` walk vs `parse_descriptor` return-type extension) to "the implementation plan + R0." This R0 must adjudicate it to prevent implementation variance.

The two options are:
- **(i)** Walk `MdDescriptor.tree` directly at each Adapter-A call site after `parse_descriptor` returns. Cost: ~6 walk calls (one per A-surface call site). No change to `parse_descriptor`'s return type. Adapter A lives entirely in `timelock_advisory.rs`.
- **(ii)** Modify `parse_descriptor` to return `(MdDescriptor, Vec<TimelockAdvisory>)`. Cost: breaks 8+ callers in `wallet_import/*`, `bundle.rs`, `verify_bundle.rs`. Massive churn.

Option (i) is clearly minimal-churn and correctly places the advisory logic in `timelock_advisory.rs` as the spec intends. Option (ii) is explicitly disproportionate.

The R0 must formally pick option (i) and the spec must state the choice before implementation.

**Fix:** Add to §4 (walk-input mechanism disposition): "Walk-input mechanism: Adapter-A sites use direct `MdDescriptor.tree` walk in `timelock_advisory.rs`. `parse_descriptor`'s return type is UNCHANGED. Each Adapter-A surface calls `walk_md_tree(&descriptor.tree)` after its existing `parse_descriptor` call."

---

## Minor

### m1 — `TimelockMaskConsequence` needs `PartialEq` for `debug_assert!`

The spec §3.3 says "a `debug_assert!(consequence != Bit31Disabled)` is valid ONLY at Adapter-B and A-post-from_str call sites." For this assertion to compile, `TimelockMaskConsequence` must derive or implement `PartialEq`. The spec's type definition in §3.1 does not include `#[derive(PartialEq)]` (or `Debug`, `Eq`). The implementer will discover this at compile time, but the spec should specify the required derives to avoid ambiguity.

**Fix:** In §3.1, annotate the enum with `#[derive(Debug, PartialEq, Eq)]`.

### m2 — §8.5 is resolved by this R0: add explicit disposition

The spec lists §8.5 as an open item: "trace whether any `verify-bundle --md1` card-only path (no `--descriptor`) independently walks the decoded card's tree for `older()` — if so it is A-raw-card (needs bit-31 handling + a test cell); if it only checks card identity / never surfaces the card's `older()`, document it explicitly out of scope."

This R0 has traced the path. `verify-bundle --md1` card-only (template mode, no `--descriptor`) goes through `run_multisig` → `emit_verify_checks` → `emit_md1_checks`. That function checks: decode success, wallet_policy mode, xpub match. It never traverses the policy tree or surfaces any `older()` content to the user. The `emit_watch_only_xpub_path_cross_check` function similarly only accesses path/fingerprint metadata.

Verdict: **OUT OF SCOPE**. `verify-bundle --md1` card-only does not surface `older()` content and needs no advisory.

**Fix:** Close §8.5 in the spec: "Resolved (R0-r1): `verify-bundle --md1` card-only (template mode, no `--descriptor`) — `run_multisig` → `emit_md1_checks` checks decode/wallet_policy/xpub_match only; the policy tree is never traversed for `older()` content; this path is out of scope for the advisory."

### m3 — §7 manual-lockstep omits the file name `41-mnemonic.md`

The spec §7 says "under `docs/manual/src/40-cli-reference/`" but does not name the specific file. All four CLI tools have separate files. The affected file is `41-mnemonic.md` (the `mnemonic` CLI manual). The implementer should not have to guess.

**Fix:** Change "under `docs/manual/src/40-cli-reference/` (the `bundle` + `restore` sections)" to "`docs/manual/src/40-cli-reference/41-mnemonic.md` (under the `bundle`, `restore`, and other affected subcommand sections)."

### m4 — §3.1 pub vs mod visibility of `older_consensus_masked` not specified

The spec declares `pub fn older_consensus_masked(n: u32) -> Option<TimelockMaskConsequence>` but doesn't specify whether the module is `pub mod timelock_advisory` (lib-exported) or `mod timelock_advisory` (bin-crate internal). Given the consumers are cmd/ modules (bin-crate), this can be `pub(crate)` or simply `pub` within a `mod` declared in `main.rs`. However, if future lib consumers (e.g., `mnemonic-gui` importing toolkit lib) need the predicate, it should go in `lib.rs`. For this PATCH cycle, bin-private is sufficient.

**Fix:** Spec should state: "Register `timelock_advisory` in `main.rs` as `mod timelock_advisory` (bin-crate internal; no lib.rs change required for this cycle)." If lib promotion is ever needed it is a separate MINOR.

### m5 — `older(0)` regime classification needs one sentence in §3.3

The spec discusses `older(0x80000001)` (bit-31) and `older(65536)` (stray bits) but does not explicitly classify `older(0)` in the three-regime table. `older(0) = Sequence::ZERO` is rejected by `TryFrom<Sequence>` (the `seq != Sequence::ZERO` check) at both the Adapter-B and A-post-from_str boundaries. For A-raw-card, `older(0)` in the md1 tree would return `Masked { effective: 0, unit: Blocks }` (predicate: `(0 & 0xFFBF0000) = 0` — false; `(0 & 0xFFFF) = 0` — TRUE). So A-raw-card correctly classifies `older(0)` as `Masked{0, Blocks}`, not `Bit31Disabled`. The §6 A-raw-card test already tests `older(0)` and lists `Masked{0}`. This is consistent. But §3.3 does not mention `older(0)` explicitly.

**Fix:** Add one sentence to §3.3: "`older(0)`: bit-31 clear, low-16 zero → `Masked { effective: 0, unit: Blocks }` (A-raw-card reachable; A-post-from_str and Adapter B unreachable via `Sequence::ZERO` check)."

---

## Settled Items (No Finding)

The following were examined and are CORRECT:

1. **gate.rs citations:** `:257` (PolicyNode::Older arm entry), `:262-264` (predicate), `:269-285` (consequence branches), `:298` (After arm), `:990` (clean values), `:161-178` (validate_with_allow step 1 flow) — all verified.

2. **parse_descriptor.rs citations:** `:633` (Terminal::Older arm), `:748` (parse_descriptor fn signature), `:780` (from_str call) — all verified.

3. **wallet_import/* 8 lines:** `bitcoin_core.rs:278`, `pipeline.rs:322`, `bsms.rs:227`, `sparrow.rs:419`, `specter.rs:234`, `coldcard.rs:308`, `coldcard_multisig.rs:463`, `electrum.rs:377` — all verified at correct lines.

4. **bundle.rs:1228, 1603, 1936** — all confirmed as `parse_descriptor` call sites for `--descriptor`, `--descriptor-file`, and `--import-json` respectively.

5. **verify_bundle.rs:709, 1017** — both confirmed as `parse_descriptor` call sites in `descriptor_mode_verify_run`.

6. **restore.rs:833, 1277** — both confirmed as `from_str` re-parse sites.

7. **cost/strip.rs:21** — confirmed as `Descriptor::from_str(input)`.

8. **export_wallet.rs:452, 566, 715** — all confirmed as `from_str` parse sites.

9. **descriptor_intake.rs:140-215, 289** — 140 is the dispatch, 210-279 is `parse_md1`, 289 is `from_str` in `parse_literal_xpub`. The range `:140-215` is a loose but non-wrong citation covering the dispatch and the beginning of `parse_md1`.

10. **md-codec encode.rs:25** — `pub tree: Node,` confirmed.

11. **tree.rs:9-52** — Node struct (line 9) and Body enum (line 17) through Tr variant (line 49). The spec says "Node {tag, body}" lives here — CORRECT.

12. **tree.rs:49** — `Tr {` variant — CORRECT.

13. **miniscript relative_locktime.rs:70-80** — TryFrom<Sequence> for RelLockTime — CORRECT. `to_consensus_u32` at :48 — CORRECT.

14. **Bit-math correctness** — predicate `(n & !0x0040_FFFF) != 0 || (n & 0x0000_FFFF) == 0` correctly classifies all edge cases. `effective==0` and 512-second-unit boundaries verified. `Bit31Disabled` variant correctly assigned to `n & 0x8000_0000 != 0` regime. No false positives or false negatives found.

15. **PATCH SemVer** — zero clap delta → no `schema_mirror` impact. CORRECT.

16. **FOLLOWUPS.md:140** — the Where line of `intake-surfaces-accept-masked-older-no-advisory` — CORRECT.

17. **§2 decision (advisory-only)** — correct funds-safety rationale; `bundle` dual-nature correctly handled; no refuse-unless-allow (would block mid-recovery backup). CORRECT.

18. **§5 `after()` exclusion** — `gate.rs:298` After arm is range-only, no BIP-68-style mask. `from_str` rejects `after(N > 0x7FFFFFFF)`. No silent mask for absolute timelocks. Advisory correctly excludes `after()`. CORRECT.

19. **§3.3 three-regime analysis** — Adapter-B surfaces (from_str) unreachable for bit-31/zero. A-post-from_str (parse_descriptor path) likewise unreachable. A-raw-card (xpub-search md1 funnel, inspect) reachable — but inspect doesn't surface `older()` to user. `restore --md1` is fail-closed via `to_miniscript::from_consensus` for bit-31/zero; stray-bits (65536 etc.) survive to from_str → Adapter B advisory fires. CORRECT.

20. **Surface completeness (7 surfaces)** — `inspect` (Md1 output = placeholder_count/tree_tag/wallet_policy/path_decl only, no `older()` content surfaced), `repair` (no descriptor intake), `addresses` (seed sources only), `convert` (no older() processing), `decode-address` (address parsing only) — all correctly excluded. No 8th intake surface found that was missed.

21. **Architecture layering** — `timelock_advisory.rs` at crate root (not inside descriptor_builder/) is correct to avoid cmd/ → descriptor_builder/ dependency. gate.rs importing from a sibling crate-root module is standard Rust. CORRECT.

22. **§6 test adequacy** — predicate unit table, gate-still-refuses characterization (byte-identical pin), per-surface integration (masked fires, clean silent, exit-0), A-raw-card bit-31/zero cell. Taproot tapscript `older()`: general taproot descriptors with `older()` are handled by both adapters — a `wsh(andor(pk,older,v:pk))` fires advisory via Adapter A/B identically. Nested/threshold `older()`: tree-walk recursion handles these. Dedup-across-different-units: spec dedupes by operand value, not by unit — a descriptor with `older(65536)` in two branches emits one advisory. CORRECT.

---

## Verdict

**YELLOW** — Two Important findings (I1 citation drift, I2 compare-cost --miniscript gap) and one Important deferred-but-required decision (I3 walk-input mechanism) must be resolved before implementation. Four Minors (m1-m5) should be folded for spec quality but do not block implementation.

The spec is architecturally sound, the bit-math is correct, the three-regime analysis is valid (modulo the line-number citation error in I1), and the funds-safety position is correct. §8.5 is resolved by this review as out-of-scope.

**Specific fold actions required before re-dispatch:**

1. **[I1]** Replace `tree.rs:169-172` with `tree.rs:293-295` everywhere in the spec.

2. **[I2]** Add `cost/translate.rs:82,84` (or `cost/mod.rs:130`) as a second cite for the `compare-cost` surface in the §4 table. Note that the Adapter-B advisory walk applies to the parsed `Translated` result's miniscript AST (post-parse). Update the §4 table and the implementation scope.

3. **[I3]** Add a definitive disposition to §4 (or new §4.1): "Walk-input for Adapter-A surfaces: direct `MdDescriptor.tree` walk inside `timelock_advisory.rs`; `parse_descriptor` return type is UNCHANGED." Close the deferred item.

4. **[I4]** Expand §7's manual lockstep from "bundle + restore sections" to all 7 affected subcommand sections (or a cross-referenced shared paragraph). Name the file `41-mnemonic.md`.

5. **[m1-m5]** Optional quality folds: derive `PartialEq+Eq+Debug` on the enum; close §8.5 with a resolved disposition; name the file `41-mnemonic.md` in the inline spec; clarify `mod` vs `pub mod` registration; add `older(0)` regime classification sentence.
