# R0 Review (Round 2) — SPEC_restore_multisig_cosigner.md (v0.44.0)

**Reviewer:** opus `feature-dev:code-reviewer` (R0 re-dispatch after R0-r1 fold)
**Date:** 2026-06-04
**Source SHA:** `fd0c89c`
**Verdict:** 0 Critical / 0 Important / 4 Minor — **GATE: GREEN**

> Persisted verbatim per CLAUDE.md. Fold note appended (all 4 Minors folded this round).

---

## Critical

None.

## Important

None. All three round-1 Important findings are correctly and completely resolved against source:

- **I1 (lint_argv_secret_flags — NO new route):** RESOLVED. §7 now states "NO new route" with the correct rationale: the gate is three set-equality closures (`flag_axis`/`from_axis`/`slot_axis`) over the gui-schema `secret==true` surface. Verified against `tests/lint_argv_secret_flags.rs:184-223` (the three `*_set_equals_gui_schema` tests) — adding `--md1`/`--cosigner` to `FLAG_ROUTES` would make `declared ⊋ live` and FAIL. `--md1`/`--cosigner` are non-secret: `flag_is_secret` (`secrets.rs:49-64`) does not list either, so both fall through to `false`. The SPEC no longer instructs adding a route.

- **I2 (network reconstruction):** RESOLVED and implementable, not hand-waved. §4 step 4 mandates building each cosigner `Xpub` with `network.network_kind()` from `--network`. Confirmed the toolkit ALREADY constructs `bitcoin::bip32::Xpub { network, depth, parent_fingerprint, child_number, public_key, chain_code }` as a struct literal with public fields in production code (`synthesize.rs:949-956`, `bip85.rs:150`, `wallet_import/bsms_round1.rs:64`). `bitcoin = 0.32.8`. So restore can populate the struct directly from the 65-byte `[chain_code‖compressed_pubkey]` (`chain_code: ChainCode::from(bytes[0..32])`, `public_key: PublicKey::from_slice(bytes[32..65])`) + `network: net.network_kind()` + depth-0 defaults — no builder needed. md-codec's `xpub_from_tlv_bytes` hardcoding `Main` + being `pub(crate)` is consistent. §8 Phase 1 cell #9 (testnet → `tpub`) guards it.

- **I3 (--from optionality):** RESOLVED. (a) `required_unless_present = "md1"` is valid clap-derive syntax referencing the FIELD name — confirmed against in-repo usage at `verify_bundle.rs:29,82,89`. The new `--md1` field is named `md1` (single token), so `"md1"` is correct. (b) Both consumption sites are real — `restore.rs:154` (`parse_from_input(&args.from)`) and `restore.rs:209` (`args.from.split('=')`); handling the `Option` is sound. (c) The GUI `--from` at `mnemonic-gui/src/schema/mnemonic.rs:356-366` IS the restore one — preceded by the restore comment block with `required: true` (`:359`), distinct from import/export-wallet `--from` entries.

Minors M1 (xpub_to_65 → `synthesize.rs:98` ✓), M2 (per-`@N` origin ✓), M3 (65-byte position inference ✓), M5 (Phase-1 depth-0 assert ✓ cell #10), M6, M7 all folded into §3/§4/§8 without contradiction. `RestoreMismatch` fields confirmed at `error.rs:279-284`. `extract_multisig_threshold` at `bundle.rs:1015`; `build_descriptor_string` signature at `pipeline.rs:18` matches the SPEC call. `ExpandedKey` fields confirmed (`canonicalize.rs:338`).

## Minor

- **§8 Phase 3 contradicts the I1 fold (`:89` vs `:79`).** Phase 3 lists "`lint_argv_secret_flags` routes" parallel to two real edits, reading as "edit the routes." §7 (the I1 fold) says the opposite ("NO new route"). Stale pre-fold wording. **Fix:** reword §8 Phase 3 to "verify `lint_argv_secret_flags` set-equality tests still pass (NO route change — I1)."
- **§8 Phase 3 omits the I3 GUI `--from` flip (`:89` vs `:77`).** Phase 3 says "+`secret:false` for both" but omits the `--from required:true→false` flip §7/I3 mandates; `schema_mirror` is flag-name-only so the SPEC prose is the sole safeguard. **Fix:** add "flip GUI restore `--from` `required`→false (I3)" to §8 Phase 3.
- **§3 citation off-by-one.** Cites the `from` field at `restore.rs:57`; `pub from: String` is at `:58` (`:57` is the `#[arg(long)]` attr). Point at `:57-58`.
- **`ExpandedKey.xpub` is `Option<[u8;65]>`, not bare.** §4 refers to "the 65-byte `ExpandedKey.xpub`" as if non-optional; the field is `Option<[u8;65]>` (`canonicalize.rs:338`). The `is_wallet_policy()` gate guarantees `Some` — say so, and handle `None` defensively (error, not `unwrap`).

These are documentation-consistency Minors (normative §7/§3/§4 fold text is correct, implementability crux sound). The first two are fold-introduced drift in the implementer's action list and should be folded this round.

VERDICT: 0 Critical / 0 Important
GATE: GREEN

The gate is GREEN (0C/0I) — implementation may proceed. Recommend folding the two §8 Phase 3 Minors before coding (§8 is the implementer's checklist; quick prose sync, not a re-review trigger).

---

## Fold note (applied after persisting)

- **§8 Phase 3 Minor 1 & 2 — FOLDED.** Phase 3 reworded: GUI adds `--md1`/`--cosigner` (`secret:false`) AND flips restore `--from required:true→false` (I3); `lint_argv_secret_flags` = NO route change, verify set-equality tests pass (I1).
- **§3 citation Minor — FOLDED.** `restore.rs:57` → `:57-58`.
- **`ExpandedKey.xpub` Option Minor — FOLDED.** §4 step 2 now notes `xpub`/`fingerprint` are `Option`, the `is_wallet_policy()` gate guarantees `Some`, reconstruction handles `None` defensively (no `unwrap`).
- R0 GATE GREEN at 0C/0I — implementation unblocked.
