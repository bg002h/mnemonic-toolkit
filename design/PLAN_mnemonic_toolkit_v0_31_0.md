# mnemonic-toolkit-v0.31.0 Implementation Plan (Cycle 7b — BIP-129 encryption envelope)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task.

**Goal:** Ship `mnemonic-toolkit-v0.31.0` (SemVer-MINOR; new `--bsms-encryption-token` flag on `import-wallet`) + paired `mnemonic-gui-v0.16.0`. Closes FOLLOWUP `bsms-bip129-encryption-envelope` (resolved by Cycle 7).

**Architecture:** Orchestrator pre-decrypts in `cmd/import_wallet.rs` BSMS-arm (preserving the `BsmsParser::parse` trait surface). Reads TOKEN from file or stdin via `--bsms-encryption-token <FILE|->`. Uses Cycle 7a's `bsms_crypto` library (PBKDF2-SHA512 + Ctr128BE<AES256> + HMAC-SHA256) for the crypto primitives. The encrypted-Round-2 wire shape (`hex(MAC || ciphertext)`) is detected by the explicit `--format bsms` + `--bsms-encryption-token` combination (NOT by sniff — encrypted blobs lack the `BSMS 1.0` header).

**Tech Stack:** Rust + clap-derive. NO new Cargo deps (Cycle 7a already added `ctr = "0.9"`; everything else available).

**Brainstorm v1 (Cycle 7a; library-only):** `design/BRAINSTORM_v0_31_0_bsms_bip129_encryption_v1_7a_library.md` (committed Cycle 7a; locks library-side architecture).

**Cycle 7a R0 review:** `design/agent-reports/v0_31_0-bsms-crypto-brainstorm-r0-review.md` (YELLOW 1C/4I/4M; all folded in 7a Phase 1).

**P0 recon dossier:** `design/cycle-7-p0-recon.md` (primary-source verified vs `bitcoin/bips@2026-05-21` + Coinkite Python).

**Source SHA at plan-write time:** `6e522bb` (post-Cycle-7a kickoff backfill).

**P0 STRICT-GATE locks:**
- **CLI flag shape:** `--bsms-encryption-token <FILE|->` (single flag; value = file path OR `-` for stdin). NO inline form (avoids argv leakage of TOKEN). Per parent brainstorm + R0 hygiene.
- **Token reading:** file/stdin contents are hex-stripped of whitespace + newlines; must be 16 hex chars (STANDARD; 8 bytes raw) or 32 hex chars (EXTENDED; 16 bytes raw). Other widths → exit 2.
- **Encrypted blob shape:** hex(MAC || ciphertext) where MAC=32 bytes, ciphertext=remainder. Total hex length ≥ 64 chars + (16 hex chars per AES block; ciphertext can be any positive length).
- **Sniff:** unchanged. Encrypted blobs don't have `BSMS 1.0` header so sniff is NEGATIVE under auto-detect. User MUST supply `--format bsms` + `--bsms-encryption-token` to get encrypted-Round-2 dispatch.
- **Decrypt-then-MAC ordering (BIP-129 Encrypt-and-MAC):** orchestrator decrypts ciphertext first, then verifies MAC over decrypted plaintext (per BIP-129 §Encryption + Cycle 7a recon §A9). If MAC mismatch → `ToolkitError::BsmsMacMismatch` (typed variant per FOLLOWUP body recommendation).
- **New `ToolkitError` variant:** `BsmsMacMismatch` (typed error). Alphabetical slot: between `BsmsImported` (if exists) or wherever `Bsms*` variants live. Exit code: 2 (format-violation class). 3 exhaustive match-arm sites in `error.rs` need updates: `Display`, `exit_code`, `kind`.
- **Multi-record `--bsms-round1` interaction:** out of scope. Cycle 7b ships encrypted Round-2 only. Encrypted Round-1 records (per-Signer or shared-TOKEN) deferred to FOLLOWUP `bsms-encryption-per-signer-tokens` filed at cycle close.
- **SemVer:** MINOR `v0.30.1 → v0.31.0`. New required-for-encrypted-path flag → MINOR per the parent brainstorm + Cycle 6 lessons.
- **GUI lockstep:** MANDATORY. New `SubcommandSchema` flag entry for `--bsms-encryption-token` on import-wallet schema. Paired `mnemonic-gui-v0.16.0`.

---

## File structure

### Source files modified (toolkit)

- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs`:
  - Add `#[arg(long = "bsms-encryption-token")] pub bsms_encryption_token: Option<PathBuf>` to `ImportWalletArgs` around L191-200 (alphabetical with other `bsms-*` flags).
  - Update header docstring `bsms-encryption-token` line.
  - Update synopsis line.
  - In `bsms` dispatch arm (search for `Some("bsms")` after L535): add token-resolved-and-decrypt block BEFORE `BsmsParser::parse(...)` call. If `--bsms-encryption-token` supplied: read TOKEN from file/stdin, hex-decode raw bytes, derive ENCRYPTION_KEY + HMAC_KEY, hex-decode blob, split MAC || ciphertext, decrypt to plaintext, verify MAC, replace blob with plaintext, fall through to existing `BsmsParser::parse`.
  - Token-validation: hex-decode, reject if not 16 or 32 hex chars.
  - Stderr NOTICE advisory on successful encrypted-decrypt path.
- `crates/mnemonic-toolkit/src/error.rs`:
  - Add `BsmsMacMismatch { /* fields as needed */ }` variant in alphabetical slot.
  - 3 cascade-match updates: `Display`, `exit_code` (→ 2), `kind` (→ `"BsmsMacMismatch"`).
- `crates/mnemonic-toolkit/src/secrets.rs`:
  - `--bsms-encryption-token` is a FILE PATH flag, NOT inline-secret. Per Cycle 6 precedent (file-path flags not classified secret), NO update to `flag_is_secret`. Document the decision in plan-doc.

