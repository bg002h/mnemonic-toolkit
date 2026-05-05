# IMPLEMENTATION PLAN: mnemonic-toolkit v0.3 — descriptor passthrough

**Date:** 2026-05-05
**Convention:** TDD-first per phase; per-phase architect-review to 0C/0I (max r4 then escalate); per-phase reports persist to `design/agent-reports/phase-X-*.md`.
**Status:** approved 0C / 0I after iterative architect-review (rounds 1–3); see Revision history.

## Context

This plan executes `SPEC_mnemonic_toolkit_v0_3.md`. Five implementation phases A–E mirror v0.2's shape. Sub-skill: `superpowers:executing-plans` reads this plan in the next conversation; per-phase TDD discipline per `superpowers:test-driven-development`.

## Pre-phase: SPIKE (before Phase A)

**Note (history).** No pre-spec SPIKE ran during the design cycle (the SPEC was drafted with SPIKE-dependent claims hedged: §4.9.a Layer 1 sortedmulti_a routing, Layer 2 `Terminal::MultiA` sortedness disambiguation, hash-terminal round-trip, and the BIP-388 walk-tree completeness). This implementation-time SPIKE is the SOLE persisted SPIKE artifact for v0.3 and resolves all hedged claims before Phase A code is written. Do NOT look for a separate pre-spec spike report — none exists.

**Goal.** Resolve the SPIKE-dependent claims in SPEC §4.9.a + confirm end-to-end miniscript-v13 round-trip for the v0.3 supported surface.

**Sub-goals:**
1. Confirm rust-miniscript v13.0.0's API surface for `sortedmulti_a` in `tr()` leaves: does `TapTree::Leaf` expose sortedness as a distinct construct, or does parsing collapse to `Terminal::MultiA`?
2. Confirm hash-locked descriptors round-trip via `MsDescriptor::<DescriptorPublicKey>::from_str()` for: `sha256(<32B-hex>)`, `hash160(<hex>)`, `hash256(<hex>)`, `ripemd160(<hex>)`. Surface any rust-miniscript v13 parser errors specific to hash terminals.
3. Confirm timelock descriptors round-trip: `after(N)`, `older(N)`. Validate u32 vs i32 representation.
4. Confirm wrapper compositions parse: `v:`, `s:`, `a:`, `d:`, `j:`, `n:` in realistic descriptors.
5. Confirm `compute_wallet_policy_id` (BIP-388) is computable from any of the above; if md-codec or rust-miniscript exposes a helper, identify it.

**Where:** local worktree at `/scratch/code/shibboleth/mnemonic-toolkit/.spike-v0.3/` (NOT committed to master). Spike code is throwaway.

**Output:** `design/agent-reports/spike-toolkit-v0_3-pre-phaseA.md` with sub-goal answers, code snippets validating each, and any wire-format anomalies discovered. Architect-review the spike report to 0C/0I before Phase A starts.

**Gate:** if SPIKE finds blockers (e.g., rust-miniscript v13 cannot round-trip hash-locked descriptors), escalate to user with options: (a) wait for upstream PR, (b) carry a `[patch]` to a fork, (c) scope hash-locks out of v0.3.

## Phase A: descriptor parser + walker

**Files:**
- NEW: `crates/mnemonic-toolkit/src/parse_descriptor.rs` — implements per §4.9 of SPEC.
- MODIFIED: `crates/mnemonic-toolkit/Cargo.toml` — adds `miniscript = { version = "13", default-features = false, features = ["std"] }`.
- MODIFIED: `crates/mnemonic-toolkit/src/lib.rs` — exposes `pub mod parse_descriptor;`.

**TDD steps (each step: write test → implement to pass → architect-review block):**

A.1. Synthetic xpub generator with `b"toolkit-v0.3"` prefix (§4.9 step 4). Test: assert deterministic output for `(0, MultiSig)` and `(0, SingleSig)` against a fixed expected base58check string (locks the prefix as normative).

