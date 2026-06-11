# Zeroize-discipline audit — 19 unrepresented secret-bearing files (2026-06-11)

Subagent audit feeding `SPEC_zeroize_lint_completeness.md`. Of 35 src files using `Zeroizing::new(`/`SecretString::new(`/`: Zeroizing<`/`: SecretString`, 16 are canonical ZEROIZE_ROWS; these 19 are not. Classification (paths relative to `crates/mnemonic-toolkit/`):

## CANONICAL — 14 files → 16 proposed rows (promote)
- `cmd/addresses.rs` — master-secret entropy. Evidence: `let entropy: zeroize::Zeroizing<Vec<u8>> = zeroize::Zeroizing::new(match from.node {`
- `cmd/electrum_decrypt.rs` — decrypt password. Evidence: `let password: zeroize::Zeroizing<String> = if let Some(pw) = &args.decrypt_password {`
- `cmd/import_wallet.rs` (ROW 1) wallet blob — evidence: `-> Result<Zeroizing<Vec<u8>>, ToolkitError> {` + `Zeroizing::new(fs::read(path).map_err(ToolkitError::Io)?)`
- `cmd/import_wallet.rs` (ROW 2) decrypt pw + BSMS records — evidence: `-> Result<Option<Zeroizing<String>>, ToolkitError> {` + `Ok(Some(Zeroizing::new(pw.clone())))`
- `cmd/ms_shares.rs` (ROW 1) parse_secret_to_entropy — evidence: `) -> Result<zeroize::Zeroizing<Vec<u8>>, ToolkitError> {` + `Ok(zeroize::Zeroizing::new(m.to_entropy()))`
- `cmd/ms_shares.rs` (ROW 2) combine recover+output — evidence: `(zeroize::Zeroizing::new(entropy.clone()), lang, cli)` + `let output: zeroize::Zeroizing<String> = match args.to {`
- `cmd/restore.rs` — seed entropy (run + resolve_seed_entropy). Evidence: `) -> Result<(zeroize::Zeroizing<Vec<u8>>, bip39::Language), ToolkitError> {` + `let (entropy, derive_language): (zeroize::Zeroizing<Vec<u8>>, bip39::Language) = match from.node`
- `cmd/seedqr.rs` — digits + phrase. Evidence: `let phrase: zeroize::Zeroizing<String> = zeroize::Zeroizing::new(phrase_plain);` + `let digits: zeroize::Zeroizing<String> = zeroize::Zeroizing::new(digits_plain);`
- `cmd/verify_bundle.rs` (ROW 1) Phrase arm — evidence: `zeroize::Zeroizing::new(args.passphrase.clone().unwrap_or_default());` + `let entropy = zeroize::Zeroizing::new(mnemonic.to_entropy());`
- `cmd/verify_bundle.rs` (ROW 2) Entropy arm — evidence: `let entropy_bytes = zeroize::Zeroizing::new(hex::decode(entropy_hex)`
- `cmd/xpub_search/account_of_descriptor.rs` — passphrase. Evidence: `let passphrase: zeroize::Zeroizing<String> = if args.passphrase_stdin {`
- `cmd/xpub_search/passphrase_of_xpub.rs` — mandatory passphrase. Evidence: `let passphrase: zeroize::Zeroizing<String> = if args.passphrase_stdin {`
- `cmd/xpub_search/path_of_xpub.rs` — passphrase. Evidence: `let passphrase: zeroize::Zeroizing<String> = if args.passphrase_stdin {`
- `cmd/xpub_search/seed_intake.rs` — phrase/ms1 source + decoded entropy. Evidence: `Phrase(Zeroizing<String>),` + `let entropy: Zeroizing<Vec<u8>> = Zeroizing::new(payload.as_bytes().to_vec());`
- `seed_xor.rs` (LIBRARY, ≠ cmd/seed_xor.rs) — shares + recovered master. Evidence: `) -> Result<Vec<zeroize::Zeroizing<Vec<u8>>>, SeedXorError> {` + `) -> Result<zeroize::Zeroizing<Vec<u8>>, SeedXorError> {`
- `slot_ms1.rs` — Ms1SlotResolution.entropy field. Evidence: `pub entropy: Zeroizing<Vec<u8>>,` + `entropy: Zeroizing::new(bytes),`
- `wallet_import/overlay.rs` — cosigner entropy. Evidence: `let entropy_and_lang: (Zeroizing<Vec<u8>>, bip39::Language) = match src {` + `(Zeroizing::new(mnemonic.to_entropy()), language.into())`

(NOTE: `import_wallet.rs`, `ms_shares.rs`, `verify_bundle.rs` get 2 rows each → 14 files = 16 rows + the rest 1 each. `verify_bundle.rs` + `ms_shares.rs` CONFIRMED canonical — they own passphrase/entropy.)

## CRYPTO-INTERNAL — 3 files (allowlist)
- `bsms_crypto.rs` — PBKDF2 AES key + AES-CTR plaintext buffer (consumer import_wallet owns plaintext).
- `electrum_crypto.rs` — ECIES/CBC primitive (AES key, scalar, ECDH shared secret, key block).
- `slip39/feistel.rs` — SLIP-0039 Feistel L/R halves + round key (consumer slip39/mod.rs owns output).

## PASS-THROUGH — 1 file (allowlist)
- `nostr.rs` — `decode_nostr_key` decodes an INPUT key, hands it upstream; `cmd/nostr.rs` (2 rows) owns the derived secret.

## PRIMITIVE/N-A — 1 file
- `secret_string.rs` — the `SecretString` newtype DEFINITION, not an allocation site.

## Bonus finding (separate FOLLOWUP)
`cmd/addresses.rs` + `cmd/restore.rs` hold the BIP-39 passphrase as a PLAIN `String` (only the entropy is `Zeroizing`). A real (small) secret-hygiene gap → FOLLOWUP `addresses-restore-passphrase-not-zeroizing`.
