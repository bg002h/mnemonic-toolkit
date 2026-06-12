# R0 Review — general-tr faithful restore (PLAN) — Round 1
Reviewer: Fable 5, 2026-06-12. Verified against mnemonic-toolkit origin/master `77a361b` (restore.rs byte-identical to recon `1971ffa`). md-codec 0.35.3, miniscript `95fdd1c`. Live probes (temp test, deleted).

## Verdict: RED (0C/1I/5m)

Core design SOUND + probe-confirmed: the 3-way classifier restructure, strict-NUMS `Single` pass-through, conservative structural ≤2-leaf gate, and zero-new-machinery fall-through all correct. One Important: a `--format` surface gap the plan punted.

## Critical: none.

## Important
**I1 — the `--format` matrix for the new GeneralFaithful-tr arm is mis-specified.** Probe (`export-wallet --descriptor 'tr(<NUMS-hex>,{pk,pk})'` = identical `EmitInputs{template:None}` shape): `script_type_from_descriptor` (`wallet_export/mod.rs:229-242`) classifies a general tr WITHOUT a `multi_a(`/`sortedmulti_a(` substring as `P2tr` (taproot SINGLESIG) — BOTH this cycle's flagship shapes (`and_v(v:pk,after)` single-leaf; `{pk,pk}` 2-leaf). Consequences: `--format green` gates on `script_type.is_multisig()` (P2tr is NOT) → **emits exit 0** a "singlesig" Green payload, CONTRADICTING `restore.rs:731-738` doc-comment + the manual (`41-mnemonic.md ~:980` "green is refused"). The wsh-general arm never hit this (`P2wshMulti.is_multisig()==true`). `--format bip388`: `iter_pk` visits the NUMS `Single` (no `/<0;1>/*` suffix) → refuses with "requires every descriptor key to end in `/<0;1>/*` … got key 50929b74…" — loud (not silent-wrong) but message blames a "descriptor key" while naming the NUMS internal key, and diverges from wsh-general (emits) + tr-template (emits). NOT funds-dangerous (both watch-only; descriptor inside is faithful) → Important not Critical. **Fold:** (1) DECIDE green for this arm — accept export-wallet parity (fix the manual + `:731-738` doc-comment) OR extend the refusal so a policy-tr restore doesn't emit a "singlesig" payload (preferred — matches the manual's existing promise); (2) add `--format green` + `--format bip388` test cells (plan §3 pins only descriptor/bitcoin-core); (3) manual restore `--format` ¶. REST OF MATRIX VERIFIED SAFE: electrum/sparrow/jade/coldcard refuse on `template:None`; specter → MissingField::WalletName; bsms → BsmsTaprootRefused (P2tr|P2trMulti); descriptor/bitcoin-core emit faithfully.

## Minor
- **M1** exit-code: the §5 `from_str` backstop = `bad()` = `BadInput` = exit **1** (not 2). Refusal arms (ModeViolation) = exit 2. Spec the NEW depth≥2/sortedmulti_a refusals as `ModeViolation` exit 2 (matching `:689`/`:710`); plan's "exit ≠0" is too loose for the cells.
- **M2** golden asymmetry covers the XPUBS too: reconstructed descriptor carries md-codec's depth-0 reconstructed xpubs (`xpub661My…`) ≠ bundle-input account xpubs, + `<0;1>/*` promotion. Derive-once-then-pin; do NOT hand-construct by hex-substituting the input (v0.49.1 I2 trap).
- **M3** depth-gate predicate: REFUSE on a `Tag::TapTree` whose body ≠ exactly 2 children (don't silently treat malformed as a leaf, unlike the `#[cfg(test)] count_tap_leaves` pattern). Spine-only walk acceptable (a TapTree under a non-TapTree leaf → md-codec errors → `bad()` wrap).
- **M4** ADD cell `tr(NUMS,{multi_a(2,K0,K1),pk(K2)})` — pins multi_a-bearing TapTree → GeneralFaithful (not Template), the k_opt/threshold_field/label interaction, + the P2trMulti side of I1.
- **M5** `restore.rs:731-738` doc-comment update rides I1.

## Funds-safety analysis (the crux)
(a) Left-heavy depth-2 if routed: Displays `{{a,b,c}}` (1-child outer) → reparse FAILS `IncorrectNumberOfChildren`; CAUGHT downstream by §5 `from_str` (`:1152`) + `build_multisig_import_payload` re-parse (`:745`) → abort exit 1, zero stdout. No malformed descriptor printed. (b) Right-spine depth-2: Displays FINE → would silently reconstruct → Display-luck accepted-set → the CONSERVATIVE structural gate (refuse ALL depth≥2 on the Node tree, never on Display) is the right funds-safe call (explainable, chirality-independent, dissolves on #953). (c) WRONG-but-PARSEABLE depth≥2 reconstruction — searched hard, NONE: the bug is brace-flattening (a TapTree's 1 sibling slot → ≥2 children → some brace gets ≥3 or outer gets 1 → structurally unparseable in a 2-children-per-branch grammar); cannot produce valid-but-different. Residual wrong-reconstruction vectors: (i) future parseable Display infidelity → the Q4 equality leg guards it (ADD); (ii) key mis-translation → `translate_pk` is positional 1:1, strict-NUMS can't substitute; (iii) chain/multipath drift → shipped since v0.54.0, anchored by golden bc1p. Golden addresses are the anti-wrong-reconstruction anchor; with M2+M4 the test set is complete.

## Design-Q answers
1. **strict-NUMS** — YES. `build_nums_internal_key` (to_miniscript.rs:183-190) is the ONLY `Single` producer; every policy key (incl. is_nums:false IK) → XPub. No legitimate card surfaces a non-NUMS Single → strict never false-refuses. Compare x-only `==` H-point (NOT string-contains — Debug prints the internal 64-byte repr). Pass through unchanged (no multipath/network).
2. **conservative ≤2-leaf** — YES (structural; "no TapTree child of a TapTree" ⟺ ≤2 leaves since md-codec TapTree is strictly binary).
3. **sortedmulti_a pre-gate** — YES (converter message is engineer-speak; single-leaf sortedmulti_a still routes Template first).
4. **Display-fidelity equality leg** — ADD (`parsed.to_string() == descriptor`, internal-error refusal — the only guard against parseable-wrong).
5. **--format** — NOT fully sane → I1.
6. **template_opt=None routing** — YES, clean (traced §3→§8: §4 slots from leaf keys, NUMS never in key list; §5 string path already selected by `is_taproot||template_opt.is_none()`; §6 key-based; §7 labels on template_opt; `extract_multisig_threshold` walks Body::Tr benignly). The `Single` pass-through is the ONLY new machinery beyond classifier + gates.

## Lockstep (verified live @ 0.55.0)
6 version sites: `crates/mnemonic-toolkit/Cargo.toml:3`, `Cargo.lock:727`, `README.md:13`, `crates/mnemonic-toolkit/README.md:9`, `scripts/install.sh:32`, CHANGELOG `[0.55.1]`. Manual prose: `41-mnemonic.md ~:750-753`, `~:777` (--md1 row), `:986-990` (Scope ¶), + the I1 green/bip388 sentence ~:980-984. Full manual lint before push. SemVer PATCH confirmed (v0.49.1/v0.54.1 precedent); no schema_mirror, no md-codec change/publish/pin.

**Path to GREEN:** fold I1 + M1-M5, re-dispatch Round 2.
