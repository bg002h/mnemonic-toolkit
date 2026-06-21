# cycle-prep recon — 2026-06-21 — cycle-10 md-codec LIBRARY cluster (M3, L6, L14, L15, L16, L17, D-md-chunk-budget, D-mk-crosschunk)

**Repo:** `descriptor-mnemonic` (`/scratch/code/shibboleth/descriptor-mnemonic`)
**Origin/main SHA at recon time:** `1a4b322` (`1a4b322618e3831fdbb2578bc6f98c7a23bc58e3`)
**Last commit:** `release: md-cli 0.9.0 — lexer/parser robustness + template-classification fixes (cycle-9)`
**Local branch:** detached HEAD (= the toolkit feature branch checkout reference; descriptor-mnemonic local is 0 ahead / 4 behind origin/main — synced to origin bytes for this recon)
**Sync state:** `origin/main` UNCHANGED since the bug-hunt report (report cited md HEAD `54dd765`, but the current `1a4b322` is cycle-9's md-cli-0.9.0 release on top; **md-codec source is byte-stable at 0.38.0** — verified per-finding below).
**Untracked:** none in descriptor-mnemonic.

**md-codec version (origin/main):** `0.38.0` (tag `md-codec-v0.38.0` exists). **Published crate** (registry source).
**md-cli:** `0.9.0`, pins `md-codec = { path = "../md-codec", version = "=0.38.0" }` (EXACT pin).
**Toolkit pin:** `md-codec = "0.37"` → Cargo.lock resolves **0.37.0** (toolkit is one published-minor BEHIND repo's 0.38.0; cycle-10 fix is the natural pin-bump trigger).
**GUI pin:** `md-codec` resolves 0.37.0 in `mnemonic-gui/Cargo.lock` (direct dep for `canonical_origin` classifier; flag-name `schema_mirror` gate does NOT fire on a pure library-internal md-codec change).

All eight findings live in **`crates/md-codec/src/`**. All citations re-checked against `git show origin/main:<path>`. Drift is minimal (the report was written at md HEAD `54dd765`; md-codec source did not move between then and `1a4b322`).

---

## Per-finding verification

### M3 — `derive_address` chain gate reads baseline use-site only → override change-chains UNDERIVABLE (AVAIL, the funds-relevant one)
- **WHAT:** The pre-flight chain gate in `derive_address` bounds the allowable `chain` solely from `self.use_site_path.multipath` (descriptor baseline), ignoring `self.tlv.use_site_path_overrides`. For the legal D5(b) mix (`None` baseline + `Some(<0;1>)` per-`@N` override), the gate takes the `else if chain != 0` arm and rejects every non-zero chain with `ChainIndexOutOfRange { alt_count: 0 }` — even though the overridden key's change address is real and fundable.
- **Citations:**
  - `derive.rs:108-122` chain gate — **DRIFTED-by-1 (block is `:109-122`)**. Confirming bytes: `:110 if let Some(alts) = &self.use_site_path.multipath {` … `:117 } else if chain != 0 {` → `:118 return Err(Error::ChainIndexOutOfRange { chain, alt_count: 0 });`. The gate reads ONLY `self.use_site_path.multipath`; **no** read of `self.tlv.use_site_path_overrides` anywhere in the pre-flight. **ACCURATE.**
  - override field — **ACCURATE**: `tlv.rs:26 pub use_site_path_overrides: Option<Vec<(u8, UseSitePath)>>`.
  - `validate.rs:112-148` D5(b) legal mix — **ACCURATE**: `validate_multipath_consistency` doc (`:124-138`) explicitly: "a `Some`-multipath baseline mixed with a `None`-multipath override … is a **legal divergent STRUCTURE** … NOT a reject." Confirms the None-baseline + Some-override descriptor is valid input.
- **STILL-REPRODUCES: YES (confirmed by code trace).** The per-key resolution path already does the right thing — `to_miniscript_descriptor` (`to_miniscript.rs:54-65`) calls `expand_per_at_n(d)` (composes override over baseline per `@N`) and builds each key from `e.use_site_path` (the *expanded* path); `use_site_to_derivation_path` (`:277`) resolves `chain` against **each key's own** override-aware multipath alts and only errors `ChainIndexOutOfRange` if that specific key lacks the alt. So if the pre-flight gate let `chain=1` through, derivation would SUCCEED. The bug is purely the over-narrow pre-flight gate rejecting before `to_miniscript_descriptor` runs. **Fail-closed (errors, never wrong-address) but a real availability loss for a valid wallet.**
- **Fix-site:** `derive.rs:109-122` — bound `chain` from the **max alt-count across baseline AND every `use_site_path_overrides` entry** (treat `None` as 0), allow `chain in 0..alt_count`. Widening the gate is sufficient; the per-key resolution is already correct. Cite source SHA `1a4b322`.

### L6 — `canonicalize_placeholder_indices` indexes Divergent `path_decl` without a `len==n` guard (lib-only PANIC)
- **WHAT:** Non-identity permutation branch reorders a `Divergent` path vector with `old_paths[inverse[new_idx]]` for `new_idx in 0..n`, with NO `old_paths.len()==n` check — panics on a short Divergent vector.
- **Citations:**
  - `canonicalize.rs:206-219` — **ACCURATE (off-by-a-few lines; the indexing line is ~`:215`)**. Confirming bytes: `if let PathDeclPaths::Divergent(paths) = &mut d.path_decl.paths {` … `new_paths.push(old_paths[inverse[new_idx] as usize].clone());` with no length guard.
  - sibling `expand_per_at_n` HAS the guard (`canonicalize.rs:~420-431`): `if paths.len() != d.n as usize { return Err(Error::DivergentPathCountMismatch { … }) }` — confirms the asymmetry.
- **STILL-REPRODUCES: YES, library-only.** `PathDecl::read` (origin_path.rs) always reads exactly `n` paths from the wire, so NOT reachable from decoded wire. Reachable by a library consumer who hand-builds a `Descriptor` with a short `Divergent` vector + non-canonical tree, then calls `encode_payload` / `compute_wallet_policy_id` (both call `canonicalize_placeholder_indices`). CLI is shielded (decoder gates inputs).
- **Fix-site:** add the same `paths.len()==n` guard (→ `Error::DivergentPathCountMismatch`) at the top of the non-identity branch in `canonicalize_placeholder_indices`. Cite `1a4b322`.

### L14 — `WalletPolicyId` NOT stable across origin-path elision (contradicts its doc-invariant)
- **WHAT:** The doc-comment claims the id is "Stable across origin- and use-site-elision," but `compute_wallet_policy_id` canonicalizes placeholder *indices* yet never canonical-**fills** an elided empty origin path; it reads `path_decl.paths` verbatim. So `wpkh(@0)` with an empty `path_decl` vs an explicit `m/84'/0'/0'` hash to DIFFERENT ids for the same logical wallet.
- **Citations:**
  - `identity.rs:106-113` false doc — **ACCURATE**: doc (`:105-113`) "…produce identical IDs whether they elide canonical paths or write them out explicitly. Stable across origin- and use-site-elision."
  - `identity.rs:172-240` `compute_wallet_policy_id` — **ACCURATE**: canonicalizes a clone (`:173-176 canonicalize_placeholder_indices(&mut d_canonical)`) but the INVARIANT comment (`:154-170`) admits `path_decl.paths` is assumed pre-populated and that elision-fill happens "at encode time only … this function does NOT consult canonical_origin." So an elided empty `path_decl` is hashed as empty → different id.
  - `canonicalize.rs:420-474` `canonical_origin` used only as error-gate (`expand_per_at_n` calls it solely to decide `MissingExplicitOrigin`, never to fill before hashing) — **ACCURATE**.
- **STILL-REPRODUCES: YES at the library boundary.** The doc/code contradiction is real; a consumer that dedups/matches engravings by `WalletPolicyId` (e.g. mk1 cosigner-stub binding) gets false non-matches between elided and explicit forms. Library-internal (CLI decoder always populates `path_decl`).
- **Fix-site:** `identity.rs:106-113` — either make the doc honest (id is origin-significant; drop the "stable across elision" claim) OR implement canonical-fill in `compute_wallet_policy_id` (re-introduce `canonical_origin` lookups per the INVARIANT note). Coherent with L15/L17. Cite `1a4b322`.

### L15 — `compute_wallet_descriptor_template_id` doesn't canonicalize placeholder ordering (asymmetry vs policy-id)
- **WHAT:** The WDT-id hashes raw placeholder indices with NO canonicalization, while `compute_wallet_policy_id` canonicalizes a clone first → `wsh(multi(2,@1,@0))` vs `(@0,@1)` produce different WDT-ids.
- **Citations:**
  - `identity.rs:71-104` `compute_wallet_descriptor_template_id` — **ACCURATE**: no `canonicalize_placeholder_indices` call; writes `use_site_path` + tree + override-TLV with raw `*idx` (`:~81 sub.write_bits(u64::from(*idx), …)`).
  - `identity.rs:172-177` policy-id DOES canonicalize a clone first — **ACCURATE** (the asymmetry).
- **STILL-REPRODUCES: YES, library-only.** Not CLI-reachable (decoder gates input ordering); a library asymmetry/footgun.
- **Fix-site:** `identity.rs:71-104` — canonicalize a clone before hashing (mirror policy-id), OR document the precondition. Cite `1a4b322`.
- **SemVer note:** this CHANGES the WDT-id VALUE for non-canonical placeholder inputs (an identity-value/semantic change). md-codec is **pre-1.0**, so under Cargo SemVer a 0.x breaking semantic change ships as a **MINOR** bump (0.38→0.39), NOT major. The 128-bit WDT-id is an in-memory/dedup identifier (spec §8.1), **not embedded in the md1 wire string** — so this does not change emitted cards, only id comparisons. Still: flag it loudly in release notes.

### L17 — test `walletpolicyid_stable_across_origin_elision` is VACUOUS (masks L14)
- **WHAT:** Both test operands carry an explicit `Shared(BIP84)` `path_decl`; the "override" is byte-identical to the baseline, so the test never exercises the elided empty-path form its name promises — which is why L14's false invariant passes CI.
- **Citations:**
  - `identity.rs:571-588` test + `:385-419` fixture — **ACCURATE**: `cell_7_wpkh_descriptor()` builds an explicit `Shared(OriginPath { components: [84',0',0'] })`; the test sets `d_override.tlv.origin_path_overrides = Some(vec![(0u8, bip84)])` where `bip84` is the SAME path extracted from the baseline. So it asserts "override-beats-baseline yields same id" — NOT "elided empty path == explicit path." Never constructs an empty `path_decl`.
- **STILL-REPRODUCES: YES.** Test is logically sound for what it tests but does NOT cover origin-elision → it masks L14.
- **Fix-site:** `identity.rs:571-588` — rewrite to build a genuinely elided empty `path_decl` and assert the real (currently differing, or — post-L14-fix — equal) behavior. Cite `1a4b322`.

### L16 — LP4-ext varint cannot encode BIP-32 child numbers ≥ 2²⁹ (valid descriptors fail to encode — graceful AVAIL gap)
- **WHAT:** `write_varint` caps at 29 bits (`VarintOverflow` for value ≥ 2²⁹), but `PathComponent.value` / `Alternative.value` carry BIP-32 child numbers up to 2³¹−1 (doc: "u31 effective range"). A child index in [2²⁹, 2³¹−1] makes an otherwise-valid descriptor fail to encode (graceful typed error, no panic/wrong-address).
- **Citations:**
  - `varint.rs:15-42` `write_varint` cap — **ACCURATE**: `:~33 if l_high > 15 { return Err(Error::VarintOverflow { value }); }` (≡ value ≥ 2²⁹). Test `varint_overflow_returns_error_instead_of_panicking` confirms `1<<30 → Err(VarintOverflow)`.
  - `origin_path.rs:16-17` doc "u31 effective range, encoded as LP4-ext varint" + `value: u32`; `use_site_path.rs:14` `Alternative { value: u32 }` — **ACCURATE**.
- **STILL-REPRODUCES: YES, graceful.** Clean typed `VarintOverflow`, never a panic/wrong-address.
- **Triage — borderline NO-fix / low-priority:** BIP-32 unhardened indices in [2²⁹ ≈ 536M, 2³¹) are LEGAL but vanishingly rare in real wallets (hardened range starts at 2³¹; standard purpose/coin/account/change/index never approach 2²⁹). This is a fidelity-completeness gap, not a funds-safety bug. Reasonable options: (a) extend varint to the full 31-bit range, OR (b) enforce/document the 2²⁹ ceiling at the parse boundary with a typed error (cheap, sufficient). Candidate to DEFER or batch as a one-line doc/guard.
- **Fix-site:** `varint.rs:15-42` (extend) or origin_path/use_site_path parse boundary (document/guard).

### D-md-chunk-budget — `chunk::split` ignores the 37-bit per-chunk header in the length budget (lib; SUB-OPTIMAL, likely NO-fix)
- **WHAT (filed):** `split` sizes chunks by raw payload bits (`SINGLE_STRING_PAYLOAD_BIT_LIMIT = 320`), ignoring the 37-bit per-chunk header in the codex32 length budget.
- **Citations:**
  - `chunk.rs:219-289` — **ACCURATE on the omission**: `split` (`:236-289`); sizing math `:249-250 payload_bit_count_for_sizing = payload_bytes.len()*8; chunks_needed = …div_ceil(SINGLE_STRING_PAYLOAD_BIT_LIMIT)` (320 = 64×5); per-chunk wire then prepends the 37-bit header (`:285 chunk_bit_count = 37 + 8*chunk_payload_bytes.len()`). The 320-bit sizing budget does NOT subtract the 37-bit header. Header confirmed 37 bits (`ChunkHeader::write` 4+1+20+6+6, `:34-58`).
- **STILL-REPRODUCES: YES but consequence is benign.** Worst case = 37 + 320 = 357 bits = 72 symbols, well under the codex32 BCH(93,80,8) **80-data-symbol / 93-codeword cap** — `wrap_payload` (which rejects > 80 symbols) NEVER overflows here. So this is **sub-optimal packing (~43 bits / ~8 symbols wasted per chunk, ≈11%), NOT an over-length defect** — it is NOT the H6/M4 over-93 problem.
- **Triage — recommend DROP or DOC-ONLY.** No correctness/funds impact; purely packing efficiency. If touched: change the sizing divisor to the header-aware usable budget (e.g. 363 bits) so fewer chunks are emitted. Lowest-value item in the cluster.
- **Fix-site (if pursued):** `chunk.rs:249-250` sizing divisor.

### D-mk-crosschunk — md-codec multi-chunk reassemble binds chunks by only a 20-bit id (defense gap; ~2⁻²⁰ swapped-chunk accept)
- **WHAT (filed):** md-codec's OWN multi-chunk reassembly binds chunks by a 20-bit `chunk_set_id` (vs mk-codec's wider/32-bit SHA approach), so chunks from two different descriptors sharing the same 20-bit id can cross-assemble (~2⁻²⁰). Defense-in-depth. (NB: despite the "mk" in the slug, the FIX is in **md-codec**.)
- **Citations:**
  - `chunk.rs` `ChunkHeader.chunk_set_id: u32` is a **20-bit** field (`:25-26` doc, `:47-50` range-reject ≥ 2²⁰, `:54 write_bits(…, 20)`, `:77 read_bits(20)`) — **ACCURATE**.
  - `derive_chunk_set_id` (`:176-180`) = top 20 bits of the 16-byte `Md1EncodingId` (`((b0<<12)|(b1<<4)|(b2>>4))`) — **ACCURATE**.
  - `reassemble` (`:306-…`) validates all chunks share `chunk_set_id`/`count`/`version` (`:~339-346 Error::ChunkSetInconsistent`) and re-derives the csid from the assembled descriptor (`:~378-383 compute_md1_encoding_id → derive_chunk_set_id`, `Error::ChunkSetIdMismatch`) — **ACCURATE**. The binding identifier is 20-bit throughout.
- **STILL-REPRODUCES: YES, genuine defense-in-depth gap.** Residual risk: a swapped chunk that reassembles into a *decodable* descriptor whose re-derived `Md1EncodingId` top-20-bits collide with the expected csid (~2⁻²⁰ ≈ 1-in-1M for random collision; lower in practice because the swapped chunk must also yield a structurally-valid descriptor that re-derives the same csid). Reachable via CLI `md decode <chunk> <chunk> …`.
- **Fix-site:** `chunk.rs` chunk-header id width — widening 20→32 bits is a **WIRE-FORMAT change** (the 37-bit header layout is fixed: 4+1+**20**+6+6; +12 bits → 49-bit header) and would break the existing chunked wire (SPEC v0.30 §2.2). This is the ONLY finding in the cluster that touches the wire format, and it carries the highest design cost. **Recommend a separate design pass (or defer)** — it is not a one-liner and needs a SPEC bump + wire-version bump, distinct from the rest of the batch.

---

## Cross-cutting observations

1. **SHA / sync clean.** `origin/main = 1a4b322`; md-codec source byte-stable at 0.38.0 since the bug-hunt report's md HEAD `54dd765`. All eight citations re-verified; only trivial off-by-1/few-line drift (M3 `:108`→`:109`, L6 indexing line). NO finding is STRUCTURALLY-WRONG. NO finding is REFUTED.

2. **Exactly one finding is funds/availability-relevant: M3.** It is fail-closed (errors, never wrong-address) but renders valid change-chain addresses underivable for a legal D5(b) None-baseline + Some-override wallet → received funds invisible to the restoring tool. This is the only item warranting the full formal R0-gated treatment in its own right.

3. **L14 + L15 + L17 are ONE coherent WalletPolicyId/WDT-id-stability sub-fix:** honest-doc-or-canonical-fill (L14) + symmetric canonicalization in WDT-id (L15) + de-vacuify the masking test (L17). They share `identity.rs`, the same root theme (canonicalization invariants the code doesn't uphold, hidden by a vacuous test), and should land as one atomic change.

4. **Two items are borderline NO-fix / doc-only:** L16 (2²⁹ child numbers are legal-but-unrealistic; a parse-boundary doc/guard is sufficient) and D-md-chunk-budget (benign sub-optimal packing, no correctness impact — the 357-bit worst case is comfortably under the 80-symbol codex32 cap). Recommend DROP-or-document both.

5. **One item is wire-format / higher-cost: D-mk-crosschunk.** Widening the 20-bit `chunk_set_id` is the only change touching the fixed 37-bit chunk-header wire layout (SPEC v0.30 §2.2) → needs a SPEC bump + chunked-wire version bump. It does not batch cleanly with the rest; recommend a separate design pass or defer.

6. **L6 is a clean one-liner** (add the existing `DivergentPathCountMismatch` guard from `expand_per_at_n`) — batches naturally with anything.

7. **Publish→pin chain (downstream version bumps):**
   - **md-codec** MINOR **0.38.0 → 0.39.0** (pre-1.0; even L15's id-value change ships MINOR). Publish to crates.io.
   - **md-cli** MUST bump its EXACT pin `version = "=0.38.0"` → `=0.39.0` and re-release (current 0.9.0 → **0.9.1**), because the path+exact-version pin won't resolve otherwise. (md-cli has no CLI-surface change from this cluster — these are all library-internal — so no manual-mirror / no new flags.)
   - **toolkit** pin-bump `md-codec = "0.37"` → `"0.39"` (or `"=0.39.0"`) + Cargo.lock; toolkit MINOR/PATCH per whether any toolkit-visible behavior changes (M3's `derive_address` widening could newly succeed where it errored — surfaces as a behavior improvement, likely PATCH unless a toolkit feature gains a new capability). **≈2-3 downstream version bumps** (md-codec 0.39.0; md-cli 0.9.1; toolkit pin-bump). GUI Cargo.lock will pull 0.39.0 transitively on its next pin bump — no GUI source change (library-internal; `schema_mirror` is flag-NAME only).

8. **SemVer protocol facts (verified):**
   - md-codec **pre-1.0** → Cargo treats `0.x.y` breaking changes as MINOR bumps. L15's WDT-id VALUE change is therefore MINOR, not major.
   - WDT-id / WalletPolicyId / Md1EncodingId are 128-bit in-memory IDENTIFIERS (spec §8 / §8.1 / §5.3), **not** embedded in the md1 wire payload — so L14/L15 do not change emitted cards, only id comparisons.
   - M3: BIP-388 multipath substitution + `<a;b>` semantics; D5(b) None+Some mix is spec-legal (`validate.rs`). Fix is a gate-widening; no wire change.
   - L16: BIP-32 child numbers are 0..2³¹−1 (hardened bit at 2³¹); [2²⁹,2³¹) is legal-but-rare.
   - D-mk-crosschunk: SPEC v0.30 §2.2 fixes the 37-bit chunk header (4+1+20+6+6); a wider id is a wire change.

---

## Recommended cycle-10 scope

**Split the cluster into a FORMAL subset + a BATCH subset + DEFER/DROP, per funds-relevance and wire-cost:**

- **FORMAL (own R0-gated treatment — funds/availability):**
  - **M3** — `derive_address` chain-gate widening. The single funds-relevant item; trace-confirmed reproduces. ~10-20 LOC + a None-baseline+Some-override change-chain derive test. **This anchors the cycle.**

- **BATCH together with M3 (library-internal, low-risk, same crate, one md-codec 0.39.0 release):**
  - **L14 + L15 + L17** — the coherent WalletPolicyId/WDT-id-stability sub-fix (honest-doc-or-fill + symmetric WDT canonicalize + de-vacuify test). One atomic change in `identity.rs`; flag L15's id-value change in release notes. ~30-50 LOC.
  - **L6** — one-line `DivergentPathCountMismatch` length guard in `canonicalize_placeholder_indices`. Trivial.

- **DROP / DOC-ONLY (recommend NOT spending a fix on):**
  - **L16** — close as "document the 2²⁹ ceiling / known-limitation" or a cheap parse-boundary guard; 2²⁹ unhardened children are unrealistic. (If the user wants completeness, a small varint extension is acceptable but low value.)
  - **D-md-chunk-budget** — benign sub-optimal packing, NO correctness/funds impact, comfortably under the codex32 cap. Close as won't-fix or a one-line sizing-divisor tweak only if a chunk-count reduction is independently desired.

- **DEFER to a separate design pass (wire-format cost):**
  - **D-mk-crosschunk** — widening the 20-bit `chunk_set_id` is the only wire-format change (SPEC v0.30 §2.2 + chunked-wire version bump); does not batch with the library-internal items. Defense-in-depth, ~2⁻²⁰, partially mitigated by the reassemble-time `compute_md1_encoding_id` re-derive cross-check. Track as its own FOLLOWUP/cycle.

**Net recommended cycle-10 = M3 (formal anchor) + L14/L15/L17 + L6 (batched), shipped as md-codec 0.39.0 → md-cli 0.9.1 (pin bump) → toolkit pin-bump.** L16 + D-md-chunk-budget → doc/won't-fix; D-mk-crosschunk → separate wire-format cycle.

**SemVer:** md-codec **MINOR 0.39.0** (pre-1.0; L15 id-value change rides MINOR). **md-cli** re-release **0.9.1** (exact-pin bump only, no CLI surface change). **toolkit** pin-bump (PATCH/MINOR per behavior surfaced; M3's widening is a strict capability gain).

**Locksteps:** NO `schema_mirror` (no clap flag-name change). NO manual-mirror (`docs/manual/src/40-cli-reference/` — no CLI-surface change; all library-internal). Sibling-codec FOLLOWUP companions only if D-mk-crosschunk's mk-codec contrast is formalized (mk-codec uses the wider id — a companion note may be warranted when the wire-format pass is scheduled).

---

## R0 gate reminder (project standard)

This is recon only. Before ANY implementation, the cycle-10 brainstorm spec + plan-doc MUST pass an opus architect R0 review and converge to **0 Critical / 0 Important** — fold → persist verbatim to `design/agent-reports/` → re-dispatch until GREEN. No code, no implementer dispatch, no phase advance, no tag/ship while any Critical/Important is open. (CLAUDE.md Conventions, first bullet.)
