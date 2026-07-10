# SPEC R0 review — Cycle I test-hardening (gating + wc-codec CI) — round 1

**Reviewer:** Fable (SPEC R0, read-only), per user directive. SPEC @ toolkit `e6b87323` + sibling HEADs.
**Dispatched:** 2026-07-10 (Cycle I, SPEC R0 round 1). Persisted verbatim per CLAUDE.md.

**VERDICT: NOT GREEN — 2 Critical / 2 Important / 3 Minor.** The context strings, green-state, and push-trigger claims are all individually accurate (every one verified live), but the SPEC fails its own guard-rail G1 on two repos via a hazard it never examined: **path-filtered workflow triggers** — the exact wedge class this repo already ruled on for the `examples` gate.

## CRITICAL

**C1 — toolkit: `test (ubuntu-latest)` + `clippy` come from a path-filtered workflow; requiring them wedges every docs-only PR.** `.github/workflows/rust.yml` (`e6b87323`) has `paths:` filters on **both** `push` and `pull_request` (:13-27): `crates/**`, `Cargo.toml`, `Cargo.lock`, `.github/workflows/rust.yml`. A PR touching only `docs/**`/`design/**`/`scripts/install.sh` (a routine class here — manual PRs #58/#60) never triggers rust.yml, so neither context is ever reported → the PR sits at "Expected — waiting for status" forever for any non-admin merge. The repo's OWN ruling is quoted in `examples.yml:9-14` ("a path-filtered required check wedges forever…"; `design/agent-reports/examples-pdf-branch-protection-ruling.md` §7). The recon verified this on a CODE-touching release commit (`e6b87323` bumped Cargo.toml → rust.yml fired); on a docs-only commit it would not have. **Fix:** extend #2 to also edit rust.yml — remove `paths:` from `pull_request:` (keep the push-side filter; admin direct pushes bypass anyway), optionally add the fail-safe PR-only guard. Land in the #2 commit, verify green, then PUT.

**C2 — ms: all four required contexts have the identical path-filter wedge.** `mnemonic-secret` `rust.yml` (`ffc9d71`) filters push+PR on `crates/ms-cli/**`, `crates/ms-codec/**`, `Cargo.toml`, `Cargo.lock`, the workflow file (:19-35). A docs-only PR (e.g. a `design/FOLLOWUPS.md` companion — routine per the cross-repo convention) reports none of the four. Same wedge, and §3 FORBIDS the fix ("md/mk/ms/gui code untouched by #2"). The SPEC must (a) add a small ms-repo CI-only NO-BUMP commit applying the same trigger fix before the ms PUT, or (b) drop the ms contexts. **md / mk / gui are CLEAN** — no path filters; contexts confirmed reporting on docs-only HEAD commits (md `ef1f3e71` "docs(followups)" → `cargo test (ubuntu-latest)`/`cargo clippy` success; gui `350b913` "docs: companion FOLLOWUP" → all 5 contexts success).

## IMPORTANT

**I1 — the `-p` vs `--workspace` rationale is factually false.** The fuzz sub-crate is its OWN `[workspace]` (`crates/wc-codec/fuzz/Cargo.toml:17`) and is INVISIBLE to root `--workspace` — `cargo metadata` workspace_members = exactly the two crates. Both forms are safe today, but `--workspace` is the ANTI-RECURRENCE choice: eval #2 exists precisely because wc-codec was added as a member and never `-p`'d; a future third member silently repeats under the targeted form. Switch to `--workspace` (or keep targeted + correct the rationale + add a "new members must be appended" comment).

**I2 — fuzz-smoke.yml `paths:` don't cover wc-codec, so the new compile gate never fires on wc-codec changes.** Push/PR paths are `fuzz/**`, `crates/mnemonic-toolkit/src/parse_descriptor.rs`, `…/lib.rs`, the workflow file (:29-41). Adding wc-codec fuzz build without extending paths to `crates/wc-codec/fuzz/**` + `crates/wc-codec/src/**` leaves drift detection to the daily cron only. Not a required context (no wedge) but half-delivers #2. Extend both paths lists.

## MINOR
- **M1:** PUT with legacy `contexts:[…]` records `app_id:null` (any app satisfies); existing entries are `app_id:15368` (GitHub Actions). Prefer `checks:[{"context":…,"app_id":15368}]` for consistency + exact G4 round-trip.
- **M2:** Citation drift: fuzz-smoke build job is :51-73 (SPEC :63-73), smoke :75-104 (SPEC :76-96).
- **M3:** If option (a) matrixes the smoke job, its check-run name becomes `cargo fuzz run (60s smoke) (<target>)` — harmless (never required) but note it.

## Verified-correct (exact evidence)
- **Contexts exist + green on default-branch HEAD:** toolkit `e6b87323`: `test (ubuntu-latest)`, `clippy`, `examples`. md `ef1f3e71`: `cargo test (ubuntu-latest)`, `cargo clippy` (the `cargo ` prefix is real + md-only; toolkit/ms unprefixed — SPEC has each repo's prefix exactly right). ms `ffc9d71`: `test (ubuntu-latest)`, `clippy`, `test (ms-codec)`, `clippy (ms-codec)`. mk `1c9fbf72`: `build (stable on ubuntu-latest)`. gui `350b913`: `clippy`, `headless (no-default-features)`, `schema-mirror gate`, `x86_64-unknown-linux-gnu`, `snapshots`.
- **Push-event provenance:** md/ms/mk/gui contexts all confirmed on push-event runs on the default branch (mk ci.yml fires on branch push, not tag-only). mk steps ci.yml:52/55/58 = Build/Test/Clippy — exact.
- **Baseline protection:** toolkit=[examples], gui=[snapshots] (strict:false, enforce_admins:false); md/ms/mk 404 unprotected — matches §0.
- **#2 facts:** members exactly the two crates; rust.yml:105/:112 `-p mnemonic-toolkit`, job `test (${{matrix.os}})` (matrix ubuntu+macos) → adding coverage cannot change the context name (G5 holds); clippy :200 workspace-wide. **`cargo test -p wc-codec` = 100 tests pass**; **`cargo +nightly-2026-04-27 fuzz build --fuzz-dir crates/wc-codec/fuzz --target x86_64-unknown-linux-gnu` = exit 0** (both targets built; `--fuzz-dir` real; nested workspace pins the same nightly; no collision).
- **Shape:** `enforce_admins:false` + `strict:false` + `required_pull_request_reviews:null` + `restrictions:null` is the correct full PUT body + preserves direct-FF (empirical: toolkit shipped v0.77.0→v0.83.0 by admin push under required [examples] + enforce_admins:false). PUT-on-unprotected creates protection. G4 reversibility accurate (DELETE for md/mk/ms; PUT-back [examples]/[snapshots]).
- **Exclusions right:** macOS/windows excluded (macOS ICE precedent). `bitcoind-differential` runs tag+cron only → correctly omitted (would wedge). `miri`/`g6` in the same path-filtered rust.yml + flake history → excluding is defensible.
- **Ordering #2→#1:** correct; the #2 commit touches rust.yml (in rust.yml's own push paths) so the broadened context self-verifies green before the PUT (G3 sound).

**Bottom line:** context matrix string-exact + green everywhere, but do NOT apply #1 to toolkit or ms until the rust.yml `pull_request` path filters are removed (examples.yml ruling) — C1/C2. Fold C1/C2/I1/I2 + Minors, re-dispatch.
