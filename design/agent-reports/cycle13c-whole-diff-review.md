# WHOLE-DIFF REVIEW — cycle-13 Lane C (M1 + M7 + L18)

Reviewed-patch lane; this review is the gate. Worktree `wt-cycle13c`, off `origin/master = d55bf4c3` (v0.65.2). Commit-set `28ee63fb`/`fced9d03`/`7216ba5c`.

## VERDICT: GREEN (0 Critical / 0 Important)

### Gates
`cargo test -p mnemonic-toolkit` GREEN (194 ok, 0 failed; 9 new lane tests pass); `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo build --workspace` clean. Scope: exactly 3 src + 3 test + 2 fixture files; zero overlap with Lane A (`wallet_export/coldcard.rs`/`jade.rs`, `wallet_import/coldcard_multisig.rs`) or Lane B (`cmd/restore.rs`). No version-site churn, no new `ToolkitError` variant (reuses `ImportWalletParse`/`BadInput`/`Io`), no clap-flag/ValueEnum change → schema_mirror unaffected.

### M1 (`import_wallet.rs`) — sound
`account_from_origin_path` extracts the 3rd hardened component; all adversarial inputs (2-comp, 4-comp, unhardened, missing/empty, non-numeric) fall to `None → 0` safely, no panic; accepts `'`/`h`. Single-sig-only (`if n==1`); multisig stays `account:0` driving origins per-slot via `origin_path_bare()` (unaffected). Round-trip: `export-wallet --from-import-json` reads `envelope.bundle.account` (export_wallet.rs:869) → re-emits `m/84'/0'/5'` on sparrow+electrum (account-5 preserved, account-0 not regressed). Account-5 fixture is pre-existing (legit reuse).

### M7 (`bundle.rs`) — sound
Uses the SAME `extract_multisig_threshold(&tree)` as the card path (`bundle.rs:1214`); fallback `args.threshold.or(descriptor_threshold).unwrap_or(n)` parity-correct, cannot regress (Some(K) unchanged; None now reads real K, falls to n only when extraction empty). `path_family` left untouched (already-fixed sub-claim). Test: 2-of-3 → `threshold==2`, `cosigner_count==3`. A `--json` wire-VALUE change → not schema_mirror-gated (gui_schema tests reference the `--threshold` FLAG-name, not the output value); GUI `--json`-consumer paired-PR concern + optional manual prose.

### L18 (`electrum.rs`, highest scrutiny) — sound, fail-soft
- **Protocol fact verified** against live `spesmilo/electrum/keystore.py`: `BIP32_KeyStore.dump()` writes `get_derivation_prefix()`/`get_root_fingerprint()` (both `Optional[str]`); `from_xpub()` → `BIP32_KeyStore({})` leaves both `None` → JSON `null`. Fix targets exactly the "use a master key" watch-only flow; populated wallets keep strict 8-hex validation.
- **SLIP-132→script-type mapping verified** vs `slip0132.rs` version bytes (zpub→wpkh/84', ypub→sh-wpkh/49', neutral xpub→pkh/44', tr/86'; multisig Zpub→P2WSH/48'/…/2', Ypub→P2SH-P2WSH/1', neutral→P2SH).
- **Synthesized origin is derivation-irrelevant** (core safety property): per BIP-380 the `[fp/path]` bracket is key-origin metadata for PSBT/hardware matching only; addresses derive from `xpub + /<change>/<index>`. A synthesized (even wrong) origin CANNOT produce a wrong address — worst case a flagged PSBT-match miss. Both NOTICEs fire; `00000000` sentinel marks unknown-origin; descriptor shows inferred script type. Multisig: real cosigners keep real origins; a mixed-variant null cosigner is REJECTED (exit 2). Populated-wallet regression GREEN.

### Minor (non-blocking)
1. **NOTICE wording** (`electrum.rs:560,569`): for a neutral `xpub` + null derivation the wrapper falls to the conservative `pkh` (BIP-44) default, but the NOTICE says "inferring purpose from the SLIP-132 xpub prefix" when there's no prefix. Cosmetic only — the `pkh` default is pre-existing, output is watch-only, script type visible. **Worth a 1-line wording polish at integration.** Not a regression.
2. **fmt non-canonicity** at `bundle.rs:926` (M7's 4-line `let threshold`): `cargo fmt --check` would one-line it, but `bundle.rs` is already non-canonical on origin/master (`:3012` predates this lane), the repo forbids `cargo fmt --all` (mlock.rs/g6 exemption), no fmt CI gate. No action.

## Disposition
GREEN. Lane C clears the gate; HELD for integration into toolkit v0.66.0 (with Lanes A + B). Apply the Minor-1 NOTICE-wording polish during integration.
