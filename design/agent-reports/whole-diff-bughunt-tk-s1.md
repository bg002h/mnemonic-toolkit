# Whole-diff review: tk bughunt G1+G2, slice 1 (design, threat, executed advice)

Target: `bughunt/triage`, diff `a9476fe1..ad1761b5`. Reviewer: Opus, medium effort. First-round review (not a re-review).

## Findings (appended as found)

### Executed advice (passes)

- #2 `--format` refusal: ran on signet `wpkh`/`pkh`/`tr` template cards (account 2); refusal printed as specified (exit 2), and the printed alternative `restore --template <bip84|bip44|bip86> --account 2 --format bitcoin-core` (no `--md1`) yields the same descriptor and first address as the completion without `--format` in all three. bip49 is absent from the placeholder list, but `sh(wpkh)` cannot be minted as a template (refused at base and head), so the list is complete.
- 11b restore guard: base-minted `wpkh(@0/<2;3>/*)` and `tr(@0/*)` template cards are refused (exit 2) with and without `--from`. The advice "a --md1-form=policy card": a signet `wpkh([73c5da0a/84'/1'/0']@0/<2;3>/*)` policy card restores faithfully (`/<2;3>/*`, `tb1qpslzv42d…`). `mnemonic inspect --md1 <card>` prints `template: wpkh(@0/<2;3>/*)`, so the holder can recover the use-site.

- #45 `--network` remedy (bundle.rs:2049): a testnet-default BSMS envelope refused under `--network signet` with the long text; the printed remedy `import-wallet --network signet` from the same blob, then `bundle --network signet --import-json`, exits 0 and labels the bundle signet.
- #46 seed-overlay advice (bundle.rs:2122): on a signet 2-of-2 BSMS envelope (cosigner 0 = 73c5da0a at 48'/1'/0'/2'), `--slot @0.entropy=` is refused with the advice; following it literally (`convert --from entropy=- --to phrase`, strip `phrase: `, `--slot @0.phrase=-`) gives exit 0, mode `full`, ms1 `[set, empty]`. `convert --from ms1=-` and `--from seedqr=-` also print `phrase: <words>` as the advice says.

- X3 repair K-of-N advice (repair.rs:1252-1258): a signet 2-of-3 ms1 share set with one share corrupted is corrected (exit 4) with the UNVERIFIED reason; the printed `ms-shares combine` command, placeholders filled with the corrected share and one other, recovers the `abandon … about` seed (exit 0, warnings only). The BIP-93 claim it cites matches the raw BIP text (line 154: "SHOULD NOT automatically proceed with a corrected codex32 string without user confirmation").
- #51 inspect (inspect.rs:208-213): two positional ms1 cards are refused (exit 1) with the new text; the advice (inspect each card singly) works: the entr card exits 0, and a share gets the existing combine pointer. A repeated `--ms1` flag is still a clap error (exit 64), so only the positional form reaches the refusal.
- #1 xpub mismatch: a matching fingerprint with a wrong signet tpub gives exit 4, empty stdout and the MISMATCH/xpub-mismatch error (no descriptor or address printed). `--allow-mismatch` gives exit 0 with the overridden banner.
- #38 taproot non-standard use-site (restore.rs:~3995): the guard routes `Template(_, ik)` cards whose use-site is not `<0;1>/*` to the faithful arm. Addresses match the FIX's Core goldens: multi_a `/<2;3>/*` gives bc1prhzr704…, multi_a `/*` gives bc1plqgfat0…, and sortedmulti_a `/<2;3>/*` gives bc1pv963as6…. Override cards are routed by the earlier branch, so behaviour there is unchanged.
- #19 Coldcard BIP-45 root (coldcard_multisig.rs `is_bip45_root`): a tpub root resolves to Testnet and an xpub root to Bitcoin, the same mapping the existing coin-type path uses.
- X1 mixed-case refusal: the text matches the FIX (exit 2), and all-capitals input still repairs.
- Silent-payment refusal and advice: the printed alternative runs as printed. #41's remedies are covered by existing tests that pass on the head.

### Minors and Nits

- Minor (a): restore.rs 11b refusal ("use the wallet's full descriptor or a --md1-form=policy card instead") names no command for minting the policy card. It fails closed (exit 2, no address printed). Two unusual conditions: a base-minted template card with a non-standard use-site, and a holder without the original descriptor. File as deferred-minor.
- Nit (b): the #2 `--format` refusal says `--template <bip84|bip44|bip86>`; it could name the one value that matches the card being refused.
- Nit (c): production comments say "scratch fix #NN" (restore.rs:1897, 2014, 3334, 3995, 4191; verify_bundle.rs #12, #13); they should cite `FIX_bughunt_tk_g1.md` §2.x instead.

### Out of scope (outside the diff, unverified as a defect)

- `convert --template wsh-sortedmulti --network signet --to xpub` seems to ignore `--path`, and its xpub differs from bundle's 48'/1'/0'/2' derivation. Not graded here; it may need a triage.

### Round accounting

This is a first-round review: no previous fix commit, so 0 of these findings came from a fix commit.

C / I / M / N: 0 / 0 / 1 / 2
