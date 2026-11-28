# PLAN: `export-wallet --format coldcard-edge` (taproot multisig for Coldcard Edge)

Status: plan r2 (author Claude Opus 5.5, 2026-11-28). r1 folds R0
(`agent-reports/coldcard-edge-plan-R0.md`: 2C/8I/8M, every item); r2 folds R1
(`agent-reports/coldcard-edge-plan-R1.md`: 0C/2I/9M, every item). Baseline:
mnemonic-toolkit `origin/master` d72a18b1. Requested by Brian, 2026-11-28 (cards
"Add an Edge descriptor export", then "Build as coldcard-edge, warn").

## 1. Why

`--format coldcard` / `coldcard-multisig` refuse `tr-multi-a` / `tr-sortedmulti-a`
(`wallet_export/coldcard.rs::emit_coldcard_multisig_text`, FOLLOWUP
`coldcard-tr-multi-a-pending-firmware`). Right for mainline firmware, which has no
`tr()` parser (master `shared/descriptor.py:258-259` only sniffs the prefix).
Coldcard's experimental **Edge** firmware imports taproot multisig as a BIP-380
descriptor. Today a user can only get there by passing `--format descriptor`,
with no Edge-specific checks, so several inputs exit 0 with a file Edge refuses
(R0 C2, I2-I4, I8).

## 2. External facts (authoritative: Coldcard/firmware `new_edge` @ `825ab7d`, 2026-11-17, v6.6.2X)

The old `edge` branch stopped at `158c8a7` (2024-03-18, 6.2.2X) and is history
only (R0 C1). Re-check the branch and its head before the build: a GREEN
expires when its inputs move. Copies: session scratch `ccfw-r0/ne/`.

| # | Fact | Source |
|---|---|---|
| E1 | `<0;1>/*` multipath accepted; `/0/*` silently normalized to `<0;1>/*`; `/1/*`, non-ranged and hardened suffixes refused | `shared/desc_utils.py:168,174-218` |
| E2 | `#checksum` optional, verified when present | `shared/descriptor.py:415-441` |
| E3 | Static (bare hex) internal keys refused since 6.3.5X: "only extended pubkeys allowed". Edge's NUMS form is an xpub whose pubkey is H with a random chain code: a different wallet | `shared/desc_utils.py:290-291`; `releases/History-Edge.md:162-165`; `docs/taproot.md:28-53` |
| E4 | `{"name","desc"}` JSON accepted; `name` overrides the filename; BIP-388 `{name,desc_template,keys_info}` also accepted (≥ 6.4.0X) | `shared/auth.py:1585-1603`; `History-Edge.md:126-127` |
| E5 | `multi_a` and `sortedmulti_a` both accepted | `shared/miniscript.py:816-850` |
| E6 | tpub accepted only when the device is in testnet mode | `shared/desc_utils.py:297`; `shared/chains.py:367-372,501-507` |
| E7 | Key cap: `MAX_TR_SIGNERS = 34`, counted over all keys incl. the internal key; docs say 32 | `shared/descriptor.py:136-138,194`; `docs/taproot.md:60-62` |
| E8 | Origin derivation length must equal xpub depth; at depth 1 the origin fp must equal the parent fp | `shared/desc_utils.py:314-342` |
| E9 | No duplicate keys; no pubkey with a conflicting chain code or origin | `shared/descriptor.py:146,170-185` |
| E10 | The device's own xfp must be among the keys ("My key … missing") | `shared/descriptor.py:196` |
| E11 | Name: 1-30 printable ASCII; a plain file takes its name from the filename stem; picker wants `.txt`/`.json` | `shared/wallet.py:29,521-523,1016,1030` |
| E12 | Key path has precedence when Coldcard can sign either way | `docs/taproot.md:64` |
| E13 | Current Edge warning text: `releases/EdgeChangeLog.md:3-10`; the "DO NOT use for large Bitcoin amounts" sentence survives in `History-Edge.md:7` | as cited |

Mainline (master) has no taproot multisig import (R0 table, last row).

## 3. Behaviour

New value `coldcard-edge` in `CliExportFormat`; new emitter
`wallet_export/coldcard_edge.rs`.

### 3.1 Admission (structural, not substring)

