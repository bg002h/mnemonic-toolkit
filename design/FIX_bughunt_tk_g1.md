# FIX: the tk bug-hunt G1 Criticals and Importants (slices S1 + S2) — restore reference checks (#1, #2), single-sig template use-site (#11, 11b), verify-bundle seed card (#37), taproot use-site (#38), and 23 Importants

Small-fix track (RULES.md "Process", small-fix track). Branch `bughunt/triage` of `mnemonic-toolkit`, at `2814a2c0` (it carries the S1 and S2 RED rows from `a9f781b2` and earlier). One PR with G2's FIX document (`design/FIX_bughunt_tk_g2.md`); the build applies this document first, then G2's (section 10). The toolkit manual is in the same repo, so its lockstep is in the same PR.

Sources, all in `design/agent-reports/`:

- `bughunt-2026-10-31-tk.md` (hunt report)
- `bughunt-triage-tk-s1.md`, `bughunt-triage-tk-s2.md` (triage; RED rows `tests/bughunt_s1.rs`, `tests/bughunt_s2.rs`, `crates/wc-codec/tests/bughunt_s2.rs`)
- `bughunt-grade-tk-g1.md` (grades and product rulings, `d78646bd`)
- `bughunt-grade-tk-g2.md` (`2814a2c0`), read only for the overlap check of section 10

