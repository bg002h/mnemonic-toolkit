# BRAINSTORM — toolkit walker emits bare PkK/PkH in non-tap (wire fix, v0.55.0)

Status: R2 **GREEN (0C/0I)** — cleared for implementation. 2026-06-12.
Reviews: check-pkk-fix-r0-round1-review.md (YELLOW 0C/2I — folded
`[I1]`/`[I2]` + M1-M4) and …-round2-review.md (GREEN 0C/0I — the whole fix
+ all 4 test legs proven end-to-end in scratch; 4 citation/scope minors
m1-m4 folded). Goldens empirically derived via md-cli's already-conformant
output. Resolves FOLLOWUP
`toolkit-check-pkk-non-tap-non-canonical` (Cycle-D find; companion in
descriptor-mnemonic). Source SHA: da5c162 (origin/master).

## Problem

The toolkit descriptor walker `walk_miniscript_node(.., tap_context)`
(parse_descriptor.rs ~601-624) GATES the `Terminal::Check(PkK|PkH) → bare
Tag::PkK|PkH` collapse on `tap_context`: it collapses inside tap leaves but
KEEPS `Tag::Check(Tag::PkK)` in wsh/sh. descriptor-mnemonic SPEC v0.30 §5.1
mandates bare `PkK`/`PkH` "regardless of context; `Tag::Check` is never
emitted wrapping a key leaf on the wire", and md-cli collapses
unconditionally. So the toolkit emits a NON-CONFORMANT md1 for
`wsh(pk)`-shaped descriptors → a DIFFERENT `wallet_policy_id` than md-cli
for the same wallet (interop hazard; wire-canonicity, NOT funds-loss — both
forms decode to the identical descriptor). Cycle-D proved it:
`wsh(pk)` → toolkit policy-id `9ad78e4f` vs md-cli `58d18033`.

## Recon (2026-06-12; verified)

- **The gate is an unexamined artifact.** Introduced in v0.3.0 A.4
  (port-from-md-cli, commit 6502da5), test-pinned in A.5 (commit 3dfca1c,
  `walk_check_kept_in_non_tap_context`), but NO stated rationale in any
  comment / design doc / git log. md-cli (the port source) never gated.
  → no countervailing reason; the fix is unblocked.
- **Round-trip SAFE.** md-codec 0.35.1 `to_miniscript` accepts BOTH forms:
  bare `Tag::PkK` re-wraps as `Check(pk_k)` at the miniscript layer
  (to_miniscript.rs:290-303), and the `Tag::Check`-over-bare-key
  idempotence arm (:304-324) collapses `Check(PkK)` to the same `c:pk_k`.
  Both wire forms → identical miniscript AST. The toolkit restore path
  (cmd/restore.rs) uses the same md-codec arm. Cycles D+E proved both forms
  `md decode` to the same descriptor and derive the same addresses
  (bitcoind-confirmed).
- **Blast radius (8 shapes change wire output):** `wsh(pk)`, `wsh(pkh)`,
  `wsh(and_v(…pk…))`, `wsh(or_d(…pk…))`, `wsh(thresh(…pk…))`, `sh(pk)`,
  `sh(pkh)`, `sh(wsh(pk))` [M1] — anything with a `Check(PkK|PkH)` in
  non-tap context (the list is the reachable set, not exhaustive of every
  combinator nesting).
  **UNAFFECTED (common shapes):** wpkh / pkh top-level (Layer-1, no
  miniscript walk), sh(wpkh), wsh(multi) / wsh(sortedmulti) (use MultiKeys
  not Check), tr / tap leaves (already collapse). Confirm at impl time.
- **No GUI / schema-mirror / manual impact** (internal walker logic; no
  clap surface change).
- **SemVer = MINOR (0.54.4 → 0.55.0)** — wire-content change for a class of
  descriptors, no CLI/schema change, no card-READING regression. Precedent:
  v0.48.0 (the tr-multisig NUMS wire change, signalled MINOR).

## The fix

Drop the `if tap_context` gate around the Check→bare collapse in
`walk_miniscript_node` (the SOLE consumer of the param, parse_descriptor.rs
:602): collapse `Terminal::Check(PkK|PkH) → bare Tag::PkK|PkH`
UNCONDITIONALLY (matching md-cli template.rs:607). The fall-through
`Tag::Check` emit (:621) is PRESERVED for `Check(<non-key>)` — only
Check-over-bare-key collapses. One-arm change.