- `--template tr-multi-a | tr-sortedmulti-a` with `--taproot-internal-key @N`.
- `--descriptor`: admitted only if the parsed `Descriptor::Tr` has an internal key
  plus **exactly one** leaf whose miniscript node is `Terminal::MultiA` or
  `Terminal::SortedMultiA` at the root (no wrappers, no `and_v`, no tree).
  Sorted/unsorted comes from the node. Not via `template_from_descriptor`
  (refuses every `Tr`, `wallet_export/mod.rs:289-291`) nor the `multi_a(`
  substring classifier (`mod.rs:234-246`).
- `--from-import-json`: refused for **every** envelope under `coldcard-edge`, at
  the top of `run_from_import_json` (the Fix-α taproot refusal at
  `cmd/export_wallet.rs:961-973` alone would let a wsh envelope reach
  `emit_payload` at `:1112`).
- `restore --format coldcard-edge` (same enum, `cmd/restore.rs:250`): refused
  at the **top of `restore::run`**, before the md1 dispatch (`:340-375`) and the
  existing `--format` gate (`:421`), so both routes into `emit_payload`
  (single-sig `:665`→`:3257`, md1 multisig `run_multisig` `:4337`→`:3582`) are
  covered. Pointer to `export-wallet --format coldcard-edge`.
- Every other shape: refused with a pointer (wsh/sh-wsh multisig →
  `coldcard-multisig`; bip44/49/84 single-sig → `coldcard`; P2TR single-sig and
  anything else → `descriptor`, since `coldcard` refuses bip86, `coldcard.rs:122`).

### 3.2 Checks (refuse: exit 1 `BadInput`, as the existing coldcard taproot refusal, `error.rs:643`; stdout empty, no file)

**Where:** inside the `coldcard-edge` emitter, i.e. *after* the shared admission
gate (so `every_format_meets_the_same_gate_before_its_own_verdict` still holds:
the gate's exit 2 comes first), on the **parsed canonical descriptor** (the
emitted text), never on `ResolvedSlot` (the template path replaces an empty or
`m` slot path with the template default, `pipeline.rs:46-62`, so a slot-level
depth check would pass a depth-0 xpub). An omitted `.fingerprint=` shows up as
`00000000` in the canonical text, so no raw-`SlotInput` check is needed. Each
check below says whether it is an Edge rule or toolkit policy.

1. **Internal key** must be a ranged xpub with origin (Edge rule E3; the origin
   requirement is toolkit policy). A bare/single key, which is how
   `--taproot-internal-key nums` renders, is refused on both paths by this one
   check. Text
   says why: current Edge refuses a static internal key, and Edge's own NUMS form
   is a different wallet. Points to `@N`. (FOLLOWUP for an Edge-NUMS option, §4.)
2. **Origin present, fingerprint non-zero** for every key (toolkit policy; an
   omitted `.fingerprint=` becomes `00000000`, `cmd/bundle.rs:641-646`). Names the
   key by its `@N` (template path) or its position (descriptor path).
3. **Path consistent with the xpub**: origin length == `xpub.depth` (Edge E8);
   at depth 1 fp == `xpub.parent_fingerprint` (Edge E8); at depth 0 fp == the
   xpub's own fingerprint (Edge, "master xfp mismatch", `desc_utils.py:318-322`);
   last element == `xpub.child_number` (toolkit policy).
4. **Distinct keys** by pubkey over every key including the internal key (Edge
   E9 refuses within-leaf duplicates and conflicting origins; refusing the
   internal key repeated in the leaf with the same origin is toolkit policy).
   Reuse the logic of `check_bip388_distinct_keys` (`wallet_export/bip388.rs:95`)
   with coldcard-edge wording, not the function itself (its text names bip388
   and keys_info positions).
5. **Key count** ≤ 32 counting leaf keys + internal key (docs' 32, under the code's
   34; E7).
6. **Suffix** exactly `/<0;1>/*` on every key (E1: Edge would rewrite `/0/*`,
   changing the registered wallet); same rule as the private
   `strip_multipath_suffix` (`wallet_export/pipeline.rs:266-276`), reimplemented
   or made `pub(crate)` with its own wording.
7. **Name** (E11), only when JSON is emitted: explicit `--wallet-name` must be 1-30
   printable ASCII.

### 3.3 Output

