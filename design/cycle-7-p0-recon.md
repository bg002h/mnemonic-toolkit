# Cycle 7 — P0 STRICT-GATE recon dossier (7a + 7b)

**Date:** 2026-05-21
**Cycle target:** Cycle 7a = `bsms_crypto.rs` library (PRE-TAG; this session). Cycle 7b = `--bsms-encryption-token` CLI + parser integration + ship (next session).
**Source SHA at recon time:** master HEAD `b03cac3` (post-Cycle-6b ship; in sync with origin/master).
**Predecessor recon (verified-still-current):** `design/agent-reports/v0_27_0-phase-2-bip129-recon.md` — comprehensive BIP-129 byte-level construction + all 4 test vectors. Cycle 7 recon is an INCREMENTAL re-verification + supplement, not a re-derivation.

## A1 — BIP-129 encryption-scheme citations (primary source: `bitcoin/bips` master)

Verified against `gh api repos/bitcoin/bips/contents/bip-0129.mediawiki` (2026-05-21):

| Element | Primary-source citation | Concrete value |
|---|---|---|
| PBKDF2 PRF | bip-0129.mediawiki line 142 | `SHA512` |
| PBKDF2 password | bip-0129.mediawiki line 143 | `"No SPOF"` (literal 7-byte ASCII) |
| PBKDF2 salt | bip-0129.mediawiki line 144 | `TOKEN` (raw bytes; see A2 below) |
| PBKDF2 iterations | bip-0129.mediawiki line 145 | `c = 2048` |
| PBKDF2 output length | bip-0129.mediawiki line 146 | `dkLen = 256 bits = 32 bytes` |
| Encryption | bip-0129.mediawiki line 150 | `AES-256-CTR` (RFC 3686) |
| MAC | bip-0129.mediawiki line 152 | `HMAC-SHA256(HMAC_Key, hex-encoded TOKEN || Data)` |
| HMAC_Key | bip-0129.mediawiki line 162 | `SHA256(ENCRYPTION_KEY)` (single SHA-256, NOT sha256d) |
| IV | bip-0129.mediawiki line 154 | first 16 bytes of `MAC` |
| Wire | bip-0129.mediawiki lines 84-85 | `hex(MAC || ciphertext)` — concatenated, hex-encoded |
| AE ordering | bip-0129.mediawiki line 165 | Encrypt-and-MAC (MAC is over PLAINTEXT, not ciphertext) |

**Verbatim quote (line 165):** *"Because it is a MAC over the entire plaintext, this is essentially an Encrypt-and-MAC form of authenticated encryption."*

## A2 — Critical asymmetry: PBKDF2 salt vs HMAC input

Per the v0.27.0 dossier (§"Critical asymmetry the impl must respect") + re-verified against BIP-129 + Coinkite Python ref:

- **PBKDF2 `salt = TOKEN`** uses RAW bytes (unhex of line-2 hex string).
- **MAC `hex-encoded TOKEN || Data`** uses ASCII-hex bytes (literally the hex string).

Concrete example with TV-3 (TOKEN hex = `a54044308ceac9b7`):
- PBKDF2 salt: `[0xa5, 0x40, 0x44, 0x30, 0x8c, 0xea, 0xc9, 0xb7]` (8 bytes).
- HMAC input prefix: `[0x61, 0x35, 0x34, 0x30, 0x34, 0x34, 0x33, 0x30, 0x38, 0x63, 0x65, 0x61, 0x63, 0x39, 0x62, 0x37]` (16 bytes — ASCII chars `a 5 4 0 4 4 3 0 8 c e a c 9 b 7`).

**Foot-gun:** any impl that conflates the two will fail TV-3 cross-validation.

## A3 — TOKEN modes (BIP-129 lines 123-125; verbatim)

