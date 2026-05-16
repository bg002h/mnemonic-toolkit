# Phase P2.4 sub-batch 6b (Track M — md encode) — R0 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-v1`
**Scope:** 6b — `53-encode.md` (NEW, ~228 LOC, 11 flags + 1 positional + 2 enumerated-flag outlines); `.cspell.json` (+1 word `unchunked`).

**Verdict:** **LOCK 0C / 0I / 0N / 4n.** Three nits folded for polish; one (N-1 byte-exactness) deferred per reviewer's own note.

## R0 verification matrix (all PASS)

- 11-bullet outline matches ENCODE_FLAGS.
- All 11 per-flag anchors `#md-encode-<flag>` present.
- `--context` outline + 2 variants (`tap`, `segwitv0`) byte-correct.
- `--network` outline + 4 variants byte-correct.
- Positional `[TEMPLATE]` correctly documented as Optional clap-level + Required-by-runtime.
- Conditional visibility prose matches `form/conditional::md_encode` exactly (positional → Disabled/Hidden trio; --from-policy → --context Required; neither → both Required marker; --context segwitv0 → --unspendable-key Disabled value-inspect).
- Canonical `xpub6CatWdiZi...VMrjPC7PW6V` + fingerprint `73c5da0a` cross-verified against `crates/mnemonic-toolkit/src/slip0132.rs:172` BIP84_REF_XPUB and `docs/manual/transcripts/24-recover-mk1.out:2`.

## Nits folded inline (3 of 4)

### N-2 (folded) — refusal-table classification corrections

The "both positional AND --from-policy set" row was mis-classified as runtime pre-check; per `md-cli/src/main.rs:66` `conflicts_with = "template"` it's a clap-level error. Fixed; cited the exact line in the table.

### N-3 (folded) — missing third refusal mode for `--unspendable-key`

`md-cli/src/main.rs:284-287` rejects `--unspendable-key` when supplied without `--from-policy` (i.e. with the positional template path) with `--unspendable-key is only meaningful with --from-policy`. Added a row to the refusals table noting this is not GUI-reachable due to conditional-visibility Hide.

### N-4 (folded) — `--force-long-code` is a no-op since md-cli v0.12.0

The flag was dropped upstream and is now accepted-for-forward-compat-only with no effect (per `crates/md-cli/src/cmd/encode.rs:90-94`, status: wont-fix at md-cli v0.15.2). Updated the chapter section to lead with the no-op designation; preserved the historical-intent prose.

## Nit deferred (1)

### N-1 (NOT folded) — refusal-text byte-exactness paraphrase

Several refusal strings in the Refusals table paraphrase md-cli error strings rather than quoting them byte-exactly. Per the reviewer's own note, sibling chapter `58-compile.md:73` uses the same paraphrase pattern and the project pattern is consistent across prior batches (ms-encode, etc.). Not folded; if a future "verify-examples" cell asserts byte-equality, this can be revisited as a sweep across all chapters.

## Markdown table pipe-escape

The new refusal row used `tap|segwitv0` literal which broke markdownlint's table-column-count. Fixed by escaping `|` as `\|` (same fix as batch 5d).

## Lint state (post-fold)

- Phase 4 schema-coverage RED at **86 missing** (was 104 → -18 = 1 sub + 11 flags + 6 variants).
- Phase 5 outline-coverage RED at **11 missing** (was 14 → -3 = 1 subcommand-outline + 2 flag-outlines: --context + --network).
- Phases 1-3 GREEN.
- HTML 33 H1 chapters (was 32 → +1).
- PDF 151 pages (was 143 → +8).

R1 not dispatched — R0 was LOCK criterion (0C/0I); fold-set was nit-only with no semantic re-verification surface beyond what R0 already covered.
