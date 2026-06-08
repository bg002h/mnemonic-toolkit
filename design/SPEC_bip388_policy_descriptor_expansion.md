# SPEC — BIP-388 wallet-policy → concrete-descriptor expansion on `--descriptor`

**Repo:** `mnemonic-toolkit`. **Resolves:** `bip388-wallet-policy-to-descriptor-expansion-not-surfaced` (FOLLOWUPS.md).
**Date:** 2026-06-08. **Source SHA:** `origin/master` == local `HEAD` == `053cc1c`.
**Disposition:** toolkit **MINOR → v0.49.0**, tag-only (`mnemonic-toolkit-v0.49.0`). **No new clap flag / value / subcommand → NO GUI `schema_mirror` lockstep.** Manual: a prose addition documenting the new accepted `--descriptor` input shape (no flag-coverage delta). (Reclassified PATCH→MINOR post-impl-review per advisor + precedent: a new accepted input FORMAT on an existing flag is a new capability, matching `addresses --from electrum-phrase` v0.47.0 + `export-wallet --format descriptor` v0.42.0, both MINOR. The persisted agent-reports predate this and still say "PATCH / v0.48.1" — verbatim snapshots, not re-edited.)
**Direction:** architect-decided **option (a)** (`(a) >> (c) > (b)`; consult persisted in `cycle-prep-recon-bip388-wallet-policy-to-descriptor-expansion.md` "Architect direction"). (b) dropped (seed-gated); (c) already covered by `--format descriptor` once (a) lands.

---

## 0. Problem

The toolkit converts **concrete descriptor → BIP-388 wallet policy** (`export-wallet --descriptor <concrete> --format bip388` → `{name, description_template (@N/**), keys_info[]}`, emitter `wallet_export/pipeline.rs:166-211`) but has **no user-facing inverse**. Users round-trip this shape "fairly often" (user signal). The inverse expander **already exists** (`cmd/xpub_search/descriptor_intake.rs::parse_bip388_json` `:199-222` — substitutes `@N/**` → `keys_info[N] + "/<0;1>/*"`, concrete string built at `:218`; declared "exact inverse of `pipeline.rs:192-198`"; unit cells `:438-458`) but is reachable **only inside `xpub-search`**, which consumes it to locate a seed's cosigner role and **never echoes the descriptor**. And `export-wallet`/`bundle --descriptor` **refuse** the policy/`@N` shape.

## 1. The fix — one shared expander + a pre-check at two existing call sites

No new CLI surface. Auto-detect the leading-`{` BIP-388 policy on the **existing `--descriptor`** flag, expand to a concrete descriptor string, then fall into the unchanged concrete-descriptor pipeline. Closes the round-trip at the same surface that emits policies:
- `--format bip388` (forward) ↔ `--format descriptor` (string inverse) / `--format bitcoin-core|sparrow|specter|…` (artifact inverse) / `bundle --descriptor` (watch-only md1 inverse).

### 1.1 Extract the pure substitution into a shared module (Phase 1 — dedup, no happy-path behavior change)

In **`crates/mnemonic-toolkit/src/wallet_import/pipeline.rs`** (already `pub(crate)` home of `is_at_n_form` `:115`, `classify_descriptor_form` `:132`, `DescriptorForm` `:121`; already imported by both `export_wallet` and `bundle`):

```rust
/// Strict BIP-388 wallet-policy schema (mirrors the emitter at
/// `wallet_export/pipeline.rs:166-211`). `deny_unknown_fields`.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct BipPolicyJson {
    // Deserialized-but-unread: the `_name` underscore silences `dead_code`;
    // the `#[serde(rename = "name")]` is LOAD-BEARING — without it,
    // `deny_unknown_fields` would reject the real `"name"` JSON key and demand
    // a `"_name"` key, breaking every real policy (R0-r1 I-1 trap).
    #[serde(rename = "name")]
    _name: String,
    description_template: String,
    keys_info: Vec<String>,
}

/// True iff `s` (trimmed) begins with `{` — the BIP-388 policy-JSON sniff.
pub(crate) fn is_bip388_policy_shape(s: &str) -> bool { s.trim_start().starts_with('{') }

