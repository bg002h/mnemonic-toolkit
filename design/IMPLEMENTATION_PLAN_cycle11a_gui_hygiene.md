# IMPLEMENTATION PLAN — cycle-11a (mnemonic-gui hygiene cluster: M9 / L12 / L13)

**Status:** DESIGN ONLY — no code. This plan-doc feeds its OWN mandatory opus-architect **R0 loop to 0 Critical / 0 Important** (CLAUDE.md hard gate) BEFORE any implementation begins.
**Repo being changed:** `mnemonic-gui` at `/scratch/code/shibboleth/mnemonic-gui`.
**Cycle target SemVer:** GUI MINOR **0.45.0 → 0.46.0** (single PR; PR + 5-target CI green BEFORE tag-push — NOT direct-FF).
**Toolkit pin:** UNCHANGED — `mnemonic-toolkit-v0.60.0` (no sibling-codec change; no toolkit pin bump).
**Author date:** 2026-06-21.
**Upstream R0-GREEN spec:** `design/BRAINSTORM_cycle11a_gui_hygiene.md` (GREEN round 3 — `design/agent-reports/cycle11a-spec-r0-round3-review.md`).
**Round-3 Minor folded:** all lifted TOOLKIT citations use the FULL path prefix (see §1 table; `src/cmd/bundle.rs`, `src/cmd/gui_schema.rs`, `src/parse_descriptor.rs`).

---

## 1. Source-SHA table (every cited line re-grepped against these SHAs at author time)

| Repo | Ref | SHA | Role |
|---|---|---|---|
| `mnemonic-gui` | `origin/master` (v0.45.0) | **`0bbe3e1`** (`0bbe3e1e72618eedc1f95e516bda52b831f37941`) | the repo being changed |
| `mnemonic-toolkit` | `origin/master` (v0.65.0) | **`bea7a607`** (`bea7a6076c4709f6e09c7006aa11242636ee16ea`) | cited only for grammar/parser AUTHORITY (re-grepped this cycle; not changed) |
| `mnemonic-toolkit` | pinned tag | **`mnemonic-toolkit-v0.60.0`** | the binary the GUI shells out to (UNCHANGED this cycle) |

**Re-grep verification performed at author time (all citations below confirmed against the SHAs above):**

GUI (`0bbe3e1`):
- `src/secrets.rs:294` `pub fn zeroize_form_state(...)`; `.zeroize()` precedent at `:300`; `secret_widgets` loop `:323`; fn close `}` at `:326`. **Confirmed.**
- `src/form/tree_model.rs`: `TreeState` `:38`, `TreeState.root: TreeNode` `:48`; `TreeNode` struct `:72`; `key` `:81`, `keys` `:89`, `hex` `:97`, `children` `:104`; `impl TreeNode` block opens `:190`; on-disk model `redacted_for_persistence` `:176`, `blank_non_extended_public_keys` `:695`, hex-untouched comment `:693-694`. **Confirmed.**
- `src/schema/mod.rs:333` `pub tree: Option<crate::form::tree_model::TreeState>`. **Confirmed.**
- `src/form/conditional.rs`: `classify_descriptor_canonicity` `:99`; the THREE single-key regexes `:107` (pkh), `:109` (wpkh), `:111` (tr); multisig regexes (untouched) `:113`/`:115`; `is_descriptor_non_canonical` `:136`; `bundle()` `:190`; pin push `if !is_descriptor_non_canonical(state) { … PinValue }` at `:238`. **Confirmed.**
- `tests/canonicity_drift.rs`: shell-out harness `:47-51`; prefix fixture `pkh([deadbeef/44'/0'/0']@0/<0;1>/*)` `:110`; FIXTURES table close `];` `:131`; count comment `// 11 Canonical + 4 NonCanonical + 3 ParseFails = 18 (15 classify, 3 parse-fail).` at **`:132`**. **Confirmed.**
- `src/schema/mnemonic.rs`: comment block `:133-139`; `const NODE_TYPES: &[&str]` `:140` (13 values, `:141-153`); convert `--from` `name:` `:1114` / `kind: FlagKind::NodeValueComposite(NODE_TYPES)` `:1115`; convert `--to` `name:` `:1124` / `kind: FlagKind::Dropdown(NODE_TYPES)` `:1125`. **Confirmed.**
- `tests/secrets.rs`: `zeroize_form_state_clears_*` zeroize test family; undo-ring caveat asserts `:62-68`. **Confirmed.**
- `tests/non_canonical_descriptor_account_pin.rs`: `state_with_descriptor` `:20`; `canonical_descriptor_pins_account_to_zero` `:27`; `non_canonical_descriptor_lifts_account_pin` `:44`; `tr_with_taptree_non_canonical_lifts_pin` `:62`. **Confirmed.**
- `Cargo.toml:3` `version = "0.45.0"`; `Cargo.toml:20` `zeroize = "1"`. **Confirmed.**
- `README.md:42` GUI self-pin `mnemonic-gui-v0.45.0`; `README.md:50` toolkit pin `mnemonic-toolkit-v0.60.0`; `pinned-upstream.toml:22` `tag = "mnemonic-toolkit-v0.60.0"`. **Confirmed.**
- `FOLLOWUPS.md:716` `### v0.2: enforce PR-CI gate before tag-push`. **Confirmed.**

