# v0.36.0 — Plan-doc R0 architect review (opus) — MANDATORY pre-implementation gate

**Date:** 2026-05-23
**Cycle:** v0.36.0 `verify-message` (legacy+BIP-322) + `decode-address` + convert-help freebie + electrum lock-tests
**Branch:** `v0.36.0-verify-decode-address`
**Reviewer:** opus (feature-dev:code-reviewer), R0
**Target:** `design/IMPLEMENTATION_PLAN_v0_36_0_verify_decode_address.md`
**Reviewer agentId:** a81c65b57207226bb

---

## Critical

**C1 — Phase 1's central premise is factually false: the `entropy` row ALREADY EXISTS in the `--from` long-help; following Step 3 produces a DUPLICATE row.**
Live `convert.rs:175`: `///   entropy          raw entropy hex (secret)` — exactly between `seedqr`(173-174) and `xprv`(177), where Step 3 says to insert. The plan's "omits the entropy row" is false (root cause: the controller's grep patterns omitted the literal "entropy", so the existing row was never seen — in both the survey probe and the plan recon). The Step-4 test asserting `contains("raw BIP-39 entropy")` does NOT pass against current `raw entropy hex`. Phase 1 must be rewritten: there is no missing row — at most a wording enrichment + a (valuable) regression lock-test that `--from entropy=` works.

**C2 — `is_signed_by_address` is P2PKH-ONLY in bitcoin 0.32.8; it returns `Err(UnsupportedAddressType)` for segwit/wrapped/taproot, not a verification result.**
`bitcoin-0.32.8/src/sign_message.rs:146-161`: matches `Some(AddressType::P2pkh) => Ok(...)`, `Some(other) => Err(UnsupportedAddressType)`, `None => Ok(false)`. So legacy ("Bitcoin Signed Message"/BIP-137) verification via the crate covers **P2PKH only**. The plan's claim that legacy covers "P2PKH + BIP-137 P2WPKH/P2SH-P2WPKH" is wrong; `bsms_verify.rs` (which compares a raw pubkey, never an address) is NOT a segwit precedent. In `--format legacy` over a segwit address, `try_legacy`'s `.ok()` swallows the Err → fires a misleading "not a valid 65-byte signature" error. Resolution: scope legacy → P2PKH only; route P2WPKH/P2SH-P2WPKH/P2TR through BIP-322 (the bip322 crate covers exactly those three and NOT P2PKH — so the two partition cleanly by address type). Document it; give an honest error for `--format legacy` on non-P2PKH; choose a P2PKH legacy test vector.

## Important

**I1 — `VerifyMessage` alphabetical slot references variants that are NOT in `ToolkitError`.**
Plan candidate "between `Unset`@320 and `XpubParse`@336" is wrong: `Unset` is in `enum EnvVarMissingReason` (316-324); `XpubParse` is in `enum BitcoinErrorKind` (332-338). `ToolkitError`'s actual tail (279-311): `SlotInputViolation`(276-284) → `UnknownHrp`(285-290) → `XpubSearchNoMatch`(291-311). Correct slot: `VerifyMessage(String)` between `UnknownHrp` and `XpubSearchNoMatch`, with matching arms in `exit_code` (500-501), `kind` (558-559), `message` (732-743); NO `details` arm (String → `_=>None`@773, per SilentPayment/NostrKeyParse precedent).

**I2 — `AddressType` is `#[non_exhaustive]` with SIX variants (incl `P2a`); a 5-arm match won't compile.**
`bitcoin-0.32.8/src/address/mod.rs:64-79`: `#[non_exhaustive] enum AddressType { P2pkh, P2sh, P2wpkh, P2wsh, P2tr, P2a }`. A 5-arm match with no `_` is a compile error; `address_type()` can return `Some(P2a)` (mod.rs:503-504). Fix: use `AddressType`'s `Display`/`to_string()` (mod.rs:81-92 yields lowercase `"p2pkh"…"p2a"`) — exactly the desired output, forward-compatible, no enumeration.

## Minor

**M1 — `gui_schema_lists_all_subcommands` is a hardcoded sorted vec, not a count.** `tests/cli_gui_schema.rs:71-101`. Insert `"decode-address"` between `"convert"`(76)/`"derive-child"`(77); `"verify-message"` between `"verify-bundle"`(92)/`"xpub-search-account-of-descriptor"`(93) (`verify-b` < `verify-m`). Update prose comment count (69) to 25.

**M2 — Phase 3 test name `..regtest..` is misleading; regtest NOT in a `tb1` set, testnet4 silently dropped.** `mod.rs:208-213`: `tb1` = Testnet|Testnet4|Signet; Regtest=`bcrt1` distinct. Rename + document testnet4.

**M3 — `decode_address::run` signature.** main.rs:135-136 dispatches `(args, stdin, stdout, stderr)` uniformly. Keep 4-arg `run<R,W,E>` (ignore stdin) for uniformity.

