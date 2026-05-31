# R0 Review (R1) — SPEC_toolkit_mk1_origin_path.md v2

Opus architect, continuing from v1 R0 (RED 2C/3I/3M). Verified the fold against live
toolkit source + the mk-codec 0.4.0 registry crate. Persisted by controller.

## Fold verification
- **C1 — RESOLVED.** Traced §3.5 against live `emit_watch_only_xpub_path_cross_check`
  (false-positives confirmed: Check 1 `:2119` `3≠4`, Check 2 `:2135` child `0'` vs
  `md_path.last()=2'`) and `emit_full_path_parent_fingerprint_check` (`:2379-2381`
  derives parent at `full[..md_depth-1]` → wrong level). §3.5a Checks A/B/C + §3.5b
  `full[..d-1]` go silent on correct 3→4 (Check B reads `components[d-1]=components[2]=0'`
  == xpub child; parent at `full[..2]=m/48'/0'` = correct preserved parent_fingerprint
  via `reconstruct_xpub xpub_compact.rs:103`). **Index-safe:** Check B gated `d≤md_depth`
  (`d-1 < len`); parent-fp gated `d<2 skip` + `d>full.len() skip` (`1≤d-1<len`). Still
  fires on genuine tampering.
- **C2 — RESOLVED (Minor M3).** Guard makes `encode(&inconsistent)` impossible; the
  consistent-cards-disagree precedent (`cross_check_mk1_child_number_ne_md1_last_warns:290-324`)
  rebuilds without a bypass encoder. A consistent depth-2 card is constructible. M3: the
  rebuilt fixture must pin a depth-2 terminal that DIFFERS from `md1.components[d-1]` (else
  Check B won't fire) — constructible, needs divergent values.
- **I1 — RESOLVED.** `xpub6FQya…` is the depth-4 `m/48'/0'/0'/2'` leaf (empirical decode +
  convergence-test corroboration). `tr_multi_a_nums_2of3` read directly: descriptor-passthrough
  (no seed) feeding the depth-4 key annotated with depth-3 `[87'/0'/0']` → genuine 4→3, NOT an
  anomaly. Decisive: the 0.4.0 guard checks ONLY count→depth + last→child; intermediates never
  validated → helper extend-by-append provably `len==depth && last==child` → encodes regardless
  of fictional intermediate. Near-homogeneous; no per-test audit (snapshot regen + 2 fixture fixes).
- **I2 — RESOLVED.** `bind_descriptor_keys` (`parse_descriptor.rs:901`) carries
  `#[allow(dead_code)]` (`:895`), ZERO production callers (all in `#[cfg(test)]` `:1259+`).
  v1 citation was wrong; §3.7 corrects it. Helper-pad sound + lower-blast-radius than bind-default
  (which feeds md1 path_decl `:710-717` → policy_id). All production mk1 emission funnels through
  the 4 live `KeyCard::new` sites → helper normalizes 3→0 regardless of source.
- **I3 — RESOLVED.** `synthesize.rs:12` lacks `ChildNumber`; §3.2/§2 add it. Helper APIs exist
  in-tree (`master()`, `from(Vec)`, `Normal{index}`).

## New findings
- Helper formula hand-verified for depth-0 (empty), depth-1 (`[child]`), + all 6 classes —
  every output `len==depth && last==child` ⟹ encodes (cross-checked vs guard + reconstruct).
- 8 sites + live/test split ACCURATE; reject loop at `:495-506`; md1 path_decl `:710-717` unchanged.
- **SemVer PATCH / no lockstep CONFIRMED:** `bundle --json` origin fields derive from
  `ResolvedSlot::origin_path_bare()` (`bundle.rs:642-648`) = descriptor path, INDEPENDENT of the
  mk1 card path. The fix changes only the `KeyCard::new` argument. mk1 card path surfaced only by
  `inspect --mk1` (plaintext, not a gated JSON wire field). No GUI/manual lockstep.
- No internal contradictions (§3.5 dual of §3.1; census table ↔ §3.2 behavior table).

## CRITICAL — None.  ## IMPORTANT — None.
## MINOR
- **M1** §3.5a `split_child(...)` is illustrative — no such helper in `verify_bundle.rs`; inline
  the `ChildNumber` match (live `:2131-2134`) or add the helper. Pin in plan.
- **M2** §3.5a Check C is prose-only; rekeying the depth-0/depth-1 branches (`:2167-2195`) from
  `md_depth` to `d` must preserve the depth-1 `claimed_master_fp` fallback (`md_fp_for` ∨
  `origin_fingerprint`). Make explicit in plan Phase 1.
- **M3** §3.6 rebuilt fixture must pin divergent depth-2-child vs md1.component[1] (see C2).

## VERDICT: GREEN (0C / 0I / 3M)
All five prior findings resolved + verified. §3.5 redesign logically correct + index-safe; I1
settled (helper extends, near-homogeneous); PATCH/no-lockstep correct; no fold drift. The 3 Minors
are plan-phase precision (pseudocode helper, Check C d-keying, fixture divergence). Clear to the
plan-doc — which itself needs its own R0 → 0C/0I before any code.
