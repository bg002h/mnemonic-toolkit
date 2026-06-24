# POST-IMPLEMENTATION ADVERSARIAL WHOLE-DIFF REVIEW — BSD hygiene parity + FreeBSD compile-gate

**Scope:** the 4-repo lockstep implementing `design/SPEC_bsd_hygiene_and_freebsd_gate.md` Cycle A (BSD secret-hygiene parity arm, PATCH ×4 CLI crates) + Cycle C (FreeBSD compile-gate CI, NO-BUMP ×4 repos).

**Commits reviewed:**
- mnemonic-toolkit `0088dc57` (branch `feat/bsd-hygiene-parity`) → v0.73.1
- descriptor-mnemonic / md-cli `b86c8125` (branch `feat/bsd-hygiene-parity`) → md-cli v0.11.1
- mnemonic-secret / ms-cli `d54f4de5` (branch `feat/bsd-hygiene-parity`) → ms-cli v0.13.1
- mnemonic-key / mk-cli `ca137732` (branch `feat/bsd-hygiene-parity`) → mk-cli v0.11.1

NOTE: the M2 consistency nit (ms-cli's new freebsd job omits `Swatinem/rust-cache@v2`) was investigated post-review and DECLINED with rationale; see the post-review disposition note at the bottom. No code changed as a result. The fold does not alter any verdict in this review.

---

## VERDICT: GREEN — 0 Critical / 0 Important / 3 Minor

The 4-repo lockstep is sound. The `procctl`/`setrlimit` pointer contracts, argument order, and cast soundness are authoritatively confirmed against the FreeBSD libc 0.2.186 signatures and the `procctl(2)` man page. The executable arm and test module are byte-identical across all 4 crates (md5 verified). The sibling FreeBSD gates are whole-crate (not `--lib`); the toolkit's `--lib` choice is correct because its `process_hardening` is `pub mod` in lib.rs. No CLI/clap/argv/`--json`/subcommand surface changed. No findings block ship.

## Critical
None.

## Important
None.

## Minor

**M1 — Toolkit `install.sh` sibling pins lag this lockstep (pre-existing, ungated, not introduced here).**
`scripts/install.sh:35,38,41` still pin `descriptor-mnemonic-md-cli-v0.11.0`, `ms-cli-v0.13.0`, `mk-cli-v0.11.0` while this cycle ships those siblings at `0.11.1`/`0.13.1`/`0.11.1`. This is **expected** — the toolkit's install.sh sibling pins are bumped on a constellation pin-bump cycle, not in a CLI-only PATCH, and the staleness is a known, deliberately-ungated condition (the file itself cites FOLLOWUP `install-sh-gui-sibling-pin-staleness-ungated` at install.sh:57). The toolkit does not claim to vendor siblings at these versions. Non-blocking; flag only so a future reader doesn't mistake it for drift introduced by this change.

**M2 — `ms-cli`'s new CI job omits `Swatinem/rust-cache@v2`; md/mk/toolkit include it.**
The new `freebsd-compile-gate` job in ms-cli `rust.yml` has steps `checkout → rust-toolchain → run`, with no cache step, whereas md, mk, and the toolkit's matrix all cache. Purely a CI-runtime-cost difference (cold compile each run); zero correctness impact. DECLINED post-review — see disposition at bottom: ms-cli's `rust.yml` uses NO `Swatinem/rust-cache` in ANY of its jobs, so the new job is internally consistent with ms-cli's own house style; adding caching to only this job would make it the odd one out within ms-cli. The reviewer's comparison was cross-repo (md/mk/toolkit); the governing convention is the per-repo one.

**M3 — Cross-repo toolchain-action style is heterogeneous but each is internally consistent.**
md/ms use `dtolnay/rust-toolchain@1.85.0`; mk uses `@master` + `toolchain: '1.85'`; toolkit pins `@1.85.0` via its matrix. Each new job matches that repo's own pre-existing convention. All pin Rust 1.85 effectively. Not a defect — noting only that the constellation has no single house style for the toolchain action.

---

## SPEC-MANDATED (a)–(f) CHECKLIST

**(a) SAFETY comments accurate for procctl/setrlimit pointer contracts — PASS.**
- `procctl(idtype, id, cmd, data: *mut c_void) -> c_int` confirmed (libc 0.2.186, freebsd). Call site passes `P_PID, 0, PROC_TRACE_CTL, &mut ctl as *mut c_int as *mut c_void` — argument order correct; `data` points to a valid `c_int`; `procctl(2)` confirms PROC_TRACE_CTL's data "points to an integer variable." Kernel reads sizeof(int); `ctl` (stack) outlives the call; no aliasing.
- `setrlimit(resource: c_int, rlim: *const rlimit) -> c_int` confirmed. Call site passes `RLIMIT_CORE, &lim` — `&lim` coerces to `*const rlimit` (correct immutable pointer; not `&mut`). `lim` fully-initialized and outlives the call. No overclaim.
- The `&mut ctl as *mut c_int as *mut c_void` two-step cast is sound (provenance-preserving raw-pointer reinterpret).

**(b) Executable arm + test module BYTE-IDENTICAL across all 4 crates — PASS.**
Executable arm + test module md5-identical on all 4. Only the doc-comment header differs in length (toolkit fuller), the accepted exception.

**(c) No --json / argv / clap-flag / subcommand / dropdown surface changed — PASS.**
Touched-file set is exclusively process_hardening.rs ×4, CI YAML ×4, version sites, CHANGELOG ×4, FOLLOWUPS ×4. No added `#[arg`/`#[command`/`Subcommand`/`ValueEnum`/`--json`/`serde_json`. Manual flag-mirror lint + GUI schema_mirror correctly untouched.

**(d) No test binds rlim_cur to u64 / assumes rlim_t signedness on FreeBSD — PASS.**
`std::mem::zeroed::<libc::rlimit>()` + `assert_eq!(lim.rlim_cur, 0)` — bare `0` literal, signedness-agnostic. Executable arm `rlim_cur: 0, rlim_max: 0` also bare literals. Footgun avoided.

**(e) Sibling FreeBSD gate is WHOLE-CRATE (not --lib) for each of md/ms/mk — PASS.**
- md: `cargo check --target x86_64-unknown-freebsd -p md-cli`
- ms: `cargo check --target x86_64-unknown-freebsd -p ms-cli`
- mk: `cargo check --target x86_64-unknown-freebsd -p mk-cli`
None use `--lib`. All 3 siblings have NO `src/lib.rs` and declare `mod process_hardening;` in `main.rs` (md:10, ms:23, mk:13) — so whole-crate is mandatory; `--lib` would be silent false-green. The toolkit's `--lib` (rust.yml:265) is the correct special case: `pub mod process_hardening` at lib.rs:101 is UNGATED, so `--lib` genuinely compiles the BSD arm. `set_non_dumpable` callsite exists in all 4 mains (symbol is live, not dead).

**(f) BSD runtime asserts are compile-gated-only / never executed by CI, made explicit — PASS.**
Test module carries an explicit block-comment in all 4 crates ("COMPILE-GATED ONLY … NEVER executed by the CI … documentation / future-native-VM scaffolding, not a live runtime gate"). CHANGELOG + FOLLOWUPS repeat "compile-checked but never executed by the chosen CI." No future reader can mistake green CI for runtime verification of `status == -1` / `rlim_cur == 0`.

---

## Additional adversarial checks (all PASS)
- cfg-arm OS targeting: outer `#[cfg(any(freebsd, openbsd, netbsd))]`; inner procctl `#[cfg(freebsd)]`. setrlimit on all 3 BSDs; procctl FreeBSD-only. Cannot fire on Linux/macOS/Windows.
- aarch64 apt-install guard `if: matrix.target == 'aarch64-unknown-linux-gnu'` — does NOT fire on the freebsd row (no cross-compiler needed; check stops before link).
- procctl PROC_TRACE_STATUS assert: man page confirms `-1` == tracing disabled; `assert_eq!(status, -1)` correct (documented precondition: no external tracer attached).
- Version-site completeness: toolkit 0.73.1 across Cargo.toml + root README + crate README + install.sh self-pin + Cargo.lock + fuzz/Cargo.lock + CHANGELOG. Siblings each Cargo.toml + Cargo.lock + CHANGELOG (mk's at `crates/mk-cli/CHANGELOG.md`).
- CI structural integrity: mk `release-on-tag` intact after the inserted `freebsd-compile-gate`; no YAML displacement. ms's new job has a trusted-context note. No untrusted-input injection in any new job.
- libc uniformity: 0.2.186 locked identically in all 4 — no skew, no bump.
- FOLLOWUPS status flips correct: 3 toolkit slugs RESOLVED w/ companion cross-cites; `ms-cli-ungated-mod-mlock-windows-asymmetry` filed `open` / note-only.

**Recommendation: SHIP.** No open Critical or Important findings; the 3 Minors are all pre-existing-pattern or optional-consistency items that do not gate GREEN.

---

## Post-review disposition (M2 — DECLINED)
The M2 consistency nit was investigated and DECLINED (no code change). Rationale: ms-cli's `.github/workflows/rust.yml` uses NO `Swatinem/rust-cache@v2` in ANY of its jobs (`fmt`, `test-ms-codec`, `clippy-ms-codec`, `test`, `test-release-mlock-einval`, `miri`, `clippy`, `g6-invariant`). The new `freebsd-compile-gate` job is therefore internally consistent with ms-cli's own house style — adding caching to only this one job would make it the lone outlier within the workflow. The reviewer's comparison was cross-repo (md/mk/toolkit cache; ms does not); the governing convention is the per-repo one, and ms-cli's per-repo convention is no caching. M1 and M3 are likewise non-actionable (M1 = pre-existing, deliberately-ungated sibling-pin staleness bumped only on constellation pin-bump cycles; M3 = each new job correctly mirrors its own repo's toolchain-action convention). No Minor required a code change; the commits stand as originally made.
