# SPEC — A2-gui-msrv-job: CI-enforce the GUI declared MSRV (rust-version = 1.88)

**Repo:** `mnemonic-gui` (single-repo)
**Class:** CI-only YAML + FOLLOWUPS entry. **NO-BUMP / NO-TAG.** No crate-version touch.
**Slug to file:** `gui-msrv-ci-tested-invariant`
**Source SHAs (grounding, re-verified live this session):** `mnemonic-gui` origin/master = HEAD = `e7f0c63` (clean, 0 ahead/0 behind); `mnemonic-toolkit` origin/master = `572a15d1`. All citations below re-grepped against live `e7f0c63`.
**Surface gates touched:** none of the lockstep gates fire — NO clap flag/value/subcommand change (no `schema_mirror` delta), NO CLI surface change (no `docs/manual/src/40-cli-reference/` mirror), NO sibling-codec companion. The only new gate is the `msrv` CI job itself. `actionlint` must stay clean.

---

## 1. Problem

`mnemonic-gui/Cargo.toml:8` declares `rust-version = "1.88"` (corrected from a false `"1.85"` by PR#18 / `e7f0c63`). Verified live:

```
3   version = "0.49.0"
4   edition = "2021"
...
8   rust-version = "1.88"
```

But **no GUI CI job compiles at 1.88.** All three `dtolnay/rust-toolchain` invocations across the workflows are `@stable` (re-verified live):

- `.github/workflows/build.yml:22` — `clippy` job, `@stable` + `components: clippy`
- `.github/workflows/build.yml:73` — `build` matrix job, `@stable` + `targets: ${{ matrix.target }}`
- `.github/workflows/schema-mirror.yml:24` — `schema-mirror` gate, `@stable`

There is no `toolchain:` literal and no other rust-toolchain pin anywhere in `.github/workflows/`. The declared MSRV is therefore an **un-tested assertion** that can silently re-drift (e.g. a future dep bump raising the real floor, or an accidental edit of `Cargo.toml:8`) with nothing in CI to catch it.

**Binding-constraint fact (authoritative, re-verified live):** the live `Cargo.lock` pins `image = 0.25.10`, whose published `rust_version = "1.88.0"` is the **binding MSRV floor** (the icu_*@2.2.0 / idna_adapter@1.2.2 family are only `1.86`, satisfied with margin). So a job pinned to **exactly `1.88.0`** under `--locked` goes GREEN today (zero margin) and is load-bearing from day one: any future silent re-drift above 1.88 (e.g. an un-`--locked` `image` bump) turns it RED.

## 2. Fix

Add ONE new single-host job named `msrv` to `.github/workflows/build.yml` (co-located with the existing `clippy` + `build` compile gates — same `on:` triggers, no new workflow file, no new trigger config). The job:

- runs on `ubuntu-latest` only (NOT the 5-target release matrix — cross-compile + win/mac add no MSRV signal and ~5× the minutes),
- installs `dtolnay/rust-toolchain@1.88.0` (the exact pin matching `Cargo.toml:8`),
- runs `cargo check --locked` (NOT `cargo build`, NOT tests, NOT clippy).

**Why `cargo check` (not build):** `check` is the conventional MSRV gate — it drives type/borrow/trait resolution and per-crate MSRV (`rust-version`) enforcement across the whole locked dependency graph, which is exactly the "does this genuinely compile at 1.88" question. It is faster than a full `build` (no codegen/link). Codegen-stage issues are already covered by the 5-target `build` matrix on `@stable`.

**Why NO `--all-targets`:** the recon skeleton showed `--all-targets`, but the task scope is explicitly "NOT tests". `--all-targets` pulls the test/bench/example tree (and their dev-deps) into the MSRV compile; the binary-crate MSRV question is answered by the default target set (lib + bins). Default `cargo check` (no `--all-targets`) keeps the gate scoped to shipped code and avoids dragging dev-dependency MSRVs into the floor. (Default `cargo check` already covers bins + lib; this crate is a single binary crate.)

**Why `--locked` is mandatory:** without it `cargo` re-resolves the graph; a newer `image`/icu could be pulled that changes (raises or masks) the binding MSRV, defeating the test. The job must compile the **committed lockfile**, so the MSRV it proves is the one users actually get. (`cargo metadata --locked` was verified to resolve cleanly against the live `Cargo.lock`.)

**2-site MSRV invariant (note for future maintainers):** after this lands, an MSRV change is a **two-site edit** — `Cargo.toml:8 rust-version` AND this job's `dtolnay/rust-toolchain@<ver>` pin must move together. The YAML carries an inline comment pointing back at `Cargo.toml:8` so the coupling is discoverable. (This human-coupling is itself the invariant being added; there is intentionally no auto-derivation — pinning the toolchain literal is what makes the gate deterministic.)

## 3. Exact files to change

1. `mnemonic-gui/.github/workflows/build.yml` — add the `msrv` job (full YAML in §4). **Do not touch** the existing `clippy` (`:22 @stable`), `build` (`:73 @stable` matrix), or `release` jobs, nor the `on:` block.
2. `mnemonic-gui/FOLLOWUPS.md` (repo ROOT — confirmed live via `git ls-files | grep -i followup` → single `FOLLOWUPS.md` at root; there is NO `design/FOLLOWUPS.md` in this repo) — file the slug under the `## Resolved` section, marked resolved in this shipping commit (full entry in §5).

Optional (NOT required for GREEN; only if cross-instance coordination wants it): a one-line companion note appended to `mnemonic-toolkit/design/FOLLOWUPS.md` entry `install-sh-gui-sibling-pin-staleness-ungated` (its `rustc-MSRV tension` leg, OPEN-remainder (b)) recording "CI-enforcement piece now tracked + RESOLVED as gui `gui-msrv-ci-tested-invariant`." This is a doc-only cross-ref, no toolkit release. The recon notes the toolkit entry already shipped the manifest+README half (leg b) at `e7f0c63`/PR#18; only the CI-test piece was untracked. If the toolkit lane is out of scope for this CI-only GUI cycle, skip it — the GUI-side slug is self-sufficient.

## 4. Full YAML to add (build.yml)

Insert this job in `.github/workflows/build.yml` between the `clippy` job (ends at line 30) and the `build` job (begins at line 32) — i.e. immediately after the `clippy` job's last step and before the `build:` key. (Placement is cosmetic; YAML job order is irrelevant to execution. Co-locating it with the other compile gates and BEFORE the heavy `build` matrix reads naturally.) Indentation is **2 spaces** for the job key under `jobs:` and 4/6/8 spaces for nested keys, matching the existing `clippy`/`build` jobs exactly.

```yaml
  msrv:
    # gui-msrv-ci-tested-invariant: compile-at-MSRV gate. Cargo.toml declares
    # `rust-version = "1.88"` (the binding floor is `image@0.25.10`, whose
    # rust_version is 1.88.0); the @stable clippy/build jobs never prove the
    # crate actually builds at that floor. This job pins the toolchain to
    # 1.88.0 and runs `cargo check --locked` so the declared MSRV is an
    # ENFORCED invariant, not an un-tested assertion. `--locked` is required:
    # it checks the committed lockfile's graph, so an unlocked re-resolve
    # cannot pull a newer-MSRV dep and mask drift.
    # NOTE: bumping the MSRV is a 2-site edit — Cargo.toml `rust-version`
    # AND the `@1.88.0` pin below must move together.
    name: msrv (1.88.0)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5

      - name: install-rust-1.88.0
        uses: dtolnay/rust-toolchain@1.88.0

      - name: cargo-cache
        uses: Swatinem/rust-cache@v2

      - name: cargo check --locked
        run: cargo check --locked
```

Action-version pins (`actions/checkout@v5`, `Swatinem/rust-cache@v2`) are copied verbatim from the existing jobs in this same file (re-verified live: `checkout@v5` at build.yml:19/56, `rust-cache@v2` at build.yml:27/78) — keep them identical so there is no mixed-version drift. The `dtolnay/rust-toolchain` action is the same action the other three jobs use; only the ref differs (`@1.88.0` vs `@stable`), which is the action's documented version-pin form (a Git tag of the rust version).

## 5. FOLLOWUPS.md entry (file + flip in the shipping commit)

Append this entry under the `## Resolved` section of `mnemonic-gui/FOLLOWUPS.md` (the section header is at line 793 live; place the new `###` block immediately after the `## Resolved` header, matching the test-only/no-bump precedent set by `canonicity-drift-gate-floor-too-lenient`). It is filed already-resolved because the resolving change ships in the same commit:

```markdown
### `gui-msrv-ci-tested-invariant` — declared `rust-version = "1.88"` was an un-tested assertion; no CI job compiled at the MSRV (RESOLVED, CI-only NO-BUMP)

- **Surfaced:** 2026-06-23, A2 cycle-prep recon (`mnemonic-toolkit/cycle-prep-recon-gui-msrv-ci-tested-invariant.md`). Source SHAs `e7f0c63` (mnemonic-gui) / `572a15d1` (mnemonic-toolkit).
- **Where:** `Cargo.toml:8` declares `rust-version = "1.88"` (corrected 1.85→1.88 by PR#18 / `e7f0c63`), but all three `dtolnay/rust-toolchain` invocations were `@stable` — `.github/workflows/build.yml:22` (clippy), `.github/workflows/build.yml:73` (build matrix), `.github/workflows/schema-mirror.yml:24` (schema-mirror). Nothing in CI compiled at 1.88.
- **What:** the declared MSRV was an un-tested assertion that could silently re-drift (accidental `Cargo.toml` edit, or a dep bump raising the real floor). The binding floor is `image@0.25.10` (`rust_version = 1.88.0`); icu_*@2.2.0 / idna_adapter@1.2.2 are only 1.86 (satisfied). So 1.88.0 is exactly correct and the gate goes GREEN at zero margin — and catches any future drift above it.
- **Status:** **resolved** (CI-only, **NO version bump / NO tag**). Added a dedicated `msrv (1.88.0)` job to `.github/workflows/build.yml`: `dtolnay/rust-toolchain@1.88.0` + `cargo check --locked` (NOT the 5-target build matrix, NOT tests, NOT clippy). `--locked` checks the committed lockfile so an unlocked re-resolve can't mask drift. MSRV is now a 2-site edit (`Cargo.toml:8 rust-version` + the `@1.88.0` toolchain pin); an inline YAML comment records the coupling. Existing `@stable` clippy/build/schema-mirror jobs untouched. `actionlint` clean.
- **Tier:** `cross-repo` (CI-hygiene; the CI-enforcement piece of the toolkit `install-sh-gui-sibling-pin-staleness-ungated` rustc-MSRV-tension leg).
- **Companion:** `mnemonic-toolkit/design/FOLLOWUPS.md::install-sh-gui-sibling-pin-staleness-ungated` (its `rustc-MSRV tension` leg shipped the GUI `Cargo.toml`/README half via PR#18; this entry is the CI-test-the-MSRV piece that leg's OPEN-remainder (b) did not cover).
```

## 6. ci_gates_to_verify (with HOW, run from `mnemonic-gui/` repo root)

1. **`actionlint` clean (the binding lint gate for this change).**
   `actionlint .github/workflows/build.yml .github/workflows/schema-mirror.yml` → must exit 0 with no output.
   - Baseline this session: exit 0 (actionlint 1.7.12 at `/usr/bin/actionlint`). The new job uses only constructs already present in the file (`runs-on: ubuntu-latest`, `uses:` action pins, `run:` shell), so no new actionlint surface is introduced.
   - If actionlint is not on `$PATH`, install via the project's usual path or `go install github.com/rhysd/actionlint/cmd/actionlint@latest`.

2. **YAML well-formedness sanity (cheap, optional but recommended).**
   `python3 -c "import sys,yaml; yaml.safe_load(open('.github/workflows/build.yml'))"` → exits 0 (no parse error). (If PyYAML is absent, `actionlint` already subsumes this — it parses the workflow.)

3. **The new `msrv` job goes GREEN in CI (the actual invariant).**
   This runs in GitHub Actions on push-to-master / PR / tag (the `on:` triggers `build.yml` already declares). It cannot be fully reproduced on the dev host unless rustc 1.88.0 is installed (the host here is nightly 1.97). To reproduce locally with rustup:
   `rustup toolchain install 1.88.0 && cargo +1.88.0 check --locked` from the repo root → must exit 0.
   - Local lockfile-resolves-under-`--locked` was pre-verified this session: `cargo metadata --locked --format-version 1` exits 0 against the live `Cargo.lock` (`image = 0.25.10` confirmed pinned). So `cargo check --locked` will not fail on a lockfile-staleness error; it exercises a genuine 1.88.0 compile of the locked graph.
   - Expected result: GREEN (1.88.0 == the `image@0.25.10` binding floor). A RED here means a dep's real MSRV exceeds 1.88 — which is the drift this gate exists to surface.

4. **No collateral: existing jobs untouched / no lockstep gate fires.**
   - `git diff` must show changes ONLY inside the `msrv` job block in `build.yml` (plus the FOLLOWUPS entry) — the `clippy`, `build`, `release` jobs and the `on:` block are byte-identical.
   - No `schema_mirror` run is needed (no clap flag/value/subcommand change). No `docs/manual/` mirror (no CLI surface). No `Cargo.toml` version edit (NO-BUMP), no `pinned-upstream.toml` touch, no tag.

## 7. Acceptance checklist for the implementer

- [ ] `msrv` job added to `build.yml` exactly as §4 (2-space job indent; action pins copied verbatim from the file; `@1.88.0` ref; `cargo check --locked`; the inline comment incl. the 2-site-edit note).
- [ ] `clippy`/`build`/`release` jobs + `on:` block unchanged (`git diff` confirms isolation).
- [ ] FOLLOWUPS slug `gui-msrv-ci-tested-invariant` filed under `## Resolved` per §5, marked resolved in this commit.
- [ ] `actionlint .github/workflows/build.yml .github/workflows/schema-mirror.yml` → exit 0, no output.
- [ ] (If rustup available) `cargo +1.88.0 check --locked` → exit 0.
- [ ] No version bump, no tag, no `schema_mirror`/manual/sibling-codec touch.
- [ ] Stage paths explicitly: `git add .github/workflows/build.yml FOLLOWUPS.md` (no `git add -A`).