# BRAINSTORM — cycle-10: md-codec LIBRARY cluster (M3 + L14/L15/L17 + L6)

**Status:** DESIGN ONLY (no code). Feeds the mandatory opus-architect **R0 loop → 0 Critical / 0 Important**.
**Date:** 2026-06-21.
**Cycle anchor:** M3 (funds-availability) — the single funds-relevant item, treated formally; L14/L15/L17 (one coherent identity-stability sub-fix) + L6 (lib panic guard) batched into the same `md-codec` release.

**SCOPE (locked):** M3 + L14 + L15 + L17 + L6 ONLY.
- **OUT of scope (won't-fix / doc):** L16 (LP4-ext varint < 2²⁹ cap), D-md-chunk-budget (benign sub-optimal packing). FOLLOWUP-recorded below.
- **OUT of scope (separate future wire-format cycle):** D-mk-crosschunk (20-bit `chunk_set_id` widening — touches the fixed 37-bit chunk-header wire layout). FOLLOWUP-recorded below.

---

## 1. Source-SHA table (cite `1a4b322` — re-grepped at write time)

**Repo:** `descriptor-mnemonic`. **`origin/main` = `1a4b322618e3831fdbb2578bc6f98c7a23bc58e3`** (`release: md-cli 0.9.0 — lexer/parser robustness + template-classification fixes (cycle-9)`).
**md-codec version:** `0.38.0` (`crates/md-codec/Cargo.toml:3`). **md-cli:** `0.9.0`, exact-pin `md-codec = { path = "../md-codec", version = "=0.38.0" }` (`crates/md-cli/Cargo.toml:28`).
**Toolkit** (this repo, HEAD `feature/bundle-md1-template-multisig`): pin `md-codec = "0.37"` (`crates/mnemonic-toolkit/Cargo.toml:36`) → `Cargo.lock:677` resolves **0.37.0**; latest toolkit tag = **v0.65.0**.

| Finding | File @ `1a4b322` | Lines (verified) | Anchor bytes |
|---|---|---|---|
| **M3** gate | `crates/md-codec/src/derive.rs` | `92` (`pub fn derive_address`), **`110`** (`if let Some(alts) = &self.use_site_path.multipath`), **`117`** (`} else if chain != 0 {`), **`118-121`** (`return Err(Error::ChainIndexOutOfRange { chain, alt_count: 0 })`), `123` (`to_miniscript_descriptor(self, chain)?`) | gate reads ONLY `self.use_site_path.multipath`; no read of `self.tlv.use_site_path_overrides` |
| M3 override field | `crates/md-codec/src/tlv.rs` | `26` | `pub use_site_path_overrides: Option<Vec<(u8, UseSitePath)>>` |
| M3 legal D5(b) | `crates/md-codec/src/validate.rs` | `118-127` | doc: "a `Some`-multipath baseline mixed with a `None`-multipath override … is a **legal divergent STRUCTURE** … NOT a reject" |
| M3 per-key resolve | `crates/md-codec/src/to_miniscript.rs` | `54-58` (`to_miniscript_descriptor` → `expand_per_at_n(d)`), `277-292` (`use_site_to_derivation_path`: resolves `chain` against EACH key's `u.multipath`, errors `ChainIndexOutOfRange` only if THAT key lacks the alt) | per-key path is override-aware and already correct |
| **L14** false doc | `crates/md-codec/src/identity.rs` | `108-114` | "…produce identical IDs whether they elide canonical paths or write them out explicitly. Stable across origin- and use-site-elision" |
| L14 root (INVARIANT) | `crates/md-codec/src/identity.rs` | `155-172` | "this function does NOT consult `canonical_origin` … reads `OriginPathOverrides[idx]` if present, else `path_decl.paths`" |
| L14 fix locus | `crates/md-codec/src/identity.rs` | `189-194` (per-record `e.origin_path.write(&mut path_scratch)`) | empty/elided origin hashed verbatim |
| L14 fill source | `crates/md-codec/src/canonical_origin.rs` | `45-79` (`pub fn canonical_origin(tree: &Node) -> Option<OriginPath>`) | wrapper→canonical table (pkh→44'/0'/0', wpkh→84'/0'/0', tr→86'/0'/0', wsh-multi→48'/0'/0'/2', sh-wsh-multi→48'/0'/0'/1', else None) |
| L14 elision behavior | `crates/md-codec/src/canonicalize.rs` | `420-465` (`expand_per_at_n`) | origin resolved verbatim; `canonical_origin` consulted ONLY to decide `MissingExplicitOrigin` (`:450-456`), never to FILL before hashing |
| **L15** WDT-id | `crates/md-codec/src/identity.rs` | `71-115` (`compute_wallet_descriptor_template_id`) | NO `canonicalize_placeholder_indices` call; writes raw `*idx` (`:83 sub.write_bits(u64::from(*idx), …)`) |
| L15 asymmetry | `crates/md-codec/src/identity.rs` | `175-177` | policy-id DOES `canonicalize_placeholder_indices(&mut d_canonical)?` first |
| **L17** vacuous test | `crates/md-codec/src/identity.rs` | `572-588` (`walletpolicyid_stable_across_origin_elision`) + fixture `385-419` (`cell_7_wpkh_descriptor` builds explicit `Shared(BIP84)`; override is byte-identical to baseline) | never constructs an empty `path_decl` |
| **L6** panic | `crates/md-codec/src/canonicalize.rs` | `206-219` (`if let PathDeclPaths::Divergent(paths) … new_paths.push(old_paths[inverse[new_idx] as usize].clone())`) — NO `old_paths.len()==n` guard | panics on a short Divergent vector |
| L6 sibling guard | `crates/md-codec/src/canonicalize.rs` | `426-431` (`expand_per_at_n`: `if paths.len() != d.n as usize { return Err(Error::DivergentPathCountMismatch { n, got }) }`) | the guard to mirror |
| L6 error variant | `crates/md-codec/src/error.rs` | `66-72` | `DivergentPathCountMismatch { n: u8, got: usize }` |
| id consumers (in-memory only) | `crates/md-cli/src/cmd/{encode,inspect}.rs`, `crates/md-cli/src/format/{json,text}.rs` | display/compare only | NOT embedded in md1 wire |
| wire does NOT embed id | `crates/md-codec/src/encode.rs` | (grep `policy_id\|template_id\|compute_wallet` → ∅) | `encode_md1_string` emits no WDT/policy id |
| toolkit id consumers | `crates/mnemonic-toolkit/src/cmd/bundle.rs` | `1219` (`compute_wallet_policy_id`), `1221` (`compute_wallet_descriptor_template_id`); `restore.rs:1703` (`compute_wallet_policy_id` for `--expect-wallet-id`) | toolkit binds/searches on these ids in-memory |

---

## 2. Finding summary (in-scope REPRODUCE confirmed; out-of-scope noted)

| ID | Class | Severity (hunt) | Reproduces @ `1a4b322`? | Treatment |
|---|---|---|---|---|
| **M3** | B-policy-collapse (funds-AVAILABILITY) | MEDIUM | **YES** — gate at `derive.rs:110-122` rejects every non-zero `chain` for a `None`-baseline + `Some(<0;1>)`-override (legal D5(b)) wallet, though the per-key path would derive correctly | **FORMAL anchor** — fix |
| **L14** | B-policy-collapse (identity-stability) | LOW | **YES** — `compute_wallet_policy_id` hashes an elided empty origin verbatim; elided `wpkh(@0)` ≠ explicit `m/84'/0'/0'` despite the doc-invariant | fix (canonical-fill) |
| **L15** | B-policy-collapse (identity-stability) | LOW | **YES** — WDT-id hashes raw placeholder indices; `wsh(multi(2,@1,@0))` ≠ `(@0,@1)` (asymmetric vs policy-id which canonicalizes) | fix (symmetric canonicalize) |
| **L17** | test-vacuity (masks L14) | LOW | **YES** — both operands carry explicit `Shared(BIP84)`; the "override" is byte-identical to the baseline; never exercises an empty `path_decl` | de-vacuify |
| **L6** | E-panic-dos (lib-only) | LOW | **YES** (library boundary) — short Divergent vector → `old_paths[inverse[new_idx]]` OOB panic; not wire-reachable (`PathDecl::read` always reads exactly `n`), reachable by a hand-built `Descriptor` → `encode_payload`/`compute_wallet_policy_id` | add length guard |
| ~~L16~~ | E-panic-dos (graceful) | LOW | reproduces gracefully (typed `VarintOverflow`) | **OUT — won't-fix/doc** |
| ~~D-md-chunk-budget~~ | sub-optimal packing | — | reproduces (≈11% slack; worst case 357 bits ≪ 80-symbol cap) | **OUT — won't-fix** |
| ~~D-mk-crosschunk~~ | defense-in-depth (~2⁻²⁰) | — | reproduces; **WIRE-FORMAT** widening | **OUT — separate wire-format cycle** |

---

## 3. Per-finding fix design

### 3.1 M3 — `derive_address` chain-gate widening (FORMAL anchor)

**Current (`derive.rs:110-122`):**
```rust
if let Some(alts) = &self.use_site_path.multipath {
    if (chain as usize) >= alts.len() {
        return Err(Error::ChainIndexOutOfRange { chain, alt_count: alts.len() });
    }
} else if chain != 0 {
    return Err(Error::ChainIndexOutOfRange { chain, alt_count: 0 });
}
```
This bounds `chain` solely from the **baseline** `self.use_site_path.multipath`. For a legal D5(b) descriptor with a `None` baseline (`bare /*`) plus a per-`@N` override carrying `Some(<0;1>)`, the gate takes the `else if chain != 0` arm and rejects `chain=1` with `ChainIndexOutOfRange { alt_count: 0 }` — even though the overridden key's change (chain-1) address is real and fundable.

**Fix — widen the gate to the MAX alt-count across the baseline AND every override:**
```rust
// Max alt-count across the baseline use-site path AND every per-`@N`
// use-site override (treat None as alt-count 1, i.e. only chain 0).
let baseline_alts = self.use_site_path.multipath.as_ref().map(|a| a.len()).unwrap_or(1);
let max_alts = self
    .tlv
    .use_site_path_overrides
    .iter()
    .flatten()
    .map(|(_, p)| p.multipath.as_ref().map(|a| a.len()).unwrap_or(1))
    .fold(baseline_alts, std::cmp::max);
if (chain as usize) >= max_alts {
    return Err(Error::ChainIndexOutOfRange { chain, alt_count: max_alts });
}
```
- **Semantics of "alt-count" for `None`:** a `None` multipath supports exactly one chain (chain 0). Model it as alt-count **1** so the `>=` comparison is uniform. (The pre-fix `else if chain != 0` arm is exactly "alt-count 1": chain 0 OK, anything else rejected.)
- **Does NOT over-widen.** `max_alts` is the maximum number of chains supported by ANY key (baseline or override). The per-key resolution in `use_site_to_derivation_path` (`to_miniscript.rs:277-292`) is the real authority: for each key it resolves `chain` against THAT key's own (override-aware, via `expand_per_at_n`) multipath and STILL errors `ChainIndexOutOfRange` if the specific key lacks that alt. So:
  - `chain ∈ [0, max_alts)` passes the pre-flight; a key that genuinely supports it derives; a key that does not still errors in `to_miniscript_descriptor`. **Correct fail-closed behavior preserved.**
  - `chain ≥ max_alts` is rejected by the widened pre-flight (a chain beyond the real per-key max still rejects — the requested positive control).
- **Funds-availability:** a valid `None`-baseline + `Some(<0;1>)`-override wallet now derives its chain-1 change address instead of rejecting it. **Fail-closed before (errors, never wrong-address); now correctly available.**
- **alt_count in the error** changes from `0` to `max_alts` for the over-range case. This is an error-payload value change only (still a `ChainIndexOutOfRange`); see §4 SemVer note. The existing test `derive_address_chain_out_of_range` (`derive.rs:241-267`, baseline alt-count 2, `chain=5`) still expects `alt_count: 2` — unchanged because that descriptor has no overrides, so `max_alts == baseline_alts == 2`.

**The gate is the ONLY bug** — confirmed by code trace: the per-key path already composes the override over the baseline (`expand_per_at_n` → `e.use_site_path`) and resolves `chain` correctly. No change needed in `to_miniscript.rs`/`expand_per_at_n`.

### 3.2 L14 + L15 + L17 — identity-stability sub-fix (one atomic `identity.rs` change)

**L14 — canonical-fill in `compute_wallet_policy_id`.** Make the id honor its documented "stable across origin-elision" invariant: when a per-`@N` record's resolved origin path is empty (elided) AND the wrapper has a canonical origin, substitute the canonical path before hashing — so the elided form hashes identically to the explicit form.

Fix locus = the per-record loop at `identity.rs:189-194`, before `e.origin_path.write(&mut path_scratch)`:
```rust
// L14: canonical-fill an elided (empty) origin so the id is stable
// across origin-elision per the documented invariant. An empty resolved
// origin with a canonical wrapper hashes identically to the explicit form.
let origin_for_hash: OriginPath = if e.origin_path.components.is_empty() {
    crate::canonical_origin::canonical_origin(&d.tree)
        .unwrap_or_else(|| e.origin_path.clone())
} else {
    e.origin_path.clone()
};
let mut path_scratch = BitWriter::new();
origin_for_hash.write(&mut path_scratch)?;
```
- **Why empty-only:** `expand_per_at_n` already returns the EXPLICIT path verbatim when one is present (override or `Shared`/`Divergent` baseline). Only the elided case (`components.is_empty()`) needs the fill. When `canonical_origin` is `None` (forced-explicit shapes), `expand_per_at_n` would already have raised `MissingExplicitOrigin` for a truly-empty path — so the `unwrap_or_else` fallback is unreachable-but-safe (an empty path with no canonical reaching the hash is structurally precluded upstream).
- **DECISION: implement the fill, do NOT weaken the doc.** The toolkit binds/searches on this id (`bundle.rs:1219`, `restore.rs:1703`) and the documented invariant is the contract consumers rely on; honoring it is the funds-coherent choice. (Alternative — making the doc honest "origin-significant" — rejected: it would leave a real false-non-match between an md-cli-elided card and a toolkit-built explicit form.)

**L15 — symmetric canonicalization in `compute_wallet_descriptor_template_id`.** Mirror the policy-id's Step 1: canonicalize placeholder indices on a clone before hashing, so `wsh(multi(2,@1,@0))` and `wsh(multi(2,@0,@1))` produce the same WDT-id.

Fix at `identity.rs:71-77` (top of the fn body):
```rust
pub fn compute_wallet_descriptor_template_id(
    d: &Descriptor,
) -> Result<WalletDescriptorTemplateId, Error> {
    // L15: canonicalize placeholder ordering on a clone first (mirror
    // compute_wallet_policy_id) so the WDT-id is invariant to placeholder
    // index permutation, matching the policy-id's behavior.
    let mut d_canonical = d.clone();
    canonicalize_placeholder_indices(&mut d_canonical)?;
    let d = &d_canonical;
    let mut w = BitWriter::new();
    // …existing body unchanged…
```
- `canonicalize_placeholder_indices` is already imported at `identity.rs:4`. The fn returns `Result<…, Error>`, so the `?` rides the existing signature with no new error surface.
- **Identity fast-path preserved:** `canonicalize_placeholder_indices` early-returns `Ok(())` on the identity permutation (`canonicalize.rs:198-201`), so already-canonical inputs (the toolkit's `@0,@1,…` ordering) get the SAME WDT-id as before — no value change for canonical inputs.

**L17 — de-vacuify `walletpolicyid_stable_across_origin_elision` (`identity.rs:572-588`).** The existing test builds two explicit `Shared(BIP84)` operands (the override byte-identical to the baseline) and never exercises an empty path. Replace it with a test that constructs a genuinely ELIDED operand and asserts it equals the explicit form:
```rust
#[test]
fn walletpolicyid_stable_across_origin_elision() {
    // Explicit: wpkh(@0) with path_decl = Shared(m/84'/0'/0').
    let d_explicit = cell_7_wpkh_descriptor();
    // Elided: same wpkh(@0) wallet, but path_decl is an EMPTY Shared
    // origin (no explicit path) — the canonical wrapper (wpkh →
    // m/84'/0'/0') supplies the path at hash time per the L14 fill.
    let mut d_elided = cell_7_wpkh_descriptor();
    d_elided.path_decl = PathDecl {
        n: 1,
        paths: PathDeclPaths::Shared(OriginPath { components: vec![] }),
    };
    let id_explicit = compute_wallet_policy_id(&d_explicit).unwrap();
    let id_elided = compute_wallet_policy_id(&d_elided).unwrap();
    // The documented "stable across origin-elision" invariant: the elided
    // form, canonical-filled, must hash identically to the explicit form.
    assert_eq!(id_explicit, id_elided);
}
```
- **What it must ACTUALLY assert:** that an empty-`path_decl` `wpkh(@0)` (which the L14 fill resolves to the canonical `m/84'/0'/0'`) produces the SAME `WalletPolicyId` as the explicit-path form. This is RED today (the bare empty path hashes differently — the elided origin contributes a 4-bit length prefix of `0000` and no components, vs the explicit path's component bits) → GREEN after the L14 fix.
- Keep `walletpolicyid_stable_across_use_site_elision` (`:593-608`) as-is — it covers a different (use-site, override-supplied) axis and is non-vacuous (it sets `multipath: None` baseline + a `Some` override, exercising real resolution).

**Atomicity:** L14 fill + L15 canonicalize + L17 de-vacuify land as one commit in `identity.rs` — they share the canonicalization-invariant theme and the de-vacuified test is the RED→GREEN gate for L14.

### 3.3 L6 — `canonicalize_placeholder_indices` Divergent length guard

At the top of the non-identity `Divergent` branch (`canonicalize.rs:206`), add the same guard `expand_per_at_n` already has (`:426-431`):
```rust
if let PathDeclPaths::Divergent(paths) = &mut d.path_decl.paths {
    // L6: a hand-built Descriptor can carry a short Divergent vector;
    // guard before indexing old_paths[inverse[new_idx]] to surface a
    // typed error instead of an out-of-bounds panic (mirror expand_per_at_n).
    if paths.len() != n {
        return Err(Error::DivergentPathCountMismatch {
            n: d.n,
            got: paths.len(),
        });
    }
    // …existing reorder loop unchanged…
```
- `n` is the local `d.n as usize` bound at `canonicalize.rs:169`. `Error::DivergentPathCountMismatch { n: u8, got: usize }` already exists (`error.rs:66-72`) — no new error variant, no new public surface.
- **Borrow note:** `d.n` (the `u8`) must be read for the error payload while `paths` mutably borrows `d.path_decl.paths`. Bind `let n_keys = d.n;` before the `if let` (or use the already-bound `n: usize` cast back — but the error field wants the `u8`), so the error construction does not re-borrow `d`. The plan-doc resolves the exact binding; the design intent is "typed error not panic, mirroring `expand_per_at_n`."

---

## 4. SemVer / publish→pin chain

### In-memory vs wire — CONFIRMED facts (verified against `1a4b322`)
- **WDT-id / WalletPolicyId / Md1EncodingId are 128-bit IN-MEMORY identifiers, NOT embedded in the md1 wire payload.** `encode_md1_string` (`encode.rs`) emits no policy/WDT id (grep `policy_id|template_id|compute_wallet` over `encode.rs` → ∅). They surface ONLY as md-cli display/compare output (`encode.rs:66,102`, `inspect.rs:19-20`, `format/{json,text}.rs`) and as toolkit in-memory bind/search keys (`bundle.rs:1219-1221`, `restore.rs:1703`). **→ L14/L15 do NOT change emitted cards, only id comparisons.**
- **M3** is a gate-widening; no wire change. `<a;b>` multipath / BIP-388 substitution semantics unaffected.
- **L6** adds a typed error on a hand-built short Divergent vector; not wire-reachable (`PathDecl::read` always reads exactly `n`).

### Public-API change assessment (the breaking-concern check)
- **No signature changes.** `derive_address`, `compute_wallet_policy_id`, `compute_wallet_descriptor_template_id`, `canonicalize_placeholder_indices` keep their exact signatures and return types. No new public types, no new `Error` variants (`ChainIndexOutOfRange`, `DivergentPathCountMismatch` already exist).
- **Behavioral value changes (the "3 new-ish behaviors"):**
  1. M3: `derive_address` now SUCCEEDS where it errored for a valid override change-chain (strict capability gain); the over-range error's `alt_count` payload changes `0 → max_alts` for the None-baseline+override case.
  2. L14: `compute_wallet_policy_id` returns a DIFFERENT id for an **elided** origin input (now equals the explicit form). Explicit-origin inputs (what the toolkit always builds) are **unchanged**.
  3. L15: `compute_wallet_descriptor_template_id` returns a DIFFERENT WDT-id for **non-canonical placeholder ordering** input. Canonical-ordering inputs (the toolkit always builds `@0,@1,…`) are **unchanged** (identity fast-path).
- **CONFIRMED in-memory-only / no persisted-format break:** because the ids are not on the wire, no emitted/stored md1 card changes. The id-value deltas affect only in-memory dedup/match/search and md-cli `inspect`/`json` display. **→ this is a MINOR (not breaking) change under Cargo pre-1.0 SemVer**, where even a behavioral/value change ships MINOR for `0.x`. Flag the id-value change LOUDLY in release notes.

### Version bumps (the publish→pin chain, ~2–3 bumps)
1. **md-codec MINOR `0.38.0 → 0.39.0`** (`crates/md-codec/Cargo.toml:3`). The L15/L14 id-value change rides MINOR (pre-1.0). **Publish to crates.io FIRST.**
2. **md-cli re-release `0.9.0 → 0.9.1`** (`crates/md-cli/Cargo.toml:version`) with the EXACT pin hand-edited `version = "=0.38.0" → "=0.39.0"` (`crates/md-cli/Cargo.toml:28`). **This exact-pin edit is a BLOCKING hand-edit** — the path+exact-version pin will not resolve against the new md-codec otherwise. **No md-cli code change** (the cluster is entirely library-internal); md-cli's `inspect`/`encode` id display is a value-level change only, no flag/output-shape change → **no manual-mirror, no new flags.** Publish md-cli 0.9.1 to crates.io AFTER md-codec 0.39.0.
3. **toolkit pin-bump** `md-codec = "0.37" → "0.39"` (`crates/mnemonic-toolkit/Cargo.toml:36`) + `Cargo.lock` regen to 0.39.0. **Toolkit version: PATCH** (next = **v0.65.1**) — M3's widening is a strict capability gain surfaced transitively; no toolkit feature gains a new flag/capability, and the toolkit only ever builds explicit-origin / canonical-ordering descriptors so its recorded ids are unchanged in practice. (A toolkit-built explicit form now correctly MATCHES an md-cli-elided card's id — a fix, not a regression.)

**Version-site sweep (silent-drift guard, per release-ritual memory):** the md-codec MINOR touches `crates/md-codec/Cargo.toml:3`, `crates/md-cli/Cargo.toml:28` (exact pin) + its own `version`, root `Cargo.lock` (md-codec entry @ `:500` in repo; also `Cargo.lock:677` in the toolkit), `fuzz/Cargo.lock` (regen — currently stale at `0.35.1`, path-resolved), and CHANGELOG entries (`CHANGELOG.md` has 0.38.0 entries to extend). `fuzz/Cargo.toml` references md-codec by **path only** (no version literal) → no edit there beyond lock regen. The workspace `Cargo.toml` uses path members → no version literal.

**Publish ORDER (strict):** md-codec 0.39.0 → crates.io → THEN md-cli 0.9.1 (pin-bump) → crates.io → THEN one toolkit PATCH pin-bump consuming md-codec 0.39.0.

### Locksteps — NONE required
- **No `schema_mirror`** — no clap flag/option/subcommand/dropdown change (`mnemonic-gui/src/schema/mnemonic.rs` untouched; the gate is flag-NAME parity only).
- **No manual-mirror** (`docs/manual/src/40-cli-reference/`) — no CLI-surface change in any of the four CLIs.
- **No sibling-codec FOLLOWUP companion** for the in-scope items (md-codec-internal). (A companion is warranted only when D-mk-crosschunk's mk-codec wider-id contrast is formalized in the future wire-format cycle.)

---

## 5. Per-finding tests (TDD, RED-first)

All tests live in `crates/md-codec/src/{derive,identity,canonicalize}.rs` `#[cfg(test)]` modules (md-codec-internal; no CLI test needed — the cluster is library-internal).

### 5.1 M3 (`derive.rs` tests)
- **`derive_address_override_change_chain_derivable` (RED→GREEN, funds-availability):** build a `Descriptor` with `use_site_path.multipath = None` (baseline, alt-count→1) + `tlv.use_site_path_overrides = Some(vec![(0u8, UseSitePath { multipath: Some(<0;1>), … })])` and a `pubkeys` entry for `@0`; `derive_address(chain=1, index=0, network)` → **errors `ChainIndexOutOfRange { alt_count: 0 }` today** → **derives a valid change address after the fix.** Assert `chain=0` also derives (receive control). Mirror the legal D5(b) shape from `validate.rs:118-127`.
- **`derive_address_override_chain_over_max_still_rejects` (positive control):** same wallet, `derive_address(chain=2)` (beyond the override's 2-alt max) → still `Err(ChainIndexOutOfRange { chain: 2, alt_count: 2 })`. Confirms no over-widening.
- **Regression — existing `derive_address_chain_out_of_range` (`derive.rs:241-267`) must stay GREEN unchanged:** baseline alt-count 2, no overrides, `chain=5` → `alt_count: 2` (because `max_alts == baseline_alts == 2`). Cite it as the unchanged-behavior anchor.

### 5.2 L14/L15/L17 (`identity.rs` tests)
- **`walletpolicyid_stable_across_origin_elision` (de-vacuified — RED→GREEN, §3.2):** elided empty `path_decl` `wpkh(@0)` == explicit `m/84'/0'/0'` form. RED today, GREEN after the L14 fill.
- **`wdt_id_invariant_to_placeholder_ordering` (NEW, RED→GREEN for L15):** build a 2-of-2 `wsh(multi(2,@0,@1))` vs `wsh(multi(2,@1,@0))` (same keys, swapped placeholder indices); `compute_wallet_descriptor_template_id` → **differs today → equal after the L15 canonicalize.** (Model on `walletpolicyid` partial-keys fixtures; use `pkk(index)` helper pattern at `identity.rs:627`.)
- **Regression — `compute_wallet_policy_id_canonicalizes_first` (`identity.rs:791`) stays GREEN:** confirms policy-id canonicalization unchanged.
- **Regression — canonical-ordering WDT-id unchanged:** a canonical `@0,@1` input WDT-id is byte-identical pre/post L15 (identity fast-path). Pin a golden-value assertion or an explicit "id1_pre == id1_post is implied by the identity fast-path" note; the plan-doc may capture the golden bytes.
- **Regression — explicit-origin policy-id unchanged:** `cell_7_wpkh_descriptor` (explicit `Shared(BIP84)`) policy-id unchanged by the L14 fill (the fill is empty-only). Covered by the existing `golden_vector_wpkh_cell_7` (`identity.rs:469`) staying GREEN.

### 5.3 L6 (`canonicalize.rs` tests)
- **`canonicalize_short_divergent_returns_typed_error` (RED→GREEN):** hand-build a `Descriptor` with `n=2`, a non-canonical tree (e.g. `multi(@1,@0)` so the permutation is non-identity), and `path_decl.paths = Divergent(vec![one_path])` (length 1 ≠ n=2); call `canonicalize_placeholder_indices(&mut d)` → **panics (OOB index) today → `Err(Error::DivergentPathCountMismatch { n: 2, got: 1 })` after the guard.** (Today the panic is the RED state; the test asserts the typed error post-fix.)
- **Regression — identity-permutation short-Divergent is NOT reached:** the identity fast-path (`canonicalize.rs:198-201`) returns before the Divergent branch, so a canonical-ordering descriptor never hits the guard — assert a canonical short-divergent input still `Ok(())` (or document why the guard only fires on non-identity).

---

## 6. FOLLOWUP slugs

**In-scope — file as RESOLVED-in-this-cycle (flip status in the shipping commit):**
- `md-codec-derive-chain-gate-baseline-only-ignores-overrides` (M3) — RESOLVED by the gate-widening in `derive.rs`.
- `md-codec-walletpolicyid-canonical-fill-origin-elision` (L14) — RESOLVED by the canonical-fill in `compute_wallet_policy_id`.
- `md-codec-wdt-id-canonicalize-placeholder-ordering` (L15) — RESOLVED by the symmetric canonicalize in `compute_wallet_descriptor_template_id`. **Release-notes: id-VALUE change for non-canonical inputs.**
- `md-codec-walletpolicyid-elision-test-vacuous` (L17) — RESOLVED by the de-vacuified test.
- `md-codec-canonicalize-divergent-path-decl-unchecked-len-panic` (L6) — RESOLVED by the length guard.

**Out-of-scope — file/keep with explicit disposition:**
- `md-codec-lp4ext-varint-cannot-encode-child-ge-2pow29` (L16) — **WON'T-FIX / DOC.** BIP-32 unhardened indices in [2²⁹, 2³¹) are legal but vanishingly rare (standard purpose/coin/account/change/index never approach 2²⁹); the failure mode is a graceful typed `VarintOverflow`, never panic/wrong-address. Disposition: document the 2²⁹ ceiling as a known limitation (or a cheap parse-boundary guard) — no fix this cycle. Track for a future completeness pass if 31-bit child support is independently demanded.
- `md-codec-chunk-split-ignores-37bit-header-budget` (D-md-chunk-budget) — **WON'T-FIX.** Benign sub-optimal packing (~11% slack); worst-case 357 bits ≪ the codex32 80-data-symbol / 93-codeword cap, so `wrap_payload` never overflows. No correctness/funds impact. Close as won't-fix (or a one-line sizing-divisor tweak only if a chunk-count reduction is independently desired).
- `md-codec-chunk-set-id-20bit-crosschunk-bind` (D-mk-crosschunk) — **DEFERRED to a separate wire-format cycle.** Widening the 20-bit `chunk_set_id` is the ONLY change touching the fixed 37-bit chunk-header wire layout (4+1+**20**+6+6; SPEC v0.30 §2.2) → needs a SPEC bump + chunked-wire version bump. Defense-in-depth (~2⁻²⁰), partially mitigated by the reassemble-time `compute_md1_encoding_id` re-derive cross-check. NOT batched here; track its own FOLLOWUP + a future cycle. (Companion mk-codec note warranted when scheduled — mk-codec uses a wider id.)

---

## 7. Resolved decisions (no open questions)

| # | Decision | Rationale |
|---|---|---|
| D1 | **M3 gate uses MAX alt-count over baseline + all use-site overrides; `None` modeled as alt-count 1.** | Uniform `>=` bound; the per-key path (`use_site_to_derivation_path`) is the real authority and still fail-closes per key, so widening the pre-flight is necessary AND sufficient. |
| D2 | **M3 does NOT touch `to_miniscript.rs`/`expand_per_at_n`.** | Trace-confirmed the per-key path already composes override-over-baseline correctly; the gate is the sole bug. |
| D3 | **L14: IMPLEMENT canonical-fill (empty-origin only); do NOT weaken the doc.** | The documented invariant is the consumer contract (toolkit binds/searches on the id); honoring it removes a real false-non-match between an md-cli-elided card and a toolkit-built explicit form. Empty-only because `expand_per_at_n` already returns explicit paths verbatim. |
| D4 | **L15: canonicalize a clone in `compute_wallet_descriptor_template_id`, mirroring policy-id.** | Symmetry; the identity fast-path leaves canonical inputs (what the toolkit builds) unchanged. |
| D5 | **L17: de-vacuify by constructing a genuinely ELIDED empty `path_decl` and asserting it equals the explicit form.** | The existing test's "override" is byte-identical to the baseline → never exercises elision; the rewrite is the RED→GREEN gate for L14. |
| D6 | **L6: reuse the existing `Error::DivergentPathCountMismatch`; no new variant.** | The sibling `expand_per_at_n` already returns it for the same condition; mechanical symmetry, no public-surface growth. |
| D7 | **SemVer: md-codec MINOR 0.39.0; md-cli 0.9.1 (exact-pin hand-edit, no code); toolkit PATCH v0.65.1 pin-bump.** | Pre-1.0 → behavioral/id-value changes ride MINOR; ids are in-memory-only (not wire) so no card/persisted-format break; md-cli has no CLI-surface change; toolkit gains no flag. |
| D8 | **CONFIRMED in-memory-only:** WDT-id / WalletPolicyId / Md1EncodingId are NOT on the md1 wire (`encode_md1_string` emits no id); they are display/compare/bind keys only. | Verified by grep over `encode.rs` (∅) + the consumer map (md-cli formatters + toolkit bind/search). This is the load-bearing fact that keeps L14/L15 MINOR-not-breaking. |
| D9 | **L16 + D-md-chunk-budget → won't-fix/doc; D-mk-crosschunk → separate wire-format cycle.** | Per the SCOPE lock and the recon triage; no fix spent this cycle. |
| D10 | **No locksteps** (no `schema_mirror`, no manual-mirror, no sibling companion for in-scope items). | All changes are library-internal with no clap/CLI-surface delta. |
| D11 | **Publish ORDER strict:** md-codec 0.39.0 → crates.io → md-cli 0.9.1 → crates.io → toolkit pin-bump. | The md-cli `=` exact pin won't resolve until md-codec 0.39.0 is published; toolkit can't pin 0.39 until it's on crates.io. |
| D12 | **Tests are md-codec-internal `#[cfg(test)]` only** (no CLI/integration test). | The cluster is library-internal; the funds-availability behavior (M3) is exercised at the `derive_address` boundary where the bug lives. |

---

## 8. MANDATORY R0 GATE (project standard)

This is a **brainstorm spec — DESIGN ONLY, NO CODE.** Per CLAUDE.md Conventions (first bullet), before ANY implementation begins this spec MUST pass an **opus-architect R0 review** and the reviewer-loop MUST converge to **0 Critical / 0 Important**: fold findings → persist the review verbatim to `design/agent-reports/` → re-dispatch → repeat until GREEN. The reviewer-loop continues after every fold (folds can introduce drift). No code, no implementer dispatch, no SemVer bump, no publish, no tag/ship while ANY Critical or Important finding is open. A subsequent plan-doc carries the same R0 gate, and per-phase execution carries its own R0 + a mandatory post-implementation whole-diff adversarial review.
