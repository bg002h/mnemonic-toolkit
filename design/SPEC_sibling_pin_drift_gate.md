# SPEC — sibling-cli pin drift gate (`sibling-pin-check.yml`)

**Cycle:** Cycle B of the post-v0.37.8 cluster (`manual-yml-sibling-pin-vs-install-sh-drift-gate`).
**Tier:** `v0.37+-ci-hygiene`.
**SemVer:** test/CI-only — does not bump toolkit version.
**Source SHA at write time:** `origin/master` = `6ae7372` (post-v0.37.8 ship).

## 1 — Problem

`scripts/install.sh` is the **canonical** source of sibling-CLI pins (`md-cli`, `ms-cli`, `mk-cli`) for the m-format constellation install path. Multiple CI workflows independently re-pin the same siblings via `cargo install --git … --tag <tag>` for their own builds. The pins MUST stay synchronized — drift means the local CI run installs a different binary than what the curl|sh install path produces.

**Parent FOLLOWUP** (`manual-yml-and-install-sh-sibling-gui-pin-staleness`) fixed the v0.36.4 symptom (manual.yml siblings 11 versions stale; quickstart.yml 1 version stale). Today the pins match across all 3 files. But no static gate enforces this — they can drift again silently. Per `feedback_fix_the_class_hunt_for_second_instance.md`: prefer a gate over hand-fixed instances.

