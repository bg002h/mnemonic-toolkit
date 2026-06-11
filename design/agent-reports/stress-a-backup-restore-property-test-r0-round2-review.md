# R0 Review — stress-A backup-restore property test — ROUND 2

**Source SHA:** `3f4c66f` (toolkit v0.54.2). **Verdict: 🟢 GREEN — 0 Critical / 0 Important / 3 Minor.**

## Round-1 findings — fold verification
- **C1 RESOLVED.** Pipeline is concrete-watch-only and verified end-to-end: `ir.rs:21-22/:218-243` (account-level `[fp/path]xpub` + `/<0;1>/*`) → matches `key_regex`/not `at_n_probe` (`pipeline.rs:37-43`) → `classify_descriptor_form` = Concrete → `bundle.rs:318-320` → `bundle_run_concrete_descriptor` (`:1652`, no slots, MultisigWatchOnly, full md1). Origin-annotation requirement captured (`pipeline.rs:141-145`). Failure policy stated load-bearingly; coherent at this base (the one loud-refusal class `to-miniscript-check-pkh-double-wrap` is RESOLVED at v0.54.1; md1 encode covers all `Terminal::*`). Strongest evidence: the shipping `assert_md1_fixed_point` helper already runs `bundle --descriptor <concrete>` no-slots → byte-identical md1 — the pipeline generalizes green test code.
- **C2 RESOLVED.** Oracle-3 derives from the ORIGINAL `desc`; restore's addresses come from `desc'` (`restore.rs:1108-1115`) → a wrong `desc'` (C1 collapse) → wrong wsh address ≠ original-desc address → fails. Genuine input differential.
- **I1 RESOLVED.** Typed-template primary + fresh-key allocator + `max_global_rejects=cases/20` + fragment-coverage accumulator. Gate rules verified exact (SiglessBranch `:372`, Malleable `:375`, RepeatedPubkeys `:379`, HeightTimelockCombination `:382`, cap `:33`). Timelock domains match the v0.53.9 mask gate EXACTLY. One-class-per-tree handles `has_mixed_timelocks`.
- **I2 RESOLVED.** Oracle-1 structural AST-modulo-keys; `multi`↔`sortedmulti` + `sha256`↔`hash256` are distinct AST kinds → caught, no substring hole. Fingerprint-slot erasure preserves multi-order binding while tolerating depth-0 re-serialization.
- **I3 + M1-M5 RESOLVED.** Permanent oracle self-test cells (5 known-bad pairs incl. change-half corruption); negative property via `@N` slot pipeline anchored to shipping cells (`cli_restore_multisig_general.rs:280-304`); general-taproot refusal `restore.rs:651-674`; TempDir/stdin/64-cases; CI seed posture; older()-mask honesty note.

## New-drift / blind-spot sweep
No GREEN-but-vacuous path: generator collapse fails loudly (`max_global_rejects`) or trips the coverage accumulator; oracle decay caught by permanent self-test cells; no wrong-`desc'` bug class passes all three oracles (any structural change → different witness script → different wsh hash → different Oracle-3 address). The only shared-fate residual is a bug inside the pinned miniscript itself (inherent to a same-library differential; deferred to Cycle E). Bring-up proof still valid (`restore --md1` → `run_multisig`; the faithful-arm revert is exactly what the concrete watch-only pipeline exercises).

## Minor (folded without re-review)
1. Oracle-3 chain-1 wording — restore reports chain-0 only; chain-1 asserted via desc-vs-desc' differential (guarded by the change-half self-test cell). **Folded.**
2. Bound `multi`/`sortedmulti` n ≤ 20 (Segwitv0 CHECKMULTISIG cap). **Folded.**
3. Case-count: 64 (not 128). **Folded.**

## Verdict
🟢 GREEN — ready for implementation. The worst outcome (a green test that proves nothing) is foreclosed by three independent mechanisms: loud reject budget + coverage accumulator, permanent oracle self-test cells, and the desc-vs-desc' address differential (pass requires script-hash equality).
