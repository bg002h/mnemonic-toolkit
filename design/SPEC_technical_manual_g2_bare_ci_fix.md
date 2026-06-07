# SPEC — close `technical-manual-g2-uncovered-in-bare-ci` (toolkit-G2 airtight in bare CI)

**Cycle:** technical-manual Cycle E — the residual FOLLOWUP filed by Cycle C+D (`cdc0479`).
**Date:** 2026-06-07.
**Source SHA:** `origin/master` == local `HEAD` == `cdc0479`.
**Disposition:** docs + CI only — **no version bump, no tag** (binary byte-identical; same class as `cdc0479` / `6c9e629`).
**Resolves:** `technical-manual-g2-uncovered-in-bare-ci`.
**Recon:** `cycle-prep-recon-technical-manual-g2-uncovered-in-bare-ci.md` (citations ACCURATE; fix scoped empirically). **Design:** `design/agent-reports/technical-manual-g2-fix-architect-blueprint.md` (architect consult; decisions A/B/C).
**Locksteps:** none (no clap-flag / CLI / codec surface → no GUI `schema_mirror`, no manual-mirror, no sibling-codec companion).

---

## 0. The gap (recap) + the design key

The single-repo (toolkit-only) technical-manual CI (`cdc0479`) under-enforces symbol-ref **G2** in two distinct ways:
- **Gap (1) skip-logic** — siblings absent, a bare/subpath ref in a NON-authoritative chapter unresolvable in the present toolkit repo is `skip:absent-sibling`. Two sub-parts: **(i)** codec-file G2; **(ii)** a *renamed toolkit file* cited by bare/subpath from a non-auth chapter.
- **Gap (2) trigger-path** — the workflow fires on `docs/technical-manual/**`, not `crates/**`, so a toolkit-source rename that breaks a citation without a docs edit won't fire CI.

**Design key (architect-validated):** bare CI **can** fully enforce **toolkit**-G2 (toolkit source IS the repo) but **cannot** enforce **codec**-G2 (needs the deliberately-absent siblings — the rejected multi-repo coupling). So the principled, achievable target is: **100 % toolkit-G2 in bare CI; codec-G2 stays local-`make lint`-only (accepted residual).**

**Empirical scope (probe at `cdc0479`):** gap (1) part (ii) is **NOT vacuous** — **12** non-auth refs (all in `60-back-matter/61-glossary.md`) resolve to toolkit files by bare/subpath. They resolve & are checked today; a future rename of `cmd/bundle.rs` / `cmd/verify_bundle.rs` / `wallet_export/{electrum,mod}.rs` / `cmd/export_wallet.rs` would silently `skip` (not FAIL) in bare CI. Plus 59 non-auth codec refs (part i), 40 already-qualified non-auth refs, and **268** authoritative-chapter (41/42/54) toolkit refs (FAIL-on-rename once the workflow fires).

**LOAD-BEARING SHARPENING:** *subpaths are NOT rename-safe.* Only the full `crates/mnemonic-toolkit/src/...` form satisfies `is_repo_qualified` (`symbol-ref-check.py` — `"crates/" in pathpart` or a repo-dir prefix) and routes through the `crates/` early-branch (`resolve()` top: `re.search(r'crates/([A-Za-z0-9_-]+)/((?:src|tests)/.+)$', ...)`) that checks the present toolkit repo and returns `unresolved` (→ FAIL) when the file is absent post-rename. A subpath (`cmd/bundle.rs`) fails `is_repo_qualified` → suffix-match → resolves today but `skip`s after a rename. **So the canonical fix form for all 12 is full `crates/mnemonic-toolkit/src/<subpath>`, NOT a basename-disambiguation and NOT the double-prefix `mnemonic-toolkit/crates/...` variant.**

---

## Item 1 — Workflow trigger (gap 2)

`.github/workflows/technical-manual.yml`: add to BOTH `push.paths` (after line 20) and `pull_request.paths` (after line 25):

```yaml
      - 'crates/mnemonic-toolkit/src/**'
      - 'crates/mnemonic-toolkit/tests/**'
```

Keep the existing single `make lint` step (Decision A — NOT a split symbol-only job). Update the top-of-file comment block to state the workflow now also fires on toolkit-source changes so a toolkit-symbol rename re-runs `symbol-ref-check`, and the run remains bare-CI (toolkit-only checkout) — codec refs still skip.

