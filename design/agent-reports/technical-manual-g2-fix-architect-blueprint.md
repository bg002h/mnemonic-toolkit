# Architect blueprint — fix for `technical-manual-g2-uncovered-in-bare-ci`

> Design consult (pre-SPEC), opus architect, grounded at HEAD `cdc0479`. Verbatim record; the SPEC `design/SPEC_technical_manual_g2_bare_ci_fix.md` is built from this. Decisions (A)/(B)/(C) below.

## Design key (validated)

Bare CI (toolkit-only checkout) **can** fully enforce TOOLKIT-G2 (toolkit source IS the repo) but **cannot** enforce CODEC-G2 (needs the deliberately-absent siblings). Principled target: **100% toolkit-G2 in bare CI; codec-G2 stays local-only (documented accepted residual).** The only encoding of "toolkit intent" for a non-authoritative chapter is **full `crates/mnemonic-toolkit/src/...` qualification**, which routes through `resolve()`'s `crates/` early-branch that checks the present toolkit repo and FAILs (`unresolved`) when the file is absent post-rename.

## LOAD-BEARING SHARPENING

**Subpaths are NOT rename-safe.** Only the full `crates/mnemonic-toolkit/src/...` form satisfies `is_repo_qualified` and routes through the `crates/` early-branch. A *subpath* (`cmd/bundle.rs`, `wallet_export/mod.rs`) fails `is_repo_qualified` → suffix-match → resolves TODAY but **post-rename suffix-matches nothing → `skip:absent-sibling`** (silently not failed in bare CI). So a subpath is exactly as rename-unsafe as a bare basename. Canonical fix form for all 12 = full `crates/mnemonic-toolkit/src/<subpath>` (NOT the double-prefix `mnemonic-toolkit/crates/...` variant).

## Recommended fix (what changes)

1. **Workflow trigger** (`.github/workflows/technical-manual.yml`): add to BOTH `push.paths` and `pull_request.paths`:
   `- 'crates/mnemonic-toolkit/src/**'` and `- 'crates/mnemonic-toolkit/tests/**'`. Keep full `make lint`. Update the top comment to state it now fires on toolkit-source changes (so toolkit-symbol renames re-run symbol-ref-check) and remains bare-CI (codec refs still skip).
2. **Gate guard** (`symbol-ref-check.py`): inside the existing `if not qualified:` block, after the collision return, add `if not auth and repos_with == ["toolkit"]: return (None, "unqualified-toolkit")`. Add a scan-loop `err()` arm mirroring the `collision` arm: "non-authoritative chapter cites a toolkit file by bare/subpath; fully qualify as crates/mnemonic-toolkit/src/... (only that form is rename-safe)".
3. **Qualify the 12 bare/subpath toolkit refs in `61-glossary.md`** to full `crates/mnemonic-toolkit/src/...` (per-TOKEN; lines 229/385/433 mix an already-qualified token with a bare one — qualify only the bare).
4. **AUTHORING.md**: generalize the "Colliding basenames" rule (non-auth chapter → ANY toolkit-resolving ref, colliding or not, bare or subpath, must be full-qualified, because only that form is rename-safe; lint FAILs unqualified toolkit refs in non-auth chapters); fix the now-STALE "The technical manual has no CI workflow" claim (a lint-only workflow exists; `make lint` with all siblings present remains the FULL gate — only it enforces codec-G2).
5. **FOLLOWUPS flip** to resolved; record gap(2) via trigger, gap(1) part(ii) via qualify+guard, gap(1) part(i) codec-G2 accepted residual.

OUT of scope: codec-G2 in bare CI (part i), multi-repo checkout, basename/crates.io manifest, the 40 qualified + 268 auth refs (unchanged), the other 6 lint checks.

## Decisions

