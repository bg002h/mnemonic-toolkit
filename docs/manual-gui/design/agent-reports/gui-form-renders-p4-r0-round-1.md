# Per-phase R0 review — Leg-2 P4 (manual catch-up + GUI pin bump)

**Cycle:** generated, gated GUI form renders. **Leg/Phase:** Leg 2 / P4.
**Branch:** `feat/manual-gui-form-renders` @ `4abf56ad` (off `origin/master`).
**Reviewer:** opus architect (adversarial; verified against source + live gates).
**Date:** 2026-06-29.
**Diff under review:** `git diff origin/master..HEAD` — 17 files, +2596/-246
(the bulk is the documentary `tests/expected_gui_schema_inventory.json` regen).

---

## VERDICT: GREEN — 0 Critical / 0 Important

P4 is correct and complete. The pin bump is exact and lockstep across both live
version-sites; the bidirectional `gui-schema-coverage` gate is genuinely green
against a real `mnemonic-gui-v0.53.0` checkout (982 anchors / 61 subcommands,
0 missing / 0 orphan); the `word-card` chapter is an accurate description of the
real toolkit-v0.74.0 behavior and — crucially — does **not** propagate the GUI
schema's two inaccurate help strings; `verify-examples` is a verified NO-OP
against the EXACT pinned binaries (17/17, built from the pinned tags, not the
locally-ahead bins); and the diff is catch-up + pin only with no P5 render work
leaked in. Proceed to P5.

No Critical or Important findings. Two Minor / Nit items below, neither blocking.

---

## Pin-completeness check (item 1) — PASS

Live `mnemonic-gui-v0.53.0` `pinned-upstream.toml` pins:
`[mnemonic] tag = mnemonic-toolkit-v0.74.0`, `[md] = descriptor-mnemonic-md-cli-v0.11.0`,
`[ms] = ms-cli-v0.13.0`, `[mk] = mk-cli-v0.11.0`.

Manual `docs/manual-gui/pinned-upstream.toml` after P4 mirrors EXACTLY:
- `[mnemonic-gui] tag = "mnemonic-gui-v0.53.0"` (`pinned-upstream.toml:25`)
- `toolkit-tag-implied = "mnemonic-toolkit-v0.74.0"` (`:28`)
- `md-cli-tag-implied  = "descriptor-mnemonic-md-cli-v0.11.0"` (`:29`)
- `ms-cli-tag-implied  = "ms-cli-v0.13.0"` (`:30`)
- `mk-cli-tag-implied  = "mk-cli-v0.11.0"` (`:31`)

