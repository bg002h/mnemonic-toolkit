# BRAINSTORM / DESIGN — `mnemonic nostr` (nostr-key wrappers) — v0.34.0

**Date:** 2026-05-22
**Source SHA (citations verified against):** `f501ec3` (`origin/master`, 0 ahead / 0 behind at write time)
**Status:** design approved by user (2026-05-22); pre-plan. Next step: `writing-plans` → `design/IMPLEMENTATION_PLAN_v0_34_0_nostr_key_wrappers.md` + opus R0 reviewer-loop.
**SemVer:** MINOR → `mnemonic-toolkit-v0.34.0` (new top-level subcommand).

---

## §0 — Motivation & origin

Recon seed: a NIP-23 long-form article *"Can We Turn a Nostr Public Key Into a Non-Taproot Bitcoin Address In Prod?"* by TheButterZone/Alwin (`npub19ctmsmtf9jtehddhctwmnacwqtnkh0p43tafttrgzdgy0wlppcpq50zf83`, kind 30023, decoded from the supplied `naddr`, published 2026-05-21). The article derives legacy/segwit-v0 Bitcoin addresses from a nostr key by assuming **even-y parity** (`02‖x`) → HASH160 → P2PKH/P2WPKH, and shows an `nsec → WIF` path with an Electrum `p2wpkh:` import prefix.

A nostr public key is an **x-only secp256k1 key** (32 bytes, BIP-340); `npub`/`nsec` are NIP-19 bech32 encodings of the 32-byte x-only pubkey / 32-byte secret scalar. This feature ingests **existing** nostr keys and wraps them as Bitcoin addresses, descriptors, and (for `nsec`) WIF — **both** the clean taproot path and the article's non-taproot path.

### Crypto framing (the central correctness point)
- **Taproot (`p2tr`)** is the *native* mapping — an x-only key **is** a taproot internal key; no parity fabrication. `Address::p2tr(secp, x_only, None, …)` = BIP-86 key-path (output key = internal tweaked by `H_TapTweak(internal)`).
- **Non-taproot (`p2pkh`/`p2wpkh`/`p2sh-p2wpkh`)** needs a 33-byte compressed key = `02‖x` (BIP-340 even-y). This is the same "lift-x even-y / `02‖32B`" projection already established in-tree at `crates/mnemonic-toolkit/src/cost/strip.rs:92` (verified `f501ec3`).
- **`nsec → WIF` even-y normalization is the crux:** given scalar `d`, if `d·G` has odd y, the published x-only key corresponds to `(n−d)·G`, so the WIF MUST encode `d' = n−d` (`SecretKey::negate()`); else `d' = d`. This guarantees the emitted WIF actually controls the emitted address (the "consistently same keypairs" property the article asserts). The npub `= x(d·G)` is parity-independent.

---

## §1 — Scope (locked decisions)

| Dimension | Decision |
|-----------|----------|
| Direction | **Both** taproot key-path **and** non-taproot (even-y) — selected by `--script-type` |
| Key material | **npub** (public → watch-only) **and** **nsec** (private → adds WIF) |
| Outputs | **addresses + WIF + plain-text descriptor string** (`tr()`/`wpkh()`/`pkh()`/`sh(wpkh())`) |
| **NOT** emitted | **No m-format cards** (md1/mk1/ms1) — verified infeasible without fabrication (see §10) |
| Architecture | **Dedicated `mnemonic nostr` top-level subcommand** (Approach B) |
| script-type | default `p2tr`; `--all-script-types` emits all four |
| WIF | plain compressed WIF + an Electrum import-hint line |

---

## §2 — CLI surface

```
mnemonic nostr <KEY-INPUT> [--script-type <T>] [--all-script-types]
                           [--network <N>] [--json]
```