Toolkit (`bea7a607` = origin/master), lifted with FULL `src/cmd/` / `src/` prefix per the round-3 Minor:
- `src/cmd/bundle.rs:194` `DESCRIPTOR_WITH_NONZERO_ACCOUNT` (byte-exact error string); canonicity probe `canonical_origin(&canonicity_probe.tree).is_none()` `:1398`; refusal-emit site `:1400-1405`. **Confirmed.**
- `src/cmd/gui_schema.rs:1320` `canonical_origin(&desc.tree).is_some()` (the `--classify-descriptor` oracle). **Confirmed.**
- `src/cmd/convert.rs:55-72` `NodeType::as_str` (14 values, `Self::Seedqr => "seedqr"` at `:58`, index 1); `--to` `PossibleValuesParser` `:209-223` (13 values, NO seedqr); `--from` custom `value_parser = parse_from_input` `:193` (→ gui-schema `kind:"text"`, `choices:null`). **Confirmed.**
- `src/parse_descriptor.rs` (NOTE: `src/`, NOT `src/cmd/`): master named-group lexer `Regex::new(` `:97`, regex string with `(?P<pfx_fp>…)…(?P<sfx_fp>…)` `:98`; double-origin refusal `if pfx_fp.is_some() && sfx_fp.is_some()` `:113`, message "…double-origin is ambiguous — supply exactly one" `:116`. **Confirmed.**

Toolkit (pinned tag `mnemonic-toolkit-v0.60.0`):
- `src/parse_descriptor.rs:69` `Regex::new(`, suffix-only regex string `:70` `r"@(\d+)(?:\[([0-9a-fA-F]{8})…\])?…"` (NO prefix-bracket alternative; positional captures only — no `pfx_fp`/`sfx_fp`). **Confirmed.** This is the behavioral basis for the L12 "benign over-acceptance at the pin" rationale (§3.2).

> **Citation-lift discipline reminder for the implementer:** the toolkit-master line numbers above decay every merge. RE-GREP them against live `mnemonic-toolkit` `origin/master` at implementation time, and the GUI bughunt-report checkbox lines (§7) at SHIP time.

---

## 2. Execution model

- **Single implementer subagent** (NOT parallel re-implementations) executes the GREEN plan in a **dedicated GUI worktree off `origin/master`** (`0bbe3e1`). Branch: `feature/cycle11a-gui-hygiene` (or similar) created off `origin/master`.
- **TDD, RED-first.** Every phase writes its failing test(s) first, verifies genuine RED against `0bbe3e1`, then implements to GREEN. RED-proof is recorded per phase.
- **NEVER `cargo fmt` the GUI.** It has NO fmt CI gate; running fmt churns unrelated lines (project standing instruction; `MEMORY.md`). The implementer must not run `cargo fmt` / `cargo fmt --all` at any point. Hand-format new code to match the surrounding style.
- **Per-phase gates (the relevant local gates for this work):**
  - `cargo test` (the touched test files + `cargo test --workspace` before phase close) — the workspace suite is the CI gate (`schema-mirror.yml` runs `cargo test --workspace`).
  - `cargo clippy --all-targets -- -D warnings` (the `build.yml` clippy gate; warnings are denied).
  - These two are the leading local gates; the full 5-target cross-build matrix + schema_mirror gates run in CI on the PR (§6).
