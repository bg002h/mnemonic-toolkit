# cycle-prep recon — 2026-05-21 — bsms-decrypt-record-string-zeroizing + import-wallet-plaintext-blob-mlock-pin

**Origin/master SHA at recon time:** `f501ec3`
**Local branch:** `master`
**Sync state:** `up-to-date (0 ahead / 0 behind)`
**Untracked:** `.claude/` only (does not touch the cited source file)

Slug(s) verified: `bsms-decrypt-record-string-zeroizing`, `import-wallet-plaintext-blob-mlock-pin`. Both are clean — every cited path/symbol/claim is ACCURATE against current source — with two implementation subtleties surfaced below that the FOLLOWUP bodies under-state.

Both are R0 minor-findings (M1/M2) spun out of the v0.33.3 `import-wallet-blob-zeroizing` close, both target the single file `crates/mnemonic-toolkit/src/cmd/import_wallet.rs`, both `v0.33+` / `wallet` tagged, both sibling to the resolved `import-wallet-blob-zeroizing` (v0.33.3). Both `open`.

---

## Per-slug verification

### `bsms-decrypt-record-string-zeroizing`
- **WHAT (from FOLLOWUPS.md):** Migrate `decrypt_bsms_record`'s return type from plain `String` to `Zeroizing<String>` — the intermediate decrypted plaintext is un-zeroized before the v0.33.3 `blob` re-wrap. Low sensitivity (BSMS plaintext is a watch-only descriptor, not seed/xprv).
- **Citations:**
  - `import_wallet.rs` `decrypt_bsms_record` returns a plain `String` — **ACCURATE.** `2161: fn decrypt_bsms_record(text: &str, token: &BsmsToken, ctx: &str) -> Result<String, ToolkitError>`. The `String` is built at `2186: String::from_utf8(plaintext.to_vec())…`.
  - "consumed at the BSMS Round-2 reassign (`~:1043`)" — **ACCURATE** (tilde-approximate). The call is at `1033: let plaintext = decrypt_bsms_record(blob_hex, token, "bsms: encrypted Round-2 wire")?;` and the returned `String` is consumed at `1043: blob = Zeroizing::new(plaintext.into_bytes());`.
  - "+ the Round-1 decrypt path" — **ACCURATE.** Second call site at `2299: let plaintext = decrypt_bsms_record(&raw_text, token, …)?;`, inside the `--bsms-round1` encrypted-record branch.
  - Claim "v0.33.3 scrubs the `blob` copy (the re-wrapped `Zeroizing::new(into_bytes())`)" — **ACCURATE.** `1043: blob = Zeroizing::new(plaintext.into_bytes());`.
  - Sensitivity claim "BSMS Round-2 plaintext is a watch-only descriptor, not seed/xprv" — **ACCURATE** (domain-correct; BSMS records carry descriptors/xpubs).
- **Subtlety surfaced (call-site churn is broader than "minor sig change"):**
  1. **Round-2 (1043):** `Zeroizing<String>` cannot be moved out of via `.into_bytes()` (Zeroizing has a `Drop` impl; it only yields `&`/`&mut` through `Deref`). The reassign must become e.g. `blob = Zeroizing::new(plaintext.as_bytes().to_vec());` — note this introduces one transient non-zeroizing `Vec` from `as_bytes().to_vec()` (immediately moved into the `Zeroizing` wrapper, scrubbed on drop), acceptable but worth a comment.
  2. **Round-1 (2299–2333):** the value flows into `let text = if is_encrypted_bsms_record(&raw_text) { … plaintext } else { raw_text };`. Making `plaintext: Zeroizing<String>` forces the **`else` branch (`2313: raw_text`) to also wrap** so both arms unify (`text: Zeroizing<String>`). The downstream `parse_round1(&text)` (`2315`) is unchanged — `&Zeroizing<String>` deref-coerces to `&str`.
  3. The `2186` body change is `Ok(Zeroizing::new(String::from_utf8(plaintext.to_vec())…?))` (or wrap the existing `String`); the inner `plaintext` is already `Zeroizing<Vec<u8>>` from `bsms_crypto::decrypt`, so its own buffer is independently scrubbed.
- **Action for brainstorm spec:** Change `2161` return → `Result<Zeroizing<String>, ToolkitError>`; wrap at `2186`; fix the two consumers (`1043` move-out workaround; `2313` else-arm wrap). Net ~6–8 lines across one private fn + 2 call sites. Type-only, no test-behavior change (existing cells should pass unchanged). Cite source SHA `f501ec3`.

### `import-wallet-plaintext-blob-mlock-pin`
- **WHAT (from FOLLOWUPS.md):** `run()` only `mlock`-pins the blob inside the BIE1 decrypt branch; the plaintext (`use_encryption:false`, seed-bearing) Electrum path + all other formats leave the blob swappable. Pin `&blob` for ALL formats. Note the existing BIE1 pin is arm-scoped.
- **Citations:**
  - "only the BIE1 decrypt branch calls `mlock::pin_pages_for(&blob)`" — **ACCURATE.** The only `&blob` pin is `435: let _pin_pt = mnemonic_toolkit::mlock::pin_pages_for(&blob);`, inside the `Some(ElectrumStorageMagic::Bie1) => { … }` arm of the `match detect_storage_magic(&blob)` at `400`. (`418` pins the *password* bytes, not the blob.)
  - "the plaintext-import path (`use_encryption:false`) + all other formats never pin the blob" — **ACCURATE.** The `None =>` arm (`437`) and all downstream parse paths never pin.
  - "the blob is now `Zeroizing` (scrubbed on drop, v0.33.3)" — **ACCURATE.** `read_blob` returns `Result<Zeroizing<Vec<u8>>, ToolkitError>` (`2082`); bound at `390: let mut blob = read_blob(blob_path, stdin)?;`.
  - "Pin `&blob` after `read_blob` for ALL formats" — **ACTIONABLE.** `read_blob` is at `390`.
  - "the existing BIE1 pin is itself arm-scoped (dropped at the end of the decrypt arm)" — **ACCURATE.** `_pin_pt` (`435`) is scoped to the `Some(Bie1)` arm and its `PinnedPageRange` guard drops at the arm's closing brace (`436`). `pin_pages_for(buf: &[u8]) -> PinnedPageRange` (`mlock.rs:90`) is RAII-guard-based.
