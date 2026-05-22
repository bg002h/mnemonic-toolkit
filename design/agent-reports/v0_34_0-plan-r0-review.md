# v0.34.0 nostr-key-wrappers — plan-doc opus R0 review (verbatim)

**Date:** 2026-05-22
**Reviewer:** opus `feature-dev:code-reviewer` (agent `aa13734ae77b71b76`)
**Target:** `design/IMPLEMENTATION_PLAN_v0_34_0_nostr_key_wrappers.md` (commit `752f8ed`) vs spec `BRAINSTORM_v0_34_0_nostr_key_wrappers.md` + source baseline `f501ec3`
**Verdict:** **RED** — 4 Critical + 5 Important + 3 Minor.

---

## Critical

### C1 — `src/nostr.rs` (lib crate) cannot reach `ToolkitError`, `ScriptType`, or `CliNetwork`; the "library-pure split" premise is false
- **Plan location:** Task A0.2 (`lib.rs: pub mod nostr;`), Task A1/A3 (`use crate::error::ToolkitError;`, `use crate::cmd::convert::ScriptType;`, `use crate::network::CliNetwork;`).
- **Evidence:** `lib.rs` declares only these `pub mod`s: `bsms_crypto`, `electrum_crypto`, `final_word`, `mlock`, `secret_taxonomy`, `secrets`, `seed_xor`, `seedqr`, `slip39` (`lib.rs:67-90`). It does NOT declare `cmd`, `error`, or `network`. Those three are binary-crate-only modules declared in `main.rs:5,12,16` (`mod cmd;`, `mod error;`, `mod network;`). `ScriptType` lives in `cmd/convert.rs:357` (binary crate). So from a lib module, `crate::error::ToolkitError`, `crate::cmd::convert::ScriptType`, and `crate::network::CliNetwork` do not resolve — hard compile errors. The cited precedent `electrum_crypto.rs`/`seedqr.rs` are library-PURE: `seedqr.rs:24` defines its own `SeedqrError` and the binary maps it to `ToolkitError::BadInput` at the `cmd/seedqr.rs` boundary.
- **Fix:** Either (a) make `nostr.rs` library-pure: define a local error, accept primitive args, map to `ToolkitError::NostrKeyParse` only in `cmd/nostr.rs`; or (b) move `nostr.rs` into the binary crate (`mod nostr;` in `main.rs`, NOT `pub mod nostr;` in `lib.rs`) so `crate::error`/`crate::cmd::convert::ScriptType`/`crate::network` resolve. Option (b) is the smaller change and keeps `ScriptType`/`CliNetwork` reuse intact, but loses the "lib module" framing.

### C2 — `ScriptType::as_str()` does not exist; every `st.as_str()` call won't compile
- **Plan location:** Tasks B2 (line 609), B3 (line 703); self-review O3.
- **Evidence:** `convert.rs:55` `pub fn as_str` is on `NodeType`, not `ScriptType`. `ScriptType` (`convert.rs:357-362`) has NO `impl` block — no `as_str`, no `Display`. Only accessor is `parse_script_type_arg` (`convert.rs:364`). `impl ScriptType` returns nothing.
- **Fix:** Add `impl ScriptType { pub fn as_str(self) -> &'static str { match ... } }` to `convert.rs` (canonical, round-trips with `parse_script_type_arg`). Update the plan to ADD this method rather than asserting it exists.

### C3 — `flag_is_secret` takes ONE argument, not two; Task C1 test won't compile
- **Plan location:** Task C1 Step 1 (lines 864-868): `flag_is_secret("nostr", "--secret")`, etc.
- **Evidence:** `secrets.rs:49` `pub fn flag_is_secret(flag_name: &str) -> bool` — single `&str`, subcommand-agnostic (`secrets.rs:50-61`).
- **Fix:** `flag_is_secret("--secret")`, `flag_is_secret("--secret-stdin")`, `!flag_is_secret("--secret-file")`. Drop the `"nostr"` arg. `--secret`/`--secret-stdin` become globally secret (acceptable; only `nostr` uses them).

