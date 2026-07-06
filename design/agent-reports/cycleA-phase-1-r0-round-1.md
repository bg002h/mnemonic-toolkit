# Per-phase R0 review — Cycle A Phase 1 (residue-reject floor + 22-cell migration) — Round 1

**Reviewer:** opus architect. **Commit:** `72e82d1c` (diff `8c8b9183..72e82d1c`, 10 files). Read-only. FULL suite re-run in the worktree. Persisted verbatim per CLAUDE.md.
**Verdict:** NOT GREEN — 0 Critical, 1 Important.

**Suite (verified, not trusted):** `mnemonic-toolkit` **3573 passed / 0 failed / 16 ignored**; `wc-codec` **100 passed / 0 failed**. Matches implementer's claim. Key cells confirmed present+passing: `lex_rejects_fixed_step_bare`, `lex_rejects_double_star_shorthand`, `a4_seed_vs_walletfile_bitcoin_core_singlesig_converge`, `core_fixed_use_site_step_rejected_with_workaround`, `core_fixture_file_mainnet_receive_change_pair_parses`, `core_bundle_roundtrip_bip84_single_sig`.

## CRITICAL — None. No-weakening audit CLEAN.
- **Group A (`a4`/`a5`) PASS** — assert reject on BOTH legs (wallet-file `import-wallet --format bitcoin-core` exit 2 + `"bitcoin-core"`; direct-placeholder seed leg exit 2 + `"multipath"`); `a5` all three starts. Never swapped to `<0;1>`. The `assert_cards_converge` collapse premise fully deleted.
- **`:898` PASS** — flipped `bundles=2 .success()` → `.failure()` exit 2 + `"bitcoin-core"` + multipath remedy; `core-mainnet-receive-change-pair.json` UNCHANGED (preserved as pair-merge follow-up input).
- **Group B PASS** — every swapped cell keeps feature assertion + entry/bundle count (`bundles=4`/`bundles=1`+FP; active-receive/change driven by `active`/`internal` not descriptor content). `d1=d0.clone()` per-key-identical `<0;1>/*` does not dedup. No count weakening.
- **Every reject SHAPE has a dedicated positive unit reject test** (bare `/0/*`, `/0h/*`, bracketed-origin, post-mp, pre-mp, bare-unbracketed-origin, `/**`, non-first multisig slot).

## IMPORTANT
**I-1 — Missing end-to-end `import-wallet --format descriptor` reject tests for `/0/*` AND `/**`** (plan Phase-1a deliverable + plan-R0 I-D; highest-impact surface, zero CLI coverage). The diff delivers dedicated CLI reject tests ONLY for bitcoin-core (`core_fixed_use_site_step_rejected_with_workaround`, `core_receive_change_pair_rejected_with_workaround`). Verified: NO end-to-end `--format descriptor` test asserts a `/0/*` or `/**` reject anywhere. `/**` is the BIP-389 canonical shorthand + a most-common real shape this cycle turns into a hard failure; plan-R0 I-D flagged it possibly higher-impact than the whole pair-merge follow-up. The unit test proves the LEXER rejects `/**` but nothing locks the `--format descriptor` CLI surface surfaces exit 2 + workaround. Fold: add two `import-wallet --format descriptor` reject cells — `wpkh([fp/84'/0'/0']xpub…/0/*)` and `…/**` — asserting exit 2 + stderr `multipath`/`<a;b>`, and for `/**` naming the `/**` shorthand + `<0;1>/*` + `--format descriptor` workaround.

## MINOR
- **M-1** — Other Phase-1a per-surface reject tests absent (specter receive-only `/0/*`, old-`--json` replay `/0/*`, BSMS single-branch `/0/*`). Lower severity (shared choke point `lex_placeholders`, proven by unit + bitcoin-core CLI; old-json direct-`@N` path exercised by a4/a5; no Group-B swap on these surfaces so no hole created). Recommend adding before ship; not gate-blocking.
- **M-2** — `core_bundle_roundtrip_wsh_sortedmulti_2of2_envelope_semantic_match` conversion thins multisig `canonicalize_bitcoin_core` `--json` coverage (single-sig survives via P11D on the `<0;1>` `core-bip84-mainnet.json`). Non-funds (diagnostic field). Naturally restored by the pair-merge follow-up; optionally add a hand-built `<0;1>` multisig Core `--json` positive control. Defer to follow-up.
- **M-3** — Deviation 1 (message specificity): generic message used, not SPEC §6's hand-written procedure. Acceptable (see ruling); full workaround lands Phase 3 (manual+CHANGELOG).

## Ruling — Deviation 1 (message specificity): ACCEPTABLE for Phase 1.
Message is actionable + test-locked (bitcoin-core stderr contains `"bitcoin-core"`+`"multipath"`+`"<a;b>"`+`"/0/*"`). The full combine-and-`--format descriptor` procedure belongs in manual+CHANGELOG (mandatory Phase-3 per SPEC §9/plan §3). Condition: Phase 3 carries the full workaround text.

## Ruling — the 11 extra grep-missed cells: assert-reject / exclusion CORRECT.
- **6× `core_bundle_roundtrip_*`** imported `export-wallet --format bitcoin-core` output = Core's NATIVE split `/0/*`+`/1/*` (Core never exports combined — PR #22838). Pre-fix they asserted `bundles=2`/`semantic_match:true` on a COLLAPSED (corrupted) wallet — false-green covering the bug. Post-fix the split correctly rejects. A `<0;1>` swap would be WRONG (would need the exporter to emit a non-native shape). Real round-trip restored by the pair-merge follow-up. Export-side + selection + single-sig canonicalize coverage preserved. Only residual thinning = M-2.
- **5× `cli_wallet_cross_format_convergence.rs` (c1-c4, h_hop)** — bitcoin-core's native split rejects on re-import → genuinely can't participate until the follow-up. Convergence preserved for the remaining ≥3 formats per set; the removed bitcoin-core reject is covered by a4/a5 + the bitcoin-core reject cells. No property loosened.

## Other verified: residue check placement (after `.transpose()?` validator :177, before `out.push`) preserves H13; panic-safe (char-boundary offsets, codepoint-wise); `#`-guard passes; `lex_bare_at_zero` UNCHANGED. Fixture checksums independently recomputed valid (`#5ql5mvwg`, `#4e664lgt`); `core-mainnet-receive-change-pair.json` untouched. Over-rejection controls green (`:915` distinct-keys `bundles=2 .success()`; export/compare bypass controls).

## VERDICT: NOT GREEN — 0C, 1I. Fold I-1 (+ ideally M-1), re-run full suite, re-dispatch. On GREEN the phase advances.
