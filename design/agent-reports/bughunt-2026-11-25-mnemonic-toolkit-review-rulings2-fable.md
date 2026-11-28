# Review: rulings 2 (BIP-48/87 scope by key count; master key auto-m), final (fable)

- Repo: mnemonic-toolkit, branch `claude/project-thread-utm1aa`
- Range: `dcc0f62a..HEAD` (42f0b35b, bfd13376, 71c540ec, 4eece1af; fold
  1fe511e1, f6878fd1, ba26517f, dd16174b, f2472f51)
- Reviewer: final independent adversarial review, software developer lens
  (Claude, fable), 2026-11-26. No source was modified.
- Rulings checked (Brian, 2026-11-26): (1) multisig = two or more distinct
  keys; BIP-48 (including the canonical wsh / sh(wsh) multi fill) and BIP-87
  defaults only for multisig; a one-key tree with an originless @N refused at
  mint (exit 2, missing-origin, no example path); defaulted multisig trees
  keep exit 4 plus a command stating the path; existing cards keep decoding
  and verifying. (2) `restore` without a seed resolves an empty-origin key
  whose own fingerprint is the card's to `m` (origin_source "master",
  warning, exit 0); paths below m refused.
- Settled, not re-litigated:
  `bughunt-2026-11-25-mnemonic-toolkit-review-rulings2-r1.md`.

## Result

**0 Critical, 0 Important, 1 Minor, 6 Nit.**

No path was found that gives a BIP-48 or BIP-87 origin to a one-key wallet at
mint, refuses a real multisig, exits 0 after the tool chose an origin, or
labels a key "master" when the card does not prove it.

## Gates

| Gate | Result |
|---|---|
| `cargo test --locked --workspace --no-fail-fast` (once; log `/tmp/claude-0/-home-user/efb9fedd-ab0a-5019-871b-00d31aeaaeb6/scratchpad/fable2-test.log`) | exit 0. lib 1443 passed, 0 failed, 5 ignored; `all` 2990 passed, 0 failed, 19 ignored; every other binary 0 failed |
| `cargo clippy --locked --all-targets -- -D warnings` | exit 0, no warnings |
| `cargo fmt --check` | exit 0 |
| `make -C docs/manual lint` | not run (needs the sibling `md` / `ms` / `mk` binaries) |

nextest is not installed; no `--release`.

## How the binary was exercised

