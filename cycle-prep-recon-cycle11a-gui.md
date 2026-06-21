# cycle-prep recon — 2026-06-21 — cycle-11a (mnemonic-gui cluster: M9, L12, L13)

**Repo:** `mnemonic-gui` at `/scratch/code/shibboleth/mnemonic-gui`
**Origin/master SHA at recon time:** `0bbe3e1` (`0bbe3e1e72618eedc1f95e516bda52b831f37941`) — "release(v0.45.0): cycle-3 ship"
**Local branch:** `master`
**Sync state:** up-to-date (0 ahead / 0 behind — working tree == `origin/master`; all citations verified against checked-out bytes, which equal `origin/master`)
**Untracked:** none
**GUI version:** `0.45.0` · **Toolkit pin:** `mnemonic-toolkit-v0.60.0` (`Cargo.toml:42`)
**Source report:** `design/agent-reports/constellation-bughunt-2026-06-20.md` (M9 @ :720, L12 @ :553, L13 @ :563)

Slug(s) verified: `M9 (w3-gui-tree-key-not-zeroized-on-exit)`, `L12 (w2-gui-cond-02)`, `L13 (w2-gui-cond-01)`.
**All three REPRODUCE.** Only M9 has line-drift; L12/L13 citations are content-accurate.

> NOTE on process: `mnemonic-gui` is NOT a `FOLLOWUPS.md`-slug repo for these items — M9/L12/L13 originate in the **bug-hunt report's fix-checklist** (each a `- [ ]` checkbox), not the FOLLOWUPS registry. Verification therefore ran against the bug-hunt citations directly. The GUI `FOLLOWUPS.md` carries the PR-CI-gate process note (`§v0.2: enforce PR-CI gate before tag-push`, FOLLOWUPS.md:716).

---

## Per-slug verification

### M9 — GUI exit zeroize sweep skips `state.tree` (SECRET-hygiene, D-secret-leak)
- **WHAT (from bug-hunt report):** `zeroize_form_state` (the exit-time best-effort scrub) iterates `values`/`slots`/`positionals`/`secret_widgets` but NEVER `state.tree`. Descriptor-builder key material typed into the node-tree (`TreeNode.key` / `.keys`, plain `String` rendered via `text_edit_singleline` — a user can type an xprv/WIF/hex private key) is never zeroized in memory. On-disk persistence IS defended (fail-closed allowlist), so this is strictly the **in-memory non-scrub**.
- **Citations:**
  - `src/secrets.rs:278-310` (`zeroize_form_state`) — **DRIFTED-by-+16** → now **lines 294-326**. Confirmed body iterates `state.values` (`:295`), `state.slots.rows` (`:308`), `state.positionals` (`:311`), `state.secret_widgets` (`:323`); **no `state.tree` reference anywhere in the function.** ACCURATE in substance.
  - `src/schema/mod.rs:324-333` (`FormState.tree`) — **ACCURATE.** `pub tree: Option<crate::form::tree_model::TreeState>` declared at `:333` (`#[serde(default)]`).
  - `src/form/tree_model.rs:81,89` (`TreeNode.key/.keys` plain String) — **ACCURATE.** `pub key: String` @ `:81`; `pub keys: Vec<String>` @ `:89`; both plain `String` (NOT `Zeroizing`). TreeNode is **recursive**: `pub children: Vec<TreeNode>` @ `:104` → fix needs a recursive walk.
  - Caller — `src/main.rs:1145` (`on_exit()`): `zeroize_form_state` IS the exit-time scrub (called after persistence save; "SPEC §9 best-effort zeroize sweep on close"). Tree keys rendered for user edit at `src/form/tree_form.rs:697` (`node.key`) and `:717` (`node.keys[i]`) via `text_edit_singleline`.
  - On-disk protection confirmed PRESENT (so this is in-memory only): `src/persistence.rs:136-145` → `TreeState::redacted_for_persistence` (`tree_model.rs:176-187`) → `blank_non_extended_public_keys` (`tree_model.rs:695-707`) recursively blanks every non-extended-public `key`/`keys`. M9 is NOT an on-disk leak.
