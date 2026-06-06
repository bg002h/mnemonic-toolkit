# SPEC — `mnemonic addresses --from electrum-phrase`: Electrum native-seed address derivation

**Status:** R0 gate (pre-implementation). MUST converge to 0 Critical / 0 Important before any code.
**Resolves:** FOLLOWUP `electrum-native-seed-address-derivation`.
**Source SHA:** branch `addresses-electrum-native-derivation` off master `591f334` (toolkit v0.46.3). Electrum ref commit `e1099925e30d91dd033815b512f00582a8795d25`.
**SemVer:** MINOR — new derivation capability on `addresses` (un-refuses an existing `--from` node). v0.46.3 → **v0.47.0**.

---

## 1. Summary

`mnemonic addresses --from electrum-phrase=<seed>` is refused today (`addresses.rs:224-228`, `"--from {other} is not supported … (use xpub/phrase/entropy/seedqr)"`). This cycle implements **Electrum's own** seed→address derivation (distinct from BIP-39/BIP-44) so an Electrum native seed yields its Electrum-correct addresses, validated against Electrum's own end-to-end vectors. Watch-only out (no xprv on stdout), reusing the existing `addresses` chain/index loop + `render_address_from_xpub`.

**No new clap flag/value-enum:** `--from` is a free `String`; `electrum-phrase` is already a `NodeType` that parses — this cycle only un-refuses it. (R0: confirm the toolkit `gui-schema` does NOT model `addresses --from` node-prefixes as a dropdown value-enum; if it does, a paired GUI `schema_mirror` update is owed — otherwise lockstep is **manual-mirror only**.)

## 2. The crypto — pinned against Electrum primary source (`e1099925`)

> **Phase-2 correction (crypto):** the refusals below are **exit 1** (`bad()` → `BadInput`), NOT exit 2 (matching the existing `addresses` `bad()` refusals). AND `normalize_text` is NOT reusable from `normalize_phrase_for_hmac` after all — a dedicated `normalize_text_electrum` was required because (a) Electrum's order is NFKD→lower→strip (the in-tree `normalize_electrum` strips-then-lowers) and (b) Electrum's `unicodedata.combining(c) != 0` is the **canonical combining class**, not the Mark general category: marks with ccc=0 (U+034F CGJ, U+0489) are KEPT by Python but stripped by `is_combining_mark`. The UNICODE_HORROR vector (§5 vector 3) caught both — fixed via `canonical_combining_class(c) != 0`. This is exactly the R0 M1 "vector 3 is the must-pass discharge" prediction.

**Seed stretch** (`mnemonic.py::mnemonic_to_seed`): `seed64 = PBKDF2-HMAC-SHA512(normalize_text(phrase), b"electrum" + normalize_text(passphrase), 2048)` → 64 bytes. **`normalize_text` is implemented as a dedicated `electrum.rs::normalize_text_electrum`** (NOT `normalize_phrase_for_hmac` — see the Phase-2 correction above) (NFKD → strip combining/accents → lower → collapse whitespace → strip CJK-internal whitespace) — equivalent to Electrum's `normalize_text` (`mnemonic.py:80-91`) **for all shipped-wordlist scripts** (R0 M2: the in-tree order is NFKD→strip→lower vs Electrum's NFKD→lower→strip — equivalent since combining marks are caseless; and `is_cjk` covers the 9 intervals of Electrum's 28 `CJK_INTERVALS` that correspond to shipped wordlists — adequate, not literally all 28). The SAME normalization applies to BOTH the seed and the passphrase. **(R0 M1) The NFKD/lower order + the CJK-strip are exercised ONLY by §5 vector 3 (`UNICODE_HORROR`)** — that cell is the load-bearing normalization discharge (vectors 1/2 are pure ASCII). `normalize_phrase_for_hmac` is private in `electrum.rs`; the new `electrum_seed_to_bip32_seed` lives in `electrum.rs` too → calls it in-module, **no `pub(crate)` widening needed**.

**BIP-32 root + per-version path** (from `keystore.from_seed` + the test vectors in §5):
- `master = bitcoin::bip32::Xpriv::new_master(network, &seed64)`.
- **Standard** (`SeedVersion::Standard`, prefix `01`): account node = `master` (derivation `m`); script type **P2PKH**; receive at `m/0/i`, change at `m/1/i`.
- **Segwit** (`SeedVersion::Segwit`, prefix `100`): account node = `master.derive_priv(m/0')` (single hardened step); script type **P2WPKH**; receive at `m/0'/0/i`, change at `m/0'/1/i`.
- **2FA** (`Standard2FA` `101` / `Segwit2FA` `102`): **REFUSE** (exit 2) — 2FA seeds need a second factor; already the posture elsewhere. Out of scope.