**Remove the `tap_context` param [OQ1].** R0 confirmed it's used ONLY at the
gate (everything else threads it; entry points set false@wsh/sh,
true@tap). After dropping the gate it's dead → remove it (mechanical, ~15
call sites: the fn sig :558, the recursion :622/:662-689, the
walk_one_child/walk_two_children helpers :710/:724-725, and the
false@432/444/456 + true@519/523 call sites). The honest end-state; the fix
itself stays one-arm.

## TEST COVERAGE FOR THE WIRE CHANGE (first-class — user mandate
"if we change wire format, make sure our tests cover it")

A wire change MUST be (a) explicitly captured (the NEW bytes pinned), (b)
proven round-trippable, (c) proven cross-tool-equal, (d) AST-asserted. All
four:

1. **NEW live golden wire vector [the headline gap — IN-SUITE, ALWAYS-ON
   primary gate].** Add a LIVE characterization test (read by the normal
   suite — NOT an orphaned vector, NOT #[ignore]-gated, NO external binary —
   per the Cycle-A orphaned-vectors lesson) pinning the EXACT post-fix
   `wallet_policy_id` (+`wallet_descriptor_template_id`) for the affected
   shapes, for the FIXED concrete xpub below. **Emit path [I2]:
   `bundle --descriptor <D> --network mainnet --json` → `.md1` chunk array**
   (NOT build-descriptor — it emits no md1/policy_id + is wsh-only).
   **Decode IN-CRATE** [I2/m1]: `md_codec::chunk::reassemble(&[&str])` on the
   `.md1` chunk array (NOT `decode_md1_string` — that takes a single string;
   `reassemble` is the toolkit's own idiom for the chunk array) →
   `md_codec::compute_wallet_policy_id(&desc)` +
   `compute_wallet_descriptor_template_id(&desc)` → `hex::encode(id.as_bytes())`
   (the toolkit already deps md-codec 0.35.1 + hex 0.4; no `md inspect`/
   MD_BIN). Hard-code the EMPIRICALLY-DERIVED post-fix
   goldens (frozen xpub `xpub6DkF…r6KFrf` mfp `73c5da0a` m/48'/0'/0'/2';
   @1 = `xpub6Dzhy…BXd6Vk`):
   | shape | post-fix wallet_policy_id | template_id |
   |---|---|---|
   | `wsh(pk(@0))` | `58d1803363f5599914a9f4ba0afa97d7` | `9208f59035e4912d4fca8182a897fafb` |
   | `wsh(pkh(@0))` | `3d6fb9a1656b02b36378645aaea9633e` | `1499fe4902eaa084c9574ed33b7fc109` |
   | `wsh(and_v(v:pk,pk))` | `a513edb6343f69ca59841187a567a5ee` | `cb13e9cd9a18a72e538a41482f562da8` |
   | `wsh(or_d(pk,pk))` | `aa4bbe01269571d7e5940f542a3b0a3c` | `247773f7bc8f1e637d2c6f6163f811c5` |
   These are md-cli's already-conformant output (template.rs:607 collapses
   unconditionally), == the post-fix toolkit. Re-derive at impl time to
   confirm, then freeze. **Golden FORM = policy_id+template_id, NOT raw
   md1-string** [OQ2] (toolkit chunks, md-cli single-strings — md1 not
   cross-tool-comparable; policy_id/template_id are tool-agnostic, stable,
   and ARE what diverged). This in-suite literal golden is the ALWAYS-ON
   absolute-value gate that fails loud on any future re-gate.
