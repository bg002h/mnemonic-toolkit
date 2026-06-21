# BRAINSTORM — cycle-11a (mnemonic-gui hygiene cluster: M9 / L12 / L13)

**Status:** DESIGN ONLY — no code. Feeds the mandatory opus-architect **R0 loop to 0 Critical / 0 Important** (CLAUDE.md hard gate; no implementation until GREEN).
**Repo:** `mnemonic-gui` at `/scratch/code/shibboleth/mnemonic-gui`.
**Cycle target SemVer:** GUI MINOR **0.45.0 → 0.46.0** (single PR, PR-CI-gate, NOT direct-FF).
**Toolkit pin:** UNCHANGED — `mnemonic-toolkit-v0.60.0`.
**Author date:** 2026-06-21.
**Inputs folded:** `cycle-prep-recon-cycle11a-gui.md` (P0 STRICT-GATE recon, all three REPRODUCE); `design/agent-reports/constellation-bughunt-2026-06-20.md` (M9 @ :720-740 fix-checklist, L12 @ :553, L13 @ :563).

---

## 1. Source-SHA table (every cited line re-grepped against these SHAs)

| Repo | Ref | SHA | Role |
|---|---|---|---|
| `mnemonic-gui` | `origin/master` (v0.45.0) | **`0bbe3e1`** (`0bbe3e1e72618eedc1f95e516bda52b831f37941`) | the repo being changed |
| `mnemonic-toolkit` | pinned tag | **`mnemonic-toolkit-v0.60.0`** | the binary the GUI shells out to (UNCHANGED this cycle) |
| `mnemonic-toolkit` | `origin/master` | (cited only for grammar/parser authority) | suffix-form grammar + `--from`/`--to` parsers |
| `descriptor-mnemonic` (md-codec) | working tree | `canonical_origin.rs` | the toolkit's authoritative canonicity oracle (cited, not changed) |