### C4 — There is no `Display` per-variant match arm to add for `NostrKeyParse`; the actual exhaustive matches are `kind()` and `message()`, which the plan omits
- **Plan location:** Task A0.1 Step 2 (phantom Display arm).
- **Evidence:** `error.rs:747-751` `impl Display` delegates to `self.message()`; NO per-variant Display match. The compiler-forced exhaustive matches (besides `exit_code` at `error.rs:436`) are `kind()` (`error.rs:489-538`, no wildcard) and `message()` (`error.rs:543-713`, exhaustive). `details()` (`error.rs:720-743`) has `_ => None`. Build fails on missing `message()`/`kind()` arms.
- **Fix:** A0.1 adds arms in: (1) `enum` between `NetworkMismatch`/`Repair`; (2) `kind()` → `NostrKeyParse(_) => "NostrKeyParse"` (between `error.rs:531`/`:532`); (3) `message()` → `NostrKeyParse(msg) => msg.clone()` (between `error.rs:685`/`:686`); (4) `exit_code()` → `=> 1` (between `:476`/`:477`). Remove the bogus Display step.

## Important

### I1 — `crate::nostr::*` and `crate::mlock::*` paths are wrong inside `cmd/nostr.rs` (binary crate)
- **Plan location:** Tasks B2/B3 (`crate::nostr::decode_npub`, `crate::mlock::pin_pages_for`, etc.).
- **Evidence:** Existing `cmd/*.rs` reach lib modules via `mnemonic_toolkit::` (e.g. `cmd/electrum_decrypt.rs:119` `mnemonic_toolkit::mlock::pin_pages_for`; `cmd/seedqr.rs:13` `use mnemonic_toolkit::seedqr::...`). `crate::mlock` won't resolve. (`crate::nostr` resolves only if C1 option (b) is taken — `nostr` becomes a binary module.)
- **Fix:** Use `mnemonic_toolkit::mlock::...`. Under C1 option (b), `crate::nostr::...` is correct (binary module); under (a), `mnemonic_toolkit::nostr::...`.

### I2 — `cmd::nostr::run` signature mismatches the dispatch and the sibling convention (by-ref `&Args`, returns `Result<u8>`)
- **Plan location:** Task B1/B2: `run(args: NostrArgs, ...) -> Result<(), ToolkitError>` + dispatch `.map(|_| 0)`.
- **Evidence:** `main.rs:108` dispatch is `match &cli.command` → arms bind `&XArgs`. Siblings: `cmd/seedqr.rs:121-126` `run(args: &SeedqrArgs, ...) -> Result<u8, ToolkitError>`; `cmd/electrum_decrypt.rs:85-90` likewise. `NostrArgs` derives only `Debug`, not `Clone`.
- **Fix:** `pub fn run<R,W,E>(args: &NostrArgs, stdin, stdout, stderr) -> Result<u8, ToolkitError>` with `Ok(0)`; dispatch `Command::Nostr(args) => cmd::nostr::run(args, stdin, stdout, stderr),` (no `.map`), matching `Command::ElectrumDecrypt` (`main.rs:121-123`).

### I3 — The `nsec` test vector is checksum-invalid (wrong final char); the npub/nsec/hex constants are an unverified, mismatched set
- **Plan location:** Task A1 Step 1 (lines 127,129,130), Task B3 (line 645).
- **Evidence:** NIP-19 spec: `nsec1vl029mgpspedva04g90vltkh6fvh240zqtv9k0t9af8935ke9laqsnlfe5` (ends `fe5`) ↔ hex `67dea2ed018072d675f5415ecfaed7d2597555e202d85b3d65ea4e58d2d92ffa`. The plan's `NSEC` ends `...fe9` (invalid checksum). Also `PUB_HEX = 3bf0c63f...459d` is the nprofile TLV example, NOT the keypair of `SEC_HEX = 67dea2ed...92ffa`; the "Same key" comment is wrong.
- **Fix:** Use `nsec...fe5`. Use a verified npub pair, e.g. spec's `npub10elfcs4fr0l0r8af98jlmgdh9c8tcxjvz9qkw038js35mp4dma8qzvjptg` ↔ `7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e`. Remove the misleading "Same key" comment.

### I4 — Spec §4 "existing secret-on-stdout redaction pathway" does not exist in the toolkit; O1 needs reframing
- **Plan location:** Self-review O1; spec §4.
- **Evidence:** No stdout secret-redaction exists. `is_terminal()` is used only for auto-repair gating (`repair.rs:356`, `main.rs:44`). `convert` emits WIF as plain text; `secret_taxonomy.rs:72` documents `convert-minikey-stdout-redaction` as an OPEN follow-up — convert does NOT redact today. B3's plain-WIF emission is actually consistent with current behavior.
- **Fix:** Resolve O1 by correcting the spec: no shared redaction pathway exists; emit WIF plainly (matching `convert`); rely on the argv advisory + `flag_is_secret`. Drop the `[SECRET]` marker from spec §4 unless implemented.

