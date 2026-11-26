# Review: origin rulings of 2026-11-26 (final, after the r1 fold)

Scope: mnemonic-toolkit `c78f3980..HEAD` on `claude/project-thread-utm1aa`
(d15b2eff, dafede7a, c8f1a387, 5391e5bb, 73841e8a, 42be9082, 4449d3ae,
22cff158, c85dee8c). Reviewer: independent adversarial pass (Fable, software
developer lens). Source not modified; this report is uncommitted.

Verdict: 0 Critical, 2 Important, 5 Minor, 6 Nit. The r1 findings I1 and I2
are fixed as described, with one hole in the I1 fix (I-1 below).

## What was executed

- `cargo test --locked --workspace --no-fail-fast`, one run, captured to
  `/tmp/claude-0/-home-user/efb9fedd-ab0a-5019-871b-00d31aeaaeb6/scratchpad/fable-test.log`:
  exit 0, 23 result lines, **4565 passed, 0 failed, 33 ignored**.
- `cargo clippy --locked --all-targets -- -D warnings`: clean (exit 0).
  `cargo fmt --check`: clean (exit 0). No `--release`; nextest not installed.
- `make -C docs/manual verify-examples MNEMONIC_BIN=target/debug/mnemonic`:
  62 transcripts checked, 3 drifted: `24-recover-md1`, `qs-26-recover-md1`
  (`$MD_BIN decode`) and `43-ms-repair-text` (`$MS_BIN repair`). All three run
  only a sibling CLI from the sibling checkout, none runs `mnemonic`; every
  `mnemonic` transcript, including the three changed `41-*` files, matches the
  binary. FOLLOWUP `tk-bh1125-exit4-scripted-bundle-callers` already names
  these as not re-run.
- Real CLI, `target/debug/mnemonic` at HEAD, BIP-39 vectors `abandon … about`
  (73c5da0a) and `legal winner … yellow` (b8688df1), secrets via stdin or
  `@env:`. Empty-origin and master-key md1 cards were made with a scratch crate
  over md-codec at the pinned rev (the CLI itself can no longer mint either:
  a concrete descriptor without `[fp/path]` and `--slot @N.path=m` are both
  refused).

### Bundle: every printed command was re-run

Each printed `mnemonic bundle …` line was executed verbatim (placeholders
filled back in where a secret was typed) and its stdout compared byte for byte
with the exit-4 run.

| Case | First run | Printed command |
|---|---|---|
| `wpkh(@0)` phrase on stdin (canonical fill, inline rewrite) | 4 | exit 0, identical |
| `sh(wpkh(@0))`, `tr(@0)` xpub slot | 4 | exit 0, identical |
| `wsh(sortedmulti)` seed + xpub slot, with and without fingerprint | 4 | exit 0, identical |
| non-canonical `wsh(or_d(...))`, `--descriptor=`/`--slot=` forms, typed phrase and `--passphrase` | 4 | exit 0, identical; neither secret echoed |
| same shape, `[fp]@0` + `@0.entropy=`, testnet, `--account 3` | 4 | exit 0, identical |
| bare `sh(multi)` two seeds | 4 | explicit command exit 0, identical; `m/45'` command exit 0, different cards (as intended) |
| multisig `--template`, no family; one slot with a typed path; xpub slots at 87' | 4 | exit 0, identical |
| `--md1-form template` + `wpkh(@0)` | 4 | exit 0, identical |
| `--descriptor-file` | 4 | names `[73c5da0a/48'/0'/0'/2']@0` to write in the file |
| `--json` | 4 | exit 0; cards identical, the echoed `descriptor` field differs (Nit N3) |
| `--template bip84` | 0 | none |

Item 5 refusals (`tr(@0,pk(@1))`, with one or both origins missing): exit 2,
stdout empty, no example path.

No secret was echoed in any printed bundle or restore command (`--slot
@N.phrase|entropy=`, `--from phrase=|entropy=`, `--from=…`, `--passphrase`,
`--passphrase=…`); `-` and `@env:VAR` are kept.

### Restore without a seed

Right and wrong `--origin`, `--template`, raw xpubs (depth 1, 3, 4), mk1 cards
with and without fingerprint, an mk1 for another key, an mk1 for another
account of the same seed, a seed plus an assertion, `--format sparrow
--output`, a divergent card (one origin on the card, one not), malformed
`--origin` values. All behave as the manual says except I-1 and I-2.

## Critical

None.

