# Security sweep — SECRET KEY MATERIAL hygiene in `mnemonic-key`

**Repo:** `/scratch/code/shibboleth/mnemonic-key`
**Audited against:** `origin/main` @ `df7c2eb` (`release(mk-cli): v0.10.1 — M12 mixed-case repair + L20 variant off-by-one (cycle-12)`)
**Date:** 2026-06-21
**Scope:** mk-codec (xpub `mk1` codec library) + mk-cli (the `mk` CLI). RECON/AUDIT ONLY — no fixes, no spec/plan, no source edits.
**Lens:** secret-memory-hygiene as a first-class quality bar; defense-in-depth findings count.

---

## HEADLINE VERDICT: mk is PUBLIC-ONLY. No PRIVATE key material in memory. ZERO secret-hygiene slugs.

mk-codec and mk-cli handle **no private key material whatsoever**. The `mk1` format encodes **xpubs** (extended *public* keys), origin fingerprints (public), derivation paths (public), and 4-byte policy-id/template-id stubs (public identity hashes). Every code path was verified against `origin/main`:

- **No xprv / Xpriv / WIF / SecretKey / private-scalar acceptance anywhere in shipped code.** A repo-wide grep for `xprv|Xpriv|from_wif|PrivateKey::from|parse.*priv` over all `src/**/*.rs` returns **zero hits**.
- **No `zeroize` / `secrecy` dependency** in any `Cargo.toml` (root, mk-cli, mk-codec, fuzz) — and correctly so: there is no owned secret to scrub.
- **No mlock / munlock / memlock** anywhere — correctly absent (mlock is for secret seeds mk never holds).
- **All secp256k1 use in the CLI is `Secp256k1::verification_only()`** (`derive_support.rs:116-117`, `secp_verify()`); xpub children are derived via `derive_pub` (public derivation), and **hardened derivation is structurally rejected** because an xpub has no private key (`derive.rs:113-122`).
- The `OutputClass::PrivateKeyMaterial` advisory variant **exists for cross-repo byte-parity only and is NEVER constructed in mk-cli** — every command constructs `OutputClass::WatchOnly` (verified: the only `OutputClass::` constructions in `mk-cli/src/cmd/*` are all `WatchOnly`; `output_advisory.rs:14-16` documents this explicitly).
- The in-memory decoded type `KeyCard` (`key_card.rs:23-59`) holds only `policy_id_stubs: Vec<[u8;4]>`, `origin_fingerprint: Option<Fingerprint>`, `origin_path: DerivationPath`, `xpub: Xpub` — all public. `#[derive(Debug)]` on it leaks nothing secret.

This is the expected and correct posture for a public-xpub codec. Per the sweep brief: **mk is genuinely public-only — say so, and file few/no slugs.** That is the finding.

---

## Verified-clean paths (per the audit checklist)

| Surface | File:line (origin/main) | Material handled | Verdict |
|---|---|---|---|
| `mk encode` | `cmd/encode.rs:18-63` (args), `:67-117` (run) | `--xpub` (public), `--origin-fingerprint` (public), `--origin-path`, `--policy-id-stub`, `--from-md1`, `--privacy-preserving` | All inputs public. No secret. |
| `--from-md1` stub derivation | `cmd/mod.rs:72-83` (`derive_stub_from_md1`) | md1 descriptor string → `md_codec::decode_md1_string` → `compute_wallet_policy_id` / `compute_wallet_descriptor_template_id` → top 4 bytes | md1 is a **public** descriptor/template format; output is a public identity hash. No private intermediate. |
| `mk decode` / `inspect` / `verify` / `repair` | `cmd/decode.rs`, `inspect.rs`, `verify.rs`, `repair.rs` | mk1 strings (public xpub wire bytes) | Public-xpub codec round-trip; `WatchOnly` advisory. |
| `mk derive` | `cmd/derive.rs:43-105` | mk1 string → `decode` → `derive_pub` (verify-only ctx) | Public unhardened derivation; hardened **refused** (`:113-122`). No `SecretKey`. |
| `mk address` | `cmd/address.rs:73-138` | mk1 string → `derive_pub` → `render_address` | Public address rendering from xpub. No `SecretKey`. |
| `--xpub` parse / SLIP-0132 normalize | `cmd/mod.rs:148-163`, `slip132.rs:126-…` | `Xpub::from_str`; SLIP-0132 maps only `ypub/zpub/Ypub/Zpub`(+`upub/vpub` testnet) → `xpub/tpub` | **Public variants only** — no `yprv/zprv`/private variants exist in the table; comment: "key material is unchanged; only the 4 version bytes." |
| Process hardening | `process_hardening.rs:16-22` (`set_non_dumpable`), wired at `main.rs:59` | `prctl(PR_SET_DUMPABLE,0)` | Present and wired — defense-in-depth carry-over even though no secret transits. Already good. |
| argv/env/stdin intake | `cmd/mod.rs:98-128` (`read_mk1_strings`, stdin `-`) | mk1 strings (public) | Reads public mk1 strings into bare `String`. Not a secret residue. |

