# R0 Architect Review — technical-manual CI workflow + api-harvest transcript cleanup — Round 1

> SPEC under review: `/scratch/code/shibboleth/mnemonic-toolkit/design/SPEC_technical_manual_ci_and_transcript_cleanup.md`
> Source SHA: `6c9e629` (== local HEAD). All claims verified against current `src/` and `tests/` (no trust of SPEC assertions).
> Reviewer tools: Read/Glob/Grep/WebFetch. The gate could not be executed (no Bash); the fix logic was verified by static trace against `symbol-ref-check.py` and exhaustive enumeration of `.rs::` tokens in the load-bearing chapters.

## Verdict: GREEN (0 Critical / 0 Important)

The gate fix is logically sound, closes the documented hole, preserves the CI prize, and preserves local strictness. The transcript delete is link-safe. The YAML is binary-independent and correctly mirrors `manual.yml`. The disposition (no-bump/no-tag) is correct. The one substantive blemish is an internal-consistency nit between the SPEC's broad §2 disclosure and the narrow FOLLOWUP-id/ship-plan framing — recorded as Minor, not a block.

## Critical

None.

## Important

None.

## Minor

**M1 — The new-gap FOLLOWUP id + ship-plan step 5 understate the disclosed scope (internal inconsistency).** SPEC §2 (lines 109-111) honestly states the skip "extends to a non-authoritative-chapter ref to any file unresolvable in the present (toolkit) repo" — i.e. it includes *toolkit-file renames* cited by bare basename from a non-auth chapter, not just codec G2. But ship-plan step 5 (line 292) and the proposed FOLLOWUP id `technical-manual-codec-g2-uncovered-in-ci` narrow it to "codec G2." This is bounded and CI-only (locally `ABSENT == []`, the new branch never fires, so such a rename still FAILs `make lint` — verified by trace), so it does not weaken the guarantee in a blocking way. But this repo has a documented history of dropped/understated FOLLOWUPs (CLAUDE.md gui-schema-mirror case study; MEMORY notes "session-filed FOLLOWUP understated 2 sites"). Concrete fix: in the new FOLLOWUP body and ship-plan step 5, broaden the wording to inherit §2's framing — e.g. "non-auth-chapter refs whose file is unresolvable in the present (toolkit) repo are CI-skipped (includes toolkit-file renames cited by bare basename, not only codec G2); caught only by local `make lint`." Optionally rename the id to drop the `codec-g2` narrowing, or keep the id but make the body explicit.

**M2 — YAML comment slightly over-states the `*_BIN=true` mechanism.** The step comment (SPEC lines 236-239) says `*_BIN=true` "prevents the Makefile's default `cargo run ...` from compiling anything." In fact nothing in the `make lint` path ever *invokes* a binary: `api-surface-coverage.sh` explicitly ignores `MD_BIN/MK_BIN/MS_BIN/MNEMONIC_BIN` (lines 33-37, `MD_BIN=*) : ;`) and reads `lib.rs`/`format.rs` source directly; `symbol-ref-check.py` accepts-and-ignores them (line 33). So no `cargo run` would fire even with the Makefile defaults. The `=true` override is harmless and good defensive hygiene, but the "prevents compiling" framing implies the bins would otherwise run during lint, which they wouldn't. Reword to "the lint path never invokes a binary; `=true` is belt-and-suspenders to guarantee no accidental compile." Non-blocking.

**M3 — `30-address-derivation/33-network-and-addressing.md:19` cites `crates/md-cli/src/parse/keys.rs::parse_key`, exercising the `md-cli` qualified-skip arm.** Not a flaw — just noting for the audit trail that the qualified-codec-in-non-auth-chapter path (early `crates/` regex → `CRATE_REPO["md-cli"]` → `skip:md-cli` when absent) is real and correctly handled by the *existing* code, independent of the new branch. The fix does not regress it.

## Verified-correct (independently confirmed against source)

**Gate fix — hole closure (the 59 false-fails):**
- Bare codec basenames are present in exactly the two non-authoritative chapter groups the SPEC names. `30-address-derivation/{31,32,33}.md` cite bare `to_miniscript.rs::*` and `address_derivation.rs::*` (the latter an md-codec `tests/` file); `60-back-matter/61-glossary.md` cites bare `canonical_origin.rs`, `canonicalize.rs`, `key_card.rs`, `origin_path.rs`, `payload.rs`, `phrase.rs`, `identity.rs`, `to_miniscript.rs`. None of these have an `authoritative_repo` prefix (30s/60s are not in the 21/22/23/41/42/51/52/53/54 set, `symbol-ref-check.py:72-78`), so `auth is None`.
- Trace for a bare codec basename in a non-auth chapter, siblings absent: line 115 `crates/` regex misses → `qualified=False` (line 126), `auth=None` (line 127) → line 130 skipped → collision check (139-144): `suffix_matches` empty (codec absent) so `repos_with` empty, no collision → line 145 `shallowest` empty → line 148 `allhits` empty, not ambiguous → **new branch fires** (`not auth and not qualified and ABSENT`) → `skip:absent-sibling`. Hole closed. ✓

