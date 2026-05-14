# Phase 3a Re-Scope Proposal Review (R0)

**Reviewer:** Opus 4.7 (1M context), `feature-dev:code-reviewer`
**Date:** 2026-05-13
**Proposal reviewed:** `/home/bcg/.claude/plans/2026-05-13-cycle-b-phase-3a-rescope-proposal.md`
**Supersedes:** R0 v2 LOCK at `design/agent-reports/v0_9_B-phase-3a-toolkit-applications-r0.md` §10b/§10c
**Verdict:** **RE-DRAFT NEEDED** — 4 Critical, 4 Important, 2 Nit

The re-scope's one-line summary ("strip the field-type migration; rely on Site 1 cmd-handler-scope pin to cover ResolvedSlot/DerivedAccount through their lifetime") rests on a premise that does not hold against source: the entropy bytes that live in `ResolvedSlot.entropy` / `DerivedAccount.entropy` are **derived buffers** (output of `mnemonic.to_entropy()` / `hex::decode(...)`) on heap allocations DISJOINT from the `args.passphrase` / `args.slot[i].value` / `args.from[i].value` argv-input strings the proposal enumerates pinning. A Site 1 pin on the input strings does not pin the derived entropy buffer; the threat-model claim "secret entropy pages stay mlock-pinned for the buffer's lifetime" is FALSE under this proposal at Sites 2/3.

The R0 v2 LOCK that the proposal wants to supersede pinned the *derived* entropy via struct-sibling fields specifically to address this. The proposal eliminates those struct-sibling pins without replacing the coverage they provided.

Cannot proceed to Task 1 as drafted.

---

## CRITICAL findings

### C-1 (conf 100): Site 1 pin does NOT cover `ResolvedSlot.entropy` / `DerivedAccount.entropy` — different heap allocations

**Proposal claim (§1.3, lines 37-45 + §4 trade-off table line "Pages mlock-pinned while secret bytes are resident: YES"):** "moving `entropy: Vec<u8>` into `ResolvedSlot { entropy: Some(buf), .. }` and back out via `into_parts()` does not move the underlying heap allocation, so a single pin in the enclosing scope covers the buffer through every owner transition."

**Source ground truth:**

The bytes that end up in `ResolvedSlot.entropy: Option<Vec<u8>>` are **NOT** moved from any argv input field. They are produced by:

- `cmd/bundle.rs:341-353` (Phrase arm): `derive_full(phrase, ...)` calls `mnemonic.to_entropy()` internally (`derive.rs:72`, `derive_slot.rs:78` `entropy: entropy.to_vec()`). The Vec returned is then moved into `ResolvedSlot { entropy: Some(entropy), .. }` (line 353). The original argv `s.value` (the BIP-39 phrase string at `args.slot[i].value`) is never moved into the entropy field — it's parsed into a `Mnemonic`, and the `Mnemonic` produces a fresh `Vec<u8>` of entropy bytes at a NEW heap allocation.
- `cmd/bundle.rs:425-456` (Entropy arm): `entropy_bytes = hex::decode(entropy_hex)` at line 433. `hex::decode` returns a freshly-allocated `Vec<u8>`. The argv `s.value` (a hex string) and the resulting `entropy_bytes` are on disjoint heap allocations.
- `cmd/bundle.rs:921` and `cmd/bundle.rs:986-990` (descriptor-mode bundle path): explicit `mnemonic.to_entropy()` and `hex::decode(...)` allocations, then `Some((*entropy).clone())` at lines 942 and 1008 — the entropy goes into `cosigners[i].entropy` via an additional **clone** (yet another fresh heap allocation).
- `derive_slot.rs:78`: `DerivedAccount { entropy: entropy.to_vec(), ... }` — `entropy.to_vec()` is a fresh allocation copying from the caller's `&[u8]` slice.

The only two arguments to `derive_full` / `derive_bip32_from_entropy` that the proposal's Site 1 pins are `&str` borrows of `s.value` (the phrase or entropy hex). Neither of those `&str` borrows ends up in the `ResolvedSlot.entropy` / `DerivedAccount.entropy` Vec. The argv input string and the derived-entropy buffer are at **different heap addresses**.

**Additional aggravator:** at `bundle.rs:203` the entire slot Vec is cloned (`let slots = args.slot.clone();`) before being passed to `resolve_slots`. Even within bundle's `args.slot[i].value`, the buffer downstream code reads is a clone, not the originally-pinned String.