**Key input — exactly one (clap `ArgGroup`, required):**
- `--pubkey <npub1…|64-hex>` — public key → watch-only (descriptor + address).
- `--secret <nsec1…|64-hex>` — private key → also derives pubkey + emits WIF. **SECRET.**
- `--secret-file <PATH>` — secret from a file (no argv exposure).
- `--secret-stdin` — secret from stdin. **SECRET** (flag implies secret stdin consumption).

Input-form autodetect: `npub1`/`nsec1` prefix → NIP-19 bech32; exactly 64 hex chars → raw bytes. HRP/flag mismatch (`nsec` to `--pubkey`, etc.) → refused.

**Options (reuse existing machinery):**
- `--script-type <p2pkh|p2wpkh|p2sh-p2wpkh|p2tr>` — reuses `convert`'s `ScriptType` enum + `parse_script_type_arg` (`crates/mnemonic-toolkit/src/cmd/convert.rs:357,364`, verified `f501ec3`). Modeled as `Option<ScriptType>`; when **absent and `--all-script-types` is absent**, defaults to `p2tr`.
- `--all-script-types` — emit descriptor + address for all four types. Declared in a clap mutex group with `--script-type` (`conflicts_with`), so supplying both **explicitly** is a clap usage error; the `p2tr` default is applied only in code when neither is present (it is not a clap `default_value`, avoiding a false conflict).
- `--network <mainnet|testnet|signet|regtest>` — reuses `CliNetwork` (`crates/mnemonic-toolkit/src/network.rs`). Default mainnet.
- `--json` — structured output.

Argv-leak advisory fires when `--secret` is used inline (mirrors the `electrum-decrypt` / `import-wallet` password handling).

---

## §3 — Crypto / derivation semantics

**npub path:**
1. Decode npub (NIP-19 bech32, HRP `npub`) or 64-hex → 32-byte x-only.
2. Validate it is a real curve point: `XOnlyPublicKey::from_slice` (lift_x must succeed).
3. Per script-type:
   - `p2tr`: `Address::p2tr(secp, x_only, None, hrp)`; descriptor `tr(<x-only-hex>)#<csum>`.
   - `p2wpkh`: compressed `02‖x` → `Address::p2wpkh`; descriptor `wpkh(<33B-hex>)#<csum>`.
   - `p2sh-p2wpkh`: `Address::p2shwpkh`; descriptor `sh(wpkh(<33B-hex>))#<csum>`.
   - `p2pkh`: `Address::p2pkh`; descriptor `pkh(<33B-hex>)#<csum>`.
   - (Address constructors mirror `convert.rs::build_address_from_xpub` at `convert.rs:~1551-1565`, verified `f501ec3` — but a sibling helper `build_address_from_pubkey(secp, PublicKey/XOnlyPublicKey, ScriptType, CliNetwork)` is needed since nostr supplies a raw key, not an `Xpub`.)
4. Descriptor checksum computed via `miniscript`.

**nsec path:**
1. Decode nsec → 32-byte scalar; validate `1 ≤ d < n` (`SecretKey::from_slice`).
2. **Even-y normalize:** `P = d·G`; if `P` has odd-y parity → `d' = SecretKey::negate(d)`, else `d' = d`. Emit `notice: nostr: secret normalized to even-y (BIP-340) for address consistency` when negation occurs.
3. x-only pubkey = `x(P)` (independent of parity) → npub + all npub-path outputs above, computed from `d'`.
4. WIF = `PrivateKey { compressed: true, network, inner: d' }.to_wif()` (plain WIF).

---

## §4 — Output & secret handling

