## R0 Adversarial Review — SPEC `A2-gui-msrv-job` (`gui-msrv-ci-tested-invariant`)

**VERDICT: GREEN — 0 Critical / 0 Important / 4 Minor. Cleared to implement.**

CI-only YAML + FOLLOWUPS entry, NO-BUMP / NO-TAG. Every load-bearing claim was re-grepped/re-verified against live source (not the recon doc); the proposed YAML was physically constructed and run through actionlint; the one external protocol fact (`dtolnay/rust-toolchain@<ver>` ref form) was verified against the action's own README.

### SHAs / grounding (verified)
- `mnemonic-gui` origin/master = HEAD = `e7f0c63f9dcb…` — clean, 0 ahead / 0 behind. Matches spec's `e7f0c63`.
- `mnemonic-toolkit` origin/master = `572a15d18c5a…`. Matches spec's `572a15d1`.

### Claim-by-claim verification

| Spec claim | Verified | Result |
|---|---|---|
| `Cargo.toml:8` = `rust-version = "1.88"` | `git show e7f0c63:Cargo.toml` | ✓ exact (line 8) |
| clippy `@stable` at build.yml:22 | grep | ✓ :22 |
| build matrix `@stable` at build.yml:73 | grep | ✓ :73 |
| schema-mirror `@stable` at :24 | grep | ✓ :24 |
| NO other rust-toolchain pin in `.github/workflows/` | grep all | ✓ exactly 3, all `@stable` |
| `checkout@v5` at :19/:56 | grep | ✓ |
| `rust-cache@v2` at :27/:78 | grep | ✓ |
| `image@0.25.10` is binding floor, `rust_version = 1.88.0` | Cargo.lock + cargo registry `image-0.25.10/Cargo.toml` | ✓ `1.88.0` |
| icu_*@2.2.0 / idna_adapter@1.2.2 only `1.86` (satisfied w/ margin) | registry Cargo.tomls | ✓ both `1.86` |
| `FOLLOWUPS.md` at repo ROOT; NO `design/FOLLOWUPS.md` | `git ls-files \| grep -i followup` | ✓ single root file |
| `## Resolved` header at line 793 | grep | ✓ :793 |
| `## Resolved in v0.2` separate header exists | grep | ✓ :726 (distinct) |
| toolkit companion `install-sh-gui-sibling-pin-staleness-ungated` exists | `572a15d1:design/FOLLOWUPS.md` | ✓ :278 |
| `dtolnay/rust-toolchain@1.88.0` is documented valid ref | action README (`@1.89.0` example) | ✓ authoritative |
| §4 indentation matches existing jobs (2/4/6/8) | `cat -A` clippy job | ✓ identical |
| actionlint baseline clean | actionlint 1.7.12 on both files | ✓ exit 0 |
| Lockfile resolves under `--locked` | `cargo metadata --locked` | ✓ exit 0 |

### Load-bearing-design checks (the items the task flagged)

1. **`dtolnay/rust-toolchain@1.88.0` (exact pin, not `@stable`)** — present in §4 as `uses: dtolnay/rust-toolchain@1.88.0` with `name: install-rust-1.88.0`. The `@1.88.0` ref form is the action's documented version-pin (README: "`dtolnay/rust-toolchain@1.89.0` pulls in 1.89.0"). **PASS.**

2. **`cargo check --locked` (the `--locked` is load-bearing)** — §4 step `run: cargo check --locked`. The spec's rationale (§2) is correct: without `--locked`, cargo re-resolves and could pull a newer `image`/icu that raises or masks the binding MSRV, defeating the test; `--locked` forces the committed-lockfile graph so the proven MSRV is the one users get. Lockfile confirmed resolvable under `--locked`. **PASS.**

3. **Single ubuntu job, NOT the matrix, NOT tests** — `runs-on: ubuntu-latest` only; default `cargo check` (no `--all-targets`, so no test/bench/example tree, no dev-deps dragged into the MSRV floor). The `--all-targets`-omission rationale is sound: this is a single binary crate, default `check` covers lib+bins, which is the shipped-code MSRV question. **PASS.**

