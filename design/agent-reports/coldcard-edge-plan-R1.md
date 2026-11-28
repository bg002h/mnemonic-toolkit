# R1 review: `design/PLAN_export_wallet_coldcard_edge.md` (plan r1), scoped to the R0 fold

Reviewer: Claude Opus 5.5 (R1, adversarial, fold-scoped). Date 2026-11-28.
Baseline: mnemonic-toolkit `origin/master` d72a18b1 (branch `claude/project-thread-0411jd`). Binary:
`target/debug/mnemonic` 0.107.1 built from this checkout. Firmware: the R0 copies of Coldcard `new_edge` @ `825ab7d`
(scratch `ccfw-r0/ne/`), read as data. The R0 firmware findings and `new_edge` being the live Edge branch were taken as
settled and not re-derived.

**Verdict: RED. 0 Critical, 2 Important, 9 Minor.** Every R0 finding has an answer in r1, and the answers are right
in substance. Row 17's oracle is correct: an independent pure-Python BIP-341/342 implementation reproduces the
toolkit's first address exactly. Rows 1 and 7 behave today as the plan expects. The defects that remain are about where
the checks run. r1 names *what* is checked but not *what input* each check reads or *at which call site* it runs, and
the obvious placements leave holes. Two of those holes are the C2 class again: exit 0 with a file current Edge refuses.

---

## (1) R0 finding → r1 text

| R0 | Answered by | Status |
|---|---|---|
| C1 stale branch | §2 header (`new_edge` @ `825ab7d`, 2026-11-17, v6.6.2X), E1-E13 with file:line, re-check-before-build sentence, `edge` kept as history | Fixed |
| C2 NUMS refused by Edge | §3.2 check 1; rows 4, 5; rows 1, 3, 7 now use `@N`/depth (no NUMS); FOLLOWUP `coldcard-edge-nums-xpub-internal-key`; §4 `33-taproot-multi.md` caveat | Fixed. Restore path open, see **I-B** |
| I1 admission | §3.1 structural `Tr` + one leaf + root `Terminal::MultiA`/`SortedMultiA`; `--from-import-json` stays refused; row 12 negatives (multi-leaf, `and_v`) | Fixed. Implementable: vendored miniscript 13 has `Tr::leaves()`, `TapTreeIterItem::miniscript()`/`depth()`, pub `Miniscript.node`, `Terminal::SortedMultiA` (`vendor/miniscript/src/descriptor/tr/mod.rs:101-110`, `tr/taptree.rs:201-207`, `miniscript/decode.rs:159-161`). Non-taproot envelopes open, see **I-B** |
| I2 zero/missing fp | §3.2 check 2; row 6 | Answered. Input and placement wrong in part, see **I-A** |
| I3 path vs depth | §3.2 check 3; row 7 | Answered. Wrong input and depth-0 gap, see **I-A** |
| I4 duplicates | §3.2 check 4; row 8 | Answered. Reuse caveat, see M-3; E9 attribution, see M-2 |
| I5 oracle | row 17 | Fixed. Verified by execution (below); spec gaps in M-5 |
| I6 restore | §3.1 restore refused + row 13; §4 manual restore list, manual-gui `4d-restore.md`, inventory JSON, GUI mirror covering both subcommands | Answered. Placement unspecified, see **I-B** |
| I7 exit code | §3.2 heading: exit 1 `BadInput` (`error.rs:643`); §4 notes #228 | Fixed |
| I8 name | §3.2 check 7; §3.3 (only an explicit `--wallet-name` selects JSON; stem/suffix warning) | Fixed. Stem rule detail in M-7 |
| M1 count | check 5 (leaf + internal ≤ 32), row 9, hedge dropped | Fixed |
| M2 `--json` | bullet deleted | Fixed |
| M3 fixtures | §4 `cli_gui_schema.rs:289-312`, §3.5 stderr fixture, inventory JSON | Fixed. One more list is missing, see I-A(d) |
| M4 warning | §3.4 four lines (History-Edge quote cited, own-key, testnet, key-path precedence, address check) | Answered. The named address command is wrong, see M-1 |
| M5 bip388 route | §4 manual mention | Fixed. A caveat is needed, see M-8 |
| M6 suffix | check 6, row 11 | Fixed. Reuse caveat in M-3 |
| M7 origin-less | E-table no longer claims Edge needs origins | Mostly. Check 2's origin requirement on `--descriptor` is not labelled toolkit policy (`docs/miniscript.md:24` allows blinded keys). Trivial |
| M8 lockstep | §4: 37-wallet-export, 33-taproot-multi, manual-gui 45, FOLLOWUP cites `new_edge` | Mostly. Jade neighbour omitted, see M-9 |