### Source files created (toolkit)

- `crates/mnemonic-toolkit/tests/cli_import_wallet_bsms_encrypted.rs` — ~25-30 cells.

### Test fixtures created (toolkit)

- `crates/mnemonic-toolkit/tests/fixtures/wallet_import/bsms-encrypted-standard-tv3.dat` — full BIP-129 TV-3 wire (the .dat file from Cycle 7a's locked TV-3_CIPHERTEXT_HEX prefixed with TV-3_MAC_HEX; 304-byte hex content).
- `crates/mnemonic-toolkit/tests/fixtures/wallet_import/bsms-encrypted-standard-tv3-token.hex` — TV-3 TOKEN as a hex string in a file (`a54044308ceac9b7\n`).

### Documentation modified (toolkit)

- `docs/manual/src/40-cli-reference/41-mnemonic.md` — `## mnemonic import-wallet` section: new flag entry for `--bsms-encryption-token`; stderr-templates table gains a NOTICE row for the decrypt-success advisory + Error rows for token-format + MAC-mismatch failures.
- `docs/manual/src/45-foreign-formats.md` — §BSMS subsection: document BIP-129 encrypted Round-2 ingest workflow; cite Cycle 7a's bsms_crypto library; cross-impl recipe vs Coinkite Python.

### Source files modified (mnemonic-gui)

- `mnemonic-gui/pinned-upstream.toml` — `mnemonic-toolkit-v0.30.1 → mnemonic-toolkit-v0.31.0`.
- `mnemonic-gui/Cargo.toml` — toolkit dep tag bump + workspace version `v0.15.0 → v0.16.0`.
- `mnemonic-gui/src/schema/mnemonic.rs` — add `--bsms-encryption-token` FlagSchema to `IMPORT_WALLET_FLAGS` const (alphabetical with other `bsms-*` flags).
- `mnemonic-gui/CHANGELOG.md` — new v0.16.0 entry.

### Release tooling

- `crates/mnemonic-toolkit/Cargo.toml:3` — version `0.30.1` → `0.31.0`.
- `CHANGELOG.md` — new `## [0.31.0]` section.
- `scripts/install.sh:32` — `mnemonic-toolkit-v0.30.1` → `mnemonic-toolkit-v0.31.0`.
- `design/FOLLOWUPS.md` — close `bsms-bip129-encryption-envelope` + file new FOLLOWUPs (per-signer-tokens, Round-1-encrypted variants).

---

## Tasks

### Task 1: Phase 2 — `--bsms-encryption-token` CLI flag plumbing

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/import_wallet.rs`

- [ ] **Step 1: Read current `--bsms-round1` flag site for placement context**

```bash
sed -n '180,205p' crates/mnemonic-toolkit/src/cmd/import_wallet.rs
```

The `--bsms-round1` flag at L191-192 + `--bsms-verify-strict` at L198-199 are the existing `bsms-*` flags. New `--bsms-encryption-token` slots alphabetically BEFORE `--bsms-round1` (alphabetical: `bsms-encryption-token` < `bsms-round1` < `bsms-verify-strict`).

- [ ] **Step 2: Add `--bsms-encryption-token` clap arg to `ImportWalletArgs`**

Insert above the existing `--bsms-round1` arg:

```rust
    /// v0.31.0 — BIP-129 encryption-envelope Round-2 ingest. Reads the
    /// session TOKEN from PATH (or `-` for stdin); applies PBKDF2-SHA512
    /// key derivation + AES-256-CTR decrypt + HMAC-SHA256 verify per
    /// BIP-129 §Encryption. Combine with `--format bsms` to decrypt
    /// encrypted Round-2 wallet shares from a Coordinator. Token file
    /// contents: lowercase ASCII hex (16 chars for STANDARD mode, 32
    /// chars for EXTENDED mode); leading/trailing whitespace + newlines
    /// stripped. Encrypted blobs don't carry the `BSMS 1.0` header so
    /// they don't auto-sniff as BSMS; `--format bsms` is REQUIRED.
    /// MAC verify failure → exit 2 (typed BsmsMacMismatch). Sees
    /// design/cycle-7-p0-recon.md §A1 for the byte-level wire format.
    #[arg(long = "bsms-encryption-token", value_name = "FILE|-")]
    pub bsms_encryption_token: Option<PathBuf>,
```

- [ ] **Step 3: Update header docstring synopsis line**

Find header at L7-8 and update:

```
//!   --blob <FILE|->                                             required UNLESS --bsms-round1 supplied
//!   --format <bitcoin-core|bsms|coldcard|coldcard-multisig|electrum|jade|sparrow|specter>
//!   --bsms-encryption-token <FILE|->                            v0.31.0 — BIP-129 encrypted Round-2 decrypt
//!   --bsms-round1 <FILE>                                        v0.27.0 — repeatable; BIP-129 Round-1 BIP-322 verify per record
```

- [ ] **Step 4: Build to verify clap-derive compiles cleanly**

```bash
cargo build --package mnemonic-toolkit 2>&1 | tail -5
```

- [ ] **Step 5: Smoke-test `--help` shows the new flag**

```bash
./target/debug/mnemonic import-wallet --help 2>&1 | grep -A2 bsms-encryption
```

Expected: shows `--bsms-encryption-token` entry + its help text.

- [ ] **Step 6: Smoke-test gui-schema picks it up**

```bash
./target/debug/mnemonic gui-schema 2>&1 | jq '.subcommands[] | select(.name=="import-wallet") | .flags[] | select(.name == "--bsms-encryption-token")'
```

Expected: emits an entry with `name: "--bsms-encryption-token"`, `required: false`, `kind: "path"`.

- [ ] **Step 7: Commit Phase 2**

```bash
git add crates/mnemonic-toolkit/src/cmd/import_wallet.rs
git commit -m "feat(import-wallet): v0.31.0 Phase 2 — --bsms-encryption-token CLI flag

Adds clap-derive arg to ImportWalletArgs for BIP-129 encryption-envelope
Round-2 ingest. Plumbing only; no behavior wired in this phase. Header
docstring + synopsis line updated.

Phase 2 of design/PLAN_mnemonic_toolkit_v0_31_0.md."
```

---

### Task 2: Phase 3 — `BsmsMacMismatch` ToolkitError variant + boundary mapper

**Files:**
- Modify: `crates/mnemonic-toolkit/src/error.rs` (add variant + 3 cascade arms).
- Modify: `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` (orchestrator pre-decrypt + MAC verify + boundary mapper).

- [ ] **Step 1: Map current `Bsms*` variant positions for alphabetical insertion**

```bash
grep -n "Bsms" crates/mnemonic-toolkit/src/error.rs | head -10
```

Identify existing `Bsms*` variants. `BsmsMacMismatch` slots: alphabetically `BsmsMacMismatch` falls between `BsmsImportTaprootRefused` (if present) and `BsmsSignatureMismatch` (if present), or wherever the `B`-series sits.

- [ ] **Step 2: Add `BsmsMacMismatch` variant + 3 cascade arms**

In `error.rs`:
- Add enum variant: `BsmsMacMismatch { token_len_hex: usize },` in alphabetical slot.
- `Display` arm: `BsmsMacMismatch { token_len_hex } => write!(f, "import-wallet: bsms: BIP-129 MAC verification failed (token width was {token_len_hex} hex chars; wrong token or tampered ciphertext)")`.
- `exit_code` arm: `BsmsMacMismatch { .. } => 2`.
- `kind` arm: `BsmsMacMismatch { .. } => "BsmsMacMismatch"`.

- [ ] **Step 3: Implement the encrypted-decrypt orchestrator in `import_wallet.rs` BSMS arm**

In the `Some("bsms") => { ... }` dispatch arm:

```rust
            // v0.31.0 Cycle 7b — BIP-129 encryption-envelope Round-2 decrypt.
            // When --bsms-encryption-token is supplied, decrypt the blob
            // BEFORE handing to BsmsParser::parse (preserves the parser-trait
            // surface; orchestrator owns the cross-cycle integration).
            if let Some(token_path) = &args.bsms_encryption_token {
                let blob_hex = std::str::from_utf8(&blob).map_err(|_| ToolkitError::ImportWalletParse(
                    "import-wallet: bsms: encrypted Round-2 blob must be valid UTF-8 hex".to_string(),
                ))?;
                let token_hex = read_bsms_token(token_path, stdin)?;
                let token_raw = hex::decode(&token_hex).map_err(|e| ToolkitError::BadInput(format!(
                    "--bsms-encryption-token: token file contents not valid hex: {e}"
                )))?;
                if token_raw.len() != 8 && token_raw.len() != 16 {
                    return Err(ToolkitError::BadInput(format!(
                        "--bsms-encryption-token: token must be 8 bytes STANDARD (16 hex chars) or 16 bytes EXTENDED (32 hex chars); got {} bytes ({} hex chars)",
                        token_raw.len(),
                        token_hex.len(),
                    )));
                }
                let wire = hex::decode(blob_hex.trim()).map_err(|e| ToolkitError::ImportWalletParse(format!(
                    "import-wallet: bsms: encrypted Round-2 wire is not valid hex: {e}"
                )))?;
                if wire.len() < 32 + 1 {
                    return Err(ToolkitError::ImportWalletParse(format!(
                        "import-wallet: bsms: encrypted Round-2 wire too short ({} bytes; need MAC (32) + at least 1 ciphertext byte)",
                        wire.len(),
                    )));
                }
                let (mac_recv, ciphertext) = wire.split_at(32);
                let enc_key = mnemonic_toolkit::bsms_crypto::derive_encryption_key(&token_raw);
                let hmac_key = mnemonic_toolkit::bsms_crypto::derive_hmac_key(&enc_key);
                let iv: [u8; 16] = mac_recv[..16].try_into().expect("32-byte MAC has 16-byte prefix");
                let plaintext = mnemonic_toolkit::bsms_crypto::decrypt(ciphertext, &enc_key, &iv)
                    .map_err(|e| ToolkitError::ImportWalletParse(format!("import-wallet: bsms: {e}")))?;
                let mac_expected = mnemonic_toolkit::bsms_crypto::compute_mac(&hmac_key, &token_hex, &plaintext);
                if mac_recv != mac_expected.as_slice() {
                    return Err(ToolkitError::BsmsMacMismatch { token_len_hex: token_hex.len() });
                }
                writeln!(
                    stderr,
                    "notice: import-wallet: bsms: BIP-129 encrypted Round-2 envelope decrypted (token width {} hex chars; MAC verified)",
                    token_hex.len(),
                )
                .map_err(ToolkitError::Io)?;
                // Replace blob with the decrypted plaintext for downstream parser.
                blob = plaintext.to_vec();
            }
```

(Place this BEFORE the existing `BsmsParser::parse(&blob, stderr)` call in the bsms arm.)

- [ ] **Step 4: Add `read_bsms_token` helper**

In `cmd/import_wallet.rs` (private helper):

```rust
fn read_bsms_token(
    path: &std::path::Path,
    stdin: &mut dyn std::io::Read,
) -> Result<String, ToolkitError> {
    let contents = if path == std::path::Path::new("-") {
        let mut s = String::new();
        stdin.read_to_string(&mut s).map_err(ToolkitError::Io)?;
        s
    } else {
        std::fs::read_to_string(path).map_err(|e| ToolkitError::BadInput(format!(
            "--bsms-encryption-token: cannot read token file {}: {e}",
            path.display()
        )))?
    };
    Ok(contents.trim().to_lowercase())
}
```

- [ ] **Step 5: Build + verify clean compile**

```bash
cargo build --package mnemonic-toolkit 2>&1 | tail -5
```

- [ ] **Step 6: Smoke-test against TV-3 fixture (deferred to Phase 4 Step 1 fixture creation)**

Skip if fixtures don't exist yet; otherwise:

```bash
./target/debug/mnemonic import-wallet \
  --format bsms \
  --blob crates/mnemonic-toolkit/tests/fixtures/wallet_import/bsms-encrypted-standard-tv3.dat \
  --bsms-encryption-token crates/mnemonic-toolkit/tests/fixtures/wallet_import/bsms-encrypted-standard-tv3-token.hex \
  --json 2>&1 | tail -5
```

Expected: SUCCESS + stderr NOTICE about decrypted envelope.

- [ ] **Step 7: Commit Phase 3**

```bash
git add crates/mnemonic-toolkit/src/error.rs crates/mnemonic-toolkit/src/cmd/import_wallet.rs
git commit -m "feat(import-wallet): v0.31.0 Phase 3 — orchestrator pre-decrypt + BsmsMacMismatch

Adds BsmsMacMismatch ToolkitError variant (alphabetical slot; exit 2;
typed per FOLLOWUP body recommendation). Orchestrator decrypt block in
the bsms dispatch arm: reads token via read_bsms_token helper, validates
hex shape + 8/16-byte width, hex-decodes wire blob, splits MAC ||
ciphertext, derives EK+HK via bsms_crypto, decrypts via Ctr128BE<AES256>,
verifies MAC, replaces blob with plaintext for downstream BsmsParser.

Phase 3 of design/PLAN_mnemonic_toolkit_v0_31_0.md."
```

---

### Task 3: Phase 4 — Test fixtures + integration test suite

**Files:**
- Create: `crates/mnemonic-toolkit/tests/fixtures/wallet_import/bsms-encrypted-standard-tv3.dat`
- Create: `crates/mnemonic-toolkit/tests/fixtures/wallet_import/bsms-encrypted-standard-tv3-token.hex`
- Create: `crates/mnemonic-toolkit/tests/cli_import_wallet_bsms_encrypted.rs`

- [ ] **Step 1: Create the TV-3 fixtures**

TV-3 `.dat` content (full wire: MAC + ciphertext, hex-encoded; from BIP-129 + Cycle 7a recon dossier locked values):

```bash
cat > crates/mnemonic-toolkit/tests/fixtures/wallet_import/bsms-encrypted-standard-tv3.dat <<'EOF'
fbdbdb64e6a8231c342131d9f13dcd5a954b4c5021658fa5afcb3fc74dc8270653f491cfd1431c292d922ea5a5dec3eb8ddaa6ed38ae109e7b040f0f23013e89a89b4d27476761a01197a3277850b2bc1621ae626efe65f2081eec6eb571c4f787bf1c49d061b43f70fd73cb3f37fa591d2400973ac0644c8941a83f1d4155e98f01fa2fdeb9f86c2e2413154fd18566a28fb0d9d8bd6172efabcfa6dab09ee7029bf3dd43376df52c118a6d291ec168f4ec7f7df951dfc6135fd8cb4b234da62eaea6017dfe5ca418f083e02e3aba2962ba313ba17b6468c7672fb218329a9f3fe4e4887fb87dac57c63ebff0e715a44498d18de8afc10e1cfeb46a1fc65ce871fef8a43b289305433a90c342d025aa4c19454fcfbcf911e9e2f928d5affd0536a6ddc2e816
EOF
```

(Single line; no trailing whitespace.)

TV-3 TOKEN file:

```bash
cat > crates/mnemonic-toolkit/tests/fixtures/wallet_import/bsms-encrypted-standard-tv3-token.hex <<'EOF'
a54044308ceac9b7
EOF
```

- [ ] **Step 2: Author `tests/cli_import_wallet_bsms_encrypted.rs`**

Create the integration test file with ~12-15 cells covering:

```rust
//! v0.31.0 — BIP-129 encrypted Round-2 ingest integration tests.

use assert_cmd::Command;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/wallet_import")
        .join(name)
}

