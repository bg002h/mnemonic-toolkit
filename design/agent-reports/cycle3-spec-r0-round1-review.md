# cycle-3 spec R0 — round 1 review

**Artifact under review:** `design/BRAINSTORM_cycle3_gui_secret_leaks.md` (mnemonic-toolkit repo).
**Findings:** H2 (runner logs unmasked argv) + H3 (GUI uses narrow `SECRET_NODE_TYPES`, leaks `minikey` across 4 surfaces).
**Repos / SHAs independently verified against:**
- **mnemonic-gui** (fix repo) `origin/master` = **`0b1e024eab67638da844a52f09d6964c4f55c8df`** (crate 0.44.0, pins `mnemonic-toolkit-v0.60.0`).
- **mnemonic-toolkit** (dep) `origin/master` = **`c9168aac`**; toolkit constants verified at tag **`mnemonic-toolkit-v0.60.0`** (the GUI's pin).
**Date:** 2026-06-21. **Reviewer:** opus software architect (adversarial R0).

---

## Method note

Every citation in the spec was re-grepped against the live trees above (`git show <ref>:<path>`), not trusted from the spec. The toolkit `SECRET_NODE_TYPES` / `SECRET_NODE_TYPES_ARGV` slices were byte-compared at the v0.60.0 tag. All NINE `NodeValueComposite` flags in the GUI schema and ALL runtime callers of `node_type_is_secret` / `SECRET_NODE_TYPES` were enumerated to hunt for a missed leak surface.

**Citation verdict: every cited line/symbol MATCHES current source.** Specifically confirmed accurate:

- H2: `runner.rs:119` (`argv = ?argv`), `run`→`run_with_stdin` delegation (`:81`/`:89`), `mask: Vec::new()` (`:160`), exit logs (`:166-168`), `Command::new(OsStr::new(&argv[0]))`. ✔
- Toolkit: `SECRET_NODE_TYPES` 8 entries (`:76`, no minikey), `SECRET_NODE_TYPES_ARGV` 9 entries (`:95`, `minikey` at `:104`), **delta = exactly `{minikey}`** by direct slice compare. Parity test name `secret_taxonomy_argv_parity_with_is_argv_secret_bearing`, `convert.rs:117 is_argv_secret_bearing`. ✔
- H3 GUI: `secrets.rs:34` (re-export, narrow only), `:160-161` (`node_type_is_secret`), `:194` (`should_warn_on_paste`), `:230` (composite branch in `should_confirm_run`); `invocation.rs:457` (composite mask bit); `persistence.rs:31` (import) + `:96` (drop); `widget.rs:646` (plain `text_edit_singleline`); `schema/mnemonic.rs:1114` (`name:"--from"`), `:1115` (`NodeValueComposite(NODE_TYPES)`), `:1119` (false comment), `NODE_TYPES` def `:140-154` with `minikey` at `:151`; `main.rs:951` (sole `should_confirm_run` caller), `:1057` (`remove_temp` paste-bus), `:348`/`:410`/`:1121`/`:1137` (persist paths). ✔
- The spec's **schema drift correction to `:1113-1122` / comment `:1119`** is CONFIRMED correct (recon's `:912-921` snapshot was stale). ✔
- `secret_taxonomy_pin.rs`, `schema_mirror_secret_drift.rs`, `paste_warn_wiring_v0_40_0.rs`, `runner_integration.rs::cell_2_*` all exist as cited; `FOLLOWUPS.md:40`/`:48`/`:56` slug line numbers accurate (file is repo-root `FOLLOWUPS.md`, which is how the spec cites it). ✔
- Version sites: `Cargo.toml:3` = `0.44.0`, `Cargo.lock:2266`, `README.md:42` self-tag `mnemonic-gui-v0.44.0`, `pinned_version: "mnemonic 0.59.0"` (`schema:4344`), `pinned-upstream.toml [mnemonic].tag = mnemonic-toolkit-v0.60.0` (documentary-only per its own header). ✔

**No citation is wrong.** The spec is unusually clean on facts. The findings below are about *soundness and completeness of the DECISIONS and TESTS*, not citations.

---

## Critical findings

**None.**

The design is funds-safe: it routes argv-mask + run-confirm + persist-redact + paste-warn through the wide (`minikey`-bearing) set; the persistence redaction (the actual plaintext-to-disk leak) is correctly switched to `SECRET_NODE_TYPES_ARGV`; the over-redaction analysis (wide−narrow = `{minikey}`, a private key that MUST drop) is correct; and there is no node that is argv-secret-but-persist-safe, so the persistence set legitimately coincides with the wide set. No decision ships a leak or loses a restorable value.

---

## Important findings