- **Per-phase opus review persisted verbatim** to `design/agent-reports/cycle11a-phase-N-<round>-review.md` BEFORE the fold-and-commit step, looped to 0C/0I per phase.
- **Mandatory, non-deferrable whole-diff adversarial review** over the entire cycle diff after the last phase (R0 = plan correctness; this catches implementation-introduced regressions TDD misses). Persisted verbatim. If Agent-API dispatch fails mid-session, FLAG it explicitly and defer the formal review to API recovery — never silently substitute inline self-review.

---

## 3. Phased plan

Three file-disjoint-ish phases (M9 → L12 → L13), sequenced so each lands with its own RED-first tests and per-phase review. Ordering is by blast-radius/independence; phases do not share edited files except `Cargo.toml`/`README.md` (version bump, applied once at ship in §5).

### Phase 1 — M9: recursive `TreeNode` zeroize-walk wired into `zeroize_form_state`

**Goal:** the exit zeroize sweep `zeroize_form_state` (`src/secrets.rs:294-326`) currently never touches `state.tree`; tree key material (`TreeNode.key` / `.keys[i]`, plain `String`, xprv/WIF-typeable) is never scrubbed in RAM. Add a recursive walk that zeroizes `key` + every `keys[i]` and recurses `children`, EXCLUDING `hex` (public digest).

**RED-first tests (write first; verify RED against `0bbe3e1`):**
- **`tests/secrets.rs::zeroize_form_state_clears_tree_keys`** (NEW; mirror the existing `zeroize_form_state_clears_*` family). Build a `FormState` with `tree: Some(TreeState { root: TreeNode { key: "xprv9s21ZrQH…".into(), keys: vec!["xprvA…".into(), "L1aW4…WIF".into()], children: vec![TreeNode { key: "xprv…nested".into(), .. }], .. }, .. })`; call `zeroize_form_state(&mut state)`; assert `root.key.is_empty()`, every `root.keys[i].is_empty()`, AND `children[0].key.is_empty()` (proves the recursion). Use the existing test's `assert!(s.is_empty(), …)` style (`String::zeroize` clears bytes AND length).
  - **RED proof:** against `0bbe3e1` the sweep never references `state.tree`, so `root.key` is still `"xprv9s21ZrQH…"` → `is_empty()` fails. Genuinely RED.
- **`root.hex`-untouched guard** (in the same test or a sibling): plant a `hex` value, assert it is UNCHANGED after the sweep, pinning the deliberate digest-exclusion (defends against a future over-broad walk zeroizing public commitments). Document the intent in the test comment.

**Implementation:**
1. Add an `impl TreeNode` method in `src/form/tree_model.rs` (inside / adjacent to the existing `impl TreeNode` block at `:190`, sibling to the structural model `blank_non_extended_public_keys` at `:695`):
   ```text
   // pseudo — final naming/style by the implementer; mirrors blank_non_extended_public_keys's recursion shape
   pub fn zeroize_keys(&mut self) {
       use zeroize::Zeroize;
       self.key.zeroize();
       for k in &mut self.keys { k.zeroize(); }
       for child in &mut self.children { child.zeroize_keys(); }
   }
   ```
   Rationale for living in `tree_model.rs` (not `secrets.rs`): the recursion is over `TreeNode`'s structure and exactly parallels `blank_non_extended_public_keys` — co-locating keeps the two tree-walks adjacent so a future secret-field addition is caught by both. EXCLUDE `hex` (`:97`) — it is a public digest (`sha256`/`hash256`/`hash160`/`ripemd160`), excluded from on-disk redaction too (`tree_model.rs:693-694`: "Hashlock `hex` FIELDS are untouched"). `w`/`kind`/`n`/`k`/`id` are non-secret structural fields — untouched.
2. Wire it into `zeroize_form_state` (`src/secrets.rs`, after the `secret_widgets` loop, before the fn close at `:326`):
   ```text
   if let Some(tree) = state.tree.as_mut() {
       tree.root.zeroize_keys();
   }
   ```

