# SPEC — `bundle --self-check` ms1 iteration (self-check-ms1-iteration-audit)

**FOLLOWUP:** `self-check-ms1-iteration-audit`.
**Source SHA (origin/master at write time):** `8b883dd`.
**Recon:** `cycle-prep-recon-api-harvest-drift-and-self-check-ms1.md` (gap CONFIRMED; FOLLOWUP citation drifted `:1478-1504`→`:2027`).
**Cycle type:** toolkit PATCH (tag-only). `--self-check` already exists → **no CLI surface change, no GUI `schema_mirror`, no manual mirror** (internal validation strengthening; a manual note is optional). No sibling-codec change.

---

## 1. Problem

`bundle.rs::self_check_bundle` (`:2027-2112`) re-parses a just-emitted bundle to confirm internal consistency. It validates **md1** (`reassemble` → wallet-policy) + **mk1** (`MkField::Single`/`Multi` decode + policy-id-stub linkage + privacy-preserving fingerprint) — but has **ZERO `bundle.ms1` reference** (recon-confirmed: grep over `:2027-2112` finds no ms1). So a regression in the per-slot ms1 emission rule (SPEC §5.8) — most dangerously a silent reversion to **@0-only emission** (`ms1[0]` populated, `ms1[1+]` wrongly `""`) for a full-mode multisig — would pass `--self-check` undetected.

## 2. Verified facts

- `Bundle.ms1: MsField` where `MsField = Vec<String>` (per-slot codex32 `ms1` string, or `""` watch-only sentinel).
- **Emission invariant** (`synthesize.rs:294-310`): for each cosigner `i`, `match &c.entropy { Some(e) => emit ms1 from e, None => "" }`. So a non-empty `ms1[i]` ⟺ slot `i` is phrase/entropy/ms1-bearing; `""` ⟺ watch-only. The emission-side invariant is already guarded by the test `synthesize_descriptor_emits_per_slot_ms1_for_phrase_bearing_slots` (`synthesize.rs:1383`); self-check is the *artifact-re-parse* guard for the same invariant.
- `self_check_bundle(&bundle, args: &BundleArgs)` — `BundleArgs` carries the raw `--slot`/`--language`/`--passphrase`/template CLI args, **not** the resolved per-slot entropy (that lived in the `CosignerKeyInfo` consumed during synthesis and is gone by self-check time).

## 3. Design (the check) — RESOLVED at R0 round 1

**Goal:** self-check must catch a regressed/corrupted ms1 emission, especially the @0-only multisig reversion.

**The oracle is `resolved_slots[i].entropy.is_some()` — NOT `args.slot`.** (R0 C1: `args.slot` records what the user *supplied*, not what drove emission, and false-rejects two valid shapes — `--import-json` envelopes that carry `ms1[i]!=""` with NO `--slot` arg, and `wif` slots which are `is_secret_bearing()` yet emit `ms1=""`. `resolved_slots[i].entropy.is_some()` is the EXACT predicate that drives the emit rule at `synthesize.rs:296`, so the check is identity-grounded against emission.)

**Signature change:** `pub fn self_check_bundle(bundle: &Bundle, args: &BundleArgs)` → `pub fn self_check_bundle(bundle: &Bundle, args: &BundleArgs, entropy_bearing: &[bool])`, where `entropy_bearing[i] = resolved_slots[i].entropy.is_some()`. Thread it from all FOUR call sites (each has the resolved slots in scope): `bundle.rs:411-414` (`resolved`), `:1614-1615` (`resolved_slots`), `:1657-1658` (`resolved_slots`), `:1909-1910` (`resolved_slots`).

**Length safety-belt (R0-r2 M1):** at function entry assert `entropy_bearing.len() == bundle.ms1.len()`; on mismatch return `Err(BundleMismatch { card: "self-check[ms1_length_mismatch]", … })` (the invariant holds structurally at every call site — both built from the same n cosigners — but this turns a future divergence into a clean error instead of an index panic).

**The check (single pass, supersedes the old Option 1/2 framing — combines decode-validity + emptiness-parity + entropy round-trip; the round-trip is free since `resolved[i].entropy` is already materialized, no secret re-read; verified clean for BOTH `Entr` and `Mnem` payloads — `Payload::as_bytes()` extracts identical entropy for each):** for each `i` in `bundle.ms1`:
- **Emptiness parity:** assert `ms1[i].is_empty() == !entropy_bearing[i]`. A mismatch (e.g. full-mode multisig with `ms1[0]` populated but `ms1[1+]==""`, the @0-only regression; or `ms1[i]` populated for a watch-only slot) → `BundleMismatch { card: "self-check[ms1_parity[i]]", … }`.
- if `!entropy_bearing[i]`: skip (legitimate `""` watch-only sentinel).
- else (`entropy_bearing[i]`): `ms_codec::decode(&ms1[i])` MUST succeed (valid codex32; self-describing Entr/Mnem, no wire-language needed — precedent `verify_bundle.rs:1316`) → extract the payload entropy → assert it **equals `resolved_slots[i].entropy.as_deref().unwrap()`** (the entropy round-trip). On decode failure or entropy mismatch → `BundleMismatch { card: "self-check[ms1_decode[i]]" / "[ms1_entropy[i]]", … }`.