## Verification summary (confirmed correct)
- **Module privacy (A):** error.rs NOT in lib.rs; both modules return ToolkitError → MUST be binary-private (`mod …;` in main.rs, mirror silent_payment@21/nostr@17). `pub mod` in lib.rs would not compile. cmd/mod.rs `pub mod` additions correct.
- **DecodeAddress slot (B):** between `CosignersFile`(89, ends 91) and `DeriveChildLengthNotApplicable`(94). Arms in exit_code(468/469), kind(524/525), message(627/628); no details arm.
- **sign_message API (C):** `signed_msg_hash(&str)->sha256d::Hash`(201); `recover_pubkey`(133-137); `is_signed_by_address(&self,secp,&Address,sha256d::Hash)->Result<bool,_>`(146); `from_base64`(173, base64 feature on @Cargo.toml:38). msg_hash types match. (P2PKH-only limit per C2.)
- **Address API (C):** parse→`Address<NetworkUnchecked>`(FromStr 814); `is_valid_for_network`(721); `assume_checked`(788); `address_type()->Option<AddressType>`(492); `witness_program()->Option<WitnessProgram>`(543); `script_pubkey()->ScriptBuf`(593); `WitnessProgram::version()->WitnessVersion`(114); `WitnessVersion::to_num(self)->u8`(73) → `wp.version().to_num()` correct.
- **Test vectors (C):** P2WPKH `bc1qw508…f3t4`→`0014751e76e8199196d454941c45d1b3a323f1433bd6` (BIP-173) ✓; P2TR `bc1p0xlxv…z7vqzk5jj0` (BIP-350, v1, `5120…`) ✓; P2PKH `1BvBM…NVN2`→`76a914…88ac` ✓.
- **bip322 0.0.10 (D):** `verify_simple_encoded(&str,&str,&str)->Result<()>`(verify.rs:5); `verify_full_encoded`(24); supports P2TR/P2WPKH/P2SH-P2WPKH only, NOT P2PKH (verify.rs:67-98). Dep `bitcoin="0.32.5"` semver-compat w/ locked 0.32.8 — no duplicate. Spec vector `bc1q9vza2e8…gkx0l` msgs ""/"Hello World" = crate's own SEGWIT_ADDRESS test (lib.rs:42,199-247), traceable to BIP-322 mediawiki ✓. Dep acceptable (rust-bitcoin org, verify-only, no dup, MIT/CC0); pin `=0.0.10`; confirm crate name `bip322` NOT `bip322-rs`.
- **Legacy reuse (E):** bsms_verify.rs:26,38-41,49-50 uses signed_msg_hash+from_base64+recover_pubkey+verification_only exactly — reusable for digest/decode; compares raw pubkey (not address) → not a segwit precedent (reinforces C2).
- **Registration (F):** Command enum(60-98)+dispatch(116-151) mechanical; CLI tests via assert_cmd cargo_bin (BIN target) → `cargo test --bin mnemonic` / `--test` correct; cli-subcommands.list additions required.
- **SemVer+lockstep (H):** MINOR correct. `--from` free-text (parse_from_input, kind=text per cli_gui_schema.rs:200-212) → no schema_mirror impact for freebie. GUI schema_mirror gates flag-NAMES only. cli-subcommands.list additions planned.

## Open-question dispositions (G)
- (a) VerifyMessage slot: between `UnknownHrp` and `XpubSearchNoMatch` (I1).
- (b) Electrum `:460`: LEAVE as-is; loose test (`contains("electrum")`) passes (msg interpolates `from.as_str()`="electrum-phrase"); defer honest-wording to FOLLOWUP.
- (c) bip322: `verify_simple_encoded` only; do NOT auto-fallback to full (different encodings: witness-stack vs full-tx base64). Full → future `--format bip322-full` FOLLOWUP.
- (d) exit code: emit structured result first; malformed/undecodable → exit 1 via VerifyMessage error (stderr); cleanly-decoded-but-invalid → exit 1 with `valid:false` envelope on stdout (no stderr error). Pin it.
- (e) bip322 dep acceptable; pin `=0.0.10`; confirm `bip322` not `bip322-rs`.

VERDICT: RED (2C/2I)

---

## Fold disposition (controller) — round 0 → R1
Folding ALL:
- **C1:** Rewrite Phase 1 — there is NO missing row (`convert.rs:175` already has `entropy raw entropy hex (secret)`). Recharacterize as: (i) regression lock-test that `--from entropy=<hex> --to phrase` works (valuable, PASS-on-write); (ii) OPTIONAL one-word wording enrichment `raw entropy hex` → `raw entropy hex (16/20/24/28/32 bytes)` driven by a RED→GREEN help-text test. Drop the false "insert missing row" step. Loose-assert the lock-test on `contains("entropy")`.
- **C2:** Partition by address type — legacy = P2PKH ONLY; bip322 = P2WPKH/P2SH-P2WPKH/P2TR (the crate refuses P2PKH). `auto`: P2PKH→legacy, else→bip322. `--format legacy` on non-P2PKH → honest VerifyMessage error ("legacy signmessage verification is P2PKH-only; use --format bip322 / auto for segwit/taproot"). Legacy test vector MUST be P2PKH. Use `is_signed_by_address` for the P2PKH case (or recover_pubkey+compare); document the partition in the module + manual.
- **I1:** VerifyMessage between `UnknownHrp` and `XpubSearchNoMatch`; arms at exit_code 500-501 / kind 558-559 / message 732-743; no details arm.
- **I2:** `decode_address` uses `address_type().map(|t| t.to_string()).unwrap_or("unknown")` (Display, forward-compatible) — drop the 5-arm enumeration.
- **M1:** exact vec insert positions (convert/derive-child; verify-bundle/xpub-search-account); count comment →25.
- **M2:** rename test `tb1_hrp_valid_for_testnet_testnet4_signet`; loop includes Testnet4; document regtest is a distinct HRP.
- **M3:** decode_address `run<R,W,E>(args,stdin,stdout,stderr)` 4-arg uniform (ignore stdin).
- Dispositions (b)(c)(d)(e) folded verbatim into the plan; pin `bip322 = "=0.0.10"`.
Re-dispatching R1 via SendMessage to a81c65b57207226bb.