**Facts already verified (no type change needed):** `String: Zeroize` is live — `Cargo.toml:20` `zeroize = "1"` (resolved 1.8.2 in `Cargo.lock`), and `secrets.rs:300` ALREADY calls `.zeroize()` on a plain `String`. `TreeNode.key`/`.keys[i]` are plain `String` → no `Zeroizing<…>` wrapper or struct change. The `state.tree.as_mut()` → `tree.root.zeroize_keys()` wiring has no borrow conflict (confirmed in spec R0 round 1).

**Out of scope (deferred, documented):** the egui `text_edit_singleline` undo-ring residue (`tree_form.rs` widgets rendering `node.key`/`node.keys[i]`) may retain prior keystrokes after the model `String` is zeroized — the same class as the already-tracked `gui-secret-buffer-allocator-residue` / `gui-os-snapshot-secret-occlusion` caveats (`tests/secrets.rs:62-68`). M9 is satisfied by the model-side scrub (brings `state.tree` to PARITY with the values/slots/positionals/secret_widgets the sweep already covers — identical undo-ring caveat). Filed as FOLLOWUP `gui-tree-key-egui-undo-ring-residue` (§7).

**Phase 1 gate:** `cargo test --test secrets` GREEN (incl. the new test) + `cargo test --workspace` GREEN + `cargo clippy --all-targets -- -D warnings` clean. Persist `cycle11a-phase-1-<round>-review.md`; loop to 0C/0I; fold; commit.

---

### Phase 2 — L12: reposition the canonicity-regex origin bracket to also match the SUFFIX form `@N[fp/path]`

**Goal:** the GUI canonicity regexes (`src/form/conditional.rs:107`/`:109`/`:111`) anchor the optional origin bracket BEFORE `@N` (prefix form `[fp]@N` only). The pinned toolkit ALSO accepts the SUFFIX form `@N[fp/path]` and classifies it canonical (empirically: `gui-schema --classify-descriptor "wpkh(@0[deadbeef/84'/0'/0']/<0;1>/*)"` → `canonical` on the v0.60.0 binary). The GUI misclassifies it NonCanonical → lifts the `--account → PinValue(0)` pin (`conditional.rs:238`) → user `--account N` reaches the toolkit → the toolkit deems the SAME descriptor canonical and hard-errors `DESCRIPTOR_WITH_NONZERO_ACCOUNT` (toolkit `src/cmd/bundle.rs:194`, message byte-exact; canonicity decided structurally at `src/cmd/bundle.rs:1398` / `src/cmd/gui_schema.rs:1320` via `canonical_origin(&tree).is_some()`, NOT textual bracket position). A confusing hard-error on a valid input — NOT a silent wrong-address.

**RED-first tests (write first; verify RED against `0bbe3e1`):**
- **`tests/non_canonical_descriptor_account_pin.rs::suffix_form_origin_descriptor_pins_account_to_zero`** (NEW; mirror `canonical_descriptor_pins_account_to_zero` `:27`): `state_with_descriptor("wpkh(@0[deadbeef/84'/0'/0']/<0;1>/*)")`; assert the `bundle()` visibility vector PINS `--account` to `PinValue(0)` (classified Canonical → pin fires).
  - **RED proof:** against `0bbe3e1` the regex misses the suffix bracket → NonCanonical → pin LIFTED → the "pin present" assert fails. Genuinely RED.
- **Regression guards (positive controls — assert GREEN both before and after; add explicitly if not already covered):**
  - `prefix_form_origin_still_pins`: `wpkh([deadbeef/84'/0'/0']@0/<0;1>/*)` → pinned (prefix path not regressed).
  - `no_origin_form_still_pins`: `wpkh(@0)` → pinned (no-origin path not regressed).
  - Existing `non_canonical_descriptor_lifts_account_pin` (`:44`) + `tr_with_taptree_non_canonical_lifts_pin` (`:62`) MUST stay GREEN (the fix must not flip genuinely-non-canonical descriptors to pinned).