- **STILL-REPRODUCES: YES.** `zeroize_form_state` currently skips `state.tree`.
- **Coherent fix?** YES — **one coherent zeroize-sweep extension.** Add a recursive `state.tree.as_mut()` walk that `.zeroize()`s each node's `key` and each `keys[i]` (fields are already plain `String` with a `Zeroize` impl → no type changes required; the gap is purely that the loop predates the v0.32.0 tree builder). Optionally upgrade tree-key storage to a zeroizing buffer (a larger, separable change — NOT required for the fix). Add a regression test asserting a typed tree key is scrubbed on the exit sweep.
- **Fix-site:** `src/secrets.rs` `zeroize_form_state` (currently `:294-326`); recursion helper alongside `TreeNode` in `src/form/tree_model.rs`.
- **Action for brainstorm spec:** Cite the **current** lines (`secrets.rs:294-326`, not 278-310) against SHA `0bbe3e1`. Note TreeNode recursion (`children` @ `:104`) and that fields are already `Zeroize`-capable plain `String`.

### L12 — GUI canonicity regex misclassifies suffix-form `@N[fp/path]` → lifts the `--account` pin (AVAIL / confusing-error)
- **WHAT (from bug-hunt report):** `classify_descriptor_canonicity` writes the origin bracket **before** `@N` (`[fp]@N`), but the toolkit grammar writes it **after** (`@N[fp/path]`, suffix form). A toolkit-canonical suffix-form descriptor fails the GUI regex → classifies **NonCanonical** → the GUI **lifts the `--account → PinValue(0)` pin** → user-typed `--account N` flows to the toolkit, which classifies the SAME descriptor **Canonical** and hard-errors `DESCRIPTOR_WITH_NONZERO_ACCOUNT`. Confusing hard-error on a valid input (NOT silent wrong-address).
- **Citations:**
  - `src/form/conditional.rs:99-126` (`classify_descriptor_canonicity`) — **ACCURATE.** The pkh/wpkh/tr regexes (`:107`,`:109`,`:111`) use `^…\((?:\[[0-9a-fA-F]{8}(?:/\d+'?h?)*\])?@\d+…` — the optional origin bracket is anchored **BEFORE** `@\d+` (PREFIX form `[fp]@N`). The toolkit grammar (`parse_descriptor.rs:70`, current master) is `@(\d+)(?:\[([0-9a-fA-F]{8})…\])?…` — bracket **AFTER** `@N` (SUFFIX form). So `pkh(@0[deadbeef/44'/0'/0']/<0;1>/*)` does NOT match → NonCanonical.
  - `src/form/conditional.rs:136-141` (`is_descriptor_non_canonical`) — **ACCURATE.** Predicate returns true iff `--descriptor` present AND classifies NonCanonical.
  - `src/form/conditional.rs:238-245` (pin-lift in `bundle()`) — **ACCURATE.** `if !is_descriptor_non_canonical(state) { vis.push(("--account", PinValue{value: json!(0)})) }` — the pin fires (= account forced 0) ONLY when Canonical; lifted when NonCanonical → user `--account N` passes through.
- **STILL-REPRODUCES: YES.** A toolkit-standard suffix-form `@N[fp/path]` descriptor is misclassified NonCanonical, lifting the pin → toolkit hard-error `DESCRIPTOR_WITH_NONZERO_ACCOUNT` when the user supplies a non-zero account on a descriptor the toolkit deems canonical.
- **Why the drift gate misses it:** the `canonicity_drift.rs` fixtures (and the GUI regex itself) all use the PREFIX form, so GUI and toolkit happen to agree on those inputs; no suffix-form fixture exists. (Recommend extending the corpus with a suffix-form case as part of the fix.)
- **Fix-site:** `src/form/conditional.rs` regexes at `:107`/`:109`/`:111` — move the origin-bracket group to AFTER `@\d+`. (Alternative noted in the report: call `gui-schema --classify-descriptor` — the toolkit's own classifier — but that's a larger architectural shift; the minimal fix is the regex reposition.)
- **Action for brainstorm spec:** Cite `conditional.rs:99-126`/`136-141`/`238-245` against SHA `0bbe3e1`; cite the toolkit grammar at `parse_descriptor.rs:70` (current toolkit master) as the canonical suffix-form authority. Add a suffix-form fixture to `canonicity_drift.rs`.

