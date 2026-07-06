# Per-phase R0 review — Cycle A Phase 1 (residue-reject floor) — Round 2 (convergence, test-only fold)

**Reviewer:** opus architect. **Fold:** `72e82d1c..27f27bed` (test-only, 262 insertions / 0 deletions, 4 files). Read-only main tree; FULL suite re-run in worktree. Persisted verbatim per CLAUDE.md.
**Verdict:** GREEN (0C/0I).

**Suite (verified):** `mnemonic-toolkit` **3578 passed / 0 failed / 16 ignored** (+5 vs round-1); `wc-codec` **100/0**. All 5 new cells ran + passed: `descriptor_fixed_use_site_step_rejected_with_multipath_remedy`, `descriptor_double_star_shorthand_rejected_with_multipath_remedy`, `specter_receive_only_fixed_step_rejected_with_multipath_remedy`, `bundle_import_json_fixed_step_descriptor_replay_rejects`, `bsms_single_branch_fixed_step_rejected_with_multipath_remedy`.

## Anchoring property (why NONE of the 5 is vacuous)
The message substring `/<a;b>/*` is UNIQUE across `src/` — `grep "<a;b>"` returns exactly the residue-reject floor (`parse_descriptor.rs:208`) + one unit-test assertion. No checksum/miniscript/hardened-multipath/card-crosscheck/key-not-found error contains `<a;b>`. Each cell asserts `.failure()` + `code==2` + `stderr.contains("multipath") && stderr.contains("<a;b>")` — so it can ONLY pass if the residue floor fired THROUGH that specific CLI surface (silent-accept fails `.failure()`; earlier-stage reject lacks `<a;b>`). Born-green passes ARE the proof.

## CRITICAL / IMPORTANT — None. Both round-1 findings CLOSED non-vacuously.
- **I-1 CLOSED:** the 2 `cli_import_wallet_descriptor.rs` cells drive `import-wallet --format descriptor` end-to-end with `/0/*` and `/**`. `/**` proven at the CLI surface (not just the lexer unit): `concrete_keys_to_placeholders` is a textual regex substitution (`push_str(&descriptor[last_end..])`, no miniscript expansion of `/**`→`<0;1>`), so `wpkh(@0[…]/**)` reaches `lex_placeholders` → wild eats `/*`, stray `*` → residue reject. Checksum-less blobs correct (`verify_checksum` tolerates absence).
- **M-1 CLOSED:** specter (valid checksum via canonical `Engine`; bad csum would trip `specter.rs:220` sans `<a;b>` → the pass proves it reached the floor); old-`--json` replay (generates `<0;1>` envelope, mutates ONLY `bundle.descriptor` to `/0/*`+recomputed csum, replays → `DescriptorParse("--import-json: descriptor re-parse failed") → exit 2`, the concrete-reparse variant per plan-R0 I-B; doubly non-vacuous — a no-op mutation would SUCCEED and fail `.failure()`); BSMS (`build_bsms_2line` valid csum).

## MINOR
- **M-a (informational):** the message template names BOTH `/0/*` and `/**` as examples, so `contains("/0/*")`/`contains("/**")` are satisfied by the template regardless of input — they don't discriminate the actual residue (which lives in the un-asserted `found residue near {residue}` tail). Does NOT make any cell vacuous (`<a;b>`+exit-2 fully anchors per-surface); satisfies plan-R0 I-D's "message names `/**`". Residue-discrimination per input stays locked by the `parse_descriptor.rs` unit tests. No action.
- M-2/M-3 (round 1) unchanged + already-ruled (multisig Core `--json` → pair-merge follow-up; full workaround text → Phase 3 docs).

## No-collateral audit — CLEAN.
`--numstat` = 4 files, 262 ins / **0 del**, ALL under `tests/`. Zero `src/`, zero fixture, zero existing-test change. Exit mapping verified: `DescriptorParse=>2`, `ImportWalletParse=>2` (`error.rs:597,610`).

## VERDICT: GREEN (0C/0I). Phase 1 advances.