A.2. Placeholder lexer (§4.9 step 2). Test: regex parses `@0`, `@0/<0;1>/*`, `@0[fp/48'/0'/0'/2']/<0;1>/*`, malformed inputs.

A.3. `resolve_placeholders` (§4.9 step 3). Test: dense `0..n` enforced; gaps error; `PathDecl::Shared` vs `Divergent` correctly identified.

A.4. `walk_root` Layer 1 dispatch (§4.9.a Layer 1). Test: round-trip for each Layer 1 wrapper (Wpkh, Pkh, Wsh+Ms, Wsh+SortedMulti, Sh+Wpkh, Sh+Wsh, Sh+SortedMulti, Sh+Ms, Tr-keypath, Tr-singleleaf-miniscript) — 10 round-trip tests minimum. `Tr-singleleaf-sortedmulti_a` is deferred to v0.4 per pre-phase SPIKE (no upstream parser); see `design/agent-reports/spike-toolkit-v0_3-pre-phaseA.md` §1 and FOLLOWUP `tr-sortedmulti-a-via-upstream`. The mid-phase A reviewer MUST confirm the implementation matches the SPIKE conclusion (walker emits `Tag::MultiA` unconditionally for `multi_a`; never emits `Tag::SortedMultiA` in v0.3).

A.5. `walk_miniscript_node` Layer 2 — already-supported arms (PkK, PkH, Multi, MultiA, Check). Test: round-trip each (5 tests).

A.6. `walk_miniscript_node` Layer 2 — v0.3-NEW arms (After, Older, Sha256, Hash256, Hash160, Ripemd160, RawPkH, False, True, Verify, Swap, Alt, DupIf, NonZero, ZeroNotEqual, AndV, AndB, AndOr, OrB, OrC, OrD, OrI, Thresh — 23 arms). Test: round-trip each (23 tests).

A.7. `parse_descriptor` top-level orchestration (§4.9 step 7). Test: full pipeline for representative inputs (hash-locked, timelock, hybrid, multisig with annotation). **Cleanup:** remove the module-level `#![allow(dead_code)]` from `parse_descriptor.rs` (added in A.1 for incremental compilation); confirm clippy clean without it once `parse_descriptor` is fully wired and called from the orchestration layer.

A.8. Mode-determination function (§4.10). Test: `n==1` cases (wpkh, pkh, tr-keypath, wsh-pk, wsh-multi-1) all route single-sig; `n≥2` cases route multisig; `wsh(multi(1,@0))` does NOT collapse the tree (tree-faithfulness invariant).

**Architect-review:** at least one mid-phase r1 review (after A.4) and one end-of-phase review covering A.1–A.8. Persist both as `design/agent-reports/phase-A-parser-review-r{1,2}.md`. Iterate to 0C/0I; max r4.

**Phase A exit criterion:** all unit tests pass — enumerated subcounts: A.1 (≥2) + A.2 (≥4) + A.3 (≥3) + A.4 (≥10) + A.5 (≥5) + A.6 (≥23) + A.7 (≥4) + A.8 (≥6) = ≥57 unit tests; clippy clean; fmt clean; r0C/r0I from architect.

## Phase B: CLI flag wiring + mode-dispatch refactor

**Files:**
- MODIFIED: `crates/mnemonic-toolkit/src/cmd/bundle.rs` — `BundleArgs::template` becomes `Option<CliTemplate>`; new `--descriptor` / `--descriptor-file` flags; mode-dispatch ladder gains descriptor branch; pre-check ladder gains 15 §6.9 rows (TOP-TO-BOTTOM evaluation order).
- MODIFIED: `crates/mnemonic-toolkit/src/main.rs` — wire-up the new flags' help text.

**TDD steps:**

