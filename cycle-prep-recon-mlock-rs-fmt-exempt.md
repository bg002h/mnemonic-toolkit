# cycle-prep recon — 2026-06-12 — mlock-rs-fmt-exempt

**Origin/master SHA at recon time:** `dbdacfb` (entry filed in this very commit)
**Local branch:** `master`
**Sync state:** up-to-date (0 ahead / 0 behind)
**Untracked:** cycle-prep recon docs, cycle-b scratch scripts, CONTINUITY.md (unrelated)
**Sibling:** mnemonic-secret `origin/master` = `fc6fc13` (the paired revert commit)

Slug verified: `mlock-rs-fmt-exempt`. Entry is hours old (filed in `dbdacfb`), so near-zero drift expected — and that is what was found, with two measured corrections (the chore's pre-revert line anchors, and the ms-codec 0.4.4 "crates.io-pending" claim, which dissolved this morning).

---

## Per-slug verification

### `mlock-rs-fmt-exempt`
- **WHAT (from FOLLOWUPS.md `:4051`):** INTENTIONAL STANDING EXEMPTION. The 1.95.0 fmt chore (`1683159`) re-wrapped `mlock.rs`, breaking the toolkit `g6` job (which compares against the FROZEN, unformatted `ms-cli-v0.7.0` tag). Resolution (Option A, architect-reviewed): mlock.rs reverted to unformatted in BOTH repos; the toolkit fmt job exempts mlock.rs. Closes on the next ms-cli pin bump to a 1.95.0-formatted tag.

- **Citations:**
  - `.github/workflows/rust.yml` fmt-job exemption logic — **ACCURATE**. `fmt:` job at `:50` (name `fmt (pinned 1.95.0)` `:51`). The `run:` block `:66-76` is exactly as the entry describes: `out=$(cargo +1.95.0 fmt --all -- --check 2>&1) || true`, then `bad=$(printf '%s\n' "$out" | grep -oE '^Diff in [^:]+' | sed 's/^Diff in //' | grep -v '/mlock\.rs$' || true)`, fail iff `$bad` non-empty. Load-bearing exemption comment PRESENT twice: the standing block `:40-49` ("EXEMPTION — `crates/mnemonic-toolkit/src/mlock.rs` is the ONE file excluded… When the ms-cli pin is next bumped to a 1.95.0-formatted tag, reformat mlock.rs in BOTH repos in lockstep and drop this exemption") + the step-level comment `:60-65`. The closure trigger is durably recorded in the workflow itself.
  - `.github/workflows/rust.yml` `g6 invariant` job — **ACCURATE**. Job at `:257` (`name: "g6 invariant (cross-repo mlock.rs)"`). It does NOT use `ref: master`: the `Resolve pinned ms-cli tag from install.sh` step (`:270-282`) parses `scripts/install.sh` (`tag=$(grep -oE 'echo "[a-z-]+\|https://[^"]+\|…"' scripts/install.sh | … | awk -F'|' '$1=="ms-cli"{print $3; exit}')`, fail-loud on empty), then the sibling checkout (`:283-288`) uses `repository: bg002h/mnemonic-secret` + `ref: ${{ steps.pin.outputs.tag }}`. Test invoked `:297` with `--include-ignored` + `SIBLING_REPO_PATH`.
  - `scripts/install.sh:38` ms-cli pin — **ACCURATE**, live line `:38` exactly: `echo "ms-cli|https://github.com/bg002h/mnemonic-secret|ms-cli-v0.7.0|yes|"`.
  - `crates/mnemonic-toolkit/src/mlock.rs` unformatted form — **ACCURATE** (entry's `:165`/`:371` anchors are pre-revert chore-context positions; live = **DRIFTED-by-+2**): the method chain `self.total_bytes_unlocked.fetch_add(bytes, Ordering::Relaxed);` is ONE line at `:167`; `assert_eq!(count, 3, "2*page+1 spans 3 pages when starting page-aligned");` is ONE line at `:373`. Raw line count **533**; normalized (G6 rule: drop blank + `//`-leading trimmed lines, trim) = **347 lines**, matching the entry's pre-chore "347" figure — the revert restored it exactly.
  - Cross-repo g6 satisfiability — **ACCURATE / VERIFIED 3-WAY**. Normalized `mlock.rs` is byte-IDENTICAL (347 lines each) across (a) toolkit `origin/master dbdacfb`, (b) mnemonic-secret `origin/master fc6fc13` `crates/ms-cli/src/mlock.rs` (raw 538 lines — comment-only delta, allowed), and (c) the actual g6 comparator, tag `ms-cli-v0.7.0`. Normalization mirrored `tests/mlock_g6_invariant.rs::normalize` (`:126-136`). **g6 is satisfiable right now**; both reverts landed (`dbdacfb` toolkit, `fc6fc13` sibling reverting `d690212`).
  - Entry claim "ms-codec 0.4.4's crates.io-pending material" — **DRIFTED (overtaken by events)**: crates.io API shows ms-codec `max_version = 0.4.4` published **2026-06-12T10:13Z** (today, after the entry was filed). ms-cli on crates.io remains `0.7.0` (2026-06-03). ms-cli@master pins `ms-codec = { path…, version = "=0.4.4" }`, so a future `ms-cli-v0.7.1` publish is no longer blocked on an unpublished dep.

- **Action for brainstorm spec:** none — no brainstorm warranted (see Assessment). If a future closure spec is written at pin-bump time, cite toolkit `dbdacfb` (workflow `:40-49`, `:66-76`, `:270-288`; install.sh `:38`) and note ms-codec 0.4.4 is published.

---

## Assessment

**Actionable now? NO — correctly PARKED.** The exemption was deliberately chosen (Option A, architect-reviewed, shipped in `dbdacfb`/`fc6fc13` hours ago) and is in a fully consistent steady state: fmt gate green (mlock.rs exempt), g6 green (3-way normalized-identical, verified above), closure trigger recorded in three places (FOLLOWUPS entry, workflow comment block `:40-49`, step comment `:60-65`). There is no red CI, no drift, no missing guard. Re-litigating a decision made this same session would be churn.

**Proactive-close path (rejected Option B), re-scoped at today's facts:**
1. mnemonic-secret: cut `ms-cli-v0.7.1` from master — drags the whole `ms-cli-v0.7.0..fc6fc13` delta through a release gate: ms-codec 0.4.3 (panic fix) + **0.4.4 (SECURITY, breaking-marked `fix(ms-codec)!`)** + fuzz infra + the mlock revert. The original blocker ("0.4.4 crates.io-pending") **dissolved today** (0.4.4 published 10:13Z), so a v0.7.1 publish is mechanically unblocked — but it is still a full release ritual (CHANGELOG, tag CI, crates.io publish of ms-cli) whose only fmt-relevant payload is an ~8-line re-wrap.
2. Reformat `mlock.rs` (1.95.0) in BOTH repos + ensure the reformatted copy is IN the new tag (ordering matters: reformat must land before/in the tag, then toolkit bumps).
3. Toolkit: bump `scripts/install.sh:38` → `ms-cli-v0.7.1` (fires `sibling-pin-check` + g6-resolver lockstep; check any install.sh self-pin sites per the v0.53.1 lesson).
4. Drop the fmt-gate exclusion (`:68-69` grep -v) + the comment block `:40-49`/`:60-65`; flip the FOLLOWUP to resolved.

Blast radius: two repos, one crates.io publish, two CI lockstep gates, a release that exists ONLY to ferry a formatting delta — unless mnemonic-secret independently needs a v0.7.1 (it plausibly will soon, to ship the 0.4.4 SECURITY fix in a published ms-cli; ms-cli@crates.io 0.7.0 still pins the pre-fix ms-codec). **The right move: piggy-back the close on THAT release when it happens, not cut one for fmt's sake.**

**Mechanism drift/risk (residual, none actionable):**
- *Fail-open on header-format change:* the gate parses rustfmt's `Diff in <path>:` header after `|| true`. If a future rustfmt changed the header text, `bad` stays empty and the gate passes vacuously. Mitigated: the toolchain is hard-pinned (`cargo +1.95.0`, dtolnay install of exactly 1.95.0), so the header is frozen until a DELIBERATE formatter bump — and the standing comment (`:30-38`) already mandates re-running fmt in the same commit on any bump. Acceptable.
- *Edition-2024 new-crate escape:* none — `cargo fmt --all -- --check` formats every workspace crate per its own edition, and a new crate's diffs emit `Diff in <path>:` headers that do NOT match `/mlock\.rs$`, so they FAIL loudly as intended.
- *Filter over-breadth:* `grep -v '/mlock\.rs$'` exempts any file literally named `mlock.rs` in ANY crate, not just `crates/mnemonic-toolkit/src/mlock.rs`. Today only one such file exists (`git ls-tree` verified; `tests/lint_safety_first_party_mlock.rs` does not match — the leading `/` anchors it). Only a risk if a second `mlock.rs` is ever added; not worth tightening a temporary exemption for.

---

## Recommended scope

**Verdict: PARK.** Zero actionable work; the exemption is freshly shipped, internally consistent, 3-way verified green, and self-documenting with its closure trigger embedded in the workflow. The single new fact since filing — ms-codec 0.4.4 published to crates.io today — removes the hard blocker on cutting `ms-cli-v0.7.1` but does not create a reason to: close opportunistically when mnemonic-secret next cuts an ms-cli release (likely soon, to ship the 0.4.4 SECURITY fix through a published ms-cli), folding steps 2-4 above into that cycle as a NO-BUMP toolkit companion (CI + one-file fmt; no clap surface → no schema_mirror, no manual mirror).

**Tier:** `park` / `deferred-until-pin-bump` (`ci-hygiene` when it closes).
