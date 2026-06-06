# R0 Architect Review — cli-help-golden-cleanup — Round 1

> Persisted verbatim from the opus `feature-dev:code-architect` agent
> (`agentId: a525c12c8f4ed856e`). Had Read/Glob/Grep; verified against source.

---

**VERDICT: 0 Critical / 1 Important / 3 Minor**

## Critical findings: NONE

All three safety-critical claims check out:

**Claim 1 verified — no `{{#include}}` renders cli-help/.** Zero `{{#include}}` directives exist anywhere in `docs/manual/src/`. The Makefile's `ALL_SRC`/`MD_SRC`/`PDF_SRC` use `find $(SRC_DIR) -type f -name '*.md'` — only `src/` `.md`, never `transcripts/`. `verify-examples` runs `.cmd`-discovery; no `.cmd` in `cli-help/`. Deletion cannot break any build.

**Claim 2 verified — flag-coverage gates live `--help`, not the snapshots.** `docs/manual/tests/lint.sh:84` runs `eval $cmd 2>&1 | grep -oE -- '--[a-z][a-z0-9-]+'` where `$cmd` is the live binary invocation. Never touches `transcripts/cli-help/`. Deleting the snapshots leaves the gate intact.

**Claim 3 verified — verify-examples count unaffected.** `verify-examples.sh:67` discovers `.cmd` with `find … -name '*.cmd' -not -path '*/cli-help/*'`. `cli-help/` has only `.txt`. The exclusion has always been moot. Count does not change.

## Important (1)

**I1 — The quickstart has a second stale note that the SPEC misses.** `docs/quickstart/src/50-next-steps/52-troubleshooting.md:113-114` contains an identical stale sentence: "The cli-help snapshots in the manual match toolkit v0.8.0; later versions may have added flags." The SPEC's §4b reword covers only `docs/manual/src/60-appendices/67-troubleshooting.md:60`. The quickstart is a separate rendered document (`docs/quickstart/`), so after deletion it becomes a dangling claim. The SPEC §5 verification grep (`docs/manual/src docs/manual/tests`) is too narrow — it excludes `docs/quickstart/`.
**Fix:** Extend §4b to also reword `docs/quickstart/src/50-next-steps/52-troubleshooting.md:113-114` with parallel language; widen the §5 verification grep to `docs/`.

## Minor (3)

**M1 — §5 verification grep scope too narrow.** Misses `docs/quickstart/` (I1) and `docs/superpowers/` (frozen plan docs — leave). Widening to `docs/` resolves it.

**M2 — §7 item 4 (mk has no cli-help snapshot).** Confirmed: 21 files, none `mk-*.txt`. Verified; no action.

**M3 — §4c tombstone wording.** Keeping the `-not -path '*/cli-help/*'` predicate is sound, but the tombstone comment should explicitly state the rationale ("predicate kept as guard against re-introducing a `cli-help/` dir without a `.cmd` → `.out` transcript pair") so a future contributor doesn't delete it as dead code.

## Verified clean
1. Nothing renders cli-help/ — no `{{#include}}`, no SUMMARY, no Makefile copy step. Safe.
2. flag-coverage uses live `--help` (`lint.sh:84`). Snapshots not consumed.
3. `git grep cli-help` classification: (i) 21 `.txt` deleted; (ii) `design/` frozen history leave; (iii) `67-troubleshooting.md:60` reworded; (iv) `verify-examples.sh:25,65,67` tombstoned; (v) **live consumer missed = `docs/quickstart/src/50-next-steps/52-troubleshooting.md:113`** (I1); `docs/superpowers/plans/*` frozen.
4. verify-examples count / anchor-check baseline unchanged (no `.cmd`, no `{#id}`).
5. SemVer/no-bump: confirmed correct (unrendered fixture deletion + prose reword in `docs/`; precedent `dd7c228`). Plain commit to master is right.
6. §4c tombstone-and-keep is the better choice (4-word re-introduction guard / forcing function) vs full removal.
7. Offline-reference value: the snapshots are ~40 versions stale with negative information value; deletion loses nothing — the `40-cli-reference/` chapters are the accurate reference.

**VERDICT: 0 Critical / 1 Important / 3 Minor — NOT YET GREEN. Fold I1 (quickstart note + widened verification grep), then GREEN for implementation.**
