# Phase 4 Root + Glue Review — r1

**Date:** 2026-05-04
**Commit under review:** `05909c4` (parent: `b5e63b1`)
**Reviewer:** opus phase-review

## Verdict

0 critical / 1 important / 3 low / 2 nits

I-1 fixed inline at `ffd72bd`; r2 confirms.

## Critical

(none)

## Important

### I-1: SPEC §2.2.2 stderr warning silently dropped in watch-only `verify-bundle`

**File:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs`

`verify_bundle::run` signature has no `stderr` parameter, so the SPEC §2.2.2 3-line warning is unemittable. SPEC §2.2.2 lines 154-159 mandate:

```
warning: watch-only verify-bundle does not verify --xpub is actually at the
warning: claimed BIP path m/<purpose>'/<coin>'/0' (no master seed available
warning: for re-derivation). Use --phrase mode for end-to-end verification.
```

**Fix (applied at `ffd72bd`):** Added `stderr: &mut E: Write` to `run` signature; threaded through to `run_watch_only`; emitted the 3-line warning at the top of `run_watch_only` (before any parse error). main.rs dispatch updated. `run_full` unchanged (SPEC §2.2.1 has no analogous stderr warning).

## Low / Nit (defer to design/FOLLOWUPS.md)

- **L-1:** SPEC §2.2.2 prose says "four checks" but §5.4 schema requires 9 with skipped semantics. Internal SPEC inconsistency; implementation correctly follows §5.4.
- **L-2:** `BundleMismatch.card` typed `&'static str` — fine for current callers; future runtime-card-id callers would hit lifetime errors.
- **L-3:** `verify-bundle` text-mode output line `"{}: {} {}"` produces trailing-space when `detail` is empty. Cosmetic.
- **N-1:** `error::Result<T>` allow comment says "in-crate use" but `pub type` is exported.
- **N-2:** `BundleMismatch` doc says "Constructed by integration tests in Phase 5" — staleness risk.

## Verified

- §6.5 ExitCode override: clap parse failure → exit 64 (or 0 for --help/--version); ToolkitError::exit_code dispatch for runtime errors.
- §3.1 stderr for errors: `writeln!(io::stderr(), "{}", e)` correct.
- 5+1 narrow `#[allow(dead_code)]` items all justified (reserved per SPEC or used by tests).
- `Cli` parser: `name = "mnemonic"`, correct `about`, `version`. Subcommand doc-comments map to clap help.
- BundleJson / VerifyBundleJson field order matches SPEC §5.3 / §5.4.
- 9-check SPEC §5.4 array names + order match `SPEC_NAMES` constant.
- 52 unit tests + smoke-test confirm CLI works end-to-end; bundle JSON master_fingerprint=`5436d724`; verify-bundle round-trip all 9 checks ok.
