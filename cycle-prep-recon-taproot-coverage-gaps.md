# cycle-prep recon — 2026-06-12 — taproot coverage gaps (GAP 1, 4 sub-parts)

**Origin/master SHA at recon time:** toolkit `ca7d7bc` (mnemonic-toolkit `origin/master`); md-codec `422b049` (descriptor-mnemonic `origin/main`)
**Local branch:** `master` (toolkit), `main` (descriptor-mnemonic)
**Sync state:** both up-to-date (toolkit 0 ahead / 0 behind)
**Untracked:** recon/scratch files only (cycle-b-*, cycle-prep-recon-*, CONTINUITY.md, .claude/)

Sub-parts verified: (1) `md-codec-sortedmulti-a-to-miniscript-rendering-gap`, (2) `upstream-miniscript-taptree-depth2-display-asymmetry`, (3) toolkit restore general-tr-leaf refusal untested, (4) toolkit multi-leaf taptree AST-walk-only. Expectation: line drift in the FOLLOWUP bodies (filed pre-Cycle-B test expansion); two NEW load-bearing findings (toolkit pinned rev shares the Display bug; the "lower to sorted multi_a" close is semantically unsafe as worded).

---

## Per-sub-part verification

### Sub-part 1 — `sortedmulti_a` unrenderable in md-codec (`md-codec-sortedmulti-a-to-miniscript-rendering-gap`)

- **WHAT:** md-codec `to_miniscript` unconditionally errors on `Tag::SortedMultiA`, so md-codec-native consumers (`md address`, `decode → derive_address`) cannot handle `tr(NUMS, sortedmulti_a(...))` md1s. Toolkit restore routes AROUND it since v0.49.1.
- **Citations (vs descriptor-mnemonic `422b049` / toolkit `ca7d7bc`):**
  - `crates/md-codec/src/to_miniscript.rs:~423-426` SortedMultiA error arm — **ACCURATE.** Live arm `:423-428`: `(Tag::SortedMultiA, Body::MultiKeys { .. }) => { return Err(failed("Tag::SortedMultiA must be a tap-leaf root child; rust-miniscript v13 has no Terminal::SortedMultiA fragment"...))` (message `:425`). NOTE: the FOLLOWUP's own `Where:` cites `:407-411` — **DRIFTED-by-16**; the toolkit-side mirror (`design/FOLLOWUPS.md:282`) cites `:406-410` — **DRIFTED-by-17**. `Tag::MultiA` DOES render, FOLLOWUP cite `:394-398` → live `:411-416` (**DRIFTED-by-17**).
  - P7 refusal cells `tests/proptest_to_miniscript.rs:~345,:353` — **ACCURATE.** `self_test_bad_sortedmultia_wsh_leaf` at `:345`, `self_test_bad_sortedmultia_tap_leaf` at `:353` (both `assert_p7_clean_refusal`, both cite the FOLLOWUP slug).
  - "md-codec pins crates.io miniscript 13.x lacking SortedMultiA" — **ACCURATE + sharpened.** Workspace root `Cargo.toml:18` = `miniscript = { version = "13.0.0", ... }` (md-codec consumes the workspace dep, crate `Cargo.toml:29`). **Verified against the registry source: miniscript 13.1.0 (the LATEST crates.io release — index tail `12.3.7 / 13.0.1 / 13.1.0`) has ZERO occurrences of `SortedMultiA` anywhere in `src/`.** The toolkit's pinned git rev `95fdd1c` (toolkit `Cargo.toml:16-17` `[patch.crates-io]`) DOES have it (`src/miniscript/astelem.rs`, 2 hits). So "bump the pin" is NOT a crates.io option today: PR #910's fragment is merged-but-unreleased, and md-codec is a published crate (crates.io forbids git deps) → option (a) is **BLOCKED on the next upstream miniscript release** (presumably 14.0).
  - **FOLLOWUP status:** md-codec `design/FOLLOWUPS.md:1905` — `open`, tier `next-cycle`, recommended close = option (b) "lower `sortedmulti_a` → sorted `multi_a` at build time (semantically equivalent for taproot script-path)". Toolkit mirror at `design/FOLLOWUPS.md:282` — `open`, tier `v0.5`.
