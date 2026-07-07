# Post-Implementation Whole-Diff Review — Cycle A descriptor use-site collapse fix (`8c8b9183..b59b7a47`)

**Reviewer:** opus architect (MANDATORY post-impl adversarial whole-diff gate). Full suites re-run; exact CI clippy re-run. Persisted verbatim per CLAUDE.md.
**Verdict:** NOT GREEN — 0C, 1I (+2 MINOR). **I-1 FOLDED + confirmed GREEN by orchestrator** (see fold note at end).

## CRITICAL — None. Funds-safety core sound.
Adversarial analysis of the residue check in full context (regex → H13 multipath validator → residue check → `resolve_placeholders` per-`@i` override → `make_use_site_path` → `substitute_synthetic`):
- **Terminator set complete/correct.** No VALID md1-representable descriptor has a placeholder legally followed by a non-terminator: single-sig→`)`; multi→`,`/`)`; taproot `tr(K,{a,b})`→`,`/`}`/`)`; `#csum` only ever follows the closing `)` (pinned by the hash-guard cell). 9 positive-control cells lex-pass → no over-rejection.
- **No under-rejection / no residual silent-collapse.** `/0/*`, `/0h/*`, `/**` (wild eats `/*`, leaves `*`), `/<0;1>/0/*`, `/0/<0;1>/*`, bracketed `@0[..]/0/*`, bare unbracketed origin all reject (8 unit cells). Per-occurrence `@1` dirty while `@0` clean rejects on `@1`.
- **Ordering/panic-safety verified.** After multipath `.transpose()?` (H13 byte-exact reject preserved); `caps.get(0)` present on match; char-boundary slice; UTF-8-safe; `@\d+` mandated (no empty matches / infinite loop); early-return fails closed.
- **Every caller covered.** Grepped 20+ call sites. (a) `bundle --descriptor`/`--import-json` (`concrete_keys_to_placeholders` preserves the suffix outside the key-regex → residue survives → reject); (b) all wallet-import formats; (c) `verify-bundle` both forks (concrete→exit2 `DescriptorParse`; `@N`-template→exit4 `DescriptorReparseFailed`). No pre-lex normalize/strip. **verify-bundle concrete false-pass genuinely closed** (reject before card compare).
- **Sparrow "unaffected" verified in code** (`sparrow.rs:380-392`): substitutes hard-coded `[fp/path]xpub/<0;1>/*` before `parse_descriptor` → never reaches the reject; stays in the passing convergence set.

## IMPORTANT
**I-1 — diff fails `cargo clippy --all-targets -- -D warnings` CI gate (release blocker).** `tests/cli_import_wallet_bitcoin_core.rs:920` doc-comment line began with `+ ` → Markdown bullet → `doc_lazy_continuation` flags lines 921-924 → clippy exit 101 (4 errors). `cargo test` compiles clean (clippy-only lint), so per-phase reviews (which ran `cargo test`) missed it — exactly the class the whole-diff review catches. `origin/master` still at `8c8b9183` (not pushed), so caught pre-CI. Fold (mechanical, one line): reword line 920 so it doesn't start with a Markdown bullet. Reviewer verified `+ `→`plus ` locally → `cargo clippy --all-targets -- -D warnings` exit 0, then restored pristine. Only bullet-initial doc line in the diff.

## MINOR
- **M-1 — bitcoin-core end-to-end round-trip/convergence coverage withdrawn for the interim.** roundtrips → exit-2 reject; bitcoin-core removed from C1-C4 + H1 hop. Correct + documented (prior "round-trips" asserted the buggy collapse); multisig canonicalize/`unified_diff` still covered via the other 5 formats. `bitcoin-core-receive-change-pair-merge` follow-up restores it. Not blocking.
- **M-2 (nit) — loose stderr assertion** in `cli_bundle_import_json.rs` replay cell (`contains("re-parse failed") || contains("import-json")`). Exit pinned 2; cell still pins exit2 + multipath remedy. Cosmetic.

## Coherence / accuracy spot-checks (all pass)
- Fixtures: 3 swapped per-key-identical `<0;1>/*` with VALID recomputed checksums (`#4e664lgt`, `#5ql5mvwg`), counts preserved (2/4/2); `core-mainnet-receive-change-pair.json` untouched (still `/0/*`+`/1/*`, now asserted reject, kept as pair-merge input).
- Exit codes (`error.rs:597/598/610`): `DescriptorParse=2`, `DescriptorReparseFailed=4`, `ImportWalletParse=2` — match all tests + manual claims.
- Migration = no funds-weakening: every silently-collapsing shape now covered by a dedicated per-surface reject cell + A4/A5 assert reject on BOTH legs (never `<0;1>`-swapped). Funds-proof file re-derives TRUE (`bc1qcr8te4k…`) + WRONG (`bc1q8vph849…`) oracles, proves disjoint, proves correct `<0;1>/*` restores true addr through the previously-buggy pipeline.
- Docs: every load-bearing claim correct (exit2/exit4 split; Core receive=0/change=1 combine workaround; Specter receive-only; Sparrow `/**`→multipath). No funds-unsafe steer. Anchors resolve. Deferred merge consistently represented (code/tests/docs); nothing implies it exists.

## Suites: `mnemonic-toolkit` 3583 pass / 0 fail / 16 ign; `wc-codec` 100 / 0. clippy: FAILS as-is (I-1); PASSES with fold.

## VERDICT: NOT GREEN — 0C, 1I. Apply the I-1 fold, re-run clippy + suite → release-ready.

---
## FOLD NOTE (orchestrator, post-review):
I-1 folded — `cli_import_wallet_bitcoin_core.rs:920` `+ `→`then import via ` (no longer a bullet). M-2 also tightened (`|| ` dropped → `stderr.contains("import-json")`, the confirmed-present scope). CONFIRMED: `cargo clippy --all-targets -- -D warnings` = exit 0; `cargo test -p mnemonic-toolkit` = 3583/0; `cargo test -p wc-codec` = 100/0; changed cells `bundle_import_json_fixed_step_descriptor_replay_rejects` + `core_fixture_file_mainnet_receive_change_pair_parses` pass. Fold is doc-comment + cosmetic-assertion only (no logic/funds change); reviewer pre-verified the I-1 fix clears clippy. Whole-diff GREEN. Current version = v0.75.0 → MINOR bump v0.76.0.
