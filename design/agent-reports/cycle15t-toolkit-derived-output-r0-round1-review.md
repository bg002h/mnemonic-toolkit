# R0 REVIEW — cycle-15 Lane T brainstorm spec (toolkit derived-output zeroize) — Round 1

Verified live against `origin/master = 79100a66` (v0.67.0). Every Appendix `file:line` re-grepped accurate.

## VERDICT: NOT GREEN — 0 Critical / 2 Important / 4 Minor

Core security thesis sound; SecretString-vs-Zeroizing rule correctly applied; priority axes (slug-1 bip85 completeness, axis-5 ms-codec coordination) clean. The two Important are compile-break / gate-break omissions in the ripple analysis, not funds-safety errors.

## CRITICAL (0)
None. No leak-to-disk/argv/log introduced. The funds-relevant fix (electrum `:175 (*acc).clone()` → `Ok(acc)`) is correct; the production consumer `convert.rs:1751 hex::encode(&entropy)` Deref-coerces cleanly.

## IMPORTANT (2)

### I-1 — `entropy_to_phrase` return-widen breaks `compute_outputs` type-unification
`electrum.rs:215` (§2 slug-2 "widen return; update caller") vs `cmd/convert.rs:1423`. The sole production caller is in `compute_outputs`' `let v = match t {...}` (`:1300`/`:1464`) whose sibling arms produce bare `String` (`wif.encrypt_wif` `:1410`; `render_address_from_xpub` `:1466`) and push into `out: Vec<(NodeType, String)>`. Widening `entropy_to_phrase` → `Zeroizing<String>` breaks the match unification → convert.rs won't compile, OR the implementer "fixes" with `(*phrase).clone()` — re-introducing the clone-out-of-Zeroizing anti-pattern slug-2 kills.
**Fix:** adopt the slug-3 philosophy — keep `entropy_to_phrase: -> String` (`pub(crate)`), wrap only the internal `let phrase = words.join(" ")` scratch (`words: Vec<&str>` are static-wordlist refs, NOT secret → no wrap). Correct the §2 `:215` row from "widen return" to "wrap internal scratch only; return stays String." (The `phrase_to_entropy` ripple IS fine — its test `electrum.rs:454 assert_eq!` on two `Zeroizing<Vec<u8>>` compiles since zeroize 1.8.2 `Zeroizing` derives `PartialEq`.)

### I-2 — `ZEROIZE_ROWS` count guard upper bound (60) will be breached
`tests/lint_zeroize_discipline.rs:382` `(18..=60).contains(&n)`. §8 claims "bounded 18..=60 so adding rows is safe" but never checked the LIVE count = **54** (`grep -c 'ZeroizeRow {'`). Headroom 6. §8 enumerates ~6-7 new rows (bip85 ≥1, derive_child ≥1, electrum ≥3-4, new `src/seedqr.rs` ≥1) → 54+7=61 > 60 → count-guard goes RED, breaking the full suite.
**Fix:** the SHIP cycle must EITHER (a) consolidate to ≤6 new rows (stay ≤60), OR (b) widen `18..=60` → `18..=66` + update the rationale comment. Pick one explicitly; "adding rows is safe" as written is false.