B.0 (pre-requisite, surfaced by Phase A mid-phase review I-2). Add `DescriptorParse(String)` variant to `crates/mnemonic-toolkit/src/error.rs`, mapped to exit code 2 (joining `ModeViolation`/`NetworkMismatch` in the exit-2 group). Migrate the lex/resolve/walk error sites in `parse_descriptor.rs` from `BadInput` to `DescriptorParse` so SPEC §6.7 descriptor-parse failures actually exit 2. Note: `ModeViolation` (exit 2, distinct kind) covers SPEC §6.9 flag-combination violations; `DescriptorParse` covers SPEC §6.7 descriptor-content errors. Both are exit 2 but represent different SPEC categories — keep them separate variants.

B.1. `BundleArgs::template: Option<CliTemplate>` with `required_unless_present_any = ["descriptor", "descriptor_file"]`. Test: clap-level rejection when none of three are present; clap-level acceptance when any one is.

B.2. New `--descriptor` and `--descriptor-file` fields with `conflicts_with` between them. Test: clap rejects both at once.

B.3. **Internal call-site refactor (the structural cost of B.1).** When `args.template` becomes `Option<CliTemplate>`, every existing read of `args.template.<method>()` in v0.2 code fails to compile. Strategy: introduce a top-level dispatch in `bundle.rs::run()` that branches BEFORE any `args.template.<method>()` call:
```rust
match (args.template.as_ref(), args.descriptor.as_ref(), args.descriptor_file.as_ref()) {
    (Some(t), None, None) => template_mode_run(t, &args),  // existing v0.2 logic; t: &CliTemplate (not Option)
    (None, Some(s), None) => descriptor_mode_run(s.clone(), &args),  // new
    (None, None, Some(p)) => descriptor_mode_run(read_descriptor_file(p)?, &args),  // new
    _ => unreachable!("pre-check ladder rejects all other combos"),
}
```
Inside `template_mode_run(t: &CliTemplate, args: &BundleArgs)`, the existing v0.2 code is unchanged (`t.is_multisig()`, `t.derivation_path()`, `t.wrapper_node(...)` — all called on `&CliTemplate`, not `Option<CliTemplate>`). The four `synthesize_*` functions (`synthesize_full`, `synthesize_watch_only`, `synthesize_multisig_full`, `synthesize_multisig_watch_only`) keep their `template: CliTemplate` signatures and continue to receive `*t` (deref-copy from `&CliTemplate`). Test: every v0.2 fixture passes through `template_mode_run` unchanged; descriptor-mode invocations route to a `descriptor_mode_run` STUB that asserts "not yet implemented" (Phase C replaces the stub).

B.4. Pre-check ladder: 15 `§6.9` rows evaluated TOP-TO-BOTTOM, BEFORE the dispatch in B.3. Test: one test per row asserting exit code 2 + byte-exact error message; tests verify evaluation order via the worked example in SPEC §6.9 (no-keys-at-all input → row 7 fires before row 10).

B.5. Update `BundleArgs` struct doc-comment to enumerate the new flags. Emit-call-sites in `engraving_card()` and `BundleJson` construction in `bundle.rs` are NOT refactored at Phase B (Phase C handles them; the Phase B stub never reaches those sites).

**Architect-review:** end-of-phase review only (single round); persist as `design/agent-reports/phase-B-cli-review-r1.md`. Iterate to 0C/0I.

