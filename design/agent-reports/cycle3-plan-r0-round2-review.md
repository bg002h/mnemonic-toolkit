# cycle-3 PLAN R0 — round 2 — opus-architect adversarial fold-verification

**Date:** 2026-06-21
**Artifact under review:** `design/IMPLEMENTATION_PLAN_cycle3_gui_secret_leaks.md` (folded after round 1)
**Implements (R0-GREEN spec):** `design/BRAINSTORM_cycle3_gui_secret_leaks.md` (0C/0I @ round 2)
**Round-1 review:** `design/agent-reports/cycle3-plan-r0-round1-review.md` (0C/1I → RED; I1 + M1–M5)
**Gate:** HARD R0 — no code until 0 Critical / 0 Important.

## Repo SHAs (verified live this review)

| Repo | `origin/master` | Role |
|---|---|---|
| **mnemonic-gui** (fix repo) | `0b1e024eab67638da844a52f09d6964c4f55c8df` | all folded citations re-verified via `git show origin/master:<path>`; crate **0.44.0** |
| mnemonic-toolkit (dep) | `c9168aac` | exposes `SECRET_NODE_TYPES_ARGV` at tag `mnemonic-toolkit-v0.60.0`; NO change |

**Verification method:** re-fetched the live GUI `origin/master` for every fold-touched
artifact (`tests/runner_integration.rs`, `Cargo.toml`, `src/runner.rs`,
`src/schema/mnemonic.rs:2090-2098`, root `FOLLOWUPS.md`) and re-derived the RED/GREEN
state transitions for the rewritten Phase-1 test by hand against the current `:119`
macro and the planned GREEN edit. Findings cite live line numbers.

---

## Fold-verification ledger

### I1 (H2 test harness) — **RESOLVED** (substantively); one residual Minor (M6) spun off

Round-1 I1 required: adopt the PROVEN `cell_2_tracing_init_logs_subprocess_spawn`
harness (`CapturedWriter` + `set_default` + `rebuild_interest_cache()` + 3-attempt
retry); NO `serial_test`/`#[serial]`; the negative assertion runs ONLY on an attempt a
positive control confirms captured (never false-GREEN). Verified against the live
`tests/runner_integration.rs` (re-fetched in full):

- `CapturedWriter(Arc<Mutex<Vec<u8>>>)` MakeWriter, `mnemonic_bin()`, and
  `runner::run` are all present at `origin/master` and reusable **exactly as the plan
  reuses them** ✔. `serial_test` is **absent** from `Cargo.toml` (only `tracing` +
  `tracing-subscriber` with `env-filter`) — confirming the round-1 "unprovisioned dep"
  claim and the plan's removal of it ✔.
- The folded Phase-1 test (plan `:42-64`) is the `cell_2` shape verbatim: fresh
  subscriber + `set_default` guard + `rebuild_interest_cache()` per attempt, bounded
  3-attempt loop, terminal `panic!` on exhaustion ✔.
- **No false-GREEN possible.** The negative `assert!(!captured.contains("SENTINEL_SEED"))`
  is nested INSIDE `if captured.contains("argv_len")`; an empty/poisoned capture skips
  the assert, retries, and on exhaustion PANICS — it can never silently pass (plan
  `:65-68`) ✔. This is the precise property round-1 I1 demanded.
- **RED→GREEN transition holds.** Current `:119` (`argv = ?argv`) → the GREEN-only
  token `argv_len` is never emitted → loop exhausts → terminal `panic!` → **test is RED**
  on current code ✔. After the GREEN edit (`argv_len = argv.len()` field emitted, plan
  `:78`) → `argv_len` present on a captured attempt → sentinel gone → negative assert
  passes → `return` → **GREEN** ✔.
- **No contradiction with the GREEN `program=%argv[0]` + `argv_len` edit:** the positive
  control asserts `argv_len`, which the GREEN edit emits — consistent ✔. `program =
  %argv[0]` is provably never-secret (it is `Command::new(&argv[0])`) ✔.

I1's blocking substance (weaker-than-proven harness + unprovisioned dep + false-GREEN
risk) is fully resolved. A correctness-of-prose nuance survives → **M6 (Minor)** below.

### M1 (second stale comment) — **RESOLVED**

Plan `:181-186` now fixes BOTH `schema/mnemonic.rs:1119` AND `:2095`. Confirmed live:
`git show origin/master:src/schema/mnemonic.rs | sed -n '2090,2098p'` shows the
`seedqr decode --from` composite carries `secret: false, // value-dependent; per-row
paste-warn fires` — the SAME false claim as `:1119` ✔. The plan replaces both with the
accurate `// secrecy is node-dependent; …node_type_is_argv_secret (cycle-3)` and keeps
`secret: false` at both sites (no `schema_mirror`/`schema_mirror_secret_drift` delta) ✔.

### M2 (vacuous minikey persistence coverage) — **RESOLVED**

Plan `:101` (regression test 2) is now the AUTHORITATIVE minikey proof: it "MUST
construct a form state that actually contains a `--from minikey=<KEY>` composite" and
assert (iv) `redact_for_persistence` DROPS the composite AND the serialized
`state.json` bytes lack the minikey value ✔. Plan `:102` (item 3) demotes the
`tests/persistence.rs:156` loop-widening to "defense-in-depth, NOT the minikey proof,"
explicitly noting the `mixed_form_with_secrets()` fixture has only `phrase`/`xpub`
composites so the loop is vacuous for minikey on its own ✔. Exactly the round-1 M2 ask.

### M3 (FOLLOWUP file-vs-flip) — **RESOLVED**

