# Phase (Tasks 1+2 impl) — two-stage review (verbatim) — experimental depth-≥2

> Persisted verbatim per CLAUDE.md BEFORE the fold. Subagent-driven two-stage review of impl commits
> `24e3a02` (enabler) + `b1da772` (gate lift), diff base `5d04ec0`. EXPERIMENTAL never-merge POC.
> **Spec-compliance = ✅ compliant** (all 5 points, 22 tests pass, scope clean, +1 doc-accuracy flag).
> **Code-quality = Approve with minor fixes (0C/0I/3m).** Both confirm: NO reachable silently-wrong
> depth-≥2 reconstruction. M1-M3 (stale docs) folded in a polish commit.

---

## Stage 1 — Spec-compliance: ✅ COMPLIANT

All 5 spec points verified in source: (1) rev bump `ff4732e` in `Cargo.toml`+`fuzz/Cargo.toml` + both lockfiles + EXPERIMENTAL banner + `EXPERIMENTAL.md`; (2) `ensure_taptree_depth_le_one` → `ensure_taptree_wellformed` (recursive, malformed-tree refusal kept, depth cap removed, call site updated, old name fully gone); (3) `taproot_is_deep` + advisory at `:1392-1394` guarded `is_taproot && taproot_is_deep(&d.tree)`, contains "EXPERIMENTAL", after the fidelity guard; (4) retained nets `subtree_contains_sortedmulti_a` (`:745/:805`), Display-fidelity guard (`:1378`), `refuse_at_in_both` (`:781`) all untouched; (5) 2 cells flipped + genuine asymmetric depth-3 cell with distinct K3, NUMS-rooted goldens. Scope: exactly 7 files, nothing else touched. `cargo test --test cli_restore_taproot` → 22 passed. **One flag:** the stale `Cargo.toml` comment body (see code-quality M1).

## Stage 2 — Code-quality: Approve with minor fixes

### Strengths
- **Funds-safety sound:** the Display-fidelity guard (`:1378`, `parsed.to_string() != descriptor`) fires strictly after `faithful_multisig_descriptor` + parse, so any depth-≥2 shape `ff4732e` still mis-renders is caught — no silently-wrong address escapes.
- `ensure_taptree_wellformed` correct: terminates at non-TapTree leaves, validates exactly-2-children at every TapTree, no panic/unwrap, ~128 stack frames worst case (fine).
- `taproot_is_deep` correct: false for Template (`inner.tag` MultiA/SortedMultiA), keypath-only (`tree: None`), non-Tr, depth-1; true for depth-≥2. R0-r2 m1 (extract `Body::Tr{tree:Some(inner),..}` first) implemented.
- Advisory placement correct (after the fidelity guard, guarded, non-blocking).
- Test quality solid: 3 depth cells assert success + golden + `stderr(contains("EXPERIMENTAL"))`; depth-3 genuinely exercises a 3rd recursion level; K3 distinct; goldens capture-once-from-binary; pre-existing refusal cells stay green.
- EXPERIMENTAL marking adequate (banner + EXPERIMENTAL.md + runtime advisory). No scope creep.

### Issues
**Critical — None. Important — None.**

**M1 (conf 95) — `Cargo.toml:16-31` stale body contradicts the banner.** `:17` "Pinned to … 95fdd1c" (rev is now ff4732e); `:21-22` "PREDATES #953 … depth-≥2 stays refused (`ensure_taptree_depth_le_one`)" (false: rev has #953, depth-≥2 reconstructs, fn renamed); `:23-26` "per advisor we hold" (reversed by this commit). **Fix:** replace `:16-31` with a concise accurate body (ff4732e carries #910/#915/#953; depth-≥2 reconstructs on this branch).

**M2 (conf 85) — `restore.rs:67-68` clap `--md1` help still says "depth-≥2 taproot are refused."** User-visible in `--help`. **Fix:** "depth-≥2 reconstructs with an EXPERIMENTAL advisory (branch-only; unreleased miniscript pin)."

**M3 (conf 85) — `restore.rs:1280-1281` inline comment "depth ≥2 … → loud structural refusals"** (now reconstructs via GeneralFaithful) and `:1276` "the toolkit's own miniscript rev 95fdd1c HAS Terminal::SortedMultiA" (rev is now ff4732e). **Fix:** update both.

### Silently-wrong answer: **No.** No reachable depth-≥2 shape reconstructs wrong-but-parse-stable. `faithful_multisig_descriptor` → `to_miniscript` (ff4732e Display, #953-fixed) → fidelity guard round-trips. Template path Display-stable by construction + never reached at depth-≥2. @-in-both unreachable at depth-≥2 (correct). `subtree_contains_sortedmulti_a` recurses, catches sortedmulti_a at any depth.

### Assessment: Approve with minor fixes. All 3 minors are stale-doc/help-text; M1 (self-contradictory Cargo comment) + M2 (user-visible --help) worth a polish commit. None block the POC.