**Phase B exit criterion:** rows 1-6 + row 8 mode-violation tests pass (9 tests in `tests/cli_mode_violations_v0_3.rs`); rows 7 and 9-15 are descriptor-content-aware and ship with Phase C's `descriptor_mode_run` synthesis path (additional 6 tests). v0.2 fixture matrix runs unchanged (34/34 card-string-byte-identical, JSON envelope still schema_version "2" since C.6 has not yet bumped it) under the v0.3 binary; clippy + fmt clean; r0C/r0I. (The cross-phase invariant runs the v0.2 regression at every phase exit; Phase B's listing here is for emphasis since this is the first phase that touches `BundleArgs` structure.) — Updated 2026-05-05 per Phase B end-of-phase architect L-1 (the 9/15 split is correct given content-aware rows can't fire without parsed descriptor).

## Phase C: synthesis path for descriptor mode + BundleJson struct migration

**Files:**
- MODIFIED: `crates/mnemonic-toolkit/src/synthesize.rs` — adds `synthesize_descriptor` (single entry point that dispatches single-sig vs multisig internally per SPEC §4.10; resolves the FOLLOWUP `synthesize-descriptor-fn-naming`).
- MODIFIED: `crates/mnemonic-toolkit/src/cmd/bundle.rs` — replaces Phase B's descriptor-synthesis stub with the new `synthesize_descriptor` call; updates `engraving_card()` template-arg call site to use `args.template.as_ref().map_or("descriptor", |t| t.human_name())` for descriptor mode; updates `BundleJson` construction to use the new optional template field + `descriptor` field.
- MODIFIED: `crates/mnemonic-toolkit/src/format.rs` — `BundleJson.template: &'static str` becomes `Option<&'static str>`; new `descriptor: Option<String>` field; `schema_version` constant bumps from `"2"` to `"3"`; `MultisigInfo` retains `template: &'static str` and `path_family: &'static str` fields, populated with the literal `"descriptor"` static string for descriptor-mode multisig bundles.

**Rationale for combining synthesis + BundleJson migration:** Phase C must produce a binary that compiles AND emits valid JSON for descriptor-mode bundles, because Phase C's exit criterion runs descriptor-mode integration tests that exercise the FULL emit path (synthesize → engraving card → JSON serialization). Deferring the `BundleJson` struct mutation to Phase D would leave Phase C's integration tests unable to assert JSON shape. Moving the struct mutation here keeps the codebase compilable and emit-clean at every phase boundary.

**TDD steps:**

C.1. `synthesize_descriptor` entry point. Single function that takes parsed `Descriptor`, the `Mode` enum (full/watch-only) per the SPEC §4.10 rules, and the `@N` binding sources (seed for full-`@0`, xpub for watch-only-`@0`, cosigner triples for `@N≥1`). Returns a `Bundle`. Internally dispatches to single-sig path (n=1) or multisig path (n≥2). Tests: call with each combination (full single-sig, watch-only single-sig, full multisig, watch-only multisig); assert correct `Bundle` shape (single mk1 card vs n mk1 cards).

C.2. `@0` binding (§4.11): full-mode origin-annotation parsing + xpub derivation; watch-only-mode `--xpub` binding. Tests: derived xpub matches expected for canonical `48'/0'/0'/2'` path; annotation-fp mismatch errors.

C.3. `@N≥1` binding (§4.11): cosigner triple validation; descriptor annotation cross-check; ordering by index. Tests: positional indexing (full multisig: cosigner 0 → @1, etc.); annotation mismatch errors.

C.4. SELF-MULTISIG WARNING (§4.11 final paragraph): toolkit detects self-multisig via xpub equality. Tests: warning emitted to stderr verbatim from v0.2 when conditions met.

C.5. Wire-bit-identical descriptor-mode equivalence (§5.6 conditional guarantee): for descriptor inputs that exactly express v0.2 templates (3 representative cases per SPEC §10 D.2), assert ms1/mk1/md1 outputs are byte-identical to template-mode emissions.

C.6. `BundleJson` struct migration: `template: &'static str` → `Option<&'static str>`; new `descriptor: Option<String>` field; `schema_version` constant bumps to `"3"`. Default serde `Option` serialization (no `#[serde(skip_serializing_if = ...)]`); both fields ALWAYS emit (`null` when None, value when Some). Tests: serde round-trip; v0.2 fixture emits `"template": "bip84"` + `"descriptor": null` + `"schema_version": "3"` (card strings byte-identical to v0.2; JSON envelope schema_version diff acceptable per SPEC §5.6).

C.7. `MultisigInfo` descriptor-mode population (SPEC §5.6 sub-section): for descriptor-mode multisig bundles, `template = "descriptor"`, `path_family = "descriptor"` (both literal static strings); threshold derivation per SPEC: `k` for `Multi`/`SortedMulti`/`MultiA`/`SortedMultiA`/`Thresh`, `n` (placeholder count) for other compositions; `cosigner_count = n`; `cosigners` array unchanged in shape from v0.2 (per-`@N` index, sorted, with xpub/fp/path from §4.11 binding). Tests: each threshold-derivation branch; literal `"descriptor"` in template/path_family fields.

**Architect-review:** mid-phase (after C.3) and end-of-phase reviews; persist as `design/agent-reports/phase-C-synthesis-review-r{1,2}.md`. Iterate to 0C/0I.

**Phase C exit criterion:** all integration tests for the 6 representative descriptor categories (SPEC §10 C.1) pass; 3 descriptor-mode wire-bit-identical regressions pass (SPEC §10 D.2); BundleJson serde round-trips work for both template-mode and descriptor-mode bundles; the v0.2 34-fixture matrix runs under v0.3 binary with byte-identical CARD strings (JSON envelope `schema_version: "3"` + `descriptor: null` is the only allowed diff per cross-phase invariant); clippy + fmt clean; r0C/r0I.

## Phase D: verify-bundle intake + self-check + descriptor error variants

**Files:**
- MODIFIED: `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` (or equivalent) — accepts schema `"2"` or `"3"`; for schema-3 with `descriptor != null`, re-parses via `parse_descriptor` and re-runs the 9 / 3+6N check ladder.
- MODIFIED: `crates/mnemonic-toolkit/src/cmd/bundle.rs` — `--self-check` (already wired for v0.2) runs the verify-bundle check ladder against the synthesized descriptor-mode bundle.
- MODIFIED: `crates/mnemonic-toolkit/src/error.rs` (or wherever toolkit errors live) — new variant `DescriptorReparseFailed { detail }` mapped to exit code 4.

**Note (scope clarification):** `BundleJson` struct migration and `MultisigInfo` descriptor-mode population already shipped in Phase C (C.6, C.7). Phase D handles the INTAKE side of verify-bundle (consuming schema-3 JSON) and the verification check ladder for descriptor-derived bundles.

**TDD steps (ordered for TDD dependency: error type defined before its use):**

D.1. Add `DescriptorReparseFailed { detail }` error variant + exit-code-4 mapping. This is a type definition only; the behavioral test that exercises this variant lives in D.3 below. (Splitting type-definition from behavioral-test step satisfies TDD ordering: D.3 cannot write a failure-path test until the variant exists.)

D.2. verify-bundle JSON intake: schema version detection (string match on `"schema_version"` field — accept `"2"` or `"3"`; reject other values with existing schema-mismatch error from v0.1). Tests: schema-2 bundle (descriptor=null) verifies via existing template-mode logic; schema-3 bundle with `descriptor != null` triggers descriptor re-parse path; schema-3 bundle with `descriptor: null` (v0.2 invocation under v0.3 binary case) verifies via existing template-mode logic.

D.3. Schema-3 descriptor re-parse + re-compute: when `descriptor != null`, re-parse via `parse_descriptor` using the bundle's preserved cosigner triples and `@0` source; recompute expected ms1/mk1/md1; run the 9 / 3+6N check ladder. Tests: (a) success path — emit→verify round-trip for each descriptor category from Phase C.1 + C.2 (the 8 integration scenarios specified in SPEC §10 C.3); (b) failure path — corrupt the `descriptor` field in a bundle JSON; verify-bundle returns `DescriptorReparseFailed` (from D.1) with exit 4.

D.4. `--self-check` for descriptor mode: post-synthesis, toolkit invokes verify-bundle's check ladder against its own emit. Tests: pass test for valid descriptor-mode bundle (all 9 / 3+6N checks pass); fail-injection test (manually corrupted descriptor in BundleJson causes self-check to surface a `BundleMismatch` error, exit 4).

**Architect-review:** end-of-phase review; persist as `design/agent-reports/phase-D-verify-review-r1.md`. Iterate to 0C/0I.

**Phase D exit criterion:** all 8 emit→verify round-trips pass (SPEC §10 C.3); `--self-check` pass + fail tests pass; descriptor-reparse-failed error variant covered; v0.2 fixture matrix continues to verify cleanly (no regression in template-mode verify); clippy + fmt clean; r0C/r0I.

## Phase E: release prep

**Files:**
- MODIFIED: `CHANGELOG.md` — v0.3.0 entry.
- MODIFIED: `Cargo.toml` (toolkit) — version bump to `0.3.0`.
- MODIFIED: `README.md` — `--descriptor` example.
- NEW: `tests/fixtures/v0.3/` — 40+ v0.3-mode fixtures (SPEC §10 E).
- NEW: `tests/integration_v0_3.rs` — runs all fixtures.

**Steps:**

E.1. Fixture matrix: ≥40 v0.3-mode fixtures covering A.1+A.2 unit-test categories, C.1+C.2 integration scenarios, B mode-violation goldens (15). Each fixture is a JSON file with `{ "args": [...], "expected_stdout": "...", "expected_stderr": "...", "expected_exit": 0 }` (or analogous).

E.2. v0.2 fixture matrix carry: copy v0.2's 34 fixtures verbatim; assert SHA pin `a381761656fd165e8e5af28574a5baaa55973e562c610254ae6f31d6b1f06171` still passes under v0.3 binary (wire-bit-identical regression for card strings; JSON envelope schema_version diff acceptable per SPEC §5.6).

E.3. New v0.3 SHA pin: compute SHA-256 of the v0.3 fixture corpus's concatenated card strings, using the same canonical-form command established for v0.2's pin (refer to v0.2 CHANGELOG entry's reproduction command). Concretely, the pin is `cat <fixtures>/*.json | jq -r '.cards.ms1, .cards.mk1[], .cards.md1[]' | sha256sum`, applied to the v0.3 fixture set. Pin the hex value in `CHANGELOG.md` v0.3.0 entry. Reproduction command included verbatim.

E.4. Integration test runner: `tests/integration_v0_3.rs` reads each fixture and exec's the binary; asserts expected output byte-exact.

E.5. CHANGELOG entry: list features, breaking changes (`BundleArgs::template` → `Option`), schema_version bump, SHA pin, links to SPEC + plan + PR.

E.6. Tag + release: `cargo build --release`; `git tag mnemonic-toolkit-v0.3.0`; push tag; GitHub release notes copied from CHANGELOG.

**Architect-review:** end-of-phase review (final review across all phases); persist as `design/agent-reports/phase-E-release-review-r1.md`. Iterate to 0C/0I.

**Phase E exit criterion:** ≥40 v0.3 fixtures + 34 v0.2 fixtures all pass; SHA pin verified; CHANGELOG complete; tag + release pushed (gated on user approval per `feedback_iterative_review_every_phase` discipline); r0C/r0I.

## Cross-phase invariants

- TDD discipline: tests precede implementation in every phase. No commit may include both new test + new impl without a separate red-green sequence.
- Per-phase commit cadence: feature commit + per-architect-review-round fixup commit + per-phase verdict commit. Stage paths explicitly (no `git add -A`).
- Verify HEAD content post-commit: `git show HEAD:path | head -N` for each file modified in the commit.
- After each phase, run the full v0.2 fixture matrix as a regression check (34/34 must pass; card-string byte-identity).
- No phase may merge without `feature-dev:code-architect` review at 0C/0I.
- Cross-repo pushes: NONE expected (toolkit-only). The walker-backport-to-md-cli FOLLOWUP is a separate v0.4-cross-repo cycle.

## Verification (cycle exit criteria)

After Phase E ships, the repo state is:

1. `crates/mnemonic-toolkit/src/parse_descriptor.rs` — present, 100% covered by Phase A unit tests.
2. `crates/mnemonic-toolkit/src/cmd/bundle.rs` — Phase B refactor complete; 15 mode-violation rows pass.
3. `crates/mnemonic-toolkit/src/synthesize.rs` — Phase C `synthesize_descriptor` entry point present.
4. `crates/mnemonic-toolkit/src/format.rs` — Phase C (C.6, C.7): `BundleJson` schema-3 with `template: Option<&'static str>` + `descriptor: Option<String>`; `MultisigInfo` descriptor-mode literals (`template = "descriptor"`, `path_family = "descriptor"`).
5. `tests/fixtures/v0.3/` — ≥40 fixtures + 34 v0.2 carries.
6. `CHANGELOG.md` — v0.3.0 entry with new SHA pin.
7. `Cargo.toml` — version 0.3.0; `miniscript = "13"` dep present.
8. Tag `mnemonic-toolkit-v0.3.0` pushed.
9. `design/agent-reports/spike-toolkit-v0_3-pre-phaseA.md` + 5+ phase-X review reports persisted.
10. `design/SPEC_mnemonic_toolkit_v0_3.md` — present.
11. `design/IMPLEMENTATION_PLAN_v0_3_descriptor_passthrough.md` — present (this document).
12. `design/FOLLOWUPS.md` — updated with v0.3-tier entries.

## Revision history

- 2026-05-05: Round 1 draft.
- 2026-05-05: Round 2 — addressed architect r1 verdict (2C / 4I / 3L); Phase D.1 (BundleJson struct migration) and D.2 (MultisigInfo descriptor-mode population) MOVED to Phase C as C.6 / C.7 (C-1: keeps codebase compile-and-emit-clean at every phase boundary); Phase B.3 expanded to enumerate the `bundle.rs::run()` dispatch refactor that handles `args.template: Option<CliTemplate>` without cascading internal call-site changes (C-2); Phase A.4 now requires SPIKE-citation in implementation + reviewer cross-check (I-1); Phase A exit criterion enumerates subcounts ≥58 (I-2); Pre-phase SPIKE section adds historical note that no pre-spec SPIKE ran in the design cycle (I-3); Phase D.3 inputs corrected to "from Phase C.1 + C.2 (the 8 integration scenarios)" (I-4); Phase E.3 SHA-pin command made explicit (L-1); Phase B exit criterion notes cross-phase invariant covers the v0.2 regression (L-2); `synthesize-descriptor-fn-naming` resolution noted as slightly asymmetric with v0.2 (L-3 — flagged for Phase C reviewer).
- 2026-05-05: Round 3 — addressed architect r2 verdict (0C / 2I / 0L); Phase D TDD step ordering reworked to define `DescriptorReparseFailed` error variant in D.1 BEFORE D.3 (which uses it for the failure-path test); D.3 now combines success+failure paths in one TDD step (NF-2); Cycle exit criterion item 4 corrected to attribute `BundleJson`/`MultisigInfo` work to Phase C (C.6, C.7) — was stale "Phase D" reference (NF-1).
- 2026-05-05: Round 4 (post-SPIKE) — pre-Phase-A SPIKE resolved §4.9.a hedged claims; rust-miniscript v13.0.0 cannot parse `sortedmulti_a` in tap-leaves. User approved option (c) "scope sortedmulti_a out of v0.3" with soft-deferral framing. Phase A.4 — `Tr-singleleaf-sortedmulti_a` test dropped; round-trip subcount `≥11` → `≥10`; A.4 implementation cites SPIKE report §1 (not the SPIKE-dependent SPEC paragraph that no longer exists). Phase A exit criterion subcount total `≥58` → `≥57`. SPEC §4.9.a / §4.10 / §9 Q2 patched in lockstep (see SPEC revision Round 7). New FOLLOWUP `tr-sortedmulti-a-via-upstream` at v0.4-cross-repo tier.
