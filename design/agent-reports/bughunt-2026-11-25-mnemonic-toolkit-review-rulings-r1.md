# Review: origin rulings of 2026-11-26 (r1)

Scope: mnemonic-toolkit `c78f3980..HEAD` on `claude/project-thread-utm1aa`
(d15b2eff, dafede7a, c8f1a387, 5391e5bb, 73841e8a). Reviewer: independent
adversarial pass (Opus). Source not modified; this report is uncommitted.

## What was executed

- `cargo test --locked -p mnemonic-toolkit --test all`: 2959 passed, 0 failed,
  19 ignored. `--lib`: 1438 passed. `cargo fmt --check` and
  `cargo clippy --locked --all-targets -D warnings`: clean.
- `docs/manual/tests/verify-examples.sh` against the built binary (with the
  Makefile's FIXTURES_DIR): every transcript that needs only `mnemonic` passes,
  including the three changed `41-*` transcripts.
- Probes with the built `target/debug/mnemonic` and the BIP-39 vectors
  (`abandon … about` = 73c5da0a, `legal winner … yellow` = b8688df1), with
  empty-origin md1 cards made by a scratch crate over the vendored md-codec:
  - Item 5: `tr(@0,pk(@1))`, `tr(@0,pk(@0))`, `tr(@0,thresh(2,pk,s:pk))`,
    `sh(and_v(v:pk,pk))`, `sh(thresh(2,pk,s:pk))`, `sh(pk(@0))` refuse
    (exit 2, nothing on stdout); `tr(@0,multi_a(..))`,
    `tr(@0,{pk(@1),multi_a(..)})` (fragment nested in a tap tree) and
    `sh(multi(..))` keep BIP-87 with exit 4; `sh(wsh(and_v(..)))` and
    `wsh(pk(@0))` keep the BIP-48 default with exit 4; `tr(@0)` is a canonical
    fill (86'), exit 4.
  - Item 2: printed commands re-run verbatim (env/stdin secrets) or with the
    placeholders filled in give exit 0 and byte-identical md1/mk1 for: a
    non-canonical wsh with a seed and an xpub slot without fingerprint; a
    canonical wsh(sortedmulti) with seeds (inline rewrite), with one slot
    already inline at another account, and with `--privacy-preserving`;
    `--descriptor=`/`--slot=` `=`-forms with a typed phrase and
    `--passphrase hunter2` (neither echoed); multisig `--template` without
    `--multisig-path-family` (adds `bip87`). `--multisig-path-family bip48`
    exits 0. JSON mode exits 4; `> /dev/full` gives 1 for both bundle and
    restore.
  - Item 3: see I1 and I2 below. Keyless template completion with the old
    single `--origin m/84'/0'/7'` still works (exit 0); `--origin @0=…` there
    is refused. `--output` keeps exit 4 and writes the file.
- `mnemonic gui-schema`: `restore --origin` is still `{"kind":"text"}`; the
  schema carries no repeating attribute, so the output is unchanged by
  `ArgAction::Append`. The GUI follow-up is filed
  (`tk-bh1125-gui-restore-origin-repeating`). mnemonic-gui renders exit 4 as
  an amber VERIFY-ME badge (`app_window.rs:1316`), not as a failure.

## Critical

None.

## Important

### I1. An mk1 `--cosigner` contradicts the asserted path and restore says it "agrees"

`crates/mnemonic-toolkit/src/cmd/restore.rs:4720-4723` (records only
`kc.xpub.depth`, `kc.xpub.child_number`), `check_asserted_origin`
`restore.rs:4141-4170`, `asserted_origin_warning` `restore.rs:4172-4190`.

An mk1 card carries the key's full origin path and (optionally) its master
fingerprint (`vendor/mk-codec/src/key_card.rs:40,46`); its decoded xpub's depth
and child number are *rebuilt from that path*. Restore throws the path away
and compares only its length and last component, so any asserted path of the
same length ending in the same component passes.

Reproduced: 2-of-2 wsh(sortedmulti) md1 with both origins emptied; mk1 for @1
(card path m/48'/0'/0'/2'):

```
restore --md1 … --origin @0=m/48'/0'/0'/2' --origin @1=m/86'/0'/0'/2' --cosigner @1=<mk1 chunks> --json
exit 4
WARNING: md1 cosigner @1's origin m/86'/0'/0'/2' (--origin @1) is asserted by you, not on the card. Its depth and child number agree with the xpub you gave (--cosigner @1). …
JSON: {"origin": "86'/0'/0'/2'", "note": "cross-checked"}  descriptor [b8688df1/86'/0'/0'/2']xpub…
```

The tool held a card stating m/48'/0'/0'/2' and printed a wallet at a path
the card contradicts, telling the user the check *agreed*. A signer cannot
find the key at 86'; the wallet watches but cannot sign until the descriptor
is corrected. The converse also fails the ruling's intent: with the mk1 given
and no `--origin`, restore refuses with "without its seed nothing can be
derived" (`missing_origin_refusal`, `restore.rs:4100-4135`) and asks the user
to type a path that is already on a card in the same command line.

Fix: when the `--cosigner @N` value is an mk1, treat its `origin_path` as a
card-recorded origin for @N (no assertion, no exit 4, after checking
`origin_fingerprint`, when present, against the md1 fingerprint), and refuse an
`--origin @N=` that differs from it (as is done for a seed-found path,
`restore.rs:4670-4686`). Keep the depth/child check for raw xpubs, and add the
xpub's `parent_fingerprint` (it must equal the card fingerprint for a
depth-1 path). Test: the repro above must exit 2 "path-mismatch", and
mk1-without-`--origin` must render the mk1 path.

### I2. Restore's stdout does not say an origin is asserted; JSON can say "verified"

`restore.rs:4757-4770` (notes), `restore.rs:4838-4851` (status), JSON
cosigner objects `restore.rs:4902-4911`, text cosigner lines, and
`build_multisig_import_payload`.

The ruling requires the output to say the path is "asserted by you, not on the
card". Only stderr says it. stdout says the opposite:

- text: `cosigner @1: b8688df1 [48'/0'/0'/2']  from md1 (not independently verified)`
  ("from md1" for a path the md1 does not carry);
- with `--cosigner @0=… --cosigner @1=…` (raw xpubs) and both origins
  asserted: JSON `"verification": {"status": "verified"}`, notes
  `"cross-checked"`, exit 4. The cross-check verified the key bytes, not the
  path;
- `--format sparrow` (and the other export formats) writes the asserted
  `derivation` with no marker; `--output` keeps exit 4 but the file is
  indistinguishable from a verified one.

A GUI or script that reads `--json` (stderr usually discarded) sees a
"verified" wallet at an unverified path; only the bare exit code differs.

Fix: add a per-cosigner `origin_source` (`"card"`, `"searched"`, `"asserted"`;
this also closes `tk-bh1125-r1-m1-restore-json-found-path`), change the note
to e.g. "origin asserted by you, not on the card", and never report
`"verified"` while any origin is asserted (use `"partial"` or a new
`"origin-asserted"`). Test the JSON field and the status, not just stderr.

## Minor

### M1. manual-gui claims the GUI seeds `--multisig-path-family bip87`; it seeds `bip48`

`docs/manual-gui/src/40-mnemonic/42-bundle.md:687`: "The GUI seeds the field
with `bip87`, so a run that keeps it exits 0". mnemonic-gui
`src/form/conditional.rs:165-176` (`template_defaults_for`) seeds `bip48` on a
multisig template, and `src/app_window.rs:258-267` records that the bip87 seed
was removed in v0.16.0. A GUI run with the seeded value records BIP-48, not
the CLI default. Fix the sentence (the exit-0 conclusion still holds, for a
different path family).

### M2. "refused, and nothing was written" is not true for every non-zero bundle exit

`docs/manual/src/40-cli-reference/41-mnemonic.md` new Exit status table, row
`1, 2, 64`. In `bundle_run_unified` and `bundle_run_unified_descriptor`
`emit_unified` runs before `self_check_bundle` (`bundle.rs` ~557-569,
~2225-2234), so a `--self-check` failure exits non-zero after the cards are on
stdout. Say "refused" without "nothing was written", or note the self-check
exception.

### M3. BIP-48 non-multisig shapes and `thresh` k-of-n (owner question)

`template::tree_has_multisig_fragment` (`template.rs:367-381`) and
`check_bip87_default_scope` (`bundle.rs` ~2916-2961) implement the ruling as
written: only the BIP-87 family is scoped. Two consequences Brian may not have
weighed: (a) `wsh(pk(@0))` and `wsh(and_v(v:pk(@0),pk(@1)))` /
`sh(wsh(…))` non-multisig shapes still get the BIP-48 default (exit 4), though
BIP-48 is equally a "Multi-Script Hierarchy for Multi-Sig Wallets";
(b) `sh(thresh(2,pk(@0),s:pk(@1)))` and `tr(@0,thresh(2,…))`, which are k-of-n
multisig policies, are refused because `thresh` is not a "multisig fragment".
Both fail safe (exit 4 / refusal), so no code change is required; record the
reading in the FOLLOWUP or ask.

### M4. Bundle `--json` carries no chosen-origin marker

`bundle.rs` `emit_unified` call sites. A `--json` consumer that discards
stderr learns only "exit 4". Consider a top-level `origin_chosen: [{"slot":
N, "path": "…", "why": "canonical|bip48|bip87"}]` (additive), so the GUI can
show which path to verify. Pairs with I2.

### M5. `--md1-form template` + `--descriptor wpkh(@0…)` exits 4, `--template bip84 --md1-form template` exits 0, same md1

Probe: both give md1 `1c0170fe…` (canonical origin elided). The descriptor run
warns "the cards record it", which is false for the md1 (only the mk1 records
it). Either exempt the template md1 form or word the warning per card.

### M6. FOLLOWUP `tk-bh1125-fable-m1-master-double-warning` is fixed but still open

c8f1a387 deletes `empty_origin_unresolved_warning` and filters
`found_by_seed` out of the unresolved set (`restore.rs:4783-4789`), which
includes a key found at `m`; the contradictory second warning can no longer be
printed. Mark it resolved by c8f1a387 (and make
`tk2_master_key_card_is_found_at_m_with_master_warning` assert the absence).

## Nit

- N1. `restore.rs:4123-4126`: "--md1 cosigner @0, @1 carries NO origin path"
  (plural agreement); the manual quotes it verbatim.
- N2. `single_origin` (`restore.rs:1728-1737`) refuses with `bad()` (exit 1);
  every other `--origin` refusal in this change is exit 2. The manual does not
  state the code.
- N3. Manual restore paragraph: "If you also give `--cosigner @N` … and for the
  master node `m` the key must be the master key of its fingerprint" — the
  master check runs without `--cosigner` too (`check_asserted_origin`,
  `restore.rs:4163`).
- N4. With `--privacy-preserving`, the printed command writes each seed's
  master fingerprint inline (`[73c5da0a/48'/…]@0`) on stderr. The re-run
  reproduces identical cards, so it is harmless to the result; the
  fingerprints are the user's own, but the mode's intent is to keep them off
  the record, so consider saying so or using `--slot @N.path=` where accepted.
- N5. Restore `--template bip84` for a single-key card uses
  `account_primary()`; extra `--account` values are ignored silently.
- N6. Pre-existing, outside the diff (already FOLLOWUP fable-nits n2): a
  concrete `wsh(multi(2,[fp/path]xpubA…,xpubB…))` with one bare key exits 2
  with "internal: synthetic key … not found in key map".