**Consequence for the threat model:** The Cycle B SPEC's headline claim (§1 "secret material leaks to swap on memory-pressured systems" — eliminated by mlock) requires the *derived* entropy buffer to be pinned, since that's what holds the secret BIP-39 entropy bytes for the duration of synthesize/emit. The proposal pins the *input* strings (which are also secret-bearing, and that's a defensible Site 1 contribution) but loses ALL pin coverage of the derived entropy buffer at Sites 2/3. The proposal's §4 trade-off row "Pages mlock-pinned while secret bytes are resident: YES" is wrong: it's YES for the input string's pages but NO for the derived-entropy buffer's pages.

**Correction options:**

a. Re-add struct-sibling pins on `ResolvedSlot._entropy_pin` / `DerivedAccount._entropy_pin` (essentially re-adopt the R0 v2 LOCK design). The Clone-collision is real and Arc-wrap is the reasonable resolution.
b. Keep Site 1 + Site 4 only, AND explicitly accept the threat-model degradation in writing: "Sites 2/3 derived-entropy buffers are unpinned in Cycle B; the user-input strings are pinned at Site 1; bip85-derived entropy is pinned at Site 4. Toolkit-derived BIP-39 entropy is left unpinned and may swap." This is a substantively narrower threat-model claim than Cycle B has shipped in SPEC and would need explicit cycle-scope SPEC §1 / §2 row 5 / §6 G1 rewrites.

Either path requires discussion before LOCK. The proposal as drafted appears to choose (b) implicitly while presenting it as "equivalent threat-model coverage" (§4) — the equivalence claim is the bug.

**Source citations:** `derive.rs:72`, `derive_slot.rs:78`, `cmd/bundle.rs:203,341-353,425-456,921,986-990,942,1008`, `parse_descriptor.rs:872,946`.

---

### C-2 (conf 100): `convert.rs` and `derive_child.rs` have NO `apply_stdin_substitutions` — proposal's anchor function does not exist there

**Proposal claim (§3 line 96 + Step 5.3):** "the pin block lands AFTER `apply_stdin_substitutions` / `apply_slot_stdin` returns" and "Edit `cmd/convert.rs`. Same pattern (note: 2 named passphrases + 1 vec-iteration block)."

**Source ground truth:**

`apply_stdin_substitutions` exists ONLY in `cmd/bundle.rs:1227` and `cmd/verify_bundle.rs:565`. Grep for the full identifier confirms zero matches in `convert.rs` or `derive_child.rs`.

- `cmd/convert.rs:597-672` uses local-variable substitution: `effective_passphrase: Option<String>` (line 652), `effective_bip38_passphrase: Option<String>` (line 660), `primary_value: String` (line 667). These are NEW heap-allocated Strings on each call (via `args.passphrase.clone()`, `read_stdin_passphrase(stdin)?`, `read_stdin_to_string(stdin)?`). The source argv field `args.passphrase` is never mutated.
- `cmd/derive_child.rs:98-122` similarly uses `from_value: Zeroizing<String>` and `stdin_passphrase: Option<Zeroizing<String>>` locals built via `read_stdin_to_string` / `read_to_string`.

So the proposal's Step 5.3 ("Edit `cmd/convert.rs`. Same pattern") is unrealizable as written — there is no `apply_stdin_substitutions` call to position the pin "after." Pinning `args.passphrase.as_bytes()` at the top of `convert.rs::run` would pin the pre-substitution argv String, which is fine for `--passphrase <inline>` but irrelevant when `--passphrase-stdin` is used (the actual secret bytes are in `effective_passphrase`, a separate heap allocation). The proposal's I-1 mitigation (synthetic-args mutation window) does not apply because no synthetic-args struct exists in convert/derive_child.

**Correction:** The proposal needs separate Site 1 strategies per-handler:
- `bundle.rs` / `verify_bundle.rs`: pin after `apply_stdin_substitutions`.
- `convert.rs`: pin `effective_passphrase`, `effective_bip38_passphrase`, `primary_value` AFTER they're bound.
- `derive_child.rs`: pin `from_value` and `stdin_passphrase` AFTER they're bound.

Also, the post-substitution pin in `bundle.rs::run` covers `synthetic_args` (a new `BundleArgs` clone), not the original `args` — the proposal's Step 5.1 sample `pub fn run(mut args: BundleArgs)` doesn't match the actual `pub fn run<W, E>(args: &BundleArgs, ...)` signature; the pin must be attached to the `&BundleArgs` re-binding after `synthetic_args` is constructed.

**Source citations:** `cmd/convert.rs:597,652,660,667`, `cmd/derive_child.rs:77,98-122`, `cmd/bundle.rs:102,113-119,1227`, `cmd/verify_bundle.rs:105,118,565`. Grep `apply_stdin_substitutions` confirms only 2 matches across all of `cmd/`.

---

### C-3 (conf 95): `format_bip39_with_test_hook` plus `is_page_range_locked_for_test` are unrealizable as proposed — `bip85` is binary-private, not exposed via `lib.rs`

**Proposal claim (Step 2.2 + Step 4.2):** Integration test `cli_mlock_integration.rs` calls `mnemonic_toolkit::bip85::format_bip39_with_test_hook(...)` from within `tests/`.

**Source ground truth:**

`crates/mnemonic-toolkit/src/lib.rs` (full file, 13 lines) exposes ONLY `pub mod mlock;`. The crate is a hybrid lib + bin; everything else is binary-private (Phase 2 R0 Option C, locked in SPEC §4 P2). `bip85.rs` declares its `format_*` functions `pub(crate)` (lines 73, 100, 127, 158, 175, 189, 214) — they are not visible from any path outside the binary's module tree.

Adding `pub mod bip85;` to `lib.rs` cascades a transitive surface explosion (bip85 uses error/language/network types which would also need to be made public). The proposal does not mention this surface change.

The `is_page_range_locked_for_test` helper is similarly broken: the proposal places it under `#[cfg(any(test, feature = "test-helpers"))]` in `src/mlock.rs` but `Cargo.toml` defines no `test-helpers` feature (verified — `[features]` section absent entirely). Without that feature, the helper reduces to `#[cfg(test)]`, which per `RFC 1604` (and the explicit comment at `src/mlock.rs:12-15`) is per-crate-not-per-build: the helper would NOT be visible to integration tests in `tests/`. This is the exact bug Phase 2 R1 caught in I-1 (e53cca8); the proposal recreates it.

The shipped pattern in `mlock.rs:213-236` for `failure_count_for_test` etc. is `pub fn`, no `#[cfg]` gate. The proposal must follow that pattern (drop the cfg gate; expose `pub fn is_page_range_locked_for_test` unconditionally) to be reachable from integration tests.

**Correction:**
1. For `is_page_range_locked_for_test`: drop the cfg gate; make it a plain `pub fn` peer of `failure_count_for_test`. Cargo.toml unchanged.
2. For `format_bip39_with_test_hook`: either (a) expose `bip85` via `lib.rs` and accept the public-surface cascade, or (b) move the Site 4 in-process test inside `bip85.rs` itself as a `#[cfg(test)]` library unit test (the precedent the existing `g2_*` tests in `mlock.rs::tests` set per Phase 2 R1 I-1 fold).

**Source citations:** `src/lib.rs:1-13`, `src/bip85.rs:73,100,127,158,175,189,214`, `src/mlock.rs:12-15,213-236`, `Cargo.toml` (no `[features]` section), `design/agent-reports/v0_9_B-phase-2-mlock-module-r1.md` I-1 fold context.

---

### C-4 (conf 85): The proposal preserves Cycle A's `impl Drop for DerivedAccount` but Cycle A's existing FOLLOWUP `resolved-slot-entropy-zeroizing-field` was *open and scheduled* — the proposal also re-tiers and weakens that schedule

**Proposal claim (§2.2, lines 76-83):** "Filed as a single FOLLOWUP entry `resolved-slot-derived-account-zeroizing-field-and-pin` (replaces the open `resolved-slot-entropy-zeroizing-field` with broader scope)" tiered `v0.10.1`.

**Issue:** The R0 v2 LOCK §10b's deciding rationale (item 1, lines 335-336) was: "an open FOLLOWUP `resolved-slot-entropy-zeroizing-field` (surfaced 2026-05-13, Cycle A Phase 2 GREEN) explicitly schedules `ResolvedSlot.entropy` → `Option<Zeroizing<Vec<u8>>>` and notes deferral was due to '19-site cascade.' It is tiered `v0.9.2-nice-to-have`. Phase 3a is already touching every one of those 19 sites to add `_entropy_pin` siblings — the cascade cost is paid once. Landing the field-type change in the same commit is strictly cheaper than landing it as a separate `v0.9.2-nice-to-have`."

The new proposal inverts the calculus: by NOT touching the 19 sites for `_entropy_pin`, the field-type migration's cascade cost is re-paid when v0.10.1 ships. The proposal mentions this re-pay cost only obliquely ("clean diff, easy review" — but two clean diffs together are not strictly cheaper than one combined diff at scale). It also raises the FOLLOWUP from `v0.9.2-nice-to-have` (with a specific cycle-anchor close trigger) to `v0.10.1` (a brand-new tier, no existing roadmap anchor) — that's a strictly weaker schedule.

This isn't a bug in the source-checking sense, but it's a Critical decision-quality gap: the proposal's §2.2 deferral does not engage with the §10b lock's primary reasoning. The proposal §1.2 lists the Arc-wrap costs, but does not refute §10b item 2 ("G4.a invariant is structurally guaranteed under Zeroizing field-type") — under the proposal, G4.a relies on `impl Drop for DerivedAccount` continuing to scrub correctly as future contributors evolve the struct, which is exactly the human-maintained discipline gap that motivated migrating to `Zeroizing<>` field-types in the first place.

**Correction:** Either explicitly engage with and refute §10b items 1 + 2 (showing the calculus changed), or acknowledge the proposal accepts a strictly-weaker schedule + structural-discipline tradeoff in exchange for the Arc-wrap removal. The current §1.2 / §4 / §7 framing presents this as "no-cost simplification" which it isn't.

**Source citations:** `design/FOLLOWUPS.md` `resolved-slot-entropy-zeroizing-field` (referenced in R0 v2 §10b line 336 — re-verify its current status), R0 v2 LOCK §10b items 1-2 (lines 335-348).

---

## IMPORTANT findings

### I-1 (conf 90): R-3 risk-register claim about return types is wrong on 2 of 4 handlers

**Proposal claim (§6 R-3):** "Verified at proposal-write time: all 4 cmd handlers return `Result<(), ToolkitError>` (no struct returned to caller). The handler IS the secret's full-process scope."

**Source ground truth:**
- `cmd/bundle.rs:102`: `pub fn run<W, E>(args: &BundleArgs, ...) -> Result<(), ToolkitError>` — matches.
- `cmd/verify_bundle.rs:105`: `pub fn run<W, E>(args: &VerifyBundleArgs, ...) -> Result<u8, ToolkitError>` — does NOT match.
- `cmd/convert.rs:597`: `pub fn run<R, W, E>(args: &ConvertArgs, ...) -> Result<u8, ToolkitError>` — does NOT match.
- `cmd/derive_child.rs:77`: `pub fn run<R, W, E>(args: &DeriveChildArgs, ...) -> Result<(), ToolkitError>` — matches.

The substantive claim ("no struct holding entropy is returned to the caller") still holds (`u8` is a status code, not entropy). But the verbatim "all return `Result<(), ToolkitError>`" is wrong on half the handlers — yet another off-by-N narrative error in this Cycle. Per `feedback_r0_must_read_source_off_by_n`, this should be folded inline (correct R-3 to "all 4 cmd handlers return `Result<(), ToolkitError>` or `Result<u8, ToolkitError>`; none returns a struct holding entropy").

**Source citations:** `cmd/bundle.rs:102`, `cmd/verify_bundle.rs:105`, `cmd/convert.rs:597`, `cmd/derive_child.rs:77`.

---

### I-2 (conf 90): bip85 `format_*` function names in §4 Step 4.1 are inaccurate — proposal admits "GREEN-time confirmation" but should fold names now

**Proposal claim (Step 4.1):** "Confirm the 7 functions: `format_bip39`, `format_bip39_dice` (a.k.a. `format_dice_rolls` per Phase 1 R0 finding — verify name in source), `format_hd_seed_wif`, `format_hex`, `format_xprv`, `format_seed_hex`, and the 7th — list to be confirmed against `bip85.rs` at GREEN time."

**Source ground truth (`bip85.rs` grep `^pub(crate) fn format_`):**
1. `format_bip39_phrase` (line 73) — NOT `format_bip39`
2. `format_hd_seed_wif` (line 100) — matches
3. `format_xprv_child` (line 127) — NOT `format_xprv`
4. `format_hex_bytes` (line 158) — NOT `format_hex`
5. `format_password_base64` (line 175) — missing from proposal's list
6. `format_password_base85` (line 189) — missing from proposal's list
7. `format_dice_rolls` (line 214) — NOT `format_bip39_dice`

The proposal lists `format_seed_hex` which **does not exist** anywhere in the source. Five of seven names in the proposal are inaccurate. Per the R0-discipline-on-source rule, names should be folded now (not deferred to GREEN); the proposal's "list to be confirmed" framing concedes this is a known gap. Also, the SPEC §2 row 4 Phase 1 entry already enumerates the 7 callees authoritatively from Phase 1's R0/R1 work — the proposal could have cited that instead of guessing names.

**Source citations:** `src/bip85.rs:73,100,127,158,175,189,214`. Phase 1 R0 verification at `design/agent-reports/v0_9_B-phase-1-bip85-heap-promote-r0.md`.

---

### I-3 (conf 85): SPEC supersession completeness — Task 1 likely leaves SPEC §6 G4.a clauses dangling

**Proposal claim (Step 1.3):** "Edit SPEC §6 G4.a to drop the 'Sites 2/3 struct-field declaration order' sentence; replace with: 'Sites 2/3: Cycle A's existing Drop discipline ... is preserved unchanged in Cycle B. Site 1 cmd-handler-scope pin covers the buffer's full residency.'"

**Issue:** Per C-1 above, "Site 1 cmd-handler-scope pin covers the buffer's full residency" is FALSE for `ResolvedSlot.entropy` and `DerivedAccount.entropy`. So the proposed §6 G4.a replacement text would itself be inaccurate at SPEC commit time. Additionally, SPEC §2 row 5 (currently long, paragraph spanning lines 35-41) contains many sub-clauses about Sites 2/3 struct-field changes, lint anchor relabels, FOLLOWUP closure, Arc-wrap rationale, declaration-order drop semantics, into_parts cross-boundary handoff. Step 1.1's "Replace §2 row 5 Sites 2/3 paragraphs with [short paragraph]" is under-specified relative to the actual SPEC text that needs surgery. R0 v2 §10b items 9-12 also describe lint anchor updates the new SPEC text would need to undo.

**Correction:** Task 1 should enumerate every R0 v2 §10b clause currently present in SPEC + lint + FOLLOWUPS and explicitly mark each as preserved / removed / re-tiered, before any text edit happens. Otherwise the post-Task-1 SPEC will mix v2-LOCK and v3-RESCOPE language and fail review.

---

### I-4 (conf 85): R-1 lint mitigation is referenced but not scheduled as a task

**Proposal claim (§6 R-1):** "Mitigation: Add a `lint_no_mutating_calls_on_pinned_secrets` test that greps the cmd-handler bodies for `\.push\(`, `\.extend\(`, `\.reserve\(` etc. against any `args.passphrase` / `args.slot[i].value` / `args.from[i].value`."

**Issue:** Task 5 (Site 1 GREEN) does not include creating this lint. Task 9 (R1 review) doesn't reference it. If the lint is load-bearing for the safety story (it's specifically called out as the Mitigation, not "audit at proposal-write time is sufficient"), it must be a scheduled task. If audit-only is sufficient, R-1 should say so.