---

## (3) Executed preconditions (rows 1 and 7)

Fixtures: A = `COSIGNER_A_XPUB` (fp `b8688df1`, depth 4, child `80000002`), B = `COSIGNER_B_XPUB` (`28645006`, depth 4,
child `80000002`), C = `COSIGNER_C_XPUB` (`5436d724`, depth 3, child `80000000`), from `tests/cli_export_wallet*.rs`.
Depths were decoded with a base58 script.

**Row 1.** `--format descriptor`, `@0` internal, paths consistent with depth:

```
$ mnemonic export-wallet --format descriptor --template tr-sortedmulti-a --taproot-internal-key @0 --threshold 2 \
    --network mainnet --slot @0.xpub=$A --slot @0.fingerprint=b8688df1 --slot "@0.path=m/48'/0'/0'/2'" \
    --slot @1.xpub=$B --slot @1.fingerprint=28645006 --slot "@1.path=m/48'/0'/0'/2'" \
    --slot @2.xpub=$C --slot @2.fingerprint=5436d724 --slot "@2.path=m/87'/0'/0'"
tr([b8688df1/48'/0'/0'/2']xpub6FQya7…YHtF8mX/<0;1>/*,sortedmulti_a(2,[28645006/48'/0'/0'/2']xpub6DnEBN…RsYekh6/<0;1>/*,[5436d724/87'/0'/0']xpub6Buxw9…aoyKUYx/<0;1>/*))#89rnhvxk
note: stdout is watch-only — public keys only, cannot spend        (stderr)
exit=0
```

This is what row 1 expects: one line, `#checksum`, `/<0;1>/*` on every key, and `@0` lifted out of the leaf into the
internal position. Note that this wallet is "key path by @0, or 2-of-2 of @1 and @2", not 2-of-3 (see M-6).

**Row 7.** A depth-4 xpub with the default 3-component path (no `.path=`):

```
$ mnemonic export-wallet --format descriptor --template tr-sortedmulti-a --taproot-internal-key @0 --threshold 2 \
    --network mainnet --slot @0.xpub=$A --slot @0.fingerprint=b8688df1 --slot @1.xpub=$B \
    --slot @1.fingerprint=28645006 --slot @2.xpub=$C --slot @2.fingerprint=5436d724
tr([b8688df1/87'/0'/0']xpub6FQya7…/<0;1>/*,sortedmulti_a(2,[28645006/87'/0'/0']xpub6DnEBN…/<0;1>/*,[5436d724/87'/0'/0']xpub6Buxw9…/<0;1>/*))#z45vt39f
exit=0
```

Confirmed. A and B (depth 4) are emitted with 3-component origins, which Edge refuses ("deriv len != xpub depth",
`ne/shared/desc_utils.py:316-317`). Row 7 is RED today as intended.