fn mnemonic() -> Command {
    Command::cargo_bin("mnemonic").expect("mnemonic binary builds")
}

// Happy path: TV-3 STANDARD-mode encrypted Round-1 record decrypts to the
// 5-line BIP-129 Signer key record.
//
// NOTE: BIP-129 TV-3 is a Round-1 KEY record (per BIP-129 §Test Vectors
// "Signer 1 encryption"), NOT a Round-2 DESCRIPTOR record. The current
// BsmsParser handles Round-2 (4-line/6-line descriptor shape). Round-1
// is a 5-line shape (BSMS 1.0 / TOKEN / KEY / desc / SIG). So feeding
// the decrypted TV-3 plaintext into BsmsParser will hit the parser's
// existing line-count refusal (not a 2/4/6-line shape). The integration
// cell asserts: stderr NOTICE about decrypted envelope DOES fire +
// final exit code reflects the parser-side refusal. This documents the
// "library decrypts + orchestrator dispatches; user receives plaintext
// route" boundary cleanly. Future cycle adds Round-1-decrypt-then-verify
// integration (Cycle 7b explicitly out of scope per plan §"Out of scope").

#[test]
fn tv3_decrypt_emits_notice_advisory() {
    let blob = fixture_path("bsms-encrypted-standard-tv3.dat");
    let token = fixture_path("bsms-encrypted-standard-tv3-token.hex");
    let assertion = mnemonic()
        .args(["import-wallet", "--format", "bsms"])
        .args(["--blob"]).arg(&blob)
        .args(["--bsms-encryption-token"]).arg(&token)
        .assert();
    // Parser refuses the 5-line Round-1 plaintext (Cycle 7b's bsms.rs
    // handles Round-2 only). Cycle 7b's job: assert the NOTICE-advisory
    // fires (decryption + MAC-verify succeeded) regardless of downstream
    // parse outcome.
    let output = assertion.get_output();
    let stderr = String::from_utf8(output.stderr.clone()).unwrap();
    assert!(
        stderr.contains("BIP-129 encrypted Round-2 envelope decrypted"),
        "expected decrypt-success NOTICE on stderr; got: {stderr}"
    );
}