## Important

### I-1. `--allow-mismatch` lets an mk1 for a DIFFERENT key supply the origin, labelled "cross-checked; origin from its mk1 card", exit 0

`crates/mnemonic-toolkit/src/cmd/restore.rs:4783` (`mk1_origins.insert` runs
before the key comparison at `:4803`), and the loop at `:4851-4892`, which
reads `mk1_origins` without consulting `verified_positions`. The comment at
`:4840` ("its key already matched to the md1's above") holds only when the
mismatch gate at `:4820` returned; with `--allow-mismatch` it does not.

Reproduction (2-of-2 `wsh(sortedmulti)` md1 with both origins emptied; `MKA`
is A's real mk1 at m/48'/0'/0'/2'; `MKB7` is an mk1 minted from B's seed at
**m/48'/0'/7'/2'**, so same fingerprint b8688df1, different key):

```
mnemonic restore --network mainnet --md1 … \
  --cosigner @0=<MKA chunks> --cosigner @1=<MKB7 chunks> --allow-mismatch --json
exit 0
NOTE: md1 cosigner @1's origin m/48'/0'/7'/2' is read from the mk1 card you gave for it (--cosigner @1); the md1 carries none, and the mk1's key is the md1's key.
✗ MISMATCH (overridden): … the descriptor above is the md1's wallet, NOT what your --from/--cosigner asserted
{"verification":{"status":"overridden","slot":1,…},
 "cosigner_keys":[…,{"fingerprint":"b8688df1","note":"cross-checked; origin from its mk1 card","origin":"48'/0'/7'/2'","origin_source":"mk1","position":1}],
 "descriptor":"wsh(sortedmulti(2,[73c5da0a/48'/0'/0'/2']xpub…,[b8688df1/48'/0'/7'/2']xpub…))"}
```

The md1's @1 key lives at m/48'/0'/0'/2'. The tool wrote 7' in front of it,
taken from a card it had just established is for another key, said on stderr
that "the mk1's key is the md1's key", said on stdout "cross-checked", and
exited 0. The override banner promises "the descriptor above is the md1's
wallet, NOT what your --cosigner asserted", which is false for the origin.
Control: the same command against the md1 with its origins intact keeps
`[b8688df1/48'/0'/0'/2']`. An mk1 with a different fingerprint is caught
(exit 2), so the hole is a card from the same seed (another account, another
script type, a passphrase-less sibling), which is the likely mix-up.

This is the ruling's forbidden outcome (exit 0 with an origin no card for this
key records), reached through the r1 I1 fix. It needs `--allow-mismatch`, hence
Important and not Critical.

Fix: record an mk1's path only after its key matched (move the
`mk1_origins.insert` below `verified_positions.insert(*n)`, or filter the
`:4851` loop on `verified_positions.contains(&c.idx)`). A mismatching mk1 then
supplies nothing, and the key falls into `missing_origin_refusal` (exit 2)
unless the user states a path (exit 4). Do the same for `supplied_headers`: a
header from a key that is not the md1's says nothing about the md1's key, so
`asserted_origin_warning` must not say "the xpub you gave for it has the depth
and child number this path implies". Test: the reproduction must not render
7', and must not print "the mk1's key is the md1's key". Also correct FOLLOWUP
`tk-bh1125-rr1-i1-mk1-origin-ignored`, whose "after its key matched the md1's"
is not true under `--allow-mismatch`.

### I-2. An asserted non-empty path is accepted for a key the card itself proves is the master key

`restore.rs:4157-4202` (`check_asserted_origin`). The master check is one-way:
`:4195` refuses `m` for a key that is not the master of the fingerprint, but
nothing refuses a non-empty path for a key whose own fingerprint (hash160 of
the public key) EQUALS the card's master fingerprint, which proves the key is
the master key and its origin is `m`.

This is not a contrived card. `tests/bughunt_1125.rs:444-446` records that a
master key with an empty origin is "what the old descriptor-mode mint produced
for a bare `@0` + `@0.phrase=`", the card class this whole hunt is about. For
it the refusal text at `restore.rs:4121` advises "name the scheme the card was
made with (--template bip44|bip49|bip84|bip86, with --account)", and the owner
of an old `wpkh(@0)` card will answer bip84.

