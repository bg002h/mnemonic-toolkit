# Post-impl fold convergence R0 — GUI FOLLOWUP-burndown batch — Fable, scoped

**Persisted verbatim per CLAUDE.md.** Scoped to the post-impl R0 (0C/1I + 2 Minor) folds only. VERDICT: **GREEN (0C/0I)**.

## I-1 (Important) — sibling Companion entries: RESOLVED

| Sibling | Slug exact? | Back-Companion? | Facts correct? | Right section? |
|---|---|---|---|---|
| `descriptor-mnemonic/design/FOLLOWUPS.md:2101` | YES | YES — `mnemonic-gui/FOLLOWUPS.md` (primary) + ms + mk | YES — "5 `md` dropdown flags"; "6 of the 7 cross-repo backfills; the 7th is `ms encode --language`"; lists exactly `md encode --network`, `md verify --network`, `md address --network`, `md address --chain`, `md address --index`, `md address --count` | YES* — file-end append (file's established convention: 25+ entries appended after `## Convention notes`); outside any fenced block |
| `mnemonic-secret/design/FOLLOWUPS.md:60` | YES | YES — gui (primary) + md + mk | YES — "8 `ms` dropdown flags"; the 1 backfill `ms encode --language` (`[default: english]`) | YES — first entry under `## Open items` |
| `mnemonic-key/design/FOLLOWUPS.md:64` | YES | YES — gui (primary) + md + ms | YES — "3 `mk` dropdown flags"; "0 day-one backfill delta" | YES — first entry under `## Open items` |

**Cross-file consistency:** md 5 + ms 8 + mk 3 = 16, matching the GUI entry. Backfill split 6 md + 1 ms = 7 identical in all four files; GUI's 7-item list = union of md's 6 + ms's 1. "0/35" denominator consistently attributed to `md`. **I-1 RESOLVED.**

## M-1 (Minor) — phrasing: PRESENT AND CORRECT
`mnemonic-gui/FOLLOWUPS.md:12` now reads "…OMIT the `default_value` key entirely — 0/35 flags carry it on `md`, likewise `ms`/`mk`; serde `#[serde(default)]` … resolves the absent key to `None`". Old `carry "default_value": null` phrasing: zero hits. Accurate (absent key, not null).

## M-2 (Minor) — resolver doc-comment: PRESENT AND CORRECT
`tests/schema_mirror_defaults_drift.rs:108-116`: "Adapted from `tests/schema_mirror.rs:47 resolve_bin`" + NOTE that this simplified copy omits the `.replace('-', '_')` normalization (no-op for md/ms/mk). Verified against the mirror (`schema_mirror.rs:47` does carry `.replace('-', "_")`). Function body unchanged (comment-only).

## No-drift: CONFIRMED
- `mnemonic-gui` `git status --short`: exactly the original 3 files; no new files, zero `src/`, NO-BUMP holds.
- `cargo check --tests` in mnemonic-gui: clean (3.3s).
- ms/mk sibling diffs: exactly the 6-line S2 entry each, docs-only, head of `## Open items`.
- md sibling diff = the 6-line S2 entry + the co-resident 8-line `md-cli-non-chunked-single-string-repair-demote` entry from the separate toolkit-v0.86.0 cycle. NOT fold drift — staging-hygiene observation: committing descriptor-mnemonic `design/FOLLOWUPS.md` carries both; either split hunks or acknowledge both in the commit message.

**VERDICT: GREEN (0C/0I)** — all three folds complete and correct, no drift. One non-blocking staging-hygiene observation (handled: the coordinated ship commits descriptor-mnemonic once, acknowledging both cycles' entries in the message).