### L13 — GUI `convert --from` dropdown missing the valid `seedqr` node type (COVERAGE)
- **WHAT (from bug-hunt report):** `NODE_TYPES` (shared by `--from` and `--to`) omits `seedqr`, so `convert --from seedqr=<digits>` is unreachable from the GUI although the toolkit CLI supports it on `--from`. The schema-mirror gate checks flag NAMES not dropdown VALUES, so it never caught the drift.
- **Citations:**
  - `src/schema/mnemonic.rs:130-144` (`NODE_TYPES`) — **DRIFTED (array now at `:140-154`; comment at `:133-139`).** Confirmed 13 values `phrase, entropy, xpub, xprv, wif, fingerprint, path, ms1, mk1, bip38, minikey, electrum-phrase, address` — **`seedqr` ABSENT.** ACCURATE in substance.
  - "false 'exact mirror' comment" `:123-129` — **DRIFTED-by-design (now `:133-139`).** The comment claims NODE_TYPES "exactly mirrors `NodeType::as_str()`" — but the list actually matches the toolkit's `--to` **runtime restriction** (`PossibleValuesParser`, no seedqr), not the `NodeType` enum (which HAS seedqr at index 1). The mismatch is architectural, not a typo.
  - Authoritative `cmd/convert.rs:54-72` (toolkit) — **ACCURATE (current toolkit master `:54-72`).** `NodeType::as_str` has `Self::Seedqr => "seedqr"` @ `:58` (index 1, right after `phrase`). At the **pinned v0.60.0 tag**, `git show mnemonic-toolkit-v0.60.0:…/convert.rs` confirms `Self::Seedqr => "seedqr"` @ `:58` — seedqr is present at the pin.
  - GUI usage: `src/schema/mnemonic.rs:1115` `--from` = `FlagKind::NodeValueComposite(NODE_TYPES)`; `:1125` `--to` = `FlagKind::Dropdown(NODE_TYPES)` — **same list for both.**
- **STILL-REPRODUCES: YES.** `seedqr` is missing from the GUI's `--from` composite, so the toolkit-supported `--from seedqr=<digits>` is unreachable from the GUI.
- **Toolkit pin bump?** **NO.** `seedqr` (NodeType::Seedqr) was added in toolkit **v0.31.6** (commit `19c1a16d`) — long before the pinned **v0.60.0**. It is present at the pin. The fix is a **GUI-local dropdown add**; no pin bump (no v0.65.0) needed.
- **schema_mirror lockstep?** **NO — does NOT trip `schema_mirror`** (verified two ways):
  1. `tests/schema_mirror.rs` extracts and compares **flag NAMES only** (`schema_flag_names` @ `:52-53` maps `f.name`; assertion `:91-112`). It does NOT compare dropdown VALUES. `tests/schema_mirror_secret_drift.rs` gates only `(subcommand, flag)` **secret-bit** drift, not value enums.
  2. The toolkit's `gui-schema` emits a dropdown enum ONLY for `--to` (toolkit `--to` uses `PossibleValuesParser::new([... 13 values, NO seedqr])` @ convert.rs `:209-223`); `--from` uses a **custom** `value_parser = parse_from_input` → gui-schema classifies it `kind: "text"` (per gui-schema's own doc-rule: custom value_parsers collapse to `text`) → **no `--from` value enum is emitted at all.** So schema_mirror has no `--from` value list to compare against, and the GUI's `--to` list already matches gui-schema's `--to` enum (both lack seedqr).
- **CRITICAL fix-design guardrail (funds/UX-safety):** the fix MUST **split the list** — add `seedqr` to a `--from`-only set, and KEEP the `--to` dropdown on a seedqr-free set. The toolkit **rejects `--to seedqr`** (PossibleValuesParser excludes it; seedqr is decode/input-only). If the fix naively adds seedqr to the SHARED `NODE_TYPES`, the GUI's `--to` dropdown would (a) offer an invalid value the toolkit refuses, and (b) **diverge from gui-schema's `--to` enum** — which, while schema_mirror won't catch it, is exactly the latent drift that should be avoided. Recommended: introduce `CONVERT_FROM_NODES` (= 13 + `seedqr` at index 1) and `CONVERT_TO_NODES` (= current `NODE_TYPES`), wiring `:1115`→FROM, `:1125`→TO. (The report's secondary suggestion to "extend the conditional-drift test to dropdown values" is a good optional hardening but is NOT a lockstep blocker.)
- **Fix-site:** `src/schema/mnemonic.rs` — new `CONVERT_FROM_NODES` const (insert `seedqr` at index 1 to mirror `NodeType::as_str` ordering); rewire `--from` (`:1115`); update the misleading `:133-139` comment to state the `--from` vs `--to` split rationale.
- **Action for brainstorm spec:** Cite NODE_TYPES at current `:140-154` (not 130-144) and toolkit `convert.rs:54-72` + `:209-223` against SHA `0bbe3e1` (GUI) / current toolkit master. Record that seedqr is present at the v0.60.0 pin (added v0.31.6) → no pin bump. Record the split-list design + the `--to seedqr` refusal.

