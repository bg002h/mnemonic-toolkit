# Phase P2.4 sub-batch 5a (Track M — 40-mnemonic chapter overview + final-word) — R1 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-v1` (mnemonic-toolkit)
**Scope:** Verify R0 I-1 fold at `docs/manual-gui/src/40-mnemonic/41-overview.md:71-85`; confirm no new drift; lint/build state unchanged.

**Verdict:** **LOCK 0C / 0I / 0N / 1n (carried).**

---

## I-1 fold verification (PASS)

The replacement prose at `41-overview.md:71-85` is source-correct on every axis R0 specified:

- **"Five … five" claim removed.** Replaced with "**Most** of the ten subcommands" (non-numeric framing) plus an explicit "all ten can fire the modal under at least one valid form input" quantifier sourced from the predicate.
- **`should_confirm_run` citation present and accurate.** Cites `mnemonic-gui/src/secrets.rs:80-105`; verified against source — function signature opens at L80, returns at L104 (range fits).
- **Three predicate clauses correctly enumerated:** "any `secret: true` flag value, any secret-class slot subkey, any secret-class NodeValueComposite node" — maps 1:1 to `secrets.rs:85-89` (flag) + `91-95` (slot) + `97-103` (composite).
- **Secret-class node list byte-correct:** `phrase, entropy, xprv, wif, ms1, bip38, electrum-phrase` matches `crates/mnemonic-toolkit/src/cmd/convert.rs:85-95` `is_secret_bearing` true-arm exactly (7 variants, same order).
- **`export-wallet` correctly identified** as the only intended-public-fill subcommand and "most likely to fire **Run** without the modal in practice".
- **§14 Defense 2 cross-reference preserved.**

## Lint / build state (unchanged)

- Phase 4 schema-coverage RED at **445 missing** (unchanged from R0 — fold is purely prose, no anchor changes).
- Phase 5 outline-coverage RED at **57 missing** (unchanged).
- Phases 1-3 GREEN, 6-7 WARN-skip preserved.
- PDF: 50 pages (fold is small prose rewrite; ±0-1 page tolerance).
- HTML: 16 H1 chapters.

## Surrounding prose coherence (PASS)

`41-overview.md:60-90` reads cleanly: form-shape paragraph → secret-bearing paragraph → worked-example seed convention. Transitions smooth; no orphan referents.

## Carried nitpick

**n-1** (refusal table row 4 in `47-final-word.md:210`) — deliberately not folded; remains byte-faithful to CLI manual baseline at `docs/manual/src/40-cli-reference/41-mnemonic.md:283`. Acceptable per R0; CLI-manual-followup recommended for future lockstep correction. Non-blocking for LOCK.

**LOCK — proceed to commit.**
