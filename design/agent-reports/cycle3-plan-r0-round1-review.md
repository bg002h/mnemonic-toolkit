# cycle-3 PLAN R0 — round 1 — opus-architect adversarial review

**Date:** 2026-06-21
**Artifact under review:** `design/IMPLEMENTATION_PLAN_cycle3_gui_secret_leaks.md`
**Implements (R0-GREEN spec):** `design/BRAINSTORM_cycle3_gui_secret_leaks.md` (0C/0I @ round 2)
**Gate:** HARD R0 — no code until 0 Critical / 0 Important.

## Repo SHAs (verified live this review)

| Repo | `origin/master` | Role |
|---|---|---|
| **mnemonic-gui** (fix repo) | `0b1e024eab67638da844a52f09d6964c4f55c8df` | all citations re-verified via `git show origin/master:<path>`; crate **0.44.0** |
| mnemonic-toolkit (dep) | `c9168aac` (per plan) | exposes `SECRET_NODE_TYPES_ARGV` at tag `mnemonic-toolkit-v0.60.0`; NO change |

**Verification method:** every edit site, version site, predicate, test-referenced
symbol, and the toolkit taxonomy delta was re-grepped against the LIVE trees above
(GUI `origin/master`, toolkit at tag `mnemonic-toolkit-v0.60.0`). Findings below cite
the live line numbers, not the plan's snapshots.

---

## Citation-verification ledger (all PASS unless noted)

- **runner.rs:119** — verbatim `debug!(target: "mnemonic_gui::runner", argv = ?argv, stdin = stdin.is_some(), "subprocess spawn");` ✔. Fires BEFORE `Command::new(...).spawn()?` (so the event is captured even if spawn fails) ✔. `run()` (`:81`) delegates to `run_with_stdin` (`:106`) so every spawn path hits `:119` ✔. Exit logs `:166-168`-equivalent + stdin `warn!` carry no argv ✔. `program = %argv[0]` (`Command::new(OsStr::new(&argv[0]))`) is provably never-secret ✔.
- **Toolkit delta = exactly `{minikey}`** — `SECRET_NODE_TYPES` (8 entries, `:76-85`) vs `SECRET_NODE_TYPES_ARGV` (same 8 + `"minikey"` at index 8, `:95-105`) at tag v0.60.0 ✔.
- **secrets.rs:34** re-export (narrow only) ✔; **:160-161** `node_type_is_secret` (narrow-backed) ✔; **:230** composite branch of `should_confirm_run` ✔.
- **invocation.rs:457** `node_type_is_secret(node)` in the `mask.push(...)` ✔; `assemble_argv_with_secret_mask` is `pub` at `:151` (test-callable) ✔.
- **persistence.rs:31** import, **:96** `SECRET_NODE_TYPES.contains`, **:16/:73/:94** doc/inline comments ✔.
- **widget.rs:646** is a bare `ui.text_edit_singleline(value)` in the `NodeValueComposite { node, value }` arm with **no captured response, no paste scan** ✔. `node`/`value` are `&mut String`.
- **secret_widget.rs** `paste_warn_id()` is `pub fn` (`:36`), reachable at `crate::form::secret_widget::paste_warn_id()` (module is `pub mod secret_widget` in `form/mod.rs:7`) ✔. The plan's paste-detect code is byte-faithful to `SecretLineEdit::show` (`:85-105`) ✔.
- **schema/mnemonic.rs:1119** comment verbatim ✔; banner `:4344` = `"mnemonic 0.59.0"` ✔.
- **Version sites:** `Cargo.toml:3 = "0.44.0"` ✔; `README.md:42 --tag mnemonic-gui-v0.44.0` ✔; `Cargo.lock` `mnemonic-gui` block at `:2265` (plan says ~2266) ✔; `readme_pin_coherence` asserts the self-line `== format!("mnemonic-gui-v{}", cargo_version())` (`tests/readme_pin_coherence.rs:75`) ✔.
- **FOLLOWUPS** `composite-paste-warn-parity` at `FOLLOWUPS.md:40`, `slot-field-paste-warn-uncovered` at `:48` ✔. Toolkit bughunt H2 `- [ ]` `:93`, H3 `- [ ]` `:111` ✔.
- **No existing test regresses under the predicate swap:** `tests/secrets.rs` `should_confirm_run` composite cases use `node:"phrase"` (true→true) and `node:"xpub"` (false→false); `secret_node_types_set_pinned` (`:248`) pins the NARROW set, which the plan correctly keeps narrow ✔.
- **schema_mirror / schema_mirror_secret_drift** collect flag-NAME + per-flag `secret==true` sets (the `--from` flags stay `secret:false`); a comment-only edit is invisible to both ✔.