### I5 — `default_value_t = CliNetwork::Mainnet` works, but verify the established pattern
- **Plan location:** Task B1 (line 509).
- **Evidence:** `CliNetwork` derives `ValueEnum` (`network.rs:10`), no `Display`, no `Default`. `default_value_t` renders via `to_possible_value()` (compiles; cf. `seedqr.rs:71`). Toolkit `--network` precedent is `default_value = "mainnet"` (`export_wallet.rs:63`). Both valid.
- **Fix:** Keep `default_value_t = CliNetwork::Mainnet`. Do NOT add `Default`/`#[default]` to `CliNetwork`.

## Minor

### M1 — `Command::Nostr` enum placement is not alphabetical, and the enum isn't alphabetically ordered
- **Evidence:** `main.rs:59-92` `enum Command` is feature-grouped, not alphabetical. CLAUDE.md alphabetical rule is for `ToolkitError` variants, not `Command`.
- **Fix:** Place `Nostr` naturally (e.g. after `Seedqr`); no alphabetical constraint.

### M2 — `lib.rs`/`cmd/mod.rs` "alphabetical pub mod" insertion points
- **Evidence:** `cmd/mod.rs:3-18` is alphabetical → `pub mod nostr;` between `inspect`(:12)/`repair`(:13). (Cosmetic.)

### M3 — Spec §4 example shows a `[SECRET]` marker the plan never emits
- **Fix:** Make spec and impl agree (tie to I4).

---

## Verified-correct (no action needed)
- `bitcoin::bech32` re-export (`bitcoin-0.32.8/src/lib.rs:66`); `bech32::decode -> (Hrp, Vec<u8>)` (`bech32-0.11.1/src/lib.rs:208`); `Hrp::parse` + `Hrp: Display`/`PartialEq` — the decode (A1) is sound.
- `CompressedPublicKey(pub secp256k1::PublicKey)` tuple constructor (`crypto/key.rs:274`); `PublicKey::from_x_only_public_key` (`key.rs:493`); `XOnlyPublicKey::from_slice` (`key.rs:1162`); `SecretKey::{from_slice,negate(self),secret_bytes,x_only_public_key<C: Signing>}` (`key.rs:215,267,262,345`). `normalize_to_even_y<C: Signing>` bound matches.
- `Address::{p2pkh(impl Into<PubkeyHash>, impl Into<NetworkKind>), p2wpkh(&CompressedPublicKey, hrp), p2shwpkh(&CompressedPublicKey, network), p2tr<C: Verification>(secp, xonly, None, hrp)}` (`address/mod.rs:400,431,439,461`) — all `address_for` call forms match; consistent with `convert.rs:1558-1565`.
- `PrivateKey { pub compressed, pub network, pub inner }` + `to_wif`/`from_wif` (`crypto/key.rs:398-404,466,474`).
- Patched miniscript parses raw `tr(<64hex>)` (`descriptor/key.rs:757-760`) and `wpkh/pkh/sh(wpkh(<66hex>))` (`:762-773`); `Descriptor` Display appends `#checksum` (`descriptor/mod.rs:1167-1195`).
- `CliNetwork::{network_kind,known_hrp}` (`network.rs:30,40`), `mlock::pin_pages_for` (`mlock.rs:90`), `parse_script_type_arg` pub (`convert.rs:364`).
- `exit_code` insertion point: between `NetworkMismatch` (`error.rs:476`) and `Repair` (`error.rs:477`).
- gui-schema auto-walks `cmd.get_subcommands()` (`gui_schema.rs:990`) — Task C2 needs no edit.
- CLI harness `assert_cmd::Command` (`cli_electrum_decrypt.rs:8,15-17`); `serde_json`/`predicates`/`tempfile` available (`Cargo.toml:43,51,52`).
- ArgGroup design: field-name (snake_case) args, bool flag in value group, `conflicts_with = "<field>"` — validated by `import_wallet.rs:115-118`.

---

## Verdict: RED
4 Critical + 5 Important. Crypto/descriptor/address/bech32/clap-group designs are sound and grounded in real APIs, but C1 (crate-boundary architecture), C2 (`ScriptType::as_str` nonexistent), C3 (`flag_is_secret` arity), C4 (error-wiring) touch foundational tasks A0/A/B. All mechanically fixable; re-dispatch after folding.

Source: [NIP-19 spec](https://github.com/nostr-protocol/nips/blob/master/19.md)