Human-readable labeled block (default) + `--json` (structured). Example — `--secret`, `--script-type p2wpkh`:
```
nostr key (secret)
  x-only:      2e17b86d…0e02
  script-type: p2wpkh
  descriptor:  wpkh(02e17b86d…0e02)#<csum>
  address:     bc1q…
  wif:         L1aW…
  electrum:    p2wpkh:L1aW…             (Electrum ▸ Import private keys)
```
- `--all-script-types`: descriptor/address rows repeat per type; single WIF; Electrum hint per type.
- Electrum prefix tracks the script-type: `p2pkh:` / `p2wpkh:` / `p2wpkh-p2sh:` / `p2tr:`. **Plan-phase TODO:** verify the exact Electrum prefix strings against Electrum source (Electrum uses `p2wpkh-p2sh:`, not `p2sh-p2wpkh:`).
- **Secret-on-stdout (corrected per plan R0 I4):** the toolkit has **no** shared stdout TTY-redaction pathway — `convert` emits WIF as plain text and `convert-minikey-stdout-redaction` is an OPEN follow-up. So `mnemonic nostr` emits the WIF **plainly** (consistent with `convert`); secret hygiene rests on (a) the inline-`--secret` argv-leak advisory, (b) `secrets::flag_is_secret` masking the input flags, and (c) zeroize + `mlock`-pin of the in-memory secret (follow `import-wallet`/`electrum-decrypt` precedent). A future shared-redaction follow-up could cover both `convert` and `nostr`.

---

## §5 — Errors & validation
All nostr-key decode/validation failures use a **dedicated `ToolkitError::NostrKeyParse(String)`** variant (decision locked 2026-05-22), not `BadInput` — gives callers/tests a precise type and keeps nostr messages distinct from the m-format `HrpMismatch` (which is for `ms`/`mk`/`md` HRPs).
- Bad bech32 / checksum, wrong HRP for the flag (`nsec`→`--pubkey` etc.), bad hex length → `NostrKeyParse` (precise message).
- x-only not a curve point (lift_x fails) → `NostrKeyParse` ("not a valid secp256k1 x-only public key").
- scalar `0` or `≥ n` → `NostrKeyParse` ("not a valid secp256k1 secret key").
- **Exit code 1** (input/parse class, consistent with `Bip39` / `DescriptorParse`).
- **Placement (CLAUDE.md alphabetical-by-variant-name convention, verified `f501ec3`):** insert `NostrKeyParse(String)` between `NetworkMismatch` (`error.rs:243`) and `Repair` (`error.rs:250`) in `enum ToolkitError`, and at the matching alphabetical position in the `Display`, `exit_code` (`error.rs:435`), and `kind` match arms.

---

## §6 — Testing
- **NIP-19 decode KATs** — official npub/nsec spec vectors; bech32 decode + (npub) hex round-trip.
- **even-y consistency (the crux)** — a known nsec whose `d·G` is odd-y: assert WIF encodes `n−d`, **and** `WIF→address == npub→address` for every script-type (the round-trip property the article claims). Plus an even-y nsec: WIF encodes `d` unchanged.
- **Cross-impl oracle (vendored-fixture pattern; electrum/coinkite precedent)** — npub↔x-only + the four addresses + descriptor checksums cross-validated against an independent impl (`rust-nostr` for key parsing and/or Bitcoin Core `getdescriptorinfo` for descriptor+checksum). Committed fixtures + a deterministic regen script.
- **Network variants** — address HRP + WIF version byte per `--network`.
- **Secret hygiene** — nsec zeroized; inline-`--secret` argv-leak advisory fires; `--secret-stdin` path; `flag_is_secret` covers `--secret` + `--secret-stdin` (toolkit-side test).
- **CLI integration** — `--json` shape, `--all-script-types`, HRP-mismatch + invalid-key refusals.

---