Reproduction (single-key `wpkh` md1, key = A's master key, origin empty):

```
mnemonic restore --network mainnet --md1 … --template bip84      # or --origin "m/84'/0'/0'"
exit 4
WARNING: md1 cosigner @0's origin m/84'/0'/0' (--template bip84) is asserted by you, not on the card. No --cosigner @0=<xpub> was given, so not even its depth and child number were checked. …
  descriptor: wpkh([73c5da0a/84'/0'/0']xpub6BemYi…/<0;1>/*)
```

The key under that origin is A's master key (`--origin m` on the same card is
accepted and prints `xpub661MyMwAqRbcFkPHuc…`). The printed descriptor tells a
signer the key is at m/84'/0'/0'; it is at m. The addresses are right (they
come from the key bytes), so the wallet watches, and no signer will ever
recognise it. The tool held the proof and printed "not even its depth and
child number were checked". Exit 4 softens this, but the user was steered to
the wrong path by the refusal and nothing on the output contradicts it. Only a
raw master xpub given with `--cosigner` catches it (depth 0 vs 3, exit 2).

Fix: in `check_asserted_origin`, add the converse:
`if !path.is_empty() && xpub.fingerprint() == fingerprint` refuse with
`path-mismatch` ("the key IS the master key of <fp>: its origin is m"). Better,
and it removes the trap entirely: before `missing_origin_refusal`, resolve any
empty-origin key whose own fingerprint equals the card fingerprint to `m`
without an assertion (a new `origin_source`, e.g. `"master"`, with the existing
master-key warning; the depth-0 `[fp]xpub` render is true for such a key, which
was the objection in `tk-bh1125-r1-m2-no-seed-depth0-render`). That is the
owner's call (it adds an exit-0 path where the tool derives the origin from
the card's own bytes); the refusal half is not. Tests: master-key card with
`--template bip84` and with `--origin @0=m/84'/0'/0'` must exit 2.

## Minor

### M-1. The "DO FIRST" example-path refusal is still printed, and this change's own text says it is not

`crates/mnemonic-toolkit/src/cmd/bundle.rs:596-617`
(`check_defaulted_xpub_origin`): "(for example m/48'/0'/0'/2' for a BIP-48
P2WSH cosigner xpub)". FOLLOWUP `tk-bh1125-r3-m1-refusal-example-path` (open,
"DO FIRST", kind=funds; its line reference `bundle.rs:562-567` is stale) is
untouched by this range, while two new doc comments
(`check_bip87_default_scope`, `missing_origin_refusal`) cite it as the reason
they suggest no path. It is reachable in the flows this change sends users
into, including single-sig:

```
mnemonic bundle --network mainnet --template bip84 --slot @0.xpub=<A's m/84'/0'/5' xpub> --slot @0.fingerprint=73c5da0a
exit 2
error: slot @0: no `@0.path=` was given, and the default origin m/84'/0'/0' does not match the xpub's own header (depth 3, child number 5'); … (for example m/48'/0'/0'/2' for a BIP-48 P2WSH cosigner xpub).
```

A typed path is not checked against the header, so copying the example mints
a card with a wrong origin and exit 0. Fix: delete the parenthesis (as the two
new refusals do), in this PR.

### M-2. Restore's exit 4 now has two opposite meanings

`error.rs:693` (`RestoreMismatch` → 4: refused, stdout empty) and
`restore.rs:5214` (`any_asserted` → 4: wallet written). Reproduced: a wrong
`--cosigner` key gives `✗ MISMATCH`, exit 4, nothing on stdout; an asserted
origin gives exit 4 with the full wallet. With `--output FILE` a script cannot
tell them apart by stdout either. The bundle manual has a paragraph for its
analogous self-check collision; the restore section (`41-mnemonic.md`
~1587-1640, ~1730, ~1808) states both exit-4 cases and never says how to tell
them apart. mnemonic-gui shows exit 4 as an amber VERIFY-ME badge in both
cases (r1). Fix: one manual sentence (the `error: restore: … mismatch` line and
`kind: "RestoreMismatch"` versus the "asserted by you" warning), and consider a
FOLLOWUP for a distinct code.

### M-3. stderr says PARTIAL while `--json` says "unverified"

`restore.rs:5191` versus the status at `:4990-4998`. Reproduced: `--cosigner
@1=<mk1>` plus `--origin @0=…` prints `PARTIAL: cross-checked 1/2 cosigners`
on stderr and `"verification":{"status":"unverified"}` on stdout. The manual
documents the JSON side; the text banner should say the same thing (for
example "UNVERIFIED: an origin is asserted by you; cross-checked 1/2 keys").

