# PLAN — `import-wallet-blob-zeroizing` (scrub the decrypted/plaintext wallet blob)

**FOLLOWUP:** `import-wallet-blob-zeroizing` (filed v0.33.2 Cycle 19 Phase B; end-of-cycle opus I1).
**cycle-prep recon:** `cycle-prep-recon-import-wallet-blob-zeroizing.md` (origin/master `db9657f`, 0/0 sync).
**Precedent:** `resolved-slot-derived-account-zeroizing-field` (resolved `mnemonic-toolkit-v0.10.1` `ed5a1d9` — `Vec<u8>` → `Zeroizing<Vec<u8>>` field-type migration).

## 1. Goal

The `import-wallet` orchestrator reads the wallet blob into a plain `Vec<u8>` (`read_blob` → `Vec<u8>`) that can hold secret material — a plaintext Electrum wallet (`use_encryption:false`) already carries a seed in that Vec, and v0.33.2 newly writes *decrypted* BIE1 wallet JSON (seed/xprv-bearing) into it. The bytes are `mlock`-pinned (no swap) but never scrubbed before drop. Migrate the binding to `Zeroizing<Vec<u8>>` so the plaintext is wiped on drop.

**Internal hygiene only.** No CLI/wire/GUI/manual surface change → **no schema-mirror or manual lockstep**. SemVer **PATCH** v0.33.2 → v0.33.3.

## 2. Scope (verified against `db9657f`, all in `crates/mnemonic-toolkit/src/cmd/import_wallet.rs`)

### Edits
1. **`read_blob`** (`:2067`): `fn read_blob<R: Read>(...) -> Result<Vec<u8>, ToolkitError>` → `-> Result<Zeroizing<Vec<u8>>, ToolkitError>`; wrap both returns (`Ok(Zeroizing::new(buf))`, `Ok(Zeroizing::new(fs::read(path)?))`). Single caller (`:389`).
2. **`blob` binding** (`:389`): `let mut blob = read_blob(...)?;` — type now infers `Zeroizing<Vec<u8>>`.
3. **BIE1 reassign** (`:430`): `blob = plaintext.to_vec();` → `blob = plaintext;` (`plaintext` is already `Zeroizing<Vec<u8>>` from `ecies_decrypt_storage` — drops the lossy `.to_vec()`, PRESERVING the wrapper). Reorder so the `mlock::pin_pages_for` pin is taken on `&blob` AFTER the move (the current pin on `&plaintext` would otherwise borrow a moved value).
4. **BSMS Round-2 reassign** (`:1036`): `blob = plaintext.into_bytes();` → `blob = Zeroizing::new(plaintext.into_bytes());` (`plaintext` here is a `String` — the decrypted descriptor; watch-only but decrypted-secret-adjacent).
5. **Import** `use zeroize::Zeroizing;` (or fully-qualified; the file already references `zeroize::Zeroizing` in `resolve_import_decrypt_password`).

### Read sites — expected NO change (deref coercion `Zeroizing<Vec<u8>> → Vec<u8> → [u8]`)
`detect_storage_magic(&blob)` (`:399`), `from_utf8(&blob)` (`:419`, `:1023`), `sniff_format(&blob)` (`:455`), the 8 `…Parser::parse(&blob, stderr)` arms (`:1040-1047`), `&blob` in the roundtrip block (`:1151`, `:1160`). All target `&[u8]` params; multi-step deref coercion applies at the call site. **If any site fails to coerce, fix with `&blob[..]`** (compiler will flag). The `canonicalize_*` / `emit_roundtrip_*` HELPERS take `blob: &[u8]` PARAMS (`:1254+`, `:1382+`, `:1904+`, test `:2437`) — unaffected (they receive a coerced `&[u8]`).

### mlock interaction
The BIE1 path currently pins `&plaintext` then `blob = plaintext.to_vec()`. After the change: `blob = plaintext;` then `let _pin = mnemonic_toolkit::mlock::pin_pages_for(&blob);`. `pin_pages_for` takes `&[u8]` + returns an owned guard (does not hold the borrow — same usage as `electrum_decrypt.rs:119`), and the Vec move keeps the same heap buffer, so the pinned pages are unchanged. The BSMS path may add an analogous pin (optional — descriptor is watch-only; keep parity-minimal).

## 3. Non-goals
- No `impl Drop` to delete (unlike the precedent — `blob` is a local, not a struct field; no E0509 move-out concern).
- No runtime "was it zeroized?" assertion (impossible without Miri/unsafe; the precedent shipped on the `Zeroizing` type guarantee alone). Regression coverage = the existing 2253-cell suite must stay green (proves no behavior change).
- No new dep (`zeroize` already present).

## 4. Tests
- Existing full suite (2253 cells) is the regression gate — every import path still parses identically (the migration is type-only; deref coercion preserves behavior).
- Optional: a doc-comment on `read_blob` noting the `Zeroizing` invariant. No new cell (type-level guarantee; consistent with `resolved-slot-derived-account-zeroizing-field` which shipped without a runtime zeroize cell).

## 5. Ship
`Cargo.toml` 0.33.2→0.33.3; `scripts/install.sh` pin; `CHANGELOG [0.33.3]`. Opus end-of-cycle review (confirm no read site silently changed semantics, no un-pinned secret path, the two reassigns covered). Commit + tag `mnemonic-toolkit-v0.33.3` + push + GH release; `install-pin-check` CI green. Close FOLLOWUP `import-wallet-blob-zeroizing`. **No GUI/manual lockstep.**

## 6. Risks
- A `&blob` site that needs `&Vec<u8>` (not `&[u8]`) or moves `blob` by value — none found in recon, but the compiler is the backstop (mechanical `&blob[..]` fix).
- The mlock-pin reorder (covered in §2.5).
- clippy `redundant_clone` / `needless_borrow` after dropping `.to_vec()` — re-run clippy.