/// Expand a BIP-388 wallet-policy JSON into a concrete multipath descriptor
/// STRING by substituting each `@N/**` → `keys_info[N] + "/<0;1>/*"`. The exact
/// inverse of the emitter's `@N/**` substitution (`pipeline.rs:190-204`).
/// Ordering note: the expander replaces longest-N-FIRST by **digit-count**
/// (`@10` before `@1`); the emitter sorts by full-key-**string-length** — each
/// correct for its direction. (Over-defensive here since `/**` is part of every
/// replaced token, so `@1` can never be a substring of `@10/**` — but mirror
/// the existing code; do NOT "simplify" it. R0-r1 M-5.)
pub(crate) fn expand_bip388_policy(json: &str) -> Result<String, ToolkitError>
```

`expand_bip388_policy` body = the **parse + substitution** at `descriptor_intake.rs:205-218` (serde parse `:205-209` + the `@N/**` replace loop `:210-218`), minus the trailing `parse_literal_xpub` call `:219` (pure string-in/string-out; no network/account/stderr). **Post-substitution validation:** if `is_at_n_form(&expanded)` is still true (template referenced an `@N` index ≥ `keys_info.len()`), return `ToolkitError::DescriptorParse("BIP-388 policy template references @N beyond keys_info[..]")` rather than emitting a half-substituted string into the downstream parser.

Then **refactor `descriptor_intake::parse_bip388_json` to delegate**: `let template = crate::wallet_import::pipeline::expand_bip388_policy(payload)?; let mut intake = parse_literal_xpub(&template, network, account, stderr)?; intake.shape = DescriptorShape::Bip388Json; Ok(intake)`. The `BipPolicyJson` struct + substitution loop move out of `descriptor_intake.rs`. **Single-sources the inverse-of-emitter lockstep.** xpub-search behavior is **unchanged on the happy path**; the malformed-`@N` error path **strictly improves** (the new earlier, explicit message replaces a downstream miniscript parse error). NOTE (R0-r1 I-2): `descriptor_intake.rs:438-466` are `detect_shape`-only cells — they do NOT pin `parse_bip388_json`'s substitution output; the only pre-existing expansion coverage is the integration happy-path `cli_xpub_search_account_of_descriptor.rs::account_of_descriptor_bip388_json_match` (single-key `wpkh(@0/**)`). The Phase-1 `expand_bip388_policy` unit cell is what **first** pins the substitution output (a coverage GAIN), and a Phase-1 cell pins the improved malformed-`@N` message (verified: no existing cell asserts the old string, so nothing breaks).

### 1.2 `export-wallet --descriptor` pre-check (Phase 2)

At `crates/mnemonic-toolkit/src/cmd/export_wallet.rs` **before** the `is_at_n_form` refusal at `:412`:
```rust
// BIP-388 wallet-policy JSON intake: expand to a concrete descriptor, then
// fall into the existing concrete passthrough. MUST precede is_at_n_form —
// a raw policy JSON matches the @N probe (its description_template) AND the
// key_regex probe (its keys_info), so without this it trips the :413 refusal.
let desc_owned;
let desc = if crate::wallet_import::pipeline::is_bip388_policy_shape(desc) {
    desc_owned = crate::wallet_import::pipeline::expand_bip388_policy(desc)?;
    &desc_owned
} else { desc };
```
then the unchanged `is_at_n_form` guard (`:412`) + miniscript passthrough (`:418-421`) see a concrete descriptor. Preserve the deliberate `@N`-probe (NOT `classify_descriptor_form`) per the `:410` comment.

### 1.3 `bundle --descriptor` / `--descriptor-file` pre-check (Phase 2)

At `crates/mnemonic-toolkit/src/cmd/bundle.rs:303-305`, **before** `classify_descriptor_form(&body)` (`:304`):
```rust
let body = if crate::wallet_import::pipeline::is_bip388_policy_shape(&body) {
    crate::wallet_import::pipeline::expand_bip388_policy(&body)?
} else { body };
```
The expanded body is an origin-annotated watch-only multipath concrete descriptor → `classify_descriptor_form` returns `Concrete` → `bundle_run_concrete_descriptor` (`:1634`) → `descriptor_concrete_to_resolved_slots` (`:1643`) → `any_secret == false` → **watch-only** (`MultisigWatchOnly` for n≥2 cosigners `bundle.rs:1660`; `SingleSigWatchOnly` for n=1 `bundle.rs:1657` — both correct, secret-free; R0-r1 M-1). A policy carries no secrets → watch-only md1 + per-cosigner mk1, ms1 omitted.

**Scope note (R0-r1 M-2):** the **bundle** path requires **origin-annotated** `keys_info` (`[fp/path]xpub`) — `classify_descriptor_form` returns (false,false) → errors "...must carry a key origin..." (`wallet_import/pipeline.rs:141-145`) — classify fires before the slot-resolver on a bare-xpub policy (origin-less `keys_info` is legal per BIP-388 but unsupported by the bundle slot-resolver). **export-wallet** has no such requirement (miniscript parses bare-key concrete descriptors). Emitter-produced policies always carry origins, so the §6 happy-path fixtures are clean; a Phase-2 negative cell pins the bundle bare-key refusal.

## 2. THE critical invariant — auto-detect ordering

A raw BIP-388 policy JSON satisfies **both** probes: `at_n_probe` (`@\d`, matches `@0/**` in `description_template`) AND `key_regex` (`[fp/path]xpub`, matches `keys_info`). Verified: `classify_descriptor_form(raw policy)` → `(true,true)` → `DescriptorParse("descriptor mixes @N placeholders with inline keys")` (`wallet_import/pipeline.rs:136-138`); `is_at_n_form(raw policy)` → `true` → export-wallet `:413` refusal. **Therefore `is_bip388_policy_shape` (leading `{`) MUST be checked FIRST at both sites, before `is_at_n_form`/`classify_descriptor_form`.** Do NOT fold the policy branch into `classify_descriptor_form` and rely on fall-through. The four `--descriptor` shapes resolve, in order: leading `{` → **policy** (expand); else `@\d` → **AtN** (existing refusal/unified path); else `[fp/path]xpub` → **Concrete**; md1 HRP is **not** an export-wallet/bundle `--descriptor` input (that funnel is xpub-search-only) → falls to miniscript parse-failure today (clearer error = out-of-scope fast-follow).

## 3. Round-trip fidelity

`policy → concrete → --format bip388` is **byte-stable for `description_template` + `keys_info`** (modulo (i) checksum normalization — emitter strips `#…` at `pipeline.rs:196`, miniscript re-adds, emitter re-strips; (ii) `sortedmulti` ordering, canonicalized by miniscript on first emit, stable thereafter). It is **LOSSY on `name`** — PRE-EXISTING, not a regression: the expander discards the policy name (currently `_name`, `descriptor_intake.rs:87`) AND the emitter hardcodes `"name": "imported-descriptor"` (`pipeline.rs:207`). The SPEC documents this; honoring a round-tripped `name` (e.g. `export-wallet --wallet-name`) is an **out-of-scope fast-follow** FOLLOWUP, not a blocker.

## 4. The `/**` re-lex concern is a NON-issue for option (a)

Expansion yields `/<0;1>/*` (standard multipath), fed to `miniscript::Descriptor::from_str` (`export_wallet.rs:421`) / `key_regex` (`bundle`), **never** the toolkit's own single-star lexer (`parse_descriptor.rs:70`, which refuses a literal `/**` — the canonicity-drift `ParseFails` rows, cf. `project_canonicity_drift_per_fixture_table_shipped`). A literal `/**` is never emitted by this path. Stated explicitly so a reviewer doesn't re-raise it.

## 5. SemVer / lockstep

**MINOR (v0.49.0)** — a new accepted input FORMAT on `--descriptor` is a new capability (precedent: electrum-phrase v0.47.0, format-descriptor v0.42.0). No clap flag-NAME / value-enum / subcommand change → **no GUI `schema_mirror` lockstep, no `docs/manual/src/40-cli-reference/` flag-coverage delta.** Manual: add a short prose note under the `export-wallet`/`bundle` `--descriptor` description (and/or a recovery-recipe line) that `--descriptor` now accepts a BIP-388 wallet-policy JSON and round-trips with `--format bip388`. No sibling-codec companion (the substitution is toolkit-local string work; md-codec already parses/serializes the resulting concrete descriptor).

## 6. Test plan (per-phase TDD; tests RED before impl)

**Phase 1 (shared helper + dedup):**
- `wallet_import::pipeline` unit cells: `expand_bip388_policy` on a known policy → exact concrete multipath string; `deny_unknown_fields` rejects an extra key; `@N` longest-first (a 11-key policy: `@10` not clobbered by `@1`); post-substitution `@N`-beyond-keys_info → `DescriptorParse`; `is_bip388_policy_shape` ` {…`/leading-ws true, `wsh(…)`/`md1…` false. (These FIRST-pin the substitution output — the pre-existing `descriptor_intake.rs:438-466` cells are `detect_shape`-only.)
- xpub-search regression: existing `descriptor_intake`/`account-of-descriptor` happy-path cells still GREEN after delegation. **+ new cell** pinning the xpub-search malformed-`@N` path now exits non-zero with the new `"…@N beyond keys_info…"` message (I-2(c) — pins the improved error so it doesn't silently regress).

**Phase 2 (export-wallet + bundle wiring):**
- `export-wallet --descriptor <policy> --format descriptor` → the concrete descriptor; **round-trip cell:** that output piped to `--format bip388` reproduces the original `description_template` + `keys_info` (assert equal; document `name` difference). (RED first: today the policy trips the `:413` refusal.)
- `export-wallet --descriptor <policy> --format bitcoin-core` → importdescriptors JSON (receive+change, watch-only).
- **Ordering cell (load-bearing):** `export-wallet --descriptor <raw policy>` does NOT emit the "accepts only concrete descriptors" refusal (proves the pre-check precedes `is_at_n_form`).
- `bundle --descriptor <policy> --network testnet` → `ms1` omitted + N× `mk1` + `md1` (watch-only); assert card-type set.
- **Bundle bare-key negative (M-2):** `bundle --descriptor <policy with origin-less keys_info>` → refusal "...must carry a key origin..." (classify (false,false); pins the origin-annotated requirement).
- **Non-canonical round-trip (M-4):** a hand-authored policy with deliberately non-canonical `sortedmulti` key order → `--format descriptor` → `--format bip388` reproduces the **miniscript-canonicalized** order (assert the reorder happened, stable thereafter — so the impl doesn't over-promise byte-stability on hand-authored input). `multi()` (order-significant) round-trips faithfully — separate cell.
- Negative: malformed policy (`@5` with 5 keys) → `DescriptorParse` (not a half-substituted miniscript error).

**Verification (§7):** full `cargo test` (post-version-bump), `cargo clippy --all-targets -- -D warnings`, confirm `schema_mirror`/gui-schema unaffected (no surface change). Use a real fixture policy = the in-session `wsh(andor(...))` vault's `--format bip388` output (6 cosigners) + a simple `wsh(sortedmulti(2,@0,@1))` policy.

## 7. Ship plan
1. Phase 1 (shared helper + dedup) → per-phase reviewer-loop → GREEN.
2. Phase 2 (both wirings + tests) → per-phase reviewer-loop → GREEN.
3. Manual prose note (§5). 
4. Version bump **v0.49.0**: `Cargo.toml` + `Cargo.lock` + **BOTH** README markers (`README.md` + `crates/mnemonic-toolkit/README.md` `<!-- toolkit-version: 0.49.0 -->`) + `scripts/install.sh` self-pin (`mnemonic-toolkit-v0.49.0`) — ALL before the tag; **re-run full suite AFTER the bump** (per the v0.48.0 release-gate lessons: `readme_version_current.rs` + `install-pin-check.yml`).
5. Tag `mnemonic-toolkit-v0.49.0` → push; confirm `rust.yml` + `install-pin-check` green.
6. Flip FOLLOWUP `bip388-wallet-policy-to-descriptor-expansion-not-surfaced` → resolved.
7. Memory.

### Out of scope
- Option (b) (seed-gated) and (c) (redundant with `--format descriptor`).
- Honoring a round-tripped policy `name` (`--wallet-name` on bip388) — fast-follow FOLLOWUP.
- Clearer md1-on-`--descriptor` error for export-wallet/bundle — fast-follow.
- Any change to the emitter, `classify_descriptor_form`, or the xpub-search search semantics.
