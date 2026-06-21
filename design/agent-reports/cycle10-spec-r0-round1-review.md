# R0 REVIEW — cycle-10 md-codec LIBRARY cluster (M3 + L14/L15/L17 + L6) — Round 1

**Spec:** `design/BRAINSTORM_cycle10_mdcodec_lib.md`
**Branch point:** `descriptor-mnemonic` `origin/main = 1a4b322` (verified). All md-codec source is **byte-identical** between this SHA and the repo's working-tree HEAD (`836faf87`) — the only delta is in `crates/md-cli/` — so every cited line number is authoritative.

## VERDICT: **GREEN — 0 Critical / 0 Important**

Every load-bearing claim verified against source. Two Minor notes below; neither blocks implementation.

---

## M3 (funds-availability) — fail-closed property VERIFIED, highest scrutiny

The funds-critical question — *can widening the gate ever derive a WRONG address?* — is **NO**, confirmed by full code trace:

- **Bug confirmed.** `derive.rs:110-122`: gate reads ONLY `self.use_site_path.multipath`. A `None`-baseline + `Some(<0;1>)`-override wallet takes the `else if chain != 0` arm (line 117) and rejects `chain=1` with `alt_count: 0` — even though `@0`'s override carries a real change chain.
- **Legality confirmed.** `validate.rs:127-140` skips `None` entries (the `if let Some(alts)` guard) and only rejects two `Some` groups with *different* alt-counts. The `None`-baseline + `Some`-override mix is the symmetric case of the documented D5(b) example (`validate.rs:118-127`) and is legal in **both** directions.
- **Fix is sufficient AND fail-closed.** The override genuinely carries the derivation multipath: `expand_per_at_n` (`canonicalize.rs:458-460`) sets each `ExpandedKey.use_site_path = override-if-present-else-baseline`. `to_miniscript_descriptor` (`to_miniscript.rs:54-66`) passes EACH key's own `&e.use_site_path` to `build_descriptor_public_key`, and `use_site_to_derivation_path` (`to_miniscript.rs:277-292`) re-resolves `chain` against THAT key's multipath via `alts.get(chain)` — still erroring `ChainIndexOutOfRange` if the specific key lacks the alt (`:282`). So the pre-flight is a *coarse necessary* gate; the per-key path is the *sufficient authority*. Widening to MAX-alt-count can only let through requests that the per-key path then derives correctly or rejects — never a wrong subtree.
- **No bypass.** `derive_address` calls `to_miniscript_descriptor` (single-path), never the faithful-string `to_miniscript_descriptor_multipath`. The fail-closed per-key check is on the actual derivation path.
- **No over-widening.** `chain >= max_alts` still rejected by the widened pre-flight (the requested positive control). The `None->1` model exactly reproduces the old `else if chain != 0` semantics.
- **Regression preserved.** `derive_address_chain_out_of_range` (`derive.rs:241-267`): no overrides -> `max_alts == baseline_alts == 2` -> still `alt_count: 2`. Unchanged. **D2 confirmed** — no `to_miniscript.rs`/`expand_per_at_n` change needed.

Type-check: `UseSitePath { multipath: Option<Vec<Alternative>> }` (`use_site_path.rs:49-54`) and `use_site_path_overrides: Option<Vec<(u8, UseSitePath)>>` (`tlv.rs:26`) — the spec's `.iter().flatten().map(|(_, p)| p.multipath.as_ref().map(|a| a.len()).unwrap_or(1))` is correct.

## L14/L15/L17 (identity-stability) — VERIFIED, including the SemVer determinant

- **SemVer determinant — the in-memory-vs-wire claim is TRUE.** `encode.rs` (containing both `encode_payload:65` and `encode_md1_string:136`) has **zero** references to any of the three ids. The ids are SHA-256-derived in-memory comparison/bind keys only. Changing their VALUE does **not** alter any emitted md1 card -> **MINOR (pre-1.0), not wire-breaking MAJOR**. **D8 holds.**
- **L14 canonicalizer correct.** `canonical_origin(&d.tree) -> Option<OriginPath>` (`canonical_origin.rs:45`) returns `Some(84'/0'/0')` for `wpkh(@N)`. Empty-only fill is right; fast-path safe: explicit-origin inputs unchanged.
- **L15 mirror correct, no width desync.** `canonicalize_placeholder_indices` early-returns `Ok(())` on the identity permutation (`canonicalize.rs:198-201`); `key_index_width()` is a pure function of `d.n` (`encode.rs:37-41`) preserved by canonicalization — no bitstream desync.
- **L17 vacuity confirmed + fix exercises the real path.** Current test (`identity.rs:571-588`) never exercises elision; the proposed replacement sets `path_decl = Shared(OriginPath{components: vec![]})` -> RED today, GREEN after L14.

## L6 (panic guard) — VERIFIED

`canonicalize.rs:206` indexes `old_paths[inverse[new_idx]]` (`:216`) with **no** `paths.len() == n` guard — OOB panic on a short hand-built Divergent vector. Sibling `expand_per_at_n:425-432` has exactly the guard to mirror, returning the **existing** `Error::DivergentPathCountMismatch { n, got }` (`error.rs:64-71`). No new variant. **D6 holds.**

## SemVer / publish->pin chain — VERIFIED

- md-codec `0.38.0` -> MINOR `0.39.0`, publish FIRST.
- md-cli exact pin `version = "=0.38.0"` (`md-cli/Cargo.toml:28`) — hand-edit to `=0.39.0`, no code change. Only ONE md-codec version literal in the workspace; `fuzz/Cargo.toml` path-only (`:22-23`).
- Toolkit pin `md-codec = "0.37"` (`mnemonic-toolkit/Cargo.toml:36`) -> PATCH bump.
- No `schema_mirror`, no manual-mirror. **D10 holds.** L16/D-md-chunk-budget/D-mk-crosschunk correctly held out. **D9 holds.**

## Minor findings (non-blocking — fold into plan-doc)

- **M-1 (citation):** spec's md-cli base version `0.9.0` is correct at `origin/main:1a4b322`. The repo's working-tree HEAD (`836faf87`) is one commit **behind** origin/main and shows md-cli `0.8.1` — **the implementer must branch from `origin/main`, not HEAD**, or the pin-bump base version is wrong.
- **M-2 (fmt gate):** CI enforces `cargo fmt --all --check` (`ci.yml:59`). The plan-doc must carry a "run `cargo fmt`" step before the green-suite/tag.

---

**Disposition:** R0-GREEN (0C/0I). The funds-critical M3 fix is provably fail-closed; L14/L15 SemVer rests on a verified in-memory-only fact; L6/L17 are mechanical and accurate. Implementation may proceed to the plan-doc stage (own R0 gate). Fold the two Minor notes into the plan-doc.