**Row 17's oracle, executed.** I wrote an independent pure-Python implementation (own secp256k1, BIP-32 CKDpub,
tagged hashes, bech32m; no Bitcoin library). It computes the row-1 wallet's first receive address this way: derive
each key at `/0/0`; sort B and C by 32-byte x-only; build the script
`0x20‖k1 OP_CHECKSIG 0x20‖k2 OP_CHECKSIGADD OP_2 OP_NUMEQUAL`; take the TapLeaf hash with leaf version `0xc0` and
compact-size length; use that hash as the merkle root; tweak with TapTweak(x(P)‖root); lift P to even y; Q = P + tG.
Result: `bc1padu5nee9cqchqdgweks9gnavt8tyfq9n0wkamd3973ktgc3gp2yqw6aga6`. That equals the receive entry of
`export-wallet --format bitcoin-core-addresses --count 1 --descriptor '<row-1 descriptor>'`. The oracle as specified
is correct. It also matches Coldcard's construction: `ne/shared/miniscript.py:821-838` sorts `b"\x20"+xonly`, which
is equivalent to sorting by x-only, and `descriptor.py:51` uses `TAPROOT_LEAF_TAPSCRIPT`, with the single-leaf root
equal to the leaf hash. The script is in `scratchpad/oracle/o.py`. The specification has gaps, listed in M-5.

---

## Important

### I-A. The §3.2 checks read the wrong input. They must run on what Edge receives: the parsed emitted descriptor, inside the emitter

r1 says check 2 reads "the raw `SlotInput`", and leaves the input of checks 3, 4 and 6 unnamed. It does not say
whether the checks run in `cmd/export_wallet.rs::run` or in `wallet_export/coldcard_edge.rs`. Each choice has defects.

(a) **Template-path origin ≠ `ResolvedSlot.path`.** `key_origin_str` (`wallet_export/pipeline.rs:46-62`) replaces an
*empty* slot path with the template fallback. Experiment: `@0.path=m` on A emits `[b8688df1/87'/0'/0']xpub6FQya7…`
with exit 0. Check 3 run on `ResolvedSlot.path` sees length 0. For a depth-0 xpub (a master xpub, as Coldcard's
"Generic JSON" export provides) that check passes, while the file carries a 3-component origin that Edge refuses.

(b) **The depth-0 rule is missing.** Edge also asserts, at depth 0, that the origin fp equals the xpub's *own*
fingerprint ("master xfp mismatch", `ne/shared/desc_utils.py:318-322`). Check 3 lists only the depth-1 parent-fp rule.

(c) **Placement in `run` before the gate breaks the uniform admission gate.** `export_admission_gate` runs at
`cmd/export_wallet.rs:801-804`. `every_format_meets_the_same_gate_before_its_own_verdict`
(`tests/cli_export_wallet_allow.rs:941-971`) asserts that every format meets the gate (exit 2) before its own
verdict. A §3.1 shape refusal placed before the gate (exit 1) would invert that for coldcard-edge.

(d) That test's `FORMATS: [&str; 12]` is a hand list. §4 does not add `coldcard-edge` to it, so the new format would
silently escape the invariant check.

(e) Placement in `run` also misses `run_from_import_json` and restore, which reach `emit_payload` directly (I-B).

The raw `SlotInput` is reachable at the export-wallet call site (`args.slot`, `run` line 514, and export-wallet
slots are xpub-only by `validate_watch_only`, `wallet_export/mod.rs:116-130`). It is not needed, though: an omitted
`.fingerprint=` becomes `00000000` (`cmd/bundle.rs:641-646`), so a non-zero test on the emitted origin catches both
the missing and the zero cases. Only the wording "omitted or zero" changes.

**Fix.** State that §3.1 and §3.2 checks 1-6 run in the `coldcard-edge` emitter (the `emit_payload` arm, after the
admission gate) on `parse_descriptor_lenient(inputs.canonical_descriptor)`. For every key that is a
`DescriptorPublicKey::MultiXPub` with `origin = Some((fp, path))`:
- fp ≠ 0;
- `path.len() == xkey.depth`;
- `path.last() == xkey.child_number` (toolkit policy);
- depth 1: fp == `xkey.parent_fingerprint`;
- depth 0: fp == `xkey.fingerprint()`;
- suffix exactly `/<0;1>/*`;
- pubkeys pairwise distinct;
- count ≤ 32.

