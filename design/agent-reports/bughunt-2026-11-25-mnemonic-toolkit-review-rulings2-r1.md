# Review: rulings 2 (BIP-48/87 scope by key count; master key auto-m), r1

- Repo: mnemonic-toolkit, branch `claude/project-thread-utm1aa`
- Range: `dcc0f62a..4eece1af` (42f0b35b, bfd13376, 71c540ec, 4eece1af)
- Reviewer: independent adversarial review (Claude, opus), 2026-11-26
- Rulings checked against: Brian 2026-11-26 (1) multisig = two or more
  distinct keys, BIP-48/87 defaults only for multisig, one-key non-canonical
  tree with an originless @N refused exit 2; (2) restore without a seed
  resolves a provable master key (hash160(pubkey)[..4] == card fingerprint)
  to `m`, origin_source "master", exit 0; paths below m refused.
- Earlier reports (settled, not re-litigated):
  `bughunt-2026-11-25-mnemonic-toolkit-review-rulings-r1.md`,
  `bughunt-2026-11-25-mnemonic-toolkit-review-rulings-fable.md`.

## Result

**0 Critical, 1 Important, 3 Minor, 5 Nit.**

Tests: `cargo test --locked -p mnemonic-toolkit --test all -- bughunt_1125`:
75 passed, 0 failed. The whole `all` binary: 2983 passed, 0 failed, 19
ignored. `--lib`: 1438 passed, 0 failed. (nextest is not installed; no
`--release`.)

