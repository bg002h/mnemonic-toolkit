# Independent adversarial whole-diff review — BSD secret-hygiene cycle (Cycle A + C)

**Verdict: SHIP OK. critical=0, important=0, minor=3.** (Independent reviewer; did not write the code.)

```
critical: 0  important: 0  minor: 3
ship_ok: true  freebsd_compiles: true  linux_unchanged: true  arm_byte_identical: true
```

Branches reviewed: mnemonic-toolkit @ e6ce50e5 (v0.73.1), descriptor-mnemonic/md-cli @ b86c8125 (v0.11.1),
mnemonic-secret/ms-cli @ d54f4de5 (v0.13.1), mnemonic-key/mk-cli @ ca137732 (v0.11.1).

## Verified (all PASS)

1. **Syscall-arm correctness — re-derived against authoritative man pages + locked libc 0.2.186, not trusted.**
   - `procctl(P_PID, 0, PROC_TRACE_CTL, &ctl)` with `ctl=PROC_TRACE_CTL_DISABLE` correct: `procctl(2)` "P_PID — id zero is a shortcut for the calling process" → id=0 self-targets (not a latent getpid bug). `PROC_TRACE_CTL_DISABLE=2`, `_DISABLE_EXEC=3` — value 2 is the correct minimal choice (binary never execve's). Signature matches; `id_t=i64` on freebsdlike, literal 0 coerces.
   - `setrlimit(RLIMIT_CORE, {0,0})` correct on all 3 BSDs (`RLIMIT_CORE=4` in freebsdlike+netbsdlike; takes `*const rlimit`; unprivileged may always lower; anti-core guarantee is `core(5)`).
   - SAFETY comments accurate; arm has no return/panic/unwrap/?; purely best-effort (ignores returns), matching the existing prctl contract.
2. **No regression.** New arm `#[cfg(any(freebsd,openbsd,netbsd))]` gated OFF on Linux/macOS/Windows; Linux prctl arm untouched; macOS/Windows still no-op. Full Linux suite `cargo test -p mnemonic-toolkit` exit 0, 197 ok suites, 0 failed (incl. set_non_dumpable, lint_safety_*, lint_zeroize_discipline, lint_argv_secret_flags, both_readmes_carry_current_version_marker @0.73.1). Call site unchanged: first statement of main() in all 4, before Cli::parse() and any secret work.
3. **Byte-identical arm.** `pub fn set_non_dumpable` onward → md5 `5df49672` identical across all 4. Only divergence: toolkit's 3 extra module-doc-comment lines (pre-existing argv-overwrite-after-parse reference) — explicitly out-of-scope per SPEC.
4. **FreeBSD compile-gate real, not false-green.** `rustup target add x86_64-unknown-freebsd`: toolkit `--lib` compiles (pub mod in lib.rs); md/ms/mk whole-crate `cargo check --target ... -p <crate>` genuinely compile the bin. FALSE-GREEN PROOF: `cargo check --lib -p md-cli` → exit 101 "no library targets" (so a sibling --lib would fail CI → whole-crate required). Also compiled the arm for x86_64-unknown-netbsd (validates the u64 rlim_t side). actionlint 1.7.12 on all 4 workflows: 0 findings.
5. **Version sites + publish-readiness.** Toolkit ritual complete (Cargo.toml, Cargo.lock, fuzz/Cargo.lock, both README markers, install.sh self-pin, CHANGELOG). Siblings Cargo.toml+Cargo.lock+correct per-crate CHANGELOG. Codec libs NO-BUMP (md-codec 0.39.0/ms-codec 0.7.0/mk-codec 0.4.0). All 3 `cargo publish --dry-run` reached "aborting upload due to dry run".
6. **SemVer + FOLLOWUPS.** PATCH correct (BSD-only behavior change, no surface). Cross-repo companions filed + flipped RESOLVED (bsd-process-hardening-parity-procctl-rlimit-core, freebsd-compile-gate-ci). Note-only ms-cli-ungated-mod-mlock-windows-asymmetry filed open/not-fixed. rust-yml-stale-ring-comment RESOLVED.
7. **Scope.** No DECLINED-set creep (no vmactions/cirrus/qemu/*-vm/BSD-binary). Hardening runs early enough.

## Minor (non-blocking, do NOT gate ship)
- **m1** (all 4, cosmetic): comments call a sibling `--lib` "silent false-green"; ground truth it ERRORS (exit 101), i.e. fails loud not silent. Conclusion (whole-crate) + impl correct.
- **m2** (mk-cli): freebsd job uses `@master` + `'1.85'` vs others' `@1.85.0` — matches mk-cli's own house style (consistent per-repo mirroring, not drift).
- **m3** (doc): the precise citation for "limit 0 ⇒ no core" is `core(5)`, not `setrlimit(2)`. Behavior correct.

No Critical/Important. The 3 crates.io publishes + 4 tags are clear to proceed.
