# cycle-prep recon — 2026-06-12 — fuzz-nightly-quarterly-bump

**Origin SHAs at recon time:**
- mnemonic-toolkit `origin/master` = `dbdacfb`
- descriptor-mnemonic `origin/main` = `31e5895`
- mnemonic-secret `origin/master` = `fc6fc13`
- mnemonic-key `origin/main` = `21786dc` (default branch confirmed `main` via `git remote show origin`)

**Local branch (toolkit):** `master`
**Sync state (toolkit):** up-to-date (0 ahead / 0 behind)
**Untracked (toolkit):** pre-existing cycle-prep recon docs, cycle-b-* scripts, CONTINUITY.md, .claude/ — none touch fuzz infra.

Slug verified: `fuzz-nightly-quarterly-bump` (FOLLOWUPS.md:4041, canonical constellation-wide tracker). Drift expectation: low — entry was filed/verified 2026-06-12, same day as this recon. Confirmed: zero drift.

---

## Per-slug verification

### `fuzz-nightly-quarterly-bump`

- **WHAT (from FOLLOWUPS.md:4041–4049):** Recurring maintenance — quarterly bump of the dated fuzz nightly (`nightly-2026-04-27`) pinned uniformly across all FOUR repos' `fuzz/rust-toolchain.toml` + each repo's `fuzz-smoke.yml` dtolnay pin, keeping the constellation fuzz infra on one date. First due ~2026-09. Surfaced from stress Cycle C fuzzing R0 ([M5]).

- **Citations (all re-grepped against the origin SHAs above):**

  | Repo (branch) | `fuzz/rust-toolchain.toml` `channel` | `fuzz-smoke.yml` `toolchain:` (dtolnay) | `cargo +nightly-…` cmd lines | gnu-target pin | Verdict |
  |---|---|---|---|---|---|
  | mnemonic-toolkit (`origin/master` @ `dbdacfb`) | `nightly-2026-04-27` | `nightly-2026-04-27` (:61, :87) | :73, :97 | `targets: x86_64-unknown-linux-gnu` (:63, :89) + `--target x86_64-unknown-linux-gnu` (:73, :97) | **ACCURATE / UNIFORM** |
  | descriptor-mnemonic (`origin/main` @ `31e5895`) | `nightly-2026-04-27` | `nightly-2026-04-27` (:50, :84) | :62, :94 | :52, :86 + `--target` :62, :94 | **ACCURATE / UNIFORM** |
  | mnemonic-secret (`origin/master` @ `fc6fc13`) | `nightly-2026-04-27` | `nightly-2026-04-27` (:56, :89) | :68, :99 | :58, :91 + `--target` :68, :99 | **ACCURATE / UNIFORM** |
  | mnemonic-key (`origin/main` @ `21786dc`) | `nightly-2026-04-27` | `nightly-2026-04-27` (:60, :97) | :72, :107 | :62, :99 + `--target` :72, :107 | **ACCURATE / UNIFORM** |

  - `fuzz/rust-toolchain.toml` ×4, `channel = "nightly-2026-04-27"` — **ACCURATE**, byte-identical channel line in all four; each file's comment cross-cites this FOLLOWUP and `fuzz-smoke.yml`.
  - `fuzz-smoke.yml` dtolnay pin ×4 — **ACCURATE**, but note each workflow carries the date in **four** places (two `dtolnay/rust-toolchain` `toolchain:` inputs — build job + smoke job — and two `cargo +nightly-2026-04-27 fuzz …` command lines). The entry's "8 pins" counts files (4 toml + 4 yml); the literal textual occurrence count is **4 + 16 = 20** date strings across 8 files. Counting clarified below; not an error, just an under-specification of the sed surface.
  - gnu-target pin (`--target x86_64-unknown-linux-gnu`) — **ACCURATE** in all four workflows (both as dtolnay `targets:` and on every cargo-fuzz invocation; the musl/ASan-default gotcha is documented in load-bearing comments). The gnu-target itself is date-independent and does NOT change at bump time.
  - Toolkit-only miniscript `[patch.crates-io]` rev `95fdd1c` replicated in `fuzz/Cargo.toml` — **ACCURATE**: `origin/master:fuzz/Cargo.toml:27-28` has `miniscript = { git = "https://github.com/rust-bitcoin/rust-miniscript", rev = "95fdd1c5773bd918c574d2225787973f63e16a66" }`. Date-independent; only changes if the root miniscript pin moves (separate concern, not part of this bump).
  - Each repo commits a `fuzz/Cargo.lock` (verified via `git ls-tree` ×4) — the "refresh each `fuzz/Cargo.lock` if needed" step has a real target in every repo.
  - mnemonic-key extra: its `fuzz/rust-toolchain.toml` carries the load-bearing "do NOT promote to repo root" comment (no-root-toolchain rule / fmt-pin interaction) — the bump must edit ONLY the `channel` line, never restructure that file.

