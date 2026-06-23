## R0 Adversarial Review — LANE4-pincheck spec (`SPEC_sibling_pin_check_prose_scan_extension.md`)

**VERDICT: GREEN — 0 Critical / 0 Important / 2 Minor. Cleared to implement.**

Source verified against `origin/master` = `cc9f9dc27f30c234ea4bf434fa883dc6be198408` (== HEAD). Every cited path/line re-grepped live; the proposed workflow body was extracted and run end-to-end; actionlint executed on the proposed file.

### Claims verified empirically (not asserted)

| Claim | Method | Result |
|---|---|---|
| Source SHA `cc9f9dc2` == HEAD == origin/master | `git rev-parse` | CONFIRMED |
| Current regex does NOT match `--bin` prose line | `grep -nE` on `44-mk-cli.md` with OLD detector | CONFIRMED NO MATCH (the `--bin` trailing token fails `[a-z][a-z0-9_-]*`) |
| New url-keyed detector matches prose `--bin` form | run proposed body, Trial 1 | `OK docs/manual/src/40-cli-reference/44-mk-cli.md:12: mk-cli-v0.10.2` |
| Clean tree → exit 0 | Trial 1 | exit 0, 6 OK lines (cross-tool:50, manual:79/86/90, quickstart:73, + prose:12) |
| Synthetic prose drift → exit 1 + `::error::` | Trial 2 (sed `v0.10.2`→`v0.9.0`, run, restore) | exit 1; error string **byte-for-byte** matches the spec's asserted text; tree restored clean (`git status --short` empty) |
| Tagless prose NOT flagged (no false-positive) | Trial 3 + direct grep on 8 tagless/`--locked`/`.git` lines | NONE appear in output; detector requires BOTH `--git url` AND `--tag` |
| actionlint clean (heredoc gotcha avoided via `mds=$(find …)`) | `actionlint 1.7.12` on proposed file | exit 0, no output |

### Citation accuracy (all live-verified)
- `install.sh` component_info arms @ `:32/35/38/41/44` — md-cli `descriptor-mnemonic-md-cli-v0.7.1`, ms-cli `ms-cli-v0.11.0`, mk-cli `mk-cli-v0.10.2`, gui `mnemonic-gui-v0.49.0`, self `mnemonic-toolkit-v0.71.0`. EXACT.
- FOLLOWUPS b2 header `:4047`, status `:4054` (verbatim `open (gate gap unaddressed)` prefix present exactly once → flip Edit will land); b1 header `:573`, status `:579` (`resolved`, confirm-only); b3 header `:278`. ALL EXACT.
- Workflow surgical sites: CANONICAL awk `:60-62`, `canonical_tag_for` `:74-77`, `for wf` `:82`, pkg-extract `:92-93`, detector `:109`, loop `done` `:110`. ALL EXACT.
- `44-mk-cli.md:12` == canonical (`mk-cli-v0.10.2`); `:393` is a non-`--git` mention. EXACT.

### Section 5 cascade — independently re-grepped, fully accurate
- `sibling-pin-check.yml` RE-FIRES (`on: push` no paths). All others (`manual.yml` `docs/manual/**`, `quickstart.yml` `docs/quickstart/**`, `cross-tool-differential.yml` src-scoped, `rust.yml` `crates/**`, `fuzz-smoke.yml`, `technical-manual.yml`, `manual-gui.yml`, `bitcoind-differential.yml` all src/docs-scoped; `install-pin-check.yml`/`changelog-check.yml` tag-only) — NONE have a `paths:` matching `sibling-pin-check.yml` or `design/FOLLOWUPS.md`, so the pin-neutral CI-only diff trips nothing. The "do NOT bump any prose pin (would trip manual.yml forward-only flag-coverage)" constraint is sound and correctly flagged.

### Robustness probes (adversarial)
- **Self-exclusion**: toolkit `mnemonic-toolkit` correctly absent from the url-keyed table (verified by running the new awk); `sibling-pin-check.yml` self-skipped by basename.
- **Key-collision**: all 4 sibling/gui repo-urls are UNIQUE (`uniq -d` empty) → url-key is unambiguous; `awk … exit` deterministic.
- **Scan-set leakage**: `design/FOLLOWUPS.md` + the SPEC (which now contain `cargo install --git … --tag` example strings) are in `design/`, OUTSIDE the scan set (`.github/workflows/*.yml` + `docs/{manual,quickstart}/src/**.md`) → no self-poisoning.
- **GUI neutrality**: NO `mnemonic-gui` tagged cargo-install line in any scanned file (and `manual-gui.yml` has no `cargo install` at all) → GUI-in-table is genuinely latent/neutral. (One precision nit on the spec's *explanation* of this — see Minor #1.)
- **Perf**: 60 `.md` files scanned — trivial.

### Minor findings (non-blocking — GREEN stands)
1. §2.1's comment-guidance prose mis-attributes the GUI behavior delta to table membership; the GUI pkg was in the OLD pkg-keyed table too, so the only real delta is the `--bin` prose form. Bottom-line conclusion ('neutral') is correct. Reword the in-workflow comment.
2. Two latent (zero-instances-today) url-matching gaps not noted: a future `.git`-suffixed tagged url would warn-not-gate (and quickstart `21-install.md:38` already uses a `.git` tagless line, a plausible future foothold); a `cargo install --locked --git … --tag` line breaks `cargo install --git` adjacency. Optional hardening; out of current scope.

### Gate disposition
0C/0I. The spec is implementation-ready: surgical sites exact, trials reproducible and proven, actionlint clean, cascade airtight, SemVer NO-BUMP justified, FOLLOWUP flip target verbatim-present. The two Minors are cosmetic/forward-looking and explicitly do not block code per the R0 gate (GREEN = 0C/0I). Proceed to single-TDD-implementer; persist this review to `design/agent-reports/sibling-pin-check-prose-scan-R0-review.md` before fold-and-commit; whole-diff adversarial review post-impl.