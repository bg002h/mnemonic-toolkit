# PER-PHASE R0 (P0) — bip388-double-star — round 1

**Verdict: GREEN (0 Critical / 0 Important)**
**Reviewer:** opus architect, worktree `feature/bip388-double-star-shorthand` @ `6cb8b297`, base `e226c3a8`.
**Dispatched:** 2026-07-06 (Cycle C, per-phase R0 over the P0 code diff, funds-weighted). Persisted verbatim per CLAUDE.md.

P0 faithfully executes the R0-GREEN SPEC (rev-5) + plan (rev-3). Full `cargo test -p mnemonic-toolkit` exit 0 (also coordinator-confirmed: 202 ok binaries / 3624 pass / 0 fail), `cargo clippy -D warnings` exit 0. Independently re-verified the 3 changed test binaries: cli_bip388_double_star_shorthand 12/0, cli_gui_schema_classify_descriptor 9/0, cli_import_wallet_descriptor 10/0. **Advance to P1 (docs).**

## Critical — none
## Important — none

## Funds-weighted scrutiny — all SAFE
1. **Expander precision (`expand_literal_double_star`, parse_descriptor.rs:414) — AIRTIGHT.** Rewrites `/**` only when immediately followed by `)`,`,`,`}`,`#`, whitespace, or EOS. `/***`(next `*`)/`/**'`(next `'`) copied through + rejected downstream; `]` excluded so a bracketed `/**` never expands; multi-`/**` multisig expands per-key; `/**`→`/<0;1>/*` is an exact BIP-388 synonym so no rewrite yields a valid-but-wrong wallet. `after = pos+3` always a char boundary (3 ASCII bytes). `changed` flag returns `Cow::Borrowed` when nothing rewritten. 10 unit cells (parse_descriptor.rs:3549-3637) lock every branch incl. `/0/**` floor-not-weakened.
2. **Call-site completeness — 8 IN sites, no 9th.** Each expands BEFORE its `/**`-rejecting parser: parse_descriptor.rs:946 (concrete pipeline + gui-schema classify + canonicity probes), bundle.rs:1395 (AtN, inside `bundle_run_unified_descriptor` before lex@1389), verify_bundle.rs:1354 (before Concrete/AtN split, after JSON expansion — both forks), descriptor_intake.rs:301 (before miniscript from_str), bsms.rs:307 (first-address check), roundtrip.rs:247 (`recanonicalize_descriptor` BSMS `--json` canonicalize), export_wallet.rs:522 (before from_str@517), cost/strip.rs:29 (before from_str@21). The exhaustive `--descriptor` sweep confirmed complete (word-card/nostr/restore `descriptor` fields are serde output).
3. **Equivalence oracles — non-tautological.** §7.3 (cli_bip388_double_star_shorthand.rs:69-116, wpkh/tr/sortedmulti) compares two INDEPENDENT binary invocations (`/**` vs `/<0;1>/*`), nulling only the raw-echo `descriptor` field (not part of the funds property; md1 cards/addresses ARE compared). §7.4 AtN present+passing (bundle + verify-bundle AtN). §7.11 compare-cost (line 380): asserts `/**` code+stderr == `/<0;1>/*` code+stderr AND contains `"multipath key cannot be a DerivedDescriptorKey"` AND NOT `"invalid child number format"` (proves the expander fired). export-wallet asserts byte-identical stdout (genuine acceptance).
4. **Reworded reject message — correct.** parse_descriptor.rs:206-211 drops `(or the /** shorthand)`, keeps `/0/*`. §7.9 asserts a genuine `/0/*` rejects with a message not naming `/**`; §7.10 `/0/**` still rejects (floor intact).
5. **Idempotence — no double-expansion / no non-`/**` collateral.** Borrowed no-op on `/**`-free input (unit-asserted). JSON `@N/**` → `expand_bip388_policy` → `/<0;1>/*` (no `/**` survives); §7.8 pins that path unchanged. Borrowed no-op across all 8 sites on non-`/**` text.
6. **Misattribution — corrected + genuine refs left.** BIP-389→388 at comment :189, reject-message :206-211, sparrow.rs:42, cli_import_wallet_descriptor.rs:159/191; residual-grep empty. Genuine multipath `sparrow.rs:372` left intact.
7. **Determinism / discipline.** Deterministic scan; clippy clean; mlock.rs untouched (g6); error.rs unchanged (no new variant).

## Note (non-blocking, no action)
`expand_literal_double_star` allocates `out` even when a `/**` substring is present but none are terminator-bounded (e.g. a lone `/***`), then discards it → `Cow::Borrowed`. Correct; allocation negligible; degenerate path. Not worth a change.

**Conclusion: GREEN — clear to advance to P1 (docs).** The post-impl whole-diff review remains the mandatory endpoint (its call-site grep should confirm the canonicalize_*/export/compare-cost adjacencies per plan M3).
