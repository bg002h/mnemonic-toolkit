# R0 Review — general-tr faithful restore (PLAN Rev 2) — Round 2

Reviewer: Fable 5, 2026-06-12. Verified against mnemonic-toolkit `origin/master` = `77a361b` (working tree clean except untracked/CONTINUITY.md; `restore.rs` 1599 L as cited), md-codec 0.35.3 (sibling repo `descriptor-mnemonic`). Live probes run against a fresh `target/debug/mnemonic` build of `77a361b` (export-wallet bip388/green on NUMS-Single tr shapes); no files created or modified.

## Verdict: GREEN (0 Critical / 0 Important / 4 Minor)

All six round-1 findings are folded faithfully, and the make-or-break I1 fold design survives adversarial source-level scrutiny plus live probes. The four Minors are implementation-time precision nits inside already-mandated edit zones — none blocks implementation start.

## Fold-verification table (I1 + M1–M5)

| Finding | Folded correctly? | Note |
|---|---|---|
| **I1** (--format matrix) | **YES** | §2.7: decision = restore-side green refusal (the round-1-preferred option), `script_type == P2tr` condition, BadInput exit 1, bip388 pinned as-is, matrix rest documented; + 3 test cells (§3:38-40) + manual ¶ (§4) + M5 doc-comment rider. Precision claim independently re-verified below. Residual: manual wording under-specified → new m1. |
| **M1** (exit codes) | **YES** | §2.1 pins the NEW depth≥2/sortedmulti_a refusals as `ModeViolation` exit **2** (matching restore.rs:686/:707 — verified `error.rs:541` ModeViolation→2, `:502` BadInput→1) and correctly notes the §5 `from_str` backstop is `bad()` exit 1. Cells §3:34-36 assert `exit 2` explicitly. |
| **M2** (golden asymmetry) | **YES** | §3 golden note: derive-once-then-pin, depth-0 reconstructed xpubs (`xpub_from_65_bytes` depth:0, restore.rs:805) ≠ input account xpubs, `<0;1>/*` promotion, H-point hex vs literal `NUMS`, never hand-substitute (v0.49.1 I2 trap). Complete. |
| **M3** (≠2-children refusal) | **YES** | §2.3 mandates refusing a `Tag::TapTree` with body ≠ 2 children, explicitly diverging from the `#[cfg(test)]` `count_tap_leaves` silent-leaf pattern (parse_descriptor.rs:2621-2631, verified). Signature nit → new m2. |
| **M4** (multi_a-in-2leaf cell) | **YES** | §3:37 adds the cell with the correct routing claim (inner.tag == `TapTree`, not `MultiA` → GeneralFaithful — verified against restore.rs:703-705) and the P2trMulti substring flip (wallet_export/mod.rs:237-241). Ambiguity on the "format matrix side" claim → new m3. |
| **M5** (:731-738 doc-comment) | **YES** | §2.7 bullet 3. Verified the comment at restore.rs:731-738 does claim green "refuse[s] via … `is_multisig` branches" — wrong for the P2tr general-tr arm, so the update is genuinely required. |

## I1 fold design — adversarial verification (the make-or-break)

**The "precise condition" claim HOLDS.** Enumerated every path into `build_multisig_import_payload`'s general arm (restore.rs:721-752, `template == None` at :744-751):
- `script_type_from_descriptor` (wallet_export/mod.rs:210-247) returns `P2tr` ONLY from the `Tr(t)` arm without a `multi_a(`/`sortedmulti_a(` substring (:229-242); every non-tr general descriptor (wsh / sh-wsh) maps to `P2wshMulti`/`P2shP2wshMulti`/`P2shMulti` — never `P2tr`.
- In restore, a tr md1 reaches `template_opt = None` only via the new GeneralFaithful classification: keypath-only (`tree: None`) is refused at restore.rs:692-696, `is_nums: false` (including any keypath-only-with-keyed-IK singlesig card) at :685-691, and single-leaf `multi_a`/`sortedmulti_a` takes the Template arm (:703-705). So in the restore general arm, `P2tr` ⟺ a general tr policy descriptor — exactly as the plan claims. No legitimate green emission is lost (a tr policy descriptor is never a singlesig surface, and `restore --md1` cannot reach payload-build for genuine singlesig cards at all — the wallet-policy gate at :1030-1044 precedes everything).
- The M4 shape (`multi_a` under a 2-leaf TapTree) classifies `P2trMulti` → caught by the EXISTING green.rs:36 `is_multisig()` refusal — **live-probed**: exit ≠0, "--format green does not support multisig …". So the new `== P2tr` condition exactly plugs the gap with zero overlap.
- Reachability/matchability confirmed: `WalletScriptType` is `pub(crate)` with `derive(PartialEq, Eq)` (wallet_export/mod.rs:162-173); restore.rs already imports `crate::wallet_export` (:38) and binds `script_type` at :748; `CliExportFormat` is the `format` param (:722, import :31).

**Exit-code choice (BadInput 1) is RIGHT.** The refusal is a format-capability refusal, not a card-shape refusal: green's own family is `BadInput` (green.rs:37-40), pinned at exit 1 by `cli_restore_multisig_format.rs:193-204` (`.code(1)`). ModeViolation 2 is restore's "this card shape isn't restorable" family (restore.rs:686/:707) — the card here restores fine; only the format can't carry it. Exit-1 keeps "every `--format green` refusal is exit 1" uniform across template / wsh-general / tr-general arms.

