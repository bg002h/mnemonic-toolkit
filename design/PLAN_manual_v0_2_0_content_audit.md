# PLAN manual-v0.2.0 — v0.28.0 shipped-content audit via transcripts-as-audit + CI wiring

**Manual target:** `manual-v0.2.0` (minor bump on the `manual-v*` tag stream; minor because this cycle adds new transcripts infrastructure + CI gate, not just a prose patch). Current latest tag verified at write time: `manual-v0.1.10` (`git tag -l 'manual-v*' | sort -V | tail -1`). Earlier handoff memory referring to `manual-v1.0.1` was the GUI manual stream (separate; `manual-gui-v*` tags), conflated in initial draft.
**Toolkit target:** none in default path. Promote to `mnemonic-toolkit-v0.28.2` patch IFF P2 audit-report classifies a finding as toolkit-binary behavior bug per Q7 gray-area.
**GUI target:** none (no CLI surface changes; no `mnemonic-gui/src/schema/mnemonic.rs` lockstep triggered).
**Status:** LOCKED at R3 0C/0I — R0 (3C/8I) → R1 (0C/4I) → R2 (0C/1I/2M) → R3 (0C/0I/1M) all folded inline; user approved via ExitPlanMode 2026-05-20. Ready for P0 execution.
**Predecessor:** `8977389` (chore(install): bump mnemonic-toolkit pin v0.28.0 → v0.28.1, 2026-05-20).
**SHA at write time:** `8977389` (origin/master). Re-grep all line-number citations against this SHA at execution start; per CLAUDE.md, FOLLOWUPS citations decay each merge.

---

## §0 Context

The v0.28.0 cycle's wave-3 manual deliverables (master squash-merge `d18787f`, PR #52, feature-branch tip `3594a68`, +705/-58 LOC) added 6 new foreign-format parser sections to `docs/manual/src/45-foreign-formats.md` (298 → 791 lines) and 5 new cross-format recipes to `docs/manual/src/30-workflows/39-cross-format-conversion.md` (180 → 334 lines). Prior master commits touching these files: `45-foreign-formats.md` predecessor is `66c8a56` (v0.26.0 release-lockstep, three-feature); `39-cross-format-conversion.md` predecessor is `77ebfca` (v0.27.0 release — cross-format wallet conversion, PR #27). The P13D phase ran the existing `make lint` static gates against the new prose, but:

- **No human prose-quality re-read** occurred for either file post-merge.
- **No commands were run end-to-end** against the shipped binary. The lint gate is purely a static flag-coverage check (per `docs/manual/tests/lint.sh` lines 63-98) — it does not execute documented examples.
- The architect-must-run-prose-commands feedback (auto-memory `feedback_architect_must_run_prose_commands`) calls out exactly this risk class: source-faithful prose can still ship broken if the commands themselves fail. v0.13.0 P3 R1 caught 3 Critical content errors via running commands that R0 source-grep missed.

Separately, `docs/manual/transcripts/` already has 5 `.cmd`+`.out` pairs (22-first-bundle, 23-verify, 24-recover-*) covering the v0.4.x-era "first bundle / verify-bundle / recovery" walkthrough, plus a `cli-help/` subdir holding **21 `.txt` `--help` snapshots** (no `.cmd` shape; `md-*.txt`, `ms-*.txt`, `mnemonic-*.txt` — no `mk-*.txt` despite mk being a real binary in CI), plus a `verify-examples.sh` runner (66 lines) and a Makefile `verify-examples` target. **But:** `make verify-examples` is wired into neither `make lint` nor `.github/workflows/manual.yml`. CI runs `make lint MNEMONIC_BIN=true` (placeholder; line 81 of manual.yml), so the existing 5 transcript pairs are not regression-gated in CI; flag-coverage emits a load-bearing-but-actioned-by-nobody warn `[lint] WARN: no flags parsed from \`true gui-schema --help\`; skipping` per `lint.sh:84-87` for each of the 32 subcommands in `cli-subcommands.list`.

For context: the mature analog at `docs/technical-manual/transcripts/` carries **15 `.cmd`+`.out` pairs** (md1-/mk1-/ms1- encode + decode + address transcripts, plus api-roundtrip pairs and 4 `api-harvest-*.md` non-transcript artifacts).

This cycle therefore converges three concerns into one coherent piece of work:

1. **Audit** the 6 Round-trip examples (Sparrow / Specter / Coldcard-singlesig / Coldcard-multisig / Jade / Electrum) in `45-foreign-formats.md` and the 8 recipes in `39-cross-format-conversion.md` by running each against a freshly-built binary and diffing actual stdout+stderr against documented prose. BSMS Round-2 + Bitcoin Core sections have no Round-trip subsection and are descriptive-only; they are exercised via recipes 1-3 in `39-cross-format-conversion.md`.
2. **Establish** the audit artifacts as durable transcript fixtures under `docs/manual/transcripts/foreign-formats/` and `docs/manual/transcripts/cross-format-recipes/` — building on the existing transcript pattern at `docs/manual/transcripts/22-first-bundle.{cmd,out}` and the more mature analog at `docs/technical-manual/transcripts/` (15 pairs).
3. **Wire** `make verify-examples` into `.github/workflows/manual.yml` against a freshly-built `mnemonic` binary, simultaneously upgrading the `make lint` flag-coverage check from no-op (`MNEMONIC_BIN=true`) to real binary, closing FOLLOWUP `manual-yml-bind-real-mnemonic-bin`.

The intended outcome: every documented command block in chapters 45 + 30/39 has a captured `.cmd`+`.out` pair, CI fails fast on any future prose drift, and the audit either ships clean (prose was accurate) or surfaces specific findings that get fixed-in-cycle or filed as new FOLLOWUPs.

---

## §1 Locked design decisions

### Q1 — Documentation surface

**Lock (user 2026-05-20):** End-user manual (`docs/manual/`). GUI manual (`docs/manual-gui/`), SPEC/design narrative, and clap help-text drift FOLLOWUPs are out of scope.