---

## Non-findings deliberately NOT filed (and why)

- **mk1 wire bytes in bare `String`/`Vec` (encode/decode/repair/chunk/bch).** mk1 encodes a **public** xpub. Per the brief, public-xpub handling is explicitly out of scope and MUST NOT be flagged as secret residue. Not filed.
- **`gen_mk_vectors.rs` synthetic `SecretKey` from `[seed_byte; 32]`** (`crates/mk-codec/src/bin/gen_mk_vectors.rs:966-991`, `synthetic_xpub`). This is a **dev-only, feature-gated** (`required-features = ["gen-vectors"]`, off by default) codegen tool that mints *public xpub test vectors* from **hardcoded constant** seed bytes (`[seed_byte; 32]`, seed_byte 0x01..0x12). The `secret_bytes` local is a bare `[u8;32]` not zeroized, but (a) it never runs in any shipped binary, (b) the "secret" is a documented public test constant backing no funds, (c) the code comments explicitly state "Vectors exist for testing wire-format conformance, not security." This is **not funds-backing secret material** — does not meet the first-class bar. **NOT filed** (noted for completeness only; if a hygiene purist wants a defense-in-depth nit it would be MINOR/INFO at most, but it fails the "real funds, engraved on steel" lens).
- **Test-only `SecretKey` constants** in `tests/`, `test_helpers.rs`, `common/mod.rs`, `bch_adversarial.rs`, `round_trip.rs`, `indel_reject_contract.rs` — all build public xpubs from fixed bytes for conformance testing. Not secret material. NOT filed.
- **Cycle-B mlock infra companion (already-fixed):** verified N/A — mk has **no** mlock infra and **needs none** (public-only). Nothing to re-file.
- **M12 / L20 (recently shipped):** confirmed output-correctness (mixed-case repair / variant off-by-one), not secret-hygiene. Not in scope.

---

## CANDIDATE FOLLOWUP SLUGS

**Count: 0.**

No secret-memory-hygiene followups. mk handles no private key material in memory; there is no owned secret to zeroize, redact, mlock, scrub, or constant-time-compare. The single dev-only synthetic-`SecretKey` codegen path (`gen_mk_vectors.rs`) operates on hardcoded public test constants in an off-by-default binary and does not meet the funds-safety bar that would justify a slug.

If the orchestrator nonetheless wants the lone defense-in-depth nit tracked as INFO (NOT recommended — it is noise against the first-class-secrets bar), the only candidate would be:

- *(INFO, OPTIONAL, likely-decline)* `mk-gen-vectors-synthetic-seckey-not-zeroized` — gap class 1 (zeroize-on-drop) + 6 (KDF intermediate) — `crates/mk-codec/src/bin/gen_mk_vectors.rs:966-971` — the `secret_bytes: [u8;32]` / `sk: SecretKey` locals in `synthetic_xpub` are dropped without scrub. **WHAT:** dev-only feature-gated (`gen-vectors`, off by default) codegen tool builds public test-vector xpubs from hardcoded constant seed bytes. **Severity: INFO / negligible.** **NEW.** **WHY (against filing):** not funds-backing, never in a shipped binary, the "secret" is a published test constant — fails the real-funds lens; filing it dilutes the secret-hygiene signal. Listed only for orchestrator completeness.

---

## Bottom line

mk-codec/mk-cli = a **public-material codec** (xpubs + BCH checksums + chunking + public identity stubs). It ingests no xprv/SecretKey/WIF/passphrase/PIN; has no encrypt/ECIES feature; performs no private BIP-32 derivation (verify-only secp context, hardened-derivation refused); and constructs no `PrivateKeyMaterial` output. The `prctl(PR_SET_DUMPABLE,0)` hardening is present and wired as appropriate defense-in-depth. **Zero secret-hygiene followups warranted.**