- **Subtlety surfaced (the FOLLOWUP's "pin once at the `blob` binding" advice is too simple):** `blob` is `let mut` and **reassigned twice** — `434: blob = plaintext;` (BIE1) and `1043: blob = Zeroizing::new(plaintext.into_bytes());` (Round-2). A `PinnedPageRange` created at `390/391` captures the *original* buffer's pointer/len; after either reassign the **new** heap buffer is unpinned (and the old guard pins freed/relocated pages). So a single pin at `390` correctly covers the **primary concern** — the plaintext seed-bearing path (`None` arm, no reassign) — but does NOT follow the BIE1 / Round-2 reassigns. The current BIE1 arm already re-pins at `435` after its `434` reassign (correct); the Round-2 reassign at `1043` has no pin (lower stakes — watch-only BSMS descriptor). A holistic fix is therefore "pin at `390` for the common/seed-bearing path **+** retain/refresh the pin after each reassign," not literally "pin once."
- **Action for brainstorm spec:** Add `let _pin_blob = mnemonic_toolkit::mlock::pin_pages_for(&blob);` immediately after `390`, scoped to `run()`. Decide pin-refresh policy for the two reassign sites: the BIE1 arm's `435` re-pin can stay (or be folded into the run-scope guard via reassign-then-repin); the Round-2 `1043` reassign should add a re-pin if BSMS Round-2 plaintext is deemed worth pinning (optional — watch-only). Net ~1–3 lines. Cite source SHA `f501ec3`.

---

## Cross-cutting observations
1. **No structural errors, no line drift.** Every path/symbol/line is accurate against `f501ec3`. The only line cited approximately (`~:1043`) carries its own tilde and is exact for the consume site.
2. **Both slugs touch the SAME file and the SAME `run()` blob lifecycle**, but at non-overlapping points: slug-2 edits cluster at `390–391` (+ optional `435`/`1043` re-pin policy); slug-1 edits at `2161`/`2186` (def) + `1033–1043` (Round-2) + `2299–2313` (Round-1). The only shared line is the Round-2 reassign `1043` — slug-1 rewrites its RHS (`plaintext` move-out), slug-2 may add a re-pin after it. Minor, mechanical coordination if co-shipped.
3. **Shared root cause / shared precedent.** Both are the M1/M2 minors from the v0.33.3 `import-wallet-blob-zeroizing` R0 and both cite/extend the `resolved-slot-derived-account-zeroizing-field` secret-hygiene lineage. They form a natural "import-wallet secret-memory hygiene finish" pair.
4. **No lockstep obligations.** Neither changes any clap flag NAME / option / subcommand / dropdown value, so the GUI `schema_mirror` gate is untouched (no `mnemonic-gui/src/schema/mnemonic.rs` update), the manual CLI-reference mirror is untouched, and there is no sibling-codec companion. Both are internal type/lifetime changes.
5. **Both depend only on already-shipped work** (`import-wallet-blob-zeroizing` v0.33.3, `f501ec3` lineage) — no upstream blockers. Independent of each other (no inter-slug dependency).
6. **Sync state clean** — local `master` == `origin/master` `f501ec3`; working tree matches origin bytes for the cited file.

---

## Recommended brainstorm-session scope
- **Group as ONE cycle** ("import-wallet secret-memory hygiene finish") — both are tiny, same file, same `run()`/blob lifecycle, shared R0 lineage, and co-editing avoids two passes of call-site churn on `import_wallet.rs`. If split for bisect hygiene, ship as **two separate commits** in the same release (slug-2 pin first — it is the lower-churn, higher-value seed-bearing fix; then slug-1 String-zeroize).
- **LOC sizing:** slug-2 ~1–3 lines (+ re-pin policy decision); slug-1 ~6–8 lines across one private fn + 2 call sites. Test surface: type/lifetime-only — existing cells should pass unchanged; add at most a focused assertion if any. Total well under a 50-LOC cycle.
- **SemVer:** **PATCH** (likely `mnemonic-toolkit-v0.33.4`). No public CLI/library API surface change — `decrypt_bsms_record` is a private fn; the pin is internal. (Confirm `decrypt_bsms_record` is not `pub` before finalizing — it is `fn` at `2161`, file-private.)
- **Mandatory locksteps:** NONE (no flag-name change → no GUI `schema_mirror`; no CLI surface change → no manual mirror; no sibling-codec companion).
- **Ordering / dependencies:** independent; either order. Recommended within-cycle order = slug-2 (mlock pin) then slug-1 (String zeroize), so the slug-1 Round-2 `1043` rewrite lands after any slug-2 re-pin decision at that line.
- **Brainstorm must resolve:** (a) slug-2 re-pin policy at the two `blob` reassign sites (run-scope guard vs. arm-local; whether to pin the watch-only Round-2 reassign at all); (b) slug-1 Round-2 move-out idiom (`as_bytes().to_vec()` vs. alternative) and the Round-1 `else`-arm unification.