- `NO_ENCRYPTION` → TOKEN = `0x00` (one byte). Hex-form on Round-1 line 2: `00` (one hex char, BUT current Round-2 plaintext shape uses `BSMS 1.0\n` header without TOKEN-on-line-2; encryption modes attach the wire `(MAC||ciphertext)` shape elsewhere).
- `STANDARD` → TOKEN = 64-bit nonce (8 raw bytes; 16 hex chars).
- `EXTENDED` → TOKEN = 128-bit nonce (16 raw bytes; 32 hex chars).

## A4 — Test vectors (verbatim values from BIP-129 §Test Vectors + Coinkite Python)

Inherited from v0.27.0 recon dossier with no decay:

### TV-3 (STANDARD; Signer 1 of `bip-0129.mediawiki` lines 301-315)

- TOKEN (hex on Round-1 line 2): `a54044308ceac9b7`
- TOKEN (raw bytes for PBKDF2 salt): `[0xa5, 0x40, 0x44, 0x30, 0x8c, 0xea, 0xc9, 0xb7]`
- ENCRYPTION_KEY (PBKDF2-SHA512 output): `7673ffd9efd70336a5442eda0b31457f7b6cdf7b42fe17f274434df55efa9839`
- HMAC_KEY (`SHA256(ENCRYPTION_KEY)`): `3d4c422806ba8964c9ee45070cd675c024d96648a0ddb4001325818c84951de2`
- MAC (`HMAC-SHA256(HMAC_KEY, hex_token || plaintext)`): `fbdbdb64e6a8231c342131d9f13dcd5a954b4c5021658fa5afcb3fc74dc82706`
- IV (first 16 bytes of MAC): `fbdbdb64e6a8231c342131d9f13dcd5a`

### TV-4 (EXTENDED; three signers; bip-0129.mediawiki lines 354-424)

3 more signers + 3 Round-2 descriptor-record ciphertexts. Includes `sh(wsh(multi(...)))` (P2SH-P2WSH NESTED_SEGWIT). Cycle 7a unit cells should cross-validate at least one TV-4 Signer's ENCRYPTION_KEY/HMAC_KEY/MAC chain.

## A5 — Cargo dep status

| Dep | Status |
|---|---|
| `pbkdf2 = "0.12"` | ✓ already direct dep (SLIP-39 uses `pbkdf2::<Hmac<Sha256>>`); supports `pbkdf2::<Hmac<Sha512>>` via type-param swap. No code change to add the SHA512 variant. |
| `hmac = "0.12"` | ✓ already direct dep. |
| `sha2 = "0.10"` | ✓ already direct dep (`Sha256` + `Sha512` both via this crate). |
| `aes = "0.8"` | ✓ already direct dep (added Cycle 6a). |
| `cbc = "0.1"` | ✓ already direct dep (Cycle 6a; NOT needed for AES-CTR). |
| `ctr` | ✗ NOT a dep. **Cycle 7a adds `ctr = "0.9"`.** Sibling of `cbc` from the same RustCrypto block-modes family. |
| `hex` | ✓ already direct dep. |

**Net new dep for Cycle 7a:** `ctr = "0.9"`. Same compact pattern as Cycle 6a (`cbc`).

## A6 — Current `wallet_import/bsms.rs` integration surface

- File: 703 LOC at master HEAD `b03cac3`.
- Plaintext shapes supported: 2-line / 4-line / 6-line (per file header comment lines 11-17).
- Encryption-related references: ZERO (grep for `encrypt|encryption|TOKEN|MAC|PBKDF2|HMAC|AES|cipher` returns only the file header descriptive comment; no code paths).
- Sniff at L57-69: requires literal `BSMS 1.0\n` prefix. Encrypted blobs (per BIP-129, just hex-encoded `MAC||ciphertext` with NO header) sniff NEGATIVE under current parser.
- Refusal behavior: encrypted blob → `import-wallet: bsms: parse error: blob is not valid UTF-8: ...` (when hex decode passes BUT char distribution looks like binary) OR `parse error: expected header BSMS 1.0` (when blob is non-UTF8 raw bytes).

