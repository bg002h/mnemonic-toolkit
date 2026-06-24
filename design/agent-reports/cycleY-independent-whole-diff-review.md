# Independent adversarial whole-diff review — Cycle Y (toolkit v0.73.3)

**Verdict: SHIP OK — GREEN (0 Critical / 0 Important / 1 Minor).** Independent reviewer (did not write the code); verified every claim against the compiled binary + live suite.

```
critical: 0  important: 0  minor: 1
ship_ok: true  fires_on_custom: true  silent_on_baseline: true  restore_still_succeeds: true
```

Branch feat/cycle-y-taproot-advisory @ 4b424e73 (base master 2f685f09).

## Verified independently
1. **Predicate** — `custom_use_site_nums_taproot_card = taproot_override_card(d) && restorable_taproot_override_card(d)` (= `Tag::Tr ∧ use_site_path_overrides.is_some()` ∧ non-hardened NUMS-internal plain-MultiA). EXACT complement of the calm `override && !restorable`. Empirically TRUE for `tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<2;3>/*))`; FALSE for baseline-uniform, sortedmulti_a, non-NUMS-trunk, hardened, non-taproot wsh. 6 truth-table + mutual-exclusion tests pass.
2. **Fire sites** — 5 collect+emit pairs adjacent to the existing unrestorable emit: bundle.rs ×3 (unified/concrete/from-import-json, `&descriptor`), import_wallet.rs (`&p.descriptor` in the parsed loop), restore.rs `fn run_multisig` (`&d`, tail of the SUCCESS path before Ok(0)). Un-restorable subset returns early at restore.rs:3116, so only restorable cards reach the emit. End-to-end: restore exits 0, warns on stderr, STILL reconstructs @1's `<2;3>/*` on stdout (proceed-and-warn). No double-emit, no missed surface, no stdout leak.
3. **Text** — byte-exact to spec §3.3 (`WARNING (funds-safety):` … `PERMANENT LOSS OF FUNDS` … verify-against-wallet), on stderr, textually distinct from the calm `advisory:` sibling.
4. **No regression** — full `cargo test -p mnemonic-toolkit` 0 failed; 15 new tests; calm advisory + all paths unchanged; baseline silent; clippy clean.
5. **Version sites** — all 7 at 0.73.3 (Cargo.toml, Cargo.lock, fuzz/Cargo.lock, both READMEs, install.sh:32 self-pin, CHANGELOG [0.73.3]); Cycle X sibling rows (md v0.11.2/ms v0.13.2/mk v0.11.2) UNTOUCHED; install-pin-check/changelog-check/readme_version_current PASS.
6. **Scope** — no clap flag (no schema-mirror); FOLLOWUP restore-md1-taproot-use-site-override-arm stays PARTIALLY RESOLVED (sub-note "no reconstruction gap closed", NOT flipped); manual subsection + anchor accurate.

## Minor (non-blocking)
- `FundsSafetyShape`/`FundsSafetyAdvisory`/collectors are `pub` but crate-internal — mirrors the existing `UnrestorableAdvisory` `pub` API in the same module; intentional consistency.

No Critical/Important. Cleared to tag mnemonic-toolkit-v0.73.3.
