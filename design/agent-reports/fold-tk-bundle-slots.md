# Fold: toolkit#163 review I1 + I2 (bundle slot docs, exact texts)

Model: Claude Haiku 5.5 (claude-haiku-5-5). Brief: `briefs/fold-tk-bundle-slots.md`. Fix texts: `briefs/fold-tk-bundle-slots.fixtexts`.

Counts: C / I / M / N: 0 / 0 / 0 / 0 (no new findings; the fold applies review items I1 and I2 only).

## Status: STOPPED at step 2 (FOLLOWUPS), not finished

### Step 1 (done): commit `b8c249a8`
"Fold: review I1 + I2 (bundle slot docs, exact texts)"
- All five `@@ old` texts were found exactly once (checked by exact-count script before writing).
- Files changed (3): `docs/technical-manual/src/40-bundle-formation/42-anti-collision-invariants.md`, `crates/mnemonic-toolkit/src/synthesize.rs` (3 edits), `docs/technical-manual/src/60-back-matter/61-glossary.md`.
- Nothing else in those files changed. `design/FOLLOWUPS.md` was not touched.

### Step 3 (run, exit 0): gate
`export PATH=$HOME/.cargo/bin:$PATH && cargo +1.95.0 fmt --check && CARGO_BUILD_JOBS=8 cargo clippy --locked --all-targets -- -D warnings && make -C docs/technical-manual lint`
Gate result: exit=0; `[lint] OK`; `Summary: 0 error(s)`. Log: `/scratch/claude-1000-gate.log`.

### Step 2 (NOT done): FOLLOWUPS entries, blocked by open questions
The brief does not give these exactly, so I did not choose. Please decide and I will append the entries as a second commit.

1. **Ids for N1, N2, M1 are missing.** The brief gives the text and owner for each item but no `###` id. Existing ids follow the form `` `kebab-case-id` ``. Please give three ids, for example `bundle-slot-docs-n1-...`.
2. **Status for the GUI entry conflicts.** The brief says each new entry has Status `deferred-minor`. The verbatim GUI entry in the review report says `Status: \`deferred\` (gui parked)`. Which one should the GUI entry use?
3. **Surfaced line for the GUI entry conflicts.** The brief sets Surfaced to `2026-11-14, toolkit#163 review (\`design/agent-reports/review-tk-bundle-slots-docs.md\`)`. The verbatim GUI entry says `toolkit PR #163 review (review-tk-bundle-slots-docs)`. Which one?
4. **Entry form.** Open items use two forms. The newest entries use bold labels (`- **Surfaced:**`, `- **Status:**`, `- **Tier:**`). Older ones use plain labels (`- Surfaced:`, `- Status:`, `- Tier:`). Which form should the new entries use?

No FOLLOWUPS commit exists yet, so the brief's rule "same commit as step 1 is NOT allowed" is not at issue.

### Commits on `docs/bundle-slots-0-31`
- `b8c249a8` fold (step 1)
- this report commit (below), not pushed.

Nothing was pushed, merged or tagged. No `pkill`, no `git commit -a`, no stash use.