Plan `:213-219` now (a) names the GUI registry as the **repo-ROOT `FOLLOWUPS.md`** —
confirmed live (root `FOLLOWUPS.md` exists; no `design/FOLLOWUPS.md`) ✔; (b)
**FILE-NEW (RESOLVED)** for the two bughunt-id slugs `gui-runner-debug-logs-unmasked-secret-argv`
(H2) and `gui-minikey-secret-not-masked-in-argv-preview` (H3, folding
`w3-gui-minikey-persist-plaintext`) — confirmed live these are NOT present in root
`FOLLOWUPS.md` ✔; (c) **FLIP existing** `composite-paste-warn-parity` (`:40`) → RESOLVED
and note `slot-field-paste-warn-uncovered` (`:48`) STAYS OPEN — both confirmed present
at those lines ✔. The "file-and-resolve" vs "flip" distinction round-1 M3 wanted is now
explicit.

### M4 (style — `node.as_str()`) — **RESOLVED**

Plan `:167` paste gate now reads `node_type_is_argv_secret(node.as_str())` with an
inline `// plan-R0 M4: explicit .as_str()` note ✔.

### M5 (egui role) — **RESOLVED**

Plan `:144-149` now mandates the Phase-3 paste kittest query/focus the composite value
field by `Role::TextInput` (or by label), NOT `Role::PasswordInput`, and explains why
(the composite field is a non-password `text_edit_singleline`) and the vacuous-pass
trap of copying the `paste_warn_wiring_v0_40_0` model verbatim ✔.

---

## New-drift / regression checks on the folds (all PASS)

- **File-disjointness still coherent.** The Phase-1 test now lives in
  `tests/runner_integration.rs` (plan `:32`). Re-checked Phase-2 (`:242`) and Phase-3
  (`:243`) file lists: **neither touches `tests/runner_integration.rs`** ✔. The
  disjointness table's `src/runner.rs (+ test)` is slightly imprecise (does not name
  the test file) but is coherent — the file is owned solely by Phase 1; no
  cross-phase contention introduced ✔.
- **No leftover rejected-scheme prescription.** Grep of the plan for
  `with_default` / `#[serial]` / `serial_test` returns only the round-1 explanatory
  text at `:34-35` framing them as "the naive … scheme [that] is insufficient and
  uncompilable" — i.e. a description of the REJECTED approach, not a prescription ✔.
- **No contradiction between the folded test and the GREEN edit** (see I1 ledger:
  positive control = `argv_len`, which the GREEN edit emits) ✔.
- M1/M2/M3/M4/M5 folds introduced no schema/manual/pin-gate drift: comment-only schema
  edits, no flag-name/dropdown/`secret`-bool change → `schema_mirror` ×4 +
  `schema_mirror_secret_drift` stay green ✔.

---

## Critical

**None.**

## Important

**None.** The single round-1 blocking finding (I1) is resolved: the plan adopts the
proven retry-loop harness verbatim, drops the unprovisioned `serial_test`, and the
positive-control gate makes a false-GREEN structurally impossible. The RED→GREEN
transition is real on current code.

## Minor

### M6 (NEW, spun off from the I1 fold) — positive-control token `argv_len` is GREEN-only, so the plan's stated RED *mechanism* is inaccurate (non-blocking)

The folded test gates the negative assertion on `captured.contains("argv_len")`.
`argv_len` is a **GREEN-only** token — it does not exist in the RED `argv = ?argv` line.
The proven sibling `cell_2` instead gates on the event MESSAGE `"subprocess spawn"`,
which is present in BOTH the RED and GREEN states (the GREEN edit keeps that message).
Consequence:

- On **current (RED) code**, `argv_len` is never present → the loop never enters the
  `if` block → the negative `assert!` is **never reached** → RED is produced by the
  terminal `panic!("could not capture runner spawn debug event after 3 attempts")`.
- But plan `:69-70` justifies RED as "`assert!(!contains)` fails on the first captured
  attempt." That is **factually wrong** for the code as written: on RED that assert is
  never executed, and the diagnostic the implementer sees is the misleading "could not
  capture the spawn event" panic — when in fact the event WAS captured; it merely
  lacked the `argv_len` field.

This is **non-blocking**: the RED→GREEN transition is real and a false-GREEN remains
structurally impossible (the `if argv_len` gate still guards the assert). It is a Minor
because (1) the plan's own RED-justification self-contradicts the code, and (2) the RED
diagnostic is non-diagnostic (it blames capture, not the leak). Recommended fix —
gate the loop on the always-present message `"subprocess spawn"` instead of `argv_len`
(matching `cell_2` exactly). Then the negative `assert!(!contains("SENTINEL_SEED"))`
actually FIRES on RED and emits the accurate `"argv leaked to debug log: …"` message
the plan's prose already claims, while GREEN still passes (sentinel gone, message
present). `argv_len` can stay as a *secondary* GREEN-shape assertion if desired, but it
should not be the capture gate. Does not block R0 (Minor); fold at implementation time.

---

## Verdict

All five round-1 findings the author folded (I1, M1, M2, M3, M4, M5) are verified
RESOLVED against the live `origin/master` trees. The blocking I1 is substantively and
correctly resolved — the proven `cell_2` retry-loop harness is adopted verbatim,
`serial_test` is gone, and the positive-control gate makes a false-GREEN impossible.
The folds introduced no new Critical or Important drift; file-disjointness remains
coherent and no rejected-scheme prescription survives. One new **Minor (M6)** was spun
off from the I1 fold (the `argv_len` capture gate is GREEN-only, making the plan's
stated RED *mechanism* inaccurate and the RED diagnostic misleading) — non-blocking,
fold at implementation time.

**PLAN R0 ROUND 2: 0C / 0I**  → **GREEN (0C/0I)**