The existing `addresses` loop (`addresses.rs:234-251`) already derives `account_xpub / chain / index` and renders with the script type — so setting `account_xpub` = (master xpub for standard / `m/0'` xpub for segwit) + the version's script type makes the loop produce the correct addresses with NO loop change. `--chain receive`→0, `change`→1 map exactly (vectors confirm change at `…/1/0`).

## 3. Surface design — `addresses.rs` `match from.node`

Add a `NodeType::ElectrumPhrase` arm (replacing the fall-through refusal for this node):
- `--account != 0` → **refuse** (exit 2): `"--account does not apply to --from electrum-phrase= (Electrum native derivation has no BIP-44 account level)"` (mirror the `xpub` arm).
- `validate_seed_version(&from_value)?` → `Standard | Segwit | {Standard2FA|Segwit2FA}`. 2FA → refuse (exit 2): `"--from electrum-phrase= 2FA seeds (version 101/102) are not supported (require a second factor)"`.
- **`--address-type` consistency:** it is REQUIRED by clap (`:35`, no default). For electrum the script type is FIXED by the seed version, so REFUSE a mismatch (exit 2): standard requires `--address-type p2pkh`, segwit requires `--address-type p2wpkh`; on disagreement: `"Electrum {standard|segwit} seeds derive {p2pkh|p2wpkh} addresses; --address-type {X} conflicts (Electrum's script type is fixed by the seed version)"`. (Decision: refuse-on-mismatch rather than silent-override — no footgun; R0 to confirm vs. the alternative of ignoring `--address-type`.)
- Compute `seed64` via a new `pub(crate) fn electrum_seed_to_bip32_seed(phrase, passphrase) -> Zeroizing<[u8;64]>` in `electrum.rs` (PBKDF2 per §2; `Zeroizing` per the secret-hygiene discipline — the 64-byte seed is master-secret-equivalent). Then `master = Xpriv::new_master`; for segwit derive `m/0'`; `account_xpub = Xpub::from_priv(&secp, &node)`.
- Result tuple: `(account_xpub, network, account_field=None)`; the script type fed to render is the version's type (= the validated `--address-type`). Network: `args.network.unwrap_or(Mainnet)` (Electrum seeds are mainnet-oriented; testnet via `--network`).
- **Watch-only out:** only `account_xpub` (public) leaves the arm; `master`/`node` xprivs are `Zeroizing`-scrubbed. Negative test greps stdout/stderr/json for `xprv`/`zprv`/`tprv`.
- **Passphrase:** `--passphrase`/`--passphrase-stdin` ALREADY resolved into `passphrase` upstream; for electrum it feeds the PBKDF2 salt (valid — unlike xpub which refuses it). The argv-secret advisory for `--from electrum-phrase=` (secret on argv) already fires via the existing `addresses` `--from` advisory (`:126-133`, gates on `is_argv_secret_bearing()` which includes `ElectrumPhrase`) — confirmed by R0.
- **(R0 M3) `--language` is inert for `electrum-phrase`** — the seed is derived from the raw normalized phrase string via PBKDF2 (no wordlist decode), so `--language` cannot affect the derivation (unlike phrase/seedqr which decode via a BIP-39 wordlist). Document this in §6 (accepted-but-ignored); do NOT refuse it (an Electrum non-English seed user may reflexively pass `--language` — silently ignoring is harmless since the addresses are correct regardless).

## 4. Secret-input hygiene / lint
- `electrum-phrase=` is a secret-bearing `--from` value → the existing `addresses` argv-leak advisory + `lint_argv_secret_flags` coverage applies (no NEW secret flag). R0: confirm `lint_argv_secret_flags` already treats `addresses --from` as secret-bearing (it does for phrase/entropy/seedqr); electrum-phrase rides the same `--from`.
- The 64-byte seed + the master/node xprivs are `Zeroizing`. No xpriv reaches any output.