The NUMS refusal then falls out of check 1 with no special case, because NUMS is a `Single` key. Keep a raw-`SlotInput`
pre-check only if the plan wants the "missing" wording to name the slot. Add `coldcard-edge` to the `FORMATS` list
(12 → 13) in §4/§5, and add a row `coldcard_edge_refuses_depth0_master_fp_mismatch` (or the `@N.path=m` fallback
case).

### I-B. The restore refusal is not placed where restore dispatches, and `--from-import-json` only refuses taproot envelopes

**Restore.** r1 says "`restore --format coldcard-edge` refused" with one row, `restore_refuses_coldcard_edge`, but
names no location. Restore has four routes, and `--format` reaches `emit_payload` on two of them:
- single-sig: `build_import_payload` at `cmd/restore.rs:665-672` → `:3257`;
- md1 multisig: `run_multisig` at `:375`/`:3882` → `build_multisig_import_payload` at `:4337-4347` → `:3582`.

The md1 dispatch (`:340-375`) runs *before* the existing `--format` gate at `:421`. A refusal added beside that gate,
the natural spot, covers only single-sig. `restore --md1 <NUMS tr-sortedmulti-a card> --format coldcard-edge` would
then reach the emitter. If the checks live in `run` (I-A), that is exit 0 with a static-NUMS descriptor: C2 again.

**`--from-import-json`.** Fix-α (`cmd/export_wallet.rs:961-973`) refuses only `P2tr`/`P2trMulti`. A wsh envelope
with `--format coldcard-edge` passes it and reaches `emit_payload` (`:1112`), skipping anything placed in `run`.

**Fix.**
- Refuse `Some(CliExportFormat::ColdcardEdge)` at the top of `restore::run`, before the `!args.md1.is_empty()`
  dispatch (`cmd/restore.rs:340`), and at the top of `run_from_import_json`, before the envelope is read.
- Also keep the emitter's own shape admission (I-A) as defence in depth.
- Split row 13 into `restore_refuses_coldcard_edge_single_sig`, `restore_refuses_coldcard_edge_md1_multisig` and
  `coldcard_edge_refuses_from_import_json_wsh_envelope` (in addition to the taproot envelope).

---

## Minor