**Gate fix — prize preservation (renamed/fake TOOLKIT symbol in an authoritative chapter still FAILs in CI):**
- Enumerated *every* `.rs::` token in the three authoritative toolkit chapters (41-bundle-anatomy, 42-anti-collision-invariants, 54-mnemonic-toolkit-api). Every ref is either fully-qualified (`mnemonic-toolkit/crates/mnemonic-toolkit/src/...`) or a bare *toolkit-file* basename (`format.rs`, `synthesize.rs`, `bundle.rs`, `verify_bundle.rs`, `parse_descriptor.rs`, `error.rs`, `template.rs`, `parse.rs`, `derive.rs`, `bundle_unified.rs`, `slot_input.rs`, `wallet_export/*.rs`, `wallet_import/*.rs`, `cmd/*.rs`). **There is NOT a single bare codec basename in any of 41/42/54.** This is the dangerous path (auth chapter + bare codec basename → `auth` truthy → new branch's `not auth` guard does NOT fire → `unresolved` → CI FAIL). It does not occur today. ✓ (This is the claim Test E's "0 fails" stands on; I confirmed it directly rather than trusting the count.)
- All bare toolkit basenames resolve within toolkit: confirmed `crates/mnemonic-toolkit/src/{format,synthesize,parse_descriptor,template,parse,derive,error,bundle_unified,slot_input}.rs` and `cmd/{bundle,verify_bundle}.rs` and `wallet_export/*`/`wallet_import/*` exist (Glob). So in 41/42/54 the auth branch (line 130-135) resolves the file → segment check (lines 219-222) runs → a fake symbol/renamed file FAILs in CI. Prize preserved. ✓
- `auth` and `qualified` are bound at `symbol-ref-check.py:126-127`, in scope at the line-151 insertion point. ✓

**Gate fix — local strictness preserved:**
- With all repos present `PRESENT_REPOS` is full → `ABSENT == []` → the new branch's `and ABSENT` is falsy → branch never fires → a genuinely-missing symbol still reaches `return (None, "unresolved")` → FAIL. Local behavior byte-identical. Matches Test B (725 checked / 0 skipped). ✓

**Gate fix — over-skip narrowness:**
- `not qualified`: an explicit `crates/<codec>/...` ref routes through the line-115 early branch (returns `skip:<repo>` when absent) and never reaches the new branch. Verified live: `33-network-and-addressing.md:19` `crates/md-cli/.../parse_key`. ✓
- Segment check on resolvable toolkit files still runs: the new branch only fires when `shallowest`/`allhits` are *empty* (file unresolvable). A bare `bundle.rs::FakeSym` in a non-auth chapter with siblings absent still resolves to toolkit's `cmd/bundle.rs` (line 145) → segment check FAILs the fake. Only *file-unresolvable* refs skip. ✓
- Ambiguous-before-skip ordering is correct: the new branch is placed after the `ambiguous` return (line 149-150). Genuine within-toolkit ambiguity (≥2 toolkit hits, no shallowest-unique) returns `ambiguous` and FAILs both locally and in CI — it is not masked. ✓
- The `not auth` collision branch (line 143) is unaffected; the fix sits below it.
- Authoritative-fell-through path (auth chapter ref that misses its own repo, line 136 fall-through): `auth` stays truthy so `not auth` keeps it OUT of the new branch → it can still reach `unresolved`/FAIL. No such ref exists in 41/42/54 today (all resolve within toolkit), so behavior is acceptable. ✓

**Hoist safety:**
- `ABSENT = [r for r in REPO_ROOTS if r not in PRESENT_REPOS]` is safe to hoist to module scope provided it is placed *after* the `PRESENT_REPOS`-populating loop (ends line 69) and the bottom-of-file `absent = ...` (line 224) is replaced by a reference to it (no double-definition, no NameError). `resolve()` reads `ABSENT` as a global at call time (first call line 196, well after module load). **Implementation must:** assign `ABSENT` between line 69 and the scan loop, and replace line 224's `absent` with `ABSENT` in the warning at lines 224-227. ✓ (Flagged as an implementation requirement, not just "module scope.")

**Warning reword:** the bottom-of-file warning (line 226 "%d codec-chapter refs skipped") should become the SPEC's "sibling-repo refs skipped (codec G2 not enforced in bare CI)" since catch-all refs now also skip. Reasonable. ✓

**WS arithmetic (no off-by-one):** `WS = abspath(SRC_DIR/../../../..)` (line 39). Default `actions/checkout@v4` (no `path:`) lands the repo at `$GITHUB_WORKSPACE = .../work/mnemonic-toolkit/mnemonic-toolkit`; `SRC_DIR = .../mnemonic-toolkit/docs/technical-manual/src`; up-4 = `.../work/mnemonic-toolkit`; `REPO_ROOTS["toolkit"] = WS/mnemonic-toolkit/crates/mnemonic-toolkit` (line 42) = the checkout. Resolves correctly. The double-nest coincidentally mirrors the local `<workspace>/mnemonic-toolkit/` layout. ✓

