# R0 GATE REVIEW — IMPLEMENTATION_PLAN_cycleA_descriptor_use_site_collapse.md — Round 1

**Reviewer:** opus architect. **Against:** `origin/master @ 8c8b9183`. Read-only, adversarial funds-safety posture. **SPEC:** R0-GREEN rev-2. Persisted verbatim per CLAUDE.md.
**Verdict:** NOT GREEN — 0 Critical, 4 Important.

## Independent verification (checks out)
- **Phase-1 residue snippet correct + panic-safe.** Byte-logically identical to md-cli `template.rs:128-137`; placed after multipath validator (`.transpose()?` at :177 propagates H13 first), before `out.push` (:183). No UTF-8 panic: `caps.get(0).end()` is always a char boundary; `.chars().next()`/`.take(24)` are codepoint-wise. `i` in scope (:105). No off-by-one.
- **Every reject SHAPE has a dedicated Phase-1a test** (8 shapes enumerated). Positive controls all lex-pass (no false-reject).
- **export-wallet/compare-cost bypass REAL** — `export_wallet.rs` parses via `MsDescriptor::from_str` (:517/:633/:796), never `lex_placeholders`. `descriptor_to_bip388_non_multipath_refused` (exit 1) + `export_wallet_originless_concrete_still_accepted` genuinely STAY-PASSING.
- **No card-dedup → Group-B counts survive** — `:898` asserts `bundles=2` today from a same-key `/0/*`+`/1/*` pair that both collapse to byte-identical cards, yet counts 2 ⇒ one bundle per Core entry, no dedup.
- **Spot-checked buckets:** a4/a5 genuinely Group A; `homogeneous_two_mainnet_blob_override_mainnet_ok` genuinely Group B (network-override); `fires_and_non_blocking_bundle_concrete_key` Group B (advisory).

## CRITICAL
None. Residue floor is fail-closed; no path in this plan produces a card.

## IMPORTANT
### I-A. Phase 1 lands a knowingly-RED full suite → violates per-phase-GREEN gate. MERGE Phase 1+2 into ONE atomic phase.
The residue check and the 22 cells it flips are inseparable; committing the check without migrating the cells leaves `cargo test -p mnemonic-toolkit` RED at the phase boundary. "Expect it to be red, that's correct" is the pre-authorization the gate forbids, and forces the phase-1 R0 to hand-distinguish "expected red" from "regression red" across 22 cells. Fold: one atomic phase — (1) write all reject tests → red; (2) implement residue check → new rejects green, 22 incumbents red; (3) migrate all 22 → full suite GREEN; (4) then per-phase R0. Renumber downstream.

### I-B. Phase-3a asserts the WRONG verify-path error variant for the concrete descriptor (the actual false-pass site). It is `DescriptorParse`/exit 2, not `DescriptorReparseFailed`/exit 4.
SPEC §1 cites the false-pass site as `verify_bundle.rs:1352-1357` (concrete fork). A concrete `wpkh([fp/84'/0'/0']xpub…/0/*)` verify hits `classify_descriptor_form == Concrete` → `descriptor_concrete_to_resolved_slots(body_no_csum)?`, which re-wraps the lex reject as `DescriptorParse` (`pipeline.rs:417-418`) → exit 2. The `.map_err(DescriptorReparseFailed)` at `verify_bundle.rs:1375` is only the `@N`-TEMPLATE verify path. M-3's blanket claim is FALSE for the concrete path. Fold: concrete `/0/*` verify → exit 2 / `DescriptorParse` (primary false-pass regression, matches SPEC §1); `@N`-template → exit 4 / `DescriptorReparseFailed` (optional). Funds-safety intact (reparse rejects before card compare); only the assertion shape is wrong. Correct M-3.

### I-C. Plan reclassifies `:898` to a Group-B `<0;1>` swap — contradicts the GREEN SPEC's reject-flip, orphans the legacy-split reject shape, AND destroys the pair-merge follow-up's input fixture.
SPEC §8 + R0-round-2 both say `:898` FLIPS to reject. `core-mainnet-receive-change-pair.json` is the ONLY fixture modelling a raw legacy Core same-key `/0/*`+`/1/*` split; swapping its body to `<0;1>` deletes the "a real Core split pair is now rejected" coverage AND the pair-merge follow-up needs exactly this fixture as its future merge INPUT. Fold: follow the SPEC — flip `:898` to a reject assertion (exit≠0 + bitcoin-core workaround message), keep the fixture UNCHANGED, remove it from the swap list. Makes `:898` the canonical legacy-split funds regression + preserves the follow-up's input.