- **Pin age / early-bump assessment:** `nightly-2026-04-27` is ~6.5 weeks old today. Dated nightlies remain downloadable from static.rust-lang.org essentially indefinitely (Rust does not garbage-collect dated nightly archives on a quarterly horizon); CI is exercising this exact pin daily (smoke cron) so any availability regression would self-surface immediately. No yank risk, no known sanitizer/cargo-fuzz blocker. **No reason to bump early.**

- **Action for brainstorm spec (when due, ~2026-09):** no brainstorm needed — this is a mechanical lockstep chore, not a design cycle. The exact edit set per repo (×4, all on the SAME new date `nightly-YYYY-MM-DD`):
  1. `fuzz/rust-toolchain.toml` — `channel` line (1 occurrence).
  2. `.github/workflows/fuzz-smoke.yml` — both `toolchain:` inputs + both `cargo +nightly-…` command lines (4 occurrences; a repo-scoped `grep -rn 'nightly-2026-04-27' fuzz/ .github/workflows/fuzz-smoke.yml` then sed is the mechanical move — grep-verify ZERO residual old-date hits afterward).
  3. `fuzz/Cargo.lock` — refresh if the new nightly's cargo rewrites it (commit only if changed).
  4. Verification per repo: `cargo +<new-nightly> fuzz build --target x86_64-unknown-linux-gnu` (all targets), then let one daily `smoke` cron run go green; toolkit additionally exercises the `descriptor_parse` cfg(fuzzing) target.
  5. The gnu-target pin and the toolkit miniscript `[patch.crates-io]` rev are NOT part of the bump (date-independent) — leave untouched.
  6. All four repos land together (4 small commits, one per repo); NO-BUMP everywhere (CI/fuzz infra only, no crate-version change, no CLI surface → no schema_mirror, no manual mirror).
  Cite source SHAs: toolkit `dbdacfb`, descriptor-mnemonic `31e5895`, mnemonic-secret `fc6fc13`, mnemonic-key `21786dc`.

---

## Cross-cutting observations

1. **Zero drift.** Entry filed 2026-06-12 and re-verified same day — all 8 pin files (and all 20 textual date occurrences) are uniform at `nightly-2026-04-27`. The uniformity claim holds exactly as written.
2. **Claim-counting clarification:** "8 pins" = 8 FILES. The per-workflow date appears 4× (two dtolnay inputs + two cargo command lines), so the sed surface is 20 occurrences. A bump that edits only the dtolnay `toolchain:` inputs but misses the `cargo +nightly-…` lines would break CI loudly (`+nightly-…` toolchain absent) — fail-closed, but worth the grep-zero-residuals check anyway.
3. **mnemonic-key default branch is `main`** (toolkit/mnemonic-secret use `master`, descriptor-mnemonic uses `main`) — bump-day muscle memory hazard only.
4. All four `fuzz/rust-toolchain.toml` files and all four workflows carry comments cross-citing this FOLLOWUP slug by name — the documentation mesh is intact; the bump must not strip those comments.
5. Daily smoke crons double as a standing availability canary for the pinned nightly — staleness/yank would self-report long before the quarterly date.

---

## Recommended brainstorm-session scope

**Verdict: PARK until ~2026-09 (first quarterly due date). Nothing actionable now.** The recon's entire purpose was re-grounding the uniformity claim ~3 months early — it holds perfectly (8/8 files, 20/20 occurrences uniform; gnu-target + miniscript-rev side-pins also accurate). The pin is ~6.5 weeks old, exercised daily by CI, with no availability or tooling pressure.

When due: a single sitting, four-repo lockstep chore — ~5 changed lines per repo plus possible `Cargo.lock` refreshes, NO-BUMP in all four repos, no R0-grade design content (mechanical date substitution + build verification). No schema_mirror, manual-mirror, or sibling-FOLLOWUP companion locksteps fire (no CLI/flag surface touched). After the first bump, update the FOLLOWUPS entry's "verified uniform" date and next-due quarter rather than closing it (Status stays `open`, Tier `recurring-maintenance`).

**Tier: `recurring-maintenance` (confirmed) — park, calendar ~2026-09.**
