# SPEC — delete the stale cli-help `--help` scaffolding snapshots (cli-help-golden-broad-staleness-not-gated)

**FOLLOWUP:** `cli-help-golden-broad-staleness-not-gated`.
**Source SHA (origin/master at write time):** `d509361`.
**Cycle type:** docs/test-fixture cleanup. **NO toolkit version bump, NO tag** — precedent: the manual anchor-dangler cleanup (`dd7c228`) + `sibling-pin-check.yml` (`9f72d2f`) were docs/CI-only changes shipped by a plain commit to `master`. The manual carries its own `manual-vX.Y.Z` versioning; this touches unrendered transcript fixtures, not any user-facing toolkit artifact.
**Recon:** `cycle-prep-recon` performed inline (see §2). **User decision (2026-06-06):** option (c) — delete + update note.
**Locksteps:** manual lint/audit must stay GREEN. **No** GUI `schema_mirror`, **no** sibling-codec, **no** CLI surface change.

---

## 1. Problem

`docs/manual/transcripts/cli-help/*.txt` holds **21** `--help` text snapshots (9 `md-*`, 6 `mnemonic-*`, 6 `ms-*`; no `mk-*`). The FOLLOWUP reports they drift silently. Recon shows the situation is worse and the fix is different from the FOLLOWUP's preferred "gate them":

## 2. Recon findings (decisive)

- **Authoring scaffolding, not a rendered artifact.** Created in `db183f3` ("Phase 4 — eight workflow chapters + CLI help captures") per `design/IMPLEMENTATION_PLAN_user_manual_v0_1.md:284` ("Generate `--help` snapshots into `transcripts/cli-help/` and turn them into the four CLI-reference chapters"). They were the source material for hand-writing the `40-cli-reference/` chapters.
- **Pinned to v0.8.0, ~40 versions stale.** `docs/manual/src/60-appendices/67-troubleshooting.md:60-61` states "The cli-help snapshots in this manual match toolkit v0.8.0; later versions may have added flags." Measured drift vs live `--help`: 22–96 changed lines per file (e.g. `mnemonic-convert.txt` 96, `mnemonic-derive-child.txt` 74, `mnemonic-export-wallet.txt` 58).
- **Not rendered anywhere.** No `{{#include}}` of `cli-help/` in any `src/*.md`, `SUMMARY`, or `book.toml`. The ONLY live reference in the rendered manual is the troubleshooting prose at `67-troubleshooting.md:60`.
- **Not gated, and never were.** No `.cmd` files exist in `cli-help/`, so the `verify-examples.sh:67` `-not -path '*/cli-help/*'` exclusion is moot (the `.cmd` discovery would find nothing there anyway).
- **Superseded + already gated elsewhere.** The hand-written `40-cli-reference/41-mnemonic.md` (and `42-md`/`43-ms`/`44-mk`) chapters ARE the rendered CLI-surface mirror, and their flag-NAME parity is gated by the `flag-coverage` lint (`docs/manual/tests/lint.sh`) which runs live `--help` against `cli-subcommands.list` (per `CLAUDE.md` "manual chapters mirror clap-derive's `--help` output … bidirectional flag-coverage check").
- **Therefore "gate the snapshots" (FOLLOWUP option b) is redundant + brittle:** it would duplicate the flag-coverage gate's purpose and additionally fire on every help-string *wording* reword. Live `--help` is the authoritative source.

## 3. Decision (user-approved: option c — delete)

Delete the 21 unrendered, stale, superseded scaffolding snapshots. You cannot have silent golden drift if there is no golden. Flag-NAME parity remains gated by `flag-coverage`; the live `--help` is authoritative.

## 4. Changes

### 4a. Delete (21 files)
`docs/manual/transcripts/cli-help/*.txt` — the entire `cli-help/` snapshot set (`md-{address,bytecode,compile,decode,encode,inspect,vectors,verify}.txt`, `md.txt`, `mnemonic-{bundle,convert,derive-child,export-wallet,verify-bundle}.txt`, `mnemonic.txt`, `ms-{decode,encode,inspect,vectors,verify}.txt`, `ms.txt`). The directory becomes empty → remove it.

### 4b. Reword the troubleshooting note(s) — TWO rendered docs (R0 I1)
Both rendered troubleshooting chapters carry the same stale "match toolkit v0.8.0" claim:
- `docs/manual/src/60-appendices/67-troubleshooting.md:60-61`
- `docs/quickstart/src/50-next-steps/52-troubleshooting.md:113-114` (**R0 I1 — separate rendered mdBook; the SPEC originally missed it**)

