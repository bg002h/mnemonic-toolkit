# End-of-cycle R0 review — output-type-stderr-advisory Phase 1 (mnemonic + ms)
**Date:** 2026-05-31 · **Reviewer:** opus architect · **Verdict: GREEN (0C/0I/1m).**

Reviewed the full cross-repo diff (toolkit 80 files, ms-cli 12 files) against SPEC §3.

## Verified correct
- Lattice `Template<WatchOnly<PrivateKeyMaterial>` + derive(Ord).max() = most-sensitive; `worst_class_on_stdout([])`→None; byte-identical lines (toolkit literal — vs ms-cli `\u{2014}`, same bytes; byte-parity test pins it).
- Per-command classes match SPEC §3: bundle (P-if-any_secret_bearing else W), convert (per-target: secret→P, xpub/mk1/address→W, path/fingerprint→inert/None), repair/inspect (collect card_kind_class at ALL stdout-write branches; Unrecoverable returns early w/o push), import-wallet (P iff entropy.is_some()), nostr (npub→W/nsec→P per-branch).
- Auto-repair re-route: Ms1 guard removed; emit_repair_report emits card_kind_class(outcome.kind) for all kinds; repaired card IS on stdout; reaches verify-bundle×6/xpub-search/inspect/convert short-circuits.
- File-output suppression: seedqr/electrum --json-out (exclusive) suppress; export-wallet emits W only on --output=='-'; slip39/final-word/seed-xor --json-out (side-effect) still emit.
- ms-cli: encode/decode P after both stdout branches; derive W unconditional (coexists with language note); repair P.
- Legacy removal: zero `secret material on stdout` in either src/.
- **Security boundary INTACT: every emit → stderr, never stdout; NO stdout artifact (md1/mk1/xpub/seed) altered; transcripts advisory-only (no card/xpub drift).**
- TTY-gate drop on 5 commands unconditional; orphaned IsTerminal removed; bespoke addenda kept.

## Minor (FOLDED — one-line comment added)
import_wallet.rs:391 — BSMS Round-1-only verify branch (`--bsms-round1` no `--blob`) returns before the advisory, emits no line. Defensible INERT (pass/fail verdict + public signer pubkey, mirrors verify-bundle, principles b+c; no spend-capable under-warning). Added a documenting comment.

**END-OF-CYCLE R0 GREEN (0C/0I).** Ready for tag (toolkit v0.38.2) + crates.io publish (ms-cli v0.5.1).