- **`tests/canonicity_drift.rs` fixture add** (the live GUI-vs-toolkit parity harness, `:47-51`): add `("wpkh(@0[deadbeef/84'/0'/0']/<0;1>/*)", Expect::Canonical)` to the `FIXTURES` table (currently only PREFIX `pkh([deadbeef/44'/0'/0']@0/<0;1>/*)` at `:110`).
  - **RED proof:** today the GUI says NonCanonical, toolkit `gui-schema` says canonical → the harness reports a `regressed` mismatch → RED. GREEN after the regex fix. (This is the gate that should have caught L12 originally — the prefix-only corpus let GUI + toolkit happen to agree.)
  - **UPDATE the fixture-count comment** at `tests/canonicity_drift.rs:132`: `// 11 Canonical + 4 NonCanonical + 3 ParseFails = 18 (15 classify, 3 parse-fail).` → `// 12 Canonical + 4 NonCanonical + 3 ParseFails = 19 (16 classify, 3 parse-fail).` (new Canonical suffix fixture: Canonical 11→12, total 18→19, classify-count 15→16).

**Implementation — exact regex change (apply to ALL THREE of `:107`/`:109`/`:111`):** insert a SECOND optional origin-bracket group immediately AFTER `@\d+`, KEEPING the existing prefix group. Per-regex, the substring
```
@\d+(?:/<[0-9;]+>)?
```
becomes
```
@\d+(?:\[[0-9a-fA-F]{8}(?:/\d+'?h?)*\])?(?:/<[0-9;]+>)?
```
i.e. the new suffix bracket sits between `@\d+` and the optional `/<…>` multipath, exactly where the toolkit's suffix grammar places it (`@N[fp/path]/<mpath>/*`). The new sub-pattern is byte-identical to the existing prefix bracket; the leading prefix group is retained (the toolkit still accepts prefix form; the prefix drift fixture `:110` must keep passing).

The wsh/sh-wsh multisig regexes (`:113`/`:115`) carry no origin bracket and are UNTOUCHED.

**Why this is correct / benign over-acceptance (verified two ways; do NOT re-derive — folded GREEN at spec round 3):**
- Empirically (v0.60.0 binary): the suffix form classifies `canonical`; the GUI regex fix makes the GUI agree. Prefix / no-origin / multipath cases unchanged (proven via Python `re` mirroring Rust anchored semantics + the live binary).
- The double-origin form `wpkh([fp]@0[fp]/<0;1>/*)` now also matches the GUI regex (Canonical). This is BENIGN under BOTH toolkit versions: **v0.60.0 (the pin) ACCEPTS** it — its suffix-only lexer (`src/parse_descriptor.rs:69-70`) matches only the suffix bracket; a leading `[fp]` is silently skipped by `captures_iter` → `canonical`, exit 0 → GUI→Canonical→`--account 0` pin is compatible (0 is the descriptor-mode default; the guard at `src/cmd/bundle.rs:1398-1405` fires only on `account != 0`). **master / v0.62.0+ REFUSES** it at parse (named-group lexer `src/parse_descriptor.rs:98`, refusal `:113-116` "supply exactly one", H7 `36095b88`) → the pin value is moot. Never silently-wrong-address either way. The GUI canonicity regex is, by design, a COARSE pin-gate, not a full validator (the toolkit is the authority); classifying a string the pinned toolkit accepts as Canonical AGREES with the toolkit's verdict → NOT a regression. Tightening to "exactly one of prefix/suffix" adds complexity for zero behavioral gain and v0.62.0+ already enforces it toolkit-side → explicitly out of scope (D7).

**Phase 2 gate:** `cargo test --test non_canonical_descriptor_account_pin` GREEN + `cargo test --test canonicity_drift` GREEN (requires `MNEMONIC_BIN` = the pinned v0.60.0 binary on `$PATH`/env per the harness) + `cargo test --workspace` GREEN + `cargo clippy --all-targets -- -D warnings` clean. Persist `cycle11a-phase-2-<round>-review.md`; loop to 0C/0I; fold; commit.

> **canonicity_drift harness note:** it shells out to `MNEMONIC_BIN gui-schema --classify-descriptor`. Run it with the PINNED `mnemonic-toolkit-v0.60.0` binary (NOT a stale/newer `mnemonic` on `$PATH`) — a stale-PATH binary is the known false-fail mode (cf. the v0.60.0 schema_mirror gotcha). CI installs the pinned binary; locally the implementer must set `MNEMONIC_BIN` to a v0.60.0 build.

---

### Phase 3 — L13: split `NODE_TYPES` into a `--from` set (+seedqr) and a seedqr-free `--to` set

