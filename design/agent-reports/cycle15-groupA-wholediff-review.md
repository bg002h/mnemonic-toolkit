# WHOLE-DIFF REVIEW (verbatim) — cycle-15 Group A (toolkit v0.69.0)

**Dispatched:** 2026-06-22. **Range:** `a7188aca..360a6e88` (3 impl commits, off master `9b7c78a7`=v0.68.0). **Mandatory independent post-impl adversarial review.**
**Verdict:** GREEN (0C/0I) — cleared to push/tag.

---

Final verification complete. The stale defending doc ("attacker who can read process stack", "short-lived stack value", "A caller retaining") is fully removed from the source — only present in the FOLLOWUPS.md historical record (which is correct, as it documents the prior rationale). The new doc correctly frames it as secret-class with the scrub obligation in the type.

The body builds output directly as `Zeroizing::new([0u8; 32])`, fills it in place via `copy_from_slice`, and returns by move (`out`). No bare `[u8;32]` copy of the HMAC_KEY escapes — `finalize()` produces a `GenericArray` consumed immediately by `copy_from_slice` into the Zeroizing buffer (no intermediate named bare copy). `compute_mac` correctly stays bare with the public-auth-tag note.

All checks pass. Here is my verdict.

---

## VERDICT: GREEN (0C / 0I)

This shipped-but-not-yet-tagged cycle-15 Group A (toolkit v0.69.0) is clean. No Critical, no Important, no Minor findings. Cleared to push/tag.

### Critical: none
### Important: none
### Minor: none

### Verification notes (what I ran + outcomes)

**#3 secret-hygiene correctness — VERIFIED.** `derive_hmac_key` (`bsms_crypto.rs:112-119`) builds output directly as `Zeroizing::new([0u8;32])`, fills in place via `copy_from_slice`, returns by move. No bare `[u8;32]` HMAC_KEY copy escapes anywhere (the `finalize()` `GenericArray` is consumed immediately, never bound to a named bare copy). The stale defending doc ("attacker who can read process stack…", "short-lived stack value", "A caller retaining…") is fully removed from source — present only in the FOLLOWUPS historical record, which is correct. `compute_mac` (`:137`) correctly left bare with the new public-auth-tag note (first-16-bytes-→-IV, compared vs untrusted `mac_recv`).

**#3 ZERO-caller-edit claim — VERIFIED TRUE.** All 4 call-site categories are byte-identical to base (`git diff` on `cmd/import_wallet.rs` and the integration test = empty): prod `cmd/import_wallet.rs:2428/2435`, in-module tests `bsms_crypto.rs:238/290`, integration helper `cli_import_wallet_bsms_encrypted.rs:316`. All use `compute_mac(&hmac_key, …)` / `hex::encode(hmac_key)` unedited — `&Zeroizing<[u8;32]>` deref-coerces to `&[u8;32]`; `hex::encode` rides `AsRef`. No `*`/`&*`/deref noise introduced. The `derive_hmac_key_returns_zeroizing` fn-pointer fence is a genuine compile fence (return types are invariant). **TV3 byte invariant CONFIRMED unchanged** — ran `--lib bsms_crypto` (21 passed) + `--test cli_import_wallet_bsms_encrypted` (28 passed): `tv3_derive_hmac_key_matches_bip129`, `tv3_compute_mac_matches_bip129`, `tv3_iv_is_first_16_bytes_of_mac`, `tv3_encrypt_produces_bip129_ciphertext_byte_identical`, `tv3_end_to_end_round_trip`, `coinkite_xref_round2_full_plaintext_byte_equal`, `wrong_token_yields_mac_mismatch_exit_2` all green.

**#4 guard soundness — VERIFIED LOAD-BEARING, NOT VACUOUS.**
- (a) Union present in BOTH consumers. I temporarily removed the `.chain(TEST_ONLY_SECRET_FILES.iter())` from the scan test → it went **RED with exactly `src/bundle_unified.rs` undeclared** (panic at `:601`), then restored to clean (`git diff` empty). The union is load-bearing.
- (b) Index math correct. `production_secret_lines` uses `.take(boundary)` where `boundary` = 0-based index of first `#[cfg(test)]`, correctly **excluding** the boundary line. Verified against the real file: `bundle_unified.rs` has `#[cfg(test)]` at line 118 and its sole `SecretString::new` at line 128 (after the marker) → empty production lines → passes correctly; a secret added above line 118 would be flagged.
- (c) Negative unit test (`confinement_helpers_flag_production_secret_above_cfg_test`) proves the property both ways — synthetic with a prod `Zeroizing::new` above `#[cfg(test)]` asserts `== vec![0]`; clean variant asserts empty. Not a pass-if-broken test.
- (d) `src/bundle_unified.rs` genuinely UNCHANGED (`git diff` over range = empty).
- (e) Live partition count = **38** (computed independently mirroring the 4 SECRET_PATTERNS scan); `SECRET_FILE_FLOOR = 37` unchanged (38 ≥ 37). `bundle_unified.rs` IS in the partition, so the TEST_ONLY entry is genuinely required.

**nit#2 — VERIFIED.** All three decaying numbers (`live 54`, `60 to 66`, `36 + 16 = 52`) removed (grep empty), `{n}` interpolation + `18..=66` bound kept, `canonical_zeroize_list_has_expected_row_count` passes.

**nit#3 + FOLLOWUPS — VERIFIED.** Both `bsms-derive-hmac-key-not-zeroizing` and `bundle-unified-whole-file-allowlist-precision` flipped `open → resolved` with accurate, detailed v0.69.0 notes. New `bip85-encode-helper-internal-scratch-zeroizing` slug is well-formed; its cites are accurate — `bip85.rs:189` = `let encoded = base64_standard(&entropy[..])`, `:204` = `let encoded = base85_btc(&entropy[..])`, `:252` = `let mut out: Vec<String> = Vec::with_capacity(...)`.

**SemVer / release — VERIFIED.** All 6 version sites at 0.69.0 (Cargo.toml `:3`, root README, crate README, install.sh, Cargo.lock toolkit pkg `:727`, fuzz/Cargo.lock `:575`). No stray 0.68.0 residue in any version-coupled file. CHANGELOG entry present and accurate. MINOR is correct for the pub-signature secret-type migration.

**Scope fences — VERIFIED.** Exactly the 10 planned files touched; no clap/CLI/schema/manual edits, no `cargo fmt` churn, nothing outside the planned set.

**Full gate — GREEN.** `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `cargo test -p mnemonic-toolkit` exit 0 (lint suite 6/6, bsms unit 21/21, bsms integration 28/28, all binaries 0 failed).
