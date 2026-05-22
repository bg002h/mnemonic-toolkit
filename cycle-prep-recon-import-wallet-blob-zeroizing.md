# cycle-prep recon — 2026-05-21 — `import-wallet-blob-zeroizing`

> NOTE: the `/cycle-prep` skill is not loadable in this session (present in
> `skillUsage` history but no on-disk definition; the Skill tool returns
> "Unknown skill"). This recon was produced manually in the skill's
> established `cycle-prep-recon.md` format.

**Origin/master SHA at recon time:** `db9657f`
**Local branch:** `master`
**Sync state:** `up-to-date` (0 ahead / 0 behind)
**Untracked:** `.claude/` (gitignored per session)

Slug verified: `import-wallet-blob-zeroizing` (filed THIS session at the v0.33.2 Cycle 19 Phase B close, end-of-cycle opus I1). Drift expected minimal (same-session filing). The recon surfaced **one citation under-count** (a SECOND secret-reassign site the FOLLOWUP body omits) and **one mechanical-scope clarification** (call-site `&blob` deref churn). No STRUCTURAL errors; no DRIFTED-by-N findings.

---

## Per-slug verification

### import-wallet-blob-zeroizing

- **WHAT (from FOLLOWUPS.md L2954-2958):** the import-wallet `blob: Vec<u8>` can hold secret material; migrate it to a zeroizing field type (mirror `resolved-slot-derived-account-zeroizing-field`) so all import paths scrub the plaintext before drop. `read_blob` returns a plain `Vec<u8>`; the BIE1 decrypt path does `blob = decrypted.to_vec()`, dropping the `Zeroizing` wrapper.
- **Citations:**
  - `read_blob -> Result<Vec<u8>, ToolkitError>` (plain `Vec`) — **ACCURATE** (`cmd/import_wallet.rs:2067`).
  - `let mut blob = read_blob(...)` single binding — **ACCURATE** (`:389`).
  - `blob = plaintext.to_vec()` drops `Zeroizing` (BIE1 path) — **ACCURATE** (`:430`; `plaintext` is the `Zeroizing<Vec<u8>>` returned by `ecies_decrypt_storage`).
  - "all 8 import paths benefit" — **CLAIM-COUNTING / UNDER-COUNT**: there are **8 vendor formats** (bitcoin-core, bsms, coldcard, coldcard-multisig, electrum, jade, sparrow, specter) but **9** `…Parser::parse(&blob` call sites at HEAD (one format is reached from two sites, OR a test — reconcile at brainstorm time; the architectural point holds regardless: there is ONE `blob` binding read by every arm). **MORE IMPORTANTLY — the FOLLOWUP omits a SECOND secret-bearing reassign:** `blob = plaintext.into_bytes()` at **`:1036`** (the BSMS encrypted Round-2 decrypt path) ALSO writes decrypted material into the plain `Vec`. That plaintext is a descriptor (watch-only, lower sensitivity than seed/xprv), but it is a decrypted secret-adjacent reassign and MUST be covered by the same migration. The brainstorm spec should cite BOTH `:430` (BIE1, seed/xprv-bearing) and `:1036` (BSMS Round-2, descriptor).
  - precedent `resolved-slot-derived-account-zeroizing-field` — **ACCURATE + APT** (FOLLOWUPS.md L535; `resolved ed5a1d9` / `mnemonic-toolkit-v0.10.1`). It migrated `ResolvedSlot.entropy` + `DerivedAccount.entropy` `Vec<u8>` → `Zeroizing<Vec<u8>>`, deleted `impl Drop for DerivedAccount`, fixed move-out destructuring (E0509), relabeled the lint anchor + added a row, + CHANGELOG. Directly transferable pattern.
- **Action for brainstorm spec:**
  1. Migrate `read_blob` return type → `Result<Zeroizing<Vec<u8>>, ToolkitError>` and the `blob` binding → `Zeroizing<Vec<u8>>`. The BIE1 site becomes `blob = plaintext;` (no `.to_vec()`, preserving the wrapper — a clean simplification); the BSMS site becomes `blob = Zeroizing::new(plaintext.into_bytes());`.
  2. **Mechanical call-site churn (the real cost):** `Zeroizing<Vec<u8>>` Derefs to `Vec<u8>`, but `&blob` is NOT `&[u8]`. Every consumer that passes `&blob` to a `&[u8]` parameter needs `&blob[..]` / `blob.as_slice()` / `&*blob`: the 9 `Parser::parse(&blob, …)` arms (`:~923-939`), `sniff_format(&blob)` (`:392`), `detect_storage_magic(&blob)` (`:399`), and the BSMS `std::str::from_utf8(&blob)` (`:907`-area). Enumerate + update each. ~20-40 LOC, mechanical.
  3. No `impl Drop` to delete here (unlike the precedent — `blob` is a local, not a struct field), so the E0509 move-out concern does not apply; simpler than `resolved-slot-derived-account-zeroizing-field`.
  4. Cite source SHA `db9657f`.

---

## Cross-cutting observations

1. **One citation under-count** — the FOLLOWUP body cites only the BIE1 `:430` reassign; the BSMS `:1036` `into_bytes()` reassign is a second secret-adjacent site the migration must cover. Re-word the body to "two decrypt-reassign sites (`:430` BIE1 + `:1036` BSMS Round-2) plus all plaintext-import reads share the one `blob` binding."
2. **No STRUCTURAL errors, no DRIFTED-by-N** — same-session filing; all line cites align with HEAD `db9657f`.
3. **Scope is mechanical, not architectural** — one type change on `read_blob` + `blob`, propagated to ~12 `&blob` call sites via deref. No new dep, no wire/behavior change, no GUI/manual surface (internal hygiene only). The import OUTPUT remains watch-only/non-secret; this only scrubs the in-memory plaintext earlier.
4. **Sync state** — local master ≡ origin/master at `db9657f`; recon verified against HEAD bytes (= origin bytes); no `git show origin/master:` fallback needed.

---

## Recommended brainstorm-session scope

Single small cycle (no pairing needed). **SemVer PATCH** (internal hygiene; no CLI/wire/GUI surface change → no schema-mirror or manual lockstep). ~30-50 LOC. Test angle: a regression cell is hard (you cannot assert a `Vec` was zeroized post-drop without `unsafe`/Miri); the precedent shipped without a runtime zeroize assertion and relied on the type system + an `#[ignore]` Miri/`Drop`-invariant gate. Recommend: rely on the `Zeroizing` type guarantee + a compile-level check, and note the no-runtime-assertion limitation in the cycle close (as `resolved-slot-derived-account-zeroizing-field` did). Optional: a `tests/` doc-comment or a clippy lint pin that the `blob` binding type is `Zeroizing`.
