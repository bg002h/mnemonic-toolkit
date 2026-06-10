# PLAN — CHANGELOG ritual resurrect (backfill 0.48.0–0.51.0 + tag-time guard)

**Status:** R0 round 1 YELLOW (0C/2I/6M) → all findings folded; awaiting round 2
**Source grounding verified at:** toolkit `origin/master` = `2228ad3` (2026-06-09)
**Resolves:** `design/FOLLOWUPS.md::changelog-md-release-ritual-lapsed-since-v0-47-4` (`:3776`) — user decision: **resurrect** (architect direction-consult concurs; backfill, not tombstone).
**Recon:** `cycle-prep-recon-changelog-md-release-ritual.md` (untracked).
**Shape:** docs+CI only — NO version bump, NO tag. One master commit (backfill + guard + checklist + FOLLOWUP flip + new readme-staleness FOLLOWUP; ride-along CUT per R0-r1 I2).

## 0. Why backfill (not tombstone), why a guard

- 4 README sites bill `CHANGELOG.md` as "the full release history" (`README.md:16`, `:61`; `crates/mnemonic-toolkit/README.md:10`, `:207`); the 5-version gap contains a **wire-content change** (v0.48.0 NUMS internal key) and the two biggest feature releases of the cycle (v0.50.0/v0.51.0). All 5 have rich verified sources — backfill is hours, a tombstone is permanent damage to the file's one claim.
- The lapse mechanism is located: `design/RELEASE_CHECKLIST.md` mentions the changelog **zero times** — the ritual lived in habit only. Same failure class as the install.sh self-pin lapse that produced `install-pin-check.yml` (its header: "silently lagged 4 releases") → same proportionate fix: a tag-time CI check + a checklist line.
- GUI repo: **no action** (its CHANGELOG is current through v0.29.0).

## 1. Backfill — 5 sections at the head of `CHANGELOG.md` (newest first)

Match the existing entry style exactly (see `[0.47.4]`): `## mnemonic-toolkit [X.Y.Z] — YYYY-MM-DD` + a bold one-line **SemVer-LEVEL — summary.** + 3-5 bullets + `Resolves <slug>` + audit-trail pointer (SPEC + agent-reports paths). Dates from the tag commits (all 5 verified):

