# R0 Review (R1) — IMPLEMENTATION_PLAN_toolkit_mk1_origin_path.md

Opus architect, continuing from plan-R0 (RED 1C/2I/5M). Verified the fold against live
toolkit + mk-codec 0.4.0 registry source. Persisted by controller.

## Fold verification
- **C1 (overlap-prefix Check 1) — RESOLVED** on all 4 axes: (a) compiles — `card.origin_path:
  DerivationPath` (`key_card.rs:24`) `.into_iter().copied()`→ChildNumber vs `md_path.components[k]:
  PathComponent {value:u32, hardened:bool}` (`origin_path.rs:18-24`); (b) bounds-safe (`overlap=min`);
  (c) logically correct — 3→4 (mk1⊆md1 silent), 4→3 (md1⊆mk1 silent — the case Check A wrongly
  failed), 4→4 (equal silent), tampering (overlap-disagreement fires); (d) Task 1.3 covers 4→3.
- **I1 RESOLVED** — `MkField #[serde(untagged)]` (`format.rs:64-70`) → single-sig flat array; `json["mk1"]` as Vec<String>.
- **I2 RESOLVED** — `:2653`/`:2731` ARE the shared `path` bindings feeding both `synthesize_full`
  (`:2688`/`:2764`) AND `synthesize_multisig_watch_only`; change to `m/84'/0'/0'` fixes both.
- **M1/M2/M3 RESOLVED** — new subsuming slug + drop dangling ref; local `MAINNET_WIF`; `ChildNumber`
  test import. mk-codec 0.4.0 variant `{xpub_depth:u8, path_depth:u8, xpub_child:ChildNumber,
  path_child:Option<ChildNumber>}` (`error.rs:176-185`) matches both the friendly.rs arm + test ctor;
  both mirror insertion points confirmed (`friendly.rs:124` `_=>`, `error.rs:393` `_=>1`).

## Fold-drift checks
- §3.5b parent-fp gate `if d-1 > full.len() { continue }` CORRECT — admits `d==full.len()+1` (4→3 leaf:
  `full[..d-1]`=all=parent), no underflow (gated `d>=2`); strictly more correct than the old `[..len-1]`.
- Check 2 depth-1 `claimed_master_fp` fallback preserved; M5 depth-0 child==0 drop safe (decoded card
  always has `child==path.last()` → old terminal checks tautological; genuine depth-0 covered by Check 2 d==0).
- 3→0 (overlap=0) correctly silent — md1 origin empty whenever mk1 padded; nothing to disagree on.
- **Overlap-prefix hides NO real inconsistency:** both legit shapes are prefix-containment; a genuine
  tamper within the shared prefix fires; key-identity (right path, wrong key) delegated to `mk1_xpub_match`.
  The old depth-strict check caught no additional real inconsistency — only added false positives.
- SPEC §3.5 ↔ plan Task 1.1 consistent; remaining "Check A/B" strings are historical fold-annotations.

## CRITICAL — None.  ## IMPORTANT — None.
## MINOR
- **M-a** SPEC line 9 stale `Resolves: mk1-wif-bundle-depth0-invalid-card` (folded post-review).
- **M-b** reject-loop range `:494-503` (SPEC) vs `:494-506` (plan) — body closes `:506`; harmless.

## VERDICT: GREEN (0C/0I/2M) — clear to implement.
The overlap-prefix redesign is strictly safer (removes false positives, hides nothing); all folds
verified against source. The 2 Minors are doc drift. The plan-doc gate is cleared; per the mandatory-R0
standard, the end-of-cycle R0 still gates ship.