**Goal:** `convert --from seedqr=<digits>` is unreachable from the GUI: `NODE_TYPES` (`src/schema/mnemonic.rs:140`, 13 values) is shared by BOTH `--from` (`:1115`, `NodeValueComposite`) and `--to` (`:1125`, `Dropdown`) and omits `seedqr`. The toolkit accepts `--from seedqr` (`NodeType::as_str` has `seedqr` at index 1, `src/cmd/convert.rs:58`) but REJECTS `--to seedqr` (its `--to` `PossibleValuesParser` `src/cmd/convert.rs:209-223` deliberately omits it — seedqr is decode/input-only). So `--from` needs seedqr; `--to` must NOT offer it.

**RED-first tests (write first; verify RED against `0bbe3e1`):**
- **`convert_from_dropdown_includes_seedqr`** (NEW): assert the `--from` flag's value list (`CONVERT_FROM_NODES`) CONTAINS `"seedqr"`, at index 1 (after `phrase`).
  - **RED proof:** against `0bbe3e1`, `--from` shares `NODE_TYPES` which lacks `seedqr` → fails. Genuinely RED.
- **`convert_to_dropdown_excludes_seedqr`** (NEW): assert the `--to` flag's value list (`CONVERT_TO_NODES`) does NOT contain `"seedqr"` (mirrors the toolkit `--to` `PossibleValuesParser` refusal). This is the guard that the split didn't LEAK seedqr into `--to`. GREEN today (shared list lacks it) and stays GREEN after the split.
  - Place these where the schema's flag/value lists are introspectable (extend `tests/secret_taxonomy_pin.rs` or a small new `tests/convert_node_dropdowns.rs` — final home is an impl detail, not a design decision).

**Implementation:**
1. Introduce a NEW const **`CONVERT_FROM_NODES`** = the 14-value list with `seedqr` inserted at index 1 (after `phrase`), mirroring `NodeType::as_str` ordering (`src/cmd/convert.rs:57-58`):
   `phrase, seedqr, entropy, xpub, xprv, wif, fingerprint, path, ms1, mk1, bip38, minikey, electrum-phrase, address`.
2. **Rename** the existing 13-value seedqr-free `NODE_TYPES` → **`CONVERT_TO_NODES`** (paired, self-documenting names; both consumed once each, so rename blast-radius is the two consumer sites + any test referencing the name — re-grep `NODE_TYPES` across the crate before renaming).
3. Rewire:
   - `--from` (`:1115`): `FlagKind::NodeValueComposite(CONVERT_FROM_NODES)`.
   - `--to` (`:1125`): `FlagKind::Dropdown(CONVERT_TO_NODES)`.
4. Replace the misleading `:133-139` comment (it claims `NODE_TYPES` "exactly mirrors upstream `NodeType::as_str()`" — it actually matches the `--to` runtime restriction) with the split rationale: `CONVERT_FROM_NODES` mirrors `NodeType::as_str` (the INPUT enum, includes `seedqr`); `CONVERT_TO_NODES` mirrors the toolkit's `--to` `PossibleValuesParser` (EXCLUDES `seedqr` — decode-only). Cite toolkit `src/cmd/convert.rs:54-72` (as_str) and `:209-223` (--to parser).

**Post-fix:** `--from`'s composite offers 14 nodes incl. `seedqr`; `--to`'s dropdown offers the same 13 as today (no `seedqr`). A user can now select `convert --from seedqr=<digits>` (previously unreachable); `--to seedqr` remains un-offerable (correct — toolkit refuses it).

**NO toolkit pin bump / NO schema_mirror trigger (verified; do NOT re-litigate — GREEN at spec round 3):** `seedqr` was added to the toolkit in v0.31.6 (`5f0b7b45`) — present at the pinned v0.60.0. `tests/schema_mirror.rs` compares flag NAMES only; `tests/schema_mirror_secret_drift.rs` compares per-(sub,flag) secret-bits only; NEITHER compares dropdown VALUES. The toolkit's `gui-schema` emits NO `--from` value enum (custom `parse_from_input` value_parser → `kind:"text"`, `choices:null`), and `--to` keeps the seedqr-free list (still matches gui-schema's `--to` enum). Zero schema_mirror surface touched. `--from`'s `secret:` bit is unchanged (node-dependent composite, `schema/mnemonic.rs:1119`).