---

## Critical

**None.** The plan's edit sites, predicates, version sites, and the toolkit delta are
all verified-correct. The four routed surfaces are the right four; the narrow
predicate correctly stays narrow; no funds-leak surface is missed; no existing test
regresses.

---

## Important

### I1 — H2 test mitigation is materially weaker than the repo's PROVEN solution to the IDENTICAL race, and names an unprovisioned dependency

The plan's I1 mitigation (plan `:35-40`, spec `:113-124`) is: scoped
`with_default` + `rebuild_interest_cache()` + mandatory positive control +
`#[serial]` (serial_test) fallback. Two problems, both verified against the live tree:

1. **`rebuild_interest_cache()` alone is KNOWN-INSUFFICIENT for this exact race in
   this exact repo.** The canonical sibling test
   `cell_2_tracing_init_logs_subprocess_spawn` lives at
   `tests/runner_integration.rs:140-193` and already uses `set_default` +
   `rebuild_interest_cache()` — yet its own comment (`:147-152`) and the GUI FOLLOWUP
   `runner-tracing-test-flaky-under-parallel-load` document that a concurrent test can
   **re-race the global callsite-interest cache between the rebuild and the spawn**,
   transiently dropping the `:119` event. The proven, no-new-dep fix is a
   **3-attempt retry loop** (each attempt: fresh subscriber → rebuild → spawn →
   check), `tests/runner_integration.rs:155-182`. The plan does not adopt this
   retry-loop pattern; it reaches for `#[serial]` instead.

2. **`serial_test` is NOT a dependency.** `Cargo.toml` (verified) has `tracing` +
   `tracing-subscriber` dev-available but **no `serial_test`**. The plan names
   `#[serial]` as the fallback without provisioning the crate, and Phase 1's file list
   omits a `Cargo.toml` dev-dep edit. As written, the fallback is uncompilable.

3. **The negative-assertion interaction makes this load-bearing, not cosmetic.** The
   new test's primary assertion is NEGATIVE (sentinel absent). Under a poisoned
   attempt the local layer captures NOTHING — which makes the positive control
   (`argv_len` present) correctly FAIL (good: it catches the vacuous case) but turns
   a real race into a **flaky RED**, not a stable GREEN. Without the
   retry-until-captured loop, the test is flaky-by-construction under parallel
   `cargo test` — the precise failure mode the repo already hit and fixed via retry.

**Required fix:** mandate the **proven `cell_2` shape** — a bounded retry loop
(fresh subscriber + `rebuild_interest_cache()` each attempt; retry while the positive
control `argv_len` is absent; on a captured attempt assert BOTH `argv_len` present AND
sentinel absent). Drop the `#[serial]`/`serial_test` fallback (no new dep) OR, if
retained as a contingency, explicitly add `serial_test` to `[dev-dependencies]` in
Phase 1's edit set. State that the new test SHOULD live in (or mirror) the existing
`tests/runner_integration.rs` harness (`CapturedWriter` MakeWriter, `mnemonic_bin()`),
which already solves capture + race, rather than re-deriving a weaker scheme in
`runner.rs`'s inline `#[cfg(test)] mod tests` (which today has no tracing-capture
infrastructure at all — verified `:175-260`).

---

## Minor

### M1 — Phase 3 leaves a SECOND identical false comment unaddressed (stale-comment-hygiene scope-miss)