**Correction:** Either add Task 5.5 "create lint_no_mutating_calls_on_pinned_secrets" OR rewrite R-1 mitigation as "audit at proposal-write time + R1 reviewer reads the cmd handlers fresh to re-confirm." The current text reads as scheduled but no task creates the test.

I confirmed via grep there are currently no `.push` / `.extend` / `.reserve` / `.clear` / `.truncate` / `.resize` / `.insert` / `.remove` / `.append` calls against `.passphrase`, `.slot[…].value`, `.from[…].value`, `.bip38_passphrase`, or `.entropy` fields in the codebase. The audit holds today. Scheduling the lint is a separate question (R1 should not be the sole regression backstop).

---

## NIT findings

### N-1 (conf 60): YAML quoting in Step 7.1 release-build CI snippet is fine but `mlock subprocess` in `name:` should be quoted defensively per memory `feedback_r2_blocking_vs_cosmetic_gate`

**Proposal claim (Step 7.1):**
```yaml
test-release-mlock:
  name: "test (release, ubuntu-latest, mlock subprocess)"
```

The `name:` value is already quoted (good). The `run:` value `cargo test --release --test cli_mlock_g2_subprocess` is bare and contains no YAML-special tokens (`:`, leading `-`, bare booleans), so it is safe. Actionlint will pass. No defensive change needed beyond what the proposal already shows. Flagging only because the `feedback_r2_blocking_vs_cosmetic_gate` memory says "quote YAML strings defensively" — the snippet already does this for `name:`. Step 7.2 correctly schedules `actionlint .github/workflows/rust.yml`. OK as-is.