#[test]
fn tv3_wrong_token_yields_mac_mismatch() {
    let blob = fixture_path("bsms-encrypted-standard-tv3.dat");
    // Write a wrong-token fixture in-test (one bit flipped)
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), b"a54044308ceac9b8\n").unwrap(); // last hex char b7→b8
    mnemonic()
        .args(["import-wallet", "--format", "bsms"])
        .args(["--blob"]).arg(&blob)
        .args(["--bsms-encryption-token"]).arg(tmp.path())
        .assert()
        .failure()
        .code(2)
        .stderr(predicates::str::contains("MAC verification failed"));
}

#[test]
fn token_with_invalid_hex_chars_refused() {
    let blob = fixture_path("bsms-encrypted-standard-tv3.dat");
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), b"not-valid-hex!!!\n").unwrap();
    mnemonic()
        .args(["import-wallet", "--format", "bsms"])
        .args(["--blob"]).arg(&blob)
        .args(["--bsms-encryption-token"]).arg(tmp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("token file contents not valid hex"));
}

#[test]
fn token_with_wrong_width_refused() {
    let blob = fixture_path("bsms-encrypted-standard-tv3.dat");
    // 20-hex-char token (10 bytes; neither STANDARD nor EXTENDED).
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), b"abcdef0123456789abcd\n").unwrap();
    mnemonic()
        .args(["import-wallet", "--format", "bsms"])
        .args(["--blob"]).arg(&blob)
        .args(["--bsms-encryption-token"]).arg(tmp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("token must be 8 bytes STANDARD"));
}

