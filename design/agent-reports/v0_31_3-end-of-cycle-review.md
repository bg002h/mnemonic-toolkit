# v0.31.3 end-of-cycle architect review

**Reviewer:** opus (feature-dev:code-reviewer)
**Round:** end-of-cycle
**Cycle:** Cycle 10 (seedqr-bundle-slot-integration)
**Date:** 2026-05-21
**Pre-tag SHAs reviewed:**
- Phase 2 (slot_input.rs lib extension): `8a6892b`
- Phase 3a (bundle.rs consumer + map_seedqr_error promotion): `f9cc176`
- Phase 3b (verify_bundle.rs + export_wallet.rs): `251495a`
- Phase 4 (manual mirror): `2dbb0d0`
- Phase 4b (master_xpub clap-help drift touch-and-fix): `43b507c`
- Clippy fix: `3ff584c`
- Phase 5 (uncommitted on disk at review time): Cargo.toml 0.31.2→0.31.3 + install.sh self-pin + CHANGELOG entry

## Verdict

**GREEN.** 0 Critical / 0 Important / 0 Minor across all 12 verification items.

## Verification matrix

1. **SlotSubkey enum-order** — VERIFIED. `Seedqr` at position 1 (`slot_input.rs:28`, between `Phrase:18` and `Entropy:29`). `is_legal_set` arms `[Seedqr]`, `[Seedqr, Path]`, `[Seedqr, Fingerprint, Path]` ascending-sorted. `exempted_v0_19_0` matcher includes both Phrase+Seedqr variants.
2. **map_seedqr_error promotion** — VERIFIED. `pub(crate)` at `cmd/seedqr.rs:62`. Reused at `cmd/bundle.rs:454` and `cmd/verify_bundle.rs:738`.
3. **Bundle.rs branch placement + lifetime** — VERIFIED. New unified Phrase+Seedqr branch at `cmd/bundle.rs:440-486`, before Xpub branch at `:487`. `decoded_phrase: String` extends lifetime correctly.
4. **Path-override extension** — VERIFIED. `cmd/bundle.rs:1138-1140` and `cmd/verify_bundle.rs:671-673` both gate via `Phrase || Seedqr` predicate.
5. **Export-wallet refusal** — VERIFIED. `wallet_export/mod.rs:107-121::validate_watch_only` includes `SlotSubkey::Seedqr`. `REFUSAL_SECRET_INPUT` canonical text routed via `ExportWalletSecretInput` (exit 2).
6. **Test coverage** — VERIFIED. 15 net cells: 6 bundle + 2 verify-bundle + 1 export-wallet refusal + 6 lib unit. Byte-equal assertions present on BOTH 12-word and 24-word happy paths (M1 fold).
7. **secret_taxonomy parity** — VERIFIED. `SECRET_SLOT_SUBKEYS` at `secret_taxonomy.rs:90` includes `"seedqr"`. `declare_slot_subkey_variants!` macro includes Seedqr.
8. **Manual chapter mirror** — VERIFIED. All three consumer sections enumerate `seedqr` (`41-mnemonic.md:44,497,652`). `master_xpub` drift backfilled at same three sites (M2 fold).
9. **SemVer PATCH** — VERIFIED. `Cargo.toml:3 = "0.31.3"`. R0 I1 rationale correctly cited.
10. **install-pin-check self-pin** — VERIFIED. `scripts/install.sh:32` = `mnemonic-toolkit-v0.31.3`.
11. **CHANGELOG completeness** — VERIFIED. Entry accurate, cites Cycle 10 FOLLOWUP closure, documents R0 3C/2I/2M folds + R1 GREEN + end-of-cycle GREEN; mentions follow-on `gui-seedqr-slot-subkey-help-mirror`.
12. **Architectural integrity** — VERIFIED. `import_wallet.rs:163` narrows `--slot` to phrase-only via `value_name = "@N.phrase=<phrase>"` intentionally; no other orthogonal per-`@N.X=` dispatch site exists.

## Cleared for tag

Proceed with the version-bump commit + `mnemonic-toolkit-v0.31.3` tag.