## 5. Tests — validate against Electrum's OWN end-to-end vectors (`test_wallet_vertical.py` @ `e1099925`)
Vendor these 3 vectors (mainnet, passphrase as shown):
1. **Standard / P2PKH** — seed `cycle rocket west magnet parrot shuffle foot correct salt library feed song`, pp `""`: receive[0] `1NNkttn1YvVGdqBW4PR6zvc3Zx3H5owKRf`, change[0] `1KSezYMhAJMWqFbVFB2JshYg69UpmEXR4D`. (master xpub `xpub661MyMwAqRbcFWohJWt7PHsFEJfZAvw9ZxwQoDa4SoMgsDDM1T7WK3u9E4edkC4ugRnZ8E4xDZRpk8Rnts3Nbt97dPwT52CwBdDWroaZf8U`.)
2. **Segwit / P2WPKH** — seed `bitter grass shiver impose acquire brush forget axis eager alone wine silver`, pp `""`: receive[0] `bc1q3g5tmkmlvxryhh843v4dz026avatc0zzr6h3af`, change[0] `bc1qdy94n2q5qcp0kg7v9yzwe6wvfkhnvyzje7nx2p`.
3. **Segwit + passphrase** (normalization torture) — same seed, pp = `UNICODE_HORROR` (`₿ 😀 😈 … horrors lie in the dark heart of unicode?` — the exact literal from `test_wallet_vertical.py:34`): receive[0] `bc1qx94dutas7ysn2my645cyttujrms5d9p57f6aam`, change[0] `bc1qcywwsy87sdp8vz5rfjh3sxdv6rt95kujdqq38g`. Pins the `normalize_text` passphrase path.

Cells (new `tests/cli_addresses_electrum.rs`): per-vector `--from electrum-phrase=<seed> --address-type <p2pkh|p2wpkh> --chain both --count 1 [--passphrase …]` → assert receive[0]+change[0] match; `--address-type` mismatch → exit 2; `--account 1` → exit 2; a 2FA seed → exit 2; `--json` shape; watch-only-out (no xpriv). Full workspace `cargo test --no-fail-fast` + clippy GREEN.

## 6. Lockstep / scope
- **Manual mirror (REQUIRED):** `docs/manual/src/40-cli-reference/` `addresses` chapter — add `electrum-phrase` to the `--from` source list + a note that the script type/derivation is fixed by the Electrum seed version (so `--address-type` must match it, `--account` is refused, and `--language` is ignored — R0 M3). Add a transcript if the chapter carries `addresses` examples. `make -C docs/manual audit` GREEN.
- **GUI `schema_mirror`:** NO clap flag/value-enum change (un-refusing a runtime-parsed `--from` node) → likely NONE. **R0 MUST confirm** the toolkit `gui-schema` does not expose `addresses --from` node-prefixes as a dropdown `choices` set (if it does, file a paired `gui-addresses-electrum-from-node-pending-pin-bump` FOLLOWUP + GUI mirror). No new error variant (reuses `BadInput`).
- **Sibling-codec:** none.

## 7. Phased plan
- **Phase 1 (RED):** the 3 Electrum vectors as cells (RED — currently `--from electrum-phrase` refuses). Verify RED-for-the-right-reason (refusal, not a wrong address). **(R0 M1) Vector 3 (`UNICODE_HORROR` passphrase) is the must-pass normalization discharge** — it is the only cell exercising the NFKD/lower order + CJK-strip; a GREEN-phase mismatch there is Critical (reopens R0), not a Minor.
- **Phase 2 (GREEN):** `electrum_seed_to_bip32_seed` in `electrum.rs` + the `addresses.rs` `ElectrumPhrase` arm + effective-script-type threading + refusals. Workspace test + clippy GREEN. Per-phase opus review → persist.
- **Phase 3 (release):** manual mirror + `make audit`; CHANGELOG `[0.47.0]`; version v0.46.3 → **v0.47.0** (Cargo.toml/lock + 2 READMEs + install.sh self-pin); FOLLOWUP `electrum-native-seed-address-derivation` → resolved. Per-phase review.
- **Phase 4 (ship):** clean tree → ff-merge → tag `mnemonic-toolkit-v0.47.0` → push → watch CI (rust, install/sibling-pin-check, manual if a manual file changed).

## 8. Risk
Moderate (crypto), but de-risked: PBKDF2 + both derivations are pinned against Electrum source AND 3 end-to-end vectors (incl. a unicode-passphrase normalization torture vector). The reuse of the existing chain/index loop minimizes new code. R0 MUST confirm: (i) `Xpriv::new_master` + `derive_priv(m/0')` + `Xpub::from_priv` produce the §5 vector addresses (the implementer runs the RED cells against a build to prove byte-exact — [[feedback_verify_the_actual_artifact_not_an_analogous_emitter]]: test the REAL Electrum-derived address, not an analogous BIP-32 path); (ii) `normalize_phrase_for_hmac` == Electrum `normalize_text` and is reachable; (iii) the `--address-type` refuse-on-mismatch vs the required-flag interaction is clean; (iv) the gui-schema `--from` dropdown question (§6); (v) the segwit `m/0'` is ONE hardened step (depth-1), matching the zpub depth in the vectors.