**bip388 pin-as-is confirmed loud, both branches.** Live probe of `export-wallet --descriptor 'tr(<H-hex>,{pk,pk})' --format bip388` (identical EmitInputs shape): exit 1, message = pipeline.rs:228 ("requires every descriptor key to end in `` `/<0;1>/*` `` … got key \"50929b74…803ac0\""). The wildcard-only variant would hit pipeline.rs:177-180 instead — and BOTH messages contain the `/<0;1>/*` wording the §3:39 cell asserts, so the test pin is robust to which branch fires. Not silent-wrong; pin-as-is is sound. The green probe on the same shape emitted exit 0 with a "singlesig" header — re-confirming round-1's bug and that the plan's restore-side fix targets a real, still-live emission.

## Rest of Rev 2 re-verified against source

- All restore.rs citations live: classifier :675, refusals :685-691/:692-696/:707-712, translator reject :856-863, `faithful_multisig_descriptor` :911, §3 call site :1081-1088, §5 `from_str` :1152-1154, payload doc-comment :731-738. md-codec `TapTree` decode is strictly 2-child (descriptor-mnemonic tree.rs:239-243) — the ≤2-leaf ⟺ no-nested-TapTree equivalence in §2.3 holds.
- **Equality-leg safety at §5 (new check the fold made necessary):** if the Display-fidelity leg lands at the §5 site it also covers the taproot TEMPLATE arm — verified NO false-refusal risk: `build_descriptor_string` already returns `parsed.to_string()` (pipeline.rs:27-31), so its output is Display-stable by construction; a mismatch there would itself be a true parse/Display infidelity worth refusing. Safe.
- T1 cells match the plan's FLIP/KEEP list exactly (cli_restore_taproot_refusal.rs:77/:98/:120; shapes `and_v(v:pk,after)` / `{pk,pk}` / cosigner-IK, all `.code(2)`).
- `extract_multisig_threshold` (bundle.rs:1046-1057) walks `Body::Tr{tree:Some}` → for M4 returns `Some(2)` — the k_opt interaction the M4 cell pins is real, pre-existing semantics shared with the shipped wsh-general arm.
- 6 lockstep sites live at 0.55.0: Cargo.toml:3, Cargo.lock:727, README.md:13, crates/…/README.md:9, install.sh:32, CHANGELOG `[0.55.0]` @ :9. Manual sites live: :751 (Two-modes ¶), :776 (`--md1` row; plan says ~:777 — off-by-one under a `~`, acceptable), :978-984 (--format ¶ incl. the green sentence at :980), :986-990 (Scope ¶).
- Matrix-rest claims spot-checked: specter `MissingField::WalletName` (specter.rs:35), bsms `BsmsTaprootRefused` on `P2tr|P2trMulti` (bsms.rs:79-81).

## Critical
None.

## Important
None.

## Minor
- **m1 (NEW, fold-introduced) — the manual green sentence needs more surgery than "stays TRUE".** 41-mnemonic.md:979-981 reads "`green` is refused (no multisig support) — both identically to `export-wallet`". After the restore-side-only fix: (a) the "(no multisig support)" reason is WRONG for the general-tr arm (P2tr is precisely NOT multisig — the refusal reason is Green's singlesig-only file surface vs a policy descriptor); (b) "identically to `export-wallet`" becomes FALSE (probe: `export-wallet --descriptor tr(H,{pk,pk}) --format green` still emits exit 0 and will continue to). §4's instruction ("the sentence stays TRUE … via the new explicit refusal") under-specifies the edit and could leave both inaccuracies standing. Required: reword the parenthetical + qualify/drop the export-wallet-parity clause for the general-tr arm. Same ¶'s "Supported:" list (:978) also over-promises for general-policy arms (pre-existing since v0.54.0 for wsh-general; this cycle widens the audience) — a one-clause scoping note is cheap while editing the ¶.
- **m2 — §2.3 helper signature vs M3 semantics.** `fn taptree_depth_le_one(&Node) -> bool` cannot itself distinguish "depth≥2" from "TapTree with ≠2 children", so a bool-driven caller would refuse the malformed case with the depth-2 slug message. Since md-codec decode makes ≠2 children wire-impossible (tree.rs:239-243), this is defensive-only — either return a 3-way/`Result`, or keep bool + a comment stating decode guarantees arity 2 and the conflation is deliberate.
- **m3 — M4 cell's "pins the P2trMulti side of the format matrix" claim is not realized by the cell as written** (§3:37 runs only the bare restore, exit 0 + goldens). Pinning that side needs an explicit `--format green` leg on the M4 shape asserting the EXISTING refusal (exit 1, "does not support multisig" — probe-confirmed, a DIFFERENT message from the new singlesig-only wording). Add the leg or drop the claim.
- **m4 — §2.6 placement wording "in `faithful_multisig_descriptor`/§5" is ambiguous**: `faithful_multisig_descriptor` contains no `from_str`; the leg belongs at the §5 site (after restore.rs:1153), the single site that covers both general arms AND (safely, per the pipeline.rs:27-31 argument above) the template-tr arm. Pin the site in the plan or the implementing commit. Relatedly, make the §3 cell-1 flip explicit that the bundle-side wire-faithfulness assert (`emitted == desc`, literal-NUMS leg at cli_restore_taproot_refusal.rs:81-84) is KEPT, not deleted with the refusal half.

## Path forward
GREEN — the R0 gate is satisfied; implementation may begin. Fold the 4 Minors at implementation time (m1 wording in the §4 manual edit, m2-m4 as implementer notes), persist this review to `design/agent-reports/general-tr-faithful-restore-plan-r0-round2-review.md` before the fold-and-commit step.
