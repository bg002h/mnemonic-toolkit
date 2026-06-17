# C4 (keyless honest message) impl review — code reviewer (verbatim)

> Reviewer: opus code reviewer (full tools — 19-case regex harness, ran suite + export-wallet).
> Branch `feature/c4-keyless-honest-message`. Verdict GREEN (0C/0I); the one cosmetic Minor folded.

---

**Verdict: GREEN (0C/0I)**

**Item 1 — regex (highest value): PASS.** `[xtyzuvYZUV]pub[A-HJ-NP-Za-km-z1-9]+|\b0[23][0-9a-fA-F]{64}\b` tested against 19 adversarial cases on the real `regex` v1.12.3: matches xpub-family + 66-hex 02/03 compressed pubkeys (w/ or w/o origin); does NOT match 40-hex ripemd160/hash160 nor 64-hex sha256/hash256 (keyless sha256 hashlock correctly keyless). `\b…{64}\b` boundaries tight — a 64-hex run, an over-length 68/72-hex run starting 02, and a 66-window inside a longer hex run all correctly fail (hex digits are word chars). xpub alt can't collide with a hex hash (no literal `pub` in hex).

**Item 2 — both existing assertions green (make-or-break): PASS.** `pipeline::tests` 17/17 (`classify_atn_concrete_mixed_garbage` raw-pubkey → "must carry a key origin"); `cli_bip388_policy_intake bare_key` passes (bare-xpub policy → unchanged). Full crate suite GREEN (1500+).

**Item 3 — message + routing: PASS.** `export-wallet --descriptor "<keyless>" --format descriptor` → exit 0; `--format bitcoin-core` → exit 0. Routing accurate + strictly better than the vacuous origin message.

**Item 4 — scope: PASS.** Arm shared only by `bundle.rs:325` + `verify_bundle.rs:707` (xpub-search uses `expand_bip388_policy`; export-wallet uses `is_at_n_form`). Keyless can never be a valid bundle on either; message accurate, no caller breaks.

**Item 5 — NO-BUMP: PASS.** Both branches return pre-existing `DescriptorParse` (exit 2); only text differs. No version bump, no new ToolkitError variant, no flag/schema_mirror. Tracked diff = pipeline.rs + FOLLOWUPS.md.

**Item 6 — non-vacuous: PASS.** Revert the split → keyless input emits "must carry a key origin" → `classify_keyless_routes_to_export_wallet` RED.

**Item 7 — other:** clippy `--all-targets` clean; rustfmt clean on both changed files (only mlock.rs g6-exempt diff). R0 GREEN persisted. FOLLOWUP filed.

## Minor
- pipeline.rs `classify_descriptor_form` doc-comment "Rule 4: neither → origin-required error" described only one outcome. [FOLDED: rule-4 doc now notes the keyless-route vs origin-required split via `has_any_key_token`.]
