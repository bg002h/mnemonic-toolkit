# Plan-R0 (toolkit bitcoind end-to-end oracle) round 2 — architect review (verbatim)

> Persisted verbatim per CLAUDE.md BEFORE the fold. **Verdict: NOT GREEN — 0 Critical / 1 Important.**
> The round-1 folds RESOLVED I-1 (shape-6 default-suite cell is placeable + non-vacuous) and
> I-2's architecture (independent golden breaks the circularity). But the I-2 fix prescribed a
> WRONG API expression → new Important. Controller fix: reuse the non-deprecated `derive_at_index`
> + `Result` handling (mirroring `derive_address.rs:49,:96` / STRESS-A `derive_receive` `:383-401`).

---

## Verbatim agent output

## Verdict: NOT GREEN — 0C / 1I

The two folds resolve the round-1 architecture findings (I-1's coverage gap is genuinely closed by a placeable, non-vacuous Phase-2a cell; I-2's circularity is genuinely broken by an independent rust-miniscript oracle). But the I-2 fold introduced a NEW Important defect by prescribing a concrete API expression that is both deprecated-under-`-D-warnings` AND non-compiling.

**I-1 (shape-6 default-suite coverage) — RESOLVED.** Both candidate host files exist; `cli_restore_multisig_general.rs` is the precise home (12 `bundle_general(...)→restore_json(...)` cells with `assert_md1_fixed_point`, e.g. `:115`,`:145`,`:254`). The gap is real (n=1 `wsh(and_v(v:pk,older(144)))` round-trip is exercised through `bundle` only at `cli_cross_tool_differential.rs:378`, never `restore`; the closest general cell puts a `multi` at the trunk, `:116`). Routing is sound: concrete-key n=1 v:pk bundle → `tlv.pubkeys` populated → `is_wallet_policy()` true → `run_multisig` (`restore.rs:177-178`) → wallet-policy gate (`:1232`) → `plain_template_from_tree` None (`:1154-1182`) → GeneralFaithful (`:1344`). Cell RED if shape-6 restore broke (C1 collapse class). No refusal applies. miniscript dep accessible.

**I-2 (independent golden) — RESOLVED IN PRINCIPLE; the literal API call is WRONG.** The independence argument is sound (separate code path from restore's internal derivation, mirrors STRESS-A O3 `derive_receive` `prop_backup_restore_roundtrip.rs:383-401`; Core is the real external oracle). BUT the prescribed expression has two faults (Problem A + B below).

### I-1 (NEW, from the I-2 fold) — the prescribed golden/independent-derivation API call breaks the plan's own clippy + compile gates.
`PLAN…:45` (also `:17`, referenced `:39`/`:49`/`:66`): `Descriptor::<DescriptorPublicKey>::from_str(WPKH_DESC).into_single_descriptors()[0].at_derivation_index(0).address(Network::Bitcoin)`
- **Problem A — `at_derivation_index` is `#[deprecated(since="13.0.0", note="use derive_at_index…")]`** in the pinned fork (`Cargo.toml:28-29` git rev `95fdd1c`). Toolkit + STRESS-A use `derive_at_index` (`derive_address.rs:49,:96`; `prop_backup_restore_roundtrip.rs:393`). CI `cargo clippy --all-targets -- -D warnings` (`rust.yml:199`) compiles integration tests + the Phase-2a cell and denies `deprecated` → HARD-FAIL of the plan's own §4 + §7-Phase-2 gates. (The only other `at_derivation_index` mention, `cli_export_wallet_bsms.rs:243`, is a `///` doc-comment, not live code.)
- **Problem B — `into_single_descriptors()` returns `Result<Vec<Descriptor>, Error>`**, so `…()[0]` cannot index; `derive_at_index`/`at_derivation_index` return `Result<…, NonDefiniteKeyError>`, so `.address(…)` cannot chain. As written it does not compile.

**Fix (mechanical):** prescribe reuse of STRESS-A's helper `derive_receive(WPKH_DESC, 1)[0]` (`prop_backup_restore_roundtrip.rs:383-401`) — exactly round-1's I-2 fix text — OR the corrected expression:
`Descriptor::<DescriptorPublicKey>::from_str(WPKH_DESC).unwrap().into_single_descriptors().unwrap().remove(0).derive_at_index(0).unwrap().address(Network::Bitcoin).unwrap().to_string()`.
Apply at every occurrence (`:17`,`:45`) + align the `:39`/§7 prose.

**M-1..M-4 folds — all sound.** M-1: origin opaque metadata (depth byte from script-context `n`, not origin label, `parse_descriptor.rs:769-849`); verbatim `/48h/0h/0h/2h` accepted by bundle for wpkh/tr watch-only. M-2: companion + sequencing cite holds (`FOLLOWUPS.md:4178,:4186`). M-3: `cross-tool-differential.yml` exists. M-4: process step.

**Round-1 TRUE verifications still hold:** restore JSON shape (`restore.rs:1661-1666`); FOLLOWUP open (`:4178`); miniscript pinned via `[patch.crates-io]`; clippy/fmt gates.

Fix this one snippet, re-grep-verify, persist, re-dispatch — it is the only remaining gate.