### I-D. The `/**` hard-fail (a MAINSTREAM `--format descriptor` form) is under-disclosed.
`/**` reaches the lexer un-expanded (`concrete_keys_to_placeholders` pushes `/**)` verbatim, `pipeline.rs:401`) → `wpkh(@0[fp…]/**)` → wild eats `/*`, residue `*` → REJECT. No pre-lexer `/**`→`<0;1>` expansion exists. `/**` is the standard BIP-389 combined shorthand (Sparrow/Nunchuk/Core `doc/descriptors.md`), so `import-wallet --format descriptor …/**` — common, TODAY silently collapses to a wrong card — now hard-fails. Correct + funds-safe, but transitioning a high-frequency form success→hard-fail with no release note (while disclosing narrower cases) is a material gap. Fold: add `/**`-hard-fail disclosure (+ `<0;1>/*` workaround) to CHANGELOG §4b AND manual §4a; add a CLI-level `--format descriptor` `/**` reject regression; note `bip389-double-star-shorthand-support` may be higher-impact than the pair-merge. (Sparrow NOT affected — self-expands `@i/**` before lexing.)

## MINOR
- **M-a.** §2b names 4 `.json` fixtures but several Group-B cells build descriptors INLINE (`build_core_multi`/`build_core_single` `d0=…/0/*`; the two `cli_older_advisory.rs` cells) with no fixture file — the implementer must swap in-body LITERALS. State explicitly. (`active_receive`/`active_change` fixtures become semantically odd post-swap but mechanically preserve the `active && !internal`/`active && internal` assertions — acceptable.)
- **M-b.** SPEC §8:178 says `descriptor_to_bip388_non_multipath_refused` "now rejects earlier, exit 2 not 1" — WRONG (export-wallet bypasses the lexer → STAYS exit 1). Plan/sweep correctly bucket it STAYS-PASSING. Note the SPEC line is superseded.
- **M-c.** M-9(ii) sparrow-passthrough discharge not written into the plan. Risk nil (sparrow self-expands `@i/**`→`<0;1>/*`; taproot-multisig passthrough embeds `[fp/path]xpub/<0;1>/*`) but record it + add a Sparrow taproot-multisig-passthrough positive-control test (over-rejection guard).
- **M-d.** PATCH bump conflicts with precedent: this cycle turns previously-accepted imports into hard failures (breaking, user-visible); prior funds-CRITICAL cycle (bughunt) shipped MINOR (v0.61.0/v0.62.0); under 0.x semver a breaking change is MINOR. Reconcile — MINOR, or justify PATCH.
- **M-e.** a4/a5 have TWO reject sites (direct-`--descriptor` AND `walletfile_to_bundle` legs); plan reads as descriptor-side only. State both assert the reject (wallet-file leg via the bitcoin-core workaround message).

## Rulings on the 4 open items
1. (a) no-weakening MOSTLY FAITHFUL — every reject shape has a dedicated test, EXCEPT `:898` (I-C). Faithful once `:898` flips to reject.
2. (b) fixture-swap CORRECT (counts survive; no dedup; internal-based selectors survive) — clarify inline-blob cells (M-a).
3. (c) `/**` reject-not-expand ACCEPTABLE + REQUIRED (same collapse class) — but DISCLOSE (I-D).
4. (d) Phase-1→2 RED window NOT ACCEPTABLE — MERGE (I-A).

## VERDICT
**NOT GREEN — 0 Critical, 4 Important.** Residue floor correct/panic-safe/fail-closed. Fold I-A (merge phases), I-B (`DescriptorParse`/exit 2 concrete verify), I-C (flip `:898` to reject, keep fixture), I-D (disclose `/**` + CLI test), the 5 Minors; persist; re-dispatch round 2.