The CLI was run end to end with "abandon ... about" (A, fp 73c5da0a) and
"legal winner ... yellow" (B, fp b8688df1). Master-key cards were made with a
scratch helper in the scratchpad (`rv2m`, vendored md-codec), which does what
the test fixture `master_key_card` does: it puts A's master key into @0 of a
`wsh(sortedmulti(2,...))` or `wpkh` card and empties its origin. md-codec
refuses an empty origin on a non-canonical shape at decode ("non-canonical
wrapper requires explicit origin"), so only canonical-shape cards can reach
the master path.

---

## Important

### I-1: A canonical 1-of-1 `multi`/`sortedmulti` still gets the BIP-48 origin by default

- `crates/mnemonic-toolkit/src/cmd/bundle.rs:2921-2923` (the doc comment
  says "Canonical shapes never take a family default (their BIP-48
  canonical fill is only for wsh / sh(wsh) of multi / sortedmulti)").
- `bundle.rs:2870-2891` (`canonical_filled`).
- `vendor/md-codec/src/canonical_origin.rs:58-62, 72-76`.

`check_default_origin_scope` looks only at `bound_paths.defaulted`, which
holds the non-canonical family defaults. The canonical fill
(`canonical_filled`) gives `m/48'/c'/0'/2'` (or `/1'`) to any
`wsh(multi|sortedmulti(...))` or `sh(wsh(...))`, including 1-of-1. That case
has one key, so under ruling 1 it is not a multisig.

Reproduction:

```
mnemonic bundle --network mainnet --descriptor 'wsh(multi(1,@0/**))' --slot @0.phrase=-   # A on stdin
exit=4
warning: no origin was given for @0; the toolkit chose m/48'/0'/0'/2' (this descriptor shape's canonical origin) and the cards record it. ...
mnemonic bundle ... --descriptor 'sh(wsh(sortedmulti(1,@0/<0;1>/*)))' --slot @0.phrase=@env:P1   -> exit 4, m/48'/0'/0'/1'
mnemonic bundle ... --descriptor 'wsh(sortedmulti(1,@0/<0;1>/*))' --slot @0.xpub=<x> --slot @0.fingerprint=...   -> exit 4, m/48'/0'/0'/2'
```

The same one-key multisig in other forms is refused:

- `tr(NUMS,multi_a(1,@0/<0;1>/*))` exits 2 ("this descriptor holds only one
  key. BIP-87 ...").
- `--template wsh-multi --threshold 1` with one slot exits 2 ("requires
  N > 1").

So the result depends on the spelling. The manual (41-mnemonic.md:725-745)
gives `tr(NUMS,multi_a(1,...))` as a single-key example. It says nothing
about the wsh/sh(wsh) 1-of-1, so a reader concludes that every one-key
descriptor is refused.

Funds relevance: this is the same risk class the ruling closes. A one-key
wallet is recorded at a multisig hierarchy nobody chose. Exit 4 and the
warning reduce the risk without removing it.

Fix (owner decision, since this is a canonical shape):

- (a) In Emit mode, refuse the canonical fill when
  `!template::tree_has_multiple_keys(root_tree)`. Use exit 2 with
  `kind:"missing-origin"`, like the non-canonical refusal. This is
  mint-time admission only. Decode, verify-bundle and restore of existing
  1-of-1 canonical cards keep md-codec's canonical meaning (CLAUDE.md
  hazard, "admission versus existing cards").
- (b) Keep the current behaviour and say in the doc comment and the manual
  that the canonical BIP-48 fill applies to a 1-of-1 `multi` too.

Whichever is chosen, add a test row (`wsh(sortedmulti(1,@0/<0;1>/*))`, both
forms) and fix the doc comment at `bundle.rs:2921-2923`.

---

## Minor

### M-1: The master auto-`m` trusts the card's fingerprint field, and its wording covers only a chance match

- `crates/mnemonic-toolkit/src/cmd/restore.rs:3997-4005` (warning),
  `4029-4035` (`is_master_key`), `5000-5022` (selection), `5082-5096`
  (apply).

The comparison uses the right field. `c.fingerprint` is the md1's per-@N
fingerprint TLV (`expand_per_at_n`, restore.rs ~4495-4500), not the wallet
id or policy id. A card without a fingerprint for the key is refused before
this code ("has no fingerprint"), so no `m` is produced from a missing
field. Privacy-preserving bundles still carry md1 fingerprints (checked), so
the auto-`m` behaves normally for them.

The proof is still only as good as the card's fingerprint field. If the
field holds the key's own identifier instead of the seed's master
fingerprint, restore now reports exit 0, labels the key "master", and
re-serialises the key as a depth-0 `xpub661…` at `m`. Before this change
the same input was refused (exit 2). The field can hold that value in two
ways:

- the card was forged;
- it is an old-mint card whose operator typed `--slot @N.fingerprint=` with
  the xpub's own fingerprint, a common confusion.

Reproduction: A's `m/48'/0'/0'/2'` key at @0 of a `wsh(sortedmulti(2,...))`
card, fingerprint field set to that key's own id `abc63537`, origin
emptied. Then `restore --md1 ...` gives:

```
exit=0
descriptor: wsh(sortedmulti(2,[abc63537]xpub661MyMwAqRbcGQnC8zMGw...
cosigner @0: abc63537 [m]  origin m: the key is the master key of its fingerprint; key not independently verified
WARNING: ... so its origin is m (a chance match is about 1 in 2^32) ...
```

Addresses are unaffected, because the pubkey and chain code are unchanged.
The output still names the wrong origin, so a signer cannot find the key.
A cross-check catches it:

- `--cosigner @0=<real depth-4 xpub>` gives exit 2, "has depth 4 ... while
  that path has 0 components".
- `--from` with A gives exit 4, fingerprint MISMATCH.

This does not open a new attack: a forged card could carry any wrong
explicit origin. The new part is the stronger label ("the key is the master
key", exit 0) and the "1 in 2^32" text, which speaks only to chance. The
ruling accepted this.

Fix (wording only):

- Qualify the claim as "if the card's fingerprint is the seed's master
  fingerprint". Alternatively, make the master line in the UNVERIFIED banner
  ask for `--cosigner @N=<xpub>` (depth 0) or `--from` to confirm.
- Optionally mention the typed-own-fingerprint mistake in 41-mnemonic.md
  ~1740.

### M-2: `restore --format coldcard` on a master-key card gives advice restore cannot follow

- `crates/mnemonic-toolkit/src/wallet_export/coldcard.rs:348`

The auto-`m` now reaches the Coldcard refusal without any user statement
(the same thing happened with `--origin @0=m` before this change):

```
error: --format coldcard multisig text: at least one cosigner has an empty origin derivation path — a faithful per-cosigner export is not possible. Supply each cosigner's origin path (`--slot @N.path=m/...`) ...
```

`restore` has no `--slot`, and for a proven master key `m` is the only
correct path. Fix: when the caller is restore and the key is the master,
say that Coldcard's format cannot express a wallet on the master key, and
name no flag.

### M-3: The seed-found-`m` refusal prints `m/` and prints its warning before the refusal

- `crates/mnemonic-toolkit/src/cmd/restore.rs:4801-4811` and `4967`, `4989`

The N1 fix (`origin_text`) did not reach these sites. Reproduction on the
`wpkh` master card, with A on stdin:

```
restore --md1 ... --from phrase=- --origin "m/84'/0'/0'"
exit=2
WARNING: md1 cosigner @0 carries NO origin path on the card. Its key was FOUND by searching ... at m. ...
error: --origin @0 states m/84'/0'/0' for md1 cosigner @0, but the --from seed's key is at m/; remove one of them.
```

The same `m/{}` with an empty path occurs in:

- `4967`: "the --from seed's key is at m/{}" (mk1 path versus a seed that
  found `m`);
- `4989`: "{source} states m/{path}" (an asserted `m` versus an mk1 path,
  for a non-master key).

The WARNING about a wallet that is then not shown goes against the
"check before warning" rule stated at restore.rs ~5034. Fix: use
`origin_text` at all three sites, and run the seed/asserted comparison
before the found-path warning is printed.

---

## Nit

- **N-1** `restore.rs:5053-5056`. The master key's source string reads "the
  card (the key's own fingerprint is the card's) states m for md1 cosigner
  @0, but the xpub you gave ...". The card states no origin. Suggested: "the
  card proves @0 to be the master key (origin m), but ...".
- **N-2** `template.rs:372-399` has no unit test. Integration rows cover
  NUMS internal key, `or_d`, `thresh` and taproot leaves. Nothing covers
  `InternalKey::LianaUnspendable` (not counted), a slot internal key plus
  one leaf of the same index (`tr(@0,pk(@0/<2;3>/*))`; the parser refuses
  it today), or repeated indices in `MultiKeys`. The match is exhaustive,
  so a new `Body` variant fails to compile; that part is good.
- **N-3** "two or more `@N`" is the test, but two `@N` can hold the same
  concrete xpub at two typed paths (`--slot @0.xpub=X --slot @0.path=m/84'...
  --slot @1.xpub=X --slot @1.path=m/48'...` exits 0). With one of them
  defaulted, the other one's BIP-48 default would apply to what is a single
  key. This is reachable only through a typed path that contradicts the
  xpub header, which is `tk-bh1125-rf-m4-typed-origin-header-unchecked`
  territory. When all slots are defaulted, the distinct-key check
  ("identical (xpub, path)") refuses it. Note only.
- **N-4** `xpub-search account-of-descriptor` on a one-key descriptor prints
  `info: ... defaulting origin path for @0 to m/48'/0'/0'/2' (BIP-48
  cosigner path)` and then reports the real match at `m/84'/0'/0'`. This is
  documented as intended (41-mnemonic.md ~4607, manual-gui 4d-xpub-search).
  The info line still names a multisig hierarchy for a one-key descriptor;
  consider dropping the notice when `!tree_has_multiple_keys`.
- **N-5** verify-bundle still rebuilds a one-key card at the family default
  (exit 0, `info:` line). This is intentional, documented (41-mnemonic.md
  ~741-745) and tested
  (`bip48_scope_verify_bundle_still_rebuilds_a_pre_ruling_single_key_card`).
  The match proves only what the card records, and that the operator never
  stated the path. Recorded as accepted.

---

## Checked, no finding

**Key counting (`tree_has_multiple_keys`).** Probed through the CLI:

- Refused as one key (exit 2): `wsh(thresh(1,pk(@0),snl:after))`,
  `wsh(and_v(v:pk(@0),sha256(H)))` (hashlock),
  `tr(NUMS,pk(@0))`, `tr(NUMS,multi_a(1,@0))`.
- Multisig, exit 4 with the warning and an explicit-path command:
  `wsh(thresh(2,pk(@0),s:pk(@1),snl:after))`,
  `wsh(thresh(1,pk(@0),s:pk(@1)))`, `wsh(or_d(pk(@0),and_v(v:pk(@1),older)))`,
  `tr(@0,pk(@1))`, `tr(NUMS,{pk(@0),pk(@1)})`.
- The parser refuses the same @N with different suffixes ("inconsistent
  path"), and the same concrete key under two defaulted @N (BIP-388
  distinct-key).
- A raw x-only internal key other than NUMS fails earlier (internal error,
  pre-existing, unrelated).
- `bundle` descriptor mode lifts concrete xpubs into @N slots, so mixing is
  counted correctly.

**Other places a default origin could reach a one-key tree.**

- Template mode: multisig templates require N > 1.
- `--import-json`: goes through `bundle_run_unified_descriptor`, so the
  same check applies.
- `export-wallet --descriptor`: concrete passthrough, no default added.
- verify-bundle and xpub-search: see N-4 and N-5.
- restore: never assigns BIP-48/87.

**Master resolve.**

- Compares against the per-@N card fingerprint; a missing fingerprint is
  refused.
- `--origin @N=m` and `--origin m` on a proven master give "master", exit 0.
- Any path below m (`--origin`, `--template bip84`, an mk1 with a path) is
  refused, exit 2.
- `--cosigner` with the depth-0 master xpub gives "verified"; a depth-4
  xpub is refused.
- `--from` gives "seed-search", the same `m`, the same warning and the same
  descriptor.
- In a mixed multisig, only the other key is refused, and the command it
  prints names only @1.
- The master warning is on stderr for every `--format` (sparrow,
  bitcoin-core, bsms, bip388, descriptor, electrum); Sparrow writes
  `"derivation": "m"`.
- No path mints a master-key card: bundle refuses `@N.path=m` on canonical
  shapes, and md-codec refuses an empty origin on non-canonical shapes.

**`--origin` exit codes (N4, N5 of rf-nits).** All refusals exit 2. An empty
value is refused, both for `@N=` and for a single-key `""`.

**Manual and FOLLOWUPS.** The 41-mnemonic.md, manual-gui 42-bundle,
4d-restore and 4d-xpub-search text matches the observed behaviour, except
for the I-1 omission. FOLLOWUPS entries are resolved with the correct
commits. The rf-nits entry keeps N3 and N6 open, correctly.