---

### N-2 (conf 50): "~245 LOC" footprint estimate vs ~185 LOC for R0 v2 LOCK is misleading — the larger LOC isn't a virtue if a chunk is duplicated test plumbing for an unrealizable hook

The proposal frames the LOC delta as "more *test* code, less *production* code change" (§2.1 and §7), which is generally a virtue. But ~80 LOC of integration tests (Step 2.1-2.3) depend on the unrealizable `format_bip39_with_test_hook` and missing `test-helpers` feature (per C-3). After C-3 is folded the LOC count will likely shift. Not a blocker; just an estimate that depends on resolving C-3.

---

## Summary

| Severity | Count |
|---|---|
| Critical | 4 |
| Important | 4 |
| Nit | 2 |

**Verdict: RE-DRAFT NEEDED.**

The central issue is C-1: the proposal's foundational §1.3 premise about Vec heap-pointer stability is technically true but applied to the wrong buffers. Site 1 pins the user-input strings; the derived-entropy buffers (which `mnemonic.to_entropy()` and `hex::decode(...)` produce on fresh allocations) are NEVER pinned under this proposal. The §4 trade-off table claim "Pages mlock-pinned while secret bytes are resident: YES" is false for `ResolvedSlot.entropy` / `DerivedAccount.entropy`.