The plan's Phase 3 corrects only `schema/mnemonic.rs:1119`
(`// secrecy is value-dependent; per-row paste-warn fires`). But there are **nine**
`NodeValueComposite` flags (verified: lines 1115, 1496, 1601, 1789, 1928, 1984, 2038,
**2089**, 2212), and the `SEEDQR_DECODE` `--from` flag at **`:2095`** carries the
*same* false claim `// value-dependent; per-row paste-warn fires`. Before this cycle
"per-row paste-warn fires" is FALSE for every `--from` composite (the live path is
`SecretLineEdit`'s bus-flag, which the composite widget never used); after Phase 3's
generic-arm wiring it becomes TRUE — so `:2095` is in the same boat as `:1119`. The
spec itself elevates stale-comment hygiene to a named recurring bughunt class
(spec `:198-202`), so leaving the twin comment stale is inconsistent with the cycle's
own discipline. (No functional impact; the `Edit` at `:1119` is still unambiguous
because the `:1119` string has the distinguishing `secrecy is ` prefix.)
**Required fix (Minor):** either also update `:2095` to an accurate comment, or add an
explicit one-line note in Phase 3 that the other composite comments are knowingly left
(and why the `:1119` one was singled out as "the false comment"). Recommend updating
`:2095` for consistency since the same wiring makes the same claim true/stale.

### M2 — `tests/persistence.rs:156` loop-widening gives ZERO real minikey coverage by itself (TDD-integrity nuance)

Plan item 3 (plan `:72`, spec `:301-305`) says widening the `:156`
`for node in SECRET_NODE_TYPES` on-disk-absence loop to the WIDE set gives `minikey`
"explicit `state.json`-bytes-absence coverage." Verified: that loop iterates over the
secret-node token strings and asserts each token is absent from the on-disk JSON of
`mixed_form_with_secrets()` — but that fixture contains only `node:"phrase"` and
`node:"xpub"` composites (`tests/persistence.rs:56,63`); it has **no minikey
composite**. So widening the loop to include `"minikey"` asserts `"minikey"` is absent
from a state.json that never contained a minikey composite — **vacuously GREEN**, no
exercise of the redaction code path. The genuine minikey-redaction coverage comes
only from the H3 regression test (item 2.iv), which must ADD a `--from minikey=<KEY>`
composite to the form and assert the value is dropped on disk.
**Required fix (Minor):** clarify that the `:156` loop-widening is a defense-in-depth
token-absence belt, and the LOAD-BEARING minikey on-disk coverage is item 2.iv's
regression test, which MUST construct a form state containing a minikey composite (not
reuse the existing `mixed_form_with_secrets()` unchanged). Without this, "explicit
minikey absence coverage" overstates what the loop-widen delivers.

### M3 — Phase 4 FOLLOWUP wording conflates "flip" with "file-and-resolve" for two slugs

Plan `:174` lists `gui-runner-debug-logs-unmasked-secret-argv` (H2) and
`gui-minikey-secret-not-masked-in-argv-preview` (H3) as "→ RESOLVED." Verified: these
are bughunt-report IDs, NOT existing headed entries in GUI `FOLLOWUPS.md` (only
`composite-paste-warn-parity` `:40` and `slot-field-paste-warn-uncovered` `:48` exist
there). So H2/H3 must be **created as entries and marked RESOLVED** in the shipping
commit, whereas `composite-paste-warn-parity` is **flipped**.
**Required fix (Minor):** word Phase 4 to distinguish "create-and-resolve" (H2, H3
new entries) from "flip existing to RESOLVED" (`composite-paste-warn-parity`), per
`feedback_followup_status_discipline`.

### M4 — Phase 3 gate should pass `node.as_str()` (style consistency) — non-blocking

The plan's gate `node_type_is_argv_secret(node)` (plan `:131`) is called with
`node: &mut String`. This compiles via `&mut String → &str` deref coercion, so it is
correct. But the adjacent existing code uses `node.as_str()` explicitly
(`widget.rs:639` `combo.selected_text(node.as_str())`). Recommend `node.as_str()` for
local consistency. Non-blocking.

### M5 — Phase 3 kittest RED test must focus a `TextInput` role, not `PasswordInput`

The existing model `tests/paste_warn_wiring_v0_40_0.rs` focuses the secret field via
`Role::PasswordInput` (verified `:41,59`). The composite value field is a
NON-password `text_edit_singleline`, so it carries `Role::TextInput`. The plan says
the RED test mirrors that file "(kittest)" without flagging the role/harness
difference — an implementer copying the model verbatim would query the wrong role and
get a no-focus vacuous pass. Non-blocking but worth one explicit line.

---

## Cross-cutting checks (all PASS)

- **Phase file-disjointness:** P1 = `runner.rs` (+ test); P2 = `secrets.rs`,
  `form/invocation.rs`, `persistence.rs`, `tests/secret_taxonomy_pin.rs`,
  `tests/persistence.rs`, new regression test; P3 = `form/widget.rs`,
  `schema/mnemonic.rs` (comment), paste test. No file appears in two phases →
  independently reviewable ✔. (`secrets.rs` is touched only in P2 — P3 merely *calls*
  `node_type_is_argv_secret` added in P2, a true sequential dep, correctly ordered.)
- **Per-phase FULL-suite + clippy gate:** stated at every phase gate ✔ (matches
  `feedback_r0_review_run_full_package_suite` — full `cargo test`, not `--test` target).
- **Mandatory post-impl whole-diff adversarial review:** present, non-deferrable,
  before the version bump ✔.
- **`schema_mirror` (×4) + `schema_mirror_secret_drift` + `archetype_schema_mirror` +
  `gui_schema_conditional_drift` + `xpub_search_schema_mirror`:** no flag-name /
  dropdown-value / per-flag-`secret`-bool delta; `minikey` already in `NODE_TYPES`
  (`schema:151`); comment-only edit invisible → all stay green ✔.
- **`readme_pin_coherence` is the one bump-tripped gate** (self-tag == Cargo.toml
  version) ✔. `pinned-upstream.toml` / `pin_coherence` untouched (no pin bump) ✔.
- **Banner `:4344`/`pinned-upstream.toml` "do NOT touch":** correct — pre-existing
  `0.59.0`-vs-`v0.60.0` skew is cosmetic and out of scope ✔.
- **NEVER `cargo fmt` the GUI:** stated ✔.
- **TDD RED integrity:** P1 RED (sentinel present on the `argv = ?argv` line → A2
  fails) ✔; P2 regression RED (`minikey` falls through narrow predicate) ✔; P2
  drift-guard / sibling fallback mod (Q3) compiles a `const _: () = assert!(...)`
  mirroring the existing `v0_3_canonical_fallback` pattern ✔; P3 RED (no detection on
  the bare composite widget) ✔ — subject to M5's role caveat.
- **`node_type_is_argv_secret` over-warn / over-redaction folds:** wide−narrow ==
  `{minikey}`; nothing argv-secret-but-persist-safe exists → wide == correct for
  persistence; non-secret `--from` nodes (xpub/fingerprint/path/mk1/address) don't
  warn/confirm/mask ✔.
- **`response.changed()` paste attribution:** matches the proven SecretLineEdit
  reasoning — `ui.input` reads frame-global events but only the focused field's
  `response.changed()` is true, so no multi-row false-trigger across the nine
  composite widgets ✔.

---

## Verdict

The plan is structurally sound: every edit site, version site, predicate route, and
the toolkit `{minikey}` delta is verified-correct; the four surfaces are the right
four; no funds-leak surface is missed; no existing test regresses; schema/manual/pin
gates are correctly analyzed as untriggered (except the lockstep README bump). The
one blocking issue is the H2 test mitigation (I1): it is weaker than — and diverges
from — the repo's own proven solution to the identical callsite-interest race, and
names an unprovisioned `serial_test` dep. Fold I1 (adopt the `cell_2` retry-loop
pattern / `runner_integration.rs` harness, drop or provision the dep) and the Minors,
then re-dispatch.

**PLAN R0 ROUND 1: 0C / 1I**  → **RED**