2. **Round-trip test (toolkit-level) — target `wsh(pk)`/`wsh(pkh)` [m4].**
   Build the new bare-PkK md1 (`bundle --descriptor wsh(pk(…))`) → `restore
   --md1` → assert the recovered descriptor (`wsh(pk(…/<0;1>/*))#csum`) +
   addresses match. Proves the toolkit reads its OWN new wire form. NOT
   redundant with the prop test [m4]: the prop generator only places pk/pkh
   INSIDE combinators — never a bare top-level `wsh(pk)`/`wsh(pkh)` (the
   flagship shape leg-1 pins), so leg-2 covers what the prop test
   structurally can't. (R0 proved the round-trip: post-fix `bundle
   --descriptor wsh(pk(…)) → restore --md1` reconstructs `wsh(pk(…))` + valid
   bc1q addresses.)
3. **Cross-tool wire equality [Cycle-D differential FLIPS + VACUITY-GUARD
   RESTRUCTURE — I1].** Flip the 4 Cycle-D entries
   (`wsh-pk`/`wsh-pkh`/`wsh-and_v`/`wsh-or_d`,
   cli_cross_tool_differential.rs:304/312/320/331) `Verdict::Diverge →
   Match`. **CRITICAL [I1]:** that removes the ONLY Diverge entries, and the
   test's anti-vacuity guards (`n_diverge>=1` :370-374, `saw_diverge`
   :423-426) would then PANIC. No other known toolkit-vs-md-cli divergence
   exists today. So RESTRUCTURE the guards to a verdict-agnostic
   non-vacuity check: keep `n_match>=1` + `saw_match` (proves real
   agreement happened, not all-error), DROP the hard `n_diverge>=1` /
   `saw_diverge` requirement, and assert `n_bothError==0 && n_toolError==0`
   (the real vacuity risk = both tools erroring → false match). Comment:
   the canonicity fix landed; the harness is now a cross-tool MATCH
   regression gate (catches a FUTURE re-divergence). Drop the FOLLOWUP
   citation from the flipped entries; rename refs to the renamed AST test
   [M4]. The differential is #[ignore]-gated (cross-tool-differential.yml
   only) → it's the cross-tool CONFIRMATION; leg-1 is the in-suite primary.
4. **AST unit tests [invert the 4 + 1].** parse_descriptor.rs tests
   currently asserting `Tag::Check`: `walk_wsh_pk_root` (:1457),
   `walk_sh_ms_pk_root` (:1519), `walk_check_kept_in_non_tap_context`
   (:2550 — the deliberate test; RENAME to `walk_check_collapsed_in_non_tap`
   + invert to assert bare `Tag::PkK`), `walk_pk_h_via_wsh_andor` (:2564 —
   note: uses explicit `c:pk_h(@0)` → post-fix `wsh_kids[0].tag` becomes
   `Tag::PkH` DIRECTLY, dropping one nesting level — invert accordingly).
   Each flips `Tag::Check` → bare `Tag::PkK`/`PkH`.

Plus [M2/m2]: cite `crates/mnemonic-toolkit/tests/prop_backup_restore_roundtrip.rs`
(fn `backup_restore_roundtrip` :408, NORMAL-suite, non-#[ignore]) as
SUPPLEMENTARY always-on coverage — it generates pk/pkh leaves INSIDE
combinators through `bundle --descriptor`→restore with an O2 md1 FIXED-POINT
oracle (:431); round-trip-safe ⇒ stays green (but O2 is a fixed-POINT,
self-consistent, so leg-1's literal golden is the absolute-value gate, the
prop test the structural-stability gate). Re-run the full suite; confirm the
common-shape tests (wpkh/multi/tr goldens) are UNCHANGED (proves blast radius
contained — recon found NO other golden depends on the old form; R0 r2 proved
exactly the 4 AST tests fail post-fix, nothing else).

## Resolved decisions (round-1 R0 answers, adopted)

1. `tap_context` param: SOLE consumer is the gate → REMOVE it [OQ1].
2. Golden form = `wallet_policy_id` + `wallet_descriptor_template_id`
   (NOT md1-string; chunking differs) [I2/OQ2]. Empirical post-fix values
   frozen in leg-1 above.
3. Emit path = `bundle --descriptor --json` → `.md1`, decoded IN-CRATE via
   `md_codec::compute_wallet_policy_id` (NOT build-descriptor, NOT
   `md inspect`) [I2/OQ3].
4. SemVer = MINOR (0.54.4 → 0.55.0); v0.48.0 "SemVer-MINOR (wire-content
   change)" precedent [OQ4].
5. Blast radius = 8 shapes (added sh(wsh(pk)) [M1]); single Check arm;
   Check-over-non-key preserved [OQ5].
6. Only cli_cross_tool_differential.rs (the 4 entries) + the 4 AST unit
   tests pin changing output; + prop_backup_restore_roundtrip stays green
   (always-on supplementary). No other golden depends on the old form [OQ6].

## Release mechanics (folded)

v0.54.4 → **0.55.0** MINOR. 6-site lockstep [M3]: Cargo.toml version,
Cargo.lock, README.md + crate README `<!-- toolkit-version -->` markers,
install.sh tag, CHANGELOG `[0.55.0]` (describe the wire change + the 8
affected shapes + "restore reads both old Check(PkK) and new bare-PkK md1
via md-codec's idempotence arm — no card-reading regression"). Resolve
`toolkit-check-pkk-non-tap-non-canonical` in BOTH repos + the cross-linked
descriptor-mnemonic v2-design-questions item. Tag `mnemonic-toolkit-v0.55.0`
(no crates.io publish — toolkit is git-tag-only).
