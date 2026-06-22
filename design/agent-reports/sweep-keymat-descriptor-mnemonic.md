# Security sweep — secret key-material hygiene — `descriptor-mnemonic`

**Repo:** `/scratch/code/shibboleth/descriptor-mnemonic`
**Audited against:** `origin/main` @ `8c73b4d` (md-codec 0.39.0 / md-cli 0.9.1, cycle-10)
**Scope:** PRIVATE key material only (xprv/`Xpriv`/WIF/`SecretKey`/BIP-32 derivation secrets/passphrase/PIN/raw key bytes). Public material (xpub, descriptor templates, policies, fingerprints, checksums, BCH residues) excluded.
**Mode:** RECON / AUDIT ONLY. No fixes, no spec, no source edits.

---

## Headline verdict

**md-codec and md-cli are PUBLIC-ONLY at the memory level.** Neither crate ever holds a real wallet private key, seed, passphrase, or PIN in memory along any production code path.

The design is structurally public-key-only:

- **Every descriptor parse** uses `miniscript::Descriptor<DescriptorPublicKey>` — the PUBLIC key variant. There is **no** `DescriptorSecretKey` / `DescriptorXKey<…priv…>` parse anywhere (verified across all ~40 `MsDescriptor::from_str` sites in `crates/md-cli/src/parse/template.rs` + the codec's `to_miniscript.rs`).
- **`md`'s `--key @i=XPUB` intake** (`crates/md-cli/src/parse/keys.rs::parse_key`) hard-rejects anything that is not a serialized **xpub**: it matches the BIP-32 xpub version bytes (`0488B21E` mainnet / `043587CF` testnet) and errors otherwise — so an `xprv` (version `0488ADE4`) cannot enter. Payload retained is `chaincode(32)‖compressed-pubkey(33)` — public.
- **md1 wire format** carries xpubs/policies/templates only (in scope-EXCLUDE; confirmed not a secret).
- **Address derivation** (`crates/md-codec/src/derive.rs`) reconstructs a `bitcoin::bip32::Xpub` from the 65-byte public TLV payload and calls `Xpub::derive_pub`. No `Xpriv`, no private derivation.
- **`md repair`'s stdin path** (`crates/md-cli/src/cmd/repair.rs:75-92`) reads only md1 strings (public) from stdin.
- **No deps** on `zeroize` / `secrecy` / `mlock` in either `Cargo.toml` — and **none needed**, because nothing private is held.
- **No passphrase / PIN / seed-phrase intake** exists in the CLI surface (`main.rs` Command enum: encode/decode/verify/inspect/bytecode/vectors/compile/address/gui-schema/repair — all consume public templates, md1 strings, xpubs, fingerprints, paths).

Notably, the repo already practices defense-in-depth for the *argv-residue of public-but-sensitive* inputs:
- `crates/md-cli/src/process_hardening.rs` calls `prctl(PR_SET_DUMPABLE, 0)` to deny other-UID `/proc/$PID/cmdline` reads + disable core dumps.
- `crates/md-cli/src/output_advisory.rs` carries a `PrivateKeyMaterial` output-class advisory variant — but it is `#[allow(dead_code)]` and **explicitly never constructed** ("md emits only Template/WatchOnly"); it exists solely for byte-parity with mnemonic-toolkit's advisory text (gated by `tests/cli_output_class.rs`).

The only `secp256k1::SecretKey` / `Xpriv` constructions in the whole tree are: (a) one production helper that builds a *deterministic, domain-separated, non-wallet placeholder* point (detailed as the single Low candidate below), and (b) `#[test]`-only fixtures using the public `abandon abandon … about` vector mnemonic (`template.rs:2262-2277`, `identity.rs:861`) — out of scope (test code, public test vector).

---

## Candidate followup slugs

### 1 candidate (Low, defense-in-depth / hardening-completeness)

#### `md-cli-synthetic-xpub-secretkey-not-zeroized`
- **Secret type / gap class:** transient `secp256k1::SecretKey` scalar; gap class **1** (zeroize-on-drop gap) + **6** (key-derivation intermediate not scrubbed).
- **Site:** `crates/md-cli/src/parse/template.rs:623-643` (`fn synthetic_xpub_for`), specifically the `let secret = SecretKey::from_slice(&seed.to_byte_array())…` at **line 634** and the source `seed` bytes at **line 632**.
- **WHAT:** To make miniscript's parser accept a placeholder `@i`, this helper hashes a domain-separated tag (`b"md-v0.15"‖i‖depth`) to 32 bytes, wraps them in a `SecretKey`, and multiplies to get a public point. The `secret` (and the `seed` hash it came from) are bare on the stack and dropped un-zeroized; the 33-byte public key alone is what's emitted.
- **Severity:** **Low.** This is **not** a wallet key — it is a fully deterministic, public, domain-separated synthetic value derived from a compile-time-constant tag (no entropy, no user seed, reproducible by anyone reading the source). Disclosure of these bytes reveals nothing about any real key. Flagged only because (a) a bare `SecretKey` lingering in freed memory is precisely the pattern the constellation's first-class hygiene bar wants scrubbed-on-drop everywhere, and (b) it is the *sole* place in either crate where the `SecretKey` type is materialized in production — a natural "hold the line so the next maintainer doesn't copy the pattern for a real key" hardening item.
- **Reproduces on current source:** YES (`origin/main` @ 8c73b4d, line numbers exact).
- **NEW vs already-tracked:** Appears NEW (no `zeroize`/`secrecy` machinery present in the repo to suggest prior tracking; no FOLLOWUPS access from here — orchestrator should dedup).
- **WHY:** Keeps the "every materialized `SecretKey` is `Zeroizing`/scrubbed" invariant total, even for non-wallet scalars, so the codebase never normalizes a bare-`SecretKey` pattern that a future real-key path could inherit. Borderline-fileable; could legitimately be WONTFIX'd as "not secret material."

---

## Explicitly checked and CLEARED (verify-don't-refile)

| Site | Why cleared |
|---|---|
| `parse/keys.rs::parse_key` `--key @i=XPUB` | xpub-version-gated; xprv structurally rejected. Public payload only. |
| `parse/template.rs` ~40× `MsDescriptor::<DescriptorPublicKey>::from_str` | Public key variant exclusively; no `DescriptorSecretKey`. |
| `md-codec/src/derive.rs` (`xpub_from_tlv_bytes`, `derive_address`) | `Xpub::derive_pub` over public TLV bytes; no `Xpriv`. |
| `md-codec/src/to_miniscript.rs` | Builds `DescriptorPublicKey::XPub` only. |
| `md-codec/src/phrase.rs:18` `Mnemonic::from_entropy(id)` | `id` is the 128-bit **PolicyId** (public wallet-policy identity fingerprint) rendered as a 12-word phrase for human verification — NOT seed entropy. In scope-EXCLUDE (fingerprint/identity). |
| `cmd/repair.rs` stdin read | Reads md1 strings (public) only. |
| `template.rs:2262-2277`, `identity.rs:861` (`Xpriv`/`to_seed`/abandon mnemonic) | `#[cfg(test)]` only; public test-vector mnemonic. Out of scope. |
| `output_advisory.rs::PrivateKeyMaterial` | `#[allow(dead_code)]`, never constructed; advisory-text-parity placeholder only. No secret flows. |
| `process_hardening.rs` | Mitigation, not a leak — already hardens argv/core-dump residue. |
| Cargo.toml (both crates) | No `zeroize`/`secrecy` dep — and none required (nothing private held). |