| Section | Date | SemVer line | Primary sources |
|---|---|---|---|
| `[0.51.0]` | 2026-06-09 | MINOR — descriptor-builder archetype presets (Release B): `--archetype` (5 presets) + 9 param flags + `--emit-spec`; kind-aware diagnostic `flag` provenance; `--spec-schema` `archetypes` section | tag body @ `c1b1375`; `design/FOLLOWUPS.md:3746` Status; `design/SPEC_descriptor_builder_presets.md` |
| `[0.50.0]` | 2026-06-09 | MINOR — descriptor-builder engine (Release A): `mnemonic build-descriptor` — versioned JSON PolicyNode IR → 4-step funds-safety gate → descriptor + BIP-388 + cost preview + node-addressed diagnostics + `--spec-schema`; watch-only-out | tag body @ `ecba644`; `FOLLOWUPS.md:3746`; `design/SPEC_descriptor_builder_engine.md` |
| `[0.49.1]` | 2026-06-09 | PATCH — `restore --md1` reconstructs taproot NUMS multisig (tr-multi-a + tr-sortedmulti-a) incl. golden receive-address tests (routes around md-codec's SortedMultiA gap) (R0-r1 M4) | tag @ `b596d3f`; `FOLLOWUPS.md:67` |
| `[0.49.0]` | 2026-06-08 | MINOR — `export-wallet`/`bundle --descriptor` accept a BIP-388 wallet-policy JSON (auto-detect, expand → concrete descriptor; round-trip with `--format bip388` byte-stable for `description_template`+`keys_info` — `name` lossy, see open FOLLOWUP `bip388-policy-roundtrip-wallet-name-not-honored`) (R0-r1 I1) | tag @ `fa72455`; `FOLLOWUPS.md:3718` |
| `[0.48.0]` | 2026-06-08 | MINOR (**wire change**) — bundled tr-multisig md1 emits the BIP-341 NUMS internal key (`is_nums:true`) instead of cosigner @0; whole-bundle md1+mk1 shift | release commit `223b538` + tag annotation (tag → `dcbd14c`; v0.48.0 predates the one-commit-bump practice — R0-r1 M1); `FOLLOWUPS.md:77` |

Authoring rules: (a) facts ONLY from the cited sources — no from-memory embellishment; every bullet must be traceable to a tag body, resolved-FOLLOWUP sentence, or release commit; (b) keep each section 3-6 bullets (the gap entries are summaries, not the full per-release essays the live-ritual entries were — note this honestly in the first backfilled section is NOT needed; the style table above already keeps them shaped like real entries); (c) SemVer level must match the actual bump (verify against `git show <tag>:crates/mnemonic-toolkit/Cargo.toml`).

## 2. Guard — `.github/workflows/changelog-check.yml`

Sibling of `install-pin-check.yml` (NOT an edit to it — that workflow self-documents as self-pin-scoped). Same trigger + recovery-note pattern:

```yaml
name: changelog-check

# Fires on every `mnemonic-toolkit-v*` tag push. Asserts CHANGELOG.md has a
# section for the tagged version. Catches the drift mode where a release is
# tagged without the per-release CHANGELOG entry (the ritual silently lapsed
# for 5 releases, v0.48.0-v0.51.0, after v0.47.4 — filed 2026-06-09;
# both READMEs link CHANGELOG.md as "the full release history").
#
# Recovery if this fails: add the `## mnemonic-toolkit [X.Y.Z] — <date>`
# section to CHANGELOG.md, commit, force-push the tag to the new commit
# (or delete + re-create the tag and re-push).
#
# Scope: toolkit release tags only. Doc-manual tag namespaces
# (manual-v*/manual-gui-v*/quickstart-v*/tech-manual-v*/ultraquickstart-v*)
# are not gated.

on:
  push:
    tags:
      - 'mnemonic-toolkit-v*'

jobs:
  changelog-has-section:
    name: CHANGELOG.md has a section for this tag
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - name: Verify CHANGELOG section exists for tag
        env:
          TAG: ${{ github.ref_name }}
        run: |
          set -eu
          VERSION="${TAG#mnemonic-toolkit-v}"
          echo "tag: $TAG (version $VERSION)"
          if ! grep -q "^## mnemonic-toolkit \[$VERSION\]" CHANGELOG.md; then
            echo "::error::CHANGELOG.md has no '## mnemonic-toolkit [$VERSION]' section"
            echo "::error::Add the per-release section, commit, and force-push the tag."
            exit 1
          fi
          echo "OK CHANGELOG.md has a [$VERSION] section"
```

Notes: anchored BRE with escaped brackets (R0-r1 M3 — anchoring guards a future bullet quoting a header verbatim; `\[` keeps the bracket literal; `$VERSION` dots lax = nil risk); `actions/checkout@v5` matches the repo's pinned action major (the v5 bump cycle). Pre-push validation: `actionlint` on the YAML (established discipline) — the workflow can't be live-tested until the next tag (v0.52.0, stream B = its first live exercise; state this in the commit message — same posture as the crates/-trigger precedent: "fires first on a future toolkit tag").

## 3. Checklist — `design/RELEASE_CHECKLIST.md`

The file is currently install.sh-pin-scoped; add a new top-level section (before "## install.sh component table") so the toolkit ritual has a durable home, AND touch up the preamble (R0-r1 M5: its "cross-repo pins that no single repo's CI can verify" framing would contradict a CI-gated section — amend to "...cross-repo pins plus the toolkit per-release ritual"):

```markdown
## Toolkit per-release ritual (every `mnemonic-toolkit-v*` tag)

1. **CHANGELOG.md** — add the `## mnemonic-toolkit [X.Y.Z] — <date>` section
   in the release commit (CI-gated: `changelog-check.yml` fails the tag push
   without it; lapsed silently for v0.48.0–v0.51.0 — don't trust habit).
2. Version bump sites in ONE commit: `Cargo.toml` + `Cargo.lock` + both
   README `<!-- toolkit-version -->` markers + `scripts/install.sh` self-pin
   (CI-gated: `install-pin-check.yml`).
3. Full suite AFTER the bump; push; ALL master CI green; THEN tag.
```

(Items 2-3 codify the already-practiced release gate so the checklist is the single durable home of the whole ritual, not just the changelog line.)

## 4. Ride-along — CUT (R0-r1 I2); FOLLOWUP filed instead

R0 found the README staleness is 4 sites, not 1: `README.md:14` ("v0.43.x — twenty-three") + `README.md:44` ("Twenty-one", inventory missing `ms-shares`/`restore`/`build-descriptor`) + `crates/mnemonic-toolkit/README.md:10` (same "v0.43.x — twenty-three") + `crates/mnemonic-toolkit/README.md:32` (same "Twenty-one" + 3 omissions). Ground truth: 24 subcommands at v0.51.0. Fixing one site leaves identical falsehoods elsewhere → CUT from this cycle (scope discipline; option (a) of the R0 ruling). File FOLLOWUP `readme-subcommand-inventory-stale-4-sites` in the same commit: de-version/de-count the prose openers + complete the two inventories, tier docs-hygiene.

## 5. FOLLOWUP flip

`changelog-md-release-ritual-lapsed-since-v0-47-4` → **Status: resolved** (resurrect chosen; backfilled 0.48.0–0.51.0; guard `changelog-check.yml`; checklist section added). Note the guard's first live exercise = the next toolkit tag.

## 6. Verification

- `actionlint .github/workflows/changelog-check.yml` clean.
- Local guard rehearsal (R0-r1 M6): loop ALL FIVE backfilled versions `TAG=mnemonic-toolkit-v0.{48.0,49.0,49.1,50.0,51.0}` (each must PASS post-backfill) and `TAG=mnemonic-toolkit-v0.52.0` (must FAIL — proves the check discriminates).
- `grep -c "^## mnemonic-toolkit \[" CHANGELOG.md` = exactly **124** post-backfill (119 at `2228ad3` + 5).
- Backfill facts spot-audit: each section's bullets traceable to its cited source (R0 reviewer verifies).
- No other files touched; full test suite unaffected (docs+CI only) — run it anyway per ritual.

---

## Fold log

- **R0 round 1 (YELLOW → folded, 2026-06-09; persisted at `design/agent-reports/changelog-ritual-resurrect-r0-r1-review.md`):** I1 [0.49.0] "byte-perfect" → byte-stable-for-template+keys_info / name-lossy (an open FOLLOWUP refutes byte-perfect). I2 README ride-along CUT (staleness is 4 sites not 1; option (a)) → file `readme-subcommand-inventory-stale-4-sites` instead. M1 [0.48.0] source relabeled (tag → `dcbd14c`; `223b538` = release commit). M2 workflow comment += ultraquickstart-v*. M3 grep → anchored BRE w/ escaped brackets. M4 [0.49.1] "bc1p" softened to "golden receive-address tests". M5 checklist preamble touch-up added. M6 rehearsal loops all 5 versions + count pinned 119→124. Fact audit: all 5 SemVer levels + dates confirmed against the tags.