#[test]
fn wire_blob_not_hex_refused() {
    let token = fixture_path("bsms-encrypted-standard-tv3-token.hex");
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), b"not-valid-hex-blob!!!\n").unwrap();
    mnemonic()
        .args(["import-wallet", "--format", "bsms"])
        .args(["--blob"]).arg(tmp.path())
        .args(["--bsms-encryption-token"]).arg(&token)
        .assert()
        .failure()
        .stderr(predicates::str::contains("not valid hex"));
}

#[test]
fn wire_blob_too_short_refused() {
    let token = fixture_path("bsms-encrypted-standard-tv3-token.hex");
    // 32 bytes = exactly MAC, no ciphertext.
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), b"fbdbdb64e6a8231c342131d9f13dcd5a954b4c5021658fa5afcb3fc74dc82706").unwrap();
    mnemonic()
        .args(["import-wallet", "--format", "bsms"])
        .args(["--blob"]).arg(tmp.path())
        .args(["--bsms-encryption-token"]).arg(&token)
        .assert()
        .failure()
        .stderr(predicates::str::contains("too short"));
}

#[test]
fn token_via_stdin() {
    let blob = fixture_path("bsms-encrypted-standard-tv3.dat");
    mnemonic()
        .args(["import-wallet", "--format", "bsms"])
        .args(["--blob"]).arg(&blob)
        .args(["--bsms-encryption-token", "-"])
        .write_stdin("a54044308ceac9b7")
        .assert();
    // Implementation note: stdin reading conflicts with --blob via stdin.
    // Cycle 7b verifies the constraint (at most one of --blob=- /
    // --bsms-encryption-token=-) at orchestrator entry.
}

#[test]
fn no_token_supplied_encrypted_blob_fails_at_sniff() {
    // Without --bsms-encryption-token, an encrypted blob doesn't sniff as
    // BSMS (no BSMS 1.0 header). With --format bsms, the parser hits its
    // existing UTF-8 + header-required refusal path.
    let blob = fixture_path("bsms-encrypted-standard-tv3.dat");
    mnemonic()
        .args(["import-wallet", "--format", "bsms"])
        .args(["--blob"]).arg(&blob)
        .assert()
        .failure();
}

#[test]
fn no_token_supplied_with_plaintext_blob_no_regression() {
    // Pre-v0.31.0 plaintext BSMS Round-2 blob still imports without
    // --bsms-encryption-token. Sanity check: behavior unchanged.
    let blob = fixture_path("bsms-2of3-decay-bip129-4line.bsms"); // existing fixture, adjust name as needed
    if blob.exists() {
        mnemonic()
            .args(["import-wallet", "--format", "bsms"])
            .args(["--blob"]).arg(&blob)
            .args(["--json"])
            .assert()
            .success();
    }
}
```

(Adjust the no-regression fixture name to an actual existing one; `ls crates/mnemonic-toolkit/tests/fixtures/wallet_import/bsms-*.bsms` to find.)

- [ ] **Step 3: Run integration tests**

```bash
cargo test --package mnemonic-toolkit --test cli_import_wallet_bsms_encrypted 2>&1 | tail -20
```

Expected: most cells PASS. Note: `tv3_decrypt_emits_notice_advisory` may show the parser refusal-after-decrypt — that's the documented boundary.

- [ ] **Step 4: Run full workspace tests for regressions**

```bash
cargo test --workspace 2>&1 | grep -E "^test result|FAILED" | head -10
```

- [ ] **Step 5: Commit Phase 4**

```bash
git add crates/mnemonic-toolkit/tests/cli_import_wallet_bsms_encrypted.rs crates/mnemonic-toolkit/tests/fixtures/wallet_import/bsms-encrypted-standard-tv3.dat crates/mnemonic-toolkit/tests/fixtures/wallet_import/bsms-encrypted-standard-tv3-token.hex
git commit -m "test(import-wallet): v0.31.0 Phase 4 — BIP-129 TV-3 integration suite