**Phase 3 gate:** the new convert-dropdown test GREEN + `cargo test --workspace` GREEN (incl. `schema_mirror` + `schema_mirror_secret_drift` STILL passing — proves no drift) + `cargo clippy --all-targets -- -D warnings` clean. Persist `cycle11a-phase-3-<round>-review.md`; loop to 0C/0I; fold; commit.

---

## 4. Mandatory whole-diff post-implementation review

After Phase 3, dispatch a **single independent adversarial whole-diff review** over the entire cycle-11a diff (M9 + L12 + L13 combined). This is **mandatory and non-deferrable** (R0 = plan correctness; the whole-diff review catches implementation-introduced regressions TDD misses — e.g. an over-broad zeroize accidentally clearing `hex`, a regex change that silently re-numbers a capture group, a rename that missed a `NODE_TYPES` consumer). Persist verbatim to `design/agent-reports/cycle11a-whole-diff-<round>-review.md`; loop to 0C/0I. If Agent-API dispatch fails, FLAG explicitly and defer to API recovery — never inline-substitute.

---

## 5. Version sites + SemVer (applied once, at ship)

GUI MINOR **0.45.0 → 0.46.0** (L13 adds a user-reachable dropdown VALUE `--from seedqr` = additive user-facing surface → MINOR; M9 security fix + L12 bug fix ride along; highest-precedence change sets MINOR).

| Site | Current | Change | Gate |
|---|---|---|---|
| `Cargo.toml:3` | `version = "0.45.0"` | → `"0.46.0"` | — |
| `README.md:42` | GUI self-pin `--tag mnemonic-gui-v0.45.0` | → `mnemonic-gui-v0.46.0` | `tests/readme_pin_coherence.rs` (self-line `<TAG>` must equal `mnemonic-gui-v{Cargo.toml version}`) |
| `README.md:50` | toolkit pin `--tag mnemonic-toolkit-v0.60.0` | **UNCHANGED** | `readme_pin_coherence` / `pin_coherence` assert it matches `pinned-upstream.toml:22` (NOT touched) |
| `pinned-upstream.toml:22` | `tag = "mnemonic-toolkit-v0.60.0"` | **UNCHANGED** | — |

No other version-markered site (GUI has no second README / no own fuzz lockfile for this cycle). `Cargo.lock` updates only if the version bump dirties it — commit the resulting lockfile bump if so.

---

## 6. Ship mechanism — PR + 5-target CI green BEFORE tag

GUI is **PR-CI-gated, NOT direct-FF** (`FOLLOWUPS.md:716` `### v0.2: enforce PR-CI gate before tag-push`).

1. Push `feature/cycle11a-gui-hygiene` to `origin`; open ONE PR to `master` (M9 + L12 + L13 + version sites + FOLLOWUP flips in the single PR).
2. Wait for the **full CI matrix GREEN before any tag-push**:
   - `build.yml`: `clippy` job (`cargo clippy --all-targets -- -D warnings`) + the **5-target cross-build matrix** (`x86_64-linux`, `aarch64-linux`, `x86_64-windows`, `x86_64-macos`, `aarch64-macos`).
   - `schema-mirror.yml`: installs the 4 pinned CLIs (`mnemonic`@v0.60.0, `md`, `ms`, `mk`) → `cargo test --test schema_mirror` + `cargo test --workspace` (the full suite, incl. `canonicity_drift`, `secrets`, `secret_taxonomy_pin`, `readme_pin_coherence`).
3. Only AFTER CI is fully green: merge the PR, then tag `mnemonic-gui-v0.46.0` on the merge commit and push the tag (the `build.yml` `release` job attaches artifacts on tag).
4. **NEVER `cargo fmt`** at any point (no fmt CI gate; would churn unrelated lines).

---

## 7. FOLLOWUP flips + bughunt report ticks (at ship)

**FOLLOWUP entries** — `mnemonic-gui/FOLLOWUPS.md` (repo root, NOT `design/`). **plan-R0 m-1: the three M9/L12/L13 slugs below do NOT currently exist in `mnemonic-gui/FOLLOWUPS.md`** (verified against `origin/master` — the findings live only as checkboxes in the toolkit-side bughunt report). Per the repo convention (cf. the H2/H3 entries at `FOLLOWUPS.md:729`/`:736`), **FILE a NEW `### <slug>` entry with `Status: resolved`** referencing the bughunt report, IN the shipping commit — do NOT attempt to flip a non-existent line. (Standing [[feedback_followup_status_discipline]]: verify status at decision time — here it is "not-yet-filed".)