`target/debug/mnemonic` (0.107.1), phrase A "abandon ... about" (73c5da0a) on
stdin, phrase B "legal winner ... yellow" (b8688df1) by `@env:`. Master-key
and empty-origin cards were made with the scratch helper `rv2m` (vendored
md-codec; it replaces a card's key, fingerprint or origin), as r1 did.

Bundle, one key, no origin: all exit 2 with the one-key refusal.
`wsh(pk)`, `wsh(and_v(v:pk,older))`, `sh(wsh(and_v))`, bare `sh(and_v)`,
`tr(NUMS,pk)`, `tr(NUMS,and_v(v:pk,after))`, `tr(NUMS,multi_a(1,@0))`,
`tr(@0,pk(@0))`, `wsh(or_d(pk(@0),and_v(v:pk(@0),older)))`,
`wsh(thresh(1,pk,sln:after,sln:older))`, `wsh(thresh(2,pk,sln:after,sln:older))`,
`wsh(multi(1,@0))` (also `@0/**`), `wsh(sortedmulti(1,@0))`,
`sh(wsh(multi(1,@0)))`, `sh(multi(1,@0))`, `sh(sortedmulti(1,@0))`; with
phrase, entropy, ms1, SeedQR and xpub slots; with `--account 2`.

Bundle, single-sig canonical (`pkh`, `wpkh`, `sh(wpkh)`, key-path `tr(@0)`):
canonical BIP-44/49/84/86 fill, exit 4, printed command re-mints identical
cards (exit 0).

Bundle, two keys, no origin: all exit 4 with the warning and a command; each
printed command was run and gave byte-identical cards at exit 0.
`wsh(or_d(pk(@0),and_v(v:pk(@1),older)))`, `wsh(thresh(2,pk,s:pk,sln:after))`,
`wsh(thresh(1,pk,s:pk))` (testnet, `--account 3`), `sh(wsh(or_d))` (ms1 slot),
bare `sh(or_d)` and `sh(multi(2))` (BIP-87 command identical; the m/45'
alternative differs, as it says), `tr(NUMS,{pk(@0),pk(@1)})`, `tr(@0,pk(@1))`
(phrase, entropy and xpub slots), `wsh(multi(2))`, a partial inline
`wsh(sortedmulti(2,[fp/48'/0'/0'/2']@0,@1))`, template `sh-wsh-sortedmulti`.
Redacted commands (ms1, entropy) re-mint identically once the secret is put
back.

Refusal advice followed: `--slot @0.path=` (non-canonical phrase and xpub
slots, canonical xpub slot) and inline `[fp/path]@0` (phrase, entropy, and
the canonical 1-of-1 seed slots) all mint at exit 0 with the typed origin on
the card. The canonical 1-of-1 `[fp]@0` refusal no longer suggests dropping
the brackets. One exception is Minor M-1 below.

Other mint routes: template multisig with one slot refused (N > 1);
`import-wallet` and concrete `--descriptor` require origins;
`--import-json` cross-checks the envelope's mk1 fingerprint against the
descriptor, so the mk1 own-fingerprint fallback
(`wallet_import/json_envelope.rs:391`) cannot reach a card;
`export-wallet --descriptor` refuses `@N`; xpub slots without
`@N.fingerprint=` record `00000000`, never the key's own fingerprint.

Existing cards: a 1-of-1 `wsh(sortedmulti(1,...))` card at `m/48'/0'/0'/2'`
and a one-key `wsh(and_v)` card at the old default still pass `verify-bundle`
with their original (originless) command lines (exit 0, `info:` line) and
restore unchanged.

Restore without a seed:

| Card | Result |
|---|---|
| `wsh(sortedmulti(2))`, @0 master | exit 0, `[m]`, origin_source "master", warning, UNVERIFIED banner |
| both keys master | exit 0, both "master" |
| `wpkh` master; `wsh(sortedmulti(1))` master | exit 0, "master" |
| same plus `--origin m` / `@0=m` | exit 0, "master" |
| same plus `--origin m/84'/0'/0'`, `--template bip84`, `--origin @1=m/1` | exit 2, "is the master key of ..." |
| plus `--cosigner @0=<depth-0 xpub>` | exit 0, "cross-checked", status verified |
| plus `--cosigner @0=<other xpub>` | exit 4 MISMATCH |
| plus `--from A` | exit 0, "seed-search", same descriptor |
| @0 master, @1 empty and not master | exit 2 naming only @1; the printed command with `@1=m/48'/0'/0'/2'` gives exit 4, @0 "master", @1 "asserted" |
| @0 empty, not master (multisig, wpkh, 1-of-1) | exit 2; `--origin @0=m` exit 2 ("not the master key"); a stated path exit 4 "asserted" |
| forged fingerprint (r1 M-1) | exit 0 with the new wording; `--cosigner` real xpub exit 2; `--from` exit 4 |
| no-fingerprint xpub-slot card, origin emptied | exit 2 (fp 00000000 is not the key's) |
| `--format` sparrow, bitcoin-core, bsms, bip388, descriptor, electrum | exit 0, origin `m`, warning on stderr |
| `--format coldcard`, `jade` | exit 1, new message, no `--slot` |

## Fold of r1

- **I-1** fixed (f6878fd1). `canonical_fill_is_bip48` plus the shared scope
  check refuse the canonical fill for fewer than two keys; single-sig
  canonical fills and two-key canonical fills are unchanged; seed-slot and
  entropy-slot advice is correct. No new defect found.
- **M-1** wording fixed (ba26517f): warning, cosigner note and manual all say
  the proof rests on the card's fingerprint and ask for a cross-check.
- **M-2** fixed (ba26517f). See Nit N-4 for the doc comment.
- **M-3** fixed for the case reported. One sibling case remains, Nit N-3.
- **N-2, N-4** fixed (dd16174b). **N-1** still open and recorded.

---

## Critical

None.

## Important

None.

## Minor

### M-1: An `@N.xprv=` slot on a one-key descriptor gets advice that cannot be followed

- `crates/mnemonic-toolkit/src/cmd/bundle.rs:1790` (scope check) runs before
  `bundle.rs:2047` ("subkey set [\"xprv\"] not supported in descriptor mode").

Reproduction (XPRV = A's master xprv):

```
mnemonic bundle --network mainnet --descriptor 'wsh(and_v(v:pk(@0/<0;1>/*),older(144)))' --slot @0.xprv=$XPRV --allow-argv-secret
  exit 2: no origin was given for @0 ... Give each key's real origin with --slot @N.path=m/... or inline [fp/path]@N
... --slot @0.xprv=$XPRV --slot "@0.path=m/84'/0'/0'"
  exit 2: slot @0 has both secret-bearing input and watch-only input; pick one per slot.
... --descriptor "wsh(multi(1,[73c5da0a/84'/0'/0']@0/<0;1>/*))" --slot @0.xprv=$XPRV
  exit 1: --slot @0 subkey set ["xprv"] not supported in descriptor mode in v0.4.2
```

Both remedies the refusal names are refused; the real answer (an xprv slot is
not supported in descriptor mode) appears only after the user has followed
one. Nothing is minted, so there is no funds risk. Before 42f0b35b a `wsh`
one-key xprv slot reached the "not supported" error directly.

Fix: in `check_default_origin_scope`, skip (or word differently) slots whose
subkey set descriptor mode does not support, so the "not supported" error is
the first one; or run the unsupported-subkey check before the scope check.

## Nit

- **N-1** `restore.rs:4128`. A malformed `--origin` path prints a doubled
  prefix and drops the `@N=`:
  `restore ... --origin '@0=m/84x'` gives
  `error: error: --origin m/84x: invalid child number format` (exit 2).
  `malformed(e.to_string())` wraps a `ToolkitError` whose `Display`
  (`error.rs:1054`) already starts with `error: `. Introduced by 71c540ec
  (rf-nits N5). Fix: wrap `e.message()`, or build the text from `v`.
- **N-2** `restore.rs:1742` (`parse_explicit_own_origin`). Leading `m`s are
  stripped greedily, so `--origin mm` reads as `m` (exit 0 on a master-key
  card) and `--origin "m/m/84'/0'/0'"` reads as `m/84'/0'/0'` (exit 4,
  asserted). The N4 fix refuses only an empty text. No wrong origin results
  (a non-master key still refuses `m`), but a typo is accepted. Predates the
  range; the same idiom is at `restore.rs:976`, `verify_bundle.rs:785, 1062`,
  `import_wallet.rs:2120`. Fix: strip one `m` or `m/` prefix only.
- **N-3** `restore.rs:4781`. The found-path WARNING is still printed before
  a refusal about another key. Card with @0 master and @1 empty, not master,
  `--from A`: stderr shows `WARNING: md1 cosigner @0 ... FOUND ... at m ...`
  and then `error: --md1 cosigner @1 carries NO origin path ...` (exit 2).
  The mk1-versus-seed comparison (`restore.rs:4991`) also runs after the
  warning. r1 M-3 fixed only the asserted-versus-found case; FOLLOWUPS
  `tk-bh1125-rr2-m3-seed-found-m-text` describes that case accurately. The
  found path is useful to the user here, so this may be left as it is.
- **N-4** `wallet_export/coldcard.rs:255-262`. The new const was inserted
  between `emit_coldcard_multisig_text` and its doc comment, so the
  function's four doc lines now document `EMPTY_ORIGIN_REFUSAL_PREFIX` and
  the function has none. The refusal at `coldcard.rs:352` repeats the text
  as a literal instead of using the const; the test `rulings2_m2_*` would
  catch a drift end to end. Fix: move the const above the doc comment and
  build the message from it.
- **N-5** r1 N-1 is unchanged: with a forged card and the real xpub, the
  refusal reads "the card (the key's own fingerprint is the card's) states m
  for md1 cosigner @0, but the xpub you gave ..." (`restore.rs:5086`).
  Recorded open in `tk-bh1125-rr2-nits`.
- **N-6** Outside the range, noted while probing:
  - `restore --md1` of a key-path `tr(@0)` card exits 1 ("keypath-only tr is
    single-sig, not multisig") with or without an origin, so the master-key
    rule cannot apply to that shape. The manual's master-key paragraph
    (41-mnemonic.md, restore `--origin`) does not say so.
  - `bundle --descriptor` with one annotated and one bare concrete xpub
    exits 2 with `error: internal: synthetic key ... not found in key map`.
    Same output from the 2026-11-25 binary.
  - `--json` gives `"origin": ""` for a master key while the text shows
    `m`; documented in the manual, so consumers must read `origin_source`.

## Checked, no finding

- **Key counting.** Distinct `@N` across key slots, multi lists, taproot
  internal key and leaves; NUMS and Liana-unspendable are not keys. The
  parser refuses one `@N` with two suffixes, and the distinct-key check
  refuses one seed in two defaulted slots.
- **Canonical 1-of-1.** Refused for phrase, entropy, SeedQR, ms1 and xpub
  slots, `/**` and `<0;1>/*`, `wsh` and `sh(wsh)`. A typed canonical path
  (`[fp/48'/0'/0'/2']@0`) and any other typed path mint at exit 0. A wrong
  inline fingerprint for a phrase slot is refused.
- **Exit codes.** Every tool-chosen origin at mint exits 4 (text and
  `--json`); every explicitly stated origin exits 0. An xpub slot whose
  header contradicts the default is refused (exit 2), not defaulted.
- **Master label.** Applied only to an empty-origin key whose own
  fingerprint is the card's per-@N fingerprint, with no stated path below m.
  A card-origin key, a seed-found key ("seed-search") and an asserted key
  keep their labels. Testnet renders `tpub` at depth 0.
- **xpub-search.** Notice printed for two distinct keys, not for one; the
  search results are the same.
- **Manual, manual-gui, FOLLOWUPS.** The bundle refusal text, the one-key
  examples (`tr(NUMS,...)` and `@0/**` are accepted spellings), the
  canonical 1-of-1 paragraph, the restore master-key paragraph (exit codes,
  `origin_source`, Coldcard and Jade exit 1, depth-0 cosigner), the
  `--origin` refusals (empty, twice, unknown `@N`, all exit 2) and the
  xpub-search notice match the binary. FOLLOWUPS entries name the right
  commits and states; `tk-bh1125-rr2-m1-master-auto-m-trusts-card-fp` and
  `tk-bh1125-rr2-nits` are correctly left open.
