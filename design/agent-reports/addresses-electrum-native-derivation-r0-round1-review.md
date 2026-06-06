# R0 Architect Review (round 1) — `SPEC_addresses_electrum_native_derivation.md`

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory pre-implementation R0 gate). **Date:** 2026-06-06.
**Branch:** `addresses-electrum-native-derivation` (off master `591f334`). **Verdict:** **0 Critical / 0 Important** (+ 3 Minor). **GREEN — conditional on the Phase-1 vector cells passing (esp. vector 3).**

> Persisted verbatim per CLAUDE.md. Reviewer had no shell; verified all crypto inputs against pinned Electrum source + confirmed in-tree APIs, and correctly delegated the byte-exact runtime proof to the Phase-1 RED→GREEN vector cells (higher fidelity than a replica probe). A vector mismatch reopens R0 as Critical. Key lockstep finding: NO GUI schema_mirror change owed (manual-mirror only).

---

## VERDICT: 0 Critical / 0 Important (+ 3 Minor) — GREEN (conditional)

The mandated empirical probe could not run in the review env (Read/Grep/WebFetch only). The reviewer verified every input to the derivation against Electrum `e1099925` + confirmed the cited Rust APIs exist in-tree with exact signatures; the byte-exact proof is delegated to the SPEC's Phase-1 vector cells (which drive the real path). **GREEN is conditional on those cells passing; a mismatch reopens R0 as Critical.**

---

## What verified clean

**Crypto pinned to source (`mnemonic.py`/`keystore.py` @ `e1099925`):** `pbkdf2_hmac('sha512', normalize_text(mnemonic), b'electrum' + normalize_text(passphrase), 2048)` → matches SPEC §2 verbatim. `normalize_text` = NFKD → lower → strip-combining → `' '.join(split())` → strip-CJK-internal. Derivation: standard `der="m/"` (identity) → `m/0/i`, `m/1/i`; segwit `der="m/0'/"` (one hardened step) → `m/0'/0/i`, `m/0'/1/i`. Exactly what the existing `addresses` loop computes (`addresses.rs:236-251`) with zero loop change. `Xpriv::new_master` (key `"Bitcoin seed"`) == Electrum `bip32_root` — byte-exact, not approximate.

**All 3 §5 vectors transcribed correctly** (re-fetched `test_wallet_vertical.py` @ `e1099925`, compared char-by-char incl. xpub/zpub + `UNICODE_HORROR_HEX`). 2FA 101/102 = `Standard2FA`/`Segwit2FA` (`electrum.rs:27-28`) — correct to refuse.

**APIs present in-tree:** `pbkdf2::<Hmac<Sha512>>` (`electrum_crypto.rs:299`), `Xpriv::new_master` (`synthesize.rs:384`), `derive_priv`, `Xpub::from_priv` (`synthesize.rs:336`), `render_address_from_xpub` (`address_render.rs:18`). No hallucinated API.

**Item 2 — normalization reachability:** `normalize_phrase_for_hmac` is private in `electrum.rs`; the new `electrum_seed_to_bip32_seed` also lives in `electrum.rs` → calls it in-module, **no `pub(crate)` widening needed** (SPEC's "or lift it" is moot). `validate_seed_version` already `pub(crate)`.

**Item 3 — surface soundness:** `--address-type` refuse-on-mismatch is clean given clap requires it (`:35`). `--account != 0` refusal mirrors the `xpub` arm (`:168-172`). `NodeType::ElectrumPhrase` parses from `--from` (`convert.rs:88/:50`); `is_secret_bearing()` includes it (`:104`) → argv-leak advisory (`addresses.rs:126-133`) fires automatically. The `other =>` fall-through (`:224-228`) is exactly the arm the new one replaces.

**Item 4 — gui-schema lockstep SETTLED (no GUI change owed):** `addresses --from` is `pub from: String`, `#[arg(long)]`, **no `value_parser`/`value_enum`** → `classify_kind` (`gui_schema.rs:1262`) emits `kind:"text", choices:null` — free text, not a dropdown. `--address-type` uses a custom fn parser (`parse_script_type_arg`), not `PossibleValuesParser`, and reuses existing `p2pkh`/`p2wpkh`. `schema_mirror` gates flag-NAME + value-enum parity only → neither changes → **NO paired GUI update, NO FOLLOWUP**. SPEC §6 correct.

**Item 5 — hygiene:** `addresses --from` already secret-bearing in `lint_argv_secret_flags` axis 2 (`tests/lint_argv_secret_flags.rs:107`) → electrum-phrase rides it, no new secret flag. Arm returns only `account_xpub` (public); master/node xprivs `Zeroizing`-scrubbed; 64-byte seed `Zeroizing<[u8;64]>` correct.

**Item 6 — SemVer MINOR** (v0.46.3 → v0.47.0) correct (un-refuses a node, no break).

---

## Minor (non-blocking)

**M1 — Normalization-order residual; vector 3 is its sole discharge.** In-tree order is NFKD→strip-combining→**lower** (`normalize_electrum`, `wordlists/mod.rs:139-141`); Electrum is NFKD→**lower**→strip-combining. Equivalent for realistic inputs (combining marks caseless); vectors 1&2 are pure ASCII (order-irrelevant). **Vector 3 (`UNICODE_HORROR`) is the ONLY vector exercising this path** — make its Phase-1 cell the top must-pass; a mismatch there is Critical, not Minor.

**M2 — `is_cjk` covers 9 of Electrum's 28 `CJK_INTERVALS`** (`electrum.rs:216-228`). The 9 include every shipped-wordlist script (CJK Unified, Hiragana/Katakana, Hangul Syllables), so adequate in practice; not exercised by any vector. Soften SPEC §2's "byte-for-byte" to "byte-for-byte for all shipped wordlist scripts." Pre-existing.

**M3 — `--language` silently inert for `electrum-phrase`.** The arm derives `seed64` from the raw normalized phrase string (PBKDF2), never a wordlist decode → `--language` has no effect. Add a sentence to §3/§6 noting `--language` is accepted-but-ignored for electrum-phrase (or refuse it) so a user passing `--language spanish` isn't misled. Cosmetic.

---

**GREEN — implementation may proceed**, binding condition: Phase 1 includes all 3 §5 vector cells through the real `mnemonic addresses --from electrum-phrase=…` path, and **vector 3 (`UNICODE_HORROR`) must pass** (sole runtime discharge of M1). Any vector mismatch reopens R0 as Critical.