- Default: the canonical descriptor with checksum, one line: the same bytes
  `--format descriptor` emits for the same inputs. Extension `txt`.
- **Only an explicit `--wallet-name`** selects JSON `{"name":"NAME","desc":"…"}`
  (E4); a lifted or default name does not. Extension `json`.
- With `--output PATH` and plain output: warn (not refuse) if the file name up
  to its **first** dot (Edge's rule, not Rust's `file_stem`) is longer than 30
  characters or not printable ASCII, or the suffix is not `.txt`/`.json` (E11).

### 3.4 Warning (stderr, once, exit stays 0)

```
warning: --format coldcard-edge is for Coldcard's experimental Edge firmware (Coinkite: "DO NOT use for large Bitcoin amounts"); mainline Coldcard firmware cannot import taproot multisig.
warning: import it with Settings > Miniscript > Import from file; the Coldcard's own key must be one of the cosigners, and a testnet export needs the device in Testnet mode.
warning: cosigner @N is the internal key: it can spend alone by key path, and Coldcard prefers the key path.
warning: before funding, check the first receive address on the Coldcard against `mnemonic export-wallet --format bitcoin-core-addresses --count 1` with these same inputs.
```

(Exact wording settled in the build; the four facts are required. The quoted
sentence cites `History-Edge.md:7`. `@N` is the actual index on the template
path; on the `--descriptor` path line 3 reads "the internal key
`[fp/path]` can spend alone by key path …". `mnemonic addresses` is not the
command: it takes single-key sources only, R1 M-1.)

### 3.5 Unchanged elsewhere

The `coldcard` / `coldcard-multisig` taproot refusal gains one sentence: `For a
Coldcard running Edge firmware, use --format coldcard-edge.` Everything else in it
stays byte-identical (fixture `tests/export_wallet/coldcard_refusal_tr_multi_a.stderr`
and any `coldcard-multisig`/restore fixtures carrying the text are updated).

## 4. Lockstep and records (same PR unless noted)

- Manual: `41-mnemonic.md` export-wallet format table and restore `--format` list
  (`:1396`, `:1598-1610`); `30-workflows/37-wallet-export.md:444-451`;
  `30-workflows/33-taproot-multi.md` (Edge caveat: a NUMS wallet cannot be
  registered on current Edge). Mention `--format bip388` as the alternative
  route for Edge ≥ 6.4.0X (E4), and that it skips the Edge checks this format
  runs.
- Manual-gui: `45-export-wallet.md` list + anchor section; `4d-restore.md` list;
  `docs/manual-gui/tests/expected_gui_schema_inventory.json` (export-wallet and
  restore lists).
- `tests/cli_gui_schema.rs:289-312` (choices 12 → 13), and the hand-written
  `FORMATS` list in `every_format_meets_the_same_gate_before_its_own_verdict`
  (12 → 13).
- mnemonic-gui `src/schema/mnemonic.rs` `EXPORT_FORMATS` (shared by export-wallet
  and restore): paired PR, draft until a toolkit release ships.
- CHANGELOG.
- FOLLOWUPS:
  - `coldcard-tr-multi-a-pending-firmware`: mainline still blocked; Edge served
    by `coldcard-edge`; cite `new_edge`, correct the 2026-05-12 check.
  - `jade-tr-multi-a-pending-firmware`: its "once Coldcard ships, Jade follows"
    gets a note that only Coldcard Edge has shipped it.
  - new `coldcard-edge-nums-xpub-internal-key`: an Edge-NUMS (H-xpub) option needs
    its own spec (new wallet identity, chain-code source, md1/bundle/restore).
  - new `export-wallet-zero-fingerprint-path-depth-duplicate-keys`: R0 I2-I4
    showed these exit 0 under every format today (zero fp from an omitted
    `.fingerprint=`, a 3-component default path on a depth-4 xpub, the same xpub
    under two fps or repeated as internal key); this PR checks them only for
    `coldcard-edge`. Funds-relevant; owner none.
- Conflicts expected with open #228 (exit codes; BadInput may renumber), #229
  (`--template` named shapes become `--scheme`), #231 (coldcard.rs dead-arm
  removal): noted in the PR body.

## 5. Tests (RED first; `tests/export_wallet_coldcard_edge.rs`, plus its `all.rs` line)

