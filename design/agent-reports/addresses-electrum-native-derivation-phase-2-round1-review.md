# Phase 2 (GREEN) Code Review — round 1 — `addresses --from electrum-phrase`

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory per-phase review). **Date:** 2026-06-06.
**Branch:** `addresses-electrum-native-derivation`. **Verdict:** **0 Critical / 1 Important** (+ 1 Minor). I1 folded → re-dispatch.

> Persisted verbatim per CLAUDE.md BEFORE the fold. Reviewer had no shell; verified crypto statically against Electrum `e1099925` + confirmed the vectors are Electrum's OWN published values matched by provenance. Operator ran the suite GREEN (recorded). I1 (wire-shape `source:"unknown"`) folded; re-dispatch per the per-fold loop.

---

## VERDICT: 0 Critical / 1 Important (+ 1 Minor)

### Important — I1 (FOLDED)
**`--from electrum-phrase= --json` emitted `"source":"unknown"`** (`addresses.rs:355-363` `source_label` had no `ElectrumPhrase` arm → `_ => "unknown"`). `source` is a documented `--json` envelope field; this mislabeled 100% of electrum-phrase `--json` output and shipped silently (the only `--json` test grepped for xpriv, never asserted `source`).
**FOLD:** added `NodeType::ElectrumPhrase => "electrum-phrase"` to `source_label`; strengthened `electrum_watch_only_no_xpriv` to assert `v["source"] == "electrum-phrase"`. Operator-confirmed: `gui`... `addresses --from electrum-phrase=… --json` now emits `{"source":"electrum-phrase",…}`; all 7 cells GREEN + clippy clean.

### Minor — M1
`source_label`'s `_ => "unknown"` catch-all could silently mislabel a future added node (the I1 class). Left as-is (the other ~10 NodeTypes are refused before `emit_json`, so `_` is genuinely unreachable for them); the 5 supported nodes are now all listed explicitly. Non-gating.

---

## What verified clean

**Crypto gate 1 — the vectors are Electrum's OWN, matched (not baked-in).** Fetched Electrum `tests/test_wallet_vertical.py` @ `e1099925`: `UNICODE_HORROR_HEX` byte-identical; all 3 address pairs (standard `1NNkttn1…`/`1KSezY…`, segwit `bc1q3g5…`/`bc1qdy9…`, segwit+UNICODE_HORROR `bc1qx94…`/`bc1qcyww…`) match Electrum's asserted values exactly. Matching Electrum's published e2e vector IS the proof (wrong normalize → wrong PBKDF2 seed → wrong address).

**Crypto gate 2 — `normalize_text_electrum` Electrum-exact** (`electrum.rs:79-88`): NFKD via `.nfkd()` ⇔ `normalize('NFKD')`; `.to_lowercase()` ⇔ `.lower()`; `filter(canonical_combining_class(c)==0)` ⇔ `[c for c if not combining(c)]` (ccc, NOT Mark category — U+034F/U+0489 ccc=0 kept, the R0-M1 / Phase-2 correction); lower-BEFORE-strip, no re-NFKD after lower; `split_whitespace().join(" ")` ⇔ `' '.join(split())`; `strip_cjk_internal_whitespace` ⇔ Electrum CJK-ws removal (after collapse, only ASCII space remains, so the is_whitespace superset is moot). CJK coverage 9/29 = shipped-wordlist scripts (R0-M2 accepted).

**`normalize_electrum` NOT disturbed** (`wordlists/mod.rs:132-145` unchanged); `normalize_phrase_for_hmac` + `validate_seed_version` untouched. Dual-normalize accepted (the two agree on valid wordlist phrases; the torture input is passphrase-only → routes through `normalize_text_electrum`).

**Derivation arm correct** (`addresses.rs`): standard→account_xpub=master (m); segwit→`master.derive_priv(m/0')` via `ChildNumber::from_hardened_idx(0)` = hardened 0' (depth-1), confirmed not `from_normal_idx`; type p2pkh/p2wpkh per version; `--address-type` mismatch / `--account!=0` / 2FA all refused via `bad()`→BadInput→**exit 1** (matches the Phase-2 correction + the tests). Existing loop derives standard `m/0/i`, segwit `m/0'/0/i`. ✓

**Watch-only / hygiene:** only `Xpub::from_priv(&node)` (public) leaves the arm; `seed64` is `Zeroizing<[u8;64]>`; master/node `Xpriv` locally scoped, never written. `Xpriv` not Zeroize-on-drop = known accepted gap (`rust-bitcoin-xpriv-zeroize-upstream`), no new finding.

**Scope/SemVer/lockstep:** `is_argv_secret_bearing(ElectrumPhrase)`=true → argv advisory fires; `--language` inert (not refused) per R0-M3; manual mirror correctly still Phase-3 TODO; no GUI schema_mirror change (R0 settled).

---

## Operator-run gates (reviewer had no shell)
- `cargo test -p mnemonic-toolkit --test cli_addresses_electrum` → **7/7** (incl. UNICODE_HORROR + strengthened json source assertion).
- `cargo clippy -p mnemonic-toolkit --all-targets` → **exit 0**.
- Full `cargo test -p mnemonic-toolkit --no-fail-fast` → **0 failures** (pre-I1-fold; re-run post-fold below).
- `addresses --from electrum-phrase=… --json` → `{"source":"electrum-phrase",…}` (I1 confirmed fixed).

**I1 folded → re-dispatch round 2 before GREEN.**
