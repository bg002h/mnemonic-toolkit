# CONTINUITY — resume point (2026-06-12)

## LATEST: ms-codec 0.4.3 PUBLISHED + toolkit v0.54.3 SHIPPED (char-boundary panic fix propagated)
- **ms-codec 0.4.3 PUBLISHED to crates.io** (user-authorized) + tag `ms-codec-v0.4.3` @ mnemonic-secret `4d96c05`. Fixes the Cycle-C `decode_with_correction` char-boundary panic.
- **toolkit v0.54.3** @ master (pin bump ms-codec 0.4.2→0.4.3 + 1 regression cell in repair.rs) + tag `mnemonic-toolkit-v0.54.3`; rust/changelog-check/install-pin-check/sibling-pin-check all GREEN. `mnemonic repair café` no longer panics.
- **In-progress batch (user: "publish and bump then do D and E. And all the also fileds"):** DONE = publish+bump. REMAINING = ms-codec-error-display-echoes-input (secret leak in ms Error Display; R0-gated, will need its own publish+bump), toolkit-descriptor-fuzz-target (cfg(fuzzing) lib.rs mount), Cycle D (cross-tool md vs md-cli differential), Cycle E (bitcoind differential CI), fuzz-nightly-quarterly-bump (not due ~2026-09, note-only).

---


Everything below is SHIPPED and CI-green. Both repos clean, branches in sync. Safe to clear context.
Memory auto-loads the detail: `project_stress_testing_program.md`, `project_faithful_general_policy_restore_v0_54_0.md` (+ MEMORY.md index). Fable 5 for all agent/R0 dispatches; trailer `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>`.

## What shipped this session (newest → oldest)
**Restore campaign — the general-policy-collapse fix, all 3 surfaces (R0-gated, funds-safety):**
- `mnemonic-toolkit-v0.54.0` @ 2a764e0 — C1: `restore --md1` reconstructs general wallet policies faithfully (was silently collapsing → plain multisig, dropping timelocks). R0 ×3 + impl-review.
- `md-codec 0.35.1` PUBLISHED to crates.io (descriptor-mnemonic @ 762a4f8/69b7a74) — Check double-wrap fix (PART 2).
- `mnemonic-toolkit-v0.54.1` @ 9533fba — pin bump → pk(@N)/pkh(@N) flagship reconstructs (zero toolkit code change).
- `mnemonic-toolkit-v0.54.2` @ 3f4c66f — C2: `export-wallet --from-import-json` refuses general policies for template formats (same collapse, 2nd door).

**Earlier:** `mnemonic-toolkit-v0.53.9` @ 5d599f7 (BIP-68 `older()` mask funds-safety); zeroize-lint completeness @ a7c1920.

**Stress-testing program (6 cycles, 2 done):**
- **Cycle A SHIPPED** @ 9d3da6c (NO-BUMP) — backup→restore property test (`tests/prop_backup_restore_roundtrip.rs`). proptest typed-template generator + 3 oracles (structural AST / md1 fixed-point / rust-miniscript address differential). R0 ×2. Found a real bug run #1 (sortedmulti-in-combinator engrave-but-can't-restore → FOLLOWUP `bundle-accepts-sortedmulti-in-combinator-restore-cannot`).
- **Cycle B SHIPPED** @ descriptor-mnemonic `3ec324c` (NO-BUMP, test-only; toolkit companion @ e33c147), CI green both repos — md-codec proptest expansion: W tier (full wire domain × nesting × TLV randomization → P1/P2/P4/P5) + T tier (typed correct-by-construction grammar w/ TLV xpubs → NEW P6 to_miniscript/reparse/derive leg, P7 clean-refuse, P8 encoder-side). R0 ×4 + impl review ×2 (all persisted to descriptor-mnemonic `design/agent-reports/cycle-b-proptest-expansion-*`). FOUND run #1: **upstream rust-miniscript 13.0.0 depth-2 taptree Display/parse asymmetry** (FOLLOWUP `upstream-miniscript-taptree-depth2-display-asymmetry`, md-codec repo; flip cell + T-gen depth≤1 constraint; toolkit pins rev 95fdd1c — exposure unverified, check before mirroring). Also filed `encode-accepts-k-greater-than-n` (both repos; encoder accepts k>n that decode rejects).