### Q2 — Cycle scope within the end-user manual

**Lock (user 2026-05-20):** Audit the two v0.28.0 P13A/P13B files first (`docs/manual/src/45-foreign-formats.md` + `docs/manual/src/30-workflows/39-cross-format-conversion.md`). The 6 other registered manual FOLLOWUPs (`manual-v0.18-stale-md1-scenario-phrases`, `cli-manual-html-target`, `xpub-search-manual-gui-chapters`, etc.) are out of scope except where naturally folded (see Q6).

### Q3 — Audit rigor

**Lock (user 2026-05-20):** Comprehensive — every command run, every prose claim checked. Trades depth for time. Aligns with `feedback_architect_must_run_prose_commands`.

### Q4 — Transcript capture

**Lock (user 2026-05-20):** Yes — capture stdout+stderr during the audit into a new `docs/manual/transcripts/{foreign-formats,cross-format-recipes}/` subdir structure. CI hookup is part of the scope.

### Q5 — Structural approach

**Lock (user 2026-05-20):** Approach C — transcripts-as-audit. Capturing the fixture IS the audit; a transcript mismatch IS a finding. The audit pass and the infrastructure pass converge into one activity.

### Q6 — FOLLOWUP folds

**Lock (user 2026-05-20):** Fold both:

- `inheritance-example-transcript-coverage` — capture `41-inheritance.{cmd,out}` as a single composite pair driving the chapter-41 bundle + verify-bundle commands end-to-end (matches FOLLOWUP literal at `design/FOLLOWUPS.md:2046-2054` post-I2 fold; FOLLOWUP body cites `41-mnemonic.md:209-216` + `:351-357` but those are prose-context paragraphs — the actual command code blocks are pinned by P1a as its first task per §2.1 clarification fold).
- `manual-yml-bind-real-mnemonic-bin` — P3 CI wiring builds the toolkit binary fresh and sets `MNEMONIC_BIN=<built-path>`, simultaneously upgrading lint flag-coverage from no-op to real.

**Not folded** (out-of-scope explicit):

- `manual-cli-surface-mirror` — meta / always-open by design.
- `manual-v0.18-stale-md1-scenario-phrases` — different file (`docs/manual/src/30-workflows/31-singlesig-steel.md`); separate scope.
- `cli-manual-html-target`, `xpub-search-manual-gui-chapters` — distinct infrastructure / cross-stream concerns.

### Q7 — Fix-in-cycle vs FOLLOWUP boundary

**Lock (R0 proposal; subject to architect review):**

**Fix in this cycle** (rolled into the manual-v0.2.0 PR):

- Cosmetic typos, grammar, markdown lint, cspell additions.
- Documented stderr/stdout sample drift where the binary's actual output is correct and the prose is stale → update prose to match captured transcript.
- Broken intra-manual links / dead lychee refs.
- Wrong flag name or missing flag value in prose.
- Missing entries in `docs/manual/tests/cli-subcommands.list` if any.

**File a new FOLLOWUP** (deferred to a later cycle):

