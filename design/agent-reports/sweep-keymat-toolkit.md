# Security sweep — secret key-material memory hygiene (mnemonic-toolkit)

- **Audited against:** `origin/master = ddabf5e3` ("design(cycle-14): persist whole-diff
  review GREEN (0C/0I) — v0.67.0"). `git fetch -q origin` run first; the worktree at
  `/scratch/code/shibboleth/wt-tk-master` is at this exact SHA. All `file:line` citations
  re-verified against this SHA (NOT the stale v0.60.0 working-tree branch).
- **Date:** 2026-06-21.
- **Scope:** SECRET key material only (BIP-39 phrase/entropy, xprv/SecretKey, BIP-38/PBKDF2
  passphrases, SLIP-39/seed-XOR shares, raw entropy, ECIES scalars, BIP-85 derived secrets,
  SeedQR/WIF). Public material (xpub/address/descriptor/fingerprint/digest) excluded.
- **Deliverable:** candidate FOLLOWUP slugs for the orchestrator to dedup + file. RECON ONLY —
  no source edited.

---

## Method

Read the existing hygiene infrastructure first (`secret_string.rs`, `secrets.rs`,
`secret_taxonomy.rs`, `secret_advisory.rs`, `process_hardening.rs`,
`tests/lint_zeroize_discipline.rs`) to establish the clean baseline + the allowlist, then swept
every secret-bearing `src/*.rs` for the 7 hygiene-gap classes. Confirmed-clean (no finding):
`electrum_crypto.rs` (keys/scalars/plaintext all `Zeroizing`; `encrypt_field` bare buffers are
TEST-ONLY — no production caller), `slip39/feistel.rs` (L/R halves + round_key `Zeroizing`,
`password`/`salt` locals explicitly `.zeroize()`d), `verify_message.rs` (verify-only, public),
`slot_ms1.rs` (`Zeroizing<Vec<u8>>` field; correct move-out-of-`Payload` idiom),
`parse_descriptor.rs` (seed/entropy `Zeroizing`; `.to_string()` only on public descriptor keys),
`process_hardening.rs` (`set_non_dumpable()` wired at `main.rs:152` — core-dump leak defended),
`secret_advisory.rs` (warnings never embed secret values), `nostr.rs`/`silent_payment.rs`
libraries (cmd boundary wraps derived priv in `SecretString`).

**Cross-cutting pattern found:** cycle-14 (v0.67.0) closed the *clap-arg / handler-local /
persistent-field* leg of the secret-`String` story (L22). It did NOT touch the **library-layer
and derived-output `String`** leg: several `pub`/`pub(crate)` functions that *produce* secret
text build the value in bare `String`/`Vec` intermediates and/or return a bare `String`. That is
the recurring residue class across findings 1–4 below. The already-filed
`stdin-reader-transient-buf-zeroizing` is the narrow `read_stdin_*` instance of this same class;
these findings are the broader, previously-uncounted siblings.

---

## Candidate findings

### 1. `bip85-derive-child-output-secretstring`  — **HIGH/MED (genuinely NEW)**

- **Secret type:** BIP-85 derived child secrets — BIP-39 phrase / HD-seed WIF / child xprv /
  raw hex entropy / derived password (base64/base85) / dice (entropy-derived).
- **Gap class:** 1 (zeroize-on-drop gap) + 4 (`.to_string()`/`to_wif()` escapes into bare copy).
- **Where:**
  - `cmd/derive_child.rs:224` `let output = match … {…}` → bare `String`; `:304`
    `writeln!(stdout, "{output}")` then dropped un-scrubbed.
  - `bip85.rs:72/103/131/163/181/196/222` — all 7 `format_*` fns return a bare `String`
    (`mnemonic.to_string()` / `pk.to_wif()` / `xprv.to_string()` / `hex::encode(…)` /
    encoded password / `out.join(",")` dice).
- **WHAT:** The 64-byte BIP-85 entropy buffer IS `Zeroizing` + mlock-pinned, but the
  *rendered secret output* — a full child seed phrase / WIF / xprv / password — is a bare,
  un-scrubbed, un-pinned heap `String` that lingers until function exit. The handler emits
  `OutputClass::PrivateKeyMaterial` (it KNOWS this is spendable), yet does not wrap it.
- **Severity:** HIGH within the defense-in-depth tier — this is full spending authority
  (`hd-seed`/`xprv`) rendered to a bare heap String; the single highest-value un-wrapped
  secret-output site in the crate. (Not a leak-to-disk/argv/log, so not Critical.)
- **NEW?** Genuinely new. Untracked. `bip85.rs` FOLLOWUPS cover only the third-party-blocked
  carriers (SecretKey/Xpriv/Mnemonic/Shake256), never the rendered output.
- **WHY:** A direct, in-repo precedent already exists — `cmd/silent_payment.rs:286-287` and
  `cmd/nostr.rs:235` wrap their derived-priv-key output in `SecretString`. `derive-child` is the
  one derived-private-key emitter that was never given the same treatment. Clean consistency fix:
  flip the 7 `format_*` returns + `output` to `SecretString` (serialize-transparent; `writeln!`
  via `Display` unchanged) and add lint rows.

### 2. `electrum-native-seed-normalize-intermediates-zeroizing` — **MED (genuinely NEW)**

- **Secret type:** BIP-39/Electrum seed phrase + BIP-39 passphrase (normalized copies); Electrum
  native-seed entropy.
- **Gap class:** 6 (key-derivation intermediates not scrubbed) + 1 + 4.
- **Where (`electrum.rs`):**
  - `normalize_text_electrum` (`:79-88`) builds 4 bare secret `String`s (`nfkd`/`lowered`/
    `stripped`/`collapsed`); returns bare `String`.
  - `electrum_seed_to_bip32_seed` (`:97-107`): `norm_phrase` (`:98`) + `norm_pp` (`:99`) are bare
    normalized copies of the phrase AND passphrase, alive across the PBKDF2 call (the 64-byte
    output IS `Zeroizing`, but its two secret inputs are not).
  - `phrase_to_entropy` (`:147-176`): `words: Vec<String>` (normalized secret words) bare;
    `:175` `Ok((*acc).clone())` copies the secret entropy OUT of `acc: Zeroizing<Vec<u8>>` into a
    bare returned `Vec<u8>`.
  - `entropy_to_phrase` (`:215`) `let phrase = words.join(" ")` bare secret phrase, returned bare;
    `normalize_phrase_for_hmac` (`:243`) + `strip_cjk_internal_whitespace` (`:249`) build bare
    `stage1`/`collapsed`/`out` copies.
- **WHAT:** The master-secret-equivalent phrase + passphrase materialize in ~6 un-scrubbed heap
  `String`s during Electrum seed derivation; the secret entropy is `clone()`d out of its
  `Zeroizing` wrapper on return. `acc` was wrapped; the text intermediates and the returned clone
  were not.
- **Severity:** MED — defense-in-depth (the final 64-byte seed is `Zeroizing`), but these are full
  copies of the highest-value secret (the phrase) sitting un-scrubbed during the derivation window.
- **NEW?** Genuinely new. The existing `electrum-native-seed-address-derivation` FOLLOWUP is about
  *derivation correctness*, not memory hygiene; no slug covers these intermediates.
- **WHY:** First-class hygiene bar — a normalized phrase copy is exactly the secret an attacker
  wants in swap/core. Wrap the normalize helpers' returns + the two `norm_*` locals + the
  `phrase_to_entropy` return in `Zeroizing<String>`/`Zeroizing<Vec<u8>>`.

### 3. `seedqr-codec-internal-secret-string-zeroizing` — **MED/LOW (genuinely NEW)**

- **Secret type:** BIP-39 phrase + SeedQR digit string + raw entropy hex (all forms of the seed).
- **Gap class:** 1 (library-internal bare secret) + 6.
- **Where (`seedqr.rs`):** `decode` (`:96`) — `stripped` (raw digits), `:131` `phrase =
  words.join(" ")`, returns bare; `encode` (`:141`) — `:161` `digits: String`, `normalized`;
  `encode_compact` (`:182`) — `:196` `Ok(hex::encode(m.to_entropy()))` (raw entropy hex bare);
  `decode_compact` (`:205`) — `stripped`, `:218` `Ok(m.to_string())`.
- **WHAT:** The SeedQR library functions build secret phrase/digits/entropy in bare `String`s and
  return bare `String`. The `cmd/seedqr.rs` handler DOES re-wrap each return in `Zeroizing`
  immediately (`:177-198`, `:244-264`), so the lingering window is small (return-move + the
  internal scratch buffers), but the internal intermediates (`stripped`/`phrase`/`digits`/
  `normalized`) are never scrubbed.
- **Severity:** MED/LOW — the cmd boundary wraps the final value fast; the residue is the
  internal scratch + the transient return. Sibling of finding 2 (same library-layer class),
  one notch lower because the consumer re-wraps.
- **NEW?** Genuinely new. Untracked.
- **WHY:** Same first-class bar; cheap to wrap the four return values + the scratch buffers.

### 4. `inspect-ms1-payload-husk-not-zeroizing` — **LOW (NEW site, KNOWN class)**

- **Secret type:** ms1 master-seed entropy held in `ms_codec::Payload`.
- **Gap class:** 1 (un-scrubbed owned secret husk).
- **Where:** `cmd/inspect.rs:160-162` `InspectPayload::Ms1 { payload: ms_codec::Payload }`,
  built at `:171-172` via `ms_codec::decode(chunks[0])`, lives for the whole inspect-handler
  scope and drops un-scrubbed. (`entropy_hex` print is correctly `--reveal-secret`-gated at
  `:206`, so this is NOT an output leak — only the in-memory husk.)
- **WHAT:** Identical defect class to the already-filed `self-check-ms1-decode-not-zeroizing`
  (`bundle.rs:2529`): `ms_codec::Payload` is `#[non_exhaustive]` + not zeroize-wrapped, so the
  caller must MOVE the bytes into `Zeroizing`. `inspect` does not. `inspect` was NOT among the
  "other 4 sites that correctly move bytes" enumerated in that slug, nor flagged as the 1 miss —
  it is a previously-uncounted sibling occurrence.
- **Severity:** LOW (defense-in-depth; matches the existing slug's tier).
- **NEW?** New SITE; same class as the filed `self-check-ms1-decode-not-zeroizing`. File as a
  related variant OR fold both ms1-`Payload`-husk sites into one slug — recommend folding (one
  shared fix idiom: destructure `Payload`→`Zeroizing` at decode).

### 5. `bsms-derive-hmac-key-not-zeroizing` — **LOW (NEW; deliberate-by-doc, flag for re-decision)**

- **Secret type:** BIP-129 `HMAC_KEY` (= `SHA256(ENCRYPTION_KEY)`, derived from secret material).
- **Gap class:** 6 (derivation intermediate) + 1.
- **Where:** `bsms_crypto.rs:114-120` `derive_hmac_key` returns a bare `[u8; 32]` (NOT
  `Zeroizing`); `compute_mac` (`:136-145`)'s `out` also bare.
- **WHAT:** The HMAC key is a stack `[u8;32]` derived from the (`Zeroizing`) ENCRYPTION_KEY,
  returned un-wrapped. The doc-comment (`:108-113`) explicitly justifies this: "short-lived stack
  value … an attacker who can read process stack already has access to ENCRYPTION_KEY."
- **Severity:** LOW — a documented, defensible decision; included only because the guiding
  principle says local-memory defense-in-depth is in-scope. Under the first-class bar, an HMAC key
  derived from secret material is itself secret-class and the wrap is ~1 line.
- **NEW?** New, untracked. The rationale is sound; recommend the orchestrator treat this as a
  *flag-for-decision*, not an auto-file. (`bsms_crypto.rs` is whole-file-allowlisted as
  CRYPTO-INTERNAL in the zeroize lint, so this would not trip the gate regardless.)

### 6. `bundle-unified-whole-file-allowlist-precision` — **LOW (gate-precision; matches prompt M1)**

- **Secret type:** n/a (gate-coverage risk, not a live secret).
- **Gap class:** meta (lint precision).
- **Where:** `tests/lint_zeroize_discipline.rs:450` whole-file-allowlists `src/bundle_unified.rs`
  in `NON_ROW_SECRET_FILES`; the file's only `SecretString::new` is `#[cfg(test)]`
  (`bundle_unified.rs:121-128`) — no production secret allocation today.
- **WHAT:** The zeroize-completeness scan checks per-FILE (declared-or-allowlisted), not
  per-allocation. A future PR adding a PRODUCTION secret allocation to `bundle_unified.rs` would be
  silently masked by the whole-file allowlist. Current code is clean (classify logic operates on
  `is_secret_bearing()` booleans, not values), so this is a LATENT gate-precision risk, not a leak.
- **Severity:** LOW.
- **NEW?** New. I AGREE it is worth tracking (matches the prompt's M1 whole-diff finding). Fix:
  narrow the allowlist to a test-only assertion, or split the `s()` test helper out so the file
  carries no `SecretString::new` and can drop off the allowlist entirely.

---

## Confirmed-still-open but ALREADY FILED (do NOT re-file — verified vs `ddabf5e3`)

- **`self-check-ms1-decode-not-zeroizing`** (LOW) — STILL reproduces at `bundle.rs:2529-2533`
  (`ms_codec::decode(ms)` → `payload.as_bytes()` compare → husk dropped un-scrubbed). Fold target
  for finding 4.
- **`addresses-restore-passphrase-not-zeroizing`** — now **RESOLVED by cycle-14/L22**:
  `addresses.rs:151` + `restore.rs:401/832/1296/3050` all wrap passphrase + from_value in
  `Zeroizing<String>`. The FOLLOWUP is stale-open; it should be flipped to resolved, not re-filed.
- **`phrase-overlay-secretstring`** (filed) — `import_wallet.rs:~1229` `phrase_overlays:
  Vec<(u8,String)>` bare copy; noted, not duplicated.
- **`stdin-reader-transient-buf-zeroizing`** (filed) — the narrow `read_stdin_*` `buf` instance of
  the library-layer class that findings 1–4 generalize; noted, not duplicated.

---

## Severity ranking of candidates

| Slug | Sev | New? |
|---|---|---|
| `bip85-derive-child-output-secretstring` | **HIGH/MED** | NEW |
| `electrum-native-seed-normalize-intermediates-zeroizing` | MED | NEW |
| `seedqr-codec-internal-secret-string-zeroizing` | MED/LOW | NEW |
| `inspect-ms1-payload-husk-not-zeroizing` | LOW | NEW site / known class (fold w/ self-check) |
| `bsms-derive-hmac-key-not-zeroizing` | LOW | NEW (deliberate-by-doc; flag, don't auto-file) |
| `bundle-unified-whole-file-allowlist-precision` | LOW | NEW (gate precision; = prompt M1) |

No Critical (no leak-to-disk/log/argv). All are local-memory defense-in-depth.