## MINOR (4) — fold for plan-doc quality
- **M-1 (existing-test deref ripple).** Flipping `format_*` → `SecretString` breaks `assert_eq!(pwd, "dKLoep…")` (bip85.rs:384), `assert_eq!(rolls, "1,0,0,…")` (:399) — `SecretString` has no `PartialEq<&str>`. Fix mechanical (`pwd.as_str()` / `&*pwd`), caught by the full suite, but the plan-doc MUST enumerate the existing-test deref updates so TDD no-behavior-change stays GREEN. (`.split(',')`/`.parse()` keep working via Deref.)
- **M-2 (slug-3 justification factually wrong).** §2/§3 claims "consumer `cmd/seedqr.rs` already wraps each return in Zeroizing immediately" — true only for `cmd/seedqr.rs:194/264`. The other 8 callers of `seedqr::decode` (`convert.rs:1259`, `restore.rs:439/867/3417`, `addresses.rs:208`, `bundle.rs:552`, `verify_bundle.rs:1535`) bind to bare `String`. The keep-public-String DECISION is still right (those locals are a separate pre-existing class, out of scope; widening the public return is the API break the spec rightly avoids) — but correct the justification to "kept stable to avoid the API break; the 8 bare consumer-locals are a separate pre-existing residue class, explicitly out of scope."
- **M-3 (slug-3 scratch enumeration incomplete).** §2 omits sweep-flagged scratch: `encode` `words: Vec<String>` + `digits: String` (:155/:163), `encode_compact` `words: Vec<String>` (:185), `decode_compact` `bytes: Vec<u8>` (hex-decoded raw entropy, :214). Plan-doc must enumerate exhaustively.
- **M-4 (electrum normalize scoping).** §2 rows for `electrum.rs:147`/`:243` reference `normalize_electrum`, imported from `crate::wordlists` (defined `wordlists/mod.rs:132`) with OTHER callers (`:88,122`). Widening its return is cross-module scope-creep. Wrap at the `electrum.rs` consumption boundary (`.map(|w| Zeroizing::new(normalize_electrum(&w)))`), leave `wordlists::normalize_electrum: -> String` untouched. (Cosmetic: secret_string test citations off-by-2 — `debug_redacts_the_secret:117`, `eq_failure_debug_does_not_leak:157`.)

## Affirmed (priority axes + SemVer)
- **Axis 1 (bip85):** all 7 `format_*` confirmed; `derive_child.rs:224 output` single emitter, used twice (`let` + `writeln!("{output}")`:304), no un-wrapped escape, NO `--json` in derive_child. SecretString Display/Deref byte-identical; redacting Debug guards the cycle-14 class. Precedents (silent_payment.rs:286-287, nostr.rs:235) genuinely use `SecretString::new`. Clean modulo M-1.
- **Axis 5 (ms-codec pin):** pin `ms-codec = "0.5"` (Cargo.toml:29; "0.39" is md-codec :36). Toolkit references NO `ms_codec::inspect()`/`InspectReport` (full-crate grep) — only `Payload`/`decode`/`encode`/`Tag`/`Error`/`Threshold`/`combine_shares`/etc. All `Payload::Mnem` matches use `..` rest patterns → 0.6.0 field ADDITIONS tolerated; only variant/field renames or `Entr` tuple changes break (caught by ship `cargo build`+suite). Widen `"0.5"`→`"0.6"` at SHIP is correctly a coordination item, recompile-only, no toolkit source change.
- **Axis 4 SemVer: MINOR 0.68.0.** bip85/electrum/derive_child are bin-private (`main.rs:4,14`; absent from `lib.rs` incl. `cfg(fuzzing)`); seedqr `pub mod` but returns stay `String` → nothing public changes shape (honest floor PATCH 0.67.1). MINOR is the correct call per the v0.10.1 precedent + the co-shipped `"0.5"`→`"0.6"` pin cross. Release version-sites: both READMEs + fuzz/Cargo.lock + root + install.sh self-pin; re-run full suite + fuzz-lock sync before tag.
- **Axis 3 slug-3:** ratified keep-public-String.
- **Axis 6:** behavior/wire/`--json` byte-identical (Display/Deref/Serialize transparent; no JSON in derive_child). No schema_mirror/manual. `src/seedqr.rs` is a NEW source_file lint row (gains `Zeroizing::new(`). `SECRET_FILE_FLOOR=37` is a `>=` floor, unaffected. NEVER cargo fmt the toolkit (honored). The only lint gate-break is I-2.

## Path to GREEN
Fold I-1 (keep `entropy_to_phrase -> String`, wrap scratch only) + I-2 (pick consolidate-≤60 or widen the count-guard, state it) + M-1..M-4; persist; re-dispatch R0.