Scope (controller, settled): every C and I of `bughunt-grade-tk-g1.md` at its final grade — 3 C (#1, #37, #38), 11b as C, and 23 I — with the ruled fix, not the triage patch where the grade overrode or amended it. The two Minors (#14, #7) are deferred (section 9.2). Where this document's text differs from the grade report, section 11 says so and why.

**How this document was checked.** A scratch worktree, `/scratch/code/shibboleth/tk-worktrees/grade-g1-scratch` (detached at `a9f781b2`; production source identical to `2814a2c0`), holds every production and manual edit below and nothing else. They are saved as two patches:

- production: `/scratch/cache/bughunt/tk/g1fix/fix-bughunt-tk-g1-reference.patch` (sha256 `b843217da328822153faafc4bb517aa4d850ea7d1233a06f85cf025e0550b93e`; 8 files, +519 −186; most of the size is #37 moving the keyed ms1 check block into a shared function unchanged);
- manual: `/scratch/cache/bughunt/tk/g1fix/fix-bughunt-tk-g1-manual.patch` (sha256 `90cd9859a89ef213633f4c87e885160bdba8dcdd14e40f6f0db07e60c31269dc`), produced by the script `/scratch/cache/bughunt/tk/g1fix/manual-edits.py` (sha256 `91746b96b59a91fb846e94d0c77fb7d61204c0ba6f3e67dfb4437ed2984e136b`), which asserts each old text occurs once.

`git apply --check` of both on this worktree at `2814a2c0` passes. The patches are evidence, not the build: the build writes the code from sections 2 and 3, and the doc decides where they differ.

All runs used `PATH=$HOME/.cargo/bin:$PATH` (cargo 1.85.0), `--locked`, no `--release`.

| check (scratch copy, all edits) | result |
|---|---|
| `cargo nextest run --locked --workspace --no-fail-fast` | 4148 run, 4111 passed, 37 failed, 20 skipped. The 37: 34 are G2's slice rows (`bughunt_s3/s4/s5`, `bughunt_tk_36`), fixed by G2's document; `bughunt_tk_14` and `bughunt_tk_07` (deferred, section 6 ignores them); `parse_descriptor::tests::lex_residue_floor_accepts_multipath_sans_wildcard` (flips, section 7). No other existing test changes. Log `/scratch/cache/bughunt/tk/g1fix/ref-suite2.log` |
| `cargo nextest run --locked --workspace -E 'binary(bughunt_s1) \| binary(bughunt_s2)'` | 28 run, 26 passed, 2 failed (#14, #7). On unmodified code (this worktree): 28 run, 0 passed, 28 failed (`red-cur.log`) |
| `cargo clippy --locked --all-targets --workspace -- -D warnings` | exit 0 |
| `cargo fmt --all --check` | 6 diffs in the edited files: the build runs `cargo fmt` |
| `docs/manual` `make lint` | the same 10 FAIL lines as unmodified `2814a2c0`, all in `44-mk-cli.md` (stale `mk` markers against the locally installed `mk`); this document adds none |
| `docs/manual` `make verify-examples` (fixed binary) | the same 8 drifted transcripts as unmodified code with the current binary (`44-mk-*`, `qs-26-recover-mk1`: local `mk` drift); `41-bundle-inheritance-json` drifts with the fixed binary until its `.out` is edited (section 5.14), and passes after |
| `docs/manual` `make anchor-check` | OK, 9 danglers, matching baseline |
| Bitcoin Core (`deriveaddresses`, offline mainnet node) | the four #38 golden addresses and the 11b policy-card remedy address (section 2) |

Every command printed below was run on two binaries: the current one, `target/debug/mnemonic` of this worktree (`mnemonic 0.105.1`, built at `a9f781b2`), and the scratch build with all edits. The runner is `/scratch/cache/bughunt/tk/g1fix/cases.py <binary> <out.json>`; it executes exactly the argv shown, with cwd `/scratch/cache/bughunt/tk/g1fix/fx` and the environment below, and its outputs are `cur.json` and `fix.json` in the same folder. Phrases and seed values go through the environment (`@env:VAR`) or stdin, never argv.

~~~sh
export PA='abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about'   # fp 73c5da0a
export SA='legal winner thank year wave sausage worth useful legal winner thank yellow'                    # fp b8688df1
export SB='letter advice cage absurd amount doctor acoustic avoid letter advice cage above'                # fp 28645006
export SC='zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong'                                              # fp 3f635a63
export E0=00000000000000000000000000000000 M0=ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f          # = $PA
FX=/scratch/cache/bughunt/tk/g1fix/fx; cd $FX
~~~

The fixture files in `$FX` were minted with the current binary from these seeds (`bundle … --json`, `import-wallet … --json`); each is named where it is used. In transcripts and printed commands, a run of more than 64 non-blank characters (an xpub, a card, a descriptor) is cut to its first 28 characters and `…`, also inside an error line (the prose quotes such an error in full where it matters); nothing else is cut. The full values are in the fixture files and in `cur.json` / `fix.json`. `stdout:` / `stderr:` show each stream's lines; `(empty)` means no bytes.

## 1. Summary

Criticals:

1. **#1 `restore` skipped `--expect-xpub` when `--expect-fingerprint` matched**, and reported success. Both references are now checked; a wrong xpub is `✗ MISMATCH`, exit 4.
2. **#37 `verify-bundle` on a single-sig template bundle never read the `--ms1` card**, so a seed card for another seed verified OK. The card is now checked with the keyed path's `ms1_decode` / `ms1_entropy_match` rows; a wrong card fails, exit 4. (The triage's fix-37 is not used: it broke `--origin`.)
3. **#38 `restore` of a `tr(NUMS, multi_a/sortedmulti_a)` card with a uniform non-standard use-site (`/<2;3>/*`, `/*`) printed the `/<0;1>/*` wallet.** It now restores the card's wallet (Core-verified addresses).
4. **11b `bundle --md1-form=template` accepted any single-sig use-site, and `restore` rebuilt every single-sig template as `/<0;1>/*`**, printing a different wallet under `✓ wallet-id verified`. Both ends now refuse a single-sig template whose use-site is not `/<0;1>/*` (exit 2). This also closes #11 (`/*h`).

Importants (restore): 5. **#2** template completion honours `--expect-fingerprint`, `--expect-xpub`, `--allow-mismatch`, and refuses `--format`. 6. **#10** a multisig-template completion and its candidate listing honour `--output FILE`. 7. **#40** the own key keeps `--account` when a cosigner is a bare xpub (search and explicit modes). 8. **#41** explicit `--cosigner @N=` with `--search-address` is refused (exit 1). 9. **#42** an own slot whose engraved fingerprint differs from the seed's is a mismatch (exit 4), not "verified". 10. **#53** a taproot leaf that repeats a key index is refused (exit 2).

Importants (verify-bundle): 11. **#12** a stray mk1 card no longer fails the correct cards. 12. **#13** the positional fallback keeps the supplied card order. 13. **#43** a supplied md1 with more cosigners than expected is a mismatch (exit 4), not a panic. 14. **#39** single-sig `md1_xpub_match` also compares the policy tree and use-site. 15. **#44** descriptor-mode verify uses the `[fp/path]@N` annotation fingerprint, refuses a disagreeing `--slot @N.fingerprint=` (exit 2) and an unparseable one (exit 1). 16. **#47** `--expect-wallet-id` is validated as on `restore`: empty, odd/non-hex and over-16-byte values are refused (exit 1). 17. **#52** an `--mk1` value with no readable chunk_set_id fails `mk1_template_stub_bind` (exit 4).

Importants (descriptor parsing, bundle, Electrum, wc-codec): 18. **#3** `/<a;b>` with no trailing `/*` is refused (exit 2), not encoded as `/<a;b>/*`. 19. **#4** the Electrum passphrase normalization uses Electrum's full CJK table. 20. **#5** `export-wallet --format electrum --template bip86` is refused (exit 1). 21. **#6** descriptor-mode `bundle` checks the annotation fingerprint for `entropy` and `ms1` slots (exit 2). 22. **#9** an unparseable `--slot @N.fingerprint=` is refused (exit 1). 23. **#11** (closed by 11b's gate). 24. **#45** `bundle --import-json` refuses a `--network` that disagrees with the envelope (exit 1). 25. **#46** `bundle --import-json` refuses every `--slot` subkey except `phrase` (exit 1). 26. **#54** descriptor-mode `bundle --json` reports `multisig.path_family: "descriptor"` (SPEC v0.3 §5.3). 27. **#8** `wc_codec::encode` refuses `u_slots` outside 1..=7 (`InvalidParams`).

That is 27 fixes (#11 and 11b share one gate). Nothing else changes for a holder: no flag, command or type is added; `--help` texts are unchanged; versions are unchanged in this PR.

## 2. Holder-visible behaviour, per fix

Errors print as `error: <message>` on stderr. Exit codes follow the `ToolkitError` variant: `BadInput` 1, `ModeViolation` / `DescriptorParse` / `TemplateFormUnsupportedShape` 2, `RestoreMismatch` 4. A verify-bundle run with a failed check row prints `mismatch` (text) or `"result":"mismatch"` (`--json`) and exits 4. "Spec" and "Manual" name the section 4 / 5 entry, or say "none".

### 2.1 #1 (C) `restore`: `--expect-xpub` is checked even when `--expect-fingerprint` matches

**Behaviour.** In the seed-only single-sig path (`restore --template …`), `--expect-fingerprint` is checked first; if it matches or is absent, `--expect-xpub` is checked. A mismatch of either is the existing hard gate: stderr `✗ MISMATCH`, then `master fingerprint: <fp>  (passphrase: applied|none)`, then the `RestoreMismatch` error; nothing on stdout; exit 4. `--allow-mismatch` is unchanged (descriptors under the overridden banner, exit 0). Texts: all existing. Spec: none (`SPEC_mnemonic_restore.md` lines 38-39 already require each reference to match). Manual: none.

The `--expect-xpub` value below is the account-1 bip84 xpub of `$PA` (`ACCT1_XPUB_BIP84` in the RED row); restore derives account 0.

~~~text
$ printf '%s\n' "$PA" | mnemonic restore --from phrase=- --template bip84 --expect-fingerprint 73c5da0a --expect-xpub xpub6CatWdiZiodmYVtWLtEQsAg1…
-- current
exit=0
stdout: master fingerprint: 73c5da0a  (passphrase: none)
stdout: CONFIRM: this fingerprint matches the wallet you are restoring before importing any descriptor.
stdout: 
stdout: bip84 (native segwit P2WPKH):
stdout:   descriptor: wpkh([73c5da0a/84'/0'/0']xpu…
stdout:   first recv: bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=4
stdout: (empty)
stderr: ✗ MISMATCH
stderr: master fingerprint: 73c5da0a  (passphrase: none)
stderr: error: restore: xpub mismatch — derived xpub6CatWdiZiodmUeTDp8LT5or8… expected xpub6CatWdiZiodmYVtWLtEQsAg1…
~~~

The full error line after the fix is: `error: restore: xpub mismatch — derived xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V, expected xpub6CatWdiZiodmYVtWLtEQsAg1H9ooS1bmsJUBwQ83FE1Fyk386FWcyicJgEZv3quZSJKA5dh5Lo2PbubMGxCfZtRthV6ST2qquL9w3HSzcUn`.

### 2.2 #2 (I) `restore` of a keyless single-sig template: reference flags honoured, `--format` refused

**Behaviour**, in `run_singlesig_template_completion`, in this order:

1. The 11b use-site guard (2.9).
2. `--format` (any value) is refused: `ModeViolation{mode:"restore", flag:"--format"}`, exit 2, nothing on stdout, text: `--format is not supported when completing a keyless single-sig template md1; run without --format to print the completed descriptor, or (canonical origin only) restore with --template <bip44|bip84|bip86> --account <N> --format <FORMAT> and no --md1`
3. `--from` required (unchanged), seed and key derivation (unchanged).
4. After the concrete descriptor is built and before `--expect-wallet-id`: `--expect-fingerprint` (compared with the master fingerprint, trimmed, lowercased), then — if it matched or was absent — `--expect-xpub` (compared with the account xpub at the template default or `--origin`). On a mismatch without `--allow-mismatch`: stderr `✗ MISMATCH`, then `master fingerprint: <fp>  (passphrase: applied|none)`, then `RestoreMismatch{reference:"fingerprint"|"xpub"}` (texts `restore: fingerprint mismatch — derived <fp>, expected <exp>` / `restore: xpub mismatch — derived <xpub>, expected <exp>`), exit 4, nothing on stdout.
5. With `--allow-mismatch`, the output is printed as usual and then stderr carries the keyed path's banner verbatim: `✗ MISMATCH (overridden): derived material does NOT match the supplied reference; descriptors above were produced by the passphrase you provided, NOT the expected wallet`. Exit 0.
6. The `UNVERIFIED: no --expect-wallet-id supplied; …` banner (text unchanged) is printed only when none of `--expect-wallet-id`, `--expect-fingerprint`, `--expect-xpub` was given, and never together with the overridden banner.

`--json`: the template-completion envelope is unchanged (it has no `verification` object, before or after); a refused run prints no JSON. Spec: 4.1 (b). Manual: 5.3, 5.10, 5.11.

~~~text
$ printf '%s\n' "$PA" | mnemonic restore --network mainnet --from phrase=- --md1 md1fvwrwqqpqqgqpsqqq3uaau4ctxyl7 --expect-fingerprint deadbeef
-- current
exit=0
stdout: master fingerprint: 73c5da0a  (passphrase: none)
stdout: CONFIRM: this fingerprint matches the wallet you are restoring before importing.
stdout: 
stdout: bip84 (native segwit P2WPKH):
stdout:   descriptor: wpkh([73c5da0a/84'/0'/0']xpu…
stdout:   first recv: bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
stderr: UNVERIFIED: no --expect-wallet-id supplied; verify the master fingerprint above (73c5da0a) against your records before importing
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=4
stdout: (empty)
stderr: ✗ MISMATCH
stderr: master fingerprint: 73c5da0a  (passphrase: none)
stderr: error: restore: fingerprint mismatch — derived 73c5da0a, expected deadbeef
~~~

~~~text
$ printf '%s\n' "$PA" | mnemonic restore --network mainnet --from phrase=- --md1 md1fvwrwqqpqqgqpsqqq3uaau4ctxyl7 --expect-xpub xpub6CatWdiZiodmYVtWLtEQsAg1…
-- current
exit=0
stdout: master fingerprint: 73c5da0a  (passphrase: none)
stdout: CONFIRM: this fingerprint matches the wallet you are restoring before importing.
stdout: 
stdout: bip84 (native segwit P2WPKH):
stdout:   descriptor: wpkh([73c5da0a/84'/0'/0']xpu…
stdout:   first recv: bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
stderr: UNVERIFIED: no --expect-wallet-id supplied; verify the master fingerprint above (73c5da0a) against your records before importing
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=4
stdout: (empty)
stderr: ✗ MISMATCH
stderr: master fingerprint: 73c5da0a  (passphrase: none)
stderr: error: restore: xpub mismatch — derived xpub6CatWdiZiodmUeTDp8LT5or8… expected xpub6CatWdiZiodmYVtWLtEQsAg1…
~~~

~~~text
$ printf '%s\n' "$PA" | mnemonic restore --network mainnet --from phrase=- --md1 md1fvwrwqqpqqgqpsqqq3uaau4ctxyl7 --expect-fingerprint deadbeef --allow-mismatch
-- current
exit=0
stdout: master fingerprint: 73c5da0a  (passphrase: none)
stdout: CONFIRM: this fingerprint matches the wallet you are restoring before importing.
stdout: 
stdout: bip84 (native segwit P2WPKH):
stdout:   descriptor: wpkh([73c5da0a/84'/0'/0']xpu…
stdout:   first recv: bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
stderr: UNVERIFIED: no --expect-wallet-id supplied; verify the master fingerprint above (73c5da0a) against your records before importing
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=0
stdout: master fingerprint: 73c5da0a  (passphrase: none)
stdout: CONFIRM: this fingerprint matches the wallet you are restoring before importing.
stdout: 
stdout: bip84 (native segwit P2WPKH):
stdout:   descriptor: wpkh([73c5da0a/84'/0'/0']xpu…
stdout:   first recv: bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
stderr: ✗ MISMATCH (overridden): derived material does NOT match the supplied reference; descriptors above were produced by the passphrase you provided, NOT the expected wallet
stderr: note: stdout is watch-only — public keys only, cannot spend
~~~

~~~text
$ printf '%s\n' "$PA" | mnemonic restore --network mainnet --from phrase=- --md1 md1fvwrwqqpqqgqpsqqq3uaau4ctxyl7 --format bitcoin-core
-- current
exit=0
stdout: master fingerprint: 73c5da0a  (passphrase: none)
stdout: CONFIRM: this fingerprint matches the wallet you are restoring before importing.
stdout: 
stdout: bip84 (native segwit P2WPKH):
stdout:   descriptor: wpkh([73c5da0a/84'/0'/0']xpu…
stdout:   first recv: bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
stderr: UNVERIFIED: no --expect-wallet-id supplied; verify the master fingerprint above (73c5da0a) against your records before importing
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=2
stdout: (empty)
stderr: error: --format is not supported when completing a keyless single-sig template md1; run without --format to print the completed descriptor, or (canonical origin only) restore with --template <bip44|bip84|bip86> --account <N> --format <FORMAT> and no --md1
~~~

Control — a matching reference now suppresses the UNVERIFIED banner:

~~~text
$ printf '%s\n' "$PA" | mnemonic restore --network mainnet --from phrase=- --md1 md1fvwrwqqpqqgqpsqqq3uaau4ctxyl7 --expect-fingerprint 73c5da0a
-- current
exit=0
stdout: master fingerprint: 73c5da0a  (passphrase: none)
stdout: CONFIRM: this fingerprint matches the wallet you are restoring before importing.
stdout: 
stdout: bip84 (native segwit P2WPKH):
stdout:   descriptor: wpkh([73c5da0a/84'/0'/0']xpu…
stdout:   first recv: bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
stderr: UNVERIFIED: no --expect-wallet-id supplied; verify the master fingerprint above (73c5da0a) against your records before importing
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=0
stdout: master fingerprint: 73c5da0a  (passphrase: none)
stdout: CONFIRM: this fingerprint matches the wallet you are restoring before importing.
stdout: 
stdout: bip84 (native segwit P2WPKH):
stdout:   descriptor: wpkh([73c5da0a/84'/0'/0']xpu…
stdout:   first recv: bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
stderr: note: stdout is watch-only — public keys only, cannot spend
~~~

The refusal's alternative, run as printed (no change; shown so the advice is executed):

~~~text
$ printf '%s\n' "$PA" | mnemonic restore --network mainnet --from phrase=- --template bip84 --account 0 --format bitcoin-core
-- current (unchanged after the fix)
exit=0
stdout: [
stdout:   {
stdout:     "active": true,
stdout: … (22 lines in all; the import payload)
stderr: master fingerprint: 73c5da0a  (passphrase: none)
stderr: CONFIRM: this fingerprint matches the wallet you are restoring before importing the payload above.
stderr: bip84 (native segwit P2WPKH):
stderr:   descriptor: wpkh([73c5da0a/84'/0'/0']xpu…
stderr:   first recv: bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
stderr: UNVERIFIED: no --expect-fingerprint/--expect-xpub supplied; verify the master fingerprint above (73c5da0a) against your records before importing
stderr: note: stdout is watch-only — public keys only, cannot spend
~~~

### 2.3 #10 (I) `restore` multisig-template completion and candidate listing honour `--output FILE`

**Behaviour.** `emit_completed_multisig` and `emit_candidate_list` write their stdout content (the text document, the listing rows, or the `--json` object) to `--output <FILE>` when its value is not `-`, else to stdout. Stderr lines are unchanged. A write failure is the existing `BadInput` `--output <path>: <io error>`, exit 1. No new text. Spec: 4.3 (listing). Manual: none (the `--output` row already promises this).

Fixtures: `tsm2_md1.txt` / `tsm2_id.txt` — the keyless template md1 and wallet-id of the 2-of-2 BIP-48 `wsh-sortedmulti` over `$SA`, `$SB` (account 0); `tsm2_mk1_b.txt` — `$SB`'s policy-form mk1. For the listing, `mk1_b.txt`, `mk1_c.txt` — `$SB`'s and `$SC`'s mk1 from the 2-of-3 BIP-48 `wsh-multi` over `$SA`,`$SB`,`$SC`; `mk1_out.txt` — `$PA`'s mk1 from a 2-of-2 `wsh-multi` with `$SA` (an outsider key). The 2-byte prefix `a693` over the 592128-assignment opt-in space matches 9 assignments (deterministic).

~~~text
$ mnemonic restore --network mainnet --md1 $(cat tsm2_md1.txt) --from phrase=@env:SA --account 0 --expect-wallet-id $(cat tsm2_id.txt) --output out/restored.txt $(sed 's/^/--cosigner /' tsm2_mk1_b.txt)
-- current
exit=0
stdout: multisig wallet completed from template:
stdout:   descriptor: wsh(sortedmulti(2,[b8688df1/…
stdout:   first recv: bc1q2a4vm7ww7v0c02qerrgg0znr4ck4kez2la82kzpm9gturx68q4nsfdl000
stderr:   scan complete in 25ms
stderr: ✓ wallet-id (completed): 3cf8c584d31c9871a863ec2c364c6396
stderr:   your seed completes cosigner slot @0
-- after the fix
exit=0
stdout: (empty)
stderr:   scan complete in 25ms
stderr: ✓ wallet-id (completed): 3cf8c584d31c9871a863ec2c364c6396
stderr:   your seed completes cosigner slot @0
file:
  multisig wallet completed from template:
    descriptor: wsh(sortedmulti(2,[b8688df1/…
    first recv: bc1q2a4vm7ww7v0c02qerrgg0znr4ck4kez2la82kzpm9gturx68q4nsfdl000
~~~

~~~text
$ mnemonic restore --network mainnet --md1 md1fcgp9qqpqsgqpsgvzyxqqkfp25qywj3q3p --from phrase=@env:SA --own-account-max 256 --search-cosigner-subset --expect-wallet-id a693 $(sed 's/^/--cosigner /' mk1_b.txt mk1_c.txt mk1_out.txt) --output out/list.txt
-- current
exit=0
stdout:   [1] wallet-id a693d0f4447150c94fce12d852c19af5
stdout:       @0=b8688df1 @1=28645006 @2=3f635a63
stdout:       first recv bc1qhsmtw8hxhlx89z383hya3wnlw2mtsdnwg9jxq2ds6nzs449max3stw2qy9
stdout:   [2] wallet-id a69322cce7e49f7aaf037fe54d41ec03
stdout:       @0=28645006 @1=b8688df1 @2=b8688df1
stdout:       first recv bc1q49rlw99vd46a6vj2m5xsayhgwgct4sjleh3hm7fk93wa2rhkd93sn2kj3a
stdout:   [3] wallet-id a693ab7a9c9c46e0f4982d2a9d033185
stdout:       @0=b8688df1 @1=b8688df1 @2=3f635a63
stdout:       first recv bc1qhgf99wvx8kyzfqy6f6npv8fuqst08dezpm4nkxhwz95kwq25rjzsxw4m9f
stdout:   [4] wallet-id a6938beade205ad8c9ab824b9475e5c3
stdout:       @0=28645006 @1=b8688df1 @2=b8688df1
stdout:       first recv bc1q67xxmlxwlfah8tgwwwxhyjff7m53v068pg43lnyrp7rusxarlktswjzrls
stdout:   [5] wallet-id a693d1f087f8f2b5135abf4eb2ac0916
stdout:       @0=b8688df1 @1=b8688df1 @2=28645006
stdout:       first recv bc1qxxyeuqs8g52sthul3yfsha8ewuxt0n2kqnfahxj988x4029f9nhsfwnv5r
stdout:   [6] wallet-id a69356272805fdb1e54b0f2e170e9695
stdout:       @0=b8688df1 @1=3f635a63 @2=b8688df1
stdout:       first recv bc1qjm0amuf6ymargt643apqnscwsjk9375rj0656fxcaxfusgec46tsuxkkrk
stdout:   [7] wallet-id a6931929346a275b2223afaca9c1dca4
stdout:       @0=b8688df1 @1=73c5da0a @2=b8688df1
stdout:       first recv bc1qwefucqc70trepgmyfnm3t04lyhal3645445v67jglm77glqgusyqh78s4x
stdout:   [8] wallet-id a69310a3013e976d047f7b47c1400b8c
stdout:       @0=b8688df1 @1=3f635a63 @2=b8688df1
stdout:       first recv bc1qhh8fhrr5eyzqx5s0wd99yzh3uakpfpl0z6cusw2z2mlk7r3e475qn9euar
stdout:   [9] wallet-id a693d560665ce217bee1a1f950e4dfef
stdout:       @0=b8688df1 @1=b8688df1 @2=28645006
stdout:       first recv bc1qr5zsdx6a8x8slcxhkstsgkr4n23j52sunqq3k0l7h9hwth0ucljsfkvvnk
stderr:   scan complete in 175ms
stderr: 9 assignments match prefix a693 (of 592128 opt_in_subsets in the realized space); none reconstructed — supply more id, or re-run with --search-address instead.
-- after the fix
exit=0
stdout: (empty)
stderr:   scan complete in 175ms
stderr: 9 assignments match prefix a693 (of 592128 opt_in_subsets in the realized space); none reconstructed — supply more id, or re-run with --search-address instead.
file:
    [1] wallet-id a693d0f4447150c94fce12d852c19af5
        @0=b8688df1 @1=28645006 @2=3f635a63
        first recv bc1qhsmtw8hxhlx89z383hya3wnlw2mtsdnwg9jxq2ds6nzs449max3stw2qy9
    [2] wallet-id a69322cce7e49f7aaf037fe54d41ec03
        @0=28645006 @1=b8688df1 @2=b8688df1
        first recv bc1q49rlw99vd46a6vj2m5xsayhgwgct4sjleh3hm7fk93wa2rhkd93sn2kj3a
    [3] wallet-id a693ab7a9c9c46e0f4982d2a9d033185
        @0=b8688df1 @1=b8688df1 @2=3f635a63
        first recv bc1qhgf99wvx8kyzfqy6f6npv8fuqst08dezpm4nkxhwz95kwq25rjzsxw4m9f
    [4] wallet-id a6938beade205ad8c9ab824b9475e5c3
        @0=28645006 @1=b8688df1 @2=b8688df1
        first recv bc1q67xxmlxwlfah8tgwwwxhyjff7m53v068pg43lnyrp7rusxarlktswjzrls
    [5] wallet-id a693d1f087f8f2b5135abf4eb2ac0916
        @0=b8688df1 @1=b8688df1 @2=28645006
        first recv bc1qxxyeuqs8g52sthul3yfsha8ewuxt0n2kqnfahxj988x4029f9nhsfwnv5r
    [6] wallet-id a69356272805fdb1e54b0f2e170e9695
        @0=b8688df1 @1=3f635a63 @2=b8688df1
        first recv bc1qjm0amuf6ymargt643apqnscwsjk9375rj0656fxcaxfusgec46tsuxkkrk
    [7] wallet-id a6931929346a275b2223afaca9c1dca4
        @0=b8688df1 @1=73c5da0a @2=b8688df1
        first recv bc1qwefucqc70trepgmyfnm3t04lyhal3645445v67jglm77glqgusyqh78s4x
    [8] wallet-id a69310a3013e976d047f7b47c1400b8c
        @0=b8688df1 @1=3f635a63 @2=b8688df1
        first recv bc1qhh8fhrr5eyzqx5s0wd99yzh3uakpfpl0z6cusw2z2mlk7r3e475qn9euar
    [9] wallet-id a693d560665ce217bee1a1f950e4dfef
        @0=b8688df1 @1=b8688df1 @2=28645006
        first recv bc1qr5zsdx6a8x8slcxhkstsgkr4n23j52sunqq3k0l7h9hwth0ucljsfkvvnk
~~~

~~~text
$ mnemonic restore --network mainnet --md1 md1fcgp9qqpqsgqpsgvzyxqqkfp25qywj3q3p --from phrase=@env:SA --own-account-max 256 --search-cosigner-subset --expect-wallet-id a693 $(sed 's/^/--cosigner /' mk1_b.txt mk1_c.txt mk1_out.txt) --json --output out/list.json
-- current
exit=0
stdout: {"candidates":[{"assignment"…
stderr:   scan complete in 175ms
stderr: 9 assignments match prefix a693 (of 592128 opt_in_subsets in the realized space); none reconstructed — supply more id, or re-run with --search-address instead.
-- after the fix
exit=0
stdout: (empty)
stderr:   scan complete in 175ms
stderr: 9 assignments match prefix a693 (of 592128 opt_in_subsets in the realized space); none reconstructed — supply more id, or re-run with --search-address instead.
file:
  {"candidates":[{"assignment"…
~~~

### 2.4 #40 (I) multisig-template completion keeps `--account` for the own key when a cosigner is a bare xpub

**Behaviour.** In `complete_multisig_template`'s `own_origin_for`, when the cosigner family yields no own origin (a bare xpub has an empty origin), the canonical fallback gets the account substituted (`own_origin_from_family(&fallback, acct)`), as the all-own branch already does. This applies to every mode (id-search, address-search, explicit `@N=`). No text. Spec: none. Manual: none.

Fixtures: `t40_md1.txt` — the template md1 of a 2-of-2 BIP-48 `wsh-sortedmulti` with `$SA` at account 1 (`m/48'/0'/1'/2'`) and `$SB` at account 0; `xb0.txt` — `$SB`'s `m/48'/0'/0'/2'` xpub, bare. The wallet's true first addresses (from its policy card): `bc1qutaes7u4vv3fn9wtmdfpx3wvset9exs59un9r00aw7lyryxd3ymqsf73hu`, `bc1qfaen6p88s6nzyz6nxcnp6j08vgvs7jfk7u9q8qttrad5xrfcsw3srenn5m`.

Explicit mode (the case the grade report raised to I: a wrong wallet at exit 0):

~~~text
$ mnemonic restore --network mainnet --md1 $(cat t40_md1.txt) --from phrase=@env:SA --account 1 --cosigner @1=$(cat xb0.txt)
-- current
exit=0
stdout: multisig wallet completed from template:
stdout:   descriptor: wsh(sortedmulti(2,[b8688df1/…
stdout:   first recv: bc1q2a4vm7ww7v0c02qerrgg0znr4ck4kez2la82kzpm9gturx68q4nsfdl000
stderr: warning: explicit --cosigner @N= mode builds the wallet from the ASSERTED key→slot assignment WITHOUT verifying it against a recorded id/address. A wrong assignment produces a wrong wallet silently. Record + check --expect-wallet-id or a receive address.
stderr: ✓ wallet-id (completed): 0378674e410e0d83fff4d305101eccaf
stderr:   your seed completes cosigner slot @0
-- after the fix
exit=0
stdout: multisig wallet completed from template:
stdout:   descriptor: wsh(sortedmulti(2,[b8688df1/…
stdout:   first recv: bc1qutaes7u4vv3fn9wtmdfpx3wvset9exs59un9r00aw7lyryxd3ymqsf73hu
stderr: warning: explicit --cosigner @N= mode builds the wallet from the ASSERTED key→slot assignment WITHOUT verifying it against a recorded id/address. A wrong assignment produces a wrong wallet silently. Record + check --expect-wallet-id or a receive address.
stderr: ✓ wallet-id (completed): a58f97f835dbe106238ee3dac1fbe6fb
stderr:   your seed completes cosigner slot @0
~~~

In the explicit-mode run the own key's origin is `[b8688df1/48'/0'/0'/2']` today and `[b8688df1/48'/0'/1'/2']` after the fix (the descriptor line is cut above).

Address-search (fails closed today):

~~~text
$ mnemonic restore --network mainnet --md1 $(cat t40_md1.txt) --from phrase=@env:SA --account 1 --search-address $(head -1 t40_golden.txt) --cosigner $(cat xb0.txt) --count 2
-- current
exit=4
stdout: (empty)
stderr:   scan complete in 25ms
stderr: ✗ NO MATCH
stderr: error: restore: multisig-template-search mismatch — derived no key→slot assignment of the supplied keys, expected the recorded wallet (--expect-wallet-id / --search-address)
-- after the fix
exit=0
stdout: multisig wallet completed from template:
stdout:   descriptor: wsh(sortedmulti(2,[b8688df1/…
stdout:   first recv: bc1qutaes7u4vv3fn9wtmdfpx3wvset9exs59un9r00aw7lyryxd3ymqsf73hu
stdout:   first recv: bc1qfaen6p88s6nzyz6nxcnp6j08vgvs7jfk7u9q8qttrad5xrfcsw3srenn5m
stderr:   scan complete in 25ms
stderr: ✓ wallet-id (completed): a58f97f835dbe106238ee3dac1fbe6fb
stderr:   your seed completes cosigner slot @0
~~~

### 2.5 #41 (I) explicit `--cosigner @N=` cannot combine with `--search-address`

**Behaviour.** In `complete_multisig_template`, before the subset-search conflict check: if any `--cosigner` is assigned (`@N=`) and `--search-address` is given, refuse with `BadInput`, exit 1, nothing on stdout, text: `explicit @N= assignment cannot combine with --search-address: @N= asserts the key→slot placement, so the address would not be checked. Drop the @N= prefixes to let --search-address place the cosigners, or keep them and check the placement with --expect-wallet-id instead of --search-address.` Explicit mode with `--expect-wallet-id` is unchanged (A2: a wrong placement refuses, exit 4). Spec: 4.2. Manual: 5.5, 5.9.

Fixtures: the 2-of-3 BIP-48 `wsh-multi` template md1 over `$SA`,`$SB`,`$SC` (`md1fcgp9…`, wallet-id `a693d0f4447150c94fce12d852c19af5`); `mk1_b41.txt` / `mk1_c41.txt` — `$SB`'s / `$SC`'s mk1; `t41_addr.txt` — the true first address `bc1qhsmtw8hxhlx89z383hya3wnlw2mtsdnwg9jxq2ds6nzs449max3stw2qy9`. The truth is `@1=B`, `@2=C`; the commands assert them swapped.

~~~text
$ mnemonic restore --network mainnet --md1 md1fcgp9qqpqsgqpsgvzyxqqkfp25qywj3q3p --from phrase=@env:SA --account 0 --search-address $(cat t41_addr.txt) --count 1 $(sed 's/^/--cosigner @1=/' mk1_c41.txt) $(sed 's/^/--cosigner @2=/' mk1_b41.txt)
-- current
exit=0
stdout: multisig wallet completed from template:
stdout:   descriptor: wsh(multi(2,[b8688df1/48'/0'…
stdout:   first recv: bc1qfxjd9qcr30knmzzxmvmhkewl3a33lrrce7gvw3gtvxkeava9ph3se6ny5s
stderr: warning: explicit --cosigner @N= mode builds the wallet from the ASSERTED key→slot assignment WITHOUT verifying it against a recorded id/address. A wrong assignment produces a wrong wallet silently. Record + check --expect-wallet-id or a receive address.
stderr: ✓ wallet-id (completed): 4fd92f23d243c80ee55642dbd4b790c7
stderr:   your seed completes cosigner slot @0
-- after the fix
exit=1
stdout: (empty)
stderr: error: explicit @N= assignment cannot combine with --search-address: @N= asserts the key→slot placement, so the address would not be checked. Drop the @N= prefixes to let --search-address place the cosigners, or keep them and check the placement with --expect-wallet-id instead of --search-address.
~~~

The advice, followed literally. Dropping the `@N=` prefixes:

~~~text
$ mnemonic restore --network mainnet --md1 md1fcgp9qqpqsgqpsgvzyxqqkfp25qywj3q3p --from phrase=@env:SA --account 0 --count 1 --search-address $(cat t41_addr.txt) $(sed 's/^/--cosigner /' mk1_c41.txt mk1_b41.txt)
-- current (unchanged after the fix)
exit=0
stdout: multisig wallet completed from template:
stdout:   descriptor: wsh(multi(2,[b8688df1/48'/0'…
stdout:   first recv: bc1qhsmtw8hxhlx89z383hya3wnlw2mtsdnwg9jxq2ds6nzs449max3stw2qy9
stderr:   scan complete in 25ms
stderr: ✓ wallet-id (completed): a693d0f4447150c94fce12d852c19af5
stderr:   your seed completes cosigner slot @0
~~~

Keeping the (swapped) placements and checking them with the recorded id instead:

~~~text
$ mnemonic restore --network mainnet --md1 md1fcgp9qqpqsgqpsgvzyxqqkfp25qywj3q3p --from phrase=@env:SA --account 0 --count 1 --expect-wallet-id a693d0f4447150c94fce12d852c19af5 $(sed 's/^/--cosigner @1=/' mk1_c41.txt) $(sed 's/^/--cosigner @2=/' mk1_b41.txt)
-- current (unchanged after the fix)
exit=4
stdout: (empty)
stderr: warning: explicit --cosigner @N= mode builds the wallet from the ASSERTED key→slot assignment WITHOUT verifying it against a recorded id/address. A wrong assignment produces a wrong wallet silently. Record + check --expect-wallet-id or a receive address.
stderr: ✗ NO MATCH
stderr: error: restore: multisig-template-explicit mismatch — derived wallet-id 4fd92f23d243c80ee55642dbd4b790c7, expected --expect-wallet-id a693d0f4447150c94fce12d852c19af5 — the asserted --cosigner @N= placement does NOT build the wallet you recorded; re-check the placements, or drop them and let the search resolve the assignment
~~~

The correct placements with the recorded id:

~~~text
$ mnemonic restore --network mainnet --md1 md1fcgp9qqpqsgqpsgvzyxqqkfp25qywj3q3p --from phrase=@env:SA --account 0 --count 1 --expect-wallet-id a693d0f4447150c94fce12d852c19af5 $(sed 's/^/--cosigner @1=/' mk1_b41.txt) $(sed 's/^/--cosigner @2=/' mk1_c41.txt)
-- current (unchanged after the fix)
exit=0
stdout: multisig wallet completed from template:
stdout:   descriptor: wsh(multi(2,[b8688df1/48'/0'…
stdout:   first recv: bc1qhsmtw8hxhlx89z383hya3wnlw2mtsdnwg9jxq2ds6nzs449max3stw2qy9
stderr: warning: explicit --cosigner @N= mode builds the wallet from the ASSERTED key→slot assignment WITHOUT verifying it against a recorded id/address. A wrong assignment produces a wrong wallet silently. Record + check --expect-wallet-id or a receive address.
stderr: ✓ wallet-id (completed): a693d0f4447150c94fce12d852c19af5
stderr:   your seed completes cosigner slot @0
~~~

### 2.6 #42 (I) keyed multisig `restore --from`: the own slot's engraved fingerprint must match the seed

**Behaviour.** In `run_multisig`, when the seed's key matches slot `@i` (65-byte compare) but the slot's engraved master fingerprint differs from the seed's, that is a mismatch: `RestoreMismatch{reference:"cosigner-fingerprint", derived:"seed master fp <seed fp>", expected:"md1 cosigner @<i> engraved fingerprint <card fp>", slot:Some(i)}`. Without `--allow-mismatch`: stderr `✗ MISMATCH`, then `error: restore: cosigner-fingerprint mismatch at slot @<i> — derived seed master fp <seed fp>, expected md1 cosigner @<i> engraved fingerprint <card fp>`; nothing on stdout; exit 4. With `--allow-mismatch`: status `overridden`, the own slot is labelled `from md1 (not independently verified)`, and the existing overridden banner prints; exit 0. Spec: none (`SPEC_restore_multisig_cosigner.md` line 27: "position inferred by fingerprint"; line 28: a cross-check mismatch is `RestoreMismatch`). Manual: 5.8.

Fixture `c42_md1.txt`: the md1 of `wsh(sortedmulti(2,[deadbeef/87'/0'/0']<$SA's m/87'/0'/0' xpub>/<0;1>/*,[28645006/87'/0'/0']<$SB's>/<0;1>/*))` — `$SA`'s real key under a wrong fingerprint.

~~~text
$ mnemonic restore --network mainnet $(sed 's/^/--md1 /' c42_md1.txt) --from phrase=@env:SA
-- current
exit=0
stdout: 2-of-2 multisig restore
stdout: CONFIRM: verify each cosigner fingerprint against your records before importing.
stdout:   descriptor: wsh(sortedmulti(2,[deadbeef/…
stdout:   first recv: bc1qhh7pna9m0lrs3lu4hgc8mcq4juar8srcfaqe5d7dx5rydzefqrnsrdar28
stdout:   cosigner @0: deadbeef [87'/0'/0']  ← your seed (verified)
stdout:   cosigner @1: 28645006 [87'/0'/0']  from md1 (not independently verified)
stderr: PARTIAL: cross-checked 1/2 cosigners; positions @1 were NOT independently verified — confirm their fingerprints against your records before importing
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=4
stdout: (empty)
stderr: ✗ MISMATCH
stderr: error: restore: cosigner-fingerprint mismatch at slot @0 — derived seed master fp b8688df1, expected md1 cosigner @0 engraved fingerprint deadbeef
~~~

~~~text
$ mnemonic restore --network mainnet $(sed 's/^/--md1 /' c42_md1.txt) --from phrase=@env:SA --allow-mismatch
-- after the fix
exit=0
stdout: 2-of-2 multisig restore
stdout: CONFIRM: verify each cosigner fingerprint against your records before importing.
stdout:   descriptor: wsh(sortedmulti(2,[deadbeef/…
stdout:   first recv: bc1qhh7pna9m0lrs3lu4hgc8mcq4juar8srcfaqe5d7dx5rydzefqrnsrdar28
stdout:   cosigner @0: deadbeef [87'/0'/0']  from md1 (not independently verified)
stdout:   cosigner @1: 28645006 [87'/0'/0']  from md1 (not independently verified)
stderr: ✗ MISMATCH (overridden): a supplied cross-check key does NOT match the md1 wallet; the descriptor above is the md1's wallet, NOT what your --from/--cosigner asserted
stderr: note: stdout is watch-only — public keys only, cannot spend
~~~

### 2.7 #53 (I) `restore` refuses a taproot leaf that repeats a key index

**Behaviour.** In `classify_taproot_restore`, before the tag dispatch: a `multi_a` / `sortedmulti_a` leaf whose key indices are not distinct is refused: `ModeViolation{mode:"restore", flag:"--md1"}`, exit 2, nothing on stdout, text: `taproot md1 repeats a key index inside its multi_a/sortedmulti_a leaf; the toolkit cannot reconstruct it faithfully`. Spec: none. Manual: 5.12.

Fixture `c53_md1.txt`: the 4 chunks of `tr(NUMS, multi_a(2, @0, @1, @1))`, built through md-codec's public API (the RED row's own fixture; `bundle` refuses this shape).

~~~text
$ mnemonic restore --network mainnet $(sed 's/^/--md1 /' c53_md1.txt)
-- current
exit=0
stdout: 2-of-2 multisig restore
stdout: CONFIRM: verify each cosigner fingerprint against your records before importing.
stdout:   descriptor: tr(50929b74c1a04954b78b4b603…
stdout:   first recv: bc1p67j2gprdapqzex789ng7g9sd22y5qe9jk7988ulxlzjxxxh6nllq0j954c
stdout:   cosigner @0: 00bbccdd [48'/0'/0'/2']  from md1 (not independently verified)
stdout:   cosigner @1: 01bbccdd [48'/0'/0'/2']  from md1 (not independently verified)
stderr: UNVERIFIED: no --from/--cosigner cross-check supplied; verify each cosigner fingerprint above against your records before importing
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=2
stdout: (empty)
stderr: error: taproot md1 repeats a key index inside its multi_a/sortedmulti_a leaf; the toolkit cannot reconstruct it faithfully
~~~

### 2.8 #38 (C) `restore` of a `tr(NUMS, multi_a/sortedmulti_a)` card with a non-standard uniform use-site

**Behaviour.** In `run_multisig`, a taproot card that `classify_taproot_restore` maps to the template arm (`TaprootRestore::Template`) is routed to the faithful general arm when its baseline use-site is not `/<0;1>/*` (the template arm hard-codes `/<0;1>/*`). The faithful arm prints the card's own use-site and its addresses; the heading line becomes the general arm's `miniscript policy restore (<n> cosigners)`. A standard `/<0;1>/*` card is unchanged. No new text. Spec: none. Manual: none.

Fixtures: the md1 of `tr(NUMS,<op>(2,K0<us>,K1<us>,K2<us>))` with K0..K2 the `[fp/87'/0'/0']` keys of `$PA`, `$SA`, `$SB`. Golden first receive addresses from Bitcoin Core `deriveaddresses` on the same descriptor (receive branch, index 0): `multi_a /<2;3>/*` → `bc1prhzr704juej9qza2zdcdycs4x8umnlewumnzkkkf77ky5rqs5fks9zcjsd`; `multi_a /*` → `bc1plqgfat0ncukgfwtr5jdfsmwtwn5vrgpwjzpqn73ur3qnpktpwwhsye6qt7`; `sortedmulti_a /<2;3>/*` → `bc1pv963as6zz3nf2qk6xqhux4pgduqldc3tx88a6a2szglrgr92h5msgcq2my`; `multi_a /<0;1>/*` → `bc1p6fy2e76ele2vwrhdpsu6l9cu3ayc9h6me6wgkeu78qkc5r6rpzas8e5cak`.

~~~text
$ mnemonic restore --network mainnet $(sed 's/^/--md1 /' tr23_md1.txt)
-- current
exit=0
stdout: 2-of-3 multisig restore
stdout: CONFIRM: verify each cosigner fingerprint against your records before importing.
stdout:   descriptor: tr(50929b74c1a04954b78b4b603…
stdout:   first recv: bc1p6fy2e76ele2vwrhdpsu6l9cu3ayc9h6me6wgkeu78qkc5r6rpzas8e5cak
stdout:   cosigner @0: 73c5da0a [87'/0'/0']  from md1 (not independently verified)
stdout:   cosigner @1: b8688df1 [87'/0'/0']  from md1 (not independently verified)
stdout:   cosigner @2: 28645006 [87'/0'/0']  from md1 (not independently verified)
stderr: UNVERIFIED: no --from/--cosigner cross-check supplied; verify each cosigner fingerprint above against your records before importing
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=0
stdout: miniscript policy restore (3 cosigners)
stdout: CONFIRM: verify each cosigner fingerprint against your records before importing.
stdout:   descriptor: tr(50929b74c1a04954b78b4b603…
stdout:   first recv: bc1prhzr704juej9qza2zdcdycs4x8umnlewumnzkkkf77ky5rqs5fks9zcjsd
stdout:   cosigner @0: 73c5da0a [87'/0'/0']  from md1 (not independently verified)
stdout:   cosigner @1: b8688df1 [87'/0'/0']  from md1 (not independently verified)
stdout:   cosigner @2: 28645006 [87'/0'/0']  from md1 (not independently verified)
stderr: UNVERIFIED: no --from/--cosigner cross-check supplied; verify each cosigner fingerprint above against your records before importing
stderr: note: stdout is watch-only — public keys only, cannot spend
~~~

~~~text
$ mnemonic restore --network mainnet $(sed 's/^/--md1 /' tr_star_md1.txt)
-- current
exit=0
stdout: 2-of-3 multisig restore
stdout: CONFIRM: verify each cosigner fingerprint against your records before importing.
stdout:   descriptor: tr(50929b74c1a04954b78b4b603…
stdout:   first recv: bc1p6fy2e76ele2vwrhdpsu6l9cu3ayc9h6me6wgkeu78qkc5r6rpzas8e5cak
stdout:   cosigner @0: 73c5da0a [87'/0'/0']  from md1 (not independently verified)
stdout:   cosigner @1: b8688df1 [87'/0'/0']  from md1 (not independently verified)
stdout:   cosigner @2: 28645006 [87'/0'/0']  from md1 (not independently verified)
stderr: UNVERIFIED: no --from/--cosigner cross-check supplied; verify each cosigner fingerprint above against your records before importing
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=0
stdout: miniscript policy restore (3 cosigners)
stdout: CONFIRM: verify each cosigner fingerprint against your records before importing.
stdout:   descriptor: tr(50929b74c1a04954b78b4b603…
stdout:   first recv: bc1plqgfat0ncukgfwtr5jdfsmwtwn5vrgpwjzpqn73ur3qnpktpwwhsye6qt7
stdout:   cosigner @0: 73c5da0a [87'/0'/0']  from md1 (not independently verified)
stdout:   cosigner @1: b8688df1 [87'/0'/0']  from md1 (not independently verified)
stdout:   cosigner @2: 28645006 [87'/0'/0']  from md1 (not independently verified)
stderr: UNVERIFIED: no --from/--cosigner cross-check supplied; verify each cosigner fingerprint above against your records before importing
stderr: note: stdout is watch-only — public keys only, cannot spend
~~~

~~~text
$ mnemonic restore --network mainnet $(sed 's/^/--md1 /' trs23_md1.txt)
-- current
exit=0
stdout: 2-of-3 multisig restore
stdout: CONFIRM: verify each cosigner fingerprint against your records before importing.
stdout:   descriptor: tr(50929b74c1a04954b78b4b603…
stdout:   first recv: bc1p550zvnachy40z6hh8llka93mkm0c3635samp264ck6rfd0dcdc8s00n8c8
stdout:   cosigner @0: 73c5da0a [87'/0'/0']  from md1 (not independently verified)
stdout:   cosigner @1: b8688df1 [87'/0'/0']  from md1 (not independently verified)
stdout:   cosigner @2: 28645006 [87'/0'/0']  from md1 (not independently verified)
stderr: UNVERIFIED: no --from/--cosigner cross-check supplied; verify each cosigner fingerprint above against your records before importing
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=0
stdout: miniscript policy restore (3 cosigners)
stdout: CONFIRM: verify each cosigner fingerprint against your records before importing.
stdout:   descriptor: tr(50929b74c1a04954b78b4b603…
stdout:   first recv: bc1pv963as6zz3nf2qk6xqhux4pgduqldc3tx88a6a2szglrgr92h5msgcq2my
stdout:   cosigner @0: 73c5da0a [87'/0'/0']  from md1 (not independently verified)
stdout:   cosigner @1: b8688df1 [87'/0'/0']  from md1 (not independently verified)
stdout:   cosigner @2: 28645006 [87'/0'/0']  from md1 (not independently verified)
stderr: UNVERIFIED: no --from/--cosigner cross-check supplied; verify each cosigner fingerprint above against your records before importing
stderr: note: stdout is watch-only — public keys only, cannot spend
~~~

In 38a-38c the descriptor line (cut above) ends every key with `/<0;1>/*` today; after the fix it carries the card's `/<2;3>/*` or `/*`, and the first receive address equals the Core golden of 2.8 in each case.

Control (standard use-site, unchanged):

~~~text
$ mnemonic restore --network mainnet $(sed 's/^/--md1 /' tr01_md1.txt)
-- current
exit=0
stdout: 2-of-3 multisig restore
stdout: CONFIRM: verify each cosigner fingerprint against your records before importing.
stdout:   descriptor: tr(50929b74c1a04954b78b4b603…
stdout:   first recv: bc1p6fy2e76ele2vwrhdpsu6l9cu3ayc9h6me6wgkeu78qkc5r6rpzas8e5cak
stdout:   cosigner @0: 73c5da0a [87'/0'/0']  from md1 (not independently verified)
stdout:   cosigner @1: b8688df1 [87'/0'/0']  from md1 (not independently verified)
stdout:   cosigner @2: 28645006 [87'/0'/0']  from md1 (not independently verified)
stderr: UNVERIFIED: no --from/--cosigner cross-check supplied; verify each cosigner fingerprint above against your records before importing
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=0
stdout: 2-of-3 multisig restore
stdout: CONFIRM: verify each cosigner fingerprint against your records before importing.
stdout:   descriptor: tr(50929b74c1a04954b78b4b603…
stdout:   first recv: bc1p6fy2e76ele2vwrhdpsu6l9cu3ayc9h6me6wgkeu78qkc5r6rpzas8e5cak
stdout:   cosigner @0: 73c5da0a [87'/0'/0']  from md1 (not independently verified)
stdout:   cosigner @1: b8688df1 [87'/0'/0']  from md1 (not independently verified)
stdout:   cosigner @2: 28645006 [87'/0'/0']  from md1 (not independently verified)
stderr: UNVERIFIED: no --from/--cosigner cross-check supplied; verify each cosigner fingerprint above against your records before importing
stderr: note: stdout is watch-only — public keys only, cannot spend
~~~

### 2.9 11b (C) and #11 (I): a single-sig template exists only at `/<0;1>/*`

**Behaviour — bundle.** `template_admissible` (`synthesize.rs`), `n == 1` branch: return false unless `descriptor.use_site_path == UseSitePath::standard_multipath()` and `descriptor.tlv.use_site_path_overrides.is_none()`; then the existing `cli_template_from_tree` check. This refuses bare `@0` (encoded as `/*`), `/*`, `/<2;3>/*` and `/*h` (#11). The refusal is the existing `TemplateFormUnsupportedShape`, exit 2, nothing on stdout, with its message extended to: `--md1-form=template cannot template this descriptor shape: it either does not render (e.g. tr(sortedmulti_a) — the rust-miniscript v13 SortedMultiA gap — or sortedmulti inside a combinator), or it uses a hardened use-site path (/*h or a hardened multipath alt) which an xpub cannot derive, or it is single-sig with a use-site other than /<0;1>/* (a single-sig template completes only at /<0;1>/*); use --md1-form=policy for a faithful keyed backup`

**Behaviour — restore.** First check in `run_singlesig_template_completion`, right after the template type is derived: a single-sig template md1 whose use-site is not `/<0;1>/*` or which carries use-site overrides is refused before any seed is read: `ModeViolation{mode:"restore", flag:"--md1"}`, exit 2, nothing on stdout, text: `this keyless single-sig template md1 has a use-site path other than /<0;1>/*, which single-sig template completion cannot rebuild (it would print a different wallet); use the wallet's full descriptor or a --md1-form=policy card instead`. verify-bundle needs no guard: its single-sig template path re-synthesizes the expected `/<0;1>/*` template, so another card fails `md1_template_match` already.

Spec: 4.1 (a). Manual: 5.4.

~~~text
$ mnemonic bundle --network mainnet --descriptor 'wpkh(@0/<0;1>/*h)' --md1-form template --slot @0.xpub=xpub6FQya7zGhR92kacY… --slot @0.fingerprint=704c7836 --json
-- current
exit=0
stdout: {"schema_version":"4","mode"…
stderr: advisory: restore --md1 cannot reconstruct this descriptor — it uses a hardened use-site path (`/*h` wildcard or a hardened multipath alternative, baseline or per-cosigner), from which watch-only addresses cannot be derived and which would silently render an unhardened path. The engraved card is a faithful backup; keep the full descriptor to restore. Tracked: restore-md1-per-key-use-site-and-hardened-wildcard
stderr: notice: this is a SHAREABLE TEMPLATE — the engraved md1 does not identify YOUR wallet.
stderr:         record this wallet-id with the engraving (as many bytes as you wish):
stderr:         wallet-id (hex):    7a8f0171bdfb276b9ebd7f48a491a5bd
stderr:         wallet-id (prefix): 7a8f0171  (4-byte convenience prefix)
stderr:         wallet-id (words):  kind job fragile know raven remind kit quiz emerge cause harvest knock
stderr:         restore with: restore --md1 <template> --from <seed> --account 0 [--expect-wallet-id 7a8f0171]
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=2
stdout: (empty)
stderr: error: --md1-form=template cannot template this descriptor shape: it either does not render (e.g. tr(sortedmulti_a) — the rust-miniscript v13 SortedMultiA gap — or sortedmulti inside a combinator), or it uses a hardened use-site path (/*h or a hardened multipath alt) which an xpub cannot derive, or it is single-sig with a use-site other than /<0;1>/* (a single-sig template completes only at /<0;1>/*); use --md1-form=policy for a faithful keyed backup
~~~

~~~text
$ printf '%s\n' "$PA" | mnemonic bundle --network mainnet --descriptor 'wpkh(@0)' --md1-form template --slot @0.phrase=- --json
-- current
exit=0
stdout: {"schema_version":"4","mode"…
stderr: notice: this is a SHAREABLE TEMPLATE — the engraved md1 does not identify YOUR wallet.
stderr:         record this wallet-id with the engraving (as many bytes as you wish):
stderr:         wallet-id (hex):    7774c4fde477f44b7f4aceb6d4aec730
stderr:         wallet-id (prefix): 7774c4fd  (4-byte convenience prefix)
stderr:         wallet-id (words):  jazz plate disorder similar leg certain wood flush replace pistol range gentle
stderr:         restore with: restore --md1 <template> --from <seed> --account 0 [--expect-wallet-id 7774c4fd]
stderr: warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
-- after the fix
exit=2
stdout: (empty)
stderr: error: --md1-form=template cannot template this descriptor shape: it either does not render (e.g. tr(sortedmulti_a) — the rust-miniscript v13 SortedMultiA gap — or sortedmulti inside a combinator), or it uses a hardened use-site path (/*h or a hardened multipath alt) which an xpub cannot derive, or it is single-sig with a use-site other than /<0;1>/* (a single-sig template completes only at /<0;1>/*); use --md1-form=policy for a faithful keyed backup
~~~

(`wpkh(@0)` and `wpkh(@0/*)` mint the same card, `md1fu85lqqpqqqqqnm3nr4htwmp8w`, `template: wpkh(@0/*)`; 11b2 has the same before/after as 11b1.)

~~~text
$ printf '%s\n' "$PA" | mnemonic bundle --network mainnet --descriptor "wpkh([73c5da0a/84'/0'/0']@0/<2;3>/*)" --md1-form template --slot @0.phrase=- --json
-- current
exit=0
stdout: {"schema_version":"4","mode"…
stderr: notice: this is a SHAREABLE TEMPLATE — the engraved md1 does not identify YOUR wallet.
stderr:         record this wallet-id with the engraving (as many bytes as you wish):
stderr:         wallet-id (hex):    1c0170fe82855f60eeca91a9899b0abe
stderr:         wallet-id (prefix): 1c0170fe  (4-byte convenience prefix)
stderr:         wallet-id (words):  bright argue distance agree field race robot fall praise error section lady
stderr:         restore with: restore --md1 <template> --from <seed> --account 0 [--expect-wallet-id 1c0170fe]
stderr: warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
-- after the fix
exit=2
stdout: (empty)
stderr: error: --md1-form=template cannot template this descriptor shape: it either does not render (e.g. tr(sortedmulti_a) — the rust-miniscript v13 SortedMultiA gap — or sortedmulti inside a combinator), or it uses a hardened use-site path (/*h or a hardened multipath alt) which an xpub cannot derive, or it is single-sig with a use-site other than /<0;1>/* (a single-sig template completes only at /<0;1>/*); use --md1-form=policy for a faithful keyed backup
~~~

Control (standard use-site, unchanged):

~~~text
$ printf '%s\n' "$PA" | mnemonic bundle --network mainnet --descriptor "wpkh([73c5da0a/84'/0'/0']@0/<0;1>/*)" --md1-form template --slot @0.phrase=- --json
-- current
exit=0
stdout: {"schema_version":"4","mode"…
stderr: notice: this is a SHAREABLE TEMPLATE — the engraved md1 does not identify YOUR wallet.
stderr:         record this wallet-id with the engraving (as many bytes as you wish):
stderr:         wallet-id (hex):    1c0170fe82855f60eeca91a9899b0abe
stderr:         wallet-id (prefix): 1c0170fe  (4-byte convenience prefix)
stderr:         wallet-id (words):  bright argue distance agree field race robot fall praise error section lady
stderr:         restore with: restore --md1 <template> --from <seed> --account 0 [--expect-wallet-id 1c0170fe]
stderr: warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
-- after the fix
exit=0
stdout: {"schema_version":"4","mode"…
stderr: notice: this is a SHAREABLE TEMPLATE — the engraved md1 does not identify YOUR wallet.
stderr:         record this wallet-id with the engraving (as many bytes as you wish):
stderr:         wallet-id (hex):    1c0170fe82855f60eeca91a9899b0abe
stderr:         wallet-id (prefix): 1c0170fe  (4-byte convenience prefix)
stderr:         wallet-id (words):  bright argue distance agree field race robot fall praise error section lady
stderr:         restore with: restore --md1 <template> --from <seed> --account 0 [--expect-wallet-id 1c0170fe]
stderr: warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
~~~

restore of the two cards the current bundle mints (`md1fkykhqqpqqgzskqqzxcjnwq6cq0gv` = `wpkh(@0/<2;3>/*)`, `md1fu85lqqpqqqqqnm3nr4htwmp8w` = `wpkh(@0/*)`):

~~~text
$ printf '%s\n' "$PA" | mnemonic restore --network mainnet --md1 md1fkykhqqpqqgzskqqzxcjnwq6cq0gv --from phrase=- --account 0
-- current
exit=0
stdout: master fingerprint: 73c5da0a  (passphrase: none)
stdout: CONFIRM: this fingerprint matches the wallet you are restoring before importing.
stdout: 
stdout: bip84 (native segwit P2WPKH):
stdout:   descriptor: wpkh([73c5da0a/84'/0'/0']xpu…
stdout:   first recv: bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
stderr: UNVERIFIED: no --expect-wallet-id supplied; verify the master fingerprint above (73c5da0a) against your records before importing
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=2
stdout: (empty)
stderr: error: this keyless single-sig template md1 has a use-site path other than /<0;1>/*, which single-sig template completion cannot rebuild (it would print a different wallet); use the wallet's full descriptor or a --md1-form=policy card instead
~~~

~~~text
$ printf '%s\n' "$PA" | mnemonic restore --network mainnet --md1 md1fkykhqqpqqgzskqqzxcjnwq6cq0gv --from phrase=- --account 0 --expect-wallet-id 1c0170fe
-- current
exit=0
stdout: master fingerprint: 73c5da0a  (passphrase: none)
stdout: CONFIRM: this fingerprint matches the wallet you are restoring before importing.
stdout: 
stdout: bip84 (native segwit P2WPKH):
stdout:   descriptor: wpkh([73c5da0a/84'/0'/0']xpu…
stdout:   first recv: bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
stderr: ✓ wallet-id verified: completed wallet matches --expect-wallet-id
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=2
stdout: (empty)
stderr: error: this keyless single-sig template md1 has a use-site path other than /<0;1>/*, which single-sig template completion cannot rebuild (it would print a different wallet); use the wallet's full descriptor or a --md1-form=policy card instead
~~~

~~~text
$ printf '%s\n' "$PA" | mnemonic restore --network mainnet --md1 md1fu85lqqpqqqqqnm3nr4htwmp8w --from phrase=- --account 0
-- current
exit=0
stdout: master fingerprint: 73c5da0a  (passphrase: none)
stdout: CONFIRM: this fingerprint matches the wallet you are restoring before importing.
stdout: 
stdout: bip84 (native segwit P2WPKH):
stdout:   descriptor: wpkh([73c5da0a/84'/0'/0']xpu…
stdout:   first recv: bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
stderr: UNVERIFIED: no --expect-wallet-id supplied; verify the master fingerprint above (73c5da0a) against your records before importing
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=2
stdout: (empty)
stderr: error: this keyless single-sig template md1 has a use-site path other than /<0;1>/*, which single-sig template completion cannot rebuild (it would print a different wallet); use the wallet's full descriptor or a --md1-form=policy card instead
~~~

Control (the standard template card, unchanged):

~~~text
$ printf '%s\n' "$PA" | mnemonic restore --network mainnet --md1 md1fvwrwqqpqqgqpsqqq3uaau4ctxyl7 --from phrase=- --account 0
-- current
exit=0
stdout: master fingerprint: 73c5da0a  (passphrase: none)
stdout: CONFIRM: this fingerprint matches the wallet you are restoring before importing.
stdout: 
stdout: bip84 (native segwit P2WPKH):
stdout:   descriptor: wpkh([73c5da0a/84'/0'/0']xpu…
stdout:   first recv: bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
stderr: UNVERIFIED: no --expect-wallet-id supplied; verify the master fingerprint above (73c5da0a) against your records before importing
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=0
stdout: master fingerprint: 73c5da0a  (passphrase: none)
stdout: CONFIRM: this fingerprint matches the wallet you are restoring before importing.
stdout: 
stdout: bip84 (native segwit P2WPKH):
stdout:   descriptor: wpkh([73c5da0a/84'/0'/0']xpu…
stdout:   first recv: bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
stderr: UNVERIFIED: no --expect-wallet-id supplied; verify the master fingerprint above (73c5da0a) against your records before importing
stderr: note: stdout is watch-only — public keys only, cannot spend
~~~

The bundle refusal's remedy, `--md1-form=policy`, run as printed: it mints a keyed card whose restore prints the `/<2;3>/*` wallet; Core gives the same first address (`wpkh([73c5da0a/84h/0h/0h]xpub6CatWdiZiodmUeTD…/2/*)` index 0 → `bc1qgypllrw8plmfj3tutsff7c0p9j9adukc4qagn6`). `c11p_md1.txt` holds the three md1 chunks the first command prints.

~~~text
$ printf '%s\n' "$PA" | mnemonic bundle --network mainnet --descriptor "wpkh([73c5da0a/84'/0'/0']@0/<2;3>/*)" --md1-form policy --slot @0.phrase=- --json
-- current (unchanged after the fix)
exit=0
stdout: {"schema_version":"4","mode"…
stderr: warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
~~~

~~~text
$ mnemonic restore --network mainnet $(sed 's/^/--md1 /' c11p_md1.txt)
-- current (unchanged after the fix)
exit=0
stdout: miniscript policy restore (1 cosigner)
stdout: CONFIRM: verify each cosigner fingerprint against your records before importing.
stdout:   descriptor: wpkh([73c5da0a/84'/0'/0']xpu…
stdout:   first recv: bc1qgypllrw8plmfj3tutsff7c0p9j9adukc4qagn6
stdout:   cosigner @0: 73c5da0a [84'/0'/0']  from md1 (not independently verified)
stderr: UNVERIFIED: no --from/--cosigner cross-check supplied; verify each cosigner fingerprint above against your records before importing
stderr: note: stdout is watch-only — public keys only, cannot spend
~~~

### 2.10 #37 (C) `verify-bundle` of a single-sig template bundle checks the `--ms1` card

**Behaviour.** In `verify_singlesig_template`, when the first `--ms1` value is non-empty, two rows are pushed first, before `md1_template_match`: `ms1_decode` and `ms1_entropy_match`, built by the keyed single-sig logic (decode; byte compare; on a decode error, unless `--no-auto-repair`, the auto-repair ground-truth compare). That logic moves unchanged from `emit_checks` (`verify_bundle.rs` ≈2348-2464) into one function both call. The expected card is the canonical seed slot's (`resolved`, which keeps the entropy under `--origin`), never `resolved_for_synth`. Compute it as `ms1[0]` of a second `crate::synthesize::synthesize_unified(&resolved, template, 1, args.network, args.privacy_preserving, args.language.unwrap_or_default().into(), crate::synthesize::Md1Form::Template)` call. Use the same arguments as the existing `expected` call, with `resolved` in place of `resolved_for_synth`, and never encode the entropy directly. The card carries the wordlist language (`Mnem` for a non-English seed), and the compare is by string. With no `--ms1` there are no ms1 rows (unchanged). Row texts are the existing ones: `decoded successfully`, `ms1 byte-identical`, `expected ms1 bytes differ from supplied`, `recovered via auto-repair, confirmed against expected seed`, `recovered via auto-repair (unconfirmed)`, `auto-repair candidate did not match the expected seed — this card is not a card for this seed`, `ms1 decode failed; entropy match cannot run`. A failed row gives `mismatch`, exit 4. Spec: 4.1 (d). Manual: 5.3.

Fixtures: `st_ms1.txt`, `st_mk1.txt`, `st_md1.txt` — the `# ms1`, `# mk1`, `# md1` sections of `bundle --template bip84 --md1-form template --group-size 0` for `$PA`; `sa_ms1.txt` — the `# ms1` section of the same for `$SA` (another seed).

~~~text
$ cat st_ms1.txt | mnemonic verify-bundle --network mainnet --slot @0.phrase=@env:PA --ms1 - $(sed 's/^/--mk1 /' st_mk1.txt) --md1 $(cat st_md1.txt)
-- current
exit=0
stdout: OK (single-sig template recomposed)
stdout: wallet type: bip84
stdout: descriptor:  wpkh([73c5da0a/84'/0'/0']xpu…
stdout: first recv:  bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
stderr: ✓ md1_template_match: supplied md1 matches the expected keyless single-sig template
stderr: ✓ mk1_template_stub_bind: supplied mk1 binds via the template-id stub
-- after the fix
exit=0
stdout: OK (single-sig template recomposed)
stdout: wallet type: bip84
stdout: descriptor:  wpkh([73c5da0a/84'/0'/0']xpu…
stdout: first recv:  bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
stderr: ✓ ms1_decode: decoded successfully
stderr: ✓ ms1_entropy_match: ms1 byte-identical
stderr: ✓ md1_template_match: supplied md1 matches the expected keyless single-sig template
stderr: ✓ mk1_template_stub_bind: supplied mk1 binds via the template-id stub
~~~

~~~text
$ cat sa_ms1.txt | mnemonic verify-bundle --network mainnet --slot @0.phrase=@env:PA --ms1 - $(sed 's/^/--mk1 /' st_mk1.txt) --md1 $(cat st_md1.txt)
-- current
exit=0
stdout: OK (single-sig template recomposed)
stdout: wallet type: bip84
stdout: descriptor:  wpkh([73c5da0a/84'/0'/0']xpu…
stdout: first recv:  bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
stderr: ✓ md1_template_match: supplied md1 matches the expected keyless single-sig template
stderr: ✓ mk1_template_stub_bind: supplied mk1 binds via the template-id stub
-- after the fix
exit=4
stdout: mismatch
stderr: ✓ ms1_decode: decoded successfully
stderr: ✗ ms1_entropy_match: expected ms1 bytes differ from supplied
stderr: ✓ md1_template_match: supplied md1 matches the expected keyless single-sig template
stderr: ✓ mk1_template_stub_bind: supplied mk1 binds via the template-id stub
~~~

`--json`, after the fix (`cat sa_ms1.txt | mnemonic verify-bundle --network mainnet --slot @0.phrase=@env:PA --ms1 - --json $(sed 's/^/--mk1 /' st_mk1.txt) --md1 $(cat st_md1.txt)`): exit 4, `"result":"mismatch"`, `"mode":"single-sig-template"`, and `checks` is exactly

~~~json
[{"detail":"decoded successfully","name":"ms1_decode","passed":true},{"detail":"expected ms1 bytes differ from supplied","name":"ms1_entropy_match","passed":false},{"detail":"supplied md1 matches the expected keyless single-sig template","name":"md1_template_match","passed":true},{"detail":"supplied mk1 binds via the template-id stub","name":"mk1_template_stub_bind","passed":true}]
~~~

Today the same run exits 0 with `"result":"ok"` and only the last two rows. Under `--origin`, the existing tests `cli_verify_bundle_md1_template::verify_template_bundle_expect_wallet_id_skipped_under_origin` and `::verify_template_bundle_recomposes_under_origin_matches_restore` supply the correct ms1 and stay green (the triage's fix-37 broke them).

### 2.11 #39 (I) `verify-bundle` single-sig `md1_xpub_match` compares the policy as well as the key

**Behaviour.** In `emit_md1_checks`, wallet-policy branch: `policy_match = expected.tree == supplied.tree && expected.use_site_path == supplied.use_site_path && expected.tlv.use_site_path_overrides == supplied.tlv.use_site_path_overrides`. The row passes only if the first-key compare and `policy_match` both hold (detail unchanged, `65-byte xpub matches expected`). If `policy_match` fails: `passed:false`, detail `md1 reconstructs a DIFFERENT wallet — structural policy mismatch: <classes>`, where `<classes>` joins with `, ` the applicable strings `policy tree (threshold/script-type/order)`, `use-site path (change-chain/multipath)`, `per-cosigner use-site override` (the multisig branch's strings), `expected`/`actual` the Debug of the two trees, `diff_byte_offset` null. Otherwise (policy equal, key different) the existing `md1 xpub differs from expected` row. The name stays `md1_xpub_match`; exit 4 on failure. Spec: none (`SPEC_mnemonic_toolkit_v0_5.md` lists the row names only). Manual: none.

Fixtures: `k84_*.txt` — the keyed `bundle --template bip84` cards for `$PA`; `alt23_md1.txt` / `altpkh_md1.txt` — the md1 of `wpkh([73c5da0a/84'/0'/0']<$PA's account xpub>/<2;3>/*)` and `pkh([73c5da0a/84'/0'/0']<same>/<0;1>/*)`: the same key in another wallet.

`mnemonic verify-bundle --template bip84 --network mainnet --slot @0.phrase=@env:PA --json --ms1 $(cat k84_ms1.txt) $(sed 's/^/--mk1 /' k84_mk1.txt) $(sed 's/^/--md1 /' <alt md1 file>)`:

| `--md1` | today | after the fix |
|---|---|---|
| `alt23_md1.txt` | exit 0, `"result":"ok"`, `md1_xpub_match` `{"passed":true,"detail":"65-byte xpub matches expected"}` | exit 4, `"result":"mismatch"`, `md1_xpub_match` `{"passed":false,"detail":"md1 reconstructs a DIFFERENT wallet — structural policy mismatch: use-site path (change-chain/multipath)","expected":"Node { tag: Wpkh, body: KeyArg { index: 0 } }","actual":"Node { tag: Wpkh, body: KeyArg { index: 0 } }"}` |
| `altpkh_md1.txt` | exit 0, `"result":"ok"` | exit 4, `"result":"mismatch"`, `md1_xpub_match` `{"passed":false,"detail":"md1 reconstructs a DIFFERENT wallet — structural policy mismatch: policy tree (threshold/script-type/order)","expected":"Node { tag: Wpkh, body: KeyArg { index: 0 } }","actual":"Node { tag: Pkh, body: KeyArg { index: 0 } }"}` |
| `k84_md1.txt` (control) | exit 0, `ok` | exit 0, `ok` (unchanged) |

### 2.12 #44 (I) descriptor-mode `verify-bundle` uses the annotation fingerprint, as `bundle` does

**Behaviour.** In `descriptor_mode_verify_run`, xpub arm, mirroring `bundle.rs` H7 (≈1615-1636): parse `--slot @<idx>.fingerprint=` strictly (`BadInput`, exit 1, `--slot @<idx>.fingerprint parse: <e>`); if both it and the descriptor's `@<idx>` annotation fingerprint are present and differ, refuse (`DescriptorParse`, exit 2, `--slot @<idx>.fingerprint specifies <slot> but descriptor @<idx> annotation specifies <anno>`, bundle's text verbatim); then `fingerprint = slot.or(annotation).unwrap_or_default()`. Both refusals print nothing on stdout. Spec: none. Manual: none (matches `bundle`).

Fixtures: `d44_desc.txt` = `wsh(sortedmulti(2,@0[b8688df1/48'/0'/0'/2']/<0;1>/*,@1[28645006/48'/0'/0'/2']/<0;1>/*))`; `xb44.txt` — `$SB`'s `m/48'/0'/0'/2'` xpub; `d44.json` — the `bundle --descriptor "$(cat d44_desc.txt)" --network mainnet --json --slot @0.phrase=@env:SA --slot @1.xpub=$(cat xb44.txt)` output (exit 0 on both binaries).

| command (`mnemonic --no-auto-repair verify-bundle --descriptor "$(cat d44_desc.txt)" --network mainnet --json --slot @0.phrase=@env:SA --slot @1.xpub=$(cat xb44.txt) <extra> --ms1 $(jq -r '.ms1[0]' d44.json) --ms1 '' $(jq -r '.mk1[][] \| "--mk1 " + .' d44.json) $(jq -r '.md1[] \| "--md1 " + .' d44.json)`) | today | after the fix |
|---|---|---|
| no `<extra>` | exit 4, `"result":"mismatch"`, the one failed row `mk1_fingerprint_match[1]` (`cosigner[1] fingerprint mismatch`) | exit 0, `"result":"ok"` |
| `<extra>` = `--slot @1.fingerprint=deadbeef` | exit 4, `mismatch` | exit 2, stdout empty, stderr ``notice: cosigner[1] marked watch-only via empty `--ms1` sentinel (SPEC §5.8); no seed will be derived for this slot`` then `error: --slot @1.fingerprint specifies deadbeef but descriptor @1 annotation specifies 28645006` |
| `<extra>` = `--slot @1.fingerprint=zzzz` | exit 4, `mismatch` | exit 1, stdout empty, stderr the same notice then `error: --slot @1.fingerprint parse: failed to parse hex` |

### 2.13 #47 (I) `verify-bundle --expect-wallet-id` is validated as on `restore`

**Behaviour.** In `verify_singlesig_template`, the prefix is decoded by restore's `decode_wallet_id_prefix` (made `pub(crate)`), in the same place (not under `--origin`, which still prints the existing notice and checks nothing). Refusals, all `BadInput`, exit 1, nothing on stdout: `--expect-wallet-id must not be empty`; `--expect-wallet-id must be an even-length hex prefix of the WalletPolicyId: <hex error>`; `--expect-wallet-id prefix is <n> bytes; the WalletPolicyId is only 16 bytes — supply a prefix of at most 16 bytes (32 hex chars)`. The `advisory: --expect-wallet-id prefix is only <n> byte(s); ≥4 recommended.` line stays for 1-3 bytes. Spec: 4.1 (c). Manual: 5.6.

~~~text
$ mnemonic verify-bundle --network mainnet --slot @0.phrase=@env:PA --expect-wallet-id '' $(sed 's/^/--mk1 /' st_mk1.txt) --md1 $(cat st_md1.txt)
-- current
exit=0
stdout: OK (single-sig template recomposed)
stdout: wallet type: bip84
stdout: descriptor:  wpkh([73c5da0a/84'/0'/0']xpu…
stdout: first recv:  bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
stderr: advisory: --expect-wallet-id prefix is only 0 byte(s); ≥4 recommended.
stderr: ✓ md1_template_match: supplied md1 matches the expected keyless single-sig template
stderr: ✓ mk1_template_stub_bind: supplied mk1 binds via the template-id stub
stderr: ✓ wallet_id_match: completed wallet matches --expect-wallet-id
-- after the fix
exit=1
stdout: (empty)
stderr: error: --expect-wallet-id must not be empty
~~~

~~~text
$ mnemonic verify-bundle --network mainnet --slot @0.phrase=@env:PA --expect-wallet-id 1c0170fe82855f60eeca91a9899b0abe00 $(sed 's/^/--mk1 /' st_mk1.txt) --md1 $(cat st_md1.txt)
-- current
exit=4
stdout: mismatch
stderr: ✓ md1_template_match: supplied md1 matches the expected keyless single-sig template
stderr: ✓ mk1_template_stub_bind: supplied mk1 binds via the template-id stub
stderr: ✗ wallet_id_match: completed wallet does NOT match --expect-wallet-id
-- after the fix
exit=1
stdout: (empty)
stderr: error: --expect-wallet-id prefix is 17 bytes; the WalletPolicyId is only 16 bytes — supply a prefix of at most 16 bytes (32 hex chars)
~~~

~~~text
$ mnemonic verify-bundle --network mainnet --slot @0.phrase=@env:PA --expect-wallet-id 1c0 $(sed 's/^/--mk1 /' st_mk1.txt) --md1 $(cat st_md1.txt)
-- current
exit=1
stdout: (empty)
stderr: error: --expect-wallet-id hex: Odd number of digits
-- after the fix
exit=1
stdout: (empty)
stderr: error: --expect-wallet-id must be an even-length hex prefix of the WalletPolicyId: Odd number of digits
~~~

Control (a 4-byte prefix of the true id, unchanged):

~~~text
$ mnemonic verify-bundle --network mainnet --slot @0.phrase=@env:PA --expect-wallet-id 1c0170fe $(sed 's/^/--mk1 /' st_mk1.txt) --md1 $(cat st_md1.txt)
-- current
exit=0
stdout: OK (single-sig template recomposed)
stdout: wallet type: bip84
stdout: descriptor:  wpkh([73c5da0a/84'/0'/0']xpu…
stdout: first recv:  bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
stderr: ✓ md1_template_match: supplied md1 matches the expected keyless single-sig template
stderr: ✓ mk1_template_stub_bind: supplied mk1 binds via the template-id stub
stderr: ✓ wallet_id_match: completed wallet matches --expect-wallet-id
-- after the fix
exit=0
stdout: OK (single-sig template recomposed)
stdout: wallet type: bip84
stdout: descriptor:  wpkh([73c5da0a/84'/0'/0']xpu…
stdout: first recv:  bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
stderr: ✓ md1_template_match: supplied md1 matches the expected keyless single-sig template
stderr: ✓ mk1_template_stub_bind: supplied mk1 binds via the template-id stub
stderr: ✓ wallet_id_match: completed wallet matches --expect-wallet-id
~~~

### 2.14 #52 (I) `mk1_template_stub_bind` fails on an `--mk1` value with no readable chunk_set_id

**Behaviour.** In `check_mk1_template_stubs` (multisig-template verify): if any `--mk1` value yields no chunk_set_id, the row is `passed:false` with detail `supplied mk1 stub(s) do NOT bind via the template-id stub: an --mk1 value has no readable chunk_set_id (damaged card or not a template stub)`; the full check table is still emitted; `mismatch`, exit 4. A readable set that does not bind keeps the existing detail `supplied mk1 stub(s) do NOT bind via the template-id stub (card mismatch or policy/template cross-mix)`. No intake refusal. Spec: none. Manual: none.

Fixture: `tsm2.json` / `psm2.json` (2.3). Command: `mnemonic verify-bundle --network mainnet $(jq -r '.md1[] | "--md1 " + .' tsm2.json) $(jq -r '.mk1[][] | "--mk1 " + .' tsm2.json) <extra> --from phrase=@env:SA --account 0 --expect-wallet-id $(cat tsm2_id.txt) --json $(jq -r '.mk1[1][] | "--cosigner " + .' psm2.json)`.

| `<extra>` | today | after the fix |
|---|---|---|
| none | exit 0, `ok`, row `{"passed":true,"detail":"supplied mk1 template stub(s) bind via the template-id stub"}` | unchanged |
| `--mk1 mk1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq` | exit 0, `ok`, the same passing row | exit 4, `mismatch`, row `{"passed":false,"detail":"supplied mk1 stub(s) do NOT bind via the template-id stub: an --mk1 value has no readable chunk_set_id (damaged card or not a template stub)"}` |

### 2.15 #12, #13, #43 (I) `verify-bundle` multisig card mapping

No text changes; check rows change. Fixtures: `full3.json` — `bundle --template wsh-sortedmulti --threshold 2 --multisig-path-family bip48 --json` over `$SA`,`$SB`,`$SC`; `stray<k>.txt` — the phrase of 16 bytes of `k` (`convert --from entropy=@env:E --to phrase`), `fullst<k>.json` — the same bundle over `stray<k>`,`$SB`,`$SC`. Base command: `mnemonic --no-auto-repair verify-bundle --network mainnet --template wsh-sortedmulti --threshold 2 --multisig-path-family bip48 --json --slot @0.phrase=@env:<P0> --slot @1.phrase=@env:<P1> [--slot @2.phrase=@env:<P2>] <ms1, mk1, md1 cards>`.

- **#12** (`emit_multisig_checks`, fix-13 then fix-12 verbatim): an mk1 card for a key not in the policy no longer consumes or blames other slots. Cards: `full3` slot 0, `fullst<k>` slot 0 in place of slot 1, `full3` slot 2. For k = 1 today: `mk1_decode[0]` `passed:false` (`cosigner[0] supplied mk1 card xpub absent from descriptor policy`), `[1]` false, `[2]` true. After: `[0]` true (`cosigner[0] mk1 decoded`), `[1]` false (`cosigner[1] supplied mk1 card xpub absent from descriptor policy`), `[2]` true; still exit 4 (slot 1 is wrong). Today 6 of k = 1..8 blame slot 0; after, none.
- **#13**: the positional fallback (md1 undecodable) keeps the supplied card order instead of sorting by chunk_set_id. Cards: all of `fullst<k>` with the last md1 chunk dropped. Today for k = 2..6, `mk1_xpub_match` / `mk1_fingerprint_match` fail at slots 0 and 2 (k = 5: also 1); after, every `mk1_*_match[i]` passes; still exit 4 (`md1_decode` fails).
- **#43** (fix-43 verbatim): a supplied md1 with more cosigners than the expected wallet no longer indexes out of bounds. Cards: all of `full3`; slots `$SA`,`$SB` only. Today: exit 101, stderr `thread 'main' panicked at crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:2762:54:` / `index out of bounds: the len is 2 but the index is 2`. After: exit 4, `"result":"mismatch"`, failed rows include `md1_xpub_match` (`md1 reconstructs a DIFFERENT wallet — structural policy mismatch: policy tree (threshold/script-type/order)`) and `md1_origin_match` (`expected 2 origins, card declares 3`); stderr empty.

Spec: none. Manual: none.

### 2.16 #3 (I) a multipath use-site with no trailing `/*` is refused

**Behaviour.** In `lex_placeholders` (`parse_descriptor.rs`), after the residue check: a placeholder whose `mpath` group matched and whose `wild` group did not is refused: `DescriptorParse`, exit 2, nothing on stdout, text ``@<i>: a multipath use-site `/<a;b>` with no trailing `/*` (a non-ranged multipath) is not representable in md1, which encodes only a ranged `/<a;b>/*` or bare `/*` use-site`` (the inner backticks and `/<a;b>` are literal). It reaches every literal-descriptor surface that lexes (`bundle --descriptor`, `verify-bundle --descriptor`, `import-wallet`, `bundle --import-json`); an inline key is lexed as `@N`, so the message names `@0` for it too. Bare `@N` (D1, deferred) and the fixed-step residue refusal are unchanged. The text gives no "add `/*`" remedy on purpose: that describes another wallet. Spec: none. Manual: 5.2.

~~~text
$ mnemonic bundle --network mainnet --descriptor 'wpkh(@0/<0;1>)' --slot @0.xpub=xpub6FQya7zGhR92kacY… --slot @0.fingerprint=704c7836 --json
-- current
exit=0
stdout: {"schema_version":"4","mode"…
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=2
stdout: (empty)
stderr: error: @0: a multipath use-site `/<a;b>` with no trailing `/*` (a non-ranged multipath) is not representable in md1, which encodes only a ranged `/<a;b>/*` or bare `/*` use-site
~~~

~~~text
$ mnemonic bundle --network mainnet --descriptor "wpkh([704c7836/84'/0'/0']xp… --json
-- current
exit=0
stdout: {"schema_version":"4","mode"…
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=2
stdout: (empty)
stderr: error: @0: a multipath use-site `/<a;b>` with no trailing `/*` (a non-ranged multipath) is not representable in md1, which encodes only a ranged `/<a;b>/*` or bare `/*` use-site
~~~

Unchanged (G2's #57 quotes this text, section 10), and bare `@0`:

~~~text
$ mnemonic bundle --network mainnet --descriptor 'wpkh(@0/0/*)' --slot @0.xpub=xpub6FQya7zGhR92kacY… --slot @0.fingerprint=704c7836 --json
-- current (unchanged after the fix)
exit=2
stdout: (empty)
stderr: error: @0: derivation steps after the placeholder are not representable in md1; the use-site path must be a multipath `/<a;b>/*` (or bare `/*`) as the final step — a fixed single step like `/0/*` is un-representable (found residue near `/0/*)`)
~~~

~~~text
$ mnemonic bundle --network mainnet --descriptor 'wpkh(@0)' --slot @0.xpub=xpub6FQya7zGhR92kacY… --slot @0.fingerprint=704c7836 --json
-- current
exit=0
stdout: {"schema_version":"4","mode"…
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=0
stdout: {"schema_version":"4","mode"…
stderr: note: stdout is watch-only — public keys only, cannot spend
~~~

### 2.17 #4 (I) Electrum passphrase normalization uses Electrum's whole CJK table

**Behaviour.** `is_cjk` in `electrum.rs` becomes Electrum's `CJK_INTERVALS` verbatim: the 29 intervals of `electrum/mnemonic.py` lines 40-70 at `spesmilo/electrum` `e1099925` (fetched 2026-11-01; byte-compared with fix-04's table, identical). Today it has 9 of them. Whitespace between two CJK characters is stripped after NFKD, so a space between two Hangul syllables (which NFKD splits into Jamo, U+1100-11FF, not in today's table) is now removed, as in Electrum. No text. Spec: 4.5. Manual: none.

~~~text
$ export PP='한 글'; printf '%s\n' 'bitter grass shiver impose acquire brush forget axis eager alone wine silver' | mnemonic addresses --from electrum-phrase=- --passphrase @env:PP --address-type p2wpkh --chain both --count 1
-- current
exit=0
stdout: receive (m/0/i):
stdout:   0  bc1qeszqkdyegvtcgtl9jwzy7p66szuuqzmr8zg23m
stdout: change (m/1/i):
stdout:   0  bc1qhckw57csutkges06pc8gxzr4zd7vnqnxf8jc87
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=0
stdout: receive (m/0/i):
stdout:   0  bc1qp8jxewpvuuunzdg9e6gytwhfccas32jdgpuqdn
stdout: change (m/1/i):
stdout:   0  bc1q22daph22zsmzcxsht5ay5aff7zeu7y97ac9g47
stderr: note: stdout is watch-only — public keys only, cannot spend
~~~

The same passphrase without the space (unchanged, the Electrum result):

~~~text
$ export PP='한글'; printf '%s\n' 'bitter grass shiver impose acquire brush forget axis eager alone wine silver' | mnemonic addresses --from electrum-phrase=- --passphrase @env:PP --address-type p2wpkh --chain both --count 1
-- current (unchanged after the fix)
exit=0
stdout: receive (m/0/i):
stdout:   0  bc1qp8jxewpvuuunzdg9e6gytwhfccas32jdgpuqdn
stdout: change (m/1/i):
stdout:   0  bc1q22daph22zsmzcxsht5ay5aff7zeu7y97ac9g47
stderr: note: stdout is watch-only — public keys only, cannot spend
~~~

### 2.18 #5 (I) `export-wallet --format electrum` refuses `--template bip86`

**Behaviour.** In the Electrum emitter, after the taproot-multisig refusal: a P2TR single-sig wallet is refused: `BadInput`, exit 1, nothing on stdout, text `--format electrum does not support --template bip86 (P2TR): a plain xpub "standard" wallet is read by Electrum as p2pkh. Use --format bitcoin-core (descriptor) or --format sparrow for taproot watch-only setup.` Spec: 4.4. Manual: 5.7.

~~~text
$ mnemonic export-wallet --format electrum --template bip86 --network mainnet --wallet-name T --slot @0.xpub=xpub6FQya7zGhR92kacY… --slot @0.fingerprint=704c7836 --output -
-- current
exit=0
stdout: {
stdout:   "seed_version": 17,
stdout:   "wallet_type": "standard",
stdout:   "use_encryption": false,
stdout:   "keystore": {
stdout:     "type": "bip32",
stdout:     "xpub": "xpub6FQya7zGhR92kacYsNnjreo…
stdout:     "derivation": "m/86'/0'/0'",
stdout:     "root_fingerprint": "704c7836",
stdout:     "label": "T"
stdout:   }
stdout: }
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=1
stdout: (empty)
stderr: error: --format electrum does not support --template bip86 (P2TR): a plain xpub "standard" wallet is read by Electrum as p2pkh. Use --format bitcoin-core (descriptor) or --format sparrow for taproot watch-only setup.
~~~

The remedy's two formats, run as printed (unchanged):

~~~text
$ mnemonic export-wallet --format bitcoin-core --template bip86 --network mainnet --wallet-name T --slot @0.xpub=xpub6FQya7zGhR92kacY… --slot @0.fingerprint=704c7836 --output -
-- current (unchanged after the fix)
exit=0
stdout: [
stdout:   {
stdout:     "active": true,
stdout: … (22 lines in all; the import payload)
stderr: note: stdout is watch-only — public keys only, cannot spend
~~~

~~~text
$ mnemonic export-wallet --format sparrow --template bip86 --network mainnet --wallet-name T --slot @0.xpub=xpub6FQya7zGhR92kacY… --slot @0.fingerprint=704c7836 --output -
-- current (unchanged after the fix)
exit=0
stdout: {
stdout:   "name": "T",
stdout:   "network": "mainnet",
stdout: … (24 lines in all; the import payload)
stderr: note: stdout is watch-only — public keys only, cannot spend
~~~

(G2's #20 changes the Sparrow keystore `label` to `Keystore 1`; the export still exits 0, section 10.)

### 2.19 #6 (I) descriptor-mode `bundle`: annotation fingerprint checked for `entropy` and `ms1` slots

**Behaviour.** In `bundle_run_unified_descriptor`, the Entropy and Ms1 arms get the Phrase arm's check: if the `@<idx>` annotation fingerprint differs from the slot's derived master fingerprint, `DescriptorParse`, exit 2, nothing on stdout, text `--slot @<idx>.entropy derives master fingerprint <fp> but descriptor @<idx> annotation specifies <anno>` (`.ms1` for the ms1 arm). Spec: none. Manual: none.

~~~text
$ mnemonic bundle --network mainnet --descriptor "wsh(sortedmulti(2,[deadbeef/48'/0'/0'/2']@0,@1))" --slot @0.entropy=@env:E0 --slot @1.xpub=xpub6DnEBNkSJKBYQmsb… --json
-- current
exit=0
stdout: {"schema_version":"4","mode"…
stderr: warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
-- after the fix
exit=2
stdout: (empty)
stderr: error: --slot @0.entropy derives master fingerprint 73c5da0a but descriptor @0 annotation specifies deadbeef
~~~

~~~text
$ mnemonic bundle --network mainnet --descriptor "wsh(sortedmulti(2,[deadbeef/48'/0'/0'/2']@0,@1))" --slot @0.ms1=@env:M0 --slot @1.xpub=xpub6DnEBNkSJKBYQmsb… --json
-- current
exit=0
stdout: {"schema_version":"4","mode"…
stderr: warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
-- after the fix
exit=2
stdout: (empty)
stderr: error: --slot @0.ms1 derives master fingerprint 73c5da0a but descriptor @0 annotation specifies deadbeef
~~~

Control (the Phrase arm today, unchanged):

~~~text
$ mnemonic bundle --network mainnet --descriptor "wsh(sortedmulti(2,[deadbeef/48'/0'/0'/2']@0,@1))" --slot @0.phrase=@env:PA --slot @1.xpub=xpub6DnEBNkSJKBYQmsb… --json
-- current (unchanged after the fix)
exit=2
stdout: (empty)
stderr: error: --slot @0.phrase derives master fingerprint 73c5da0a but descriptor @0 annotation specifies deadbeef
~~~

### 2.20 #9 (I) descriptor-mode `bundle`: an unparseable `--slot @N.fingerprint=` is refused

**Behaviour.** In `bundle_run_unified_descriptor`, xpub arm: `--slot @<idx>.fingerprint=` is parsed strictly: `BadInput`, exit 1, nothing on stdout, text `--slot @<idx>.fingerprint parse: <e>` (template mode already refuses it). Spec: none. Manual: none.

~~~text
$ mnemonic bundle --network mainnet --descriptor 'wpkh(@0/<0;1>/*)' --slot @0.xpub=xpub6FQya7zGhR92kacY… --slot @0.fingerprint=zzzz --json
-- current
exit=0
stdout: {"schema_version":"4","mode"…
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=1
stdout: (empty)
stderr: error: --slot @0.fingerprint parse: failed to parse hex
~~~

### 2.21 #45 (I) `bundle --import-json` refuses a `--network` that disagrees with the envelope

**Behaviour.** In `bundle_run_from_import_json`, right after the envelope network is read: if `--network` differs, `BadInput`, exit 1, nothing on stdout, text `--network <flag> disagrees with the --import-json envelope's bundle.network (<env>); re-run with --network <env>` (names from `CliNetwork::human_name()`: `mainnet`, `testnet`, `signet`, `regtest`). When the envelope's `bundle.network` is `testnet` and `--network` is `signet` or `regtest` (an envelope imported without `--network` records every coin-type-1 wallet as testnet), the text is instead `--network <flag> disagrees with the --import-json envelope's bundle.network (testnet); if the wallet is on <flag>, re-create the envelope with import-wallet --network <flag> from the same blob, otherwise re-run with --network testnet`. Spec: none. Manual: 5.1.

Fixture `env-testnet.json`: `import-wallet --format bsms --blob tests/fixtures/wallet_import/bsms-testnet-2of2.txt --json`.

~~~text
$ mnemonic bundle --network mainnet --import-json env-testnet.json --json
-- current
exit=0
stdout: {"schema_version":"4","mode"…
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=1
stdout: (empty)
stderr: error: --network mainnet disagrees with the --import-json envelope's bundle.network (testnet); re-run with --network testnet
~~~

The remedy, run as printed (unchanged):

~~~text
$ mnemonic bundle --network testnet --import-json env-testnet.json --json
-- current (unchanged after the fix)
exit=0
stdout: {"schema_version":"4","mode"…
stderr: note: stdout is watch-only — public keys only, cannot spend
~~~

A signet or regtest `--network` against a default (testnet) envelope:

~~~text
$ mnemonic bundle --network signet --import-json env-testnet.json --json
-- current
exit=0
stdout: {"schema_version":"4", … "network":"signet" …}
-- after the fix
exit=1
stdout: (empty)
stderr: error: --network signet disagrees with the --import-json envelope's bundle.network (testnet); if the wallet is on signet, re-create the envelope with import-wallet --network signet from the same blob, otherwise re-run with --network testnet
~~~

The remedy, run as printed:

~~~text
$ mnemonic import-wallet --format bsms --blob tests/fixtures/wallet_import/bsms-testnet-2of2.txt --network signet --json > env-signet.json
$ mnemonic bundle --network signet --import-json env-signet.json --json
exit=0
stdout: {"schema_version":"4", … "network":"signet" …}
~~~

### 2.22 #46 (I) `bundle --import-json` refuses every `--slot` subkey except `phrase`

**Behaviour.** In `bundle_run_from_import_json`, before the phrase overlay: the first `--slot` whose subkey is not `phrase` is refused, `BadInput`, exit 1, nothing on stdout, with one of two texts:

- `entropy`, `ms1`, `seedqr`: `--slot @<N>.<subkey>=: not supported with --import-json; only --slot @N.phrase= can add a seed to a watch-only cosigner. Get the phrase with: mnemonic convert --from <subkey>=- --to phrase (it reads the value from stdin and prints "phrase: <words>"), then give the words alone, without the "phrase: " label, on stdin to --slot @<N>.phrase=-`
- any other subkey (`xpub`, `master_xpub`, `fingerprint`, `path`, `wif`, `xprv`): `--slot @<N>.<subkey>=: not supported with --import-json; the envelope already supplies every cosigner's key, fingerprint and path (only --slot @N.phrase= is accepted)`

`<subkey>` is `SlotSubkey::as_str()`. Spec: none. Manual: 5.1.

Fixture `env-3of3.json`: `import-wallet --format bsms --json` of a 2-line BSMS blob holding the 2-of-3 `wsh(sortedmulti(…))` over the `[fp/87'/0'/0']` keys of `$PA`, `$SA`, `$SB` (the RED row's envelope; all slots watch-only).

~~~text
$ mnemonic bundle --network mainnet --import-json env-3of3.json --slot @0.entropy=@env:E0 --json
-- current
exit=0
stdout: {"schema_version":"4","mode"…
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=1
stdout: (empty)
stderr: error: --slot @0.entropy=: not supported with --import-json; only --slot @N.phrase= can add a seed to a watch-only cosigner. Get the phrase with: mnemonic convert --from entropy=- --to phrase (it reads the value from stdin and prints "phrase: <words>"), then give the words alone, without the "phrase: " label, on stdin to --slot @0.phrase=-
~~~

~~~text
$ mnemonic bundle --network mainnet --import-json env-3of3.json --slot @0.ms1=@env:M0 --json
-- current
exit=0
stdout: {"schema_version":"4","mode"…
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=1
stdout: (empty)
stderr: error: --slot @0.ms1=: not supported with --import-json; only --slot @N.phrase= can add a seed to a watch-only cosigner. Get the phrase with: mnemonic convert --from ms1=- --to phrase (it reads the value from stdin and prints "phrase: <words>"), then give the words alone, without the "phrase: " label, on stdin to --slot @0.phrase=-
~~~

~~~text
$ mnemonic bundle --network mainnet --import-json env-3of3.json --slot @1.fingerprint=97139860 --json
-- current
exit=0
stdout: {"schema_version":"4","mode"…
stderr: note: stdout is watch-only — public keys only, cannot spend
-- after the fix
exit=1
stdout: (empty)
stderr: error: --slot @1.fingerprint=: not supported with --import-json; the envelope already supplies every cosigner's key, fingerprint and path (only --slot @N.phrase= is accepted)
~~~

The seed remedy, run as printed (unchanged; the phrase then overlays slot 0):

~~~text
$ printf '%s\n' "$E0" | mnemonic convert --from entropy=- --to phrase
-- current (unchanged after the fix)
exit=0
stdout: phrase: abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
stderr: warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
~~~

~~~text
$ printf '%s\n' "$M0" | mnemonic convert --from ms1=- --to phrase
-- current (unchanged after the fix)
exit=0
stdout: phrase: abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
stderr: warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
~~~

~~~text
$ printf '%s\n' "$PA" | mnemonic bundle --network mainnet --import-json env-3of3.json --slot @0.phrase=- --json
-- current (unchanged after the fix)
exit=0
stdout: {"schema_version":"4","mode"…
stderr: warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
~~~

Control (a phrase overlay, unchanged):

~~~text
$ mnemonic bundle --network mainnet --import-json env-3of3.json --slot @1.phrase=@env:SA --json
-- current (unchanged after the fix)
exit=0
stdout: {"schema_version":"4","mode"…
stderr: warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
~~~

### 2.23 #54 (I) descriptor-mode `bundle --json` reports `multisig.path_family: "descriptor"`

**Behaviour.** In `emit_unified`, `MultisigInfo.path_family` is `"descriptor"` when the bundle has no template (`--descriptor`, `--descriptor-file`, `--import-json`), per `SPEC_mnemonic_toolkit_v0_3.md` §5.3 ("populated with the literal static string `"descriptor"` for both fields"); template mode is unchanged (`--multisig-path-family`, default `bip87`). `multisig.template` is already `"descriptor"` there. `import-wallet`'s own envelope field is not changed (section 10). Spec: none. Manual: 5.13, 5.14.

`mnemonic bundle --network mainnet --descriptor "wsh(sortedmulti(2,[704c7836/48'/0'/0'/2']@0/<0;1>/*,[97139860/48'/0'/0'/2']@1/<0;1>/*))" --slot @0.xpub=xpub6FQya7zGhR92kacYsNnj… --slot @1.xpub=xpub6DnEBNkSJKBYQmsbhS1s… --json | jq -c '.multisig | [.template, .path_family]'`: today `["descriptor","bip87"]`, after `["descriptor","descriptor"]` (exit 0 both). Same for `--import-json env-testnet.json` (`--network testnet`) and `env-3of3.json`. Control, `--template sh-wsh-sortedmulti --threshold 2 --multisig-path-family bip48` over `$SA`,`$SB`: `["sh-wsh-sortedmulti","bip48"]` on both binaries.

### 2.24 #8 (I) `wc_codec::encode` refuses `u_slots` outside 1..=7 (library only)

**Behaviour.** `encode_inner` (`crates/wc-codec/src/pipeline.rs`): `u_slots == 0 || u_slots > 7` → `Err(WcError::InvalidParams)` (fix-08). The GEOM header stores U in 3 bits; today U = 8 writes a card whose header says 0 and which never decodes (RED row: a 57-word card, `decode` → `Sync(AmbiguousRealignment)`). Measured on the scratch build: U = 1..7 each encode and decode back the payload (43…55 words for a 33-byte payload); U = 0, 8, 9, 255 each return `InvalidParams`. No toolkit caller sets `u_slots` (`grep -rn u_slots crates/mnemonic-toolkit/src` is empty). Rustdoc: `EncodeOpts::u_slots` gains "Must be 1..=7: the GEOM header stores U in 3 bits."; `WcError::InvalidParams`'s example list `u_slots == 0` becomes `u_slots outside 1..=7`. Spec: none (no word-card spec states the range). Manual: none.

## 3. Code delta (production)

The reference patch implements all of it; the build writes it from sections 2 and 3. Triage patches (`/scratch/cache/bughunt/tk/{s1,s2}/fix-NN.patch`) used **verbatim**: fix-01, -04, -05, -06, -08, -09, -12, -13, -38, -40, -43, -53. Used **with the texts or shape of section 2**: fix-02, -03, -10, -41, -42, -46, -52. **Not used**: fix-37 (breaks `--origin`), fix-39 and fix-44 (replaced by 2.11 and 2.12), fix-45 and fix-54 (other rulings), fix-47 (replaced by the shared decoder), fix-11 (superseded by the 11b gate), fix-07 and fix-14 (deferred).

1. `crates/mnemonic-toolkit/src/cmd/restore.rs`
   - `run`, verification gate: #1 (two independent checks, fingerprint first).
   - `run_singlesig_template_completion`: 11b guard, then the #2 `--format` refusal (both before `--from` is read); the #2 reference checks after the descriptor is built; the #2 banners after the output.
   - `emit_completed_multisig`, `emit_candidate_list`: #10 routing (the listing builds its body into one `String`, then stdout or file).
   - `complete_multisig_template`: #41 refusal (before the subset-search conflict check); #40 `own_origin_for` fallback.
   - `run_multisig`: #42 fingerprint check at the own-slot match; #38 template-arm routing on the baseline use-site.
   - `classify_taproot_restore`: #53 distinct-index check before the tag dispatch.
   - `decode_wallet_id_prefix`: `pub(crate)` (#47).
2. `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs`
   - New private `fn ms1_card_checks(supplied, expected, no_auto_repair) -> Vec<VerifyCheck>`: the keyed single-sig ms1 block, moved unchanged; `emit_checks` calls it; `verify_singlesig_template` gains a `no_auto_repair: bool` parameter (the dispatcher passes `effective_no_auto_repair`) and calls it for #37.
   - `emit_md1_checks`: #39. `descriptor_mode_verify_run`: #44. `verify_singlesig_template`: #47. `check_mk1_template_stubs` returns a private three-state result (bound / not bound / unreadable) for #52. `emit_multisig_checks`: #12, #13, #43.
3. `crates/mnemonic-toolkit/src/cmd/bundle.rs`: `bundle_run_unified_descriptor` #6 (Entropy, Ms1 arms), #9 (xpub arm); `bundle_run_from_import_json` #45, #46; `emit_unified` #54.
4. `crates/mnemonic-toolkit/src/parse_descriptor.rs`: `lex_placeholders` #3.
5. `crates/mnemonic-toolkit/src/synthesize.rs`: `template_admissible` (#11, 11b) and the `TemplateFormUnsupportedShape` message in `synthesize_template_bundle`'s gate.
6. `crates/mnemonic-toolkit/src/electrum.rs`: `is_cjk` #4. `crates/mnemonic-toolkit/src/wallet_export/electrum.rs`: `ElectrumEmitter::emit` #5.
7. `crates/wc-codec/src/pipeline.rs`: `encode_inner` #8; `crates/wc-codec/src/lib.rs`: the two rustdoc lines of 2.24.

No new public item, flag, error variant or JSON key. The `RestoreMismatch.reference` value `"cosigner-fingerprint"` (#42) appears only in its message (the variant has no JSON envelope, SPEC_mnemonic_restore line 106).

## 4. Spec delta

Five specs. No fix changes more than four sentences of a spec (#2's new bullet, three sentences, is the longest; the template-only spec gets six in all, from four fixes). Each old text occurs once (grep-checked).

**4.1 `design/SPEC_bundle_md1_template_only_2026-06-19.md`**

(a) §4.2, 11b and #11. Old:
~~~text
**Canonical gate (C1).** `--md1-form=template` REQUIRES **`descriptor.n == 1 && canonical_origin(&d.tree).is_some()`**.
~~~
New:
~~~text
**Canonical gate (C1).** `--md1-form=template` REQUIRES **`descriptor.n == 1 && canonical_origin(&d.tree).is_some()`**, the standard `/<0;1>/*` use-site and no use-site overrides: single-sig completion rebuilds only `/<0;1>/*`, so any other use-site (`/*`, `/<2;3>/*`, `/*h`) is refused here, and `restore` refuses such a card (FIX_bughunt_tk_g1 11b).
~~~

(b, c) §4.5, #47 and #2. Old:
~~~text
**advisory minimum ≥4 bytes** (M-flex — warn, don't enforce, on a shorter prefix).
~~~
New:
~~~text
**advisory minimum ≥4 bytes** (M-flex — warn, don't enforce, on a shorter prefix). The prefix is 1–16 bytes of even-length hex; an empty, odd-length or non-hex, or longer value is refused (`BadInput`, exit 1) on `restore` and `verify-bundle` alike (FIX_bughunt_tk_g1 #47).
- **Card use-site and the single-sig reference flags (FIX_bughunt_tk_g1 11b, #2).** A template md1 whose use-site is not `/<0;1>/*`, or which carries use-site overrides, is refused before the seed is read, and so is `--format` (`ModeViolation`, exit 2 each). `--expect-fingerprint`, `--expect-xpub` and `--allow-mismatch` behave as in seed-only restore (`SPEC_mnemonic_restore.md`): a mismatch is `RestoreMismatch` (exit 4) with nothing on stdout, and `--allow-mismatch` prints the wallet under the overridden banner. The `UNVERIFIED` banner prints only when no reference (`--expect-wallet-id`, `--expect-fingerprint`, `--expect-xpub`) is given.
~~~

(d) §4.6, #37. Old:
~~~text
`--expect-wallet-id` supported here too (same recompute-and-match).
~~~
New:
~~~text
`--expect-wallet-id` supported here too (same recompute-and-match). A supplied `--ms1` is compared against the seed slot's card with the keyed single-sig `ms1_decode` / `ms1_entropy_match` rows (FIX_bughunt_tk_g1 #37).
~~~

**4.2 `design/SPEC_bundle_md1_template_multisig_2026-06-20.md`** §2 (line 34), #41. Old:
~~~text
**Mode/trigger precedence at completion:** (a) `--expect-wallet-id` → id-search; (b) else `--search-address` → address-search; (c) else all keys given as `--cosigner @N=` → explicit (Mode B) + warning; (d) else REFUSE (`bad`): "supply a recorded `--expect-wallet-id`, a `--search-address`, or explicit `--cosigner @N=` assignments." Sorted shapes (§6.1) skip the search.
~~~
New:
~~~text
**Mode/trigger precedence at completion:** any `--cosigner @N=` selects explicit mode (Mode B) + warning; it checks a supplied `--expect-wallet-id` against the built wallet (a mismatch refuses, exit 4) and REFUSES `--search-address` (`bad`, exit 1: the address would not be checked; FIX_bughunt_tk_g1 #41). Without `@N=`: (a) `--expect-wallet-id` → id-search; (b) else `--search-address` → address-search; (c) else REFUSE (`bad`): "supply a recorded `--expect-wallet-id`, a `--search-address`, or explicit `--cosigner @N=` assignments." Sorted shapes (§6.1) skip the search.
~~~

**4.3 `design/SPEC_restore_wallet_id_prefix_enumerate.md`** stream table, #10. Old:
~~~text
| candidate rows | **stdout** (they are the payload) | in `candidates[]` |
~~~
New:
~~~text
| candidate rows | **stdout** (they are the payload), or `--output <FILE>` when given | in `candidates[]`, routed the same way |
~~~
and after the sentence ending `success-with-no-output.`, add: `` `--output <FILE>` moves the payload (the text rows or the `--json` object) to FILE, as it does for a reconstructed wallet; stderr is unchanged (FIX_bughunt_tk_g1 #10). ``

**4.4 `design/SPEC_export_wallet_v0_8.md`** §9.1, #5. After the line
~~~text
- `keystore.xpub`: **SLIP-132 form** matching script-type × network (`vpub`/`zpub`/`upub`/`ypub`/`tpub`). Conversion via `crate::slip0132`.
~~~
insert
~~~text
- `--template bip86` (P2TR) is REFUSED (`BadInput`, exit 1; FIX_bughunt_tk_g1 #5): Electrum reads a plain `xpub` "standard" keystore as p2pkh (`electrum/wallet.py` line 4241 at `e1099925`: `self.txin_type = 'p2pkh' if xtype == 'standard' else xtype`). Text: `--format electrum does not support --template bip86 (P2TR): a plain xpub "standard" wallet is read by Electrum as p2pkh. Use --format bitcoin-core (descriptor) or --format sparrow for taproot watch-only setup.`
~~~
(The wallet.py line was fetched from `raw.githubusercontent.com/spesmilo/electrum/e1099925/electrum/wallet.py` on 2026-11-01 and grep-checked.)

**4.5 `design/SPEC_addresses_electrum_native_derivation.md`** line 20, #4. Old:
~~~text
and `is_cjk` covers the 9 intervals of Electrum's 28 `CJK_INTERVALS` that correspond to shipped wordlists — adequate, not literally all 28
~~~
New:
~~~text
and `is_cjk` is Electrum's `CJK_INTERVALS` table verbatim (all 29 intervals, `mnemonic.py` lines 40-70 at `e1099925`): the passphrase is free text, and a table limited to the shipped wordlists' scripts derived another seed for, e.g., a space between two Hangul syllables (FIX_bughunt_tk_g1 #4)
~~~

No spec change for the other fixes: they bring the code in line with existing text (`SPEC_mnemonic_restore.md` §3 for #1; `SPEC_restore_multisig_cosigner.md` lines 27-28 for #42; `SPEC_mnemonic_toolkit_v0_3.md` §5.3 for #54; `SPEC_cycleA_descriptor_use_site_collapse.md` §2, "UseSitePath is always wildcarded", for #3), or no spec describes them.

## 5. Manual delta (same PR)

All in this repo. The script `/scratch/cache/bughunt/tk/g1fix/manual-edits.py <repo-root>` applies the 14 replacements below, asserting each old text occurs once; on the scratch copy it gave the checks of the header table (lint unchanged, verify-examples clean for `41-bundle-inheritance-json`, anchor-check OK). `docs/manual-gui/` is not edited in this PR (section 9.3).

**5.1 #45, #46 — `bundle` flag table, `--import-json` row** (`docs/manual/src/40-cli-reference/41-mnemonic.md`). Replace, once:

~~~text
supplying overlay for an already-seeded slot is `BadInput`. The replayed
~~~

with:

~~~text
supplying overlay for an already-seeded slot is `BadInput`. Any other `--slot` subkey (`entropy`, `ms1`, `seedqr`, `xpub`, `fingerprint`, `path`, …) is refused (exit 1): get the phrase of a seed card with `mnemonic convert --from <subkey>=- --to phrase` (it prints `phrase: <words>`) and give the words alone, without the "phrase: " label, on stdin to `--slot @N.phrase=-`. `--network` must equal the envelope's `bundle.network`; a disagreement is refused (exit 1). An envelope imported without `--network` records a signet or regtest wallet as testnet: re-create it with `import-wallet --network signet` (or `regtest`). The replayed
~~~

**5.2 #3 — `bundle` § Non-representable use-site steps (a new paragraph after the `Remedy:` paragraph)** (`docs/manual/src/40-cli-reference/41-mnemonic.md`). Replace, once:

~~~text
the `/**` shorthand for the same thing.

**Sparrow is unaffected.**
~~~

with:

~~~text
the `/**` shorthand for the same thing.

A **multipath with no trailing wildcard** — `@0/<0;1>` (or an inline
`[fp/path]xpub/<0;1>`) with no `/*` after it — names a non-ranged wallet
that md1 cannot encode either. It used to be encoded silently as the
ranged `/<0;1>/*` wallet; it is now refused (exit 2, `DescriptorParse`) on
the same surfaces. Bare `@N`, with no use-site at all, is unaffected. The
remedy above does **not** apply: adding `/*` describes a different wallet
(`/<0;1>/*` ranges over `/0/i` and `/1/i`; `/<0;1>` is the two keys `/0`
and `/1`). md1 has no card for it; keep the full descriptor.

**Sparrow is unaffected.**
~~~

**5.3 #2, #37 — `bundle` § Template-only md1: the verify-bundle paragraph (corrected) and a new paragraph** (`docs/manual/src/40-cli-reference/41-mnemonic.md`). Replace, once:

~~~text
The same completion works on [`verify-bundle`](#mnemonic-verify-bundle)
via its `--from` + `--account` / `--origin` / `--expect-wallet-id`
flags. `--expect-wallet-id` recomputes the `WalletPolicyId` from the
completed wallet and refuses loudly on mismatch (exit 4); it is **not**
checked when `--origin` is supplied (the explicit-origin id is a
different preimage from the canonical-account one).

~~~

with:

~~~text
The same completion works on [`verify-bundle`](#mnemonic-verify-bundle)
via its `--slot @0.phrase=` seed slot (or `.ms1=` / `.entropy=` /
`.seedqr=`) + `--account` / `--origin` / `--expect-wallet-id` flags.
`--expect-wallet-id` recomputes the `WalletPolicyId` from the
completed wallet and refuses loudly on mismatch (exit 4); it is **not**
checked when `--origin` is supplied (the explicit-origin id is a
different preimage from the canonical-account one). On `verify-bundle`, a
supplied `--ms1` is compared against the seed (the `ms1_decode` /
`ms1_entropy_match` rows of the keyed single-sig check); a card for a
different seed fails (exit 4).

The single-sig reference flags work on `restore` here too: a wrong
`--expect-fingerprint` or `--expect-xpub` refuses with `✗ MISMATCH`
(exit 4) unless `--allow-mismatch`, which prints the descriptor under a
`✗ MISMATCH (overridden)` banner. `--format` is refused (exit 2); run
without it to print the completed descriptor.

~~~

**5.4 #11, 11b — `bundle` § Template-only md1: the shapes paragraph** (`docs/manual/src/40-cli-reference/41-mnemonic.md`). Replace, once:

~~~text
The single-sig template form covers the canonical shapes `bip44` /
`bip84` / `bip86`; `bip49` (nested-segwit) and non-canonical single-sig
are refused — use `--md1-form=policy` for those.
~~~

with:

~~~text
The single-sig template form covers the canonical shapes `bip44` /
`bip84` / `bip86` at the standard `/<0;1>/*` use-site; `bip49`
(nested-segwit), non-canonical single-sig, and any other use-site (`/*`,
`/<2;3>/*`, a hardened `/*h`) are refused (exit 2) — use
`--md1-form=policy` for those. `restore` also refuses (exit 2) a
single-sig template card with another use-site, which earlier versions
could emit.
~~~

**5.5 #41 — `bundle` § Multisig template completion, mode 3** (`docs/manual/src/40-cli-reference/41-mnemonic.md`). Replace, once:

~~~text
3. **explicit assignment** — pin every cosigner with `--cosigner @N=` (no
   search).
~~~

with:

~~~text
3. **explicit assignment** — pin every cosigner with `--cosigner @N=` (no
   search). It checks `--expect-wallet-id` if given (a mismatch refuses,
   exit 4) and refuses `--search-address` (exit 1): drop the `@N=` prefixes
   to let the address place the cosigners.
~~~

**5.6 #47 — `verify-bundle` flag table, `--expect-wallet-id` row** (`docs/manual/src/40-cli-reference/41-mnemonic.md`). Replace, once:

~~~text
a mismatch is a **failed check** (overall `mismatch`, exit 4). Only meaningful for a template bundle;
~~~

with:

~~~text
a mismatch is a **failed check** (overall `mismatch`, exit 4). An empty, odd-length or non-hex, or longer-than-16-byte prefix is refused (exit 1), as on [`restore`](#mnemonic-restore). Only meaningful for a template bundle;
~~~

**5.7 #5 — `export-wallet` flag table, `--format` row** (`docs/manual/src/40-cli-reference/41-mnemonic.md`). Replace, once:

~~~text
emits an `addr()` WATCH LIST instead of a descriptor — see "Watching a wallet Core will not describe" below |
~~~

with:

~~~text
emits an `addr()` WATCH LIST instead of a descriptor — see "Watching a wallet Core will not describe" below. **`electrum`** refuses every taproot template (`bip86`, `tr-multi-a`, `tr-sortedmulti-a`; exit 1): Electrum reads a plain-xpub `standard` wallet as p2pkh and has no taproot multisig wallet type, so use `bitcoin-core` or `sparrow` for taproot |
~~~

**5.8 #42 — `restore` flag table, `--from` row** (`docs/manual/src/40-cli-reference/41-mnemonic.md`). Replace, once:

~~~text
where it cross-checks the own cosigner position (inferred by matching the derived key against the md1's slots).
~~~

with:

~~~text
where it cross-checks the own cosigner position (inferred by matching the derived key against the md1's slots) and that slot's engraved master fingerprint (a different engraved fingerprint refuses, exit 4, unless `--allow-mismatch`).
~~~

**5.9 #41 — `restore` flag table, `--search-address` row** (`docs/manual/src/40-cli-reference/41-mnemonic.md`). Replace, once:

~~~text
Recommended over `--expect-wallet-id` (full-scriptPubKey match — collision-free). See [Multisig template completion](#multisig-template-completion) |
~~~

with:

~~~text
Recommended over `--expect-wallet-id` (full-scriptPubKey match — collision-free). Not combinable with explicit `--cosigner @N=` placement (exit 1). See [Multisig template completion](#multisig-template-completion) |
~~~

**5.10 #2 — `restore` flag table, `--expect-xpub` row** (`docs/manual/src/40-cli-reference/41-mnemonic.md`). Replace, once:

~~~text
| `--expect-xpub <EXPECT_XPUB>` | reference account xpub (requires `--template`); mismatch → exit 4 (unless `--allow-mismatch`) |
~~~

with:

~~~text
| `--expect-xpub <EXPECT_XPUB>` | reference account xpub (requires `--template`, or a keyless single-sig template `--md1`, whose type it takes); mismatch → exit 4 (unless `--allow-mismatch`) |
~~~

**5.11 #2 — `restore` flag table, `--format` row** (`docs/manual/src/40-cli-reference/41-mnemonic.md`). Replace, once:

~~~text
REQUIRES a single `--template` (one-descriptor-in/one-out); `--format` with no `--template` → exit 2.
~~~

with:

~~~text
REQUIRES a single `--template` (one-descriptor-in/one-out); `--format` with no `--template` → exit 2. Refused (exit 2) when completing a keyless single-sig template `--md1`.
~~~

**5.12 #53 — `restore` § Multisig-cosigner restore, the `Scope.` paragraph** (`docs/manual/src/40-cli-reference/41-mnemonic.md`). Replace, once:

~~~text
`restore-non-nums-tr-internal-key-also-in-leaf`), as is a `sortedmulti_a`
leaf inside a multi-leaf tree (exit 2, citing its tracking slug).
~~~

with:

~~~text
`restore-non-nums-tr-internal-key-also-in-leaf`), as is a `sortedmulti_a`
leaf inside a multi-leaf tree (exit 2, citing its tracking slug), and a
`multi_a` / `sortedmulti_a` leaf that repeats a key index (exit 2).
~~~

**5.13 #54 — `MultisigInfo` field table** (`docs/technical-manual/src/50-rust-api/54-mnemonic-toolkit-api.md`). Replace, once:

~~~text
| `path_family` | `&'static str` | `"bip48"` or `"bip87"` |
~~~

with:

~~~text
| `path_family` | `&'static str` | `"bip48"` or `"bip87"` (`--template` mode); `"descriptor"` in descriptor mode (`--descriptor`, `--descriptor-file`, `--import-json`), per SPEC v0.3 §5.3 |
~~~

**5.14 #54 — transcript line 43 (the only drift `make verify-examples` reports with the fixed binary)** (`docs/manual/transcripts/41-bundle-inheritance-json.out`). Replace, once:

~~~text
        "path_family": "bip87",
~~~

with:

~~~text
        "path_family": "descriptor",
~~~


## 6. RED rows

Rows are in `crates/mnemonic-toolkit/tests/bughunt_s1.rs`, `bughunt_s2.rs` and `crates/wc-codec/tests/bughunt_s2.rs`. "RED today" quotes the failure on this worktree's unmodified code (`/scratch/cache/bughunt/tk/g1fix/red-cur.log`). Rows drive the binary with `--allow-argv-secret`, so stderr may begin with `warning: secret material on argv …` lines; assertions on stderr are "contains", and on stdout "is empty" where stated. The header comments that say "open product question" become the ruling; their wording is not binding.

| row | after the build | exact assertion | RED today |
|---|---|---|---|
| `bughunt_tk_01_restore_checks_xpub_even_when_fingerprint_matches` | GREEN (tightened) | exit 4; stdout empty; stderr contains `✗ MISMATCH` and `restore: xpub mismatch — derived xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V, expected <ACCT1_XPUB_BIP84>` | `left: Some(0) right: Some(4)` |
| `bughunt_tk_02_singlesig_template_md1_honours_reference_flags` | GREEN (tightened to the ruling) | (a) `--expect-fingerprint deadbeef`: exit 4, stdout empty, stderr contains `✗ MISMATCH\nmaster fingerprint: 73c5da0a  (passphrase: none)\nerror: restore: fingerprint mismatch — derived 73c5da0a, expected deadbeef`; (b) `--expect-xpub ACCT1_XPUB_BIP84`: exit 4, stdout empty, stderr contains `restore: xpub mismatch`; (c) (a) + `--allow-mismatch`: exit 0, stdout contains `first recv: bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`, stderr contains the overridden banner of 2.2 item 5 and not `UNVERIFIED`; (d) `--format bitcoin-core`: exit 2, stdout empty, stderr contains `error: ` + the 2.2 item 2 text; (e) new: `--expect-fingerprint 73c5da0a`: exit 0, stderr does not contain `UNVERIFIED` | `(a) --expect-fingerprint deadbeef: want exit 4 (or a refusal), got exit 0` |
| `bughunt_tk_10_multisig_template_restore_honours_output_file` | GREEN (unchanged) | as written | `--output … was not created (the document went to stdout instead)` |
| **NEW** `bughunt_tk_10b_candidate_list_honours_output_file` | GREEN | cos = `[(SEED_A,0),(SEED_B,0),(SEED_C,0)]`, `wsh-multi`, k 2: `md1 = emit_template_md1`, `prefix = emit_template_wallet_id(..)[..4]` (`a693`), `mk1_b`, `mk1_c` = `emit_cosigner_mk1(.., 1/2)`, `mk1_outsider = emit_cosigner_mk1("wsh-multi","2",&[(SEED_A,0),(PHRASE_A,0)],1)`; args `restore --network mainnet <md1> --from phrase=<SEED_A> --own-account-max 256 --search-cosigner-subset --expect-wallet-id a693 --cosigner <each chunk of b, c, outsider>`. Run 1 (no `--output`): exit 0, stderr contains `9 assignments match prefix a693` and `none reconstructed`, stdout has 27 lines and begins `  [1] wallet-id a693d0f4447150c94fce12d852c19af5`. Run 2 (+ `--output <tmp>/list.txt`): exit 0, stdout empty, the file's bytes == run 1's stdout. Run 3 (+ `--json --output <tmp>/list.json`): exit 0, stdout empty, the file parses as JSON with `match_count == 9`, `candidates.len() == 9`, and no `wallets` key | (new; run 2 today: the file is not created and the rows go to stdout, measured in 2.3) |
| `bughunt_tk_38_taproot_multi_a_nonstandard_usesite_not_silently_replaced` | GREEN (extended) | the three existing cases, with "exit 0 and stdout contains the golden" now required for `/<2;3>/*` and `/*` too (no refusal branch); add `tr(NUMS,sortedmulti_a(2,TK0/<2;3>/*,TK1/<2;3>/*,TK2/<2;3>/*))`: exit 0, stdout contains `sortedmulti_a(` and `/<2;3>/*` and `first recv: bc1pv963as6zz3nf2qk6xqhux4pgduqldc3tx88a6a2szglrgr92h5msgcq2my` (Core-derived literal, 2.8) | `use-site /<2;3>/*: restore exit 0 but first recv is NOT the card's wallet (want bc1prhzr704…)` |
| `bughunt_tk_53_taproot_multi_a_repeated_key_index_is_refused` | GREEN (tightened) | exit 2; stdout empty; stderr contains `error: taproot md1 repeats a key index inside its multi_a/sortedmulti_a leaf; the toolkit cannot reconstruct it faithfully` | `assertion left != right failed … restore must refuse` |
| `bughunt_tk_40_template_restore_bare_xpub_cosigner_keeps_own_account` | GREEN (unchanged) | as written | `✗ NO MATCH` (exit 4) |
| **NEW** `bughunt_tk_40b_explicit_placement_bare_xpub_cosigner_keeps_own_account` | GREEN | the #40 fixture; args `restore --network mainnet <md1> --from phrase=<SEED_A> --account 1 --cosigner @1=<xpub_b> --count 2 --json` (no search target): exit 0, `wallets[0].first_addresses == golden` (`["bc1qutaes7u4vv3fn9wtmdfpx3wvset9exs59un9r00aw7lyryxd3ymqsf73hu","bc1qfaen6p88s6nzyz6nxcnp6j08vgvs7jfk7u9q8qttrad5xrfcsw3srenn5m"]`) | (new; today exit 0 with `first_addresses[0] == "bc1q2a4vm7ww7v0c02qerrgg0znr4ck4kez2la82kzpm9gturx68q4nsfdl000"`, 2.4) |
| `bughunt_tk_41_explicit_placement_does_not_ignore_search_address` | GREEN (tightened) | exit 1; stdout empty; stderr contains `error: ` + the 2.5 text | `left: Some(0)` and a printed wallet |
| `bughunt_tk_42_own_slot_with_wrong_engraved_fingerprint_is_not_verified` | GREEN (tightened, plus a sub-case) | exit 4; stdout empty; stderr contains `✗ MISMATCH\nerror: restore: cosigner-fingerprint mismatch at slot @0 — derived seed master fp b8688df1, expected md1 cosigner @0 engraved fingerprint deadbeef`. Sub-case `+ --allow-mismatch`: exit 0, stdout contains `cosigner @0: deadbeef [87'/0'/0']  from md1 (not independently verified)` and not `your seed (verified)`, stderr contains `✗ MISMATCH (overridden)` | `yet it is labelled verified (exit Some(0))` |
| `bughunt_tk_12_stray_mk1_does_not_blame_correct_cosigner_slots` | GREEN (unchanged) | as written | `stray #1: mk1_decode[0]=Some(false) mk1_decode[2]=Some(true)` |
| `bughunt_tk_13_positional_fallback_keeps_supplied_card_order` | GREEN (unchanged) | as written | `variant 2: mk1_xpub_match[0] = Some(false) although cards were supplied in slot order` |
| `bughunt_tk_14_multisig_json_has_md1_origin_row_when_md1_does_not_decode` | `#[ignore = "deferred-minor: verify-bundle-multisig-json-md1-origin-row-on-decode-fail"]`, body unchanged | — | still fails when run ignored |
| `bughunt_tk_43_supplied_md1_with_extra_cosigner_does_not_panic` | GREEN (tightened) | exit 4 (not 101); stderr does not contain `panicked`; stdout JSON `result == "mismatch"` | `index out of bounds: the len is 2 but the index is 2` |
| `bughunt_tk_37_singlesig_template_verify_checks_the_ms1_card` | GREEN (tightened) | control: exit 0 and stderr contains `✓ ms1_entropy_match: ms1 byte-identical`; other seed: exit 4, stdout == `mismatch\n`, stderr contains `✓ ms1_decode: decoded successfully` and `✗ ms1_entropy_match: expected ms1 bytes differ from supplied`; Japanese control (JP = the Japanese phrase of 16 zero bytes, `あいこくしん` ×11 then `あおぞら`): `mnemonic bundle --network mainnet --template bip84 --md1-form template --group-size 0 --language japanese --slot @0.phrase=@env:JP --json`, then `mnemonic verify-bundle --network mainnet --language japanese --slot @0.phrase=@env:JP --ms1 - <its mk1, md1>` with that bundle's `ms1[0]` on stdin: exit 0, stderr contains `✓ ms1_entropy_match: ms1 byte-identical` | `an --ms1 card from a DIFFERENT seed must fail verification (exit 4)`, got 0 |
| `bughunt_tk_39_singlesig_md1_with_other_usesite_or_script_does_not_pass` | GREEN (tightened) | each alternative: exit 4, `result == "mismatch"`, the `md1_xpub_match` row has `passed: false` and detail `md1 reconstructs a DIFFERENT wallet — structural policy mismatch: use-site path (change-chain/multipath)` (use-site case) / `… policy tree (threshold/script-type/order)` (pkh case) | `use-site <2;3>: … verify-bundle says result="ok" (exit Some(0))` |
| `bughunt_tk_44_descriptor_mode_verify_uses_annotation_fingerprint` | GREEN (unchanged) | as written | `failed checks: ["mk1_fingerprint_match[1]" …]` |
| **NEW** `bughunt_tk_44b_descriptor_mode_verify_refuses_fingerprint_annotation_disagreement` | GREEN | the #44 fixture; the verify command adds `--slot @1.fingerprint=deadbeef`: exit 2, stdout empty, stderr contains `error: --slot @1.fingerprint specifies deadbeef but descriptor @1 annotation specifies <fp_b>` (fp_b = `28645006`) | (new; today exit 4, 2.12) |
| **NEW** `bughunt_tk_44c_descriptor_mode_verify_refuses_unparseable_slot_fingerprint` | GREEN | the verify command adds `--slot @1.fingerprint=zzzz`: exit 1, stdout empty, stderr contains `error: --slot @1.fingerprint parse: ` | (new; today exit 4) |
| `bughunt_tk_47_empty_expect_wallet_id_is_rejected_by_verify_bundle` | GREEN (tightened) | verify: exit 1, stdout empty, stderr contains `error: --expect-wallet-id must not be empty`; the restore half unchanged | `exit Some(0)` with `✓ wallet_id_match` |
| `bughunt_tk_03_multipath_without_wildcard_rejected` | GREEN (tightened) | both forms: exit 2, stdout empty, stderr contains ``error: @0: a multipath use-site `/<a;b>` with no trailing `/*` (a non-ranged multipath) is not representable in md1, which encodes only a ranged `/<a;b>/*` or bare `/*` use-site`` | ``placeholder: `wpkh(@0/<0;1>)` exited 0`` |
| `bughunt_tk_04_electrum_passphrase_strips_space_between_hangul` | GREEN (unchanged) | as written | `left: "receive (m/0/i):\n  0  bc1qeszqkdyegvtcgtl9jwzy7p66szuuqzmr8zg23m…"` |
| `bughunt_tk_05_electrum_export_bip86_refused` | GREEN (tightened) | exit 1; stdout empty; stderr contains `error: ` + the 2.18 text | `exited 0 and wrote a plain-xpub "standard" wallet` |
| `bughunt_tk_06_descriptor_anno_fp_checked_for_entropy_and_ms1_slots` | GREEN (tightened) | each: exit 2, stderr contains `--slot @0.entropy derives master fingerprint 73c5da0a but descriptor @0 annotation specifies deadbeef` / `--slot @0.ms1 derives master fingerprint 73c5da0a but descriptor @0 annotation specifies deadbeef` | `@0.entropy: exit 0; annotation deadbeef silently ignored` |
| `bughunt_tk_09_unparseable_slot_fingerprint_refused` | GREEN (tightened) | exit 1; stdout empty; stderr contains `error: --slot @0.fingerprint parse: ` | `exit 0, master_fingerprint "00000000"` |
| `bughunt_tk_11_singlesig_hardened_usesite_template_form_refused` | GREEN (unchanged; its substring is kept) | as written | `accepted a hardened use-site (/*h)` |
| **NEW** `bughunt_tk_11b_singlesig_nonstandard_usesite_template_form_refused` | GREEN | `bundle --network mainnet --descriptor <D> --md1-form template --slot @0.phrase=<PHRASE_A> --json` for D ∈ {`wpkh(@0)`, `wpkh(@0/*)`, `wpkh([73c5da0a/84'/0'/0']@0/<2;3>/*)`}: each exit 2, stdout empty, stderr contains `cannot template this descriptor shape` and `or it is single-sig with a use-site other than /<0;1>/*`. Control D = `wpkh([73c5da0a/84'/0'/0']@0/<0;1>/*)`: exit 0 and JSON `md1 == ["md1fvwrwqqpqqgqpsqqq3uaau4ctxyl7"]` | (new; today each D exits 0, 2.9) |
| **NEW** `bughunt_tk_11b_restore_refuses_nonstandard_usesite_singlesig_template` | GREEN | `restore --network mainnet --md1 <M> --from phrase=<PHRASE_A> --account 0` for the literals M ∈ {`md1fkykhqqpqqgzskqqzxcjnwq6cq0gv`, `md1fu85lqqpqqqqqnm3nr4htwmp8w`}: each exit 2, stdout empty, stderr contains `error: ` + the 2.9 restore text; the first M with `--expect-wallet-id 1c0170fe` too: exit 2, stderr not containing `wallet-id verified`. Control M = `md1fvwrwqqpqqgqpsqqq3uaau4ctxyl7`: exit 0, stdout contains `first recv: bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`. The cards are literals because the fixed bundle no longer mints them | (new; today exit 0 and `✓ wallet-id verified`, 2.9) |
| `bughunt_tk_45_import_json_testnet_envelope_not_labelled_mainnet` | GREEN (rewritten: today it passes vacuously on any refusal) | `--network mainnet`: exit 1, stdout empty, stderr contains `error: --network mainnet disagrees with the --import-json envelope's bundle.network (testnet); re-run with --network testnet`; `--network testnet`: exit 0 and JSON `network == "testnet"`; `--network signet`: exit 1, stdout empty, stderr contains `error: --network signet disagrees with the --import-json envelope's bundle.network (testnet); if the wallet is on signet, re-create the envelope with import-wallet --network signet from the same blob, otherwise re-run with --network testnet`; remedy: the envelope from `import-wallet --format bsms --blob tests/fixtures/wallet_import/bsms-testnet-2of2.txt --network signet --json`, then `bundle --network signet` on it: exit 0 and JSON `network == "signet"` | `left: String("mainnet") right: "testnet"` |
| `bughunt_tk_46_import_json_slot_overlays_other_than_phrase_not_silently_dropped` | GREEN (tightened) | entropy: exit 1, stderr contains `error: --slot @0.entropy=: not supported with --import-json; only --slot @N.phrase= can add a seed to a watch-only cosigner. Get the phrase with: mnemonic convert --from entropy=- --to phrase (it reads the value from stdin and prints "phrase: <words>"), then give the words alone, without the "phrase: " label, on stdin to --slot @0.phrase=-`; ms1: the same with `ms1` in both places; add `--slot @1.fingerprint=97139860`: exit 1, stderr contains `error: --slot @1.fingerprint=: not supported with --import-json; the envelope already supplies every cosigner's key, fingerprint and path (only --slot @N.phrase= is accepted)` | `--slot @0.entropy=...: exit 0 but ms1[0] is the empty watch-only sentinel` |
| `bughunt_tk_52_template_stub_check_not_passed_by_garbage_mk1` | GREEN (tightened) | the experiment: exit 4, `result == "mismatch"`, the `mk1_template_stub_bind` row has `passed: false` and detail `supplied mk1 stub(s) do NOT bind via the template-id stub: an --mk1 value has no readable chunk_set_id (damaged card or not a template stub)`; control unchanged | `verify-bundle reports result=ok with an unparseable extra --mk1 card` |
| `bughunt_tk_54_descriptor_mode_path_family_not_bip87_for_bip48_origin` | GREEN (tightened) | `v["multisig"]["path_family"] == "descriptor"` (rename optional: `…_path_family_is_descriptor`) | `left: String("bip87")` |
| wc-codec `bughunt_tk_07_single_insertion_never_aligned_with_unerased_wrong_positions` | `#[ignore = "deferred-minor: wc-codec-sync-h1-unverified-later-checkpoints"]`, body unchanged | — | `Aligned grid has wrong positions [4, 5, 6, 7, 8, 9, 10, 11] that are not in erasures [0, 1, 2, 3]` |
| wc-codec `bughunt_tk_08_u_slots_8_rejected_or_roundtrips` | GREEN (tightened) | `encode(SourceKind::Mk1Xpub, &[0x5A; 33], 264, &EncodeOpts{u_slots: 8, ..Default::default()})` matches `Err(WcError::InvalidParams)`; control: `u_slots: 7` encodes and `decode` returns the payload | `encode accepted u_slots=8 but the resulting 57-word card does not decode: Some(Sync(AmbiguousRealignment))` |

Counts after the build (scratch copy, rows as today): `-E 'binary(bughunt_s1) | binary(bughunt_s2)'` 28 run, 26 passed, 2 failed (#14, #7) before the `#[ignore]`s; with the ignores and the six new rows: 32 run, 32 passed, 2 skipped (expected; the new rows' outcomes are the 2.x transcripts above).

## 7. Tests that change

**One existing test flips, none else.** On the scratch copy, the whole workspace gives 4148 run, 4111 passed, 37 failed (header table): the 37 are G2's 34 slice rows, the two deferred rows, and:

- `crates/mnemonic-toolkit/src/parse_descriptor.rs` `tests::lex_residue_floor_accepts_multipath_sans_wildcard` → rename `lex_rejects_multipath_sans_wildcard`; it asserts `lex_placeholders("wpkh(@0/<0;1>)")` is `Err(ToolkitError::DescriptorParse(m))` with `m` == the 2.16 text for `@0`. Its comment ("still lex-passes …") is replaced by: "a multipath with no trailing wildcard is refused (FIX_bughunt_tk_g1 #3); bare `@N` is the D1-deferred idiom and still lexes."

The two `--origin` tests the triage's fix-37 broke stay green under this document's #37 (header table; section 2.10).

Greps for other pins of the changed texts, on this worktree (`grep -rln -e <text> crates/mnemonic-toolkit/tests crates/wc-codec/tests`, excluding `bughunt_s*`): `cannot template this descriptor shape` none; `UNVERIFIED: no --expect-wallet-id` none; `--format is not supported` none; `expect-wallet-id hex` none; `explicit @N= assignment cannot combine` none; `your seed (verified)` none; `path_family` in `cli_mode_violations_v0_5.rs` and `cli_gui_schema_conditional_rules.rs` (the `--multisig-path-family` flag rules, template mode, unchanged) and the input fixture `fixtures/wallet_import/envelope_v0_27_0.json` (an import-wallet envelope, not changed); `md1_xpub_match` in `cli_json_envelopes.rs`, `cli_shwpkh_canonical_flip.rs`, `cli_verify_bundle_partial.rs` (genuine cards; green). Goldens and transcripts: `docs/manual/transcripts/41-bundle-inheritance-json.out` (section 5.14); no `tests/` golden changes.

## 8. Build impact

Each item is a line of 9.1 with status `owed-to-build`, closed by its GREEN commit.

1. `cmd/restore.rs`: #1, #2, 11b (restore guard), #10, #38, #40, #41, #42, #47 (`pub(crate)`), #53 (sections 2.1-2.9, 3 item 1).
2. `cmd/verify_bundle.rs`: #12, #13, #37 (shared `ms1_card_checks`), #39, #43, #44, #47, #52 (sections 2.10-2.15, 3 item 2).
3. `cmd/bundle.rs`: #6, #9, #45, #46, #54 (sections 2.19-2.23).
4. `parse_descriptor.rs`: #3, with the unit-test flip of section 7.
5. `synthesize.rs`: #11 / 11b gate and message (2.9).
6. `electrum.rs` #4 and `wallet_export/electrum.rs` #5 (2.17, 2.18).
7. `wc-codec` `pipeline.rs` #8 and the `lib.rs` rustdoc (2.24).
8. Specs: the section 4 edits (five specs).
9. Manual: the 14 edits of section 5 (CLI manual, its `41-bundle-inheritance-json.out` transcript, the technical manual's `MultisigInfo` table).
10. Tests: every row of section 6 (tightened, new, ignored) and the flip of section 7; then `cargo fmt`.
11. `design/FOLLOWUPS.md`: append 9.1, 9.2 and 9.3, in that order, before G2's entries.

**Gate (not a change).** Run every job of `.github/workflows/rust.yml` and `manual.yml` locally or as green CI on the PR head (incl. clippy `-D warnings`, both fmt toolchains, the `manual` job's `make audit` and `ci/doc-flag-lint.test.sh`), `cargo test --locked --doc --workspace`, and the counts of section 6. Then `coordinator/scripts/mutation-gate.sh <worktree> <the commit that adds this document>` before any review. Review class: **B** (restore, verify-bundle, bundle and descriptor code reach keys, addresses and card verdicts) and **A** (manual text): s1 Opus design + executed advice (run section 2's commands, including every printed remedy), s2 Sonnet tests-can-fail on the survivors.

**Release (recommendation, not a build item).** Versions do not change in this PR. At release (toolkit MINOR or PATCH per its convention), CHANGELOG notes: the new refusals (#2 `--format`, #3, #5, #11/11b both ends, #41, #45, #46, #47, #53); #54's JSON value; that a single-sig template card minted by an earlier version at a use-site other than `/<0;1>/*` is now refused by `restore` (it used to print a different wallet). Then 9.3.

## 9. FOLLOWUPS entries (exact text)

### 9.1 Append to `design/FOLLOWUPS.md` (owed to this build)

~~~markdown
### `bughunt-tk-g1-fix` — tk bug-hunt G1 (slices S1 + S2): restore reference checks (#1, #2), single-sig template use-site (#11, 11b), verify-bundle seed card (#37), taproot use-site (#38) and 23 Importants: build obligations (repo: **mnemonic-toolkit**; owning phase: **the build of branch `bughunt/triage`**)

- **Surfaced:** 2026-10-31 Claude API bug hunt (`design/agent-reports/bughunt-2026-10-31-tk.md`), triage `bughunt-triage-tk-s1.md` / `-s2.md`, grades `bughunt-grade-tk-g1.md` (`d78646bd`). FIX: `design/FIX_bughunt_tk_g1.md` (small-fix track; built before `FIX_bughunt_tk_g2.md` in the same PR).
- **Grade:** Critical ×4 (#1, #37, #38, 11b), Important ×23 (#2 #3 #4 #5 #6 #8 #9 #10 #11 #12 #13 #39 #40 #41 #42 #43 #44 #45 #46 #47 #52 #53 #54).
- **Build impact (1):** `cmd/restore.rs` — #1, #2, 11b restore guard, #10, #38, #40, #41, #42, #47, #53: `owed-to-build`.
- **Build impact (2):** `cmd/verify_bundle.rs` — #12, #13, #37, #39, #43, #44, #47, #52: `owed-to-build`.
- **Build impact (3):** `cmd/bundle.rs` — #6, #9, #45, #46, #54: `owed-to-build`.
- **Build impact (4):** `parse_descriptor.rs` — #3, with the `lex_rejects_multipath_sans_wildcard` flip: `owed-to-build`.
- **Build impact (5):** `synthesize.rs` — #11 / 11b template gate and message: `owed-to-build`.
- **Build impact (6):** `electrum.rs` #4 and `wallet_export/electrum.rs` #5: `owed-to-build`.
- **Build impact (7):** `wc-codec` `pipeline.rs` #8 and its `lib.rs` rustdoc: `owed-to-build`.
- **Build impact (8):** the spec edits of FIX §4 (five specs): `owed-to-build`.
- **Build impact (9):** the manual edits of FIX §5 (CLI manual, `41-bundle-inheritance-json.out`, technical manual): `owed-to-build`.
- **Build impact (10):** the RED rows of FIX §6 (tightened, new, ignored) and the FIX §7 flip: `owed-to-build`.
- **Build impact (11):** the FOLLOWUPS entries of FIX §9.2 and §9.3: `owed-to-build`.
- **Status:** `open`; owed to the build.
- **Tier:** `funds-safety`
~~~

### 9.2 Append to `design/FOLLOWUPS.md` (status `deferred-minor`)

~~~markdown
### `verify-bundle-multisig-json-md1-origin-row-on-decode-fail` — verify-bundle multisig `--json` omits `md1_origin_match` when the md1 does not decode, although the check list is documented as fixed (repo: **mnemonic-toolkit**; owning phase: **none — ownerless residue**)

- **Status:** `deferred-minor`
- **Source:** 2026-10-31 bug hunt #14 (and its duplicate #55); triage S1 CONFIRMED; graded Minor in `design/agent-reports/bughunt-grade-tk-g1.md` (`d78646bd`).
- **What:** `emit_multisig_checks`' `Err` arm of the supplied-md1 decode pushes `md1_decode`, `md1_wallet_policy` and `md1_xpub_match` but not `md1_origin_match`, which the `Ok` arm emits on every branch ("`--json` consumers read a FIXED list"). The list is one row shorter when the md1 is damaged or incomplete.
- **Likelihood conditions:** fails closed (exit 4, `md1_decode` names the problem); (a) the md1 does not decode, and (b) a `--json` consumer reads the list by position or length (none in the constellation).
- **Fix (for later):** in that `Err` arm push `md1_origin_match` with `passed: true`, detail and `decode_error` `skipped: md1 decode failed`, like its siblings (`/scratch/cache/bughunt/tk/s1/fix-14.patch`).
- **RED row:** `crates/mnemonic-toolkit/tests/bughunt_s1.rs::bughunt_tk_14_multisig_json_has_md1_origin_row_when_md1_does_not_decode`, `#[ignore = "deferred-minor: verify-bundle-multisig-json-md1-origin-row-on-decode-fail"]`.
- **Tier:** `correctness` (non-gating)

### `wc-codec-sync-h1-unverified-later-checkpoints` — `sync_classify` can return `Aligned` with wrong positions left unerased after a single insertion (repo: **mnemonic-toolkit** `crates/wc-codec`; owning phase: **none — ownerless residue**)

- **Status:** `deferred-minor`
- **Source:** 2026-10-31 bug hunt #7; triage S2 CONFIRMED (contract of the public `sync_classify`); graded Minor in `bughunt-grade-tk-g1.md`.
- **What:** `classify_single_insertion`'s H1 branch (`sync.rs`) accepts the first checkpoint whose marker and index parse after the block, without the CRC and without checking later checkpoints. A data word that looks like a checkpoint yields an `Aligned` grid whose wrong positions are not erased. End to end, RS plus the 44-bit integrity tag refuse (CLI exit 2, "uncorrectable", measured by the triage).
- **Likelihood conditions:** fails closed end to end; (a) a damaged word in block 0, (b) an inserted word, and (c) a data word whose top six bits look like a checkpoint at block 1's first data slot (about 1/64).
- **Fix (for later):** after the trim, check every later checkpoint. Erase a later block whose checkpoint sits at its slot with the right index but fails its CRC; refuse (`AmbiguousRealignment`) only when a later checkpoint is missing or misplaced. Do not use the triage's `fix-07.patch` (it refuses on any later failure, including an insertion plus independent damage in a later block); the follow-up adds a row proving such a card still decodes if it decodes today.
- **RED row:** `crates/wc-codec/tests/bughunt_s2.rs::bughunt_tk_07_single_insertion_never_aligned_with_unerased_wrong_positions`, `#[ignore = "deferred-minor: wc-codec-sync-h1-unverified-later-checkpoints"]`.
- **Tier:** `correctness` (non-gating)

### `manual-gui-82-installer-ahead-text-gaps` — GUI manual §82 "When the installer is ahead of the GUI" and the CHANGELOG entry of `99310923` say less than the release does (repo: **mnemonic-toolkit** `docs/manual-gui` + `CHANGELOG.md`; owning phase: **the next edit of §82's installer-ahead section, or its removal at the GUI's ms 0.20.2 pin**)

- **Status:** `deferred-minor`
- **Source:** `coordinator/agent-reports/review-tk-pin-99310923.md` (scoped review of `99310923`, merged as `df05e813`): M1 and N1-N3, 0 C / 0 I.
- **M1:** `docs/manual-gui/src/80-troubleshooting/82-binary-and-launch.md`, the `ms repair` bullet: "where it used to be reported as a string-length error" holds only for ASCII characters; ms 0.20.1 read a non-ASCII character (U+0133 `ĳ`) as the symbol `3`, printed a repaired UNVERIFIED candidate and exited 4, where 0.20.2 refuses it (exit 1). Likelihood: fails closed; no printed command fails; (a) a non-ASCII character in the repair input and (b) a reader acting on the description of the old release. Fix: replace the bullet's last clause with "A character outside the codex32 alphabet is now named (`error: invalid character 'b' (not in codex32 alphabet)`, exit 1). Before, a letter or symbol outside the alphabet was reported as a string-length error, and a non-ASCII character could be read as an alphabet symbol and repaired."
- **N1:** the same section's `ms combine` bullet describes text mode only; with `--json` ms 0.20.2 prints a JSON object (`"kind":"preimage"` with `entropy_hex`, plus `ms1` for `--to ms1`) and no preimage `warning:` line. Fix: say "in text mode", as `43-ms.md` does.
- **N2:** `CHANGELOG.md` [Unreleased]: the v0.63.0 entry says `[installer-ahead]` and its §82 section "are gone", and the new entry says they "name the new tag" without saying they were re-added. Fix: "are re-added for it".
- **N3:** (outside the diff, ungraded) §82's symptom-matrix row "version differs from the chapter's pinned upstream → Re-install the matching pinned version per `pinned-upstream.toml`" points a holder whose `ms --version` prints 0.20.2 toward 0.20.1 while the new section says 0.20.2 is expected. The row's symptom cannot occur (the banner is a compile-time constant); no wrong result. Fix: add "(unless the installer-ahead section above names this version)".
- **Tier:** `docs` (non-gating)
~~~

### 9.3 Append to `design/FOLLOWUPS.md` (status `open`, release-tied)

~~~markdown
### `manual-gui-transcripts-path-family-descriptor` — at the GUI manual's next toolkit pin bump, five tutorial transcripts change `"path_family":"bip87"` to `"descriptor"` (repo: **mnemonic-toolkit** `docs/manual-gui` + **mnemonic-gui** `tests/snapshots`; owning phase: **the GUI-manual / mnemonic-gui pin bump to the toolkit release carrying FIX_bughunt_tk_g1**)

- **Status:** `open`
- **Source:** `design/FIX_bughunt_tk_g1.md` #54 (2.23). `manual-gui.yml` installs the pinned toolkit release (`mnemonic-toolkit-v0.105.1` today), so these files reflect the release, not this branch, and must not change in the fix PR.
- **What:** `bundle --json` in descriptor mode (incl. `--import-json`) reports `multisig.path_family: "descriptor"` after the fix. Files that carry `"template":"descriptor","path_family":"bip87"` today: `docs/manual-gui/transcripts/tutorial/tut-j2-restore-feed-bundle-json.stdout.txt`, `tut-j3-restore-feed-bundle-json.stdout.txt`, `tut-j4-restore-feed-bundle-json.stdout.txt`, `tut-j4-nums-feed-bundle-json.stdout.txt`, `tut-j5-22-bundle-json.stdout.txt`, and the same five under `mnemonic-gui/tests/snapshots/tutorial/`. At the pin bump, re-run `manual-gui` verify-examples and the GUI tutorial snapshots: the expected drift is exactly this value in these files (and any output FIX_bughunt_tk_g1 §2 changes on a path the tutorial drives); anything else is a finding.
- **Tier:** `docs`, `cross-repo`
~~~

## 10. Overlaps with G2 (`bughunt-grade-tk-g2.md`, `2814a2c0`)

No contradictory ruling: no STOP. The build applies G1 first. Every overlap:

| # | shared surface | G1 ruling | G2 ruling | verdict |
|---|---|---|---|---|
| O1 | `docs/manual/src/40-cli-reference/41-mnemonic.md` | 14 edits (section 5): bundle `--import-json` row, "Non-representable use-site steps", "Template-only md1", "Multisig template completion" mode 3, verify-bundle `--expect-wallet-id` row, export-wallet `--format` row, restore `--from` / `--search-address` / `--expect-xpub` / `--format` rows, restore Scope | import-wallet Refusals rows (#29, #31, #49, #57) and Bitcoin Core paragraph (#57), compare-cost leaf model (#28), silent-payment `--secret` row (#56), inspect `--ms1` row (#51), repair section (X1, X3), the case sentence at old line 3981 | compatible: no sentence in common. G1 adds 23 lines above every G2 anchor (old 1816 → 1839, 3135 → 3158, 3895 → 3918, 3981 → 4004, 4583 → 4606, measured with `difflib` on the scratch copy), so G2's edits must anchor on their quoted text, not line numbers. Build applies G1 first |
| O2 | `design/FOLLOWUPS.md` | appends 9.1-9.3 | appends its deferred entries and re-scopes `toolkit-repair-ms1-stdin-second-line-ignored` | compatible: G1's entries first |
| O3 | the fixed-step refusal text in `parse_descriptor.rs::lex_placeholders` | #3 adds a separate refusal after the residue check; the fixed-step message is byte-identical (2.16, `03c`) | #57 quotes that message as the holder's text for an unmerged Core pair | compatible |
| O4 | `export-wallet --format sparrow` | #5's remedy names `--format sparrow` (run: exit 0) | #20 sets every Sparrow keystore `label` to `Keystore <n>` | compatible: the export still exits 0 with a valid file |
| O5 | `mnemonic convert` | #46's remedy runs `convert --from <subkey>=- --to phrase` (run, 2.22) | #26 drops the false passphrase warning on `--to address`; #25 (deferred) minikey `@env:` | compatible: `--to phrase` from entropy / ms1 / seedqr is not touched by G2 |
| O6 | JSON field `multisig.path_family` | #54: `bundle --json` in descriptor mode reports `"descriptor"` (SPEC v0.3 §5.3) | #19: `import-wallet --json`'s own envelope keeps `"bip87"` with the non-canonical-purpose notice for a BIP-45 Coldcard import (`path_family_from_paths`) | compatible: different producers. Consequence, by both rulings: an `import-wallet` envelope can say `bip87` while `bundle --import-json` of it says `descriptor` |
| O7 | tests and goldens | flips `parse_descriptor::tests::lex_residue_floor_accepts_multipath_sans_wildcard`; RED rows in `bughunt_s1.rs`, `bughunt_s2.rs`, wc-codec `bughunt_s2.rs` | `cli_export_wallet_sparrow.rs` cells 1, 2, 3, 5 and four goldens (#20), `cli_network_fail_open.rs` (#56), `descriptor_builder::gate` test (#28); RED rows in `bughunt_s3/s4/s5.rs` | no file in common |
| O8 | production files | `cmd/restore.rs`, `cmd/verify_bundle.rs`, `cmd/bundle.rs`, `parse_descriptor.rs`, `synthesize.rs`, `electrum.rs`, `wallet_export/electrum.rs`, wc-codec `pipeline.rs` / `lib.rs` | `cmd/convert.rs`, `cmd/inspect.rs`, `cmd/silent_payment.rs`, `cmd/word_card.rs`, `cost/enumerate.rs`, `descriptor_builder/gate.rs`, `repair.rs`, `wallet_export/sparrow.rs`, `wallet_import/{bitcoin_core,bsms_round1,coldcard,coldcard_multisig,sparrow}.rs` (G2's prototype diff) | no file in common |
| O9 | `docs/technical-manual/src/50-rust-api/54-mnemonic-toolkit-api.md` (seen in G2's FIX document, `design/FIX_bughunt_tk_g2.md`, on disk uncommitted) | #54: the `path_family` row of the `MultisigInfo` table (5.13) | #20: the Sparrow keystore-label row and example | compatible: disjoint lines; G1 first |
| O10 | `bundle --import-json` consuming an `import-wallet` envelope (also from G2's FIX document) | #45 / #46: network and `--slot` refusals | #16 / #17: the envelope's `multisig.threshold` becomes K | compatible: `bundle` reads K from the descriptor tree, not the envelope field |

## 11. STOPs

None. No fix adds a flag, a user-visible type or a command, and none changes more than four sentences of a spec (section 4).

Changes from the grade report (`bughunt-grade-tk-g1.md`), found while running the printed advice:

1. **#46 seed-subkey text.** The report's "(get the phrase with: mnemonic convert --from <subkey>=<value> --to phrase)" prints a command the argv guard refuses before parsing (measured: `convert --from entropy=<hex> --to phrase` exits 2, "argument 2 on ARGV (--from entropy=) is raw seed entropy"; the same for `ms1=` and `seedqr=`). The text now says `--from <subkey>=- --to phrase (it reads the value from stdin and prints "phrase: <words>"), then give the words alone, without the "phrase: " label, on stdin to --slot @<N>.phrase=-`; both steps run (2.22).
2. **#41 text.** The report's "or check the placement with --expect-wallet-id" read literally (add the id, keep `--search-address`) is still refused; it now says "or keep them and check the placement with --expect-wallet-id instead of --search-address", and all three branches run (2.5).
3. **Manual 5.3** also corrects the sentence it extends: "The same completion works on verify-bundle via its `--from` …" is wrong for a single-sig template (measured: `verify-bundle … --from phrase=@env:PA …` exits 2, "requires the seed (`--slot @0.phrase=…` …)"); it now names `--slot @0.phrase=`.
4. Every other text (#2, #3, #42, #45, #47, #52, 11b) is the report's verbatim.

Observations for the review (not STOPs):

1. **#2 `--json`.** The template-completion envelope keeps its shape (no `verification` object); under `--allow-mismatch --json` the override is signalled on stderr only. Adding the object would change the template-mode JSON schema, which no ruling asked for.
2. **#38 `sortedmulti_a`.** The grade report allowed a refusal where the faithful arm cannot render; it renders `tr(NUMS, sortedmulti_a(…/<2;3>/*))` correctly (Core-verified), so no refusal is needed and the row asserts the address.
3. **11b and existing cards.** A non-standard single-sig template card minted by the current release is refused by `restore` after the fix. That is the intent (today it prints another wallet; for `/<2;3>/*` even under `✓ wallet-id verified`, because that wallet-id equals the standard wallet's, `1c0170fe…`); the release note says so (section 8). For bare `wpkh(@0)` the minted wallet-id differs (`7774c4fd…`), so a holder who recorded it was already refused (exit 4); one who did not got the wrong wallet.
4. **Multisig verify RED rows** pass ms1 cards on argv (as the triage wrote them); the argv guard accepts `--ms1 <card>` on `verify-bundle` (measured), so the rows run unchanged. The printed commands of this document put phrases and seed values in the environment or stdin.
5. **verify-bundle and 11b.** verify-bundle's single-sig template path needs no use-site guard: it compares the supplied md1 against a re-synthesized standard template, so a non-standard card fails `md1_template_match` (exit 4) today and after.
6. **`docs/manual` lint and verify-examples** show the same pre-existing failures with and without this document (local `mk` drift in `44-mk-cli.md` and the `44-mk-*` / `qs-26` transcripts); CI pins `mk-cli-v0.13.0`, so the build should confirm them green there.
