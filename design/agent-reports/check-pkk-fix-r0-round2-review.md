# R0 Review — toolkit Check-PkK non-tap canonical fix (round 2)

Reviewer: Fable 5 architect agent (ab2b9fd546744c3e1), 2026-06-12.
Target: design/BRAINSTORM_check_pkk_non_tap_canonical_fix.md (R1 fold).
Persisted verbatim per CLAUDE.md convention.

## Verdict: GREEN

0 Critical / 0 Important. Both round-1 Importants resolved + folds empirically proven end-to-end. Make-or-breaks PROVEN: (I1) `Verdict::Match` is assigned ONLY when both tools succeed + produce equal ids (`classify()` :228-242 `(Some(a),Some(b)) if a==b`), so the restructured guard's `n_bothError==0 && n_toolError==0` genuinely catches the both-error false-Match risk; (I2) the in-crate `reassemble → compute_wallet_policy_id` path compiles and emits exactly the four goldens. Implementer-ready. Minors are citation-decay + a leg-2 scope clarification — none gate.

## Critical / Important
- None.

## Minor
- **m1 — leg-1's cited decode fn is wrong for a chunk array.** `bundle --json` `.md1` is a CHUNK ARRAY (`Vec<String>`, 3 chunks for wsh(pk)); `decode_md1_string` takes a single string. Use `md_codec::chunk::reassemble(&[&str]) -> Result<Descriptor, Error>` (the toolkit's own idiom, bundle.rs:1070/2070). Built+ran with `reassemble → compute_wallet_policy_id` → all 4 goldens. Fix the one-word citation.
- **m2 — `prop_backup_restore_roundtrip` mis-located.** It's in its OWN integration file `crates/mnemonic-toolkit/tests/prop_backup_restore_roundtrip.rs`, fn `backup_restore_roundtrip` (:408), non-#[ignore], O2 fixed-point at :431. Ran 9/9 green post-fix. Fix the file path.
- **m3 — line citations decayed (re-grep at impl).** Live @ master: gate `if tap_context` :602; the 4 AST-test ASSERT lines :1465/:1527/:2557/:2579 (brainstorm cites fn-decl :1457/:1519/:2550/:2564 — grep by name). Also a STALE duplicate "8-site" release-mechanics paragraph coexists with the folded "6-site" one — trim the older.
- **m4 — leg-2 NOT redundant with the prop test; target `wsh(pk)`/`wsh(pkh)`.** The prop generator only places pk/pkh INSIDE combinators — never a bare top-level wsh(pk)/wsh(pkh) (the flagship shape leg-1 pins). Toolkit round-trip on wsh(pk) IS constructible: post-fix `bundle --descriptor wsh(pk(...)) → restore --md1` reconstructs `wsh(pk(...))` + valid bc1q addresses. Make leg-2 explicitly round-trip wsh(pk).

## Fold verification table
| Round-1 finding | Resolved? | Notes |
|---|---|---|
| I1 vacuity guards | YES (proven + necessary) | Post-fix differential: all 4 entries report Match (toolkit==md-cli); 0 Diverge → old `n_diverge>=1`/`saw_diverge` panic. Restructure valid: Match only on (Some,Some)&&equal; BothError/ToolError arms exist → `n_bothError==0 && n_toolError==0` catches the real risk. FOLLOWUPS grep: no other genuine divergence → restructure not new-entry. Implementer adds the 2 counters in the per-entry loop (:388-392). |
| I2 emit path + in-crate decode | YES (proven) | `bundle.md1` = Vec<String> (bundle.rs:934); `compute_wallet_policy_id`/`compute_wallet_descriptor_template_id` pub (md-codec lib.rs:51), take &Descriptor; locked md-codec 0.35.1; hex 0.4 dep. Built+ran `reassemble→compute_*→hex` → 4 goldens. (m1: reassemble not decode_md1_string.) |
| M1 sh(wsh(pk)) | YES | added (brainstorm :41 + decision 5). |
| M2 prop test cited | YES (m2/m4 caveats) | cited; mis-located path (m2); not redundant for wsh(pk) (m4); 9/9 green post-fix. |
| M3 6-site lockstep | YES | folded "6-site" at the new paragraph; trim the stale "8-site" duplicate (m3). |
| M4 rename + diff-comment refs | YES | leg-3 says rename + fix cli_cross_tool_differential.rs:20/:300 `…kept…:2551` refs. |

## Evidence log
- Goldens re-derived (post-fix scratch build): pre-fix toolkit wsh(pk)→`9ad78e4f` (≠); md-cli + post-fix toolkit (gate dropped) via in-crate reassemble→compute_*: wsh-pk `58d18033/9208f590`, wsh-pkh `3d6fb9a1/1499fe49`, wsh-and_v `a513edb6/cb13e9cd`, wsh-or_d `aa4bbe01/247773f7` — EXACT. Leg-1 fails pre-fix / passes post-fix.
- In-crate path: `tests/r0_scratch_golden.rs` `reassemble→compute_wallet_policy_id/compute_wallet_descriptor_template_id→hex::encode(.as_bytes())` GREEN, printed 4 goldens.
- I1: post-fix differential wpkh/pkh/wsh-multi/tr-pk-leaf still Match; the 4 now Match (`EXPECTED Diverge but got Match` ×4). classify() :228-242 Match only (Some,Some)&&equal; BothError/ToolError arms exist.
- Param removal: full removal (sig :551, helpers :706/:719, gate :602, call sites :432/:444/:456/:519/:523) → cargo build clean; walk_miniscript_node/walk_one_child/walk_two_children private, walk_root unchanged → no surface impact. Low-risk; keep-unused fallback unnecessary.
- Blast radius: full --bin suite post-fix 963 pass / 4 fail = exactly walk_wsh_pk_root:1465, walk_sh_ms_pk_root:1527, walk_check_kept_in_non_tap_context:2557, walk_pk_h_via_wsh_andor:2579. 117 integration + prop 9/9 green. No other golden depends on old form.
- Round-trip leg-2: post-fix bundle wsh(pk) → restore --md1 reconstructs wsh(pk(…/<0;1>/*))#mpjep0z8 + bc1q addrs (depth-0 xpub = known v0.49.1 behavior).
- FOLLOWUPS grep: toolkit-check-pkk… (FOLLOWUPS.md:29) sole pinned divergence; :34 lists same goldens; :39 confirms the flips. No other to add.
- Both worktrees restored pristine; git status clean both repos.