### M-4. A typed origin is never compared with the xpub's own header on the concrete-descriptor path (pre-existing, outside the diff)

```
mnemonic bundle --network mainnet --json --descriptor "wsh(sortedmulti(2,[73c5da0a/48'/0'/0'/2']<A 48' xpub>/<0;1>/*,[b8688df1/84'/0'/0']<B m/48'/0'/0'/2' xpub>/<0;1>/*))"
exit 0     origin_paths: ["m/48'/0'/0'/2'", "m/84'/0'/0'"]
```

B's xpub header says depth 4, child 2'; the card records a three-step path.
The user typed it, so the ruling's exit 0 is right, but restore now refuses
exactly this disagreement for an asserted path (`check_asserted_origin`) and
bundle refuses it for a defaulted one (`check_defaulted_xpub_origin`). Same
for `--slot @N.path=` beside `@N.xpub=`. File a FOLLOWUP (kind=funds): refuse a
typed origin whose length or last step contradicts the xpub header, at mint.

### M-5. FOLLOWUP accuracy

- `tk-bh1125-rr1-i1-mk1-origin-ignored` (resolved 4449d3ae): see I-1; reopen
  or add the `--allow-mismatch` hole.
- `tk-bh1125-r1-m2-no-seed-depth0-render` (resolved c8f1a387) says the path is
  "checked against … (and, for `m`, the fingerprint)": true one way only, see
  I-2.
- `tk-bh1125-r3-m1-refusal-example-path`: line reference stale, and the
  single-sig template reach above is not in its text.
- The other entries added in this range (`rr1-i2`, `rr1-export-file-asserted-
  marker`, `rr1-mk1-vs-card-origin`, `rr1-m3`, `rr1-m4`, `rr1-m5`, `rr1-nits`,
  `gui-restore-origin-repeating`, `exit4-scripted-bundle-callers`,
  `entropy-slot-path`, `fable-m1`, `fable-m3`, `tr-tree-bip87-scope`) match
  what the binary does.

## Nit

- N1. `restore.rs:4226`: the master node prints as "origin m/ (--origin @0)"
  and `cosigner @0: 73c5da0a []`; the refusal at `:4195` prints "states m/".
  Print `m`.
- N2. `restore.rs:4217-4220`: for an accepted `--origin m` the warning says
  "not even its depth and child number were checked", although the key was
  just proved to be the master key of the fingerprint; and the run exits 4 for
  an origin the card's own bytes confirm. Folds into I-2's fix.
- N3. `bundle --json`: the printed command's output differs from the exit-4
  run in the echoed `descriptor` field (`pkh(@0/…)` versus
  `pkh([73c5da0a/44'/0'/0']@0/…)`) wherever the inline rewrite is used. The
  cards are identical; the manual's "writes the same cards" is exact, but a
  byte comparison of the JSON fails.
- N4. `restore.rs:4023-4040`: `--origin @1=` (empty value, e.g. an unset shell
  variable) is read as the master node `m`. It is refused for a non-master
  key, and accepted for a master one; refuse an empty path text.
- N5. `--origin @9=m/0` and a malformed path exit 1 (`bad()`), other `--origin`
  refusals exit 2 (r1 N2 covers `single_origin` only).
- N6. `docs/manual-gui/src/40-mnemonic/42-bundle.md:596-598` still says the
  field "is seeded to `bip87` on first launch", while the row fixed for r1 M1
  (`:687`) says the GUI seeds `bip48` on a multisig template. Make the two
  agree (pre-existing line, same chapter).

## Settled points checked and not re-opened

- r1 I1: an mk1 whose key matches supplies its path (exit 0, `origin_source:
  "mk1"`); a differing `--origin`, mk1 fingerprint or seed-found path exits 2
  naming both; an mk1 with a different fingerprint is refused even with
  `--allow-mismatch`; a privacy mk1 (no fingerprint) still supplies the path;
  a depth-1 raw xpub's parent fingerprint is checked. Hole: I-1.
- r1 I2: stdout notes, `origin_source`, `"unverified"` status and the
  `--format` stderr warning are all present and correct in every asserted run
  tried.
- A seed-found path (`--from`) exits 0 and is labelled `seed-search`; an
  assertion contradicting it exits 2; an equal one is not an assertion.
- An assertion for a key whose card carries an origin exits 2 ("conflict").
- Exit 4 with a failed stdout write becomes 1 (r1); not re-run.