Reword BOTH (point at the live source; no stale-snapshot claim). **(R0-r2 M-new) the wording is ADAPTED per book, not byte-identical** — "chapter 40" is a manual section that does NOT exist in the quickstart book.
- **Manual** (`67-troubleshooting.md:60-61`) → "**Run `--help` directly.** Each CLI's `--help` (`mnemonic`/`md`/`ms`/`mk`, and any subcommand) is authoritative for the installed version; the CLI-reference chapters (chapter 40) mirror the current flag surface."
- **Quickstart** (`52-troubleshooting.md:113-114`, no chapter-40) → "**Run `--help` directly.** Each CLI's `--help` (`mnemonic`/`md`/`ms`/`mk`, and any subcommand) is authoritative for the installed version." (No chapter-40 clause; the existing Appendix-G cross-link two lines below already routes to the manual. Preserve the 3-space ordered-list continuation indent.)

### 4c. Tombstone the now-moot verify-examples exclusion (`verify-examples.sh`)
The `cli-help/` dir is gone, so the `-not -path '*/cli-help/*'` predicate + its two comment blocks (`:24-26`, `:65-66`) reference a nonexistent dir. Reword the comments to a one-line tombstone so a future reader isn't confused, and **keep the predicate** (R0 ratified tombstone-and-keep over full removal). **(R0 M3)** the tombstone comment MUST state the rationale explicitly so a future contributor doesn't delete it as dead code, e.g.: "`cli-help/` snapshot dir was removed (FOLLOWUP `cli-help-golden-broad-staleness-not-gated`); live `--help` is authoritative. Predicate kept as a guard: if a `cli-help/` dir is ever re-introduced with `.cmd` files but not wired as a real `.cmd`→`.out` transcript pair, this keeps the runner from mis-discovering them."

### 4d. Leave frozen history untouched
`design/agent-reports/phase-{4,6}-review-1.md`, `design/PLAN_*`, `design/IMPLEMENTATION_PLAN_user_manual_v0_1.md` reference `cli-help/` as historical audit records — do NOT edit (they record what was true at the time).

## 5. Verification (no RED phase — this is a deletion)
- **(R0 M1, widened scope)** `git grep cli-help -- docs/` returns ONLY: the two reworded troubleshooting notes (manual + quickstart), the tombstoned `verify-examples.sh`, and frozen `*/agent-reports/*` + `docs/superpowers/plans/*` history (leave those). No dangling reference into any rendered chapter/SUMMARY/book.toml.
- `make -C docs/manual audit` GREEN with all 4 bins pinned (lint + verify-examples + anchor-check). verify-examples count unchanged (cli-help had no `.cmd`, so the transcript count was never affected). anchor-check baseline unchanged (no `{#id}` lived in those `.txt`).
- `make -C docs/manual html` builds clean (no broken `{{#include}}`).

## 6. Phasing
- **Phase 1 (implement):** 4a delete + 4b reword (BOTH manual + quickstart notes) + 4c tombstone. Run §5 verification (incl. the manual audit + a quickstart build/lint if it has one — confirm both rendered docs still build clean).
- **Phase 2 (review + ship):** per-phase opus review → fold to 0C/0I → flip FOLLOWUP `cli-help-golden-broad-staleness-not-gated` → resolved → ff-merge to `master` → push → watch CI (`manual` fires on the manual change; `rust` + pin checks are no-ops for content). **No tag, no version bump.**

## 7. R0 decisions (RATIFIED round 1)
1. **No version bump / no tag** for an unrendered-fixture deletion. ✅ R0-ratified (anchor-dangler `dd7c228` precedent; "strictly unrendered transcript fixture deletion + prose reword in `docs/`").
2. **§4c tombstone-and-keep** the `-not -path '*/cli-help/*'` predicate (with explicit rationale comment, R0 M3). ✅ R0-ratified over full removal (4-word re-introduction guard).
3. **Nothing renders the snapshots.** ✅ R0 independently confirmed (no `{{#include}}`, no SUMMARY entry, no Makefile copy step; `flag-coverage` uses live `--help` not the snapshots).
4. **mk has no cli-help snapshot** — pre-existing (never captured); NOT extended this cycle. ✅ R0 confirmed.

## 8. Out of scope
- Regenerating or gating any `--help` snapshot (rejected: redundant with flag-coverage).
- The hand-written `40-cli-reference/` chapters (the live, gated mirror — unchanged).
- Adding an `mk-*` snapshot.
