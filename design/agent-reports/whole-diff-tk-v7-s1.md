**⚠ 2 Important: I-1 (J4 GUI note hides tier 4's single-key spend), I-2 (export-wallet --from-import-json trusts the envelope threshold).**

# Whole-diff review: toolkit FIX-V7 build, slice 1 (executed advice and design)

Reviewer: Claude Opus 5.5, effort high. First round on this build (not a re-review).
Target: `/scratch/code/shibboleth/tk-build-v7`, branch `build/v7-bundle-recovery`, head `9aef788e`,
diff `12e013bc..9aef788e`. Binary built from the head into
`/scratch/cache/whole-diff-tk-v7/s1/target` (`cargo 1.85.0`, debug). Probe scripts and outputs:
`/scratch/cache/whole-diff-tk-v7/s1/probe/` (`probe.sh` = `bundle --descriptor` card + `--json`;
`iw.sh` = `import-wallet` text/JSON, then `bundle --import-json` with and without `--threshold 1`,
then `export-wallet --from-import-json` in five formats). All keys are signet tpubs derived from
public BIP-39 test phrases at `m/48'/1'/0'/2'`.

## Findings

### I-1 (Important). J4 GUI interim note claims "three keys from different tiers spend nothing"; one tier-4 key spends alone after its lock

`docs/manual-gui/tutorial/50-j4-taproot-twin.md:130-132` (new in this diff):

> `Kint` alone can spend at any time, by the key path. Each fallback
> tier needs its own keys, so three keys from different tiers spend
> nothing, and tiers 1 and 2 also need the secret word.

The J4 descriptor (`tut-j4-15-canonicalise.stdout.txt`, keys abbreviated) is
`tr(A4,{or_i(…multi_a(3,A0,A1,A2)…, …multi_a(2,A3,B0,B1)…),or_i(and_v(v:older(65535),multi_a(2,B2,B3)),and_v(v:older(4255898),multi_a(1,C0,C1,C2)))})`.
Tier 4 is `multi_a(1, C0, C1, C2)` after `older(4255898)`. Followed literally, the note tells
the holder that any three keys taken from different tiers (e.g. `B0`, `B2`, `C0`) are harmless;
`C0` alone spends once tier 4's lock has passed. Machine check (head binary):

```
$ mnemonic compare-cost --miniscript 'or_i(and_v(v:older(65535),multi_a(2,B2,B3)),and_v(v:older(4255898),multi_a(1,C0,C1,C2)))'
B2 + B3 + older(blocks) | ...
C0 + older(512s)        | ...
C1 + older(512s)        | ...
C2 + older(512s)        | ...
```

This is the defect class FIX-V7 exists to remove (a recovery/spend text that "hides key sets
that can spend", CHANGELOG wording), introduced in new advice text. The J3 note gets this right
(it names the specific keys K0, K3, K6 and adds "after tier 4's lock any one of K8, K9 and K10
spends alone"); J4's note drops that clause and generalises.

Fix (replace the two sentences, `50-j4-taproot-twin.md:130-132`):

> `Kint` alone can spend at any time, by the key path. Tiers 1 to 3
> each need their own keys, so three keys taken one from each of tiers
> 1, 2 and 3 spend nothing, and tiers 1 and 2 also need the secret
> word; after tier 4's lock any one of its three keys spends alone.

(The file is outside the byte-compared GUI corpus, so the change needs only the GUI book's
cspell/lint and `make html`; see the run recorded under "Fix texts run" below.)

### I-2 (Important). `export-wallet --from-import-json` still trusts the envelope's `threshold`: a null on a plain descriptor loops on an unrunnable remedy, and a stale integer emits a wrong-K wallet

`crates/mnemonic-toolkit/src/cmd/export_wallet.rs:1007` (changed in this diff, `map` to
`and_then`): `let threshold = envelope.bundle.multisig.as_ref().and_then(|m| m.threshold);`.

FIX 2.3 decided two null cases (descriptor formats byte-identical; template formats refuse a
*general* policy) but not the third: a `null` threshold on a *plain* `multi`/`sortedmulti`
descriptor with a template format (sparrow, coldcard, electrum, jade). On base that envelope was
refused at parse (`invalid type: null, expected u8`); at head it reaches the template emitter with
`threshold: None` and gets a refusal whose remedy cannot be followed:

```
$ mnemonic export-wallet --from-import-json sm-null.json --format sparrow --network signet
error: mnemonic export-wallet --format sparrow requires the following missing fields:
  - threshold (multisig templates require --threshold <K>)
Re-invoke with all missing fields supplied.          (exit 2)
$ mnemonic export-wallet --from-import-json sm-null.json --format sparrow --network signet --threshold 2
error: mnemonic export-wallet --format sparrow requires the following missing fields:
  - threshold (multisig templates require --threshold <K>)        (exit 2, same text)
```

(`sm-null.json` = `import-wallet --json` of signet `wsh(sortedmulti(2,K0,K1,K2))` with
`bundle.multisig.threshold` set to `null`; coldcard/electrum/jade exit 1 with "requires
--threshold <K>".) No producer in this build or any release writes `null` for a plain descriptor
(`descriptor_is_general_policy` and `plain_multisig_threshold` agree on multi/sortedmulti), so the
route needs a hand-edited or third-party envelope; it fails closed. Graded Important because the
brief's likelihood test does not admit Minor when a printed remedy fails as printed, and because
FIX 2.3 left the case undecided (an implementer-must-choose gap). The same unrunnable remedy is
already reachable on base through an `n == 1` `wsh(multi(1,K))` envelope (base binary, measured),
so the loop itself predates this diff; the diff adds the null route.

The same line also keeps a pre-existing wrong-result path that FIX ruling 1 removed from `bundle`
but not from `export-wallet`: an envelope carrying a stale integer (threshold 3 on the 2-of-3
above) exports a Sparrow wallet with `"script": "wsh(sortedmulti(3,@0/**,@1/**,@2/**))"`
(base and head, measured), a different script and different addresses from the real wallet.
`bundle --import-json` recomputes K from the descriptor (row 23); `export-wallet` does not.

Fix (one change closes the null route, the stale-integer route and the `n == 1` loop): in
`run_from_import_json`, take K from the envelope's descriptor by FIX 2.1, ignoring the envelope's
`threshold` field, as `bundle --import-json` does. Replace `export_wallet.rs:1005-1007` with

```rust
    // FIX-V7 2.1: K comes from the envelope's descriptor (its md1), as on the
    // bundle --import-json path; the envelope's threshold field is ignored.
    let threshold: Option<u8> = {
        let md1: Vec<&str> = envelope.bundle.md1.iter().map(|s| s.as_str()).collect();
        md_codec::chunk::reassemble(&md1)
            .ok()
            .and_then(|d| crate::cmd::bundle::plain_multisig_threshold(&d.tree))
    };
```

and amend FIX 2.3's export-wallet bullet (and the SPEC fold's last sentence) to "`export-wallet
--from-import-json` also takes K from the envelope's descriptor by 2.1 and ignores the envelope's
`threshold`". Add this RED row to `cli_bundle_recovery_threshold_text.rs` (it uses the file's
`envelope_file` and `export_from_envelope` helpers):

```rust
#[test]
fn v7_export_wallet_takes_k_from_envelope_descriptor() {
    // A plain 2-of-3 envelope whose threshold is null or stale: every
    // template format emits the same file as the honest envelope, because K
    // comes from the envelope's descriptor (the rule bundle --import-json uses).
    let dir = tempfile::tempdir().unwrap();
    let t = "wsh(sortedmulti(2,K0,K1,K2))";
    let good = envelope_file(dir.path(), "good.json", t, Value::from(2));
    for (name, th) in [("null.json", Value::Null), ("stale.json", Value::from(3))] {
        let env = envelope_file(dir.path(), name, t, th);
        for format in ["sparrow", "coldcard", "electrum"] {
            let want = export_from_envelope(&good, format);
            assert!(
                want.status.success(),
                "{format}: {}",
                String::from_utf8_lossy(&want.stderr)
            );
            let got = export_from_envelope(&env, format);
            assert!(
                got.status.success(),
                "{name} {format}: {}",
                String::from_utf8_lossy(&got.stderr)
            );
            assert_eq!(
                String::from_utf8_lossy(&got.stdout),
                String::from_utf8_lossy(&want.stdout),
                "{name} {format}"
            );
        }
    }
}
```

Run as printed in a scratch clone of `9aef788e` (`/scratch/cache/whole-diff-tk-v7/s1/headfix`, own
target dir):
`PATH=$HOME/.cargo/bin:$PATH cargo nextest run --locked --no-fail-fast -p mnemonic-toolkit -E 'test(/(^|::)v7_export_wallet_takes_k_from_envelope_descriptor$/)'`

- RED (head production code, row added): exit 100, `Compiling mnemonic-toolkit` present,
  `panicked at …cli_bundle_recovery_threshold_text.rs:700:13: null.json sparrow: error: mnemonic export-wallet --format sparrow requires the following missing fields:`,
  `Summary [   0.019s] 1 test run: 0 passed, 1 failed, 4141 skipped`.
- GREEN (fix applied; filter widened to rows 20-23 as well): `Summary [   0.042s] 5 tests run: 5 passed, 4137 skipped`.
- Whole workspace with the fix (before the row was added): `Summary [   4.262s] 4215 tests run: 4215 passed, 38 skipped`;
  `cargo clippy --locked --all-targets -- -D warnings` exit 0 (fix and row applied).
- Re-run with the exact printed replacement (new comment included): `cargo +1.95.0 fmt --all -- --check`
  exit 0; `-E 'test(/(^|::)v7_/) | test(/unified_card_general_policy/)'` gives
  `Summary [   0.045s] 31 tests run: 31 passed, 4111 skipped` (the 30 FIX rows plus the new row).
- Probe matrix with the fix (`fix` vs head): null and stale envelopes now give the same stdout as
  the honest envelope in descriptor, bitcoin-core, bip388, sparrow, coldcard and electrum; every
  legitimate envelope (plain multi/sortedmulti, decay, timelock) is byte-identical to head; the
  `n == 1` `wsh(multi(1,K))` envelope now exports a 1-of-1 Sparrow wallet instead of the loop.

### Minors and Nits (one line each)

- **M-1** (FIX section 8 note, raised as the FIX invites) `restore --md1` still writes `threshold: null` for a pure-key `thresh` that `bundle`/`import-wallet` now report as K; no wrong number reaches a holder, but two emitters disagree on one descriptor. File a follow-up (owning phase: next toolkit cleanup) to have `restore.rs:4021` use `plain_multisig_threshold`.
- **N-1** `docs/technical-manual/src/40-bundle-formation/41-bundle-anatomy.md:136` says "the K comes from `bundle.rs::plain_multisig_threshold`"; in template mode K comes from `--threshold` (`bundle.rs:1383`). Add "(descriptor and `--import-json` modes)".
- **N-2** FIX section 2.3 says `multisig` "stays absent when n == 1"; both emitters write `"multisig": null` (probe: `bundle --descriptor 'wpkh(…)' --json` and `import-wallet --json`), which is what the SPEC fold and the manual (`41-mnemonic.md:1715`) correctly say. FIX-doc wording only.

## Verified correct (no finding)

Head binary, signet, `bundle --descriptor` card lines and `--json` `multisig.threshold`:

| descriptor | card | JSON |
|---|---|---|
| `wpkh(K0)`, `wsh(multi(1,K0))`, `tr(NUMS,pk(K0))` | neither line | `multisig: null` |
| `wsh(multi(2,…3))`, `wsh(sortedmulti)`, `sh(multi)`, `sh(wsh(sortedmulti))` | `2 of 3` + `any 2 of 3` | 2 |
| `tr(NUMS,multi_a(2,…3))`, `tr(NUMS,sortedmulti_a(2,…3))` | `2 of 3` + `any 2 of 3` | 2 |
| `wsh(thresh(2,pk,s:pk,s:pk))`, `sh(…)`, `sh(wsh(…))`, `tr(NUMS,thresh(…))`, `wsh(thresh(2,pkh,a:pkh,a:pkh))`, `thresh(1,pk,s:pk)` | plain, K | K |
| `tr(K0,multi_a(2,K1,K2,K3))`, `tr(NUMS,{multi_a,pk})`, `tr(NUMS,{multi_a,multi_a})`, `tr(K0,pk(K1))` | general, `keep all N` | null |
| `wsh(or_d(pk,and_v(v:pk,older)))`, decaying `thresh(…,sln:older)`, `thresh(…,sn:pk)`, `thresh(2,multi(1,…),s:pk,s:pk)`, `wsh(and_v(v:pk,pk))` | general | null |
| `--md1-form template`, `--privacy-preserving` | unchanged classification | same |

- Key reuse that would make K exceed the signing-key count (`multi(3,@0/<0;1>,@0/<2;3>,@1)`) is refused
  upstream (BIP-388 distinct-key), so `any K of N` never has K > N.
- Every multisig `--template` requires `--threshold`; the `tr-multi-a` / `tr-sortedmulti-a` templates
  encode the NUMS internal key (`inspect` of their md1), so "template mode is always plain" holds.
- `import-wallet` (descriptor and BSMS formats), text and `--json`, then `bundle --import-json` with and
  without `--threshold 1`: plain shapes keep K (flag ignored), general shapes print the general line
  and `null`/`none`; BSMS decaying fixtures print `threshold=none`; `wsh(multi(1,K))` prints `1`.
- `export-wallet --from-import-json` on honest envelopes: descriptor/bitcoin-core/bip388 pass through;
  template formats refuse general policies with the 2.3 text; taproot is refused upstream (pre-existing).
- No sibling repo (mnemonic-gui, refugium-wallet, mnemonic-engrave, mnemonic-refugium,
  mnemonic-transaction) reads `multisig.threshold` or `threshold=`.
- Manual transcripts `41-bundle-inheritance-{cards,json}` replay byte-identical against the head binary
  (cards `.out` differs only by the pre-existing trailing blank line); the include range `1-10` covers
  the card. The J3/J4 notes' quoted future Recovery lines equal the head binary's output byte for byte
  (run on the tutorial's own descriptors). J3's note (K0, K3, K6; tier 4 single key) is correct.
- FIX section 4 command as printed at head: `Compiling mnemonic-toolkit` present,
  `Summary [   0.056s] 30 tests run: 30 passed, 4111 skipped`.
- CHANGELOG, SPEC fold, manual and technical-manual statements checked against the probes above; the
  only mismatches are N-1 and N-2.

## Fix texts run

- I-1: the replacement applied to a scratch copy of `docs/manual-gui`
  (`/scratch/cache/whole-diff-tk-v7/s1/fixrun/manual-gui`, `old` text asserted to occur once):
  `cspell --config .cspell.json --no-progress tutorial/50-j4-taproot-twin.md tutorial/40-j3-degrading-vault.md`
  gives `CSpell: Files checked: 2, Issues found: 0 in 0 files.`; `markdownlint-cli2 tutorial/50-j4-taproot-twin.md`
  gives `Summary: 0 error(s)`. Not run: the full GUI `make lint` (needs the pinned mnemonic-gui clone).
- I-2: as recorded in I-2 (RED, GREEN, workspace, fmt, clippy, probe matrix).

Not in this slice: the five mutation survivors and tests-can-fail (s2).

C / I / M / N: 0 / 2 / 1 / 2
