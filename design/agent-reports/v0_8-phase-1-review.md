# v0.8 Phase 1 Review — Input UX bundle

**Scope:** `cmd/convert.rs`, `cmd/derive_child.rs`, `bip85.rs`, `main.rs`, `tests/cli_convert_bip38.rs`, `tests/cli_derive_child.rs`. 6 items: `--bip38-passphrase`, `--passphrase-stdin`, phrase-master, BIP-85 language codes, testnet emission, stdin-master xprv.

**Verdict:** No critical bugs. One important finding on passphrase trimming. All locked semantics are correctly implemented.

---

## Critical

None.

---

## Important

### I1 — `read_stdin_to_string` trims whitespace from `--passphrase-stdin` input (confidence 85)

**File:** `crates/mnemonic-toolkit/src/cmd/convert.rs:525`

```rust
Ok(buf.trim().to_string())
```

`trim()` strips all leading and trailing ASCII whitespace including spaces. For key-material stdin (xprv, wif) trimming a trailing newline is correct. But `--passphrase-stdin` is Phase 1's first use of this helper as a passphrase channel. A passphrase legitimately beginning or ending with a space would be silently truncated, producing a different BIP-38 ciphertext with no error or warning. The V3 spec vector passphrase (`\u{03D2}\u{0301}\u{0000}\u{10400}\u{1F4A9}`) has no leading/trailing whitespace so the unignored V3 tests do not exercise this boundary.

Fix: for passphrase reads, strip only line-ending characters (`trim_end_matches('\n').trim_end_matches('\r')`) while preserving the full `trim()` for key-material reads. The simplest approach is a dedicated `read_stdin_passphrase` helper alongside the existing `read_stdin_to_string`.

---

## Low / Nit

### N1 — `DeriveChildUnsupportedApp` error says "v0.7" after moving to v0.8 (confidence 85)

**File:** `crates/mnemonic-toolkit/src/error.rs:338`

Message reads `"out-of-scope for v0.7"`, pinned byte-exact in `cell_7_unsupported_application_rsa_refusal`. Not introduced by Phase 1, but Phase 1 moves the project into v0.8. File in FOLLOWUPS for v0.8 release-prep: update message and pinned test together.

**Resolution:** deferred to Phase 7 (BIP-85 RSA / RSA-GPG / DICE) — that phase removes the `rsa|rsa-gpg|dice` token from the unsupported list (or narrows it), which naturally rewrites this message. If Phase 7 spike defers, address in Phase 9 release-prep.

---

## Verified-correct items

1. **All four BIP-38 arms implement the R1-I3 locked semantics.**
   - `(Phrase|Entropy) → Bip38` composite: `bip38_passphrase.unwrap_or("")` — no fallback to `pbkdf2_passphrase`. BREAKING change correctly applied.
   - `(Wif) → Bip38` direct: `bip38_passphrase.unwrap_or(pbkdf2_passphrase)` — v0.7 single-flag UX preserved.
   - `(Bip38) → Wif` direct decrypt: same fallback. Correct.

2. **`--passphrase-stdin` / `--from <node>=-` mutex is correct.** Runtime guard fires before either stdin consumption. Clap `conflicts_with = "passphrase"` handles the `--passphrase + --passphrase-stdin` parse-time conflict.

3. **BIP-38 gate is correct.** Requires `effective_passphrase.is_some() || bip38_passphrase.is_some()` on any BIP-38 edge. On composite paths, passing only `--passphrase` (PBKDF2 only) satisfies the gate and Scrypt correctly defaults to `""`.

4. **BIP-85 language codes match spec.** `resolve_bip85_language` maps en=0, ja=1, kr=2, es=3, zh-Hans=4, zh-Hant=5, fr=6, it=7, cs=8. Portuguese refused with `BadInput` (exit 1). All correct per BIP-85 §"Language Codes".

5. **`NetworkKind::Main` for phrase→master internal xprv is correct.** BIP-85 entropy derivation is network-agnostic at the HMAC level; the master xprv's network field is not an input to any BIP-85 derivation byte. Emission-side network is driven by `--network` independently via `emit_network`.

6. **`composite_phrase_to_bip38_separate_passphrase_semantics_pinned` correctly pins v0.8 BREAKING semantics.** Three assertions present and logically complete: (a) `wif_a ≠ wif_b` (guard against trivial test setup), (b) decrypting `bip38_c` with empty Scrypt recovers `wif_b` (PBKDF2 leg active), (c) `recovered ≠ wif_a` (passphrase not ignored).

7. **`composite_phrase_to_bip38_independent_passphrases` and `direct_wif_to_bip38_passphrase_fallback`** cover the dual-independent-passphrase and v0.7-preserved-fallback paths end-to-end with spec vectors.

8. **Stdin tests for Items #5 and #8** correctly use `write_stdin` and cross-validate against argv-supplied equivalent outputs.

---

## Resolution actions applied

- **I1:** split into `read_stdin_to_string` (key material; full `trim`) and `read_stdin_passphrase` (line-ending strip only). `--passphrase-stdin` switches to the latter; existing `--from <node>=-` arms keep `read_stdin_to_string`.
- **N1:** deferred to Phase 7 / 9 per resolution note above.
