# cycle-3 spec R0 — round 2 review

**Artifact under review:** `design/BRAINSTORM_cycle3_gui_secret_leaks.md` (mnemonic-toolkit repo), **post-fold** (round-1 returned 0C/3I RED; I1/I2/I3 + Minors folded).
**Findings under scope:** H2 (runner logs unmasked argv) + H3 (GUI uses narrow `SECRET_NODE_TYPES`, leaks `minikey` across 4 surfaces).
**Repos / SHAs independently verified against:**
- **mnemonic-gui** (fix repo) `origin/master` = **`0b1e024eab67638da844a52f09d6964c4f55c8df`** (crate 0.44.0, pins `mnemonic-toolkit-v0.60.0`).
- **mnemonic-toolkit** (dep) `origin/master` = **`c9168aac`**; toolkit constants at tag **`mnemonic-toolkit-v0.60.0`**.
**Date:** 2026-06-21. **Reviewer:** opus software architect (adversarial R0, round 2).
**Predecessor review folded:** `design/agent-reports/cycle3-spec-r0-round1-review.md` (0C/3I).

---

## Method note

This round verifies (1) that each of the three round-1 Important findings is **actually resolved in the spec text** — bindingly, not gestured at — and (2) that the folds introduced **no new drift or contradiction** with earlier sections. Every load-bearing fact the folds rest on was re-checked against the live `0b1e024` GUI tree via `git show origin/master:<path>` (not trusted from the spec's self-assessment), including the structures the folded mitigations claim to mirror.

Live re-verifications performed this round:
- `tests/runner_integration.rs:139-194` (`cell_2_tracing_init_logs_subprocess_spawn`): confirmed the real test uses **scoped `tracing::subscriber::set_default`** (NOT `try_init`) + **`tracing::callsite::rebuild_interest_cache()`** immediately after install + a **bounded 3-attempt retry** — the exact race-defense pattern the I1 fold mandates. The fold references this real mechanism accurately.
- `src/secrets.rs:36-71`: confirmed `v0_3_canonical_fallback` mod exists, carries the doc "Removed in v0.5.0", and the `const _: () = assert!(secret_slice_eq(...))` drift-guards + the `secret_slice_eq`/`const_str_eq` helpers live **outside** that mod (reusable by a new sibling mod) — Q3's "new sibling mod" ruling is structurally implementable as written.
- `src/main.rs:956-1036`: confirmed two distinct copy renderings — `render_copy_command_masked` drives **Preview** (`:957`) + **last-run** (`:477`); `render_copy_command` (UNMASKED, `:958-959`) is the actual `ctx.copy_text(...)` clipboard payload (`:1030`/`:1036`); `any_secret` (`:956`) flips the button to `"Copy command (POSIX) — reveals secret"` (`:980`). The I2 fold's clipboard description is byte-accurate.
- `src/schema/mnemonic.rs`: enumerated all **9** `NodeValueComposite` flags (`:1115,1496,1601,1789,1928,1984,2038,2089,2212`); `minikey` appears in a composite node-list **only** at `:151` inside `NODE_TYPES` (the `--from` list, `:1115`). The other 8 use `["xprv","phrase"]` / `SLIP39_FROM_NODES` / `MS_SHARES_FROM_NODES` / `PHRASE_ONLY`×4 / `SEEDQR_DECODE_FROM_NODES` — none carry `minikey`. The M1 "all 9 composites / over-warn" fold rests on a TRUE premise.
- `tests/persistence.rs:140-175`: confirmed a `for node in SECRET_NODE_TYPES { … assert !on_disk.contains(quoted) }` loop at the cited `:156`. The M2 "widen the loop to the wide set" fold is coherent with the actual file structure.
- `src/form/widget.rs:646`: confirmed bare `ui.text_edit_singleline(value)` in the composite arm (no paste detection) — the DECISION(ii) widget-wiring premise holds.
- Version sites: `Cargo.toml:3 = 0.44.0`, `README.md:42` self-tag `mnemonic-gui-v0.44.0`, `tests/paste_warn_wiring_v0_40_0.rs` EXISTS (cited kittest mirror). ✔

---

## Critical findings

**None.**

The funds-safety core is unchanged from round 1 (wide-set routing for argv-mask + run-confirm + persist-redact, narrow-vs-wide delta = exactly `{minikey}` a private key that MUST drop). The three folds are all *clarifications and test-hardening*; none re-opened a leak or altered the routing decisions. No Critical was introduced.

---

## Important findings

**None.** All three round-1 Important findings are verified RESOLVED below, and the folds introduced no new Important.

### I1 — RESOLVED (vacuous-pass hazard).

The fold is **binding, not an aside.** The H2 test section (spec `:95-124`) now:
- Marks the positive control **MANDATORY / load-bearing**: assertion 1 is labelled "(MANDATORY positive control — load-bearing, not an aside)" with "**Without this assertion the test is worthless**" (`:104-106`). It asserts the captured buffer contains the new `argv_len` field — the only discriminator between real-GREEN and captured-nothing.
- Adds a dedicated blockquote "**R0 I1 — vacuous-pass hazard (MANDATORY mitigation, not optional)**" (`:113-124`) that explains the GLOBAL callsite-interest-cache mechanism, cites the `cell_2` flake as proof, and enumerates 4 binding requirements: (1) scoped `with_default` subscriber (NOT global `try_init`), (2) **`tracing::callsite::rebuild_interest_cache()`** at scope entry, (3) MANDATORY positive control, (4) `#[serial]` fallback.
- Resolved-Q1 (`:415`) re-states the same mitigation bindingly ("These are binding on the implementer").

Verified against live source: the fold's claimed mechanism is exactly what `cell_2_tracing_init_logs_subprocess_spawn` (`runner_integration.rs:140`) documents and defends against in-tree (scoped `set_default` + `rebuild_interest_cache()` + retry). The round-1 reviewer's nuance — that the *real* test uses scoped `set_default`, so "global `try_init`" was a strawman — is correctly absorbed: the folded spec now says "scoped `with_default` … (NOT global `try_init`)". No drift. **RESOLVED.**

### I2 — RESOLVED (clipboard mis-description).

Surface-1 (spec `:165-176`) no longer claims the value is "masked in copy." It now states the mask "is redacted in the **Preview** display and the **last-run `argv:`** line … and the **copy button flips to the informed-reveal label**", then carries an explicit "**R0 I2 clarification (do NOT mis-implement)**" sentence: "the actual `ctx.copy_text` clipboard PAYLOAD stays **cleartext BY DESIGN** … the implementer/test MUST NOT attempt to redact the copy payload (that would break the intentional informed-reveal contract)". This matches the verified `main.rs` reality (masked Preview/last-run via `render_copy_command_masked`; cleartext clipboard via `render_copy_command` → `ctx.copy_text`; `any_secret`→reveal label). The H3 regression-test surface-1 assertion (`:294-296`) correspondingly asserts only "value token `mask = true` (masked in preview)" — it does NOT assert clipboard redaction, consistent with the clarification. **RESOLVED.**

### I3 — RESOLVED (open questions → decisions before code).

A grep of the entire doc for `open question` / `lean:` / `r0 may elect` / `TBD` / `undecided` / `leave it to the implementer` returns **only** the two lines in the "Resolved decisions" header/intro (`:411`, `:413`), both of which state the four are CLOSED in past tense ("ratified each lean", "no open question remains at code time"). The dedicated section (`:411-418`) records Q1–Q4 as binding RESOLVED decisions matching the I1/I2/I3 mandate:
- Q1 = scoped + `rebuild_interest_cache()` + mandatory positive control + `#[serial]` fallback (`:415`).
- Q2 = leave slot paste-warn OUT (`:416`).
- Q3 = new sibling mod for the wide-set compile-time guard (`:417`).
- Q4 = name `node_type_is_argv_secret` (`:418`).

All inline back-references were updated to point at the resolved decisions — verified no dangling "see Open Question Qn": the remaining references read "see **resolved** Q1" (`:124`), "**RESOLVED** (R0 round 1, Q2)" (`:254`), "**RESOLVED** (R0 round 1, Q3)" (`:288`), "**RESOLVED** at R0 round 1 (Q2)" (`:403`). No open question anywhere in the doc. **RESOLVED.**

---

## Minor findings

### M1 (round-2) — no action required; folds are accurate.

The two scope-correction folds I was asked to re-verify are both TRUE against live source:
- **(a) composite-paste-warn-parity for all 9 composites:** The new "Scope-correction (R0 round-1 Minor)" block (`:257-262`) correctly states wiring detection into `widget.rs:646` closes the slug for all 9 composites with the warn gated (via `node_type_is_argv_secret(node)`) to RAISE only for secret-class nodes. Verified: all 9 `NodeValueComposite` flags render through the single `widget.rs:646` dispatch, and `minikey` lives only in `NODE_TYPES` (the `--from` list), so the over-warn analysis at `:240-245` holds (non-secret nodes `xpub`/`fingerprint`/`path`/`mk1`/`address` do not warn). No over-warn.
- **(b) persistence-test widen:** The new H3 test point 3 (`:301-305`) widening the `tests/persistence.rs:156` loop to the wide set is coherent with the actual file — the secret-node loop exists at exactly that line and iterates `SECRET_NODE_TYPES` today; widening to `SECRET_NODE_TYPES_ARGV` adds the `minikey` on-disk-absence assertion as described.

### M2 (round-2) — no contradiction in earlier sections.

Re-checked the H3 surface-routing list (`:163-181`), the H2 DECISION (`:58-94`), the FOLLOWUP-slugs list (`:390-407`), and the SemVer/lockstep section (`:309-368`) against the folds. No fold contradicts an earlier section:
- The surface-routing list (4 surfaces) is consistent with DECISION(i)/(ii) and the H3 tests.
- The FOLLOWUP-slugs list correctly flips `composite-paste-warn-parity` → RESOLVED (consistent with the M1 fold's fuller-scope claim) and keeps `slot-field-paste-warn-uncovered` OPEN (consistent with resolved-Q2).
- Version sites unchanged and correct: GUI MINOR `0.44.0 → 0.45.0` (verified `Cargo.toml:3 = 0.44.0`); README self-tag bump via `readme_pin_coherence` (verified `README.md:42 = mnemonic-gui-v0.44.0`); no toolkit/pin bump; `schema_mirror`/`schema_mirror_secret_drift` not triggered (only a COMMENT changes at `schema:1119`, `--from` stays `secret:false`, `minikey` already a dropdown value); no manual leg; no `cargo fmt`. The pre-existing `pinned_version "mnemonic 0.59.0"` vs pin-tag `v0.60.0` skew is correctly fenced as out-of-scope (`:353-355`).

---

## Scope / lockstep confirmation (re-checked, unchanged from round 1)

- **GUI-only, no toolkit change, no pin bump** — confirmed; `SECRET_NODE_TYPES_ARGV` present at pinned `mnemonic-toolkit-v0.60.0`.
- **`schema_mirror` / `schema_mirror_secret_drift` NOT triggered** — confirmed (comment-only schema edit; no flag-name / dropdown-value / per-flag-secret-bool delta).
- **Manual mirror NOT triggered** — confirmed (GUI not in toolkit manual mirror set).
- **No `cargo fmt`** — confirmed (no GUI fmt CI gate).
- **Version sites complete** — `Cargo.toml:3`, `Cargo.lock:2266`, `README.md:42` self-tag, `CHANGELOG.md` entry; README toolkit/md/ms/mk pins unchanged.
- **No regression to round-1 clean facts** — the folds touched only test-method text (I1), surface-1 prose (I2), the resolved-Q section + back-references (I3), and two Minor scope notes (M1/M2). The funds-safety routing, the over-redaction analysis, and the citation set are untouched and remain accurate.

---

## Verdict

**R0 ROUND 2: 0C / 0I** — **GREEN (0C/0I).**

All three round-1 Important findings (I1 vacuous-pass test hazard, I2 clipboard mis-description, I3 unresolved Q1–Q4) are verified RESOLVED in binding spec text, each against live `0b1e024` source rather than the spec's self-assessment. The folds introduced no new Critical or Important and no contradiction with earlier sections; the two Minor scope-corrections (all-9-composites, persistence-loop widen) are factually accurate. The spec is cleared for implementation under the project's R0 gate.
