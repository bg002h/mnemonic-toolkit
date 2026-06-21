# R0 REVIEW — cycle-11a GUI hygiene (M9 · L12 · L13) — PLAN-DOC, Round 1

**Plan:** `design/IMPLEMENTATION_PLAN_cycle11a_gui_hygiene.md`
GUI verified against `origin/master = 0bbe3e1` (v0.45.0, pins toolkit v0.60.0). Toolkit-master = `bea7a607` (= plan §1 SHA). Toolkit pin = `mnemonic-toolkit-v0.60.0`.

## VERDICT: GREEN — 0 Critical / 0 Important

Every load-bearing citation re-grepped against live source resolves; M9 zeroize design, L12 regex transform, and L13 dropdown split are all sound. Two Minor citation-hygiene items (folded into the plan).

### 1. M9 (secret zeroize) — SOUND
Recursive `zeroize_keys` on `TreeNode` covers `key` (`:81`) + `keys[i]` (`:89`) + recurses `children` (`:104`), EXCLUDES `hex` (`:97`, public digest — matches on-disk redactor `tree_model.rs:693-694`). Wired from `zeroize_form_state` (`secrets.rs:294`, close `:326`) via `state.tree.as_mut()` → `tree.root.zeroize_keys()`. `String: Zeroize` available (`Cargo.toml:20` zeroize="1" → Lock 1.8.2; precedent `secrets.rs:300`). No borrow/double-free; no missed secret field. RED-first sound (against `0bbe3e1` the sweep has zero `state.tree` ref).

### 2. L12 (regex + drift fixture) — SOUND
The three regexes (`conditional.rs:107`/`:109`/`:111`) each contain `@\d+(?:/<[0-9;]+>)?`; inserting the second optional bracket between `@\d+` and `(?:/<…>)?` is byte-identical to the existing prefix bracket and keeps the prefix group — prefix/no-origin/multipath stay matching; multisig `:113`/`:115` untouched. The `canonicity_drift.rs:132` count-comment update is arithmetically correct (18→19, 11→12 Canonical, 15→16 classify). Plan flags the pinned-v0.60.0-binary requirement for `canonicity_drift` (stale-$PATH false-fail mode). Benign-over-acceptance carries from spec round-3 GREEN with NO fabricated toolkit claim (v0.60.0 suffix-only `parse_descriptor.rs:69-70`; master double-origin refusal `:113`).

### 3. L13 (dropdown split) — SOUND
`CONVERT_FROM_NODES` (14, seedqr@1) mirrors `NodeType::as_str` (`convert.rs:57-70`); `CONVERT_TO_NODES` (13, seedqr-free) mirrors the `--to PossibleValuesParser` (`convert.rs:209-223`). Current `NODE_TYPES` (`:140-153`) = 13 → rename value-preserving. No schema_mirror/secret_drift (custom `parse_from_input` value_parser → no `--from` enum; `--to` keeps the list). No toolkit pin change; no cargo-fmt.

### 4. Citations — all resolve
Toolkit full-prefix lift verified: `src/cmd/bundle.rs:194`/`:1398`, `src/cmd/gui_schema.rs:1320`, `src/cmd/convert.rs`, and `src/parse_descriptor.rs` (at `src/` NOT `src/cmd/`). GUI citations re-grepped clean. seedqr SHA `5f0b7b45` confirmed.

### 5. SemVer + ship — correct
MINOR 0.45.0 → 0.46.0; sites `Cargo.toml:3` + `README.md:42` (gated by `readme_pin_coherence.rs`); toolkit pin `README.md:50`/`pinned-upstream.toml:22` UNCHANGED. PR + 5-target CI before tag (`FOLLOWUPS.md:716`). Bughunt ticks M9:744/L12:574/L13:584 exact; re-grep-at-ship hedge present.

## Minor (folded into plan — do NOT gate)
- **m-1 (§7 framing):** the three M9/L12/L13 slugs do NOT exist in `mnemonic-gui/FOLLOWUPS.md` — FILE NEW with `Status: resolved` in the ship commit, do not "flip" a non-existent line. (Folded.)
- **m-2 (citation):** plan cited `schema/mnemonic.rs:1116`; correct is `:1119` (spec already correct). (Folded.)

**GREEN — the lane may proceed to per-phase TDD.**