---

## Cross-cutting observations
1. **All three REPRODUCE; only M9 drifted by line number** (+16, 278-310 → 294-326). L12 and L13 are content-accurate; L13's two GUI line ranges drifted by ~10 (`:130-144`→`:140-154`, comment `:123-129`→`:133-139`) but the content claim holds.
2. **schema_mirror is a flag-NAME gate, not a dropdown-VALUE gate — confirmed against current source.** `schema_mirror.rs:52-53/91-112` compares `f.name` only; `schema_mirror_secret_drift.rs` compares the secret-bit per `(sub,flag)`. Neither compares value enums. The CLAUDE.md phrase "gate dropdown value enums" overstates the current gate for `convert` `--from`: because the toolkit's `--from` uses a custom parser, gui-schema emits no `--from` enum, so no value-level mirror exists to gate. **L13 is therefore NOT a schema-mirror lockstep** — it is a pure GUI-local additive change. (This nuance is the single most consequential cross-cutting finding for scoping.)
3. **L13 fix has a latent footgun:** adding seedqr to the *shared* list silently breaks `--to` (toolkit rejects `--to seedqr` + diverges from gui-schema's `--to` enum). The split-list design is mandatory, not optional.
4. **PR-CI gate (process invariant for this repo):** GUI feature work lands via **PR with full 5-target CI green before tag** (NOT direct-FF) — `FOLLOWUPS.md:716` (`v0.2: enforce PR-CI gate before tag-push`). The cycle-11a fixes MUST ship as a PR, not a direct push to master.
5. **No `cargo fmt` CI gate in the GUI — NEVER `cargo fmt` it** (project standing instruction; the GUI has no fmt gate and formatting it can churn unrelated lines).
6. **Version-site ritual (GUI):** bump `Cargo.toml:3` (`0.45.0`) AND `README.md:42` install pin in lockstep — both gated by `tests/readme_pin_coherence.rs` / `tests/pin_coherence.rs`. No silent README drift.
7. **M9 vs the tracked allocator-residue caveat:** M9 is distinct — here `zeroize` is never CALLED on the tree at all (not a residue-after-zeroize issue). On-disk is already defended (fail-closed allowlist), so the cycle adds in-memory scrub only.

---

## Recommended brainstorm-session scope
- **One cycle (cycle-11a), single GUI PR.** All three are GUI-local, no sibling-codec or toolkit changes, no toolkit pin bump.
- **SemVer: GUI MINOR `0.45.0 → 0.46.0`.** L13 adds a user-facing dropdown VALUE (new reachable conversion source) — additive surface → MINOR per the project convention (additive value on an existing subcommand; L13's new `--from seedqr` reachability is a new user-facing capability). M9 (security fix) + L12 (bug fix) ride along; the highest-precedence change (L13 additive feature) sets MINOR. (If the user prefers, M9+L12 alone would be PATCH-class, but bundling L13's additive dropdown makes the cycle MINOR.)
- **Rough sizing:** M9 ≈ 25-40 LoC (recursive zeroize helper + sweep call + 1 test). L12 ≈ 3 regex edits + 1 suffix-form fixture + test (~15-25 LoC). L13 ≈ new `CONVERT_FROM_NODES` const + 1 rewire + comment fix + optional value-drift test (~20-30 LoC). Total ~60-95 LoC + tests. Small cycle.
- **Locksteps / gates:**
  - **NO `schema_mirror` lockstep** for L13 (flag-name gate; `--from` has no emitted value enum) — confirmed.
  - **NO toolkit pin bump** (seedqr present since v0.31.6 < v0.60.0).
  - **NO sibling-codec FOLLOWUP companions** (GUI-only).
  - **PR-CI gate REQUIRED** (5-target green before tag) — `FOLLOWUPS.md:716`.
  - **Version-site lockstep:** `Cargo.toml` + `README.md` pin (gated by pin_coherence tests).
  - **Do NOT `cargo fmt`.**
- **Inter-slug dependencies:** none — M9/L12/L13 touch disjoint files (`secrets.rs`+`tree_model.rs` / `conditional.rs` / `schema/mnemonic.rs`). Implement in any order under TDD.
- **R0 gate reminder:** this recon FEEDS the mandatory R0 gate — it does not replace it. The brainstorm spec + impl plan-doc MUST pass an opus architect R0 review to 0C/0I BEFORE any code (per CLAUDE.md). Single-subagent-per-phase TDD; persist per-phase reviews verbatim to `design/agent-reports/`; whole-diff post-impl review.