## Cycle C COMPLETE — cargo-fuzz malformed-input (R0 ×3 GREEN; 3 phases shipped, CI green, each impl-reviewed GREEN)
- **Phase 1 md-codec** @ descriptor-mnemonic `e0d0b12` (4 targets). **Phase 2 ms-codec** @ mnemonic-secret `493c5de` (3 targets incl. ms1_no_secret_leak). **Phase 3 mk-codec** @ mnemonic-key `21786dc` (2 targets). Per-repo `fuzz/` workspaces, NO-BUMP. Brainstorm + reviews: `design/BRAINSTORM_stress_cycle_c_fuzzing.md` + each repo's `design/agent-reports/cycle-c-fuzzing-*`.
- **FOUND run #1 (ms phase) → FIXED: `ms_codec::decode_with_correction` PANIC on a single 0xaa byte** (char-boundary slice at decode.rs:151). **Fixed @ ms-codec 0.4.3 (mnemonic-secret `4d96c05`, CI green):** slice at `rfind('1')` (ASCII boundary) + whole-string got when no separator. TDD + mini-R0 GREEN. ms1_decode RE-ENABLED in smoke. ms-cli pin →=0.4.3. **PENDING USER AUTH: crates.io publish of ms-codec 0.4.3 + toolkit pin bump** (so `mnemonic`/`ms repair` get the fix; in-repo fix complete).
- **Open FOLLOWUPs from this cycle:** `ms-codec-error-display-echoes-input` (ms Error Display leaks secret share via codex32 InvalidChecksum{string}+WrongHrp{got}; toolkit already withholds via v0.53.4 friendly-mapper — its own cycle); `toolkit-descriptor-fuzz-target` (descoped; needs cfg(fuzzing) lib.rs mount + own mini-R0); `fuzz-nightly-quarterly-bump` (constellation-wide, ~2026-09).
- **CI GOTCHA (carry forward to any fuzz work):** each `fuzz-smoke.yml` MUST pin `--target x86_64-unknown-linux-gnu` + `targets:` in dtolnay step (cargo-fuzz defaults to musl host on the runner → ASan fails on static libc).

## NEXT — stress Cycles D–E (each its own R0-gated cycle)
1. **Cycle D** — cross-tool differential: toolkit `md` vs `md-cli` `md` on the same descriptor → identical md1 / `wallet_policy_id`. Surfaces the KNOWN divergence (toolkit's `tap_context` gate in `parse_descriptor.rs` disagrees with md-cli's unconditional-collapse on wsh-miniscript bare-key shapes).
2. **Cycle E** — Bitcoin Core differential CI job (pinned `bitcoind`: `deriveaddresses`/`getdescriptorinfo`). Heaviest infra.

## Open backlog (all filed in design/FOLLOWUPS.md, none funds-critical)
- `bundle-accepts-sortedmulti-in-combinator-restore-cannot` (Cycle-A find).
- Cost-layer `compare-cost` hash-position scan bugs (I-1/I-2 from the fragment review).
- hash256/ripemd160/hash160 test backfill; md-codec A2 (shape-C `Check(or_i(pk_k,pk_k))` rendering).
- GUI paired-PR for the `restore --json` `wallet_type`/nullable-`threshold` wire-shape change.
- `archetype-older-blocks-flag-accepts-time-units`, `intake-surfaces-accept-masked-older-no-advisory`, `addresses-restore-passphrase-not-zeroizing`.

## To resume
Start a fresh session in this repo (memory auto-loads) and say:

    Read CONTINUITY.md and project_stress_testing_program.md, then start stress Cycle C (R0-gated).

(Or substitute any other cycle / backlog item.) Standing rules: mandatory R0 architect gate to 0C/0I before implementing; Fable for agent dispatches; stage paths explicitly; grep-verify citations at write time.
