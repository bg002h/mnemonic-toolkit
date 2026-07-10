# SPEC R0 review — Cycle I test-hardening — round 2 (convergence)

**Reviewer:** Fable (SPEC R0 convergence, read-only), per user directive. SPEC @ toolkit `e6b87323` + sibling HEADs.
**Dispatched:** 2026-07-10 (Cycle I, SPEC R0 round 2). Persisted verbatim per CLAUDE.md.

## VERDICT — GREEN (0 Critical / 0 Important). Round-2 convergence holds; implementation is cleared.

Evidence per round-1 finding:

- **C1 RESOLVED.** Toolkit `rust.yml` (`e6b87323`) has `paths:` on both push (:14-19) and `pull_request:` (:20-27). The fold deletes ONLY the PR-side `paths:`, leaving `pull_request:` bare — byte-identical to the in-repo precedent (`examples.yml:35` "NO paths — a REQUIRED check must report on every PR"; push keeps its filter). (a) Bare PR trigger → rust.yml runs on every PR → reports `test (ubuntu-latest)` + `clippy`; a docs-only PR compiles unchanged code → green. (b) Push filter staying means docs-only direct pushes don't fire, but `enforce_admins:false` + admin-bypass means non-admins never push to master directly, so no wedge. (c) Code pushes touch `crates/**`/`Cargo.toml`/`Cargo.lock`/the workflow → push filter matches → required context fires. No gap.

- **C2 RESOLVED.** ms `rust.yml` (`ffc9d71`, :19-35) has the identical push+PR `paths:` structure; the four required contexts (`test (ubuntu-latest)`, `clippy`, `test (ms-codec)`, `clippy (ms-codec)`) all originate there. §1b's separate ms CI-only NO-BUMP commit deleting the PR-side `paths:` mirrors C1 and applies cleanly; §3's "md/mk/ms untouched" line is corrected. Re-scan of siblings confirms NO PR path filters on any required-context workflow: md `ci.yml` (`pull_request:` bare → `cargo test (ubuntu-latest)`/`cargo clippy`), mk `ci.yml` (`pull_request:` bare → `build (stable on ubuntu-latest)`), gui `build.yml` + `schema-mirror.yml` (`pull_request: branches:[master, release/**]`, no paths → `clippy`, `headless (no-default-features)`, `x86_64-unknown-linux-gnu`, `snapshots`, `schema-mirror gate`). md/mk/gui need no CI edit — correct.

- **I1 RESOLVED.** `cargo metadata` workspace_members = exactly `mnemonic-toolkit` + `wc-codec` (2); `crates/wc-codec/fuzz/Cargo.toml:17` carries its own `[workspace]`, invisible to root `--workspace`. So `--workspace` at rust.yml:105/:112 covers both members' tests without the fuzz sub-crate; the JOB name `test (${{ matrix.os }})` → `test (ubuntu-latest)` is unchanged by the step edit (G5 holds). Reasoning from the 2-member metadata + round-1's already-green `cargo test -p wc-codec` (100 tests): the union is green (coordinator confirming empirically in parallel).

- **I2 RESOLVED.** fuzz-smoke.yml push+PR `paths:` extended with `crates/wc-codec/fuzz/**` + `crates/wc-codec/src/**` — the two drift origins; adequate. Not a required context, so no wedge either way.

- **M1/M2/M3 RESOLVED.** PUT shape `checks:[{"context":…,"app_id":15368}]` is the correct GitHub API form; live GET confirms existing toolkit `[examples]` and gui `[snapshots]` are `app_id:15368` (GitHub Actions), so the binding matches. Citations corrected to build :51-73 / smoke :75-104. Matrix-rename note (`cargo fuzz run (60s smoke) (<target>)`) captured as harmless/never-required.

Convergence confirmed: §2 matrix unchanged and still string-exact/green; ordering (#2 CI edits land + verify green BEFORE the toolkit/ms PUTs) is coherent and is the precondition that closes the wedge; no new wedge or gap introduced by the folds. **Cleared for implementation.**