**Both live version-sites moved in lockstep.** The second site —
`.github/workflows/manual-gui.yml` Job-1b `verify-examples` — was bumped on all
four hardcoded `--tag` install lines (`manual-gui.yml:155 md-cli-v0.11.0`,
`:158 ms-cli-v0.13.0`, `:161 mk-cli-v0.11.0`, `:172 mnemonic-toolkit-v0.74.0`),
plus the matching banner comments (`:113`, `:121-122`, `:163-165`). No straggler
at v0.49.0 / v0.70.0 / v0.7.0 / v0.8.0 / v0.9.0 survives in any **live pin site**:
a tree-wide grep for pin-shaped lines (`tag =` / `--tag` / `-tag-implied`) at the
old versions returns only design-doc narrative
(`design/SPEC_generated_gui_form_renders.md:43` describing the pre-bump starting
state) and legitimate history (`CHANGELOG.md`, `94-release-history.md`,
`FOLLOWUPS.md`, `agent-reports/`). The residual `v0.70.0`/`v0.49.0` strings in
`src/*.md` are feature-introduction prose ("(v0.70.0; #28 phase 2)", "At v0.49.0
the schema kind is free-form") — not pins. **Clean.**

The third lockstep version-site (the `gui-render` install line) is correctly
deferred to P5 per plan §29/m3 — not a P4 omission.

---

## Coverage-gate re-verification (item 2) — PASS (genuinely green, not faked)

Built `make md` + `make html` fresh, then ran `make lint` with
`MANUAL_GUI_UPSTREAM_ROOT` = the sibling `mnemonic-gui` checkout at the EXACT
pinned tag (`git describe --exact-match` → `mnemonic-gui-v0.53.0`). Result
(`lint exit: 0`):

```
[lint] 4/7 gui-schema-coverage  OK: 982 schema anchors (61 subcommands) all present in HTML
[lint] 5/7 outline-coverage     OK: 129 outlines all present with correct bullet count
[lint] 6/7 glossary-coverage    (pass)
[lint] 7/7 index bidirectional  (pass)
[lint] OK
```

- **Bidirectional clean:** `check_gui_schema_coverage.py` live-extracts the
  v0.53.0 schema (it does NOT read the committed inventory JSON) and computes
  `missing = expected - found` AND `orphans = schema-shaped found - expected`.
  Both empty → the implementer added no orphan anchors and missed no schema
  entry. **61 subcommands** confirmed (was 56 at v0.49.0; +5 = `word-card` +
  `gen-man`×4).
- **Delta is exactly +17 anchors / +1 outline, accounted for:**
  - `word-card`: 1 sub anchor + 8 flag anchors = **9** (the `--from --decode
    --decode-plate --raid --parity-words --parity-pct --integrity-bits --json`
    set), and 1 outline (8 flags ≥ 2 → outline required).
  - `gen-man` ×4 tabs: 4 sub anchors + 4 `--out` flag anchors = **8**, 0
    outlines (1 flag each < 2 → no outline).
  - Total **9 + 8 = 17 anchors, 1 outline.** Matches the claim precisely.
- **No latent coverage hole.** The 9th word-card slice entry
  (`NO_AUTO_REPAIR_FLAG`, a `global: true` flag appended as a bare const
  reference to all 33 subcommands) is **not** extracted by
  `extract_gui_schema._split_flagschema_blocks` — it only captures inline
  `FlagSchema { … }` brace-blocks, so the bare identifier is skipped. The gate
  therefore expects exactly the 8 inline word-card flags, never a per-subcommand
  `#mnemonic-word-card-no-auto-repair`. This is the established pattern (every
  other subcommand carries the same bare ref and the gate has been green).
  Positionals (`WORD_CARD_POSITIONALS` → `words`) are likewise not extracted
  (`_collect_subcommands` captures only `flags:`), so no `words` anchor is
  required. The committed `4n-word-card.md` documents precisely the 8 gateable
  flags. **Green now = green at P5's HTML embed; no future RED.**

---

## word-card accuracy ruling (item 3) — ACCURATE (correctness PASS)

Verified the chapter against the GROUND TRUTH (toolkit-v0.74.0 source, not the
GUI schema help):

`git show mnemonic-toolkit-v0.74.0:crates/mnemonic-toolkit/src/cmd/word_card.rs`:
```
//! encode an `mk1` / `md1` card ... Word Cards carry the xpub (`mk1`) /
//! descriptor (`md1`) — **watch-only, public-ish** material, NOT spending
//! secrets ... the `ms1` entropy card is intentionally NOT word-card-able.
/// Source `m*1` card ... an `mk1` xpub card or an `md1` descriptor card ...
/// PUBLIC material (xpub / descriptor) — not a secret.
#[arg(long, value_name = "MK1|MD1")] pub from: Vec<String>,
```

- **`--from` correctly described as PUBLIC `mk1`/`md1`, NOT a seed phrase**
  (`4n-word-card.md:3,12-16,37-41`). The GUI v0.53.0 schema's `--from` help
  ("Source BIP-39 mnemonic to encode into a word-card; phrase=/ms1=/entropy=/
  seedqr=", `mnemonic-gui/src/schema/mnemonic.rs:4221-4224`) is **inaccurate** —
  it describes secret seed intake. The implementer correctly did NOT propagate
  it. A manual that mis-described `--from` as a seed phrase would have been a
  correctness defect; this one avoids it. This is exactly the item-3 hazard, and
  it was handled right.
- **ms1 EXCLUDED / no secret-bearing field / no run-confirm modal** — accurate
  (`4n-word-card.md:12-16,118-119`; Refusals row `:130`); matches the module
  doc's "ms1 ... intentionally NOT word-card-able" and the toolkit's explicit
  non-secret classification of `--from`.
- **Second un-propagated GUI inaccuracy:** the schema's `--decode-plate` help
  ("Decode from an engraved plate string (the steel-plate serialization)",
  `schema/mnemonic.rs:4178-4180`) is also wrong; the manual instead describes
  the real toolkit semantics ("one RAID plate's word list … reconstruct a lost
  data plate", `4n-word-card.md:62-68`), matching `word_card.rs:58-62`
  (`value_name="WORDS", conflicts_with="words"`). Correct.
- **All 8 flags match `mnemonic word-card --help` @ v0.74.0** (verified against
  `word_card.rs` clap args): `--integrity-bits` default 44 / min 33 (confirmed
  `wc_codec::DEFAULT_INTEGRITY_BITS=44`, `MIN_INTEGRITY_BITS=33` at
  `crates/wc-codec/src/pipeline.rs:72,74`); `--parity-words` ⌊m/2⌋ correct,
  default 0, xor `--parity-pct`; `--parity-pct` `m=ceil(K*pct/100)`;
  `--raid 0|1|2` default 0, RAID-5/6, needs ≥2 mk1 data cards; `--json`; the
  `--decode` ↔ positional `<WORD>...` vs `--decode-plate` mutual exclusion. Every
  refusal row corresponds to a real clap `conflicts_with` / validation edge.
- **Worked example is PUBLIC and SAFE** (`:18-24,107-124`): derives the `mk1`
  from the canonical swept all-`abandon` test seed inside a `:::danger` box, and
  a Word Card encodes a *public* card carrying no spend capability. No secret
  appears in the example.

**Ruling: the chapter is a faithful, complete, public/safe description of the
real toolkit-v0.74.0 `word-card`. No correctness defect.**

---

## inspect --json correction (item 4) — PASS

The mnemonic `inspect` schema at v0.53.0 already carries `--json` (and
`--reveal-secret`) — `INSPECT_FLAGS` in `mnemonic-gui/src/schema/mnemonic.rs`.
P4 did **not** touch `src/40-mnemonic/4h-inspect.md` (absent from the diff), and
`#mnemonic-inspect-json` is present in the HTML (the coverage gate is 0-missing).
So `inspect --json` is genuinely pre-existing — not a P4 coverage hole. The md1
`@N`-template `--json` output is a separate toolkit-v0.75.0 feature (RESOLVED in
commit `52d84b98`), past the v0.74.0 pin, so correctly absent from the
v0.74.0-pinned schema and correctly undocumented this cycle. Implementer's claim
confirmed.

---

## verify-examples NO-OP (item 5) — PASS (verified against the EXACT pinned tier)

The locally-installed bins are ALL ahead of the pin (`mnemonic 0.75.0`,
`ms 0.13.2`, `mk 0.11.2`, `md 0.11.3`), and the Makefile default `MNEMONIC_BIN`
is a `cargo run` over the working tree (also 0.75.0) — exactly the skew item 5
warns about. To avoid trusting it, I built the four binaries from the pinned tags
in throwaway worktrees and ran `make verify-examples` with explicit overrides:

```
mnemonic 0.74.0 / md 0.11.0 / ms 0.13.0 / mk 0.11.0
[verify-examples] OK (17 transcripts pass)     (exit 0)
```

17/17 against the precise pinned tier; **zero `.out` drift**. P4 touched zero
files under `transcripts/` (diff confirms), and a grep of all 17 goldens for
version strings is clean — so the v0.70→v0.74 implied-pin bump cannot drift them.
The intervening v0.71–v0.74 cycles were additive (man-pages, BSD/musl targets,
word-card subcommand), changing no existing-command output. Confirmed NO-OP.

---

## Inventory regen + builds (item 6) — PASS

`tests/expected_gui_schema_inventory.json` regenerated (+2440 net); large because
it was last regenerated at v0.49.0 and now reflects the full v0.53.0 schema
(+305 `name:` lines / −47). It is **documentary only** — neither `lint.sh` nor
`check_gui_schema_coverage.py` reads it (the gate re-extracts live), so its churn
carries no gate risk. `make md` (already fresh) and `make html` both build green;
the HTML drives the (green) coverage/outline/glossary/index lints. `build/` is
git-ignored — the rebuild dirtied no tracked file.

---

## Scope hygiene (item 7) — PASS

- **Catch-up + pin only.** All 17 touched files are pin (`manual-gui.yml`,
  `pinned-upstream.toml`), the `word-card`/`gen-man` chapters + nav/overview/
  glossary/release-history/index/cspell updates, and the inventory regen.
- **No P5 render work leaked in:** tight grep for `transcripts/gui/`, `*.gui`,
  `verify-examples-gui`, `src/bin/gui_render`, `30-tour/` → none.
- **No `git add -A`:** the commit staged exactly the intended 17 paths; no stray
  files.
- **Pre-existing `IMPLEMENTATION_PLAN_…md` working-tree mod left alone** (still
  shows `M`, uncommitted) — correct per the plan note. The unrelated untracked
  `design/*` and `cycle-prep-recon-*` files are pre-existing clutter from other
  cycles, untouched by P4.
- Branch `feat/manual-gui-form-renders` @ `4abf56ad` intact; my temporary build
  worktrees were removed (`git worktree remove`), leaving all repos clean.

---

## Critical
None.

## Important
None.

## Minor / Nit (non-blocking)

- **M1 (cross-repo, sibling `mnemonic-gui`, optional FOLLOWUP).** The
  `mnemonic-gui-v0.53.0` schema carries two stale/inaccurate `help:` strings —
  `word-card --from` ("Source BIP-39 mnemonic … phrase=/ms1=/entropy=/seedqr=",
  `src/schema/mnemonic.rs:4221`) and `word-card --decode-plate` ("Decode from an
  engraved plate string", `:4178`) — both contradicting the toolkit-v0.74.0
  reality (public `MK1|MD1`; a RAID plate's word list). `schema_mirror` gates
  flag NAMES, not help text, so these are ungated drift in the sibling. P4's
  manual is correct and is the right call; but per the CLAUDE.md cross-repo
  follow-up convention, consider filing a `mnemonic-gui` FOLLOWUP to fix the two
  help strings so the GUI tooltip stops telling users `--from` takes a seed
  phrase. **Not a P4 defect** — flagged only so the sibling inaccuracy is on the
  record.

- **N1 (nit).** `4n-word-card.md` outline uses a `##`-level `Outline` heading
  (`{#mnemonic-word-card-outline}`); the `lint.sh` header comment phrases the
  rule as `### Outline`. The `outline-coverage` check keys on the anchor + bullet
  count (not the heading level) and is green, so this is purely a comment/style
  nit — no action required.

---

**Re-affirmed verdict: GREEN, 0 Critical / 0 Important.** P4 may proceed to P5.
Live gates re-run this round: `make lint` (exit 0, 982 anchors / 61 subs, 0/0)
and `make verify-examples` against pinned binaries (exit 0, 17/17). Branch left
clean.