- **NEW FINDING — option (b) as worded is a silent-infidelity trap.** `to_miniscript_descriptor` builds **wildcard-xpub** keys (`to_miniscript.rs:55-64`: "the trailing wildcard `/*` remains for `at_derivation_index` to resolve"). `sortedmulti_a` semantics sort the **derived** pubkeys per address index (BIP-67-style; that's why wsh `sortedmulti` is kept as `SortedMultiVec` via `new_wsh_sortedmulti`, `to_miniscript.rs:198-205`, NOT reordered at parse). A build-time sort of the xpub placeholders produces a fixed-order `multi_a` that derives **different (wrong) addresses** at any index where derived-key order differs from xpub order — exactly the silent-collapse class v0.54.x just stamped out. Correct shapes of (b): lower **per-index inside `derive_address`** (derive the concrete keys for the index, sort the serialized x-only keys, build `multi_a`); the wildcard `to_miniscript_descriptor` render stays refused until the upstream fragment ships. The brainstorm spec MUST scope (b) at the derive layer, not the converter layer.
- **Action for brainstorm spec:** re-cite arm `:423-428` / message `:425` / MultiA `:411-416` @ descriptor-mnemonic `422b049`; replace "lower at build time, semantically equivalent" with the per-index derive-time lowering; record that option (a) is release-blocked (13.1.0 verified fragment-free). Toolkit lockfile-bump + stale-message reword (the toolkit-build-config-false "v13 has no Terminal" string, toolkit `FOLLOWUPS.md:143` WONTFIX obs) ride along.
- **Verdict:** **ACTIONABLE now** as the derive-time lowering (md-codec feature, real cycle); full wildcard render **BLOCKED upstream**.

### Sub-part 2 — tr depth-2+ taptree Display/parse asymmetry (`upstream-miniscript-taptree-depth2-display-asymmetry`)

- **WHAT:** miniscript 13.x Displays a depth-2 taptree `{{a,b},c}` as the malformed `{{a,b,c}}` which its own parser rejects. md-codec wire/converter/derive unaffected; only stringification of ≥3-leaf taptrees.
- **Citations:**
  - Deterministic cell `upstream_taptree_depth2_display_asymmetry` `tests/proptest_to_miniscript.rs` — **ACCURATE**, live at `:312` (fn), asserts converter Ok + `derive_address` Ok + `bc1p` prefix (`:329`) + wire round-trip Ok + reparse `is_err()` with the "UPSTREAM FIXED?" flip message (`:333-341`). T strategy depth ≤ 1 — **ACCURATE** (`tests/common/mod.rs::t_tr_tree` doc, per the cell comment `:307-310`).
  - **FOLLOWUP status:** md-codec `design/FOLLOWUPS.md:1925` — `open`, tier `upstream`. Claims PR #953 merged 2026-05-25, NOT in any release, 13.1.0 empirically re-verified buggy 2026-06-11.
  - **Upstream re-verified 2026-06-12:** crates.io sparse index tail for miniscript = `12.3.7, 13.0.1, 13.1.0` — **13.1.0 is still the latest release; no release containing #953 exists. CONFIRMED BLOCKED upstream.**
- **NEW FINDING — the toolkit's pinned rev `95fdd1c` SHARES the bug (the FOLLOWUP's "Companion: … exposure unverified" is now resolvable).** Two independent confirmations: (i) `95fdd1c` = merge of PR #932, dated **2026-05-02** — predates #953 (2026-05-25); (ii) code-trace of `95fdd1c:src/descriptor/tr/taptree.rs:87-114` `fmt_helper`: the comma is written on `last_depth > 0` BEFORE any close-braces are emitted, so `depths_leaves = [(2,a),(2,b),(1,c)]` emits `{{a,b,c}}` — the exact 13.0.0 bug. **Toolkit exposure today is latent, not live:** restore's taproot classifier refuses anything non-multi_a/sortedmulti_a before any Display (see sub-3), and `faithful_multisig_descriptor` (the only `miniscript::Descriptor → .to_string()` reconstruction path, `restore.rs:938`) is non-taproot-only. But any future general-tr reconstruction (sub-3 feature) or ≥3-leaf descriptor re-stringification through the patched miniscript inherits it. **A toolkit companion FOLLOWUP entry is due** (the md-codec entry explicitly asked for verify-then-mirror).
- **Action for brainstorm spec:** none needed beyond filing the toolkit companion; the flip-cell mechanics on the eventual bump are already recorded in the md-codec entry. Note for the future: the same unreleased-master bump dissolves sub-1(a) AND sub-2 AND the toolkit's `[patch.crates-io]` (its comment already says "drop when a crates.io release publishes containing #910 + #915").
- **Verdict:** **PARKED ON UPSTREAM** (watch for the first release > 13.1.0 containing #953). Only the companion-filing is do-now.

### Sub-part 3 — general taproot leaf restore-refusal UNTESTED (toolkit)

- **WHAT:** `restore --md1` on a taproot md1 handles only NUMS + `multi_a`/`sortedmulti_a` leaves; the non-NUMS arm and the unrecognized-leaf arm refuse loudly but have zero test coverage.
- **Citations (vs toolkit `ca7d7bc`):**
  - `src/cmd/restore.rs:~685-689` non-NUMS refusal — **ACCURATE.** `Body::Tr { is_nums: false, .. }` arm at `:685`, `ModeViolation` message at `:689` ("taproot multisig md1 with a non-NUMS (cosigner) internal key is not supported by restore yet …").
  - `src/cmd/restore.rs:~710` unrecognized-leaf refusal — **ACCURATE.** Leaf-tag match `:703-712`; `:710` = "taproot md1 leaf is not a recognized multisig (multi_a / sortedmulti_a)". Multi-leaf trees hit this same arm: a branch node is `Tag::TapTree` (md-codec `tag.rs:27`, wire 0x05), which falls to the `_` arm.
  - **Zero test hits — CONFIRMED.** `grep -rn "not supported by restore\|recognized multisig\|non-NUMS" crates/mnemonic-toolkit/tests/` → only `cli_compare_cost.rs` hits (a different surface: the compare-cost non-NUMS keypath advisory). `cli_restore_multisig*.rs` covers only the POSITIVE tr-multi-a / tr-sortedmulti-a reconstruction cells (`cli_restore_multisig.rs:251-289`, `cli_restore_multisig_format.rs:311-321`).
- **NEW FINDING — v0.54.0's general-policy faithful path does NOT reach taproot, sharpening the gap.** The §3 classifier (`restore.rs:1083-1089`) routes **every** `Tag::Tr` md1 through `taproot_template_and_internal_key(&d.tree)?` — the `?` propagates the refusal — so the v0.54.0 `faithful_multisig_descriptor` arm (general wsh policies, `template_opt = None`) is **non-taproot-only**. A `tr(NUMS, and_v(...))` md1 — which the v0.19.0 intake accepts and `bundle` happily engraves — loud-refuses on restore. Loud (good, not the silent-collapse class), but an engrave-vs-restore asymmetry pinned by nothing.
- **Two distinct scopes — keep separated:**
  - **(3a) Refusal-contract tests — CHEAP, NO-BUMP.** Three cells: (i) `is_nums:false` md1 → exit 2 + `:689` message (must be constructed via direct `md_codec` encode in the test — `bundle` cannot emit `is_nums:false` since v0.48.0; toolkit tests already dep md_codec); (ii) `tr(NUMS, <single-leaf general ms>)` via `bundle --descriptor` → restore → exit 2 + `:710` message; (iii) multi-leaf `tr(NUMS,{pk(@0),pk(@1)})` md1 → restore → same `:710` arm (pins the Tag::TapTree fall-through).
  - **(3b) Faithful general-tr reconstruction — REAL FEATURE.** md-codec round-trips general tr leaves fine (the depth-2 cell asserts wire + converter + `derive_address` all Ok, even at depth 2; `to_miniscript` handles `Body::Tr { is_nums: true }` via the NUMS const, `to_miniscript.rs:33-34`), so the converter is NOT the blocker. The blocker is the **string render**: `faithful_multisig_descriptor` emits via `translated.to_string()` (`restore.rs:938`) = miniscript Display on the bug-sharing `95fdd1c` → **depth ≥ 2 (≥3-leaf) trees would emit malformed unparseable descriptors**. Scope: extend the classifier to route general tr leaves (single-leaf, and 2-leaf depth-1 trees — both Display-safe) to the faithful arm, with a LOUD refusal carve-out for depth ≥ 2 citing the upstream FOLLOWUP. `is_nums:false` handling can ride along or stay deferred (decide at brainstorm). `sortedmulti_a`-INSIDE-a-general-tree stays refused (md-codec converter, sub-1).
- **Verdict:** (3a) **ACTIONABLE NOW** (toolkit, test-only, NO-BUMP). (3b) **ACTIONABLE with a depth≥2 carve-out** (toolkit PATCH per the v0.54.1 precedent; manual prose update — the restore chapter currently says NUMS multisig only).

### Sub-part 4 — multi-leaf/depth-N taproot: AST-walk-only (toolkit)

- **WHAT:** the descriptor-intake walker handles multi-leaf taptrees (so `bundle` can engrave them), but no integration bundle/restore/verify/address test exists; `compare-cost` refuses multi-leaf.
- **Citations:**
  - `src/parse_descriptor.rs:~2634-2699` `walk_tap_tree_{2,3,4}_leaf_*` — **ACCURATE.** Unit tests at `:2634` (2-leaf balanced), `:2652` (3-leaf asymmetric), `:2676` (4-leaf balanced), `:2699` (4-leaf right-spine), under the "Phase F (v0.4): walk_tap_tree multi-leaf round-trips" header `:2618`. The walker itself: `walk_tap_tree` at `:497` (called from the intake at `:476`).
  - `tests/cli_compare_cost.rs:~853` refusal — **ACCURATE.** `tr_descriptor_multi_leaf_refused_exit_3` at `:853` (exit 3 + MultiLeafTr message, header note `:700`).
  - Umbrella slug `miniscript-beyond-bip388` — **ACCURATE.** Toolkit `design/FOLLOWUPS.md:1914`, `resolved 087d0e4` (v0.19.0, 2026-05-17); its `What:` names "arbitrary `tr` taproot trees with multi-leaf miniscript" in-scope; the resolution shipped the INGEST side only. No open slug tracks the multi-leaf round-trip gap → if pursued, a new entry is due.
  - "No integration test" — **CONFIRMED.** `grep -rn "multi-leaf\|MultiLeaf" crates/mnemonic-toolkit/tests/` hits only `cli_compare_cost.rs`; no bundle/verify-bundle/restore/address test constructs a multi-leaf taptree descriptor.
- **Blocked-or-independent analysis:** the wire side is **independently testable now** — bundle (walker, unit-tested), md1 wire round-trip, verify-bundle, and md-codec address derivation all avoid the string form (the depth-2 cell proves converter+derive+wire Ok at depth 2). The **restore** leg lands on sub-3's `:710` refusal (test it as a contract now; reconstruction = sub-3b, with the depth≥2 carve-out from sub-2). The **sortedmulti_a-leaf** variant of a multi-leaf tree additionally needs sub-1. So: integration round-trip test (bundle → verify-bundle → restore-refusal) = NOT blocked; full restore round-trip for depth≥2 = blocked on upstream Display; sortedmulti_a leaves = blocked on sub-1.
- **Verdict:** **ACTIONABLE NOW** as NO-BUMP integration tests (fold into the same cycle as 3a). Full depth-N restore parity is downstream of sub-2/sub-1.

---

## Dependency graph

```
upstream miniscript release containing #910 (Terminal::SortedMultiA) + #953 (taptree Display fix)
  ├── unblocks sub-1 option (a): md-codec full wildcard sortedmulti_a render  [PARKED]
  ├── unblocks sub-2: flip the md-codec characterization cell, restore t_tr_tree depth-2 arm  [PARKED]
  ├── unblocks sub-3b depth≥2: faithful general-tr reconstruction without the carve-out  [PARKED part]
  └── lets toolkit drop its [patch.crates-io] (Cargo.toml:12-17 comment already plans this)

sub-1 option (b) per-index derive-time lowering (md-codec)   — independent, actionable now
sub-3a refusal-contract tests (toolkit)                      — independent, actionable now
sub-4 bundle/verify integration tests (toolkit)              — independent, actionable now; restore leg = sub-3a's :710 contract
sub-3b general-tr reconstruction, single-leaf + depth-1 only — actionable now; depth≥2 arm BLOCKED by sub-2;
                                                               sortedmulti_a-in-general-tree leaf BLOCKED by sub-1
sub-2 toolkit companion FOLLOWUP filing                      — actionable now (verification done in this recon)
```

Multi-leaf round-trip does NOT depend on the depth-2 Display fix for wire/verify/address (none transit the string); ONLY the restore-emitted descriptor string does.

---

## Recommended scope

**Cycle T1 — cheap do-now (toolkit, test-only + docs, NO-BUMP):**
- 3a refusal contracts: `is_nums:false` (direct md_codec-encode fixture) + general-single-leaf + multi-leaf, pinning exit codes + messages of `restore.rs:689`/`:710`.
- 4: bundle → verify-bundle multi-leaf taptree integration round-trip (2- and 3-leaf shapes).
- File the toolkit companion FOLLOWUP for `upstream-miniscript-taptree-depth2-display-asymmetry` (95fdd1c verified bug-sharing: merge-date 2026-05-02 < #953 2026-05-25 + the `fmt_helper` comma-before-close trace) and a new toolkit slug for the multi-leaf round-trip gap (the v0.19.0 umbrella is resolved and can't carry it). Cross-cite the md-codec entry per the Companion convention.
- No schema_mirror, no manual lockstep (no CLI surface change). ~150-250 LOC of tests.

**Cycle T2 — real feature (md-codec): sortedmulti_a per-index derive-time lowering.**
- Lower `SortedMultiA` to a sorted concrete `multi_a` **inside the per-index derive path only** (NOT a build-time xpub sort — silent-wrong-address trap, see sub-1 finding); wildcard `to_miniscript_descriptor` keeps refusing until upstream releases the fragment. Reword the now-build-config-false "v13 has no Terminal" message while touching the arm. Invert/extend the two P7 self-test cells. md-codec SemVer: PATCH-or-MINOR (precedent 0.35.1 turned refusals into renders as PATCH); crates.io publish + toolkit lockfile-bump tail (user authorization). Updates BOTH repos' FOLLOWUP entries in lockstep.

**Cycle T3 — real feature (toolkit): faithful general-tr restore reconstruction (3b).**
- Route `Tag::Tr` general leaves (single-leaf + depth-1 two-leaf) to the faithful arm; LOUD depth≥2 refusal citing the upstream slug; decide `is_nums:false` inclusion at brainstorm. Toolkit PATCH (v0.54.1 precedent). Manual prose lockstep (restore chapter's "taproot NUMS multisig only" claim). Order T3 after T2 if `sortedmulti_a`-bearing general trees should reconstruct too; otherwise independent.

**Parked on upstream:** sub-2 proper + sub-1(a) + dropping the toolkit `[patch.crates-io]` — all dissolve together on the first miniscript release > 13.1.0 containing #910/#915/#953. Watch the crates.io index; flip cells per the md-codec entry's "Actions on close".
