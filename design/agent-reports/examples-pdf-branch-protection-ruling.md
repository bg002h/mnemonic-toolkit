# RULING — `examples.yml` branch-protection + path-filter reconciliation (OQ1)

- **Role:** opus architect, delegated CI-governance ruling ("ask architect"). Read-only analysis.
- **Cycle:** `examples-pdf-un-ci-gated` (spec `design/SPEC_examples_pdf_modernize_and_gate.md`;
  plan `design/IMPLEMENTATION_PLAN_examples_pdf_modernize_and_gate.md`).
- **Question:** OQ1 (plan §"Open question", lines 470-475) + the "wide + required" locked decision
  (plan Inputs §2, lines 20-22) — reconcile **WIDE path-filtered triggers** with a **REQUIRED** status
  check without wedging the toolkit's docs PRs, and pin the branch-protection scope.
- **Ground truth verified this session (2026-07-05):**
  - Toolkit `master` = **UNPROTECTED** (`gh api …/branches/master/protection` → 404 "Branch not protected").
    This cycle *establishes* protection for the first time.
  - GUI `master` protection (the precedent): `required_status_checks = { strict:false, contexts:["snapshots"] }`,
    `enforce_admins.enabled = false`, `required_pull_request_reviews` = absent, `required_linear_history:false`,
    `allow_force_pushes:false`. Verified live via `gh api repos/bg002h/mnemonic-gui/branches/master/protection`.
  - The GUI `snapshots` job (the required context) has **NO path filter** — `mnemonic-gui/.github/workflows/build.yml`
    triggers `pull_request: branches:[master,"release/**"]` with no `paths:` (build.yml:7-8), so it reports on
    every PR. Its sibling `tutorial-snapshots` also runs unfiltered but is **NOT** a required context (contexts is
    `["snapshots"]` only).
  - **Every existing toolkit PR-triggered workflow is path-filtered** (`rust.yml`, `manual.yml`, `manual-gui.yml`,
    `quickstart.yml`, `technical-manual.yml`, `bitcoind-differential.yml`, `cross-tool-differential.yml`,
    `fuzz-smoke.yml`, `gui-pin-drift-check.yml`, `vendor-freshness.yml`). The toolkit has **no precedent** for an
    unconditional-run required check. The lone exception is `sibling-pin-check.yml` (`pull_request: branches:[master]`,
    no `paths:`) — it runs on every PR but is **not** required.

---

## 0. TL;DR ruling