| Slug | Action at ship |
|---|---|
| `gui-tree-key-not-zeroized-on-exit` | **FILE NEW, Status: resolved** (M9) — in-RAM model-side scrub shipped. |
| `gui-canonicity-regex-suffix-origin-misclassify` | **FILE NEW, Status: resolved** (L12) — regex reposition + suffix-form drift fixture. |
| `gui-convert-from-dropdown-missing-seedqr` | **FILE NEW, Status: resolved** (L13) — `CONVERT_FROM_NODES` split. |
| `gui-tree-key-egui-undo-ring-residue` | **FILE NEW / OPEN** — second-tier residue: egui `text_edit_singleline` undo ring may retain tree-key keystrokes after the model `String` is zeroized (same class as `gui-secret-buffer-allocator-residue` / `gui-os-snapshot-secret-occlusion`). Deferred. |
| `gui-canonicity-regex-coarse-pin-gate-not-validator` | **FILE NEW / OPEN (doc-only, optional)** — record that `classify_descriptor_canonicity` is intentionally a coarse pin-gate (over-accepts the double-origin `[fp]@N[fp]` form, benign under v0.60.0 ACCEPT + v0.62.0+ REFUSE-at-parse); the single-oracle alternative (`gui-schema --classify-descriptor`) is a larger change, deferred. |

> Per [[feedback_followup_status_discipline]]: verify each slug's "open" status at decision time (tracking lags code) and flip IN the shipping commit.

**Bughunt report ticks** — `design/agent-reports/constellation-bughunt-2026-06-20.md` on **toolkit master** (`wt-tk-master`): tick the `### - [ ]` → `### - [x]` checkboxes for **M9** (author-time line `:744`), **L12** (`:574`), **L13** (`:584`) with the fixing commit/PR reference, per the report's "fix checklist" contract. **RE-GREP these live line numbers at SHIP time** — they move on every merge to toolkit master.

---

## 8. SemVer / lockstep summary

| Item | Decision | Why |
|---|---|---|
| SemVer | GUI MINOR **0.45.0 → 0.46.0** | L13 reachable dropdown value = additive surface; M9/L12 ride along. |
| Ship mechanism | PR + 5-target CI green BEFORE tag-push | GUI PR-CI gate (`FOLLOWUPS.md:716`). |
| Toolkit pin bump | NONE (stays v0.60.0) | seedqr present since v0.31.6 < v0.60.0; M9/L12/L13 all GUI-local. |
| `schema_mirror` lockstep | NONE | flag-NAME gate; `--from` emits no value enum (custom parser); `--to` keeps seedqr-free list. |
| `schema_mirror_secret_drift` | NONE | per-(sub,flag) secret-bit gate; no secret-bit changes. |
| Sibling-codec FOLLOWUP companions | NONE | GUI-only cycle. |
| `cargo fmt` | NEVER | GUI has no fmt CI gate. |
| Manual mirror (`docs/manual/`) | NONE | no toolkit/sibling-CLI flag change; GUI not in the manual mirror scope. |

---

## 9. Mandatory R0-gate note

Per CLAUDE.md: **NO code before THIS plan-doc passes its own opus-architect R0 review to 0 Critical / 0 Important.** R0 is mandatory, never skipped/deferred. Reviewer-loop mechanics: dispatch architect → fold findings → **persist the review verbatim to `design/agent-reports/`** → re-dispatch → repeat until GREEN (the loop continues after EVERY fold — a fold can introduce drift). Only after this plan-doc is GREEN does the single-implementer-per-phase TDD begin (§2-§3), each phase with its per-phase R0 review persisted verbatim, then the mandatory non-deferrable whole-diff post-impl adversarial review (§4). This plan-doc inherits the R0-GREEN spec (`BRAINSTORM_cycle11a_gui_hygiene.md`, round 3) and folds its one non-blocking Minor (full `src/cmd/` / `src/` toolkit-citation prefixes — §1).
