## R0 Review — Cycle-C FULL doc-fidelity PLAN

**VERDICT: GREEN (0 Critical / 0 Important / 5 Minor).** Cleared to begin implementation at P0. The five Minors are census-precision and intended-diff-disclosure nits that the existing per-phase R0/TDD gates absorb; none block start.

All load-bearing claims were re-grepped LIVE at `c630c933` (v0.72.0). The PLAN is unusually well-grounded — the R0-fold corrections it already carries (4 runner forks not 3, MK-blind manual-gui, two independent filter chains, two-tier pins, error-detection-scope honesty) all reproduce against live source.

### Axis-by-axis adjudication of the review prompt

**(a) Output-block census + determinism for entropy-derived output — SOUND.**
- Census verified live: manual text 79 / json 36 (line-anchored), quickstart 10/0, tech-manual 37/14, manual-gui 36/2. Matches the PLAN exactly. (One unanchored-grep off-by-one → Minor #1.)
- The entropy-determinism worry raised in the prompt is the central risk, and the PLAN resolves it correctly. There IS a fixed-seed convention: the canonical `abandon…about` seed (fp `73c5da0a`, NEVER-FUND) + the `ms10entrsq…` vector. I verified live that this seed produces byte-stable output: `24-recover-mk1.out` pins `xpub6CatWdiZiodmU…` and fp `73c5da0a` deterministically. BIP-32 derivation is fully deterministic from a fixed seed → addresses/xpubs ARE binary-identical across runs. The DETERMINISM section's hard rule (fixed public vectors only; a real-seed capture would persist secret-bearing `.out`) is the correct safety posture. **The strategy is sound.**

**(b) Will the CI gate catch drift WITHOUT false-failing on env differences — YES, with the version-normalization correctly narrowed.**
- The two distinct mechanisms (M1 build-time include = masks prose↔golden divergence; M2 verify-examples replay = the SOLE drift detector) are correctly separated. Confirmed live that M1 has zero false-fail surface from env: the goldens are pure ASCII + seed-derived, no `$HOME`/`$RANDOM`/`date`/PID interpolation (the secret-on-argv advisory is a literal, byte-stable).
- The version-literal false-fail vector is correctly neutralized: I verified all `Pinned: mnemonic 0.13.0` sites — they are inside EXCLUDED ASCII mockups (`31-first-launch.md:21`, `33-help-icons-and-deep-links.md:37`) or PROSE that `verify-examples` never sees (`31-first-launch.md:40`, `41-overview.md:62`, + 12 broader `0.13` files confirmed). The `--version` echo reaching a transcript is ~0 surface today → the symmetric-sed normalizer is correctly scoped to near-nothing rather than over-applied. **No false-fail on paths/versions/banners.**
- Build-banner (`99-build-banner.md`, prose, PDF-only, `GIT_SHA`/`BUILD_DATE`-injected) is correctly on the exclusion allow-list — that is the one genuine env-varying surface and it is fenced out.

**(c) Phasing executable + correctly sized — YES.**
- The ~95/36/10/27 per-book conversion split is realistic and matches the live output-block counts (115/51/10/38 minus goldens/excludes). The largest single chunk (manual ~95) is explicitly sub-batched 2-3 ways.
- The standout phasing decision is splitting P1a (tech-manual golden DISCOVERY, unknown-sized) from P1b (convert, fixed-count). This is correct: the 15 never-CI-replayed goldens (11 CLI + 4 cargo-example, confirmed `md-codec-examples` with 4 `[[example]]` targets) are the one phase with real bug-surfacing potential, and bundling discovery with bulk-conversion would let unknown breakage blow the phase estimate. The triage rule ('doc-was-stale' vs 'binary-regressed' BEFORE repairing, no blind-overwrite) correctly guards the L4 false-completion pattern.

**(d) Composes the two orthogonal gates correctly — YES.**
- Axis 1 (output-fidelity) and Axis 2 (manual-gui v1.1 anchor-coverage, the R0-GREEN `SPEC_cycleC_manualgui_v1_1_P0.md`) are kept deliberately unmerged. The explicit instruction NOT to fold output-fidelity into Axis 2's SPEC (which would re-open its GREEN R0) is the right call.
- On manual-gui specifically the two gates ARE on the same book but disjoint surfaces: gui-schema-coverage (phase 4 of lint.sh) vs the new verify-examples (proposed 8th phase). Confirmed live they are orthogonal. The RED-window management is correct: add the 8th phase ONLY in the commit that lands passing transcripts, keeping the branch's RED window bounded to the gui-schema pin-bump as the SPEC plans. The N/7→N/8 step-label renumber (lines 56/64/76/84/99/113/130 verified exact) is captured as Minor #5 in the plan.

**(e) PDF + glyph ripple — SOUND.**
- Verified the two filter chains are INDEPENDENT variables: `MD_FILTER_ARGS` (Makefile:80, strip-latex + primer-box; used for md AND html) and `PDF_FILTER_ARGS` (Makefile:81, primer-box + wrap-long-code). The P0 acceptance correctly requires prepending the include filter to BOTH, before strip-latex and before wrap-long-code respectively — not treating them as one ordered chain. The `filter-smoke` PDF-path assertion (long m-string wraps via wrap-long-code, which runs ONLY in the PDF chain) is the right targeted check.
- Glyph pipeline: confirmed the `.out` bodies are ASCII + em-dash (DejaVu Mono-safe); the only non-Mono glyphs (`▾`/`?`/`☑`/box-drawing) live exclusively in the EXCLUDED mockups (3 files: 31/32/33-tour). So regenerating output blocks introduces NO new glyph-coverage requirement. The vendored codex32 PDF is correctly frozen-untouched.

**(f) The 'double-duty error-detection' claim — HONESTLY SCOPED, not over-claimed.**
- This is where the plan is strongest. It does NOT claim the ~150 net-new captures surface bugs. It explicitly states (the C′ / Important #2 fold) that a net-new capture PINS current output as baseline and CANNOT retroactively detect a pre-existing wrong-output bug — and I confirmed the predicate live: `lint.sh` has 0 `.out` references and no filter compares prose to golden, so there is genuinely NO oracle at capture time. Bug-surfacing is correctly concentrated in exactly two places: P1a's repair of the 15 never-replayed tech-manual goldens (real drift potential — committed `.out` may already diverge from the current binary) and all blocks going-forward under M2 (future-regression trip). This is the correct, non-inflated framing.

### Additional live confirmations (not findings — corroboration)
- 4 runners confirmed: manual = canonical (4 MK_BIN hits), quickstart = symlink → manual's, tech-manual = non-recursive but HAS MK_BIN, manual-gui = SEPARATE fork, 0 MK_BIN hits, non-recursive (`"$TRANSCRIPTS"/*.cmd`, no `find`/`mktemp`). manual-gui transcripts/ = `.gitkeep`-only.
- manual-gui Makefile verify-examples target (lines 230-235) passes MS_BIN but NO MK_BIN — even though MK_BIN is defined (line 45) and passed to a DIFFERENT target (line 254). The P4 3-part remediation (symlink runner + insert MK_BIN + workflow arg) is correctly targeted.
- CI stubs confirmed: quickstart.yml:77 / technical-manual.yml:82-86 / manual-gui.yml:106 all run `make lint` with `*_BIN=true`/`MK_BIN=mk` → zero error-detection today; manual.yml runs `make audit` = `lint verify-examples anchor-check` (line 266) with the real built `mnemonic` binary → manual is correctly identified as the ONLY currently-gated book.
- Two-tier pins confirmed: current-release tier (manual.yml:79/86/90 = mk-cli-v0.10.2 / md-cli-v0.9.2 `--features cli-compiler` / ms-cli-v0.11.0); gui-pinned tier (SPEC:30-33 = toolkit-v0.70.0 / md-cli-v0.7.0 / ms-cli-v0.8.0 / mk-cli-v0.9.0). Forcing identical pins would break one gate — the per-book-self-gate resolution is correct.
- PE PDF-release contradiction CONFIRMED REAL: `technical-manual.yml` is `on: push:` only (no `tags:`, no `gh release upload`) and its own comment (line 15) says 'ships no release asset' — directly contradicting `CHANGELOG.md:7` which asserts the PDF 'ships as a GitHub release asset'. PE remediation is justified.
- The canonical excerpt-drift case CONFIRMED: `22-first-bundle.out` line 1 carries `warning: secret material on argv …`; `22-first-bundle.md` has 0 hits. The M1 whole-include-vs-(b)-subset-gate per-block call is real, not theoretical.

### FORKS — recommendations
1. **Version-determinism**: keep the tiny version-echo sed (near-zero surface); the real work is P3 hand-sweep of the 4 literal sites + 12 broader `0.13` files. Concur.
2. **Gate granularity = per-book self-gate**: correct and forced by the two-tier model. Concur.
3. **manual-gui fidelity as fast-follow P4 (not folded into P3 SPEC)**: correct — preserves the GREEN R0. Concur.
4. **manual-gui pin target v0.49.0 (toolkit-v0.70.0 era)**: the SPEC pins v0.49.0 while live GUI is v0.48.0/toolkit-v0.72.0. The plan correctly flags this as a confirm-at-execution fork — a newer GUI tag would re-derive the +506 anchor delta AND shift the gui-pinned tier. **Recommend resolving this fork BEFORE P3 starts** (it is the one input that could invalidate Axis-2 work mid-flight), but it does not block P0/P1/P2 which are Axis-1 only.

### Gate decision
0 Critical / 0 Important → **GREEN**. The 5 Minors are precision/disclosure refinements folded into P0/P2/P4 acceptance; the per-phase R0+TDD loop absorbs them. Cleared to begin P0.