### I1 — H2 negative-assertion test can pass VACUOUSLY under the documented callsite-interest-cache race; the spec's positive control is necessary but the spec must MANDATE it (not leave it to the implementer) and must require the interest-cache flush.

The existing runner tracing test (`tests/runner_integration.rs:140 cell_2_tracing_init_logs_subprocess_spawn`) is not merely "flaky" — its own code documents the exact mechanism (`:141-153`): `set_default` is thread-local (already a scoped subscriber — the spec's Q1 "global `try_init`" framing is a strawman; the real test never used `try_init`), but **tracing's callsite-interest cache is GLOBAL**, so a concurrent test can transiently mark this cell's DEBUG callsite *uninterested* and the event is dropped from the capture buffer. The existing test defends with (1) `tracing::callsite::rebuild_interest_cache()` immediately after `set_default`, and (2) a 3-attempt retry loop.

This race is **asymmetric and dangerous for a negative assertion.** A positive assertion ("subprocess spawn" present) fails CLOSED under a dropped event (false-RED → retry fixes it). The H2 test asserts the *absence* of a sentinel — if the `debug!` event is dropped by the interest-cache race, the captured buffer is empty and `!captured.contains(SENTINEL)` passes **vacuously**: the test goes GREEN while proving nothing, and would stay GREEN even on the UNFIXED `argv = ?argv` line if the event happened to be dropped on that run. That defeats the "RED on current line, GREEN after fix" contract the R0 gate relies on.

The spec *does* say "Also assert it DOES contain `argv_len` (positive control — the spawn-log still fires)" — which is the right instinct — but it is phrased as an aside, and the spec does NOT require the interest-cache rebuild that the existing test proved is necessary. **Required fix (must be in the spec before code):**
1. Make the positive-control assertion **mandatory and load-bearing**, in the SAME captured buffer: assert the buffer contains a stable spawn marker (e.g. `"subprocess spawn"` and/or `program=`/`argv_len=`) so a dropped event fails the test rather than passing it. The negative assertion (sentinel absent) is then meaningful only when the positive control proves the event was captured.
2. Adopt the existing test's race defenses verbatim: scoped `set_default`, `rebuild_interest_cache()` after install, and a bounded retry. Do NOT invent a different isolation scheme.
3. State explicitly that the RED baseline is verified by running the test against the UNCHANGED `argv = ?argv` line and observing it fail on the sentinel-present assertion (with the positive control GREEN), so RED is a real failure, not a vacuous pass.

### I2 — Surface-1 ("argv-mask → masked in preview / copy / last-run") mis-describes the clipboard copy; the spec must state that the REAL `ctx.copy_text` payload stays cleartext by design (informed-reveal), so the implementer does not try to "fix" it or write a test asserting the clipboard is redacted.

Verified at `main.rs:955-1036`: there are two copy renderings. `render_copy_command_masked` drives the on-screen **Preview** (`:957`) and the **last-run `argv:` line** (`:477`, via the carried `result.mask`) — both correctly redact `minikey` once the mask is widened. But the **actual clipboard payload** is `render_copy_command` (UNMASKED, `:958-959`), copied verbatim at `ctx.copy_text(...)` (`:1030`/`:1036`). This is the deliberate v0.39.0 "reveal" half: the button is relabeled `"Copy command (POSIX) — reveals secret"` whenever `any_secret = mask.iter().any(|&m| m)` is true (`:955`,`:980-983`).

Widening the mask therefore does THREE correct things for `minikey`: masks the Preview, masks the last-run line, and flips `any_secret`→true so the copy button gets the reveal-label — bringing `minikey` to **parity with phrase/xprv/etc.** The spec's bullet ("masked in Preview / run-confirm token list / last-run") is right about preview/last-run but the parenthetical "/ copy" is misleading: the real clipboard copy is intentionally cleartext for every secret node, gated by the informed-reveal label, NOT redacted. **Required fix:** reword surface-1 to say the mask widening (a) redacts Preview + last-run, and (b) flips the copy-button to the "— reveals secret" informed-reveal label (matching the existing secret-node behavior); the real clipboard copy remains a deliberate reveal and is OUT of scope to redact. Add a negative-control note so the H3 regression test asserts the *masked-preview* token is `••••` and the *button label* gains "reveals secret" — NOT that `render_copy_command`/clipboard is redacted (that assertion would be wrong and would contradict the v0.39.0 design).

### I3 — The four Open R0 Questions (Q1–Q4) are left unresolved; an R0 gate requires decisions before code. (Rulings supplied below; spec must fold them so the implementer is not making design calls.)

Leaving Q1–Q4 "open" is itself an Important gate violation per the project's R0 discipline ("decisions before code"). I rule on all four in the dedicated section; the spec must adopt those rulings (or argue otherwise and re-review) before the GREEN gate. Q1 in particular is entangled with I1 (test soundness).

---

## Minor findings

### M1 — The H3 fix incidentally closes `composite-paste-warn-parity` for ALL nine composites, not just `convert --from`; the spec under-claims this and should say so (and confirm no over-warn for the non-minikey composites).

Verified: the GUI schema has NINE `NodeValueComposite` flags (`schema:1115,1496,1601,1789,1928,1984,2038,2089,2212`), but only `NODE_TYPES` (the `convert --from` list) contains `minikey`. The other eight are over `["phrase","entropy"]`, `["xprv","phrase"]`, `["seedqr"]`, or `["phrase"]` — all already narrow-covered for mask/confirm/persist. **All nine render through the single `widget.rs:646` `text_edit_singleline(value)`** (the widget dispatches on `FlagKind::NodeValueComposite`, not per-subcommand). So the spec's paste-detection wiring (gated on `node_type_is_argv_secret(node)`) will newly deliver paste-warn to phrase/entropy/xprv/seedqr composites too — closing `composite-paste-warn-parity` *completely*, which is strictly more than the spec claims. This is correct and beneficial (the gate is node-secrecy, so non-secret nodes like `xpub`/`fingerprint`/`path`/`mk1`/`address` still do not warn — no over-warn). The spec should (a) note this fuller scope so the post-impl reviewer expects paste-warn on all secret composites, and (b) ensure the H3 regression test's negative control (`--from xpub=…` no-warn) is complemented by at least one positive non-minikey composite (e.g. seed-xor `--from phrase=…` now warns) so the broader fix is pinned.

### M2 — `tests/persistence.rs:156` iterates the NARROW `SECRET_NODE_TYPES` to assert no leak to disk; it will not gain `minikey` coverage after the fix. Recommend widening that loop to `SECRET_NODE_TYPES_ARGV` (or add a minikey case) so the established on-disk-leak test also covers the fixed node.

The existing `cell_*` persistence test (`tests/persistence.rs:155-163`) loops `for node in SECRET_NODE_TYPES` and asserts each quoted token is absent from `state.json`. Because it iterates the narrow set, it will keep passing after the fix but will NOT assert `minikey` is absent — that coverage lives only in the spec's new H3 regression test. For belt-and-suspenders parity with the rest of the file, widen this loop to `SECRET_NODE_TYPES_ARGV` once the GUI re-exports it (the import at `tests/persistence.rs:23` would extend). Optional but cheap; keeps the canonical on-disk-leak test authoritative.

### M3 — `tests/secrets.rs:247 secret_node_types_set_pinned` hard-pins the narrow set to exactly 8 entries. The spec's two-predicate design keeps the narrow set narrow, so this test stays GREEN — but the spec should note it as the guard that would catch a future "simplify to one widened set" regression (this is precisely the failure mode the two-predicate decision defends against). No change needed; document the dependency.

### M4 — `pinned-upstream.toml [mnemonic].tag` and the `pinned_version: "mnemonic 0.59.0"` banner are already internally inconsistent at HEAD (pin is v0.60.0, banner says 0.59.0). The spec correctly says "do not touch" both — but should add a one-line note that this pre-existing banner/pin skew is a KNOWN, out-of-scope toolkit-display artifact (not introduced by this cycle), so a reviewer does not flag it as new drift.

---

## Open-question rulings

**Q1 (H2 test isolation) — RULING: scoped subscriber, with the I1 hardening MANDATORY.** Use `tracing::subscriber::set_default(local_capture_layer)` (thread-local, exactly as `cell_2` already does — NOT global `try_init`), plus `tracing::callsite::rebuild_interest_cache()` after install, plus a bounded retry. The negative (sentinel-absent) assertion MUST be paired with a load-bearing positive control in the same buffer (spawn marker present) so a dropped event fails rather than passes vacuously (see I1). The spec's fallback ("assert structurally on the format args / `#[serial]`") is acceptable as a secondary, but the scoped-subscriber-plus-positive-control path is the primary and is sufficient. **Ratified with the I1 amendment.**

**Q2 (slot paste-warn fold) — RULING: leave OUT (ratify the spec's lean).** `slot-field-paste-warn-uncovered` is a different widget (`slot_editor.rs`), and — verified — `minikey` is `--from`-only (it is not a `SlotSubkey`; `SECRET_SLOT_SUBKEYS` = phrase/seedqr/entropy/ms1/xprv/wif, no minikey, and slot rows carry subkeys, not node-types). So no `minikey` ever reaches a slot box; the slot gap is real but NOT a funds-leak surface for the two cycle-3 findings. Keep the cycle scoped to the two confirmed leaks; leave the slug OPEN and note it was considered. (If the implementer finds the ~3-LOC slot fix is literally the same bus-flag call already being added to the composite widget, folding it is harmless — but it is NOT required and must not expand the cycle's R0 surface.)

**Q3 (wide-set compile-time guard home) — RULING: new sibling mod, NOT inside `v0_3_canonical_fallback` (ratify the spec's lean).** `v0_3_canonical_fallback` is explicitly scheduled for v0.5.0 deletion (`secrets.rs:36`, `:71-77`). A `SECRET_NODE_TYPES_ARGV` drift-guard placed inside it would be silently deleted with that mod, retiring the wide-set supply-chain guard prematurely. Put the wide-set snapshot + `const _: () = assert!(secret_slice_eq(SECRET_NODE_TYPES_ARGV, snapshot))` in a new, un-sunset sibling mod (reuse the existing `secret_slice_eq`/`const_str_eq` helpers, which live outside the fallback mod). **Ratified.**

**Q4 (predicate name) — RULING: `node_type_is_argv_secret` (ratify the spec's lean).** It parallels the existing `node_type_is_secret`, both operate on `&str` (token form), and reusing the toolkit's `NodeType` method name `is_argv_secret_bearing` (`convert.rs:117`) would falsely imply a `NodeType`-typed receiver. Keep the two-predicate design (narrow `node_type_is_secret` stays for the persistence DOC semantic and the `tests/secrets.rs:247` pin; wide `node_type_is_argv_secret` for the four argv-facing surfaces). **Ratified.** Note: the spec says "`redact_for_persistence` uses the wide set via `SECRET_NODE_TYPES_ARGV` directly" (DECISION i) rather than via `node_type_is_argv_secret` — that is fine and consistent (persistence reads the const directly today at `:96`); just keep the inline `:94` comment + module doc updated to name the wide set (spec already flags this).

---

## Scope / lockstep confirmation (load-bearing, independently checked)

- **No toolkit change / no pin bump:** correct. `SECRET_NODE_TYPES_ARGV` is present at the pinned `mnemonic-toolkit-v0.60.0` tag (verified byte-exact). Pin stays v0.60.0; `pinned-upstream.toml` + `pinned_version` banner untouched (M4).
- **`schema_mirror` / `schema_mirror_secret_drift` NOT triggered:** correct. `--from` stays `secret: false` (only its COMMENT changes); `minikey` already lives in `NODE_TYPES` (`schema:151`) and is an offered dropdown value; no flag-name / dropdown-value / per-flag-secret-bool delta. Verified `schema_mirror_secret_drift` keys on the `{(sub,flag)|secret==true}` set, which is unchanged. `archetype_schema_mirror` / `gui_schema_conditional_drift` / `xpub_search_schema_mirror` unaffected.
- **Manual mirror NOT triggered:** correct (GUI is not in the toolkit manual mirror set; no toolkit CLI surface change).
- **No `cargo fmt`:** correct (GUI has no fmt CI gate; verified no rustfmt step in any GUI workflow). Hand-format.
- **Version sites complete:** `Cargo.toml:3`, `Cargo.lock:2266`, `README.md:42` self-tag (gated by `readme_pin_coherence` → MUST bump to `mnemonic-gui-v0.45.0` in lockstep), `CHANGELOG.md` (`## mnemonic-gui [0.45.0] — <date>`). This is the complete set; the README toolkit/md/ms/mk pins (`:50-53`) correctly stay unchanged. **GUI MINOR 0.44.0 → 0.45.0** is the right SemVer (new masking/confirm/paste/redaction behavior, no breaking API).
- **5th-surface hunt — NEGATIVE (no missed funds-leak):** all runtime callers of the narrow predicate are exactly the four the spec routes (`persistence.rs:96`, `invocation.rs:457`, `secrets.rs:230`, plus the `secrets.rs:161` definition body which stays narrow); `convert --from` is the ONLY `minikey`-bearing composite (8 other composites carry no `minikey`); the clipboard-copy "surface" is the intentional informed-reveal (I2), not a leak; `--to` is a `Dropdown` (no free-text secret entry). No crash-dump / error-path / alternate-persistence leak found.

---

## Verdict

**R0 ROUND 1: 0C / 3I** — **RED.**

Three Important findings (I1 test-soundness, I2 surface-1 mischaracterization, I3 unresolved Q1–Q4) must be folded; Q1–Q4 rulings are supplied above for direct adoption. No Critical findings — the core funds-safety design (wide-set routing, persistence redaction, over-redaction/over-warn analysis) is sound. After folding I1–I3 (and ideally the Minor items M1–M2), re-dispatch for round 2.