The user said they've been "thrashing all day on Phase 3a." This re-scope is not the path forward as drafted, but it might be after fold. There are two coherent paths:

**Path A (proposal-style, narrower threat model):** Accept that Cycle B does NOT pin the toolkit-derived BIP-39 entropy buffers in Sites 2/3. Pin the input strings (Site 1) and the bip85-derived entropy (Site 4) only. Update SPEC §1, §2 row 5, §6 G1 to clearly state the narrower threat-model coverage (not "equivalent to R0 v2 LOCK"). Defer struct-sibling pins AND field-type migration to v0.10.1. This is what the proposal effectively does, but it must be honest about the threat-model degradation.

**Path B (return to R0 v2 LOCK direction, possibly with smaller scope deltas):** Keep struct-sibling pins on `ResolvedSlot._entropy_pin` / `DerivedAccount._entropy_pin`. Accept the Arc-wrap. Possibly defer the field-type migration (Cycle A → Zeroizing) as the only thing that gets carved out, while keeping the pin work — this would reduce scope from 19 sites to maybe 13 (the 6 ResolvedSlot construction sites + 1 DerivedAccount construction site for the pin add, without the field-type Vec→Zeroizing<Vec> + into_parts() body change). The lint anchor relabel disappears too (since impl Drop stays). This is a meaningfully smaller diff than R0 v2 LOCK without losing pin coverage.

Either path is workable; the current proposal text presents itself as Path A while implicitly claiming Path B's threat-model coverage.

---

**Files reviewed (absolute paths):**
- `/home/bcg/.claude/plans/2026-05-13-cycle-b-phase-3a-rescope-proposal.md`
- `/scratch/code/shibboleth/mnemonic-toolkit/design/agent-reports/v0_9_B-phase-3a-toolkit-applications-r0.md`
- `/scratch/code/shibboleth/mnemonic-toolkit/design/SPEC_secret_memory_hygiene_v0_9_B.md`
- `/home/bcg/.claude/plans/2026-05-13-cycle-b-handoff.md`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/derive.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/derive_slot.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/bip85.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/main.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/lib.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/mlock.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/bundle.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/verify_bundle.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/convert.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/derive_child.rs`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/synthesize.rs` (lines 580-600)
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/parse_descriptor.rs` (lines 820-955)
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/Cargo.toml`
