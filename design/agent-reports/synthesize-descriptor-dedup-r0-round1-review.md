# R0 Architect Review (round 1) — `SPEC_synthesize_descriptor_dedup.md`

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory pre-implementation R0 gate). **Date:** 2026-06-06.
**Branch:** `synthesize-descriptor-dedup` (off master `6506948`). **Verdict:** **0 Critical / 1 Important** (+ 1 Minor). NOT GREEN.

> Persisted verbatim per CLAUDE.md BEFORE the fold. The refactor body is byte-identical (verified line-by-line); the gap is a missing multisig byte-shape guard (I1). Fold → re-dispatch.

---

## VERDICT: 0 Critical / 1 Important / 1 Minor — NOT GREEN

The refactor is sound on the merits — both back-halves' success-path output bytes are genuinely identical. But the SPEC's "no RED cell, existing cells guard it" is **hollow for the multisig (n>1) branch** — the most complex, least-covered path. One characterization cell must be captured (Phase 1) before the edit.

---

## IMPORTANT

### I1 — No byte-exact golden guards the n>1 `MkField::Multi` branch; the no-RED decision is unjustified for multisig
- The byte-exact golden `bundle_full_16_cells_byte_exact_against_pinned_vectors` (`cli_bundle_full.rs:14-37`, frozen `tests/vectors/v0_1/*.txt`) covers **n==1 only** (`bundle --slot @0.phrase=… --template {bip44/49/84/86}`). The `Single` branch (`synthesize.rs:854-868`) is well-guarded.
- The `synthesize_unified_*` multisig cells (`synthesize.rs:1637-1693`) assert only `ms1.len()`, `starts_with("ms1")`/`is_empty()`, `any_secret_bearing()` — NOT mk1/md1 contents nor exact ms1 bytes. They'd stay green even if Multi emitted structurally different cards.
- `bundle | verify-bundle` round-trips + `--self-check` CO-MOVE: both emit (`bundle.rs:399`) and verify (`verify_bundle.rs:374/464/568`) call `synthesize_unified`, so a delegation that changed Multi output changes both sides identically and still passes — not an independent guard.
- The byte-exact multisig self-check cell that WOULD have caught it (`bundle_self_check_passes_for_canonical_seed_multisig`) was **DELETED in v0.4.2** (`cli_self_check.rs:34-36`); its fixture `tests/vectors/v0_2/wsh-sortedmulti-mainnet-0-false-true.txt` (+ all `v0_2` multisig vectors) is now **orphaned** (only the single-sig `bip84-mainnet-0-false-true.txt` is read by any test).

**Net:** the `MkField::Multi` branch (`synthesize.rs:869-889`, `stubs.clone()` per-cosigner, `csi = derive_mk1_chunk_set_id(&xpub.fingerprint().to_bytes())`) — the branch this refactor most needs to prove behavior-preserving — has NO frozen byte-shape characterization.

**Fix:** Add ONE Phase-1 characterization cell pinning current `synthesize_unified` n>1 output as a frozen golden, captured BEFORE the edit. A 2-of-2 multisig with two distinct phrases (`TREZOR_12_ZERO` + `BIP39_TEST_2`, per `cli_descriptor_mode.rs:245-247`) satisfies BIP-388 distinctness. Assert byte-exact on ms1[0..1] + mk1[0..1] + md1 so any csi/ordering/stub drift goes RED. Phase 1 = one multisig characterization cell, NOT zero.

---

## MINOR

### M1 — Back-halves differ in statement ORDER (immaterial to output)
`synthesize_unified` computes ms1 → mk1 → md1 (`:827/:853/:891`); `synthesize_descriptor` md1 → mk1 → ms1 (`:247/:249/:294`). The three `Bundle` fields are independent + iteration order over the slice is identical → byte-identical `Bundle`. Only observable difference: which internal encoder error surfaces first if two failed simultaneously (reachable only on a freshly-built `is_wallet_policy`-asserted descriptor → practically never). One-line SPEC note; non-blocking.

---

## What verified CLEAN (line-by-line)

**Item 1 — back-halves byte-identical (success path):**
- policy_id/stub: `compute_wallet_policy_id` → `[..4]` (descr `:243-245` ≡ unif `:819-821`).
- ms1: identical Entr/Mnem branch, `language.unwrap_or(run_language)`, `""` for None, `Tag::ENTR`, same iteration order (descr `:295-313` ≡ unif `:828-851`).
- mk1 Single: `KeyCard::new(vec![stub], privacy?None:Some(fp), mk1_origin_path, xpub)`, csi `derive_mk1_chunk_set_id(&stub)` (descr `:249-263` ≡ unif `:854-868`).
- mk1 Multi: `stubs.clone()`, csi `derive_mk1_chunk_set_id(&xpub.fingerprint().to_bytes())` — uses the xpub's COMPUTED fingerprint, not the slot `.fingerprint` field; identical in both (descr `:265-284` ≡ unif `:869-889`).
- md1: both `md_codec::chunk::split(descriptor)` (descr `:247` ≡ unif `:891`).

**Item 2:** `pub type CosignerKeyInfo = ResolvedSlot;` (`:219`); both back-halves read only `.entropy/.language/.fingerprint/.xpub/.path` — neither reads `master_xpub`/`_entropy_pin`.

**Item 3:** `descriptor.n = n = slots.len()` (`:753,:803`) → delegated count-check (`:236`) passes; `debug_assert!(is_wallet_policy())` holds (template-built).

**Item 4 — no front-half value dropped:** front-half locals (`origin_paths`/`path_decl_paths`/`tree`/`fingerprints`/`pubkeys`, `:780-800`) feed ONLY the `Descriptor` (`:802-817`); the back-half reads only `descriptor`, `n` (∈ descriptor.n), `stub`/`stubs` (recomputed by synthesize_descriptor), the slot fields, `privacy_preserving`, `run_language`. Descriptor fully captures what the delegate needs.

**Item 5 — `run_language`:** unified passes its own param through; both resolve `{c,s}.language.unwrap_or(run_language)`.

**Item 7 — SemVer PATCH correct** (no signature/clap/error-variant change; ~9 call sites signature-stable; no lockstep). The no-RED decision is defensible for n==1 (frozen golden) but NOT n>1 — see I1.

---

**Next:** fold I1 (Phase 1 = multisig characterization cell, captured pre-edit) + note M1 → persist → re-dispatch. No §2 code edit until the multisig golden exists + R0 re-converges 0C/0I.
