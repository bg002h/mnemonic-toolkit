# WHOLE-DIFF ADVERSARIAL REVIEW — cycle-11a GUI hygiene (M9 + L12 + L13)

Post-implementation mandatory review (worktree `wt-cycle11a`, off `mnemonic-gui origin/master = 0bbe3e1` = v0.45.0; ships v0.46.0).

## VERDICT: GREEN (0 Critical / 0 Important / 0 Minor)

Ship-ready. All five axes verified; full workspace suite + clippy GREEN; scope clean.

### Axis 1 — M9 secret hygiene (deepest scrutiny) — CLEAN
- **Completeness:** `TreeNode::zeroize_keys` (`tree_model.rs:258-267`) zeroizes `key` + every `keys[i]` and recurses ALL `children` (no `.take(arity)` → covers surplus children, which DO hold secrets). Enumerated every `TreeNode` field (`id`/`kind`/`key`/`k`/`keys`/`n`/`hex`/`w`/`children`): `key`+`keys` are the only private-key inputs; `k`/`n`/`w`/`kind`/`id` structural; `hex` correctly excluded.
- **`hex` is genuinely public** (struct doc `tree_model.rs:93-97`; on-disk redactor `blank_non_extended_public_keys` also leaves it untouched — exact parity).
- **No other missed secret surface:** `TreeState.validate_ok.descriptor` can NEVER hold a private key (the toolkit's `build-descriptor` Validate path is watch-only, REFUSES private keys with a `secret_key` diagnostic, emits NO `descriptor`). `state.tree` is the only `TreeState` in `FormState`.
- **Effective scrub:** zeroize 1.8.2 `String::zeroize` overwrites heap bytes then zeroes len. Unconditional zeroize (vs the on-disk conditional xpub-skip) is strictly more thorough for an exit sweep.
- **No panic/borrow risk:** recursion bounded by `MAX_TREE_DEPTH=64`; `state.tree.as_mut()` wiring no borrow conflict.
- **RED non-vacuous:** disabling the wiring → test FAILS at `tests/secrets.rs:398` (`root.key.is_empty()`). Asserts the scrub (root.key, all keys[i], nested child.key) AND the hex-untouched invariant.

### Axis 2 — L12 canonicity regex — CLEAN
- 16-case adversarial probe (anchored semantics): the new second optional bracket REJECTS short/non-hex fp, triple brackets, second `@N`, empty `@N`, bracket-after-multipath, trailing junk, negative index. Only deliberate over-acceptance is double-origin `[fp]@N[fp]` — settled benign (v0.60.0 accepts; master refuses at parse).
- Prefix/no-origin/suffix/multipath all still match; multisig regexes (`:116`/`:118`) untouched.
- **No capture-renumbering risk:** `conditional.rs` uses only `is_match` (zero capture extraction).
- `canonicity_drift.rs` fixture + count (18→19, 15→16 classify) correct; GREEN against the pinned v0.60.0 binary (verified the binary reports `0.60.0`).

### Axis 3 — L13 dropdown split — CLEAN
- `CONVERT_FROM_NODES` (14, seedqr@1) → `--from` (`mnemonic.rs:1140`); `CONVERT_TO_NODES` (13, seedqr-free) → `--to` (`:1150`). Both byte-exact vs toolkit `NodeType::as_str` + `--to PossibleValuesParser`.
- Empirical: `--from seedqr=...` accepted, `--to seedqr` rejected — asymmetry matches toolkit semantics.
- `schema_mirror` (21) + `schema_mirror_secret_drift` (1) GREEN — zero drift.

### Axis 4 — Implementer deviations — BENIGN
- `FlagKind` is `Clone, Copy` only (no `Debug`, `mod.rs:141`) → dropping `{:?}` required to compile; struct-update is the standard `field_reassign_with_default` fix.
- Stale-prose `NODE_TYPES` refs updated (`conditional.rs:450`, `h3_minikey_paste_warn.rs:27`); remaining hits are the unrelated `SECRET_NODE_TYPES*` family + historical comments. Rename complete.

### Axis 5 — Scope / ship-readiness — CLEAN
GUI-only (13 files); no toolkit source / `pinned-upstream.toml` / README:50 touched (pin stays v0.60.0). README:42 + Cargo.toml → 0.46.0 (readme_pin_coherence GREEN). `git diff --check` clean. FOLLOWUPS: 5 entries FILED NEW (3 resolved + 2 open deferred), none pre-existed. Full `cargo test --workspace` (MNEMONIC_BIN=v0.60.0) 0 failures; clippy clean.

## Disposition
GREEN. Cleared for the PR + 5-target-CI-green-before-tag ship (PR #14 → v0.46.0). Tick M9/L12/L13 in the bughunt report on toolkit master.