**Cycle 7b implication:** the encrypted-blob sniff path is FUNDAMENTALLY different from plaintext. Either (a) add a new sniff that detects hex-only blobs as candidate BSMS-encrypted (ambiguous with other hex blobs); OR (b) require explicit `--format bsms` + `--bsms-encryption-token <FILE>` (user-explicit dispatch). Path (b) is per parent brainstorm's locked CLI surface.

## A7 — FOLLOWUP slug verification

Verified at master HEAD `b03cac3`:

- **Canonical entry:** `bsms-bip129-encryption-envelope` at `design/FOLLOWUPS.md:2546`. Body is accurate vs BIP-129 primary source (unlike Cycle 6's `wallet-import-electrum-encrypted` body which had a wrong "PBKDF2 + AES-CBC" scheme claim).
- **Predecessor entry:** `wallet-import-bsms-encrypted` at `design/FOLLOWUPS.md:2378` (filed at v0.27+ era; reconcile at Cycle 7b ship — either close both with cross-cite or merge bodies).
- **Parent:** `bsms-bip129-full-cutover` at `design/FOLLOWUPS.md:2208` (sub-item (c) carved out into `bsms-bip129-encryption-envelope`).

## A8 — Coinkite Python reference impl

Verified at `gh api repos/coinkite/bsms-bitcoin-secure-multisig-setup` (2026-05-21):
- Repo: `coinkite/bsms-bitcoin-secure-multisig-setup`
- Description: "Multisig wallet setup for multivendors"
- Files of interest: `bsms/bip129.py`, `bsms/encryption.py`, `bsms/util.py`, `test.py`
- License: per repo metadata (verify at Cycle 7b if cross-clone needed for cross-impl smoke).
- Author: Peter Gray (BIP-129 co-author).

**Cycle 7a does NOT require cloning Coinkite Python** because the v0.27.0 dossier already has cross-validated test-vector values. Cycle 7b cross-impl smoke (if needed) would clone + run `python3 test.py`.

## A9 — MAC-then-decrypt vs decrypt-then-MAC ordering (parent brainstorm flag)

Per BIP-129 line 165 (Encrypt-and-MAC):
- **On encrypt side:** MAC = HMAC(plaintext, ...); then encrypt plaintext to get ciphertext; wire = MAC || ciphertext.
- **On decrypt side:** parse MAC + ciphertext from wire; IV = first 16 bytes of MAC; decrypt ciphertext to get plaintext; compute expected_MAC = HMAC(plaintext, ...); verify expected_MAC == received_MAC.

This is "Decrypt-then-MAC" verification (MAC is over plaintext, so must decrypt first to verify). The parent brainstorm cited "MAC-then-decrypt ordering bug if found = potential SemVer-MAJOR" as a watch-for. Per BIP-129 the SPEC ordering is what we follow; "MAC-then-decrypt" would be an anti-pattern relative to the spec (would not match Encrypt-and-MAC). 7b's audit confirms the implementation follows spec.

**For AES-CTR specifically:** unauthenticated decryption is safe in the sense that AES-CTR doesn't have padding-oracle attacks (it's a stream cipher). So "Decrypt-then-MAC" with AES-CTR has no oracle risk; the spec-mandated ordering is correct.

## Recon verdict

**GREEN.** All Cycle 7a scope locks have primary-source backing:

- BIP-129 encryption scheme verified verbatim against `bitcoin/bips`.
- All test vectors inherited from v0.27.0 dossier; no decay (v0.27.0 dossier already cross-validated against Coinkite Python).
- 1 net-new Cargo dep (`ctr = "0.9"`).
- Current bsms.rs touch surface: ZERO at 7a (library-only; no parser integration this cycle).
- FOLLOWUP slug + body verified at HEAD `b03cac3`.

No findings require deferring or rescoping Cycle 7a. Brainstorm + library proceed against this dossier.