- Behavioral bug in the toolkit binary (a sniff signature actually accepts a format the docs say it rejects, a parse contract diverges from prose, a refusal template emits different text than documented).
- Substantive prose expansion (e.g., "this section needs taproot variant coverage").
- Newly-discovered deferral items (e.g., a parser path that's near-supported but blocked on upstream).
- Anything requiring changes outside `docs/manual/` or `.github/workflows/manual.yml`.

**Gray area** (architect locks per finding): stderr templates that have drifted between toolkit releases (could be doc-update OR a toolkit regression). Default to "doc-update unless the architect locks it as a toolkit-bug requiring a v0.28.2 toolkit patch."

### Q8 — Release shape

**Lock (R0 proposal + R1 self-correction):** Default `manual-v0.2.0` standalone (manual-only) tag. Promote to a paired `mnemonic-toolkit-v0.28.2` patch IFF P1b surfaces a toolkit-bin behavior bug per Q7 gray area. Pre-tag verification at P0: confirm latest existing `manual-v*` tag is still `manual-v0.1.10` (current state at plan-write `8977389`); if a newer `manual-v0.1.N` patch was tagged between plan-write and execution, advance the target to `manual-v0.2.0` regardless (this cycle is the minor bump). The conflated `manual-v1.x` framing in the initial plan draft was a confusion with the separate `manual-gui-v*` stream — corrected at R1.

### Q9 — Toolkit CHANGELOG.md entry

**Lock (R0 proposal):** No entry. `CHANGELOG.md` tracks toolkit versions per Keep-a-Changelog. A manual-only release does not get a top-level toolkit CHANGELOG section. Either (a) the manual itself can carry per-tag notes via the GitHub release auto-generated notes (manual.yml line 99-106), or (b) we add a brief docs/manual/CHANGELOG.md if user prefers; default is (a) unless architect / user pushes for (b).

### Q10 — Recipe fixture-path substitution (NEW R0 — folds C1)

**Problem:** The 8 cross-format recipes in `39-cross-format-conversion.md` use bare-filename `--blob` arguments (e.g., `--blob coordinator.bsms.txt`, `--blob wallet.json`, `--blob sparrow-multisig-2of3-p2wsh-sortedmulti.json`). None of these files exist in the repo under bare paths; the closest matches live under `crates/mnemonic-toolkit/tests/fixtures/wallet_import/` with different names (e.g., `bsms-shwsh-2of3.txt` for recipe 1's `sh(multi(2of3))` target). Without explicit resolution, recipe `.cmd` files cannot execute.

**Lock (R0 C1 fold):** Add a `$FIXTURES_DIR` sed-substitution to `verify-examples.sh` analogous to existing `$MNEMONIC_BIN` / `$MD_BIN` / `$MS_BIN` substitutions (line 45-48 of the current 66-line script). Each recipe `.cmd` file uses `--blob $FIXTURES_DIR/<actual-fixture-name>` paths. The documented prose in `39-cross-format-conversion.md` retains the bare-filename teaching convention (which is what a user reads). The audit-report at P2 records this prose-vs-transcript divergence as a documentation-only artifact (not a finding); the transcript byte-matches the runtime, while the prose teaches the conceptual recipe.

**Open sub-question (architect R1 to lock):** the recipe `.cmd` may also need a per-cmd `cd "$(mktemp -d)"` to isolate side-effect-emitted intermediate files (`> core-import.json`, `> envelope.json`, etc.) across recipes — see §12 risk #3 (now folded from I6).

### Q11 — Worktree-isolation invariant for all sub-agent dispatches (NEW R0 — folds I8)

**Lock (R0 I8 fold):** All architect-review and audit agent dispatches across all phases MUST include the load-bearing invariant per memory `feedback_no_parallelism_for_code_generation` Part B: agents MUST run `pwd && git rev-parse --show-toplevel` before any write operation; both outputs MUST match AND contain `.claude/worktrees/agent-` substring; otherwise the agent MUST abort. This applies even for solo dispatches (no parallel peers) because the worktree-isolation bug affects solo dispatches too at ~50% recurrence.

Persisting an architect review report to `design/agent-reports/manual-v0.2.0-P{N}-r{round}-review.md` IS a write operation; the invariant therefore applies to every reviewer round dispatched in this cycle.

---

## §2 Architectural strategy

### 2.1 Transcript layout

New subdirectories under existing `docs/manual/transcripts/`:

```
docs/manual/transcripts/
├── 22-first-bundle.{cmd,out}      (existing, 1 pair)
├── 23-verify.{cmd,out}            (existing, 1 pair)
├── 24-recover-*.{cmd,out}         (existing, 3 pairs)
├── cli-help/                      (existing — 21 .txt --help snapshots; SEE §2.2 ext rule)
│   ├── mnemonic.txt + mnemonic-{bundle,convert,derive-child,export-wallet,verify-bundle}.txt
│   ├── md.txt + md-{address,bytecode,compile,decode,encode,inspect,vectors,verify}.txt
│   └── ms.txt + ms-{decode,encode,inspect,vectors,verify}.txt
├── 41-inheritance.{cmd,out}       NEW — Q6 inheritance-example fold (1 pair, matches FOLLOWUP literal)
├── foreign-formats/               NEW DIR (6 entries — see "Capture target rationale" below)
│   ├── 45-sparrow.{cmd,out,err}                (3 files; from `### Round-trip example` at L299)
│   ├── 45-specter.{cmd,out,err}                (3 files; from L367)
│   ├── 45-coldcard-singlesig.{cmd,out,err}     (3 files; from L443)
│   ├── 45-coldcard-multisig.{cmd,out,err}      (3 files; from L526)
│   ├── 45-jade.{cmd,out,err}                   (3 files; from L590)
│   └── 45-electrum.{cmd,out,err}               (3 files; from L684)
└── cross-format-recipes/          NEW DIR (8 entries)
    ├── recipe-1-bsms-to-bitcoin-core.{cmd,out,err}      (L45)
    ├── recipe-2-bitcoin-core-to-bundle.{cmd,out,err}    (L68)
    ├── recipe-3-bsms-to-bip388.{cmd,out,err}            (L106)
    ├── recipe-4-sparrow-to-bsms.{cmd,out,err}           (L136)
    ├── recipe-5-specter-to-bitcoin-core.{cmd,out,err}   (L159)
    ├── recipe-6-coldcard-singlesig-to-bip388.{cmd,out,err} (L184)
    ├── recipe-7-jade-to-bsms.{cmd,out,err}              (L208)
    └── recipe-8-electrum-multisig-to-bsms.{cmd,out,err} (L227)
```

Total: 1 chapter-41 pair (Q6 inheritance fold, matches FOLLOWUP `inheritance-example-transcript-coverage` literal — see §8) + 6 foreign-format triples (cmd/out/err) + 8 recipe triples = **44 new files** (2 + 18 + 24).

**Capture target rationale (NEW R1 — corrects R0 over-count):** verification-pass grep of `### Round-trip example` H3 subsections in `45-foreign-formats.md` returns exactly 6 hits — Sparrow (L299), Specter (L367), Coldcard-singlesig (L443), Coldcard-multisig (L526), Jade (L590), Electrum (L684). **BSMS Round-2 (H2 at L62) and Bitcoin Core listdescriptors (H2 at L170) do NOT have `### Round-trip example` subsections** — their H2 sections are descriptive-only (Accepted shapes / Where it comes from / Audit fields / Per-entry metadata / Filtering). These two formats are exercised end-to-end via recipes 1 (BSMS→Core), 2 (Core→bundle), and 3 (BSMS→BIP-388) in `30-workflows/39-cross-format-conversion.md`; no separate chapter-45 transcript needed for them. Original plan-doc draft claimed 8 foreign-format triples (echoing the architect R0 recon which conflated H2 format count with Round-trip H3 count); this is corrected here.

**Chapter-41 inheritance capture sourcing (NEW R1 — clarification fold):** the `inheritance-example-transcript-coverage` FOLLOWUP body cites line ranges `41-mnemonic.md:209-216` (engraving-card stderr explainer) + `:351-357` (multisig.cosigners[] explainer). Verification-pass shows these are **prose-context paragraphs**, NOT the actual command code blocks. The transcript-capture target is the actual ```sh ... ``` fenced block(s) for the bundle + verify-bundle commands in chapter 41's inheritance worked example — at write time, the exact command-block line numbers were not re-grepped, but P1a verifies and pins them as the first task in that phase. The transcript drives both commands end-to-end as a single composite per FOLLOWUP intent.

**I2 fold note (R0):** plan originally proposed two pairs `41-bundle.{cmd,out}` + `41-verify-bundle.{cmd,out}`; per architect R0, this diverges from the FOLLOWUP body which literally asks for one pair `41-inheritance.{cmd,out}` driving bundle + verify-bundle end-to-end as a single composite. Locked to one pair to match FOLLOWUP literal.

**FOLLOWUP body correction (R0 fold):** the `inheritance-example-transcript-coverage` FOLLOWUP at `design/FOLLOWUPS.md:2046-2054` cites the path `docs/manual/tests/transcripts/` in its body — the actual path is `docs/manual/transcripts/` (no `tests/` infix). Cycle-close at P5 must annotate the FOLLOWUP entry with this path correction when flipping to resolved, so future readers don't propagate the wrong path.

**C2 fold note (R0):** plan originally described `cli-help/` as an empty dir. Actual state: 21 `--help` snapshot `.txt` files (no `.cmd`+`.out` shape). The §2.2 verify-examples.sh recursion rule MUST explicitly exclude or special-handle `cli-help/*.txt` so the runner does not interpret these as transcripts and false-fail.

**Triple format decision (R0 proposal):** Use `.cmd`+`.out`+`.err` triples for the new transcripts so stderr is separated from stdout. Existing 5 pairs use `.cmd`+`.out` only (stdout-with-stderr-merged via `2>&1` in verify-examples.sh line 49). Two options:

- **(a) Triple format:** New pairs use `.cmd`+`.out`+`.err`. Modify `verify-examples.sh` to detect triple form (presence of `.err`) and split. Backwards-compatible with existing 5 pairs.
- **(b) Match existing pair format:** Use `.cmd`+`.out` only with `2>&1` merge. Simpler; matches existing pattern. Loses ability to distinguish stderr discipline (which is load-bearing for refusal-template audits).

**Lock (R0 proposal, architect to confirm):** (a) Triple format. The foreign-formats chapter audit needs stderr discipline (refusal templates, audit-field drop notices, signature-verification deferral notices are all stderr-only). Existing 5 pairs are unchanged.

### 2.2 `verify-examples.sh` extension

Current state (verified in-band: 66 lines):
- Single-level `for cmd_file in "$TRANSCRIPTS"/*.cmd` iteration (line 37).
- sed-substitutes `$MNEMONIC_BIN`, `$MD_BIN`, `$MS_BIN` (lines 45-48). **No `$MK_BIN` substitution today, no `$FIXTURES_DIR` substitution today.**
- Merges stdout+stderr via `bash -c "$cmd_line" 2>&1` (line 49).
- Single `diff <(printf '%s\n' "$expected") <(printf '%s\n' "$actual")` invocation (line 53).
- The Makefile `verify-examples` target at L208-214 passes `MNEMONIC_BIN`, `MD_BIN`, `MS_BIN`, `TRANSCRIPTS` — but **NOT `MK_BIN`** (despite mk-cli being a real binary in CI per manual.yml:72-77). The extension MUST add `MK_BIN="$(MK_BIN)"` AND `FIXTURES_DIR="$(FIXTURES_DIR)"` to the Makefile target's argv too, not just to the script.

Required extensions (I4 honest re-budget):

1. **Recursive iteration:** Replace `"$TRANSCRIPTS"/*.cmd` glob with `find "$TRANSCRIPTS" -name '*.cmd' -not -path '*/cli-help/*'` (or equivalent `shopt -s globstar` pattern). The `cli-help/` exclusion is load-bearing per §2.1 C2 fold; that subdir holds `.txt` snapshots, not transcripts.
2. **Triple-format support:** For each `<base>.cmd`, check for `<base>.err`. If present, run via `bash -c "$cmd_line" >"$tmp_out" 2>"$tmp_err"`, then diff each stream against its expected file independently. Combined exit code: fail if EITHER diff fails. Backwards-compat: if no `.err`, fall back to current `2>&1` merge.
3. **`$MK_BIN` substitution:** add `-e "s#\$MK_BIN#$MK_BIN#g"` to the sed chain (line 45-48 today). Parse `MK_BIN=*` arg at line 18-25.
4. **`$FIXTURES_DIR` substitution:** add `-e "s#\$FIXTURES_DIR#$FIXTURES_DIR#g"`. Parse `FIXTURES_DIR=*` arg. Q10 lock requires this for recipe `.cmd` files.
5. **Per-cmd tmpdir (I6 fold):** Wrap each `bash -c` in a per-cmd `mktemp -d` cwd so recipe side-effects (intermediate `> envelope.json`, `> bundle.json`, etc.) don't leak across recipes. Each iteration: `tmpdir=$(mktemp -d); ( cd "$tmpdir" && bash -c "$cmd_line" ); rm -rf "$tmpdir"`.
6. **Dual-diff failure messaging:** When triple-format diff fails, print both stdout-diff and stderr-diff blocks with section headers so the CI log makes both streams' drift visible.

Honest estimated cost: **~40-60 lines net** (vs the original "~30-50" estimate). The rewrite is more invasive than first framing suggested — particularly the single-stream → triple-stream transition changes the failure-mode output format. R0 architect re-spec'd against the in-hand script (verify-examples.sh:1-66) confirms the 40-60 range.

### 2.3 Makefile changes

`docs/manual/Makefile`:

- Existing `make verify-examples` target (lines 208-214) already calls `tests/verify-examples.sh`. No target changes needed.
- **NEW**: add `verify-examples` as a dependency of a new `make audit` target (or fold into `make lint` as a 7th check). Option (a) keeps separation; option (b) makes CI single-target. **Lock (R0 proposal):** Option (a) — `make audit` umbrella target that runs `lint` + `verify-examples`. CI calls `make audit`. This keeps `make lint` as the no-binary static gate (useful for fast local iteration) and `make audit` as the full gate.

### 2.4 `.github/workflows/manual.yml` changes

Current state (per recon): line 81 calls `make lint MNEMONIC_BIN=true MD_BIN=true MS_BIN=true MK_BIN=mk`. No binary build, no verify-examples invocation.

New shape:

```yaml
- name: Build mnemonic binary
  run: cargo build --release --bin mnemonic
  # Estimated cold-build time on GH runner: 5-10 minutes

- name: Audit manual (lint + verify-examples with real binaries)
  run: |
    make audit \
      MNEMONIC_BIN="$(pwd)/target/release/mnemonic" \
      MD_BIN=true \
      MS_BIN=true \
      MK_BIN=mk
```

**Open question (architect review):** Should the cargo build use `--release` (slow, 5-10 min cold) or `--debug` (fast, 1-3 min, but produces a slightly different binary)? The `mnemonic` binary's behavior is identical between debug and release modulo timing and panic-message-detail differences. **Lock (R0 proposal):** debug build for CI (`cargo build --bin mnemonic`). Faster CI; behavior under test is identical for transcript-diff purposes; release binary is only needed for actual distribution.

**Sibling-binary handling:** `MD_BIN`/`MS_BIN` remain `true` placeholders because the v0.28.0 audit scope is the mnemonic CLI; the sibling-codec CLIs (md/ms) have their own manual chapters and their own audit FOLLOWUPs (out of scope for this cycle's audit work). `MK_BIN=mk` is already a real binary install (manual.yml lines 72-77).

**I1 fold (partial close of `manual-yml-bind-real-mnemonic-bin`):** the FOLLOWUP body at `design/FOLLOWUPS.md:2009-2011` explicitly cites all four binaries (`MNEMONIC_BIN=true MD_BIN=true MS_BIN=true MK_BIN=mk`) as the gap. This cycle closes only the mnemonic-side gap; MD/MS remain placeholder. The FOLLOWUP closure is therefore **partial**, not total. Per `v0.7.1 Batch B-2` precedent (memory entry `project_v0_7_1_batch_b2_closed`), partial closure is a legitimate FOLLOWUP closure mode IFF the carved-out remainder is partitioned into successor FOLLOWUPs filed in the same cycle.

§8 below codifies the partition: closing FOLLOWUP `manual-yml-bind-real-mnemonic-bin` with "**Status:** `resolved <COMMIT>` (partial — mnemonic-side only)" and a forward-pointer note to two new successor FOLLOWUPs (`manual-md-bin-real-binary-promote` + `manual-ms-bin-real-binary-promote`) filed at P3 (formerly P3, now P4 post-I7 phase split).

### 2.5 cli-subcommands.list

`docs/manual/tests/cli-subcommands.list` (32 entries per recon). After P3 swaps `MNEMONIC_BIN=true` for a real binary, the flag-coverage check will actually execute `--help` for each subcommand listed. Any mnemonic subcommand missing from this list will silently skip; any flag missing from the corresponding chapter will hard-fail. Audit point at P1: enumerate every `mnemonic <subcommand>` form via `mnemonic --help` and cross-check against cli-subcommands.list.

### 2.6 cspell + markdownlint

Both run today in `make lint`. P2 fix batch will add any new cspell entries needed (e.g., format names not yet in dictionary). Workflow runner already provides markdownlint-cli2 + cspell (manual.yml lines 56-63).

---

## §3 SPEC patches

None. This is a docs-and-CI cycle; no SPEC documents in `design/` need amendment.

**Document trail:** instead of a SPEC, write `design/AUDIT_REPORT_manual_v0_28_0_content.md` at P1 listing every finding (one bullet per `.cmd`+actual-vs-documented diff), classified as fix-in-cycle vs new-FOLLOWUP per Q7. This report is the audit artifact and ships in the PR.

---

## §4 Manual changes summary

Specifics are P1-driven (depend on what audit surfaces). Estimated change shapes:

| File | Estimated changes | Cause |
|---|---|---|
| `docs/manual/src/45-foreign-formats.md` | 10-50 line diffs across 8 format sections | Stderr/stdout sample drift updates per Q7; cosmetic touch-ups |
| `docs/manual/src/30-workflows/39-cross-format-conversion.md` | 5-30 line diffs across 8 recipes | Same as above |
| `docs/manual/src/40-cli-reference/41-mnemonic.md` | 0-5 line diffs | Only if Q6 inheritance-example fold surfaces drift in `bundle` or `verify-bundle` documented output |

If P1 surfaces zero drift (best case), the file diffs are empty and the audit reports "all prose accurate as of `8977389`."

---

## §5 New files

| Path | Purpose | Approx LOC |
|---|---|---|
| `docs/manual/transcripts/foreign-formats/45-*.{cmd,out,err}` × 6 | Foreign-format Round-trip transcripts (Sparrow, Specter, Coldcard×2, Jade, Electrum — BSMS + Bitcoin Core have no Round-trip subsection so are covered by recipes 1-3) | 18 files, ~30-80 LOC each |
| `docs/manual/transcripts/cross-format-recipes/recipe-*.{cmd,out,err}` × 8 | Recipe end-to-end transcripts | 24 files, ~50-150 LOC each (recipes are multi-step) |
| `docs/manual/transcripts/41-inheritance.{cmd,out}` | Q6 inheritance-example fold (composite chapter-41 bundle + verify-bundle pair, post-I2 fold to FOLLOWUP literal) | 2 files |
| `design/AUDIT_REPORT_manual_v0_28_0_content.md` | Audit findings + classification (Q7 fix-in-cycle vs new-FOLLOWUP) | ~200-400 LOC |
| `design/agent-reports/manual-v0.2.0-P{0,1a,1b,2,3,5}-r{0,1,2}-review.md` (P4 subsumed into P5 holistic; P0 has additional R2 + R3, the latter post-ExitPlanMode confirmation unpersistable from main checkout per Q11 worktree invariant) | Per-phase architect review persistence (per CLAUDE.md persist-verbatim invariant) | 6-10 files |

---

## §6 CI / infrastructure changes

| File | Change | Lines (rough) |
|---|---|---|
| `docs/manual/tests/verify-examples.sh` | Recursive iteration; `.cmd`+`.out`+`.err` triple support; backwards-compat with existing pairs | +30-50 / -5 |
| `docs/manual/Makefile` | New `make audit` umbrella target (lint + verify-examples) | +5-10 |
| `.github/workflows/manual.yml` | New `cargo build --bin mnemonic` step; line 81 swap `make lint MNEMONIC_BIN=true` → `make audit MNEMONIC_BIN="$(pwd)/target/debug/mnemonic"` | +5-10 / -1 |
| `docs/manual/tests/cli-subcommands.list` | Add any missing mnemonic subcommands surfaced during P3 real-binary flag-coverage | 0-5 lines if drift exists |

---

## §7 Phase breakdown

I7 fold (R0): P1 originally bundled mechanical transcript capture with judgment-loaded Q7 finding classification, putting too much load on a single R0 round. Split into P1a (mechanical capture) + P1b (audit-report + Q7 classification), with separate R0 per gate. Subsequent phases renumbered.

| # | Phase | Subject | Deliverables | Reviewer rounds | Est LOC | Persisted report path |
|---|---|---|---|---|---|---|
| P0 | Setup + recon-grounding | Confirm latest `manual-v*` tag is still `manual-v0.1.10` (current at plan-write); confirm SHA `8977389` is current master; lock plan-doc citations; pre-build mnemonic binary locally (debug) for P1a use; vendor inventory of `tests/fixtures/wallet_import/` per recipe needs; grep + pin chapter-41 inheritance command-block line numbers (per §2.1 clarification fold — the FOLLOWUP-cited :209-216 and :351-357 are prose-context, NOT command blocks) | tag-confirmed-latest, binary-built, fixture-recipe-mapping table, chapter-41 command-block line-numbers pinned, plan-doc R0/R1/R2/R3 reviewer-loop converged | R0 + R1 + R2 + R3 (R0/R1 done at plan-write; R2 + R3 confirmation rounds run, R3 post-ExitPlanMode per user direction — see §13) | 0 (planning only) | `design/agent-reports/manual-v0.2.0-P0-r{0,1,2,3}-review.md` (R3 unpersistable from main checkout per Q11 worktree invariant; returned as agent-output only) |
| P1a | Transcripts capture (mechanical) | For each documented command block in chapters 45 (6 Round-trip examples — Sparrow/Specter/Coldcard×2/Jade/Electrum) + 30/39 (8 recipes) + chapter 41 (1 inheritance composite) — run **directly via `bash -c "$cmd" >out 2>err` in a per-cmd `mktemp -d` cwd against the P0-built binary**, NOT via verify-examples.sh (the script's triple-format support is a P3 deliverable; P1a has no script dependency). **Round-trip invariant:** P1a tmpdir layout MUST mirror the P3 script's per-cmd tmpdir model (fresh `mktemp -d` per command, fixtures referenced via `$FIXTURES_DIR` substitution) so when P3's extended script replays a P1a-captured transcript, the .out/.err diff is byte-identical at first-CI-run. Persist captured stdout to `.out`, stderr to `.err`, command to `.cmd`. Architect re-runs ~25% by hand for sample verification. | 44 new files (6×3 + 8×3 + 1×2) | R0 (architect samples reproducibility — re-runs ~25% of transcripts independently by hand-replay) | +1000-2000 (mostly captured fixture content) | `design/agent-reports/manual-v0.2.0-P1a-r0-review.md` |
| P1b | Audit-report + Q7 classification (judgment) | Diff each captured transcript against the documented prose block in `45-foreign-formats.md` / `39-cross-format-conversion.md` / `41-mnemonic.md`; produce `design/AUDIT_REPORT_manual_v0_28_0_content.md` with finding-by-finding Q7 classification (fix-in-cycle vs new-FOLLOWUP); record proposed prose patches inline for P2 reference | `AUDIT_REPORT_manual_v0_28_0_content.md` (~200-400 LOC) with N findings, each tagged fix-in-cycle / new-FOLLOWUP / gray-area-architect-locks | R0 (architect verifies each Q7 classification + citation accuracy finding-by-finding) | +200-400 | `design/agent-reports/manual-v0.2.0-P1b-r0-review.md` |
| P2 | Fix-in-cycle batch | Apply prose updates for every P1b fix-in-cycle finding; file new FOLLOWUPs in `design/FOLLOWUPS.md` for every deferred finding; resolve cspell + markdownlint failures | Diffs to `docs/manual/src/{45-foreign-formats.md,30-workflows/39-cross-format-conversion.md}` (+ `41-mnemonic.md` if chapter-41 audit surfaces drift); N new FOLLOWUP entries | R0 + R1 (architect re-grep verifies citations; runs `make audit` locally) | +50-300 / -50-300 (prose) | `design/agent-reports/manual-v0.2.0-P2-r{0,1}-review.md` |
| P3 | CI wiring | Extend `verify-examples.sh` for triples + recursion + `$MK_BIN` + `$FIXTURES_DIR` substitutions + per-cmd tmpdir (per §2.2); add `make audit` umbrella target (per §2.3); modify `.github/workflows/manual.yml` to `cargo build --bin mnemonic` + invoke `make audit` with real `MNEMONIC_BIN=$(pwd)/target/debug/mnemonic FIXTURES_DIR=$(pwd)/crates/mnemonic-toolkit/tests/fixtures/wallet_import`; verify locally before commit; lint via `actionlint` | Changes per §6 table | R0 (architect: YAML correctness check per `feedback_r2_blocking_vs_cosmetic_gate`; run `actionlint` against manual.yml) | +50-80 | `design/agent-reports/manual-v0.2.0-P3-r0-review.md` |
| P4 | New successor FOLLOWUPs (I1 partition) | File `manual-md-bin-real-binary-promote` + `manual-ms-bin-real-binary-promote` in `design/FOLLOWUPS.md` as successor entries to the partial close of `manual-yml-bind-real-mnemonic-bin`. Cite their forward-pointer from the partial-close note. | 2 new FOLLOWUP entries | R0 (folded into P5 holistic — no separate review) | +30-50 | (subsumed into P5 report) |
| P5 | Cycle close | Update `design/FOLLOWUPS.md` Status flips for closed Q6 folds (per `feedback_per_phase_agents_forget_followup_status_flip` — closure for `inheritance-example-transcript-coverage` + partial closure for `manual-yml-bind-real-mnemonic-bin`); CHANGELOG decision per Q9; end-of-cycle holistic architect review; tag `manual-v0.2.0`; verify GH Release attaches PDF | FOLLOWUPS Status flips; end-of-cycle agent report; tag pushed; GH Release live with PDF | R0 holistic | +20-40 | `design/agent-reports/manual-v0.2.0-P5-end-of-cycle-review.md` |

**Total estimated effort:** ~1 working day of focused work + 1-2 hours of reviewer-loop folds. The split P1a/P1b gates 2 separate R0 rounds (more rigorous than original combined P1 single R0).

---

## §8 FOLLOWUP folds + new filings

**Closed at this cycle (Q6 folds):**

- `inheritance-example-transcript-coverage` — **Resolved (total).** Closed at P1a via `docs/manual/transcripts/41-inheritance.{cmd,out}` (1 pair matching FOLLOWUP literal, post-I2 fold). Status flip in P5 with commit-SHA pin + path-correction annotation noting the FOLLOWUP body's `docs/manual/tests/transcripts/` → `docs/manual/transcripts/` path correction (R0 I2 fold).
- `manual-yml-bind-real-mnemonic-bin` — **Resolved (partial — mnemonic-side only).** Closed at P3 via `cargo build --bin mnemonic` + `MNEMONIC_BIN=$(pwd)/target/debug/mnemonic` swap. MD/MS binaries remain `=true` placeholder. Per `v0.7.1 Batch B-2` partition-rationale precedent (memory entry `project_v0_7_1_batch_b2_closed`), partial closure is a legitimate FOLLOWUP closure mode IFF the carved-out remainder is partitioned into successor FOLLOWUPs filed in the same cycle. Status flip in P5 with explicit forward-pointer to successors below.

**Definite new FOLLOWUPs filed at P4 (I1 partition successors — NOT speculative):**

- `manual-md-bin-real-binary-promote` — promote `MD_BIN=true` to a real `md` binary in `manual.yml`, parallel to mk-cli's existing install pattern at `manual.yml:72-77`. Tier: `v0.20+-ci-hygiene` (mirrors parent FOLLOWUP). Body cross-references `manual-yml-bind-real-mnemonic-bin` partial-close.
- `manual-ms-bin-real-binary-promote` — same as above for `MS_BIN=true`. Tier and body convention identical.

**Speculative new FOLLOWUPs filed at P2 (audit-report-driven; specifics depend on P1b findings):**

- Per-format `manual-stderr-template-drift-<format>` — filed per finding if Q7-gray-area resolves as "doc update covers it for now, but the underlying stderr template's wording could be tightened."
- Other surface-specific entries surfaced during P1b classification (unknown at plan-write time).

**Not closed; remain open:**

- `manual-cli-surface-mirror` (meta; always-open by design)
- `manual-v0.18-stale-md1-scenario-phrases` (out of scope per Q2; different file `31-singlesig-steel.md`)
- `cli-manual-html-target`, `xpub-search-manual-gui-chapters`, `electrum-final-seed-version-drift`, etc. (out of scope per Q2)

---

## §9 Verification

End-to-end verification at P5 (before tag):

1. **Clean checkout test:** `git clean -fdx && cargo build --bin mnemonic && make -C docs/manual audit MNEMONIC_BIN="$(pwd)/target/debug/mnemonic" MD_BIN=true MS_BIN=true MK_BIN=mk FIXTURES_DIR="$(pwd)/crates/mnemonic-toolkit/tests/fixtures/wallet_import"` — must exit 0.
2. **Transcript-tamper test:** edit one `.cmd` file to corrupt the command, re-run `make audit`, confirm failure with helpful diff output; revert.
3. **PDF still builds:** `make -C docs/manual pdf MERMAID_FILTER=skip` produces `docs/manual/build/m-format-manual.pdf` without errors.
4. **Tag push:** verify `.github/workflows/manual.yml` fires on `manual-v0.2.0` tag, builds PDF, attaches to GH Release, exits all jobs green.
5. **Drift-gate fires correctly:** after tag CI, intentionally introduce a prose drift (e.g., change a stderr line in `45-foreign-formats.md`) on a feature branch, open PR, confirm CI fails on the verify-examples diff. Revert the test branch.

---

## §10 Reviewer-loop discipline

Per CLAUDE.md and project memory (`feedback_plan_artifact_mirror_project_convention`, `feedback_opus_primary_review_agent`, `feedback_r0_must_read_source_off_by_n`, `feedback_no_parallelism_for_code_generation`):

- **Plan-doc reviewer-loop:** this file gets at least R0 + R1 (R0 + post-fold R1 minimum since R0 returned 3C/8I) from `feature-dev:code-reviewer` agent with `model: "opus"` before ExitPlanMode. Reviewer must verify Q1-Q11 lock content against current FOLLOWUPS + against re-grepped citations.
- **Per-phase reviewer-loop:** every phase persists its review report verbatim to `design/agent-reports/manual-v0.2.0-P{N}-r{round}-review.md` BEFORE the fold-and-commit step (per CLAUDE.md "agent-review outputs persist verbatim" invariant).
- **P1a special discipline:** the architect reviewer at P1a R0 must SAMPLE-run a subset of captured transcripts independently (per `feedback_architect_must_run_prose_commands`); cannot reviewer-loop on transcript text alone.
- **P1b special discipline:** the architect at P1b R0 verifies each Q7 classification finding-by-finding (not in aggregate); per `feedback_r0_must_read_source_off_by_n` recurrence pattern, judgment-loaded steps need finding-level review.
- **Worktree-isolation invariant (Q11 lock):** every architect-review and audit-agent dispatch in this cycle MUST include the load-bearing brief: agents run `pwd && git rev-parse --show-toplevel` before any write (including persisting the review report to `design/agent-reports/...md`); both outputs must match AND contain `.claude/worktrees/agent-` substring, else ABORT. This applies to solo dispatches too per `feedback_no_parallelism_for_code_generation` Part B (~50% recurrence of the worktree-isolation bug on solo agents).
- **End-of-cycle holistic R0:** P5 dispatches one final architect review covering the whole cycle's diffs (per `feedback_per_phase_agents_forget_followup_status_flip` — verify Status flips happened on every closed FOLLOWUP, including the partial-close annotation on `manual-yml-bind-real-mnemonic-bin`).

---

## §11 Out-of-scope / explicitly deferred

- All other documentation FOLLOWUPS (~18 not folded per Q6).
- GUI manual (`docs/manual-gui/`) — separate stream.
- SPEC / design narrative drift slugs.
- clap help-text drift slugs.
- Sibling-codec manual chapters (`docs/manual/src/40-cli-reference/4{2,3,4}-*.md`) — same audit pattern could apply but is out of scope; file a `manual-audit-sibling-cli-chapters` FOLLOWUP at P2 if architect agrees the pattern should propagate.
- Toolkit binary behavior bugs (if any surface; covered by Q7 gray-area → `mnemonic-toolkit-v0.28.2` patch IFF needed).
- `crates.io` publish — still blocked on miniscript `[patch.crates-io]` per memory; not touched by this docs-only cycle.

---

## §12 Risks + open questions

1. **CI time impact:** adding `cargo build --bin mnemonic` to manual.yml adds 5-10 min cold / 1-3 min cached. Manual builds currently run on docs-touching PRs only (manual.yml line filter); adding a build step makes every docs PR slower. Mitigation: use `actions/cache@v4` for cargo registry + target dir. Decision deferred to P3 R0.
2. **Multi-step recipe transcript stability:** recipes pipe stdin/stdout through multiple commands. Capturing as a single multi-line `.cmd` (per existing `bash -c` invocation in verify-examples.sh) is feasible, but stderr ordering across pipelined steps may be nondeterministic. Mitigation: P1a captures with explicit `2>>recipe-N.err` per step. If nondeterminism surfaces, decompose into per-step transcripts.
3. **Recipe write-side-effects + cwd-mutation (R0 I6 fold):** recipes 1-8 write intermediate files to cwd (`> core-import.json`, `> envelope.json`, `> bundle.json`, `> policy.json`, `> coordinator.bsms.txt`). If verify-examples.sh runs all `.cmd` files from a shared cwd, recipe N+1's input may be a leftover from recipe N's output — false-pass risk. Mitigation locked in §2.2 item 5: per-cmd `mktemp -d` cwd isolation. P3 architect R0 verifies the isolation is durable across all 8 recipes.
4. **Binary path portability:** `MNEMONIC_BIN=$(pwd)/target/debug/mnemonic` is workflow-relative. If a developer runs `make audit` from a different cwd, the env var must be set explicitly. Mitigation: documented in the audit target's help text + the README's testing section.
5. **rustc-version stderr portability (R0 I5 fold):** captured stderr may not be byte-stable across rustc toolchain versions IF any documented flow exits via panic (panic-handler text leaks rustc version). The mnemonic binary should never panic in normal operation; failure paths use library-controlled error messages. But: refusal templates that print via `eprintln!`/`anyhow::Error::Display` are library-controlled and byte-stable; panics aren't. Mitigation: P1a captures with a fixed rustc version (the one in `rust-toolchain.toml` if present, else system cargo). P3 CI uses `actions-rust-lang/setup-rust-toolchain` with explicit version to match. Document this constraint in the captured `.cmd` files' header comments.
6. **Architect availability for P1a/P1b R0:** the split-phase R0 doubles the architect dispatch count for the audit phase (vs the original single P1 R0). Mitigation: both R0 rounds use `model: "opus"` per `feedback_opus_primary_review_agent`; P1a R0 brief is mechanical-reproducibility-only (lower cognitive load), P1b R0 brief is judgment-classification-only (higher cognitive load but limited scope).

---

## §13 Pre-execution checklist

Before Phase 0 dispatch:

- [x] Architect R0 review of this plan-doc dispatched (opus, 2026-05-20, 3C/8I returned).
- [x] R0 folds applied inline (C1 Q10 fixture-substitution; C2 cli-help/ + tree-comment; C3 SHA d18787f; I1 partial-close partition; I2 single 41-inheritance pair; I3 15-not-16; I4 honest re-budget; I5/I6 §12 risks; I7 P1a/P1b split; I8 Q11 worktree invariant; plus R1-self-correction folds: manual-v0.1.x stream, 6-not-8 Round-trip examples, verify-examples.sh 66 lines, 39-conversion predecessor 77ebfca, Makefile MK_BIN gap, chapter-41 prose-vs-command ranges).
- [x] R1 architect review dispatched (opus, 2026-05-20); returned 0C/4I; all 4 folded (I-R1-A 66-line two stale sites; I-R1-B 16-pairs §0 L27; I-R1-C P1a phase-ordering — locked as direct `bash -c` capture, no script dep; I-R1-D Q8/P0 stale v1.0.1 → v0.1.10).
- [x] R2 confirmation review dispatched (opus, 2026-05-20); returned 0C/1I/2M; folded inline (N1 §0 L26 "8 Round-trip" → 6; N2 round-trip replay invariant added to §7 P1a; N3 this R2 checklist line).
- [x] User approved plan via ExitPlanMode (2026-05-20).
- [x] R3 architect confirmation dispatched post-ExitPlanMode (opus, 2026-05-20); returned 0C/0I + 1M; the 1 Minor (§7 P0 row + §5 L262 R3-acknowledgement gap) folded inline at L283 + L262. R3 worktree-isolation check FAILED (architect dispatched from main checkout, not a worktree); architect correctly refused to persist its report under `design/agent-reports/` and returned findings as agent-output text only — process gap noted for future post-ExitPlanMode dispatches.
- [x] **Plan locked at R3 0C/0I.** Ready for P0 execution (whenever user schedules).