**Authority model (out of scope for this cycle):** `scripts/install.sh` is canonical. Workflows mirror it. Inverting the authority is a separate design discussion (the gate doesn't care which direction is canonical; it only enforces equality).

## 2 — Citations (verified against `origin/master` SHA `6ae7372`)

### 2.1 Canonical pins (install.sh)
- `scripts/install.sh:35` — `md-cli|…|descriptor-mnemonic-md-cli-v0.6.1|yes|cli-compiler`
- `scripts/install.sh:38` — `ms-cli|…|ms-cli-v0.4.1|yes|`
- `scripts/install.sh:41` — `mk-cli|…|mk-cli-v0.4.2|yes|`

### 2.2 Mirror sites (workflows)
- `.github/workflows/manual.yml:77` — `… --tag mk-cli-v0.4.2 mk-cli`
- `.github/workflows/manual.yml:84` — `… --tag descriptor-mnemonic-md-cli-v0.6.1 md-cli --features cli-compiler`
- `.github/workflows/manual.yml:88` — `… --tag ms-cli-v0.4.1 ms-cli`
- `.github/workflows/quickstart.yml:71` — `… --tag mk-cli-v0.4.2 mk-cli`

(Quickstart installs only `mk-cli` — the lint mocks `MD_BIN=true MS_BIN=true` per `:75`. md/ms install steps not present.)

### 2.3 Model gate to mirror
- `.github/workflows/install-pin-check.yml` (toolkit self-pin gate; fires on `mnemonic-toolkit-v*` tag push). The new gate mirrors its structure for sibling pins but fires on EVERY push + PR (lagging-on-tag would defeat the purpose since siblings can drift before any toolkit tag).

## 3 — Design

### 3.1 New file: `.github/workflows/sibling-pin-check.yml`

Trigger:
- `push` on every branch
- `pull_request` against `master`
- `workflow_dispatch` (manual)

Single job `sibling-pins-match-install-sh` running on `ubuntu-latest`:
1. `actions/checkout@v4`.
2. One bash step that:
   - Extracts each sibling's canonical tag from `scripts/install.sh` using `grep -oE` against the pipe-delimited component-info echo lines. The result is a **dynamic** keyed table `{pkg → tag}` parsed at run-time from install.sh's `case` arms — NOT a hard-coded list in the workflow.
   - For each `.github/workflows/*.yml` file (other than `sibling-pin-check.yml` itself), greps for every `cargo install --git … --tag <tag> <pkg>` invocation.
   - For each mirror pin found, looks up the canonical pin **by exact `<pkg>` match** against the parsed install.sh table. If `<pkg>` is in the table: compare `<tag>` to canonical; on mismatch emit `::error::sibling-pin-check: <workflow-file>:<line>: <pkg> pin '<actual-tag>' does not match scripts/install.sh canonical '<canonical-tag>'` and set the job to fail. If `<pkg>` is NOT in the table: emit `::warning::sibling-pin-check: <workflow-file>:<line>: unknown sibling '<pkg>'; add a component_info arm in scripts/install.sh to gate this pin` and continue (forward-compat: unknown sibling does not fail; it surfaces visibly).
   - Exits 0 when every sibling pin found in every workflow matches install.sh's canonical pin.

### 3.2 Authority pattern

`install.sh` is sole canonical source. Gate is one-way: mirror sites MUST equal canonical. This matches the existing `install-pin-check.yml` orientation (the toolkit's own tag drives install.sh's self-pin, then both drive the gate; install.sh is canonical for sibling pins).

### 3.3 Recovery message

On drift the gate emits an actionable message: which workflow line drifted + what install.sh says + the bash one-liner to fix (`sed -i 's/<old>/<new>/' <workflow-file>`). Mirrors `install-pin-check.yml:51-55` style.

### 3.4 Scope explicit non-goals

- **Not** auto-fixing drift — the gate is a check, not a writer. Auto-fix would obscure the authoring discipline.
- **Not** gating GUI repo's pins (separate repository; `mnemonic-gui/install.sh` mirror gate lives there per `gui-schema-mirror-lockstep-discipline`).
- **Not** gating the toolkit self-pin (already gated by `install-pin-check.yml`).
- **Not** scanning `Cargo.toml` or `Cargo.lock` for dep pins (different convention — sibling crate deps already use `*-codec` directly, not via `cargo install`).
- **Not** gating GitHub Actions tool-dep pins (`actions/checkout@v4`, `actions/setup-node@v4`, `dtolnay/rust-toolchain@*`, `lychee-v*`, `markdownlint-cli2@*`, `cspell@*`) — those are runner-tool versions, not sibling-CLI mirrors of install.sh's canonical table. (R0 M1 fold.)
- **Not** flagging the asymmetry between manual.yml (installs all 3 siblings) and quickstart.yml (installs only mk-cli). Quickstart mocks `MD_BIN=true MS_BIN=true` at `quickstart.yml:75` because its prose only exercises `mnemonic` + `mk`; the asymmetry is structural to the docs surface, not a drift. The gate verifies pin EQUALITY where both sides exist; it does not require the mirror sites to install every canonical sibling. (R0 M3 fold.)

### 3.5 Lessons applied
- [[feedback-fix-the-class-hunt-for-second-instance]]: scope is `manual.yml + quickstart.yml` (2 mirror files at write-time), not `manual.yml` alone. Hunt found the 2nd instance.
- [[feedback-ci-snapshot-test-substring-vacuity]]: the parser asserts ON load-bearing substrings; an unknown package name logs a warning rather than vacuously passing. Future siblings produce a noisy `::warning::` until added to the canonical extractor — discoverable, not silent.
- [[feedback-r2-blocking-vs-cosmetic-gate]]: YAML quoting + install actionlint locally before commit (actionlint via `nix-shell -p actionlint --run 'actionlint .github/workflows/sibling-pin-check.yml'` if available; otherwise visually-validate against the existing `install-pin-check.yml` structure).

## 4 — Test plan

### 4.1 Local verification
- Run the bash logic standalone against the live tree; assert clean exit.
- Inject a synthetic drift (`sed`-edit `manual.yml`'s `mk-cli-v0.4.2` → `mk-cli-v0.4.1`); rerun; assert exit 1 with the expected `::error::` line; revert the sed.

### 4.2 CI-side verification
- After merge, the next push fires the workflow; observe a green run in Actions.

### 4.3 Lint / quality gates
- No unit tests (workflow file).
- No GUI lockstep (no clap surface change).
- No manual mirror (no CLI surface change).

## 5 — Phase-6 release prep

CI-only PATCH; no version bump; no README marker change; no install.sh change (install.sh IS the canonical source); CHANGELOG entry under a `[Unreleased]` section OR rolled into next toolkit-bump PATCH; FOLLOWUP `Status: open → resolved`.

**Discussion point:** prior cycles have established that test/docs/CI-only cycles still bump the toolkit version (v0.28.5 = the docs-only-PATCH-bumps-toolkit-version precedent; v0.34.3 = test/docs-only; v0.37.7's manual-prose-execution gate cycle = test/docs/CI-only with no bump). The recent pattern (manual-prose-gate, just shipped) DID NOT bump version. Mirror that: no bump for sibling-pin-check.yml addition either.

## 6 — Files

**Modified (1):**
- `design/FOLLOWUPS.md` — flip `manual-yml-sibling-pin-vs-install-sh-drift-gate` status `open → resolved`.

**Added (3):**
- `.github/workflows/sibling-pin-check.yml` — the new gate.
- `design/SPEC_sibling_pin_drift_gate.md` — this spec.
- `design/agent-reports/sibling-pin-drift-gate-R0-review.md` — R0 reviewer transcript.

## 7 — Reviewer-loop disposition

This spec dispatched to opus architect for R0 BEFORE implementation per CLAUDE.md mandatory pre-impl R0 gate. R0 must converge to 0C/0I before any workflow YAML is written.