All GUI line numbers below were re-grepped against `0bbe3e1` at author time (the recon's M9 +16 drift and L13 ~+10 drift are already reconciled to current lines here).

---

## 2. Finding summary — all three REPRODUCE

| ID | Class | One-line | Reproduces @ `0bbe3e1` | Empirical proof |
|---|---|---|---|---|
| **M9** | D-secret-leak (in-memory) | exit zeroize sweep `zeroize_form_state` never walks `state.tree`; tree key material (`TreeNode.key`/`.keys`, plain `String`, xprv/WIF-typeable) is never scrubbed in RAM | YES — `secrets.rs:294-326` has no `state.tree` reference | static: the sweep body iterates values/slots/positionals/secret_widgets only |
| **L12** | A-wrong-address (confusing hard-error) | GUI canonicity regex anchors the origin bracket BEFORE `@N` (prefix `[fp]@N` only); the toolkit ALSO accepts the SUFFIX form `@N[fp/path]` and classifies it **canonical** → GUI misclassifies NonCanonical → lifts the `--account` pin → toolkit hard-errors `DESCRIPTOR_WITH_NONZERO_ACCOUNT` | YES — `conditional.rs:107/109/111` | **empirical** (v0.60.0 binary): `gui-schema --classify-descriptor "wpkh(@0[deadbeef/84'/0'/0']/<0;1>/*)"` → `canonical`; GUI regex does NOT match that string |
| **L13** | B-policy-collapse (coverage) | `convert --from seedqr=<digits>` unreachable from the GUI: `NODE_TYPES` (shared by `--from` + `--to`) omits `seedqr` | YES — `schema/mnemonic.rs:140-154` (13 values, no `seedqr`); `:1115`→`--from`, `:1125`→`--to` both use `NODE_TYPES` | static: `seedqr` absent; toolkit `NodeType::as_str` has `seedqr` at index 1 (`convert.rs:58`), present at the v0.60.0 pin |

---

## 3. Per-finding fix design

### 3.1 M9 — recursive `TreeNode` zeroize-walk wired into `zeroize_form_state`

**Current state (`0bbe3e1`):**
- `zeroize_form_state(state: &mut crate::schema::FormState)` — `src/secrets.rs:294-326`. Iterates `state.values` (`:295`), `state.slots.rows` (`:308`), `state.positionals` (`:311`), `state.secret_widgets` (`:323`). **Never touches `state.tree`.**
- `FormState.tree` is `Option<crate::form::tree_model::TreeState>` — `src/schema/mod.rs:333` (`#[serde(default)]`).
- `TreeState.root: TreeNode` — `src/form/tree_model.rs:48`. (The tree entry is a single `root` node, NOT an `Option`; the `Option` is on `FormState.tree`.)
- `TreeNode` — `src/form/tree_model.rs:72-105`. Secret-bearing fields:
  - `pub key: String` (`:81`) — `Key`-shape pk/pkh; user can type an xprv/WIF/raw-hex private key.
  - `pub keys: Vec<String>` (`:89`) — `multi`/`sortedmulti` quorum rows; same.
  - recurses via `pub children: Vec<TreeNode>` (`:104`).
- **NOT in scope (deliberate):** `pub hex: String` (`:93-97`) is a digest field (`sha256`/`hash256`/`hash160`/`ripemd160`) — a **public commitment**, never a private preimage. The on-disk redactor explicitly leaves it untouched (`tree_model.rs:693-694`: "Hashlock `hex` FIELDS are untouched"). We mirror that: **zeroize `key` + `keys` only, never `hex`** (keeps M9's blast radius exactly the secret-bearing fields and matches the established taxonomy). `w` (wrap prefix), `kind`, `n`, `k`, `id` are non-secret structural fields — untouched.

**On-disk is ALREADY defended (this is in-memory hygiene only):** `persistence.rs` → `TreeState::redacted_for_persistence` (`tree_model.rs:176-187`) → `blank_non_extended_public_keys` (`tree_model.rs:695-707`) recursively `.clear()`s every non-extended-public `key`/`keys` before write (fail-closed allowlist: anything not xpub-shaped is blanked). M9 is **strictly the in-RAM non-scrub** — `zeroize` is never *called* on the tree at all (distinct from an allocator-residue-after-zeroize issue).

**Fix:**
1. Add a recursive helper alongside `TreeNode` in `src/form/tree_model.rs` (sibling to the existing on-disk `blank_non_extended_public_keys`, which is the structural model to copy):
   ```text
   // pseudo-design, NOT final code — for R0 review only
   impl TreeNode {
       pub fn zeroize_keys(&mut self) {           // name TBD by impl; mirrors blank_non_extended_public_keys shape
           use zeroize::Zeroize;
           self.key.zeroize();                    // String: Zeroize (proven below)
           for k in &mut self.keys { k.zeroize(); }
           for child in &mut self.children { child.zeroize_keys(); }
       }
   }
   ```
   Rationale for living in `tree_model.rs` (not `secrets.rs`): the recursion is over `TreeNode`'s private structure and exactly parallels the existing `blank_non_extended_public_keys` recursion — co-locating keeps the two tree-walks adjacent so a future field addition is caught by both.
2. Wire it into `zeroize_form_state` (`secrets.rs`, after the `secret_widgets` loop at `:323-325`):
   ```text
   if let Some(tree) = state.tree.as_mut() {
       tree.root.zeroize_keys();
   }
   ```

**`String: Zeroize` is proven reachable** — no type change required:
- `Cargo.toml:20` declares `zeroize = "1"` (resolved **1.8.2** in `Cargo.lock`). `zeroize::Zeroize` is impl'd for `String` under the default `alloc` feature.
- The same module ALREADY calls `.zeroize()` on plain `String`s: `secrets.rs:300` (`s.zeroize()` on a `FlagValue::Text/Dropdown/Path` `String`). So the trait + import path are live in exactly this file. `TreeNode.key`/`.keys[i]` are the same plain-`String` type → no `Zeroizing<…>` wrapper or struct change is needed; this is a pure sweep extension.

**Covers ALL tree key fields?** YES for the secret-bearing set: `key` + every `keys[i]`, recursively through `children`. `hex` is intentionally excluded (public digest, per the established taxonomy above). There is no other private-key-bearing `String` on `TreeNode` (the `0bbe3e1` struct has exactly `key`/`keys` as private-key inputs; `w`/`kind` are structural).

**egui undo-ring residue caveat (known second-tier residue — DOCUMENT, do not chase):** the egui `text_edit_singleline` widgets rendering `node.key` / `node.keys[i]` (`tree_form.rs:697`, `:717`) maintain an internal **undo ring** that may retain prior keystrokes after our `String` is zeroized — the same class as the already-tracked `gui-secret-buffer-allocator-residue` / `gui-os-snapshot-secret-occlusion` caveats (referenced in `PASTE_WARN_MODAL_TEXT`, `tests/secrets.rs:62-68`). This cycle scrubs the model-side `String` (the authoritative store + the persisted surface's source); the widget undo-ring residue is a **separate, deferred** mitigation (FOLLOWUP `gui-tree-key-egui-undo-ring-residue`, §6). M9 is satisfied by the model-side scrub — it brings `state.tree` to parity with the values/slots/positionals/secret_widgets that the sweep already covers (those have the identical undo-ring caveat).

### 3.2 L12 — reposition the origin bracket AFTER `@\d+` in the canonicity regex

**Current state (`0bbe3e1`):** `classify_descriptor_canonicity` — `src/form/conditional.rs:99-126`. Three single-key regexes (`:107` pkh, `:109` wpkh, `:111` tr) each anchor the optional origin bracket BEFORE `@\d+`:
```
^pkh\((?:\[[0-9a-fA-F]{8}(?:/\d+'?h?)*\])?@\d+(?:/<[0-9;]+>)?(?:/\*+'?h?)?\)$
       └──────── origin bracket BEFORE @N (prefix form only) ────────┘
```
(the wsh/sh-wsh multisig regexes `:113`/`:115` carry no origin bracket and are unaffected).

`is_descriptor_non_canonical(state)` (`:136-141`) returns true iff `--descriptor` present AND classifies NonCanonical. `bundle()` (`:238-245`) pushes the `--account → PinValue(0)` pin **only when `!is_descriptor_non_canonical`**; when NonCanonical the pin is **lifted** → user `--account N` flows to the toolkit.

**Why it's wrong:** the toolkit accepts the SINGLE suffix-origin position `@N[fp/path]` (the L12 form) and decides canonicity *structurally* via `canonical_origin(&tree).is_some()` (`gui_schema.rs:1318-1325`, `bundle.rs:1396-1407`) — **not** by textual bracket position. (For grammar authority: master `parse_descriptor.rs:97-98` carries named groups `pfx_fp`/`pfx_path` for prefix `[fp]@N` AND `sfx_fp`/`sfx_path` for suffix `@N[fp/path]`; the pinned **v0.60.0** lexer accepts the suffix form via its suffix-only regex `:69-70`. A SINGLE suffix-origin is accepted at BOTH versions — the version split matters only for the DOUBLE-origin edge case discussed in the "Benign over-acceptance" note below.) **Empirically confirmed against the pinned v0.60.0 binary:** `gui-schema --classify-descriptor "wpkh(@0[deadbeef/84'/0'/0']/<0;1>/*)"` → **`canonical`**. The GUI regex does not match that suffix-form string → classifies NonCanonical → lifts the pin → `--account N` reaches the toolkit → the toolkit deems the SAME descriptor canonical and hard-errors `DESCRIPTOR_WITH_NONZERO_ACCOUNT` ("--account != 0 is meaningful only with --template; descriptor mode encodes account index in the @i origin path", `bundle.rs:194`). A confusing hard-error on a valid input — NOT a silent wrong-address.

**Fix — exact regex change (apply to all three of `:107`/`:109`/`:111`):** insert a SECOND optional origin-bracket group immediately AFTER `@\d+` (keeping the existing prefix group, so both positions match). Per-regex, the substring
```
@\d+(?:/<[0-9;]+>)?
```
becomes
```
@\d+(?:\[[0-9a-fA-F]{8}(?:/\d+'?h?)*\])?(?:/<[0-9;]+>)?
```
i.e. the new suffix bracket sits between `@\d+` and the optional `/<…>` multipath, exactly where the toolkit's suffix grammar places it (`@N[fp/path]/<mpath>/*`). The leading prefix group is retained (the toolkit still accepts prefix form; the existing `canonicity_drift.rs:110` prefix fixture must keep passing).

**Empirical regex verification (positive controls — Python `re`, mirrors Rust `regex` anchored semantics):**

| input | current regex | fixed regex | toolkit verdict (v0.60.0) | post-fix agreement |
|---|---|---|---|---|
| `wpkh([deadbeef/84'/0'/0']@0/<0;1>/*)` (prefix) | MATCH (Canonical) | MATCH (Canonical) | `canonical` | ✅ unchanged |
| `wpkh(@0[deadbeef/84'/0'/0']/<0;1>/*)` (suffix, **L12**) | **— (NonCanonical, BUG)** | **MATCH (Canonical)** | `canonical` | ✅ **fixed** |
| `wpkh(@0)` (no-origin) | MATCH | MATCH | `canonical` | ✅ unchanged |
| `wpkh(@0/<0;1>/*)` (no-origin + multipath) | MATCH | MATCH | `canonical` | ✅ unchanged |
| `wpkh(@0[…84h/0h/0h…]/<0;1>/*)` (suffix, `h`-hardened) | — | MATCH | `canonical` | ✅ fixed |

**Does it break prefix / no-origin?** NO — the prefix group is unchanged and the no-origin cases match through both optional groups (proven above). The pin continues to fire (account → 0) for every canonical form and lifts only for genuinely non-canonical descriptors (exotic miniscript, `tr` with TapTree).

**Benign over-acceptance noted for R0 (evaluated AT THE PIN, `mnemonic-toolkit-v0.60.0`):** the double-origin form `wpkh([fp]@0[fp]/<0;1>/*)` now MATCHES the GUI regex (classifies Canonical). **At the pinned v0.60.0 the toolkit ACCEPTS this form** (it does NOT reject it). Mechanism: the v0.60.0 placeholder lexer regex (`parse_descriptor.rs:69-70` at the v0.60.0 pin) matches ONLY the suffix bracket `@N[fp/path]` — it has no prefix-bracket alternative. A leading `[fp]` BEFORE `@N` is therefore not matched by the placeholder lexer; `captures_iter` skips it and the prefix bracket is silently dropped. So the v0.60.0 binary classifies the double-origin string as **`canonical`** (exit 0) and `bundle` advances to fingerprint-matching honoring ONLY the suffix annotation (empirically: v0.60.0 `bundle` on `wpkh([deadbeef/…]@0[cafef00d/…]/…)` fp-matches the SUFFIX fp `cafef00d` → exit 0 on match). The over-acceptance is therefore benign **at the pin**: the GUI classifying it Canonical → pinning `--account 0` is **compatible** with a descriptor the v0.60.0 toolkit accepts (no hard error downstream — unlike the L12 single-suffix-form bug, where the GUI under-accepted and lifted the pin).

**Version note (forward-safety across the next pin bump):** toolkit **master / v0.62.0+** TIGHTENED this. The post-v0.62.0 lexer (master `parse_descriptor.rs:98`) uses named groups (`pfx_fp`/`sfx_fp`) that match BOTH bracket positions, and explicitly REFUSES the double-origin form ("`@N carries BOTH a prefix `[fp/path]@N` and a suffix `@N[fp/path]` key-origin annotation; this double-origin is ambiguous — supply exactly one`", master `parse_descriptor.rs:113-116`, shipped by H7 commit `36095b88`). This does NOT change the GUI design: under v0.62.0+ the double-origin string is refused at parse → the `--account 0` pin value is irrelevant (the descriptor never reaches account semantics). So the GUI classifying it Canonical is compatible under BOTH toolkit versions — v0.60.0 ACCEPTS (pin fine), v0.62.0+ REFUSES at parse (pin moot). Strictly safer either way; D7's NO-tighten decision survives both.

The GUI canonicity regex is, by design, a **coarse pin-gate, not a full validator** (the toolkit is the authority); classifying a string the pinned toolkit accepts as Canonical agrees with the toolkit's own verdict, so it is NOT a regression. (A tighter "exactly one of prefix/suffix" regex is possible but adds complexity for zero behavioral gain, and v0.62.0+ already enforces it toolkit-side — explicitly out of scope.)

**Drift-corpus extension:** add a SUFFIX-form fixture `("wpkh(@0[deadbeef/84'/0'/0']/<0;1>/*)", Expect::Canonical)` to `tests/canonicity_drift.rs`'s fixture table (currently only PREFIX `[deadbeef/44'/0'/0']@0/<0;1>/*` @ `:110`). This closes the corpus gap that let the drift gate miss L12 (GUI + toolkit happened to agree on the prefix-only corpus). The fixture shells out to `gui-schema --classify-descriptor` per the existing harness (`:47-51`) so it asserts live GUI-vs-toolkit parity. **Also update the fixture-count comment** at `tests/canonicity_drift.rs:132` — `"11 Canonical + 4 NonCanonical + 3 ParseFails = 18 (15 classify, 3 parse-fail)"` becomes `"12 Canonical + 4 NonCanonical + 3 ParseFails = 19 (16 classify, 3 parse-fail)"` (the new Canonical suffix fixture bumps Canonical 11→12, total 18→19, classify-count 15→16).

### 3.3 L13 — split `NODE_TYPES` into a `--from` set (+seedqr) and a seedqr-free `--to` set

**Current state (`0bbe3e1`):** `src/schema/mnemonic.rs:140-154` — one `const NODE_TYPES: &[&str]` of 13 values (`phrase, entropy, xpub, xprv, wif, fingerprint, path, ms1, mk1, bip38, minikey, electrum-phrase, address`), shared by:
- `--from` (`:1115`): `FlagKind::NodeValueComposite(NODE_TYPES)`
- `--to` (`:1125`): `FlagKind::Dropdown(NODE_TYPES)`

The comment at `:133-139` claims `NODE_TYPES` "exactly mirrors upstream `NodeType::as_str()`" — but it actually matches the toolkit's **`--to` runtime restriction** (`PossibleValuesParser`), not the full enum. `NodeType::as_str` HAS `seedqr` at index 1 (`convert.rs:58`); the `--to` `PossibleValuesParser` (`convert.rs:209-223`) DELIBERATELY OMITS it. So the current shared list is "the `--to`-legal set", silently reused for `--from`.

**Why splitting is MANDATORY (the footgun):** the toolkit **rejects `--to seedqr`** (seedqr is decode/input-only; `PossibleValuesParser` excludes it). If we naively add `seedqr` to the SHARED list, the `--to` dropdown would (a) offer a value the toolkit refuses (a guaranteed-error UI choice) and (b) diverge from gui-schema's `--to` enum. So:

**Fix:**
1. Introduce a NEW const **`CONVERT_FROM_NODES`** = the 14-value list with `seedqr` inserted at **index 1** (immediately after `phrase`, mirroring `NodeType::as_str` ordering at `convert.rs:57-58`):
   `phrase, seedqr, entropy, xpub, xprv, wif, fingerprint, path, ms1, mk1, bip38, minikey, electrum-phrase, address`.
2. Keep the existing 13-value seedqr-free list for `--to` — either rename `NODE_TYPES` → **`CONVERT_TO_NODES`** for symmetry/clarity, or leave it `NODE_TYPES` and add `CONVERT_FROM_NODES` alongside. **Decision: introduce `CONVERT_FROM_NODES` and rename the existing const `CONVERT_TO_NODES`** (paired, self-documenting names; both consumed once each, so the rename blast-radius is the two consumer sites + any test referencing the name).
3. Rewire:
   - `--from` (`:1115`): `NodeValueComposite(CONVERT_FROM_NODES)`
   - `--to` (`:1125`): `Dropdown(CONVERT_TO_NODES)`
4. Replace the misleading `:133-139` comment with the split rationale: `CONVERT_FROM_NODES` mirrors `NodeType::as_str` (the input enum, which includes `seedqr`); `CONVERT_TO_NODES` mirrors the toolkit's `--to` `PossibleValuesParser` (which excludes `seedqr` — decode-only). Cite `convert.rs:54-72` (as_str) and `:209-223` (--to parser).

**Confirm `seedqr` reaches `--from` but NOT `--to`:** post-fix, `--from`'s composite offers 14 nodes including `seedqr`; `--to`'s dropdown offers the same 13 as today (no `seedqr`). A user can now select `convert --from seedqr=<digits>` from the GUI (previously unreachable); `--to seedqr` remains un-offerable (correct — the toolkit refuses it).

**NO toolkit pin bump:** `seedqr` (`NodeType::Seedqr`) was added to the toolkit in **v0.31.6** (release commit `5f0b7b45`, "mnemonic-toolkit v0.31.6 — SeedQR --from unification") — long before the pinned **v0.60.0**; it is present at the pin (verified: `git show mnemonic-toolkit-v0.60.0:…/convert.rs` has `Self::Seedqr => "seedqr"` at `:58`, and the empirical v0.60.0 binary accepts `--from seedqr`). This is a **GUI-local dropdown add** only.

**NO `schema_mirror` / `schema_mirror_secret_drift` trigger (stated reasoning):**
1. `tests/schema_mirror.rs` compares **flag NAMES only** (`schema_flag_names` maps `f.name`; the assertion compares the name set), NOT dropdown VALUES. `tests/schema_mirror_secret_drift.rs` gates only the per-`(subcommand, flag)` **secret-bit**, not value enums. Neither test compares dropdown value lists.
2. The toolkit's `gui-schema` emits a dropdown value enum for `--to` ONLY (its `--to` uses `PossibleValuesParser` → enumerable). `--from` uses a **custom** `value_parser = parse_from_input` (toolkit `convert.rs:44`) → gui-schema classifies it `kind: "text"` (custom value_parsers collapse to text) → **no `--from` value enum is emitted at all.** So there is no `--from` value list anywhere for schema_mirror to compare against. And `--to` keeps the existing seedqr-free list → its enum still matches gui-schema's `--to` enum (both lack seedqr). Hence **zero** schema_mirror surface is touched — confirmed two independent ways.

---

## 4. SemVer / PR-gate / lockstep

| Item | Decision | Why |
|---|---|---|
| **SemVer** | GUI MINOR **0.45.0 → 0.46.0** | L13 adds a user-reachable dropdown VALUE (`--from seedqr`) — a new user-facing conversion source = additive surface → MINOR per project convention. M9 (security fix) + L12 (bug fix) ride along; the highest-precedence change (L13 additive) sets MINOR. |
| **Ship mechanism** | **PR + 5-target CI green BEFORE tag-push** (PR-CI-gate, NOT direct-FF) | GUI process invariant — `FOLLOWUPS.md:716` (`§v0.2: enforce PR-CI gate before tag-push`). The cycle-11a fix lands as ONE PR. |
| **Toolkit pin bump** | **NONE** — stays `mnemonic-toolkit-v0.60.0` | seedqr present since v0.31.6 < v0.60.0; M9/L12/L13 are all GUI-local. No sibling-codec change. |
| **`schema_mirror` lockstep** | **NONE** | flag-NAME gate; L13 is a `--from` value add via the toolkit's custom parser (no emitted `--from` enum); M9/L12 touch no clap surface. (§3.3 reasoning.) |
| **`schema_mirror_secret_drift`** | **NONE** | gates the per-(sub,flag) secret-bit; no flag's `secret:` bit changes (the M9 fix is a sweep extension on existing state; L13 adds no new secret flag — `--from`'s composite secrecy is already node-dependent, `schema/mnemonic.rs:1119`). |
| **Sibling-codec FOLLOWUP companions** | **NONE** | GUI-only cycle; no `md`/`mk`/`ms`/toolkit surface. |
| **`cargo fmt`** | **NEVER run it** | GUI has NO fmt CI gate; formatting churns unrelated lines (project standing instruction). |

**Version-site ritual (GUI — all must move in lockstep, gated):**
- `Cargo.toml:3` `version = "0.45.0"` → `"0.46.0"`.
- `README.md:42` self-pin `--tag mnemonic-gui-v0.45.0` → `mnemonic-gui-v0.46.0` (gated by `tests/readme_pin_coherence.rs`: the self-line `<TAG>` must equal `mnemonic-gui-v{Cargo.toml version}`).
- `README.md:50` toolkit pin `--tag mnemonic-toolkit-v0.60.0` — **UNCHANGED** (no pin bump; `readme_pin_coherence`/`pin_coherence` assert it matches `pinned-upstream.toml`, which we do NOT touch).
- No other version-markered site (GUI has no second README / no fuzz lockfile of its own to bump for this cycle).

---

## 5. Per-finding tests (TDD, RED-first)

Every test below is written to FAIL against `0bbe3e1` and PASS only after the fix. RED-first verification is part of each phase.

### 5.1 M9 — `tests/secrets.rs` (mirrors `zeroize_form_state_clears_text_dropdown_path_and_slots` @ `:317-356`)
- **`zeroize_form_state_clears_tree_keys`** (NEW): build a `FormState` whose `tree` is `Some(TreeState { root: TreeNode { key: "xprv9s21ZrQH…".into(), keys: vec!["xprvA…".into(), "L1aW4…WIF".into()], children: vec![TreeNode { key: "xprv…nested".into(), .. }], .. }, .. })`; call `zeroize_form_state(&mut state)`; assert `state.tree.as_ref().unwrap().root.key.is_empty()`, every `root.keys[i].is_empty()`, AND the nested `children[0].key.is_empty()` (proves the recursion). Mirror the existing test's `assert!(s.is_empty(), …)` style (`String::zeroize` clears bytes AND length).
  - **RED proof:** against `0bbe3e1` the sweep never touches `state.tree`, so `root.key` is still `"xprv9s21ZrQH…"` → the `is_empty()` assert fails. Genuinely RED.
- Optionally assert `root.hex` (if planted) is **untouched** to pin the deliberate digest-exclusion (defends against a future over-broad walk that zeroizes public commitments). Low priority; document the intent in the test comment.

### 5.2 L12 — `tests/non_canonical_descriptor_account_pin.rs` (+ `tests/canonicity_drift.rs`)
- **`suffix_form_origin_descriptor_pins_account_to_zero`** (NEW, in `non_canonical_descriptor_account_pin.rs`, mirroring `canonical_descriptor_pins_account_to_zero` @ `:27-42`): `state_with_descriptor("wpkh(@0[deadbeef/84'/0'/0']/<0;1>/*)")`; assert the `bundle()` visibility vector PINS `--account` to `PinValue(0)` (i.e. classified Canonical → pin fires).
  - **RED proof:** against `0bbe3e1` the regex misses the suffix bracket → NonCanonical → pin LIFTED → assert (pin present) fails. Genuinely RED.
- **Positive controls retained (must stay GREEN):** `canonical_descriptor_pins_account_to_zero` (prefix/no-origin `pkh(@0)`) and `non_canonical_descriptor_lifts_account_pin` (exotic miniscript) are existing tests — the fix must not flip either. Add an explicit **`prefix_form_origin_still_pins`** assertion (`wpkh([deadbeef/84'/0'/0']@0/<0;1>/*)` → pinned) and **`no_origin_form_still_pins`** (`wpkh(@0)` → pinned) if not already covered, to prove the bracket-reposition doesn't regress the prefix/no-origin paths.
- **`tests/canonicity_drift.rs` fixture add:** `("wpkh(@0[deadbeef/84'/0'/0']/<0;1>/*)", Expect::Canonical)` — the live GUI-vs-toolkit parity harness. RED today (GUI says NonCanonical, toolkit `gui-schema` says canonical → `regressed` mismatch); GREEN after the regex fix. (This is the gate that should have caught L12 originally.)

### 5.3 L13 — `tests/` schema test (the dropdown VALUE assertion)
- **`convert_from_dropdown_includes_seedqr`** (NEW): assert the `--from` flag's `FlagKind::NodeValueComposite` value list (`CONVERT_FROM_NODES`) **contains** `"seedqr"`, at index 1 (after `phrase`).
- **`convert_to_dropdown_excludes_seedqr`** (NEW): assert the `--to` flag's `FlagKind::Dropdown` value list (`CONVERT_TO_NODES`) does **NOT** contain `"seedqr"` (mirrors the toolkit `--to` `PossibleValuesParser` refusal).
  - **RED proof:** against `0bbe3e1`, `--from` shares `NODE_TYPES` which lacks `seedqr` → `includes_seedqr` fails. (`excludes_seedqr` is GREEN today and stays GREEN — it's the guard that the split didn't leak seedqr into `--to`.) The `includes` assertion is genuinely RED.
  - Place these where the schema's flag/value lists are introspectable (extend `tests/secret_taxonomy_pin.rs` or a small new `tests/convert_node_dropdowns.rs`; final home an impl-detail, not a design decision).

---

## 6. FOLLOWUP slugs (file in `mnemonic-gui/FOLLOWUPS.md` — repo root, NOT `design/` — at ship)

| Slug | Status at ship | Note |
|---|---|---|
| `gui-tree-key-not-zeroized-on-exit` | **CLOSED by this cycle** (M9) | the in-RAM model-side scrub; flip in the shipping commit. |
| `gui-canonicity-regex-suffix-origin-misclassify` | **CLOSED by this cycle** (L12) | regex reposition + suffix-form drift fixture. |
| `gui-convert-from-dropdown-missing-seedqr` | **CLOSED by this cycle** (L13) | `CONVERT_FROM_NODES` split. |
| `gui-tree-key-egui-undo-ring-residue` | **NEW / OPEN** | second-tier residue: egui `text_edit_singleline` undo ring may retain tree-key keystrokes after the model `String` is zeroized — same class as `gui-secret-buffer-allocator-residue` / `gui-os-snapshot-secret-occlusion`. Deferred (mirrors the existing residue caveats). |
| `gui-canonicity-regex-coarse-pin-gate-not-validator` | **NEW / OPEN (doc-only, optional)** | record that `classify_descriptor_canonicity` is intentionally a coarse pin-gate (over-accepts the double-origin form `[fp]@N[fp]`, which the pinned v0.60.0 toolkit ALSO accepts — and which master/v0.62.0+ refuses at parse — so the over-acceptance is benign under both); the architectural alternative is to call `gui-schema --classify-descriptor` as the single oracle (larger change, deferred). |

(The bug-hunt report's `- [ ]` checkboxes for M9/L12/L13 — `:720`/`:553`/`:563` — get ticked with the fixing commit SHA per the report's "fix checklist" contract.)

---

## 7. Resolved decisions (NO open questions)

| # | Question | Decision |
|---|---|---|
| D1 | M9 — change `TreeNode.key/keys` to a zeroizing type, or just `.zeroize()` the plain `String`s? | **Just `.zeroize()` the plain `String`s.** `String: Zeroize` is live (proven: `secrets.rs:300` already does it; zeroize 1.8.2). A zeroizing-type upgrade is a larger, separable change not required by M9. |
| D2 | M9 — where does the recursion helper live? | **In `src/form/tree_model.rs`**, sibling to `blank_non_extended_public_keys`, as an `impl TreeNode` method; called from `secrets.rs::zeroize_form_state` via `state.tree.as_mut()` → `tree.root.zeroize_keys()`. |
| D3 | M9 — zeroize `hex` too? | **NO.** `hex` is a public digest (commitment), excluded from on-disk redaction too (`tree_model.rs:693-694`). Scope = `key` + `keys` only. |
| D4 | M9 — is the egui undo ring in scope? | **NO** — deferred FOLLOWUP `gui-tree-key-egui-undo-ring-residue`; M9 brings `state.tree` to parity with the already-swept fields (identical residue caveat). |
| D5 | L12 — reposition vs call `gui-schema --classify-descriptor`? | **Reposition the regex** (minimal, no new subprocess per keystroke). The single-oracle alternative is a larger architectural shift → deferred doc FOLLOWUP. |
| D6 | L12 — keep the prefix group or replace it with suffix? | **Keep BOTH** (toolkit accepts both positions; the existing prefix fixture must stay Canonical). Add a second optional bracket AFTER `@\d+`. |
| D7 | L12 — tighten to "exactly one of prefix/suffix"? | **NO.** Compatible under BOTH toolkit versions (see §3.2 "Benign over-acceptance" + "Version note"). At the **pin (v0.60.0)** the suffix-only lexer regex (`parse_descriptor.rs:69-70`) matches only the suffix bracket; a leading `[fp]` is not matched and is silently skipped by `captures_iter`, so v0.60.0 ACCEPTS the double-origin form (`canonical`, exit 0) → GUI→Canonical→`--account 0` is compatible. At **master / v0.62.0+** the named-group lexer REFUSES the double-origin ("supply exactly one", master `parse_descriptor.rs:113-116`, H7 `36095b88`) → the pin value is moot (refused at parse). Either way the GUI's coarse pin-gate classification is benign; tightening the GUI regex adds complexity for zero behavioral gain (and v0.62.0+ enforces it toolkit-side). |
| D8 | L13 — shared list + filter `--to`, or two consts? | **Two consts** (`CONVERT_FROM_NODES` +seedqr, `CONVERT_TO_NODES` seedqr-free). Explicit + self-documenting; no runtime filtering. |
| D9 | L13 — seedqr ordering in `CONVERT_FROM_NODES`? | **Index 1 (after `phrase`)**, mirroring `NodeType::as_str` (`convert.rs:57-58`). |
| D10 | SemVer | **MINOR 0.46.0** — L13's reachable dropdown value is additive user-facing surface; M9/L12 ride along. |
| D11 | Toolkit pin bump? | **NO** — seedqr present since v0.31.6 < pinned v0.60.0. |
| D12 | schema_mirror lockstep? | **NO** — flag-NAME gate; `--from` has no emitted value enum (custom parser); `--to` keeps the seedqr-free list. |
| D13 | Ship mechanism? | **PR + 5-target CI green before tag** (PR-CI-gate; `FOLLOWUPS.md:716`). Never direct-FF. Never `cargo fmt`. |

---

## 8. Mandatory R0-gate note

Per CLAUDE.md: **NO code before this brainstorm spec passes an opus-architect R0 review to 0 Critical / 0 Important.** R0 is mandatory, never skipped/deferred. Reviewer-loop mechanics: dispatch architect → fold findings → **persist the review verbatim to `design/agent-reports/`** → re-dispatch → repeat until GREEN (the loop continues after EVERY fold — a fold can introduce drift). Only after GREEN does the impl plan-doc get authored (and pass its OWN R0 loop), then single-subagent-per-phase TDD with per-phase R0 reviews persisted verbatim, then a mandatory non-deferrable whole-diff post-impl adversarial review. This spec is decision-complete and R0-ready (Resolved-decisions table §7 = no open questions).