1. **The trap is real and the plan as written contains it.** Plan P2.a (lines 268-309) specifies a
   **path-filtered `pull_request:` trigger** for `examples.yml`; plan P2.c (lines 335-342) simultaneously says to
   make `examples` a required context "mirroring the GUI `snapshots` precedent." Those two are **mutually
   inconsistent** — the GUI precedent is required *precisely because* `snapshots` has no PR path filter. A
   path-filtered required check wedges every PR that touches none of the gated paths ("Expected — Waiting for status
   to be reported", indefinitely). This ruling resolves the inconsistency.

2. **Trigger shape (the core reconciliation) — Option (i), refined: single always-run `examples` job + internal
   changed-paths guard.** On `pull_request`, drop the workflow-level `paths:` filter so the job **always runs and
   always reports** the `examples` context (no wedge). Inside the job, a **hand-rolled `git diff --name-only` guard**
   (PR event only) decides `relevant=true|false`; every expensive step (apt-texlive / `cargo build` / regen /
   `git diff --exit-code` / pandoc / attach) is gated `if: github.event_name != 'pull_request' ||
   steps.guard.outputs.relevant == 'true'`. Docs-only PR → guard false → all heavy steps skip → the job succeeds in
   seconds → cheap green, no wedge, **no wasted compile**. Crates/install/`.examples-build` PR → guard true → full
   gate → red on drift, pre-merge. This is the clean, dependency-free merge of options (i) and (ii); (iii) and (iv)
   are rejected as *substitutes* (see §2), but (iv)'s push-to-master run is **retained as a complement** (see §4).

3. **Keep the workflow-level `paths:` filter on `push:` and the `tags:` trigger** (they don't need the
   report-on-every-event property; `paths` never filters tag pushes — `manual.yml:3-6` precedent). The
   `push: branches:[master]` run is **load-bearing, not redundant** — it is the *only* thing that gates
   direct-to-master changes (releases, design docs), which bypass PR checks entirely (§4).

4. **Governance scope — minimal, GUI-identical:** create branch protection on toolkit `master` with
   `required_status_checks = { strict:false, contexts:["examples"] }`, `enforce_admins:false`, and everything else
   off (no required reviews, no linear history). **Do NOT enroll any existing job** (`rust.yml` etc.) — they're all
   path-filtered and would inherit the same wedge; broadening scope is the parked `gui-branch-protection-scope`
   discussion, not this cycle's call. Confirmed this does **not** break the `direct-FF+tag` release flow (§3).

5. **Born-green ordering is deadlock-free** because `enforce_admins:false` gives the admin author a bypass — but
   still apply protection in the correct order (first-green-then-require); §5.

**Residual user decisions:** two, both minor — (R1) docs-only PRs will show `examples` as a ~instant green no-op
rather than a full run (confirm acceptable — it is strictly better than wedging or burning a compile); (R2) with
`enforce_admins:false`, "required" is a **hard** block for non-admins and a **loud, overridable** speed-bump for
the admin — identical to the GUI's posture. Both detailed in §6.

---

## 1. The path-filter / required-check trap — why it wedges

GitHub evaluates a required status check by **context name**. If a workflow is **skipped** because its
`on.pull_request.paths` filter did not match the PR's changed files, that workflow **never produces a check run**,
so the required context is **never reported**. Branch protection then holds the PR at *"Expected — Waiting for
status to be reported"* **forever** — there is nothing to satisfy it, and no re-run will ever fire because the paths
still don't match. This is a documented, well-known GitHub footgun ("skipped but required checks").

**Concretely for this cycle**, with the plan's P2.a path-filtered PR trigger + P2.c required check, these PRs wedge:

- A docs-only book-leg PR touching only `docs/manual-gui/**` (e.g. this cycle's own follow-on PRs, or the
  `gui_example` book legs #62/#63 cited in the brief) — `examples.yml` path-skips → wedged.
- A `docs/manual/**` PR (manual.yml fires; examples path-skips) → wedged.
- A `design/**`-only or `.github/workflows/manual.yml`-only PR → wedged.

The GUI avoided this **structurally**: `snapshots` has no path filter, so it reports on every PR. That is not an
incidental detail of the precedent — it is *the* mechanism that makes a required check safe. Any faithful "mirror of
the GUI `snapshots` precedent" (plan P2.c) **must** drop the PR path filter. The plan's P2.a kept it; that is the
defect this ruling corrects.

---

## 2. Evaluation of the four options (OQ1 sub-question 1)

| Opt | Mechanism | Verdict |
|---|---|---|
| **(i)** | No PR path filter; run every PR; cheap internal early-exit when irrelevant | **ADOPT (refined).** Only option that both reports-on-every-PR (no wedge) *and* avoids burning a full toolkit compile + texlive install on docs-only PRs. The refinement = an internal changed-paths guard (below). |
| **(ii)** | A "skip→success" companion shim job that always reports `examples`=success when the real path-filtered job is skipped | **REDUCES TO (i), less cleanly.** A same-workflow aggregator can't path-filter one job (paths filter the whole workflow); a *separate* always-run workflow re-reporting another workflow's context is racy/GitHub-discouraged (two runs, same context, divergent conclusions). The clean form of (ii) *is* (i): one always-run job named `examples` with internally-guarded steps. No separate shim. |
| **(iii)** | Keep path filters, make `examples` **advisory** (not required) | **REJECT.** Directly reverses the user's locked "required" decision and re-opens STOP **S3** (loud-rot discipline, plan lines 405-407). Not the architect's call to downgrade a user-locked decision. |
| **(iv)** | `push:`-to-master gating + tag-run only; no PR required-check | **REJECT as a substitute; RETAIN as a complement.** As a *replacement* it is strictly weaker than the user's intent: a red master-push is **post-hoc and non-blocking** — nobody is *forced* to act; the bad merge already landed. That is exactly "can be ignored," which the user rejected. **BUT** the push-to-master run is *independently necessary* for a different reason (§4): it is the only gate on direct-to-master changes. So (iv)'s push-run is folded in *alongside* the PR required-check, not instead of it. |

### The adopted mechanism — internal changed-paths guard (dependency-free)

Reuse the *same* path set as the `push:` trigger. On the `pull_request` event only, compute whether any gated path
changed; gate the heavy steps on it:

```yaml
- name: detect-relevant-changes        # PR-event only
  id: guard
  if: github.event_name == 'pull_request'
  run: |
    git fetch --no-tags --depth=1 origin "$GITHUB_BASE_REF"
    changed="$(git diff --name-only FETCH_HEAD HEAD)"
    if printf '%s\n' "$changed" | grep -qE \
       '^(\.examples-build/|docs/Examples\.pdf$|scripts/install\.sh$|crates/|Cargo\.lock$|Cargo\.toml$|\.github/workflows/examples\.yml$)'; then
      echo "relevant=true"  >> "$GITHUB_OUTPUT"
    else
      echo "relevant=false" >> "$GITHUB_OUTPUT"
      echo "No gated paths changed — examples gate is a no-op for this PR."
    fi
# every heavy step then carries:
#   if: github.event_name != 'pull_request' || steps.guard.outputs.relevant == 'true'
```

**Fail-safe direction (critical).** The guard must only ever err toward *running* the gate, never toward skipping
it (a wrongly-skipped gate is a silent false-green — the dangerous failure). This construction is fail-safe on both
counts: (a) the two-dot `git diff FETCH_HEAD HEAD` **over-includes** (it also picks up base-side changes since the
branch point) → at worst it runs the gate unnecessarily (harmless); it can **never** miss a PR-side edit to a gated
path; (b) if the fetch/guard step itself errors, the **job fails** (checkout has `fetch-depth:0` per plan P2.a
step 1, and `set -e` is default) — a hard red, never a false green. State this property in the workflow comment.

**Why hand-rolled over `dorny/paths-filter`.** A marketplace action that gates a *required* check is itself a
supply-chain trust surface. The toolkit's CI ethos is hand-rolled logic (`bitcoind-differential.yml`,
`reproducible-musl-build.yml` all hand-roll). If the user prefers the action, pin `dorny/paths-filter@<full-SHA>`;
functionally equivalent. Primary recommendation: the six-line git-diff above.

**Cost accounting (the brief's explicit weigh-in).** The gate rebuilds `mnemonic` (a debug cargo build) + installs
texlive + runs pandoc — **not** cheap; ~3-6 min/PR. Running that unconditionally on every docs-only book-leg PR (of
which this constellation produces many) is real waste. The internal guard reduces a docs-only PR to a checkout +
six-line grep = seconds, while preserving the full leading-indicator on code PRs. The GUI accepted an unconditional
`snapshots` run, but that is a single `cargo test` compile; the toolkit's docs PRs are frequent enough and the
compile heavy enough that the guard clearly earns its keep. **Adopt the guard; do not run the full job on docs-only
PRs.**

---

## 3. Governance scope (OQ1 sub-question 2) — minimal, GUI-identical

Establish branch protection on toolkit `master` with **exactly**:

```jsonc
required_status_checks:            { strict: false, contexts: ["examples"] }
enforce_admins:                    false
required_pull_request_reviews:     null        // no human-review requirement
required_linear_history:           false
allow_force_pushes:                false
allow_deletions:                   false
required_signatures:               false
required_conversation_resolution:  false
```

This is a **byte-for-byte match of the GUI precedent** (verified live), differing only in the context name
(`examples` vs `snapshots`).

- **`contexts:["examples"]` only — do NOT enroll `rust.yml` or any existing job.** Every existing toolkit PR
  workflow is path-filtered; enrolling any of them as required would reproduce the *same* wedge (a docs PR skips
  `rust.yml` → wedged) — they were never designed to be required contexts. Retrofitting the existing suite is a
  broader, separately-owned decision (the parked **`gui-branch-protection-scope`** item, "user: discuss later") and
  is out of scope for this cycle. Minimal scope now; defer the breadth question.
- **`strict:false`** (matches GUI). Do **not** set `strict:true` ("require branches up to date before merging") — it
  forces every PR to re-run against the latest master tip, friction the GUI deliberately omitted.
- **`enforce_admins:false`** (matches GUI, MEMORY gotcha (d): "enforce_admins off → release direct-push OK").

### Confirm it does NOT break the release flow

The toolkit's release mode is **`direct-FF + tag`** (codec/toolkit convention; MEMORY gotcha (d)) — the admin/owner
pushes `master` directly and pushes the tag; there is no release PR to merge. Branch-protection required checks gate
**PR merges** and, *only if `enforce_admins:true`*, admin direct pushes. With **`enforce_admins:false`**, an admin's
direct push to `master` is **not** blocked by the `examples` required context. Therefore `direct-FF+tag` continues
to function unchanged. The GUI has run precisely this config through multiple release direct-pushes — empirical
confirmation the shape is safe. Design-doc direct pushes (the toolkit norm) are likewise unaffected.

STOP **S4** (plan lines 408-412) resolves to **no-stop**: this matches the anticipated GUI-precedent resolution
(`enforce_admins` off, `contexts:[examples]` only). Only a *divergence* from that precedent would have been an S4
escalation; there is none.

---

## 4. Coherence with the anti-rot goal (OQ1 sub-question 3)

Requirement: "every toolkit output-format change is caught before it silently ships in `Examples.pdf`." Trace every
ingress path for a change that alters captured CLI output:

1. **Change arrives via PR touching `crates/**` / `Cargo.lock` / `Cargo.toml` / `scripts/install.sh` /
   `.examples-build/**`:** internal guard → `relevant=true` → full gate → `git diff --exit-code` red on drift,
   **pre-merge**, **required** → blocked for non-admins, loud-override for admin. ✅ *Leading* indicator preserved
   exactly as the user's "wide" intent demands.

2. **Change arrives DIRECT to `master`** (the toolkit's dominant mode — releases via `direct-FF`, design docs):
   **the PR required-check never fires — there is no PR.** This is the crucial subtlety: *a required PR check does
   nothing for direct-to-master pushes.* The catch here is the **`push: branches:[master] paths:[crates/**,…]`**
   run: it fires post-push and goes **RED on master** if the direct push changed captured output without
   regenerating the golden. For the specific release case, a crate-version bump trips `gen.sh:22`'s FATAL
   (version-string mismatch) → the push run reds immediately and visibly. Combined with the now-gate-enforced
   release ritual (spec §8 / plan lines 438-443: re-pin `gen.sh` + regen golden + rebuild PDF in the same change),
   the rot is caught. This catch is **post-hoc (loud red), not blocking** — inherent to direct-push (which bypasses
   all pre-merge gating) and acceptable; the enforcement there is "red master + release ritual," not a merge block.

3. **Tag push (`examples-v*`):** `paths` never filters tags (manual.yml:3-6 precedent) → the gate re-runs at
   release and attaches the PDF. ✅

**Conclusion:** the anti-rot guarantee holds, and it requires **both** halves — the required PR check (blocking, for
PR-borne drift) **and** the push-to-master run (post-hoc, for direct-push drift). The push trigger is therefore
**not optional and not redundant**; the ruling emphasizes this because the plan lists `push: branches:[master,main]`
(P2.a line 274) without flagging that it is the *sole* line of defense for the toolkit's most common change mode.
No silent ingress remains: PR / direct-push / tag all execute `examples.yml` whenever a gated path changed.

The known residual gaps are unchanged and already disclosed in spec §7 / plan Honesty (narration blind spot →
deferred gate B; sibling-CLI drift; committed-PDF byte-staleness; coverage gaps). This ruling does not touch them.

---

## 5. Born-green ordering (OQ1 sub-question 4)

The P2 PR carries `examples.yml` + the `.gitignore` flip + `git add` of the P1-regenerated golden, atomically
(plan P2.b line 332). Ordering:

1. **Open the P2 PR.** For a same-repo branch, GitHub runs the workflow file *from the PR head*, so `examples.yml`
   exists and runs on its own PR. The PR touches `.github/workflows/examples.yml` + `.examples-build/**` → guard
   `relevant=true` → the **full** gate runs → it must be **green** (born-green: committed golden == fresh regen).
2. **After the first green run**, add `examples` to `master` required contexts (`gh api … PUT
   …/protection`). Registering the context only after it has reported once avoids a spurious "Expected — waiting
   for status." Apply while the PR is still open (post-first-green) so the *merge itself* is gated, or immediately
   post-merge — either is fine.
3. **Merge**, then tag `examples-v1.0.0`.

**No chicken-and-egg deadlock is possible** here because `enforce_admins:false` gives the admin author a bypass: even
a mis-ordered protection (context required before it has ever reported) can be admin-merged. The correct ordering is
a cleanliness nicety, not a lockout hazard. (Contrast a hypothetical `enforce_admins:true`, which *could*
self-deadlock — another reason to keep it off.)

Two ordering cautions: (a) the P2 **negative proof** (perturb one golden byte → observe red; plan P2 GATE lines
348-350) must run on a **throwaway branch/PR**, never on `master`. (b) the internal guard's path set **must** include
`.github/workflows/examples.yml` (it does, per §2) so the P2 PR's guard evaluates `relevant=true` and actually
exercises the born-green gate rather than no-op'ing itself.

---

## 6. Residual user decisions

- **R1 — docs-only PRs no-op the gate (cheap green).** Under the adopted internal guard, a PR that touches none of
  the gated paths shows `examples` as a near-instant success rather than a full build. This is a faithful expression
  of "wide + required" (wide leading coverage on code PRs; required strength everywhere) and is strictly better than
  the alternatives (wedge, or burn a ~3-6 min compile per docs PR). *Recommended: accept.* This is the one way in
  which the implemented shape is subtler than a literal reading of "wide path-filtered triggers + required" — surface
  it so the user is not surprised that docs PRs don't run the full gate.

- **R2 — "required" has an admin-shaped asterisk.** With `enforce_admins:false` (mandatory, to preserve `direct-FF`
  releases), `examples` is a **hard block for non-admin PRs** and a **loud, admin-overridable** speed-bump for the
  admin (both on PR merge and on direct push). Making it *un*-overridable even by the admin would require
  `enforce_admins:true`, which **breaks the direct-FF release flow** (releases are admin direct-pushes) — so that is
  not on the table. This posture is identical to the GUI's today. *Recommended: accept* (it is the same trade the
  constellation already runs).

- **Deferred (not this cycle):** whether to eventually enroll `rust.yml` and the other existing gates as required
  contexts (which would require giving each of them the same no-PR-path-filter + internal-guard treatment). This is
  the parked **`gui-branch-protection-scope`** discussion; keep it out of scope here.

---

## 7. Concrete config summary (implementable, corrects plan P2.a/P2.c)

**`.github/workflows/examples.yml` triggers:**

```yaml
name: examples
on:
  push:
    branches: [master, main]
    paths:                                   # push/tag DON'T need report-on-every-event
      - '.examples-build/**'
      - 'docs/Examples.pdf'
      - 'scripts/install.sh'
      - 'crates/**'
      - 'Cargo.lock'
      - 'Cargo.toml'                         # root [patch.crates-io] miniscript rev (plan M2)
      - '.github/workflows/examples.yml'
    tags: ['examples-v*']                    # paths never filter tag pushes (manual.yml:3-6)
  pull_request:                              # NO paths — a REQUIRED check must report on every PR
```

**Job:** one job **named `examples`** (this name *is* the required context). Step 1 = `actions/checkout@v6`
`fetch-depth:0`. Step 2 = the PR-only `detect-relevant-changes` guard (§2). Steps 3-N = the heavy gate (apt
texlive + `dtolnay/rust-toolchain@1.85.0` + `cargo build --bin mnemonic` + `EXAMPLES_BIN_DIR=…/target/debug bash
.examples-build/gen.sh > …/Examples.md || exit 1` + `git diff --exit-code -- .examples-build/Examples.md` + pandoc
PDF + artifact upload + `examples-v*` tag-attach per `manual.yml:147-168`), **each carrying**
`if: github.event_name != 'pull_request' || steps.guard.outputs.relevant == 'true'`. No bitcoind (Q1-(b) froze
§6.6).

**Branch protection (`gh api -X PUT repos/bg002h/mnemonic-toolkit/branches/master/protection`):** the §3 JSON —
`required_status_checks:{strict:false,contexts:["examples"]}`, `enforce_admins:false`, `required_pull_request_reviews:null`,
`restrictions:null`, `required_linear_history:false`, `allow_force_pushes:false`, `allow_deletions:false` — applied
**after** the P2 PR's `examples` job has reported green at least once.

**Delta vs the plan:** P2.a's `pull_request: paths:[ …same set… ]` (line 284-285) → **`pull_request:` with no
`paths`, plus the internal guard**. P2.c is ratified as written (required context, GUI-precedent scope,
`enforce_admins` off) — the ruling only makes P2.a *consistent* with it. Everything else in P2 stands.

---

## 8. Precedents cited

- **GUI `snapshots` required check + protection:** `mnemonic-gui/.github/workflows/build.yml:87-147` (the
  no-path-filter `snapshots` job; comment lines 93-96 explicitly note "NO path filter — the job fires on every PR /
  master push / tag push"); live `gh api repos/bg002h/mnemonic-gui/branches/master/protection` =
  `contexts:["snapshots"]`, `strict:false`, `enforce_admins:false`. MEMORY: "branch protection LIVE on gui master
  (contexts=[snapshots] only … enforce_admins off → release direct-push OK)."
- **GUI `tutorial-snapshots`** (also unfiltered, NOT required): `build.yml:149-257`.
- **Toolkit tag-ignores-paths convention:** `manual.yml:3-6`, `quickstart.yml:3-6`, `manual-gui.yml:2-7`.
- **Toolkit tag release-attach pattern (to reuse):** `manual.yml:147-168`; `quickstart.yml:150-171`;
  `manual-gui.yml:412-431`.
- **Toolkit debug-binary-in-CI precedent:** `manual.yml`/`quickstart.yml:89-93` ("Debug binary is sufficient …").
- **Every existing toolkit PR workflow is path-filtered** (no unconditional-required precedent): `rust.yml:9-13`,
  `manual.yml`, `manual-gui.yml`, `quickstart.yml`, `technical-manual.yml`, `bitcoind-differential.yml`,
  `cross-tool-differential.yml`, `fuzz-smoke.yml`, `gui-pin-drift-check.yml`, `vendor-freshness.yml`. Lone unfiltered
  (non-required) exception: `sibling-pin-check.yml:2-6`.
- **Toolkit `master` currently unprotected:** `gh api …/branches/master/protection` → 404 "Branch not protected".
- **Parked scope discussion:** `gui-branch-protection-scope` (MEMORY "PARKED burndown items", user: discuss later).

---

*Ruling by opus architect, session 2026-07-05. Verdict on OQ1: WIDE + REQUIRED is achievable and correct — via a
no-PR-path-filter `examples` job with an internal changed-paths guard (not path-filtered triggers), a
`contexts:["examples"]`-only / `enforce_admins:false` protection mirroring the GUI precedent, and the load-bearing
`push:[master]` complement for direct-to-master drift. Two minor residual user acknowledgements (R1, R2); no STOP.*