This passes all legitimate shapes (watch-only all-`""`, full single-sig, full multisig, concrete-descriptor watch-only, hybrid import-json, wif-slot-`""`) and fails the @0-only regression + any corrupted/wrong-entropy ms1 (R0 verified-clean item 2).

## 4. Phasing / TDD
- **Phase 1 (RED):**
  - **RED cell** — start from a SYNTHESIZED valid full-mode multisig bundle (R0 M1: `self_check_bundle` runs `md_codec::chunk::reassemble(bundle.md1)` FIRST, so a hand-built dummy `Bundle` dies at md1_decode for the WRONG reason → vacuous; instead synthesize via `synthesize_descriptor` with the 3-distinct-mnemonic fixture from `synthesize.rs:1383`, then mutate `bundle.ms1[1] = String::new()` / clear `ms1[1+]`). Assert `self_check_bundle(&bundle, args, &entropy_bearing)` now returns `Err(BundleMismatch …)`. RED against current self-check (ignores ms1 → Ok). (`Bundle`/`MsField=Vec<String>`/`MkField` are constructible in bundle.rs's `#[cfg(test)]` BIN unit, but md1 must be real — hence synthesize-then-mutate.)
  - **GREEN guard cells (discriminating — R0 I1; Ok before AND after, but Err under the WRONG `args.slot` oracle, so they prove C1 is correctly resolved):**
    - **G-A:** `bundle --import-json <seeded-envelope, ms1[0]!="">  --self-check` (no `--slot` args) → **Ok**. (Corrected oracle: `resolved_slots[0].entropy.is_some()` set from the envelope at `bundle.rs:1782` → parity + entropy round-trip pass.)
    - **G-B:** `bundle --template bip44 --slot @0.wif=<WIF> --self-check` → **Ok**. (`resolved_slots[0].entropy.is_none()` → expect `""` → `ms1[0]==""` matches → pass.)
    - **G-C:** a correct full-mode multisig + a watch-only/descriptor (all `ms1==""`) bundle each self-check **Ok** (not over-eager).
- **Phase 2 (GREEN):** implement §3 in `self_check_bundle`. Full suite + clippy + (if any transcript exercises `--self-check`) `make -C docs/manual audit`. Per-phase opus review.
- **Phase 3 (ship):** PATCH version bump v0.47.3 → **v0.47.4** (toolkit crate; `--self-check` behavior strengthened — a tag-only release) + CHANGELOG + flip FOLLOWUP → ff-merge → tag `mnemonic-toolkit-v0.47.4` → push → watch CI.

## 5. R0 decisions (RESOLVED round 1)
1. **Oracle = `resolved_slots[i].entropy.is_some()`** (NOT `args.slot`). ✅ R0 C1 — `args.slot` false-rejects import-json + wif shapes. Thread `entropy_bearing: &[bool]` from the 4 call sites.
2. **Single combined check** (parity + decode-validity + entropy round-trip); the round-trip is free (entropy already in `resolved_slots`, no secret re-read) → strictly stronger than decode-only. ✅ R0 (supersedes Option 1/2).
3. **Decode entrypoint:** `ms_codec::decode(&str)` — self-describing, NO wire-language. ✅ R0 verified (precedent `verify_bundle.rs:1316`).
4. **SemVer:** PATCH + tag **v0.47.4**. ✅ R0-ratified (`self_check_bundle` is BIN-crate pub, not lib API; signature add internal; behavior strengthening).
5. **RED cell:** synthesize-then-mutate (md1 must be real). ✅ R0 M1. + discriminating G-A/G-B guards (R0 I1).
6. **No existing self-check test/transcript breaks** under the corrected oracle — R0 verified 7 tests + the bip84 transcript all pass unchanged.

## 6. Out of scope
- The emission side (already guarded by `synthesize_descriptor_emits_per_slot_ms1_for_phrase_bearing_slots`).
- verify-bundle's ms1 validation (separate user-supplied-card flow).
- Any `--self-check` CLI-surface change.