4. **Existing `@stable` jobs untouched** — §3 + §4 + §7 explicitly fence `clippy`, `build` matrix, `release`, and the `on:` block as byte-identical, with a `git diff`-isolation acceptance check (§6.4, §7). I constructed the post-edit file and confirmed only the `msrv` block is inserted (between the clippy job's last step and `build:`), no edits elsewhere. **PASS.**

5. **Slug filed in the CORRECT registry** — `mnemonic-gui/FOLLOWUPS.md` at repo root is the *only* FOLLOWUPS file (no `design/FOLLOWUPS.md` in this repo, unlike toolkit). Spec §3 calls this out explicitly and correctly. **PASS.**

6. **`1.88.0` is the right pin** — `image@0.25.10` (`rust_version = 1.88.0`) is the binding floor; the next-highest in the family (icu_*@2.2.0, idna_adapter@1.2.2) is `1.86`. So `1.88.0` = the actual floor → GREEN at zero margin today, RED on any future drift above it. The "zero-margin, load-bearing from day one" framing is empirically correct. **PASS.**

7. **actionlint clean** — actionlint 1.7.12 on the physically-constructed `build.yml` (msrv job inserted): **exit 0**, with and without shellcheck. No actionlint config in the repo (defaults). The new job introduces no construct not already present in the file. **PASS.**

8. **YAML well-formed** — verified via actionlint's parse (PyYAML absent on host; spec correctly notes actionlint subsumes this). **PASS.**

### Adversarial probes that did NOT find defects
- **Hidden ordering gate on FOLLOWUPS.md?** None — nothing in `.github/` or `scripts/` references FOLLOWUPS, so the §5 top-of-section placement violates no enforced convention.
- **Does the toolkit already cover the GUI-CI piece (making this redundant)?** No. Toolkit HEAD `572a15d1` (L5) shipped only an `install.sh` skip-with-warning guard + README/manual prose and explicitly left "GUI Cargo.toml paired-PR" OPEN. The CI-test-the-MSRV piece is genuinely untracked on the GUI side — this spec's scope is real and non-overlapping.
- **Lockstep-gate collateral?** None fires — no clap flag/value/subcommand (no `schema_mirror` delta), no CLI surface (no `docs/manual/` mirror), no sibling-codec companion, no `Cargo.toml` version touch, no `pinned-upstream.toml`. The §6.4 isolation check enforces this.
- **External ref-form risk?** `@1.88.0` confirmed documented/valid against the action README (not assumed from the recon doc).

### Minor findings (none gating; no fold required)
1. **Precedent-citation spatial imprecision (§5):** the cited precedent `canonicity-drift-gate-floor-too-lenient` lives at line 944, not "immediately after `## Resolved`" (that's `gui-runner-debug-logs-…` v0.45.0). The placement instruction (top of Resolved) is itself clear and convention-OK; the precedent is correct for *entry form* (resolved/no-bump, Tier: cross-repo, Companion). Cosmetic prose conflation only.
2. **§1 Cargo.toml quote elides lines 5–7**, which are a 3-line comment already documenting the icu/idna/image floor — corroborating evidence the spec could have cited; the `...` covers it and line 8 is exact.
3. **§3 optional toolkit cross-ref** is now *more* accurate post-`572a15d1` (the toolkit MSRV sub-entry flipped but left the GUI CI piece OPEN) — correctly marked skippable for a CI-only GUI cycle.
4. **§6 step 2 (PyYAML)** not reproducible on this host (no `yaml` module); spec already routes around it via actionlint, which I confirmed parses the file.

### Conclusion
Implementation-ready. The only new gate is the `msrv` CI job itself; `actionlint` stays clean; no funds/wire/clap/manual-mirror surface; NO-BUMP/NO-TAG. The four Minors are cosmetic/informational and require no edit before coding. **Gate GREEN — proceed to implementation.**