1. `coldcard_edge_tr_sortedmulti_a_cosigner_internal_matches_descriptor_format` —
   three slots, `@0` internal, `--threshold 2` (the wallet is "@0 by key path, or
   2-of-2 of @1/@2"): stdout == `--format descriptor` stdout; exit 0; stderr has
   the four warning facts. Every fixture slot's `.path=` matches its xpub's depth
   and child number (several existing tests give the depth-3 cosigner C a
   4-component path; do not copy those into positive rows).
2. `coldcard_edge_tr_multi_a_cosigner_internal_key`.
3. `coldcard_edge_wallet_name_emits_named_json` — `desc` == row 1's line.
4. `coldcard_edge_refuses_nums_internal_key`.
5. `coldcard_edge_refuses_bare_hex_internal_key_descriptor`.
6. `coldcard_edge_refuses_missing_fingerprint` and `…_zero_fingerprint`.
7. `coldcard_edge_refuses_path_depth_mismatch` (depth-4 xpub with default
   `87'/0'/0'`), `…_refuses_slot_path_m_on_deep_xpub` (`@0.path=m`), and
   `…_refuses_depth0_fingerprint_mismatch`.
8. `coldcard_edge_refuses_same_xpub_two_fingerprints` and
   `coldcard_edge_refuses_internal_key_repeated_in_leaf`.
9. `coldcard_edge_refuses_over_32_keys`.
10. `coldcard_edge_refuses_bad_wallet_name` (31 chars; non-ASCII).
11. `coldcard_edge_refuses_non_multipath_suffix` (`/0/*` in `--descriptor`).
12. `coldcard_edge_descriptor_input_admitted` and refusals
    `…_refuses_multi_leaf`, `…_refuses_and_v_leaf`.
13. `coldcard_edge_refuses_from_import_json_wsh_envelope`,
    `…_taproot_envelope`, `restore_single_sig_refuses_coldcard_edge`,
    `restore_md1_multisig_refuses_coldcard_edge` (a NUMS md1 card included).
14. `coldcard_edge_refuses_wsh_multisig_with_pointer`, `…_single_sig_…`.
15. `coldcard_edge_testnet_tpub`.
16. `coldcard_taproot_refusal_points_to_edge` (fixture update).
17. **Independent address oracle** `coldcard_edge_first_address_matches_hand_built_taproot`
    (and the same for `multi_a`) — in the test, from `bitcoin` primitives only:
    derive each key's `/0/0` child; leaf script = `<x1> OP_CHECKSIG` then
    `<xi> OP_CHECKSIGADD` for each further key, then the threshold as a minimal
    push (`OP_2` …) and `OP_NUMEQUAL`; for `sortedmulti_a` the x-only keys sorted
    lexicographically, for `multi_a` in descriptor order; TapLeaf hash =
    tagged("TapLeaf", 0xc0 ‖ compact-size(len) ‖ script); single-leaf merkle root
    = the leaf hash; output key = internal `/0/0` child's x-only (even y) tweaked
    by tagged("TapTweak", P ‖ root); bech32m P2TR. Assert it equals the first
    receive address of the emitted descriptor (parsed in the test). Fixture slots
    are chosen so the `/0/0` x-only order differs from slot order (the row-1
    fixtures are already sorted, R1 M-5), so the sort is exercised. R1 confirmed
    this construction by an independent pure-Python computation and Coldcard's
    own (`miniscript.py:821-838`, `descriptor.py:51`).
18. `cli_gui_schema` choice list updated.

Gates: `cargo nextest run --locked` (or `cargo test --locked`), `cargo fmt --check`,
`cargo clippy --locked --all-targets -- -D warnings`, manual lint
(`make -C docs/manual lint`, available binaries), manual-gui anchor/inventory tests.
Before the plan is GREEN, rows 1 and 7 are drafted against a real build (R0 "a gate
that has never executed is a hypothesis").

## 6. Not in scope

wsh miniscript for Edge; multi-leaf trees; Edge-NUMS (H-xpub), `r=@`, `r=<hex>`
internal keys (FOLLOWUP above); BSMS or BIP-388 output for Edge (already
`--format bip388`); fixing I2-I4 for the other formats (FOLLOWUP above); restore
support.