**`--verify` is render-free:** `render-mermaid-cache.py` `verify()` (lines 126-148) is a pure `*.pdf` glob + set-difference; no `mmdc`/chromium/`subprocess` in the verify path (those are only in `render_block`/`mmdc_version`, regen-only). So `figures-cache-verify` (Makefile:214-219) needs no chromium. ✓

**`make lint` is binary-independent:** lint.sh steps 1-3 scan `$SRC_DIR` text; step 4 (`api-surface-coverage.sh`) is `|| warn` (lint.sh:82) and reads `lib.rs`/`format.rs` (api-surface-coverage.sh:189-219), exits 0 always (line 224); steps 5-6 are grep; step 7 is python. No `cargo build`/`cargo install`/chromium. ✓

**Failing lint fails the job:** lint.sh has `set -euo pipefail` (line 27) and explicit `exit 1` on `fail` (lines 142-145); the Makefile `lint` target runs `@bash lint.sh ...` with `figures-cache-verify` as a prereq (Makefile:222) — either a failing prereq or a non-zero lint.sh halts `make` → the YAML step exits non-zero → job fails. ✓

**Tool-install completeness:** `make lint` needs make + markdownlint-cli2 + cspell + lychee + python3; the YAML installs make, node20+markdownlint-cli2@^0.13+cspell@^8, lychee v0.24.2; python3/git/curl preinstalled on ubuntu-latest. No cargo, no pandoc/texlive, no chromium needed. Complete. ✓

**Triggers:** push (main/master) + PR on `docs/technical-manual/**` + `docs/tools/render-mermaid-cache.py` + the workflow file itself — covers every input `make lint` consumes (the gate, lint.sh, Makefile, and figures cache all live under `docs/technical-manual/**`; the cache verifier is the one external dep and is pathed). No tag trigger is correct: the technical manual ships no release asset; `manual.yml` owns `manual-v*` (verified manual.yml:14-15, 125-139). ✓

**Security posture:** the YAML uses no `${{ github.event.* }}` interpolation; all values are constants/built-ins. lychee install pins version + URL identical to the already-in-production `manual.yml:66-70` — supply-chain posture is proven by reuse. ✓

**Transcript delete is link-safe:** the 4 `api-harvest-*.md` exist (Glob); `api-harvest` is referenced nowhere outside the transcripts and `design/` docs (Grep across the whole repo — zero hits in `src/`, `tests/`, Makefile); no `{{#include}}` anywhere in `src/`; every `transcripts/` reference in `src/` points at `.cmd`/`.out` or `*-api-roundtrip` pairs (all kept), never at an `api-harvest-*.md`. The `.cmd`/`.out` and `*-roundtrip` pairs are untouched. ✓

**"Provably wrong to migrate" claim:** verified at `transcripts/api-harvest-mnemonic-toolkit.md:466` — it asserts the `BundleJson` doc-comment is at `format.rs:114`, but per FOLLOWUPS.md:2282 (api-harvest drift audit, resolved) `format.rs:114` now resolves to `MultisigInfo`. The transcript still carries live `file.rs:N` refs (e.g. `verify_bundle.rs:1214...`, `parse_descriptor.rs:1208,1212`). Delete-not-migrate is the right call; same class as the deleted cli-help goldens. ✓

**Disposition (no-bump/no-tag):** docs + CI only. No CLI surface change → no GUI `schema_mirror` implication (the gate iterates clap flag-names; a CI YAML + a python lint-gate touch neither). No sibling-codec lockstep (the workflow deliberately does not check out siblings; no flag/manual mirror affected). Binary byte-identical. Correct. ✓

**FOLLOWUPs:** both ids the SPEC claims to resolve exist — `technical-manual-ci-workflow-source-checkout` (FOLLOWUPS.md:2298, currently `open`) and `technical-manual-transcript-lineref-staleness` (:2306, `open`). The parent's "CI-gated OR make lint-gated" wording the SPEC leans on is confirmed at :2301. ✓

## Note to implementer

Land as GREEN. Before commit, apply the M1 wording broadening to the new FOLLOWUP entry + ship-plan step 5 (this is the only thing that touches the persisted artifact and matters for the audit trail). M2/M3 are optional cosmetic. Ensure the `ABSENT` hoist lands after the `PRESENT_REPOS` loop and the bottom-of-file `absent` reference is rewired to `ABSENT` (Verified-correct "Hoist safety"). Re-prove both legs after the fix (local 725/0 byte-identical + `/tmp/ciclone` siblings-absent Test E green) per ship-plan step 4. Stage paths explicitly.

---

**Persistence note (from reviewer):** the reviewer had no Write/Edit tool, so the parent agent persisted this verbatim. Verdict GREEN (0C/0I); the reviewer-loop converges after the M1 wording fold is applied and re-confirmed.
