# mnemonic-toolkit-v0.33.0 Implementation Plan (Cycle 18 — electrum-crypto-seed-extraction-subcommand)

> **For agentic workers:** REQUIRED SUB-SKILL: `superpowers:subagent-driven-development` or direct execution.

**Goal:** Ship `mnemonic-toolkit-v0.33.0` (SemVer-MINOR; new subcommand). Closes `electrum-crypto-seed-extraction-subcommand` FOLLOWUP. Surface the (currently dead-code) `electrum_crypto::decrypt_field` primitive via a new `mnemonic electrum-decrypt` subcommand that decrypts an Electrum field-encrypted secret (base64 `iv‖aes-cbc(plaintext+PKCS7)`, key = sha256d(password)) and emits the recovered plaintext (Electrum-native seed phrase or BIP-32 xprv).

**Architecture:** Dedicated top-level subcommand (Option A — architect + user locked 2026-05-21; the decrypted node-type is unknowable pre-decryption, so a convert-source NodeType would collide with convert's commit-types-up-front model + the `(Phrase, ElectrumPhrase)` artifact-class refusal). Mirrors the `cmd/seedqr.rs` standalone template: library primitive + `map_*_error` boundary + `secret_in_argv_warning` + `Zeroizing` + `mlock::pin_pages_for`. The new subcommand is the natural shared home for the forthcoming Format-B whole-file sibling (`wallet-import-electrum-encrypted-storage-format-b`) which reuses the same `--decrypt-password*` family.

**Tech Stack:** Rust; reuses `electrum_crypto` (Cycle 6a) + `clap`; zero new crate deps; zero new `ToolkitError` variants (map `ElectrumDecryptError` → `BadInput` via a new `pub(crate) map_electrum_decrypt_error`); new `cmd/electrum_decrypt.rs` module + 1 `Command` enum arm.

**P0 STRICT-GATE recon (verified at master HEAD `594e742`):**
- `electrum_crypto.rs:96` `derive_key` (sha256d), `:114` `decrypt_field(b64, password) -> Zeroizing<String>`, `:46` `ElectrumDecryptError` (Base64DecodeFailure / CiphertextTooShort / CiphertextNotBlockAligned / AesDecryptFailure / Utf8DecodeFailure). Zero CLI consumers (dead code).
- `electrum_crypto.rs:203-209` known cross-impl test vector: `TEST_PASSWORD=b"test-password"`, `TEST_PLAINTEXT="hello world"`, `TEST_CIPHERTEXT_B64="ABEiM0RVZneImaq7zN3u/zY0181f7qAY/NWiVQFLdHE="` (validated vs Python `cryptography` in Cycle 6a).
- `main.rs:59` `enum Command` + `:119` dispatch match. New arm `ElectrumDecrypt`.
- `cmd/seedqr.rs` template: `--json-out` envelope, `secret_in_argv_warning`, `read_stdin_to_string`, `mlock::pin_pages_for`, `map_seedqr_error` pattern.
- `cmd/convert.rs:707` `read_stdin_passphrase` (NULL-preserving stdin read) — the password-stdin model.
- `secret_advisory::secret_in_argv_warning` + the secret-on-stdout advisory pattern.

**Design locks (architect + user, 2026-05-21):**
- **Option A**: dedicated `mnemonic electrum-decrypt` subcommand.
- **Password forms: ALL THREE** — `--decrypt-password <VAL>` (inline; argv-leakage advisory) + `--decrypt-password-file <PATH>` + `--decrypt-password-stdin`. Mutually exclusive + exactly one required (clap `ArgGroup` `required(true)` `multiple(false)`).

**SemVer rationale (v0.32.3 → v0.33.0 MINOR):** new top-level subcommand = clap surface addition. Per project precedent (v0.30.0 seedqr, v0.11.0 final-word, v0.12.0 seed-xor, v0.13.0 slip39 — all new-subcommand MINORs). Triggers GUI `schema_mirror` lockstep (new `SubcommandSchema` entry) → paired `mnemonic-gui-v0.18.0`.

## File structure

### Source files (toolkit)
- `crates/mnemonic-toolkit/src/secret_advisory.rs` (MODIFY — R0 I1 fold):
  - Extract the secret-on-stdout warning text into a new `pub(crate) fn secret_on_stdout_warning_unconditional<W: Write + ?Sized>(stderr: &mut W)` that ALWAYS emits the warning; refactor the existing `CardKind`-gated `secret_on_stdout_warning` to delegate to it for `Ms1` (behavior-preserving). The existing helper cannot fire for a free-form Electrum plaintext (it's `CardKind`-typed); this generalization is the net-new mechanic the plan previously hid behind "mirror seedqr" (seedqr in fact emits NO stdout advisory — the wrong model).
- `crates/mnemonic-toolkit/src/cmd/electrum_decrypt.rs` (NEW):
  - `ElectrumDecryptArgs` (clap `Args`): `--ciphertext <VALUE|->` (the base64 field; `-` reads stdin; NOT secret — it's ciphertext, so NO argv advisory); the 3 password flags (`--decrypt-password <VAL>` `Option<String>`, `--decrypt-password-file <PATH>` `Option<PathBuf>`, `--decrypt-password-stdin` `bool`) bound by a **struct-level** `#[command(group(ArgGroup::new("decrypt_password").args(["decrypt_password", "decrypt_password_file", "decrypt_password_stdin"]).required(true).multiple(false)))]` (R0 I3 fold — mirrors `repair.rs:24-33` / `inspect.rs:23-29`); `--json-out <PATH>`.
  - `run<R,W,E>(args, stdin, stdout, stderr)`: (1) resolve password (one of 3 forms; inline → `secret_in_argv_warning`; file → read; stdin → `read_stdin_passphrase`); (2) resolve ciphertext (inline or stdin); (3) single-stdin guard (`--ciphertext -` XOR `--decrypt-password-stdin`); (4) `electrum_crypto::decrypt_field` → `map_electrum_decrypt_error`; (5) pin + zeroize the password + plaintext via `mlock::pin_pages_for`; (6) emit plaintext on stdout → `secret_on_stdout_warning_unconditional` (R0 I1) OR `--json-out` envelope → `warn_if_world_readable(path, stderr)` (R0 I2 — cite `seed_xor.rs:444` / `slip39.rs` / `final_word.rs:179`, NOT seedqr which omits it).
  - `map_electrum_decrypt_error(e) -> ToolkitError::BadInput` — `AesDecryptFailure`/`Utf8DecodeFailure` BOTH → "decryption failed (wrong password or corrupted ciphertext)" (R0 Q4 — unified; no mode leak; a right password never yields Utf8DecodeFailure per `electrum_crypto.rs:57`).
  - `ElectrumDecryptEnvelope` (Serialize): `schema_version` + `operation: "electrum-decrypt"` + `plaintext` (NO password echo).
- `crates/mnemonic-toolkit/src/cmd/mod.rs` — `pub mod electrum_decrypt;`.
- `crates/mnemonic-toolkit/src/main.rs:59` — `ElectrumDecrypt(cmd::electrum_decrypt::ElectrumDecryptArgs)` arm + doc comment; `:119` dispatch.
- `crates/mnemonic-toolkit/src/electrum_crypto.rs` — confirm `derive_key`/`decrypt_field` are `pub` (they are); no change unless the `#![allow(dead_code)]`-style suppression needs lifting once consumed.

### Test files (toolkit)
- `crates/mnemonic-toolkit/tests/cli_electrum_decrypt.rs` (NEW):
  - `decrypt_inline_password_happy_path` — `--ciphertext <TV> --decrypt-password test-password` → stdout `"hello world\n"` + secret-on-stdout advisory + argv-leakage advisory (inline password).
  - `decrypt_password_stdin_happy_path` — `--decrypt-password-stdin` (password via stdin) → no argv advisory.
  - `decrypt_password_file_happy_path` — `--decrypt-password-file <tmp>`.
  - `decrypt_wrong_password_refused` — wrong password → BadInput "decryption failed (wrong password or corrupted ciphertext)" exit 1.
  - `decrypt_bad_base64_refused`.
  - `decrypt_no_password_required` — no password flag → clap ArgGroup error (exit 64).
  - `decrypt_two_password_forms_conflict` — `--decrypt-password X --decrypt-password-stdin` → clap conflict (exit 64).
  - `decrypt_ciphertext_stdin_and_password_stdin_refused` — both `-`+`--decrypt-password-stdin` → single-stdin BadInput.
  - `decrypt_json_envelope` — `--json-out` → `{operation: "electrum-decrypt", plaintext: "hello world"}` (+ assert NO password field).
  - `ciphertext_stdin_happy_path` — `--ciphertext -` + `--decrypt-password-file`.
  - `decrypt_realistic_seed_fixture` (R0 Q9) — a realistic Electrum-seed-shaped plaintext minted via `electrum_crypto::encrypt_field` (deterministic test IV) → decrypt round-trips byte-equal (beyond the toy "hello world" TV).
  - `json_out_world_readable_advisory` (R0 I2) — `--json-out` to a 0o644 path emits the `warn_if_world_readable` advisory.

### Documentation (toolkit)
- `docs/manual/src/40-cli-reference/41-mnemonic.md` — NEW `## mnemonic electrum-decrypt` section (synopsis, flags, the 3 password forms, secret-on-stdout note, worked example using the TV). Update the chapter's subcommand index/TOC.

### Cross-repo (GUI lockstep — Cycle 18b, MANDATORY)
- `mnemonic-gui/src/schema/mnemonic.rs` — new `ELECTRUM_DECRYPT_FLAGS` SubcommandSchema (`--ciphertext`, `--decrypt-password`, `--decrypt-password-file`, `--decrypt-password-stdin`, `--json-out` + `--no-auto-repair`). Register in the subcommand list.
- `mnemonic-gui/pinned-upstream.toml` + `Cargo.toml` pin → v0.33.0. GUI v0.18.0 (MINOR).

### Release tooling
- `Cargo.toml:3` — `0.32.3` → `0.33.0`.
- `CHANGELOG.md` — `## [0.33.0]`.
- `scripts/install.sh:32` — pin → `v0.33.0`.
- `design/FOLLOWUPS.md` — close `electrum-crypto-seed-extraction-subcommand` + file `gui-electrum-decrypt-subcommand-mirror`.

## Tasks

### Task 1: Phase 2 — `cmd/electrum_decrypt.rs` + wiring
- [ ] New module: args (ArgGroup) + run + error map + envelope; register in main.rs + cmd/mod.rs.
- [ ] Build + smoke-test the TV.
- [ ] Commit Phase 2.

### Task 2: Phase 3 — integration tests
- [ ] 10 cells per the plan.
- [ ] Build + run.
- [ ] Commit Phase 3.

### Task 3: Phase 4 — manual
- [ ] New subcommand reference section + TOC + gui-schema fixture if any.
- [ ] Manual lint.
- [ ] Commit Phase 4.

### Task 4: Phase 5 — toolkit cycle close
- [ ] Version + install.sh + CHANGELOG.
- [ ] Pre-tag audit (test + clippy + manual lint + gui-schema JSON check).
- [ ] Opus end-of-cycle review (secret-handling focus).
- [ ] Commit + tag v0.33.0 + push + GH Release.
- [ ] install-pin-check CI green.
- [ ] Close FOLLOWUP + file GUI lockstep FOLLOWUP + memory.

### Task 5: Phase 6 — GUI v0.18.0 lockstep (Cycle 18b)
- [ ] Add ELECTRUM_DECRYPT_FLAGS schema + register; pin bump; schema_mirror green; tag + release.

## Cross-phase invariants
- Opus R0 review on plan-doc BEFORE Phase 2.
- Opus end-of-cycle review BEFORE tag (SECRET-HANDLING focus: zeroize + mlock + advisories + no password echo).
- MANDATORY GUI lockstep (new subcommand).
- No `cargo fmt --all`.
- install-pin-check CI gate.

## Risk register
- **Secret hygiene** — the password (inline form) + the decrypted plaintext are secret. Inline `--decrypt-password <VAL>` MUST emit the argv-leakage advisory; the plaintext stdout MUST emit the secret-on-stdout advisory; both password + plaintext MUST be `Zeroizing` + `mlock::pin_pages_for`. NO password echo in the JSON envelope. Mirror `cmd/seedqr.rs` exactly.
- **Wrong-password UX** — Format A has NO MAC, so a wrong password manifests as PKCS7-unpad failure (`AesDecryptFailure`) or non-UTF8 (`Utf8DecodeFailure`). Map BOTH to a single clear "decryption failed (wrong password or corrupted ciphertext)" message (do not leak which failure mode).
- **Single-stdin contention** — `--ciphertext -` and `--decrypt-password-stdin` both want stdin; refuse the combination (BadInput), mirroring convert's dual-stdin guards.
- **ArgGroup semantics** — `required(true) multiple(false)` enforces exactly-one-password-form at clap parse (exit 64). Verify clap-derive `#[group(...)]` produces the right behavior; test the none + multiple cases.
- **GUI schema_mirror** — a NEW subcommand is a real flag-NAME-set addition; the gate WILL fire on the GUI pin bump if the GUI schema lacks `electrum-decrypt`. MANDATORY paired GUI v0.18.0.

## Self-review (pre-R0 dispatch)
- ✓ Recon + known TV + registration site confirmed.
- ✓ Option A + 3-form password locked (architect + user).
- ✓ Secret-hygiene requirements enumerated (zeroize/mlock/advisories/no-echo).
- ✓ Wrong-password mapping (no MAC → unify the two failure modes).
- ✓ SemVer MINOR + mandatory GUI lockstep classified.
- ✓ Test surface: 10 cells incl. ArgGroup + stdin-contention.