## §7 — Lockstep & SemVer
- **SemVer MINOR → `mnemonic-toolkit-v0.34.0`** (new top-level subcommand; precedent seedqr v0.30.0, electrum-decrypt v0.33.0).
- **MANDATORY — GUI `schema_mirror`:** add a `nostr` `SubcommandSchema` to `mnemonic-gui/src/schema/mnemonic.rs` (flags + dropdown enums for `--script-type`/`--network`); bump the toolkit pin (`pinned-upstream.toml` + `Cargo.toml`) → v0.34.0; paired GUI release. New subcommand trips the flag-name-parity gate.
- **MANDATORY — secret projection:** add `--secret` + `--secret-stdin` to `secrets::flag_is_secret` in **both** toolkit and `mnemonic-gui` (the v0.33.1 leak-class lesson: omitting password flags from `flag_is_secret` is a leak class; `schema_mirror_secret_drift` enforces the GUI side).
- **MANDATORY — manual:** extend `docs/manual/src/40-cli-reference/41-mnemonic.md` with the `nostr` subcommand (mirrors `--help`); manual lint flag-coverage (`docs/manual/tests/lint.sh`) + CI `manual.yml`.
- **Verify** `mnemonic gui-schema` **JSON** emits the new subcommand (not just `--help`) — A.0 recon discipline.
- **No sibling-codec (md/mk/ms) companion** — no cards emitted; `FOLLOWUPS.md` companion not required.
- **New dep:** `bech32 = "0.11"` — already transitive via `bitcoin` 0.32 (`bech32 0.11.1` in `Cargo.lock`); promoting to a direct dep adds zero new build/supply-chain surface.

---

## §8 — Code layout / reuse map
- **New:** `crates/mnemonic-toolkit/src/cmd/nostr.rs` (clap args + `run`); `crates/mnemonic-toolkit/src/nostr.rs` (NIP-19 bech32 decode, even-y normalization, key validation) — keep crypto in a library module, CLI thin (mirrors `electrum_crypto.rs` ↔ `cmd/electrum_decrypt.rs`).
- **Reuse:** `ScriptType` + `parse_script_type_arg` + `CliNetwork` (refactor a shared `build_address_from_pubkey` alongside `convert.rs::build_address_from_xpub`); `mlock` pinning; zeroize. (No shared secret-on-stdout redaction pathway exists — see §4 / plan R0 I4.)
- **Wire:** new `Command::Nostr` arm in `crates/mnemonic-toolkit/src/main.rs` `enum Command` + dispatch in `cmd/mod.rs`; `gui_schema.rs` emission.

---

## §9 — Open items for the plan phase
1. Exact Electrum WIF script-type prefix strings (verify vs Electrum source).
2. ~~Whether to add `NostrKeyParse` or stay on `BadInput`.~~ **RESOLVED 2026-05-22 — add `NostrKeyParse(String)` (see §5).**
3. Cross-impl oracle choice (`rust-nostr` dev-dep vs Bitcoin Core CLI vs vendored Python) for the fixture regen script.
4. `--all-script-types` JSON shape (array of per-type objects).
5. Confirm `build_address_from_pubkey` extraction does not disturb `convert`'s existing `(xpub,address)` edge.

## §10 — Out of scope / non-goals (with rationale)
- **m-format cards (md1/mk1/ms1).** Verified infeasible for a single raw nostr key against `md-codec` source (`/scratch/code/shibboleth/descriptor-mnemonic`): `ExpandedKey.xpub: Option<[u8;65]>` = 32-byte chain-code ‖ 33-byte pubkey (`canonicalize.rs:347`), and `decode.rs:65-69` enforces explicit-origin + `validate_xpub_bytes`. A raw nostr key has no chain code / origin / xpub structure; ms1 needs BIP-39 entropy (no seed exists — npub→key is one-way); mk1 needs an xpub. All three would require fabricating non-derivable key material. The faithful "wrapper" is the plain descriptor string.
- **BIE2 / hardware-key analogues, signing, NIP-06 (seed→nostr derivation).** NIP-06 (ingest a BIP-39 seed, derive the nostr identity at `m/44'/1237'/…`, then a real seed-backed bundle) is the natural path to a *full card bundle* but is a **different feature** (ingests a seed, not an existing nostr key) — candidate FOLLOWUP, not this cycle.