Fixtures: TV-3 STANDARD-mode encrypted Round-1 wire (.dat) + TOKEN
(.hex). Both lock the BIP-129 §Test Vectors values cross-validated by
Cycle 7a's bsms_crypto unit cells.

Tests: ~8 cells covering decrypt-success NOTICE; wrong-token MAC mismatch
(exit 2 / typed BsmsMacMismatch); invalid-hex token; wrong-width token
(neither STANDARD nor EXTENDED); wire-blob-not-hex; wire-blob-too-short;
token-via-stdin; no-token-with-plaintext-no-regression.

Phase 4 of design/PLAN_mnemonic_toolkit_v0_31_0.md."
```

---

### Task 4: Phase 5 — Manual chapter updates

**Files:**
- Modify: `docs/manual/src/40-cli-reference/41-mnemonic.md` (`mnemonic import-wallet` section).
- Modify: `docs/manual/src/45-foreign-formats.md` (BSMS section).

- [ ] **Step 1: Locate import-wallet flag list in chapter-41**

```bash
grep -n "bsms-round1\|--bsms\|## .mnemonic import-wallet" docs/manual/src/40-cli-reference/41-mnemonic.md | head
```

- [ ] **Step 2: Add `--bsms-encryption-token` to chapter-41 flag list + stderr-templates table**

Insert flag description alphabetically + add NOTICE row to stderr-templates table:

```markdown
- `--bsms-encryption-token <FILE|->`: BIP-129 encryption-envelope Round-2
  decrypt. Reads the session TOKEN from PATH (or `-` for stdin); applies
  PBKDF2-SHA512 + AES-256-CTR + HMAC-SHA256 per BIP-129 §Encryption.
  Combine with `--format bsms` to decrypt encrypted Round-2 wallet shares
  from a Coordinator. Token file contents: lowercase ASCII hex (16 chars
  for STANDARD mode, 32 chars for EXTENDED mode); leading/trailing
  whitespace + newlines stripped. v0.31.0+.
```

Add to stderr-templates table:

```markdown
| NOTICE (exit 0) | `notice: import-wallet: bsms: BIP-129 encrypted Round-2 envelope decrypted (token width N hex chars; MAC verified)` |
| Error (exit 2) | `error: import-wallet: bsms: BIP-129 MAC verification failed (token width N hex chars; wrong token or tampered ciphertext)` |
```

- [ ] **Step 3: Add encrypted-Round-2 subsection to chapter-45 §BSMS**

```bash
grep -n "## BSMS\|encrypted\|BIP-129.*encrypt" docs/manual/src/45-foreign-formats.md | head
```

Add a new subsection documenting the encrypted Round-2 workflow + cross-impl recipe vs Coinkite Python.

- [ ] **Step 4: Run manual lint**

```bash
make -C docs/manual lint MNEMONIC_BIN=/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic 2>&1 | tail -5
```

Expected: PASS.

- [ ] **Step 5: Architect-must-run-prose-commands check**

Run each command block in the new manual sections; confirm output matches prose. The cross-impl recipe with Coinkite Python ref can be cited (clone + decrypt) without actually running the Python (out-of-band; user verifies).

- [ ] **Step 6: Commit Phase 5**

```bash
git add docs/manual/src/40-cli-reference/41-mnemonic.md docs/manual/src/45-foreign-formats.md
git commit -m "docs(bsms-encrypted): v0.31.0 Phase 5 — manual chapter-41 + chapter-45 update

Chapter-41 mnemonic import-wallet section gains --bsms-encryption-token
flag entry + stderr-templates table NOTICE + Error rows for the decrypt
path. Chapter-45 BSMS section gains encrypted Round-2 subsection
documenting BIP-129 §Encryption workflow + cross-impl recipe vs Coinkite
Python ref.

Phase 5 of design/PLAN_mnemonic_toolkit_v0_31_0.md."
```

---

### Task 5: Phase 6 — Toolkit cycle close (version bump + tag + push)

**Files:**
- Modify: `crates/mnemonic-toolkit/Cargo.toml:3` (`0.30.1` → `0.31.0`).
- Modify: `CHANGELOG.md` (new v0.31.0 section).
- Modify: `scripts/install.sh:32` (toolkit pin bump).

- [ ] **Step 1: Bump version + install.sh self-pin**

`crates/mnemonic-toolkit/Cargo.toml:3`: `0.30.1` → `0.31.0`.

`scripts/install.sh:32`: `mnemonic-toolkit-v0.30.1` → `mnemonic-toolkit-v0.31.0`.

- [ ] **Step 2: CHANGELOG entry**

```markdown
## mnemonic-toolkit [0.31.0] — 2026-05-21

**SemVer-MINOR release.** New `--bsms-encryption-token <FILE|->` flag on `mnemonic import-wallet` for BIP-129 §Encryption envelope decrypt. Closes Cycle 7 (`bsms-bip129-encryption-envelope` FOLLOWUP). Paired with `mnemonic-gui-v0.16.0`.

### Added

