# mnemonic-toolkit v0.7 SPEC — `derive-child` subcommand

**Version:** 0.7.0
**Date:** 2026-05-06
**Status:** DRAFT (converged 0C/0I after 3 user-rounds + 2 architect-rounds; ready for execution per `IMPLEMENTATION_PLAN_v0_7.md` Phase 6)
**Predecessors:** [SPEC_mnemonic_toolkit_v0_5.md](SPEC_mnemonic_toolkit_v0_5.md), [SPEC_convert_v0_6.md](SPEC_convert_v0_6.md).

## §1 Purpose

`mnemonic derive-child` deterministically derives child entropy / child keys from a master xprv per BIP-85 (<https://github.com/bitcoin/bips/blob/master/bip-0085.mediawiki>). Six data-derivation applications are in scope for v0.7; RSA / RSA-GPG / DICE applications are explicitly deferred (see §5).

The subcommand replaces hand-assembly of BIP-85 derivations from `bitcoin::bip32::Xpriv::derive_priv` calls — which works mechanically but requires the user to manage the HMAC-SHA512 step + per-application output formatting. v0.7 ships first-class support for the 6 data-derivation apps from BIP-85's main "Applications" section.

## §2 Subcommand grammar

```
mnemonic derive-child \
  --from xprv=<master-xprv> \
  --application <bip39|hd-seed|xprv|hex|password-base64|password-base85> \
  --length <N> \
  --index <N> \
  [--network <mainnet|testnet|signet|regtest>]   # for --application xprv (network of emitted child xprv); defaults to source xprv's network \
  [--language <english|...>]                     # for --application bip39
```

All four core flags (`--from`, `--application`, `--length`, `--index`) are MANDATORY at the clap-grammar level. No defaults — BIP-85's deterministic-derivation contract requires exact parameter pinning.

**Per-application `--length` validators** (range varies per app; see §4). Out-of-range `--length` emits `error: --length <N> out of range for --application <app> (valid: <range>)` and exits 2.

**Sentinel-0 convention for fixed-output applications.** For `hd-seed` and `xprv` applications, the output size is BIP-85-specified and `--length` carries no value. To preserve grammar-uniformity (the four core flags appear at parse time on every invocation), these arms accept `--length 0` as a sentinel-absent marker; any non-zero value is refused with the not-applicable taxonomy in §4 / §7. Sentinel-0 is the canonical form for these arms.

## §3 BIP-85 derivation primitive

**Path:** `m/83696968'/<app>'/<idx>'`

The master xprv is derived to the BIP-85 root (`83696968'`), then to the application-specific subtree (`<app>'`), then to the index (`<idx>'`). All three components are hardened (`'` suffix).

**HMAC-SHA512 step:** the derived child xpriv's `private_key` (32 bytes) is fed through `HMAC-SHA512(key=b"bip-entropy-from-k", msg=child_xpriv.private_key)` to produce 64 bytes of entropy.

**Per-app entropy slicing:** each application takes a prefix of the 64-byte entropy:

- BIP-39: `length_in_words * 4 / 3` bytes (e.g., 12 words → 16 bytes; 24 words → 32 bytes).
- HD-Seed WIF: 64 bytes (full output).
- XPRV: 64 bytes (32 bytes chain code + 32 bytes private key).
- HEX: `length` bytes (16 ≤ length ≤ 64).
- PWD BASE64: 64 bytes (encoded to base64; truncated to `length` chars).
- PWD BASE85: 64 bytes (encoded to base85; truncated to `length` chars).

**Cross-reference:** BIP-85 §"Specification" (<https://github.com/bitcoin/bips/blob/master/bip-0085.mediawiki#specification>) defines the path + HMAC primitive; BIP-85 §"Applications" defines the per-app slicing rules.

## §4 In-scope applications (v0.7)

| App code | Name | `--application` value | Output format | `--length` range |
|---|---|---|---|---|
| `39'` | BIP-39 mnemonic | `bip39` | N-word phrase (whitespace-separated) | `12 \| 15 \| 18 \| 21 \| 24` words |
| `2'` | HD-Seed WIF | `hd-seed` | WIF-encoded 64-byte master HD seed | not applicable (fixed; `--length` rejected) |
| `32'` | XPRV | `xprv` | Child xprv (Base58Check-encoded BIP-32 extended privkey) | not applicable (fixed; `--length` rejected) |
| `128169'` | HEX | `hex` | N raw hex bytes | `16..=64` bytes |
| `707764'` | PWD BASE64 | `password-base64` | Base64-encoded password | `20..=86` chars |
| `707785'` | PWD BASE85 | `password-base85` | Base85-encoded password | `10..=80` chars |

Output is rendered to stdout as a single line (no leading whitespace; one trailing `\n`). Secret-bearing outputs (all 6 apps emit secret material) trigger the `convert::§7 secret-on-stdout warning` to stderr.

`hd-seed` and `xprv` reject any non-zero `--length` value with: `error: --length not applicable for --application <hd-seed|xprv> (output is fixed-size)`. The flag is still required at the clap-grammar level for grammar-uniformity; pass `--length 0` (sentinel-absent) on these arms — see §2.

## §5 Application scope (out-of-v0.7)

The following BIP-85 applications are explicitly out-of-scope for v0.7:

- **`828365'` RSA** — generates an RSA private key from BIP-85 entropy. Requires the `rsa` crate (<https://crates.io/crates/rsa>) which is NOT currently in the toolkit's dependency tree. Adding `rsa` is a non-trivial dependency expansion (RustCrypto's `rsa` pulls `num-bigint-dig`, `pkcs1`, `pkcs8`, etc.). Defer to v0.8 FOLLOWUPS pending demand signal.
- **`67797633'` RSA-GPG** — generates an RSA-GPG keypair from BIP-85 entropy. Same dependency rationale as `828365'`. Defer to v0.8 FOLLOWUPS.
- **`89101'` DICE** — generates deterministic dice rolls from BIP-85 entropy. Niche application (gaming / RNG-reproduction); marginal value for a key/wallet tool. Defer pending user demand (architect R2-N1 explicit out-of-scope listing).

When a user supplies `--application <rsa|rsa-gpg|dice>`, clap's enum parser rejects the value at parse time. The toolkit emits a refusal that points to the v0.8 deferral:

```
error: --application <rsa|rsa-gpg|dice> is out-of-scope for v0.7 (rsa crate not in tree; dice is niche). Tracked for v0.8+.
```

Exit code: 2 (`ToolkitError::DeriveChildUnsupportedApp`).

## §6 Test corpus

Phase 6 RED tests cover, at minimum:

1. **BIP-39 12-word reference vector** from BIP-85 §"Test Vectors" (<https://github.com/bitcoin/bips/blob/master/bip-0085.mediawiki#test-vectors>). Master xprv `xprv9s21ZrQH143K2LBWUUQRFXhucrQqBpKdRRxNVq2zBqsx8HVqFk2uYo8kmbaLLHRdqtQpUm98uKfu3vca1LqdGhUtyoFnCNkfmXRyPXLjbKb` → bip39/`12`/`0` produces phrase `girl mad pet galaxy egg matter matrix prison refuse sense ordinary nose`.
2. **BIP-39 18-word reference vector** from BIP-85 §"Test Vectors" — same master xprv, bip39/`18`/`0` produces 18-word output.
3. **HD-Seed WIF reference vector** from BIP-85 §"Test Vectors" — hd-seed/`0` (no length param).
4. **XPRV reference vector** from BIP-85 §"Test Vectors" — xprv/`0`.
5. **HEX reference vector** from BIP-85 §"Test Vectors" — hex/`64`/`0`.
6. **Password reference vector** — Pin both PWD BASE64 and PWD BASE85 reference vectors (BIP-85 §"Test Vectors" provides both).

Plus refusal tests:

- `--application rsa` → out-of-scope refusal stderr byte-exact.
- `--application bip39 --length 16` (invalid; valid is `12|15|18|21|24`) → out-of-range refusal stderr byte-exact.
- `--application hd-seed --length 32` → not-applicable refusal stderr byte-exact.

## §7 Refusal taxonomy

**Unsupported-application refusal stderr (byte-exact):**

```
error: --application <rsa|rsa-gpg|dice> is out-of-scope for v0.7 (rsa crate not in tree; dice is niche). Tracked for v0.8+.
```

**Out-of-range `--length` refusal stderr (byte-exact, per app):**

```
error: --length <N> out of range for --application bip39 (valid: 12 | 15 | 18 | 21 | 24 words)
error: --length <N> out of range for --application hex (valid: 16..=64 bytes)
error: --length <N> out of range for --application password-base64 (valid: 20..=86 chars)
error: --length <N> out of range for --application password-base85 (valid: 10..=80 chars)
```

**Not-applicable `--length` refusal stderr (byte-exact):**

```
error: --length not applicable for --application <hd-seed|xprv> (output is fixed-size)
```

**Invalid master xprv refusal:** standard `--from xprv=` parse error from `convert.rs` shared parser; exit 1 (BadInput class). Re-uses the existing `parse_from_input` taxonomy.

Exit code: 2 for all `derive-child`-specific refusals (`ToolkitError::DeriveChildRefusal` family).

## §8 Implementation hooks

- `crates/mnemonic-toolkit/src/cmd/derive_child.rs` (~150 LOC): clap argument struct + `run()` dispatcher.
- `crates/mnemonic-toolkit/src/bip85.rs` (~200 LOC):
  - `derive_entropy(master: &Xpriv, app_code: u32, app_params: &[u32], index: u32) -> [u8; 64]` — common helper for the HMAC-SHA512 step.
  - 6 application dispatchers: `format_bip39_phrase`, `format_hd_seed_wif`, `format_xprv_child`, `format_hex_bytes`, `format_password_base64`, `format_password_base85`.
- Reuse: `bitcoin::bip32::Xpriv::derive_priv` for the BIP-32 component; `bitcoin_hashes::HmacEngine<sha512::Hash>` for HMAC-SHA512 (already in tree via `bitcoin`).

## §9 Out-of-scope for v0.7 (consolidated)

- RSA / RSA-GPG / DICE applications (see §5).
- Multi-master input (e.g., "derive child from 2-of-3 SLIP-39 reconstruction") — orthogonal to BIP-85; defer to whatever brings SLIP-39 (or ms1-shares) into the toolkit.
- `--from phrase=...` source — BIP-85 spec defines derivation from a master xprv, not from a phrase. The user pre-converts phrase → xprv via `mnemonic convert --from phrase=... --to xprv` if needed. No grammar shortcut in v0.7.