**Decision A justification (architect):** leaving gap (2) soft defeats the Item-2/3 work (a rename wouldn't fire the lint → the qualified-12 + 268 auth refs only FAIL on the next docs edit). The full `make lint` is fine on code PRs: 5 of the other 6 checks (markdownlint, cspell, lychee, glossary-coverage, index-bidirectional) scan **docs text only**, and the 6th (api-surface-coverage) reads `lib.rs`/`format.rs` source **but is warning-only** (`lint.sh:82` `|| warn`, never sets `fail=1`) — so on a code-only diff symbol-ref-check (step 7) is the *only* blocking check that can react, and the 6 **cannot newly fail** (red only if master already red). I.e. the full lint is functionally symbol-ref-check-only on code PRs; a `paths-filter` split adds disproportionate workflow complexity (a non-breaking later optimization if it proves noisy). `tests/**` is included because `INDEX` walks `tests/` and `tests::`-anchored citations are a documented anchor form. The accepted cost: a code PR that renames a cited symbol must carry the docs edit in-PR — the intended symbol-level lockstep, consistent with the repo's manual-mirror / schema-mirror culture.

---

## Item 2 — Gate guard (gap 1 part ii, going-forward)

`docs/technical-manual/tests/symbol-ref-check.py`:

**(2a) `resolve()`** — inside the existing `if not qualified:` block, immediately after the collision return (`return (None, "collision")`), add:

```python
        if not auth and repos_with == ["toolkit"]:
            return (None, "unqualified-toolkit")
```

**(2b) scan loop** — after the `if status == "collision":` arm, add a mirroring `err()` arm:

```python
                if status == "unqualified-toolkit":
                    err("%s:%d `%s::%s` — non-authoritative chapter cites a toolkit "
                        "file by bare/subpath; fully qualify as "
                        "crates/mnemonic-toolkit/src/... (only that form is "
                        "rename-safe; AUTHORING Source citations)"
                        % (rel, i, pathpart, anchor))
                    continue
```

**Decision B justification (architect-traced):** hard FAIL, not WARN (WARN re-creates the lagging-indicator failure mode CLAUDE.md already burned on). Predicate `not auth and repos_with == ["toolkit"]` (where `repos_with = sorted({h[0] for h in suffix_matches(pathpart)})`, already computed in the block). Verified against the four exclusion requirements:
- **Fires in BOTH local and bare-CI modes** — `toolkit` is present in both, so a toolkit-resolving ref yields `repos_with == ["toolkit"]` in both. (This is exactly why a *toolkit* predicate works where a *codec* one cannot.)
- **Doesn't touch authoritative chapters** — gated on `not auth`; the 268 auth-toolkit refs resolve via the earlier `if auth and not qualified:` branch and return `ok` before this code.
- **Doesn't touch bare codec refs** — bare CI: codec absent → `repos_with == []` → predicate false → falls to `skip:absent-sibling`. Local: codec present → `repos_with == ["md"]`/etc. → false → resolves & G2-checks. Codec path untouched in both modes.
- **Doesn't break the 40 qualified or 268 auth** — qualified refs are `is_repo_qualified == True` → the `crates/` early-branch returns before the block; auth refs have `not auth == False`.

It also correctly catches the ambiguous-within-toolkit case (bare `electrum.rs`, 3 toolkit paths → `repos_with == ["toolkit"]`).

**Two-part mechanism (qualify-now + guard-forever) — both halves required:** a *renamed* bare ref resolves to nothing → `repos_with == []` → the guard does NOT fire → `skip`. So the guard alone cannot protect the 12 across a rename. **Item 3 qualifies them now** (routes through the `crates/` branch → FAIL on rename). **This guard prevents new** bare/subpath toolkit refs (at commit the file exists → `["toolkit"]` → FAIL → author must fully-qualify → rename-safe thereafter). Irreducible residual (named, not chased): a bare ref to a not-yet-existing toolkit symbol (typo/future) → `skip`, genuinely indistinguishable from a codec ref.

---

## Item 3 — Qualify the 12 toolkit refs in `61-glossary.md` (gap 1 part ii, existing)

Per-TOKEN edits (grep-verified exact tokens at `cdc0479`). `::method` continuations like `` `::emit_multisig_checks` `` are NOT `TOKEN_RE` matches (no `.rs::` prefix) → left as prose. On lines 229/385/433 an already-`crates/`-qualified token coexists with a bare one — **qualify only the bare token, leave the qualified one untouched.**

| line | bare/subpath token → full-qualified |
|---|---|
| 89 (×2) | `bundle.rs::build_unified_card` → `crates/mnemonic-toolkit/src/cmd/bundle.rs::build_unified_card` |
| 113 | `verify_bundle.rs::MappingFailure` → `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::MappingFailure` |
| 141 | `wallet_export/electrum.rs::ELECTRUM_SEED_VERSION_PIN` → `crates/mnemonic-toolkit/src/wallet_export/electrum.rs::ELECTRUM_SEED_VERSION_PIN` |
| 217 (×2) | `verify_bundle.rs::emit_md1_checks` → `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::emit_md1_checks`; `verify_bundle.rs::emit_multisig_checks` → `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::emit_multisig_checks` |
| 229 | `wallet_export/mod.rs::build_missing_fields_refusal` → `crates/mnemonic-toolkit/src/wallet_export/mod.rs::build_missing_fields_refusal` (leave the already-qualified `…/mod.rs::MissingField` alone) |
| 373 | `wallet_export/mod.rs::TaprootInternalKey` → `crates/mnemonic-toolkit/src/wallet_export/mod.rs::TaprootInternalKey` |
| 385 | `cmd/export_wallet.rs::ExportWalletArgs::timestamp` → `crates/mnemonic-toolkit/src/cmd/export_wallet.rs::ExportWalletArgs::timestamp` (leave the already-qualified `…/mod.rs::TimestampArg` alone) |
| 433 (×2) | `wallet_export/mod.rs::script_type_from_template` + `wallet_export/mod.rs::script_type_from_descriptor` → `crates/mnemonic-toolkit/src/wallet_export/mod.rs::…` (leave the already-qualified `…/mod.rs::WalletScriptType` alone) |
| 453 | `verify_bundle.rs::MappingFailure::XpubNotInPolicy` → `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::MappingFailure::XpubNotInPolicy` |

File-path mapping verified against the tree: `bundle.rs`→`cmd/bundle.rs`; `verify_bundle.rs`→`cmd/verify_bundle.rs`; `export_wallet.rs`→`cmd/export_wallet.rs`; `electrum.rs` exists at 3 toolkit paths so the `wallet_export/` subpath disambiguates → full `crates/mnemonic-toolkit/src/wallet_export/electrum.rs`; `wallet_export/mod.rs` → full `crates/mnemonic-toolkit/src/wallet_export/mod.rs`.

After qualification each token routes through the `crates/` early-branch → resolves (file present) → the scan loop still runs the per-`::`-segment existence check (qualifying does NOT lose segment validation). On a future rename the file is absent → `unresolved` → FAIL.

**Prose-staleness on line 385 is OUT OF SCOPE (R0-r1 M2, recommendation (a)).** The glossary entry at lines 383-385 describes `TimestampArg` as `Now (renders to "now")` / `Unix(i64)` with `default now`; source is now `pub timestamp: TimestampArgValue` with default `0`/genesis-rescan (v0.47.3 timestamp-default-zero cycle). **G2 is unaffected** — `ExportWalletArgs` + `timestamp` segments both exist (R0-r1 verified), so the qualification is segment-safe. This cycle pins the *path* only; it does NOT re-validate prose claims (consistent with the symbol-pin cycle's scope). The stale default/type prose is filed as a separate one-line FOLLOWUP `technical-manual-glossary-timestamparg-default-prose-stale` (ship plan step 3).

---

## Item 4 — AUTHORING.md (`docs/technical-manual/AUTHORING.md`)

Two mandatory edits:

**(4a) Generalize the "Colliding basenames" rule (lines 196-202).** Today it requires qualification only for *colliding* basenames in catch-all chapters. New rule: in a non-authoritative (catch-all: Foundations / Address-derivation / Glossary) chapter, **any toolkit-resolving ref — colliding or not, bare basename OR subpath — must be fully `crates/mnemonic-toolkit/src/...`-qualified**, because only the full form is rename-safe (a subpath resolves today but `skip`s after a rename in bare CI). The lint FAILs an unqualified toolkit ref in those chapters (`unqualified-toolkit`). (Authoritative codec/toolkit chapters may keep bare/subpath — the chapter encodes the repo and such refs FAIL-on-rename, not skip.)

**(4b) Fix the STALE "no CI workflow" claim (lines 222-227).** It currently reads "**The technical manual has no CI workflow** — `make lint` is the gate" — stale since `cdc0479` added `.github/workflows/technical-manual.yml`. Correct it: a lint-only CI workflow exists, firing on `docs/technical-manual/**` + the cache tool + toolkit `crates/mnemonic-toolkit/{src,tests}/**`; it runs bare-CI (toolkit-only), enforcing G1 on all chapters + toolkit-G2; **`make lint` with all sibling repos present remains the FULL gate — only it enforces codec-G2** (bare CI skips codec refs with a warning). (Optionally tighten line 194's "resolves `<path>` against the sibling codec source trees" to "against the toolkit repo + sibling codec source trees".)

---

## Item 5 — Codec-G2 (gap 1 part i): explicitly accepted residual (Decision C)

NOT closed; documented as accepted residual in the FOLLOWUP, the workflow comment, and AUTHORING (4b). Rationale: a basename manifest proves file-existence not symbol-existence (no true G2) and drifts on every codec rename (regen tax); a crates.io source-download couples docs CI to *published* codec versions that lag the dev symbols the manual cites vs `origin/master` (re-introducing the very drift-coupling the multi-repo checkout was rejected for). Both buy only "catch a codec-filename typo in bare CI" — already caught by local `make lint`. Net negative. Full codec-G2 gate = local `make lint` with all siblings present.

---

## 6. Verification plan (acceptance — run the REAL gate, do not trust "exactly 12")

1. **Pre-qualify RED proof (guard fires):** with Items 1-2 applied but BEFORE Item 3, run `symbol-ref-check.py` in **toolkit-only mode** (siblings absent — deterministic message, Risk 3) over all `src/`; assert it FAILs with `unqualified-toolkit` on the bare glossary refs. This proves the guard is non-vacuous.
2. **Post-qualify GREEN proof (toolkit-only / bare-CI):** after Item 3, run the gate toolkit-only over all `src/`; assert GREEN. **If any non-auth chapter other than the glossary harbors a toolkit-resolving bare/subpath ref the probe under-counted, the guard REDs it here — surface and qualify it (do not merge past it).** Also assert the **skip-count is unchanged** vs the pre-fix toolkit-only run for the address-derivation chapters (R0-r1 M3 regression guard): `to_miniscript.rs`/`address_derivation.rs` must still `skip:absent-sibling` (codec-only), NOT flip into `unqualified-toolkit` — a future toolkit basename collision would otherwise silently capture them.
3. **Local full-suite GREEN (siblings present):** `make -C docs/technical-manual lint` with all sibling repos present → GREEN (the 12 qualified refs still segment-check; 268 auth + 40 qualified unchanged).
4. **Bare-CI full `make lint` GREEN:** faithful siblings-absent `make lint` (via `git stash create` + `git archive` — NOT a symlink, which `make -C` resolves to the real cwd) → all 7 steps green.
5. **Planted-rename FAIL proof:** in a throwaway tree, rename one cited toolkit file (or its symbol) and confirm the now-qualified glossary ref FAILs (`unresolved`) under siblings-absent — proving FAIL-on-rename, the whole point.
6. **actionlint** on the edited workflow.
7. **CI run** after push: confirm the workflow run succeeds and the symbol-ref-check step reports the expected skip counts.

---

## 7. Ship plan

1. Apply Items 1-4.
2. Run the full verification plan (§6). Fold any surprise the guard surfaces.
3. `design/FOLLOWUPS.md`: flip `technical-manual-g2-uncovered-in-bare-ci` → resolved (gap 2 via trigger; gap 1 part ii via qualify-12 + guard; gap 1 part i codec-G2 accepted residual). File the one-line `technical-manual-glossary-timestamparg-default-prose-stale` (R0-r1 M2; `61-glossary.md:383-385` describes `TimestampArg` default `now`/`Unix(i64)` but source is `TimestampArgValue` default `0` since v0.47.3; prose-only, G2-unaffected).
4. Stage paths explicitly (no `git add -A`). Commit (Co-Authored-By trailer, `git commit -F -`). Push to `master`. **No bump, no tag.** Watch the CI run.
5. Record cycle in memory + MEMORY.md index.

### Out of scope
- Codec-G2 in bare CI (Item 5); multi-repo checkout; basename/crates.io manifest.
- The 40 qualified + 268 authoritative refs (unchanged); the other 6 lint checks.
- Any crate/CLI/codec change.