- **M-1. The named address command does not accept this wallet.** `mnemonic addresses` takes only
  `--from xpub=|phrase=|entropy=|seedqr=` (single key). Run on cosigner A's xpub with `--address-type p2tr`, it
  prints `bc1p30w4l0nq…` (A's single-sig address), not the wallet's `bc1padu5…`. Passing the descriptor is refused
  by the argv guard. The command that does work is
  `mnemonic export-wallet --format bitcoin-core-addresses --count 1` with the same inputs: the `"internal": false`
  entry is receive index 0. Verified above. Name that command in warning line 4. The menu is
  `Settings > Miniscript > Import from file` (`ne/docs/miniscript.md:7`), and the address appears under
  Address Explorer > `<name>` (`:16-18`).
- **M-2. Part of E9 / check 4 is toolkit policy, not an Edge rule.** Edge rejects duplicates *within a leaf* and
  conflicting xpub/origin per pubkey (`ne/shared/descriptor.py:141-185`). An internal key repeated in the leaf with
  the same origin passes Edge: the internal key is not in the per-leaf set, and its xpub and origin agree. Refusing it
  is fine, but label it toolkit policy. The same applies to check 3's `child_number` clause, which Edge does not check.
- **M-3. "Reuse" needs care.** `strip_multipath_suffix` is a private `fn` in `pipeline.rs:269` and its text says
  "--format bip388 requires…". `check_bip388_distinct_keys(&[Xpub], &str)` (`bip388.rs:95`) says "--format bip388
  cannot express … keys_info @i", with indices into keys_info, not slot indices. Reuse the logic with coldcard-edge
  wording; do not call the functions. `strip_multipath_suffix` alone accepts `xpub/5/<0;1>/*`. Edge accepts that too
  and derives the same way, so it is harmless, but say whether it is admitted.
- **M-4. The `--descriptor` path has no `@N`.** Warning line 3 ("cosigner @N is the internal key") needs a
  `--descriptor` form, e.g. naming the internal key's `[fp/path]`.
- **M-5. Row 17 has specification gaps.** Each of these could make the test pass vacuously or fail falsely:
  - The leaf version (`0xc0`) and the compact-size script length in the TapLeaf preimage are not stated.
  - K must be a minimal script number (`OP_2`; `push_int`).
  - CHECKSIGADD follows *every* key after the first, which the "…" hides.
  - The tweak input is x(P), with P lifted to even y.
  - The single-leaf merkle root equals the leaf hash. r1 has this right.
  - **The row-1 fixtures are already in sorted order at `/0/0`.** Measured: slot order equals sorted order for B and
    C, so the sort is not exercised. Choose slots or an index where the orders differ, and run the oracle for
    `multi_a` (slot order) as well as `sortedmulti_a`.
- **M-6. Row 1's name says "2-of-3".** With `@0` internal the wallet is "key path by @0, or 2-of-2 of the rest". Name
  it accordingly and pass `--threshold` explicitly. Row fixtures must also satisfy check 3: several existing tests
  give C, which has depth 3, a 4-component path (`cli_export_wallet_coldcard.rs:237,428,730`), so copying them into
  positive rows would fail.
- **M-7. Stem rule.** Edge takes the name as `fn.split('/')[-1].split('.')[0]`, which is up to the *first* dot
  (`ne/shared/wallet.py:1029`), not Rust's `file_stem`, which is up to the last dot. It also requires printable ASCII
  (`utils.py:234-246`). Warn on non-printable-ASCII stems too.
- **M-8. The bip388 route skips the Edge checks.** `--format bip388` runs none of checks 1-3 or 5-7 (it does run
  distinctness and refuses NUMS). The §4 manual mention should say so, or recommend `coldcard-edge` first. Edge also
  requires `name` in a BIP-388 object (`auth.py:1587-1588`); the toolkit emits one (`bip388.rs:192`).
- **M-9. Pointers and the Jade neighbour.**
  - The §3.1 pointer "single-sig → `coldcard`" dead-ends for P2TR single-sig, because `--format coldcard` refuses
    bip86 (`coldcard.rs:122`). Point to `descriptor`/`sparrow` for `tr(K)`.
  - The FOLLOWUP `jade-tr-multi-a-pending-firmware` says "once Coldcard ships, Jade follows"
    (`FOLLOWUPS.md:2462`). Coldcard Edge has now shipped, so add a line saying Edge's support does not imply Jade's.

---

## (2) Other places checked for unchecked acceptance

- All `CliExportFormat` matches are exhaustive: `format_requires_template`, `emission_kind`, the `emit_payload`
  missing-info and dispatch matches (`cmd/export_wallet.rs:95-104,115-121,145-168,236`). The compiler forces a
  decision at each one.
- `emit_payload` has four production callers (`export_wallet.rs:830,1112`; `restore.rs:3257,3582`), covered by I-A
  and I-B. No other subcommand takes `CliExportFormat`.
- The H10 unsorted-multisig guard (`export_wallet.rs:191-205,215-234`) does not list coldcard-edge. That is correct,
  since coldcard-edge refuses wsh by shape.
- `format_requires_template(ColdcardEdge)` matters only on the `--from-import-json` path. With I-B's top-of-function
  refusal it does not matter; say `false`.

## Required before re-review

Fold I-A, which names the input and placement of each check and adds the depth-0 rule and the `FORMATS` list entry.
Fold I-B, which places the restore and `--from-import-json` refusals and splits row 13. The Minors are wording and test
specification. Fold them, but they do not require a re-review.