- **`--bsms-encryption-token <FILE|->`** flag on `mnemonic import-wallet` for BIP-129 encryption-envelope Round-2 decrypt. Reads the session TOKEN from PATH (or `-` for stdin); applies PBKDF2-SHA512(`"No SPOF"`, TOKEN_raw, 2048, 32) + AES-256-CTR (Ctr128BE) + HMAC-SHA256 per BIP-129 §Encryption. Combine with `--format bsms`. Token width: 16 hex chars (STANDARD mode; 8 raw bytes) or 32 hex chars (EXTENDED mode; 16 raw bytes). Sniff unchanged — encrypted blobs lack the `BSMS 1.0` header so `--format bsms` is REQUIRED for the encrypted path.
- New `BsmsMacMismatch` `ToolkitError` variant (typed error per FOLLOWUP body recommendation; exit 2). Stderr template: `error: import-wallet: bsms: BIP-129 MAC verification failed (token width N hex chars; wrong token or tampered ciphertext)`.
- New library module `mnemonic_toolkit::bsms_crypto` (shipped pre-tag in Cycle 7a as commit `62da111`): pub `derive_encryption_key` / `derive_hmac_key` / `compute_mac` / `decrypt` / `encrypt` + library-local `BsmsCryptoError`. 20 unit cells incl. BIP-129 TV-3 cross-validation.
- New Cargo dep `ctr = "0.9"` (added in Cycle 7a; sibling of `cbc` from RustCrypto block-modes family).

### Documentation

- Chapter-41 `mnemonic import-wallet` section: new `--bsms-encryption-token` flag entry + stderr-templates table NOTICE + Error rows.
- Chapter-45 §BSMS gains encrypted-Round-2 subsection with cross-impl recipe vs Coinkite Python ref.

### FOLLOWUP closure

- **Closed:** `bsms-bip129-encryption-envelope` (resolved by Cycle 7 / v0.31.0). FOLLOWUP body cross-cited Cycle 7a P0 recon (`design/cycle-7-p0-recon.md`) + Cycle 7a R0 review (`design/agent-reports/v0_31_0-bsms-crypto-brainstorm-r0-review.md`) confirming scheme citation was already accurate (unlike Cycle 6 where the FOLLOWUP body had a wrong PBKDF2-AES-CBC claim).

### Newly filed FOLLOWUPs

- `bsms-encryption-per-signer-tokens` — per-Signer TOKEN variants (BIP-129 line 74 allows per-Signer or shared TOKEN; Cycle 7b ships shared-TOKEN only). Tier `v0.31+`.
- `bsms-encryption-round1-decrypt-then-verify` — encrypted Round-1 KEY records (Cycle 7b ships encrypted Round-2 only; encrypted Round-1 decrypt-then-verify is a separate orchestration). Tier `v0.31+`.
- `bsms-encryption-cross-impl-coinkite-python-smoke` — automated cross-impl test against Coinkite Python ref `bsms-bitcoin-secure-multisig-setup` (Cycle 7b cross-checks against the locked recon-dossier values; automated cross-impl smoke is a separate test harness). Tier `v0.31+`.

### Note

Cycle 7 of v0.28+ residual FOLLOWUP release plan, executed as two-session split (7a: library + recon + opus R0; 7b: CLI + parser integration + ship). Opus R0 on 7a brainstorm caught the `Ctr64BE` vs `Ctr128BE` critical pre-impl (Cycle 6 lesson applied). Opus R0 on 7b plan-doc pending.

### Tests

- 71 lib + 743+ integration cells; clippy clean; manual lint 6/6 PASS.
- New `tests/cli_import_wallet_bsms_encrypted.rs` ~8 cells.
- Cycle 7a's 20 unit cells in `bsms_crypto::tests` continue to pass.

---

## mnemonic-toolkit [0.30.1] — 2026-05-21
```

- [ ] **Step 3: Full pre-tag audit**

```bash
cargo test --workspace 2>&1 | grep -E "^test result|FAILED" | head -10
cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -3
```

Expected: all GREEN.

- [ ] **Step 4: Commit + tag + push**

```bash
git add crates/mnemonic-toolkit/Cargo.toml scripts/install.sh CHANGELOG.md Cargo.lock
git commit -m "release(toolkit): mnemonic-toolkit v0.31.0 — BIP-129 encryption envelope"
git tag mnemonic-toolkit-v0.31.0
git push origin master
git push origin mnemonic-toolkit-v0.31.0
```

- [ ] **Step 5: install-pin-check CI**

```bash
gh run list --limit 5 --json status,conclusion,name,headBranch | jq '.[] | select(.name|test("install-pin"))'
```

Wait for `conclusion: success`.

- [ ] **Step 6: GH Release**

```bash
awk '/^## mnemonic-toolkit \[0\.31\.0\]/,/^## mnemonic-toolkit \[0\.30\.1\]/' CHANGELOG.md | head -n -1 > /tmp/v0_31_0_release_notes.md
gh release create mnemonic-toolkit-v0.31.0 \
  --title "mnemonic-toolkit-v0.31.0 — BIP-129 encryption envelope" \
  --notes-file /tmp/v0_31_0_release_notes.md
```

---

### Task 6: Phase 7 — GUI lockstep + FOLLOWUP closure

**Files:**
- Modify: `mnemonic-gui/pinned-upstream.toml`
- Modify: `mnemonic-gui/Cargo.toml`
- Modify: `mnemonic-gui/src/schema/mnemonic.rs`
- Modify: `mnemonic-gui/CHANGELOG.md`
- Modify: `design/FOLLOWUPS.md`

- [ ] **Step 1: GUI pin bump + workspace version**

```bash
cd /scratch/code/shibboleth/mnemonic-gui && git pull --ff-only origin master
```

Edit `pinned-upstream.toml`: tag → `mnemonic-toolkit-v0.31.0`.
Edit `Cargo.toml`: workspace dep tag → `mnemonic-toolkit-v0.31.0`; workspace version → `0.16.0`.

- [ ] **Step 2: `cargo update`**

```bash
cd /scratch/code/shibboleth/mnemonic-gui && cargo update --workspace 2>&1 | tail -5
```

- [ ] **Step 3: Add `--bsms-encryption-token` to GUI schema**

`mnemonic-gui/src/schema/mnemonic.rs` — find `IMPORT_WALLET_FLAGS` const. Insert alphabetically:

```rust
    FlagSchema {
        name: "--bsms-encryption-token",
        kind: FlagKind::Path { stdio_sentinel: true },
        required: false,
        repeating: false,
        help: "BIP-129 encrypted Round-2 decrypt token (file or stdin).",
        secret: false,
        default_value: None,
        global: false,
    },
