# R0 Round 6 (confirming, post-REFINEMENT) — reproducible-builds brainstorm

**Verdict: 0 Critical / 0 Important / 2 Minor — GREEN.**

Confirms the job-scoped-`[source]` REFINEMENT (user-approved 2026-06-24) folded into the brainstorm reaches 0C/0I. The reviewer **independently reproduced every load-bearing fact on the pinned cargo 1.85.0** rather than trusting the fold:

| Claim | Method | Result |
|---|---|---|
| R0-r5-C1: two-block `--config` FAILS `--offline` for a `[patch.crates-io]` git dep | built a minimal crate with a real `[patch.crates-io]` git dep, vendored, ran two-block vs three-block offline w/ empty isolated `$CARGO_HOME` | CONFIRMED: two-block → `can't checkout… offline mode` (reached live host); three-block → fully offline. Error text matches the brainstorm verbatim. |
| `cargo vendor` emits the `[source."git+…?rev=…"]` block | observed output | CONFIRMED (`.git`/`.rev`/`.replace-with`) |
| committed `vendor/` inert without a `[source]` block | offline build, vendor/ present, no source block | CONFIRMED (cargo ignores it → only the repro build sees vendor/) |
| isolated-`$CARGO_HOME/config.toml` fallback | full 3-block config, `--offline` | CONFIRMED builds offline |
| miniscript `[patch.crates-io]` @ `Cargo.toml:28-29`, `Cargo.lock:700` | git show | CONFIRMED exact |
| `man-pages.yml` `:50`/`:133`/`:135` | sed/grep live | CONFIRMED; `:50` man tarball not in any SHA256SUMS (m3 hygiene-only correct) |

All 6 requested checks PASS; no contradiction with LOCKED decisions (remap-via-ENV, digest-pin, SOURCE_DATE_EPOCH-off-commit-SHA, gzip -n, two-path/P4, cc-under-musl). 2 Minors are cosmetic label/forward-pointer polish (m-A: status line still labels round-5 as the convergence point — now effectively round-6; m-B: §7 cites F4-alt as a remap fallback — add a one-clause reminder that vendoring activation stays job-scoped regardless). Both fixed inline post-review.

**GATE CLEARED — brainstorm 0C/0I GREEN; implementation may proceed.**
