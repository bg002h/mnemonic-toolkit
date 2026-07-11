# Post-impl whole-diff review — T1 test-hardening (ms #12/#10/#11) — Fable, adversarial

**Persisted verbatim per CLAUDE.md.** Scope: `mnemonic-secret` @ `master 9a24999` working tree, 3 files / +231 / −0, all TEST code. Shipped as `mnemonic-secret 430008b` (NO-BUMP).

## Oracle independence — VERIFIED
- **#12 (`shares.rs`):** assertion reads `extract_wire_fields(s).share_index_byte` (fixed-offset re-parse `bytes[sep+6]`, `envelope.rs:62`) — structurally independent of `non_s_index_pool`. Distinctness + round-trip likewise oracle-side.
- **#10 (`cli_derive.rs`):** all pins hardcoded literals. Reviewer built a FROM-SCRATCH pure-Python BIP-39/BIP-32 oracle (hand-rolled secp256k1, no libs), self-validated against BIP-32 Test Vector 1 (master id `3442193e…` matches spec) → master fp `73c5da0a` + all 4 account xpubs (bip44/49/84/86) reproduce the pins byte-identically. bip86 = the published BIP-86 spec-vector account-0 xpub for `abandon×11 about`.
- **#11 (`language.rs`):** oracle words are literals independent of the `From<CliLanguage>` mapping. Hex-dumped the embedded wordlists: Korean word[0] = `U+1100 U+1161 U+1100 U+1167 U+11A8`, Spanish `U+0061 U+0301 …` — the test's `\u{…}` escapes match the official NFKD bytes (legit transcription, NOT weakening). Chinese: both share `的` at word[0], first divergence at index 9 (`这` U+8FD9 / `這` U+9019).

## RED-under-mutation — ALL REPRODUCED BY EXECUTION
- #12: drop `!= 's'` filter → n17 + n31 RED (`left: 115`=`'s'`; prints the leaked wire string with `s` in the index slot), n16 green (boundary exact).
- #10: `Bip44 => 45` → bip44 xpub pin RED, other 19 green.
- #11a: Czech↔Portuguese swap → first-word test RED (`abacate`/`abdikace`); the pre-existing `maps_to_bip39_language` stayed GREEN (proves the eval's gap was real + the new test load-bearing).
- #11b: CN-Simplified↔CN-Traditional swap → index-9 test RED (`這`/`这`), first-word test green (proves the degeneracy + that the I-1 index-9 fold is the disambiguator).
All mutations reverted; final tree blob-hashes proven identical to the diff under review.

## Test-only / gates
Insertions-only, no production line changed; `mlock.rs` git-diff empty; the Unicode anomaly was test-transcription not a wire/wordlist bug (the crate's NFKD storage IS the official byte form). `cargo test -p ms-codec` = 166/0; `-p ms-cli` = 230/0 (5 pre-existing ignored). fmt `+1.95.0 --all --check` diffs only in mlock.rs (CI-exempt); clippy clean both gates.

## Findings
Critical 0 / Important 0. Minor (note only): `encode_shares_n31_…` passes `Tag::ENTR` for the `mnem_p()` payload — harmless (tag arg unused in the k≥2 path; payload-kind rides the prefix byte, mirroring suite convention).

**Process disclosure:** RED-proofs needed temporary source mutations; reviewer also accidentally `git checkout` on language.rs mid-review, restored from the verbatim captured diff, and proved final-tree integrity via identical git blob hashes.

**VERDICT: GREEN (0C/0I).** Independent oracles, execution-proven RED under every mutation, zero production change. NO-BUMP stands.

---
**SHIP (opus, 2026-07-10):** dispatcher independently re-reproduced the #12 RED-proof (n17/n31 RED `left:115`, n16 green, reverted). Working tree verified (3 files, insertions-only, filter `!= b's'` intact, mlock.rs empty, new tests green). Committed + pushed `mnemonic-secret 430008b` (NO-BUMP; no tag/publish). **T1 SHIPPED.** Next: T2 (never-wrong-payload harnesses: #6 toolkit / #7 md / #8 mk).
