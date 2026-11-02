# Bug-hunt grading pass G1, mnemonic-toolkit (slices S1 + S2)

Grader: Opus, xhigh. Inputs: `bughunt-2026-10-31-tk.md`, `bughunt-triage-tk-s1.md`, `bughunt-triage-tk-s2.md`, RED rows `tests/bughunt_s1.rs`, `tests/bughunt_s2.rs`, `crates/wc-codec/tests/bughunt_s2.rs`, scratch fixes `/scratch/cache/bughunt/tk/{s1,s2}/fix-NN.patch`, worktree `bughunt-triage` at `a9f781b2` (production source identical to the triage base `70894e51`).

**Counts: 3 C, 23 I, 2 M, 0 N (28 graded). One new item found while checking fix-11 (11b, provisional C, needs RED rows). One disagreement with the triage: #40 is I, not M.**

## What this pass machine-checked (new evidence, not from the triage)

- **All 28 patches together.** All 28 `fix-NN.patch` apply cleanly in sequence on `a9f781b2` (fix-12, fix-13 and fix-43 included). With them plus a scratch 11b gate (scratch worktree `/scratch/code/shibboleth/tk-worktrees/grade-g1-scratch`), whole-workspace `cargo nextest run --locked --workspace --no-fail-fast` (cargo 1.85.0): **4148 run, 4111 passed, 37 failed, 20 skipped** (log `/scratch/cache/bughunt/tk/g1-allfix-suite.log`, list `g1-allfix-fails.txt`). Failures: 34 are other slices' unfixed rows (S3-S5, #36); 1 is `parse_descriptor::tests::lex_residue_floor_accepts_multipath_sans_wildcard` (expected, #3); **2 are caused by fix-37**: `cli_verify_bundle_md1_template::verify_template_bundle_expect_wallet_id_skipped_under_origin` and `::verify_template_bundle_recomposes_under_origin_matches_restore` (under `--origin` the correct ms1 is reported `✗ ms1_entropy_match`, exit 4). All 28 S1/S2 rows pass. The S1 fixes had previously been run only against their own rows.
- **#40 is not fail-closed** (`/scratch/cache/bughunt/tk/g1-40.sh`, unmodified binary). Fixture: BIP-48 2-of-2 sortedmulti template, A at account 1, B at account 0. Running `restore --md1 <template> --from phrase=<A> --account 1 --cosigner @1=<B bare xpub>` with no reference: exit 0, own key printed as `[b8688df1/48'/0'/0'/2']`, `first recv bc1q2a4vm7ww7v0c02qerrgg0znr4ck4kez2la82kzpm9gturx68q4nsfdl000`. The true wallet is `bc1qutaes7u4vv3fn9wtmdfpx3wvset9exs59un9r00aw7lyryxd3ymqsf73hu`. With fix-40 (`g1-40-fixed.sh`, scratch binary): own key `[b8688df1/48'/0'/1'/2']`, first recv equals the true wallet.
- **11b, a single-sig template use-site that restore silently replaces** (unmodified binary). `bundle --descriptor "wpkh([73c5da0a/84'/0'/0']@0/<2;3>/*)" --md1-form template --slot @0.phrase=<abandon x11 about>` exits 0. It mints `md1fkykhqqpqqgzskqqzxcjnwq6cq0gv` (`inspect`: `template: wpkh(@0/<2;3>/*)`) and the same wallet-id as the standard wallet (`1c0170fe…`), with no advisory. Then `restore --md1 md1fkykhqqpqqgzskqqzxcjnwq6cq0gv --from phrase=… --account 0 --expect-wallet-id 1c0170fe` gives exit 0, `✓ wallet-id verified`, descriptor `…/<0;1>/*)#hpg6d6w2`, `first recv: bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`. The standard wallet's address is printed, not the card's wallet. Bare `wpkh(@0)` and `wpkh(@0/*)` both mint `md1fu85lqqpqqqqqnm3nr4htwmp8w` (`template: wpkh(@0/*)`), and restore turns it into the same `/<0;1>/*` wallet at exit 0. With the scratch gate, bundle exits 2 ("cannot template this descriptor shape") for `<2;3>`, `@0` and `@0/*`; restore exits 2 with no descriptor for both cards; the `/<0;1>/*` controls still exit 0.
- **The taproot key-order note does not reproduce.** A bundle-minted `tr(NUMS,multi_a(2,@1/<0;1>/*,@0/<0;1>/*))` restores to `bc1pggkuc3l2kn8e9mkjgru7vtztr78azlz4ah2n2eq9c90xmdt556xsxmgp4w`. That equals the in-order (X1,X0) reference; (X0,X1) gives `bc1pwzwa88…` (`g1-order.sh`).
- **#4 Electrum table.** `CJK_INTERVALS` was fetched from `raw.githubusercontent.com/spesmilo/electrum/e1099925/electrum/mnemonic.py`. It is byte-identical to the triage's copy and to fix-04's 29 intervals (`normalize_text` line 90 confirmed).
- **#5 advice.** `export-wallet --template bip86 --format sparrow` and `--format bitcoin-core` both exit 0 with a P2TR payload, so fix-05's remedy runs as printed.
- **#46 advice.** `mnemonic convert --from ms1=<ms1> --to phrase` and `--from entropy=<hex> --to phrase` both print the phrase.
- **No in-tree consumers.** No consumer in the constellation reads `multisig.path_family` (#54) or the verify `md1_origin_match` row (#14). wc-codec `u_slots` (#8) is set only by wc-codec's own tests and by `Default` (U = 3).

## Grades

| # | final grade | triage proposal | agree? | row tests the defect? | likelihood conditions / why |
|---|---|---|---|---|---|
| 1 | **C** | C | yes | yes (exit 4 + "xpub mismatch") | An explicit `--expect-xpub` is silently skipped when `--expect-fingerprint` matches, and the run reports `verified` at exit 0. A verification flag fails its guarantee; "right seed, wrong account" is a plausible case. |
| 2 | **I** | I | yes | yes (4 sub-cases, honour-or-refuse) | Four flags are silently ignored and a wrong reference is accepted at exit 0. Not fail-closed. Not C: the UNVERIFIED banner still points the holder at the fingerprint. |
| 10 | **I** | I | yes | yes, for `emit_completed_multisig`. The listing site, the other half of the same report item, needs row 10b | `--output FILE` is silently ignored at exit 0. The document itself is correct on stdout. |
| 12 | **I** | I (strict; M offered) | yes, keep I | yes | Condition (1) fails: correct cards are reported as bad, which is wrong per-slot data. Condition (3) fails: only one unusual condition (a foreign card among those supplied); the chunk_set_id order that triggers it is a state, not a holder choice. |
| 13 | **I** | I (strict; M offered) | yes, keep I | yes | Condition (1) fails: correct cards get false `mk1_*_match` failures. Condition (3) fails: one unusual condition (md1 undecodable); supplying cards in slot order is typical and trips the bug in 5 of 6 orders. |
| 14 | **M** | M | yes | yes (row presence + equal length) | (1) Fails closed: exit 4, `md1_decode` names the problem, and a missing row is not wrong data. (2) No printed command. (3) Two conditions: the md1 does not decode (damaged or incomplete set), AND a `--json` consumer reads the list by position or length (none in the constellation). |
| 37 | **C** | C | yes | yes (control + other-seed ms1 → 4) | A seed plate for the wrong seed verifies OK: a false PASS on the most critical card. **fix-37 is itself wrong**: it breaks two existing `--origin` tests (see Fixes). |
| 38 | **C** | C | yes | yes (std control + 2 non-std) | A card minted by the toolkit restores to a different taproot wallet (plausible bc1p addresses) at exit 0 with no warning. This breaks the bundle → restore round trip. |
| 39 | **I** | I | yes | yes | False PASS for an md1 of a different wallet. It needs an md1 from another descriptor over the same xpub, which the `--template` flow cannot mint. A wrong result, so not M. |
| 40 | **I** | M | **no** | yes for the search path (fails closed there). The explicit-mode path needs a row extension | **Not fail-closed.** In explicit `@N=` mode with a bare-xpub cosigner and `--account 1`, the wrong wallet is printed at exit 0 (probe above). A wrong result stays C/I. I, not C: two unusual conditions (bare-xpub cosigner, own account ≠ 0) and an explicit-mode "WITHOUT verifying" warning. fix-40 fixes both paths (shared `own_origin_for`). |
| 41 | **I** | I | yes | yes | Wrong wallet at exit 0 with the supplied address ignored. Not C: explicit mode always prints its "WITHOUT verifying" warning. |
| 42 | **I** | I | yes | yes | A false "(verified)" label. The printed descriptor carries the wrong fingerprint, so signers will not recognise the key; addresses are correct. |
| 43 | **I** | I | yes | yes | Panic (exit 101) instead of a verdict. It does not name the problem, so (1) fails. |
| 44 | **I** | I | yes | yes | A correct bundle fails its own round trip when the fingerprint is given the documented way (descriptor annotation). This is typical usage; the result is a false mismatch. |
| 47 | **I** | I (strict; practical M offered) | yes, keep I | yes (2nd half pins restore) | Not fail-closed: an empty prefix yields `✓ wallet_id_match` at exit 0. The 0-byte advisory does not turn a vacuous pass into a refusal. |
| 53 | **I** | I | yes | yes (card built through md_codec's public API) | Wrong leaf at exit 0. Only a hand-built card reaches it (`bundle` and `md encode` refuse), so I, not C. |
| 3 | **I** | I | yes | yes (placeholder + concrete; bare `@N` rightly excluded) | md1 encodes a ranged wallet for a non-ranged descriptor at exit 0, silently. Rare input and the seed is unaffected, so I, not C. |
| 4 | **I** | I | yes | yes | Wrong Electrum seed for a passphrase with a space between Hangul (or other missing-range) characters, at exit 0. The table was checked against upstream. |
| 5 | **I** | I | yes | yes | Every bip86 Electrum export imports as p2pkh at exit 0. Not C: the addresses visibly differ (`1…` vs `bc1p…`) and the same keys still spend them. |
| 6 | **I** | I | yes | yes (entropy + ms1) | The annotation cross-check (the wrong-seed guard) is silently skipped for entropy and ms1 slots. |
| 7 | **M** | M | yes | yes (contract of public `sync_classify`) | (1) Fails closed end to end (measured: CLI exit 2 "uncorrectable"; RS plus the 44-bit tag). (2) None. (3) Three conditions: a damaged word in block 0, an inserted word, and a data word whose top six bits look like a checkpoint at block 1's first data slot (about 1/64). |
| 8 | **I** | I (by rule) | yes | yes | `encode` returns an unreadable card for `u_slots ≥ 8`, and an unreadable backup stays C/I. No shipped caller reaches it today; the guard is one line. |
| 9 | **I** | I | yes | yes | An unparseable `--slot @N.fingerprint=` silently becomes `00000000` (or the annotation value) at exit 0. |
| 11 | **I** | I | yes | yes (+ message) | A hardened single-sig template is accepted and the printed recipe restores a different wallet. The bundle-time advisory partly mitigates. The fix is widened by 11b. |
| 11b (new) | **C** (provisional; untriaged) | n/a | n/a | needs rows 11b-bundle and 11b-restore (below) | The same gate admits any single-sig use-site; restore hard-codes `/<0;1>/*`, and the single-sig wallet-id does not bind the use-site. Result: a toolkit-minted card restores a different wallet under `✓ wallet-id verified` at exit 0, with no advisory. This violates `SPEC_bundle_md1_template_only_2026-06-19.md` line 82 ("custom non-standard path → clear refusal at bundle-emit") and the manual ("non-canonical single-sig are refused"). C on the same footing as #38. |
| 45 | **I** | I | yes | yes (accepts refusal) | Wrong network label on the bundle and engraving card at exit 0; the required `--network` flag is silently contradicted. |
| 46 | **I** | I | yes | yes (entropy + ms1) | A seed overlay is silently dropped and a watch-only bundle is produced at exit 0. |
| 52 | **I** | I | yes | yes (control first) | The check passes over a card it never read (silently ignored input) at exit 0. |
| 54 | **I** | I | yes | yes (asserts only ≠ `bip87`; tighten) | Wrong value in a JSON field kept for recovery tooling. SPEC v0.3 §5.3 says `"descriptor"`. `--multisig-path-family` is refused in descriptor mode (`bundle.rs:311`), so the field is always `bip87` today. |

## False positives and dropped sub-cases

- **False positives:** none in S1 or S2.
- **#3, bare `wpkh(@0)`: drop accepted.** Probe: bare `@0` encodes `wpkh(@0/*)`, an md1 identical to `wpkh(@0/*)` and a representable use-site. The hunt report's claim that it encodes as `/<0;1>/*` is wrong. Its template-form restore is wrong, which is covered by 11b.
- **#7, end-to-end fail-closed narrowing: accepted** (measured by the triage).
- **#10, `emit_candidate_list` ignores `--output`: accepted into #10.** It is half of the same report item (`restore.rs:1589-1640` writes to stdout and never reads `args.output`). Row 10b is required.
- **#40, `--account 0,1` deriving one key twice: covered by fix-40** (same closure); no separate row. The explicit-mode wrong wallet gets a row extension.
- **#44, H7 annotation-agreement check skipped on verify: accepted into #44.** Also accepted: the verify-side twin of #9 (`verify_bundle.rs:1758` `.ok()` turns a bad `--slot @N.fingerprint=` into `00000000`). Without that fix, fix-44 alone would make a bad value fall back silently to the annotation, a new silent ignore. Rows 44b and 44c are required.
- **#2, misleading UNVERIFIED banner: ruled under #2.**
- **#47, missing refusal of prefixes over 16 bytes: accepted into #47** (shared helper).
- **#52, non-`mk1` garbage stopped by the HRP guard: narrowing accepted.**
- **#4, `0x3000-303F` absent from Electrum: the triage's correction is accepted** (checked against upstream).
- **Taproot Template-arm key order (S1 note): rejected as a sub-case.** It does not reproduce with a bundle-minted card (probe above). A hand-built card with non-identity indices is untested and not graded.

## Product rulings

Errors print as `error: <message>`. Exit codes follow the variants: `BadInput` 1, `ModeViolation`/`DescriptorParse`/`TemplateFormUnsupportedShape` 2, `RestoreMismatch` 4.

- **#2 (single-sig template restore).**
  - **Honour** `--expect-fingerprint`, `--expect-xpub` and `--allow-mismatch` exactly as the keyed single-sig path does (`restore.rs:609-650`, 779-793).
  - **Refuse** `--format`: `ModeViolation{mode:"restore", flag:"--format"}`, exit 2, text: `--format is not supported when completing a keyless single-sig template md1; run without --format to print the completed descriptor, or (canonical origin only) restore with --template <bip44|bip84|bip86> --account <N> --format <FORMAT> and no --md1`
  - **UNVERIFIED banner:** printed only when none of `--expect-wallet-id`, `--expect-fingerprint`, `--expect-xpub` was supplied; text unchanged.
- **#41.** Refuse the combination: `BadInput`, exit 1, text: `explicit @N= assignment cannot combine with --search-address: @N= asserts the key→slot placement, so the address would not be checked. Drop the @N= prefixes to let --search-address place the cosigners, or check the placement with --expect-wallet-id.`
- **#38.** Restore faithfully (fix-38), mirroring wsh. Where the faithful arm cannot render (e.g. `tr(sortedmulti_a)`), a refusal is acceptable. A different wallet never is.
- **#42.** Refuse: `RestoreMismatch{reference:"cosigner-fingerprint", derived:"seed master fp <fp>", expected:"md1 cosigner @<i> engraved fingerprint <fp>", slot:Some(i)}`, overridable with `--allow-mismatch`.
  - Stderr: `✗ MISMATCH`, then `error: restore: cosigner-fingerprint mismatch at slot @0 — derived seed master fp b8688df1, expected md1 cosigner @0 engraved fingerprint deadbeef`, exit 4.
  - With `--allow-mismatch`: status `overridden`, and the own slot is labelled `from md1 (not independently verified)`.
- **#47.** Yes. verify-bundle uses restore's `decode_wallet_id_prefix` (`restore.rs:3094`, made `pub(crate)`). Its texts:
  - `--expect-wallet-id must not be empty`
  - `--expect-wallet-id must be an even-length hex prefix of the WalletPolicyId: <e>`
  - `--expect-wallet-id prefix is <n> bytes; the WalletPolicyId is only 16 bytes — supply a prefix of at most 16 bytes (32 hex chars)`

  All exit 1. The `advisory: … ≥4 recommended.` line stays.
- **#14 (deferred).** The placeholder row mirrors its siblings: `passed: true`, detail and decode_error `skipped: md1 decode failed` (fix-14).
- **#3.** The D1 deferral stands: bare `@N` stays accepted. Only an explicit `/<a;b>` without `/*` is refused: `DescriptorParse`, exit 2, text: `@<i>: a multipath use-site `/<a;b>` with no trailing `/*` (a non-ranged multipath) is not representable in md1, which encodes only a ranged `/<a;b>/*` or bare `/*` use-site`. The text deliberately gives no "add `/*`" remedy, since adding it would describe a different wallet.
- **#5.** Refuse (fix-05 verbatim): `BadInput`, exit 1, text: `--format electrum does not support --template bip86 (P2TR): a plain xpub "standard" wallet is read by Electrum as p2pkh. Use --format bitcoin-core (descriptor) or --format sparrow for taproot watch-only setup.`
- **#6.** Confirmed: the texts mirror the Phrase arm (fix-06 verbatim).
- **#7 (deferred).** Use the "rich" realignment, not refuse-only. fix-07 is rejected: it refuses on any later-checkpoint failure, including an insertion plus independent damage in a later block (see Deferred).
- **#8.** Reject `u_slots > 7` with `InvalidParams` (fix-08). No wire change.
- **#45.** Refuse, rather than letting the envelope win silently (the flag is required, so the holder typed it): `BadInput`, exit 1, text: `--network <flag> disagrees with the --import-json envelope's bundle.network (<env>); re-run with --network <env>` (names via `CliNetwork::human_name()`). fix-45 is not used.
- **#46.** Refuse every non-`phrase` overlay: `BadInput`, exit 1.
  - For `entropy`/`ms1`/`seedqr`: `--slot @<N>.<subkey>=: not supported with --import-json; only --slot @N.phrase= can add a seed to a watch-only cosigner (get the phrase with: mnemonic convert --from <subkey>=<value> --to phrase)`
  - For any other subkey: `--slot @<N>.<subkey>=: not supported with --import-json; the envelope already supplies every cosigner's key, fingerprint and path (only --slot @N.phrase= is accepted)`
- **#52.** The check fails (exit 4) and the full table is kept; no intake refusal. When any `--mk1` value has no readable chunk_set_id, the row is `mk1_template_stub_bind` `passed:false`, detail: `supplied mk1 stub(s) do NOT bind via the template-id stub: an --mk1 value has no readable chunk_set_id (damaged card or not a template stub)`
- **#54.** `"descriptor"`, per SPEC v0.3 §5.3, whenever `multisig.template` is `"descriptor"` (descriptor mode, including `--import-json`). fix-54's bip48 derivation is not used.
- **#11 / 11b.** The single-sig template form exists only at the standard `/<0;1>/*` use-site with no overrides. `bundle` refuses anything else; `restore` refuses existing cards with another use-site. Texts are under Fixes.

## Fixes for the PR (C and I only)

All fixes are in `crates/mnemonic-toolkit` unless noted. "verbatim" means apply the named patch unchanged. In every row whose correct behaviour the triage left open (#2, #38, #41, #42, #45, #46, #52, #54, #3, #5, #47, #8), tighten it to the ruled exit code and text.

1. **#1 (C).** fix-01 verbatim. Existing texts: `✗ MISMATCH`, `restore: xpub mismatch — derived …, expected …`, exit 4. Tests change: none. Manual: none.
2. **#37 (C).** fix-37 is **not** used: it compares against `expected.ms1`, which is empty under `--origin` (it breaks the two `--origin` tests above).
   - When `args.ms1` has a non-empty first value, `verify_singlesig_template` pushes the `ms1_decode` and `ms1_entropy_match` rows built by the keyed single-sig logic (`verify_bundle.rs:2348-2452`: decode, byte compare, auto-repair ground-truth compare unless `--no-auto-repair`), extracted into one shared helper.
   - The expected ms1 is the card of the **canonical seed slot**: synthesize from `resolved` (the `resolve_slots` result, which carries the entropy), never from `resolved_for_synth`.
   - Rows go first in the list, before `md1_template_match`. With no `--ms1` there are no ms1 rows (unchanged).
   - Texts are the existing row texts (e.g. failure `expected ms1 bytes differ from supplied`; auto-repair mismatch `auto-repair candidate did not match the expected seed — this card is not a card for this seed`). Exit 4 on failure.
   - Tests that must stay green: `cli_verify_bundle_md1_template::verify_template_bundle_expect_wallet_id_skipped_under_origin` and `::verify_template_bundle_recomposes_under_origin_matches_restore`. They are the `--origin` non-regression rows.
   - Manual (`41-mnemonic.md` "Template-only md1", after "The same completion works on verify-bundle…"), add: `A supplied --ms1 is compared against the seed (the ms1_decode / ms1_entropy_match rows of the keyed single-sig check); a card for a different seed fails (exit 4).`
3. **#38 (C).** fix-38 verbatim. Extend row `bughunt_tk_38`:
   - multi_a with `/<2;3>/*` and `/*` must give exit 0 **and** the golden address (faithful ruling).
   - Add `tr(NUMS,sortedmulti_a(2,TK0/<2;3>/*,TK1/<2;3>/*,TK2/<2;3>/*))`: it must not exit 0 with a descriptor containing `/<0;1>/*` (refusal or `/<2;3>/*`).
   - Manual: none.
4. **11b (C, provisional) and #11 (I).** One fix replaces fix-11.
   - **Bundle gate:** in `synthesize.rs` `template_admissible`, the `n == 1` branch returns false unless `descriptor.use_site_path == UseSitePath::standard_multipath()` and `descriptor.tlv.use_site_path_overrides.is_none()` (this covers `/*h`). The `TemplateFormUnsupportedShape` message (`synthesize.rs:1049`) becomes: `--md1-form=template cannot template this descriptor shape: it either does not render (e.g. tr(sortedmulti_a) — the rust-miniscript v13 SortedMultiA gap — or sortedmulti inside a combinator), or it uses a hardened use-site path (/*h or a hardened multipath alt) which an xpub cannot derive, or it is single-sig with a use-site other than /<0;1>/* (a single-sig template completes only at /<0;1>/*); use --md1-form=policy for a faithful keyed backup` (exit 2).
   - **Restore guard:** first check in `run_singlesig_template_completion` after `template` is derived: `ModeViolation{mode:"restore", flag:"--md1", message:"this keyless single-sig template md1 has a use-site path other than /<0;1>/*, which single-sig template completion cannot rebuild (it would print a different wallet); use the wallet's full descriptor or a --md1-form=policy card instead"}` (exit 2).
   - **New RED rows** (fail on unmodified code, per the probes above):
     - `bughunt_tk_11b_singlesig_nonstandard_usesite_template_form_refused`: `bundle --network mainnet --descriptor <D> --md1-form template --slot "@0.phrase=<abandon x11 about>" --json` for D ∈ {`wpkh(@0)`, `wpkh(@0/*)`, `wpkh([73c5da0a/84'/0'/0']@0/<2;3>/*)`} → exit 2, stderr contains `cannot template this descriptor shape`. Control `wpkh([73c5da0a/84'/0'/0']@0/<0;1>/*)` → exit 0.
     - `bughunt_tk_11b_restore_refuses_nonstandard_usesite_singlesig_template`: `restore --network mainnet --md1 <M> --from "phrase=<abandon x11 about>" --account 0` for literal M ∈ {`md1fkykhqqpqqgzskqqzxcjnwq6cq0gv`, `md1fu85lqqpqqqqqnm3nr4htwmp8w`} → exit 2, no `descriptor:` on stdout, stderr contains `use-site path other than /<0;1>/*`. Control `md1fvwrwqqpqqgqpsqqq3uaau4ctxyl7` → exit 0. The literal strings are required because the bundle gate stops minting them.
   - Existing row `bughunt_tk_11` passes unchanged. Tests change: none (full-suite run above).
   - Manual ("Template-only md1", last paragraph) becomes: `The single-sig template form covers the canonical shapes bip44 / bip84 / bip86 at the standard /<0;1>/* use-site; bip49 (nested-segwit), non-canonical single-sig, and any other use-site (/*, /<2;3>/*, a hardened /*h) are refused (exit 2) — use --md1-form=policy for those. restore also refuses (exit 2) a single-sig template card with another use-site, which earlier versions could emit.`
5. **#2 (I).** fix-02 with these changes:
   - Move the `--format` refusal to the top of `run_singlesig_template_completion` and use the ruled text.
   - On a reference mismatch without `--allow-mismatch`, print `✗ MISMATCH`, then `master fingerprint: <fp>  (passphrase: applied|none)`, then return `RestoreMismatch` (exit 4, nothing on stdout). Fingerprint is checked first; xpub only if the fingerprint matched or was absent.
   - With `--allow-mismatch`, after the output print the keyed path's text verbatim: `✗ MISMATCH (overridden): derived material does NOT match the supplied reference; descriptors above were produced by the passphrase you provided, NOT the expected wallet` (not fix-02's shortened line).
   - Change the UNVERIFIED banner condition as ruled.
   - Tests change: none.
   - Manual `41-mnemonic.md` restore table:
     - `--expect-xpub` row: "(requires `--template`)" → `(requires --template, or a keyless single-sig template md1, whose type it takes)`.
     - `--format` row, append: `; refused (exit 2) when completing a keyless single-sig template md1`.
     - "Template-only md1", add: `The single-sig reference flags work here too: a wrong --expect-fingerprint or --expect-xpub refuses with ✗ MISMATCH (exit 4) unless --allow-mismatch. --format is refused (exit 2); run without it to print the completed descriptor.`
6. **#10 (I).** fix-10, plus the same routing in `emit_candidate_list`: build the listing (the JSON line, or the text rows plus the truncation line) into one String; write it to `--output FILE` when the value is not `-`, else to stdout. Stderr lines are unchanged; a write error uses the existing `--output <path>: <e>` (exit 1).
   - New row `bughunt_tk_10b_candidate_list_honours_output_file`: a listing fixture whose stderr contains `none reconstructed` (assert this as a precondition; start from `cli_restore_md1_template_multisig.rs::search_cosigner_subset_weak_prefix_warns_or_lists`), plus `--output <tmp>/list.txt`. Assert: exit 0, the file exists and contains `wallet-id`, stdout does not.
   - Manual: none.
7. **#12, #13, #43 (I).** fix-13, fix-12 and fix-43 verbatim, in that order (they apply cleanly together). Texts: none. Manual: none.
8. **#14:** deferred (M).
9. **#39 (I).** In `emit_md1_checks`, single-sig wallet-policy branch:
   - `policy_match = tree == && use_site_path == && tlv.use_site_path_overrides ==` (expected vs supplied).
   - Pass only if `xpub_match && policy_match`, with detail unchanged (`65-byte xpub matches expected`).
   - If `!policy_match`: fail with detail `md1 reconstructs a DIFFERENT wallet — structural policy mismatch: <classes>`, using the same three class strings as the multisig branch (`verify_bundle.rs:3215-3227`) and expected/actual = Debug of the trees.
   - Otherwise the existing `md1 xpub differs from expected`. Name unchanged; exit 4.
   - Manual: none.
10. **#40 (I).** fix-40 verbatim. Extend row `bughunt_tk_40` (or add `bughunt_tk_40b_explicit_placement_bare_xpub_cosigner_keeps_own_account`): same fixture, `--cosigner @1=<B bare xpub>`, `--account 1`, no `--search-address`/`--expect-wallet-id` → exit 0, first address equals `golden[0]`. Unmodified code prints the account-0 wallet (probe). Manual: none.
11. **#41 (I).** fix-41 with the ruled text.
    - Manual, "Multisig template completion" mode 3 becomes: `explicit assignment — pin every cosigner with --cosigner @N= (no search). It checks --expect-wallet-id if given, and refuses --search-address (exit 1): drop the @N= prefixes to let the address place the cosigners.`
    - Restore `--search-address` row, append: `Not combinable with explicit --cosigner @N= (exit 1).`
12. **#42 (I).** fix-42 with the ruled reference, derived and expected strings. Manual, restore `--from` row: "cross-checks the own cosigner position (inferred by matching the derived key against the md1's slots)" → add `and its engraved master fingerprint (a different engraved fingerprint refuses, exit 4, unless --allow-mismatch)`.
13. **#44 (I).** In `descriptor_mode_verify_run` (xpub arm, `verify_bundle.rs:1755-1759`), mirror `bundle.rs:1615-1636`:
    - Parse `--slot @<idx>.fingerprint=` with error `BadInput` `--slot @<idx>.fingerprint parse: <e>` (exit 1).
    - If both the annotation and the slot fingerprint are present and differ: `DescriptorParse` `--slot @<idx>.fingerprint specifies <slot> but descriptor @<idx> annotation specifies <anno>` (exit 2, bundle's text verbatim).
    - Then `fp = slot_fp.or(fingerprint_annos[idx]).unwrap_or_default()` (fix-44).
    - New rows on the #44 fixture: `bughunt_tk_44b_descriptor_mode_verify_refuses_fingerprint_annotation_disagreement` (verify adds `--slot @1.fingerprint=deadbeef` → exit 2, stderr `annotation specifies`; unmodified: exit 4) and `bughunt_tk_44c_descriptor_mode_verify_refuses_unparseable_slot_fingerprint` (`--slot @1.fingerprint=zzzz` → exit 1, stderr `--slot @1.fingerprint parse:`; unmodified: not exit 1).
    - Manual: none (matches bundle).
14. **#47 (I).** Replace `hex::decode` at `verify_bundle.rs:892` with `decode_wallet_id_prefix` (ruled texts). Manual, verify-bundle `--expect-wallet-id` row, append: `An empty, odd-length/non-hex, or longer-than-16-byte prefix is refused (exit 1), as in restore.`
15. **#53 (I).** fix-53 verbatim (`ModeViolation`, exit 2, `taproot md1 repeats a key index inside its multi_a/sortedmulti_a leaf; the toolkit cannot reconstruct it faithfully`). Manual, restore Scope paragraph, add after the `@-in-both` sentence: `A multi_a / sortedmulti_a leaf that repeats a key index is refused (exit 2).`
16. **#3 (I).** fix-03 with the ruled text.
    - Test that flips: `parse_descriptor::tests::lex_residue_floor_accepts_multipath_sans_wildcard` → rename `lex_rejects_multipath_sans_wildcard`, asserting `lex_placeholders("wpkh(@0/<0;1>)")` is `Err(DescriptorParse)` whose message contains `is not representable in md1`.
    - Manual, "Non-representable use-site steps", add after the fixed-step paragraph: `A multipath with no trailing wildcard — @0/<0;1> (or an inline [fp/path]xpub/<0;1>) with no /* after it — names a non-ranged wallet md1 cannot encode either. It used to be encoded silently as the ranged /<0;1>/* wallet; it is now refused (exit 2, DescriptorParse) on the same surfaces. Bare @N, with no use-site at all, is unaffected.`
17. **#4 (I).** fix-04 verbatim (checked against upstream). Tests and manual: none.
18. **#5 (I).** fix-05 verbatim. Manual, export-wallet `--format` row (`41-mnemonic.md:1135`), append: `electrum refuses every taproot template (bip86, tr-multi-a, tr-sortedmulti-a; exit 1): Electrum's wallet file has no taproot wallet type.`
19. **#6 (I).** fix-06 verbatim (`DescriptorParse`, exit 2). Manual: none.
20. **#8 (I).** fix-08 verbatim (wc-codec `InvalidParams`). Tighten the row to `Err(WcError::InvalidParams)`. Manual: none.
21. **#9 (I).** fix-09 verbatim (`BadInput` `--slot @<idx>.fingerprint parse: <e>`, exit 1). Manual: none.
22. **#45 (I).** Not fix-45. Compare `args.network` with `envelope_network` right after `bundle.rs:2012`; on disagreement, refuse with the ruled text. Tests change: any existing test that passes a mismatched `--network` with `--import-json` flips to exit 1 (none failed with fix-45, but fix-45 accepted the mismatch; the suite decides).
23. **#46 (I).** fix-46's placement with the two ruled texts. Manual, bundle `--import-json` row (`41-mnemonic.md:359`), append: `--network must equal the envelope's bundle.network (a disagreement is refused, exit 1); only --slot @N.phrase= overlays are accepted, any other --slot subkey is refused (exit 1).` (This covers #45 too.)
24. **#52 (I).** fix-52 with the ruled failure detail (distinct from the cross-mix detail). Manual: none.
25. **#54 (I).** Not fix-54. In `emit_unified` (`bundle.rs:972`): `path_family: if template.is_none() { "descriptor" } else { args.multisig_path_family.unwrap_or_default().human_name() }`. Tighten the row to `== "descriptor"`.
    - Lockstep:
      - `docs/technical-manual/src/50-rust-api/54-mnemonic-toolkit-api.md:345` → `"bip48", "bip87", or "descriptor" (descriptor mode)`.
      - Regenerate `docs/manual/transcripts/41-bundle-inheritance-json.out`.
      - Regenerate the `docs/manual-gui/transcripts/tutorial/*-bundle-json.stdout.txt` that carry `"template":"descriptor"` (tut-j2/j3/j4 restore-feed, tut-j4-nums-feed, tut-j5-22; CI `manual-gui`).
      - The mnemonic-gui repo's `tests/snapshots/tutorial/tut-j{2,3}-restore-feed-bundle-json.stdout.txt` change when the GUI pins the new toolkit.

## Deferred

- `verify-bundle-multisig-json-md1-origin-row-on-decode-fail` (#14): verify-bundle multisig `--json` omits `md1_origin_match` when the md1 does not decode, although the code promises a fixed list. Push the row with `passed: true` and detail/decode_error `skipped: md1 decode failed`, like its siblings (fix-14.patch). Row `bughunt_tk_14_multisig_json_has_md1_origin_row_when_md1_does_not_decode` → `#[ignore = "deferred-minor: verify-bundle-multisig-json-md1-origin-row-on-decode-fail"]`.
- `wc-codec-sync-h1-unverified-later-checkpoints` (#7): `classify_single_insertion`'s H1 branch accepts the first marker+index match without checking later checkpoints, so `sync_classify` can return `Aligned` with wrong positions left unerased (end to end, RS plus the tag refuse).
  - Fix: after the trim, check every later checkpoint. Erase a later block whose checkpoint sits at its slot with the right index but fails its CRC. Refuse (`AmbiguousRealignment`) only when a later checkpoint is missing or misplaced.
  - fix-07.patch (refuse on any later failure) is rejected: it also refuses an insertion plus independent later-block damage. The follow-up adds a row proving such a card still decodes if it decodes today.
  - Row `bughunt_tk_07_single_insertion_never_aligned_with_unerased_wrong_positions` → `#[ignore = "deferred-minor: wc-codec-sync-h1-unverified-later-checkpoints"]`.

## Housekeeping

The scratch worktree `/scratch/code/shibboleth/tk-worktrees/grade-g1-scratch` (detached at `a9f781b2`, uncommitted scratch patches plus a `target/`) is **left in place**. RULES.md "PR rules" requires Brian's typed go-ahead for worktree removal; remove it with `git worktree remove --force` once that is given. Probe scripts and logs are in `/scratch/cache/bughunt/tk/g1-*`.