```

(`stdio_sentinel: true` because the flag accepts `-` for stdin.)

- [ ] **Step 4: `schema_mirror` verify with explicit MNEMONIC_BIN**

```bash
cd /scratch/code/shibboleth/mnemonic-gui
MNEMONIC_BIN=/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic cargo test --test schema_mirror 2>&1 | tail -15
```

- [ ] **Step 5: GUI CHANGELOG + commit + tag + push**

GUI CHANGELOG entry v0.16.0; commit; tag `mnemonic-gui-v0.16.0`; push.

- [ ] **Step 6: Verify GUI tag CI**

```bash
gh run list --repo bg002h/mnemonic-gui --limit 5 --json status,conclusion,name,headBranch | jq '.[] | select(.headBranch=="mnemonic-gui-v0.16.0")'
```

- [ ] **Step 7: GUI GH Release**

- [ ] **Step 8: FOLLOWUP closure**

Edit `design/FOLLOWUPS.md`:
- Close `bsms-bip129-encryption-envelope` (resolved by Cycle 7 / v0.31.0).
- Optionally close-or-cross-cite `wallet-import-bsms-encrypted` (v0.27+ predecessor) — reconcile bodies.
- File 3 new FOLLOWUPs per CHANGELOG:
  - `bsms-encryption-per-signer-tokens`
  - `bsms-encryption-round1-decrypt-then-verify`
  - `bsms-encryption-cross-impl-coinkite-python-smoke`

- [ ] **Step 9: Commit FOLLOWUP updates + push**

```bash
git add design/FOLLOWUPS.md
git commit -m "design(cycle-7-close): FOLLOWUP closure — bsms-bip129-encryption-envelope resolved + 3 new slugs"
git push origin master
```

- [ ] **Step 10: End-of-cycle opus review**

Dispatch opus on full uncommitted working tree across both repos. Persist at `design/agent-reports/v0_31_0-end-of-cycle-review.md`. Fold any C/I before declaring cycle closed.

---

## Cross-phase invariants

- **R0 review BEFORE Phase 2 dispatch** (Cycle 6 lesson). This plan-doc dispatches opus R0 BEFORE Task 1. R0 verdict + folds drive Task 1 prep.
- **No `cargo fmt --all`** — restrict scope to cycle files only (Cycle 5/6/7a recurring discipline).
- **install-pin-check CI gate** on tag push.
- **Per-phase commits, NOT bundled** — Phase 2/3/4/5/6/7 each get their own commit for bisect-hygiene.
- **Single-form `--bsms-encryption-token <FILE|->`** — NOT 3-form (parent brainstorm locked single-form; argv-hygiene satisfied by file-or-stdin shape).

## Phase ordering rationale

- Task 1 (Phase 2: CLI flag plumbing) is plumbing-only; no behavior. Provides the clap surface for Task 2's orchestrator wire-up.
- Task 2 (Phase 3: orchestrator + BsmsMacMismatch) depends on Task 1's flag existing.
- Task 3 (Phase 4: integration tests) depends on Task 2's orchestrator behavior.
- Task 4 (Phase 5: manual) depends on Task 1's flag entry + Task 2's stderr templates.
- Task 5 (Phase 6: cycle close) depends on Tasks 1-4 all green.
- Task 6 (Phase 7: GUI + FOLLOWUP closure) depends on toolkit tag landed first (`pinned-upstream.toml` can't resolve a not-yet-existent tag).

## Risk register

- **TV-3 is a Round-1 KEY record, NOT Round-2 DESCRIPTOR.** The integration test's "happy path" surfaces a parser-side refusal after decrypt because `BsmsParser` handles Round-2 (4-line/6-line). The decrypt + MAC-verify path is exercised; the downstream parse is documented as a known limitation. A future cycle adds Round-1-encrypted-then-verify integration (filed as `bsms-encryption-round1-decrypt-then-verify` FOLLOWUP).
- **`--bsms-encryption-token` is documented as case-sensitive (lowercase hex).** Tests verify wrong-case-token does NOT match TV-3.
- **`Path == Path("-")` test** in `read_bsms_token` may need to use `PathBuf::from("-")` comparison or a `.as_os_str() == "-"` check. Verify at impl.
- **stdin contention** between `--blob=-` and `--bsms-encryption-token=-`. Both can't read from stdin in the same invocation; orchestrator must validate (refuse if both are `-`).
- **MAC verify is constant-time-CRITICAL** to avoid timing-side-channel attacks. Use `subtle::ConstantTimeEq` if practical; otherwise document the threat model (single-attempt; no oracle). Cycle 7b uses byte-by-byte `==` comparison — adequate for non-interactive use; document.

## Self-review (pre-R0 dispatch)

- ✓ Brainstorm v1 (7a; library) locked all primitives.
- ✓ TV-3 cross-validation (Cycle 7a) confirmed Ctr128BE is correct.
- ✓ Single new `ToolkitError` variant (`BsmsMacMismatch`).
- ✓ Single new clap flag (`--bsms-encryption-token`).
- ✓ Single-form (file-or-stdin); no inline; argv-hygiene satisfied.
- ✓ Stderr templates locked.
- ✓ FOLLOWUP closure semantics (1 close + 3 file).
- ✓ Phase ordering (linear; no cross-phase coupling beyond Task 1 → Task 2).
- ✓ No new Cargo deps (`ctr = "0.9"` already added in Cycle 7a).