**(A) Trigger — ADD `crates/mnemonic-toolkit/{src,tests}/**`, keep full `make lint`.** Leaving gap(2) soft would defeat the qualify-now work (a rename wouldn't fire the lint → the 12 + 268 only FAIL on the next docs edit). Keep simple full `make lint` not a split symbol-only job: the other 6 checks scan docs text only → on a code-only diff they re-run over unchanged text and CANNOT newly fail (red only if master already red), so the full lint is functionally symbol-ref-check-only on code PRs; a paths-filter split adds disproportionate complexity. Include `tests/**` (INDEX walks tests/; `tests::` anchors are a documented form). Cost — a code PR renaming a cited symbol must carry the docs edit in-PR — is the intended symbol-level lockstep, consistent with the repo's manual-mirror/schema-mirror culture. The run stays bare-CI (toolkit-only): enforces toolkit-G2 on toolkit renames; codec refs stay skipped.

**(B) Guard — hard FAIL, in `resolve()`, predicate `not auth and repos_with == ["toolkit"]`.** WARN re-creates the lagging-indicator failure mode CLAUDE.md already burned on. Traced against all four exclusions: fires in BOTH modes (toolkit present in both → `repos_with==["toolkit"]`); doesn't touch auth chapters (`not auth` gate; the 268 resolve via the `auth and not qualified` branch first); doesn't touch bare codec refs (bare CI: codec absent → `repos_with==[]` → false → `skip`; local: `repos_with==["md"]` → false → resolves & checks); doesn't break the 40 qualified (`is_repo_qualified` → skip the block) or 268 auth (`not auth` False). Catches the ambiguous-within-toolkit case (bare `electrum.rs`, 3 toolkit paths → `["toolkit"]`). **Two-part mechanism confirmed:** a *renamed* bare ref resolves to nothing → `repos_with==[]` → guard does NOT fire → skip; so the guard alone can't protect across a rename → the 12 must be **qualified now** (route through `crates/` → FAIL on rename); the **guard prevents new** bare/subpath toolkit refs (at commit the file exists → `["toolkit"]` → FAIL → author qualifies → rename-safe). Irreducible residual: a bare ref to a not-yet-existing toolkit symbol (typo/future) → `skip`, indistinguishable from a codec ref — name it, don't chase it.

**(C) Codec-G2 — accept as documented residual.** A basename manifest proves file-existence not symbol-existence (no true G2) and drifts on every codec rename (regen tax). A crates.io source-download couples docs CI to *published* codec versions that lag the dev symbols the manual cites vs `origin/master` → false results (the very drift-coupling the multi-repo checkout was rejected for). Both buy only "catch a codec-filename typo in bare CI" — already caught by local `make lint`. Net negative. Document in 3 places (workflow comment, AUTHORING, FOLLOWUPS) pointing to local `make lint` (all siblings present) as the full codec-G2 gate.

## RISKS the SPEC must address

1. **Re-run the REAL gate as acceptance (toolkit-only mode), do not trust "exactly 12".** The recon probe used different code than the gate; after qualifying, run `symbol-ref-check.py` siblings-absent over all `src/` and assert GREEN — if any other non-auth chapter harbors a toolkit-resolving bare/subpath ref the probe under-counted, the new guard will RED it; surface that pre-merge.
2. `tests/**` + non-`src`/`tests` paths: no `build.rs`/`examples/` toolkit citations exist today; INDEX walks src/+tests/ only; a citation outside those would skip (no INDEX row) — none exist.
3. **Collision-vs-guard message nondeterminism:** if a basename also exists in a CLI/codec crate, LOCAL reports `collision`, bare CI reports `unqualified-toolkit` — same outcome, different message. Run the guard test in **toolkit-only mode** for deterministic messages; do NOT pin the local-mode message.
4. **Per-TOKEN, not per-line** (229/373/385/433 mix qualified+bare): qualify each bare token, leave the already-qualified one.
5. `wallet_export/mod.rs` path-suffix: `mod.rs` is collision-prone bare, but the glossary uses the `wallet_export/` subpath → promote to full `crates/mnemonic-toolkit/src/wallet_export/mod.rs` (unambiguous, rename-safe).

## Self-check (what could go wrong)
- `crates/**` trigger generates friction on unrelated refactors (rename a cited symbol → docs-lint red until citation updated in-PR). Intended coupling; lead signs off. Fallback if noisy: paths-filter split to symbol-ref-check-only (non-breaking later; trigger paths identical).
- Guard predicate assumes `suffix_matches` returns toolkit-only for these refs — verified by logic, not against absent codec source; Risk-1's toolkit-only re-run closes it (a surprise CLI collision shows as `collision`, still safe).
- A future toolkit citation to a not-yet-existing symbol still skips in bare CI (irreducible, named).
