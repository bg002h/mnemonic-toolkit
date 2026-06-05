# mnemonic restore — Implementation-Plan R0 Review

**Verdict: GREEN (0 Critical / 0 Important).** Cleared to implement. 5 sketch-polish Minors folded into the plan.

The plan is implementable as written. Every load-bearing API, signature, field name, visibility, and test vector checked is real and current. The riskiest step (Task 1.4 watch-only `ResolvedSlot` → `build_descriptor_string`) is backed by a real, callable, tested path; the reviewer reproduced the exact claimed descriptor (`#hpg6d6w2`) and both fingerprints (`73c5da0a`/`b4e3f5ed`) at runtime.

## Critical / Important
None.

## Minor (folded into the plan)
1. **`kind()` PascalCase.** Neighbor arms return the PascalCase variant name (`error.rs:531-582`); `kind_strings_stable` (`:1182`) enforces it. Plan now says `"RestoreMismatch"` (not kebab). FOLDED.
2. **`main.rs` dispatch bare args.** `stdin/stdout/stderr` are already `&mut` (`:149-151`); pass bare (`:156` precedent), no spurious `&mut`. FOLDED.
3. **`ResolvedSlot` no `Default`.** Spell all 7 pub fields (watch-only ctor `wallet_import/pipeline.rs:200-208`): `{ xpub, fingerprint, path, entropy:None, master_xpub:None, language:None, _entropy_pin:None }`. FOLDED.
4. **P2 `EmitInputs.script_type` is `WalletScriptType`** (`mod.rs:165`), distinct from P1's `convert::ScriptType` — use `wallet_export::script_type_from_template` (`mod.rs:193`); ctor template `export_wallet.rs:483-500`. FOLDED.
5. **ms1 conflict message `slot @0.ms1=`** — cosmetic (slot_index only in the message string, not derivation), already noted SPEC §3.1 M-idx.

## Verification ledger (highlights — all RAN against `ccc9321`, base `6566941`)
- **End-to-end (runtime):** watch-only slot → `build_descriptor_string` → `wpkh([73c5da0a/84'/0'/0']…/<0;1>/*)#hpg6d6w2` (byte-exact via `export-wallet --slot @0.xpub/fingerprint --template bip84 --format descriptor`); `73c5da0a` (no-pp) / `b4e3f5ed` (TREZOR-pp) both confirmed path-independent.
- `RestoreMismatch` alpha slot RepairShortCircuit→SilentPayment in enum + 3 forced-exhaustive blocks (`error.rs:471/529/588`); `details()` `_=>None` @775/798 (no arm needed). ACCURATE.
- `ResolvedSlot` all-pub fields, hand-constructible, no Default; watch-only ctor `wallet_import/pipeline.rs:200-208`. ACCURATE.
- `convert::script_type_from_template` `convert.rs:393` private (pub(crate) bump); `derive_bip32_from_entropy` `derive_slot.rs:42`; `resolve_ms1_slot` `slot_ml1.rs:37` (`.entropy`/`.derive_language`); `render_address_from_xpub` `address_render.rs:18`; `Secp256k1::verification_only()`+`derive_pub` watch-only (`addresses.rs:232-244`); `CliTemplate::is_multisig` `template.rs:47`; `ModeViolation`→2/`BadInput`→1; reusable `--from` parser `convert.rs:136`; `is_argv_secret_bearing` `:117`; advisory/env/mlock/stdin-mutex all ACCURATE.
- gui-schema 28-name hardcoded vec (not `.len()`); "28" at `:74`/`:108`; `restore` alpha slot correct.
- TDD soundness: P1 leaves nothing un-compilable (unused `#[non_exhaustive]` variant is fine); `b4e3f5ed` re-derived-in-test (not asserted-from-memory) + confirmed correct at runtime; ms1 vectors obtainable via `convert --to ms1`. Phase split clean (P1 independently green).

**Bottom line:** 0C/0I. Cleared to implement P1.
