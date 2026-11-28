# R0 review: `design/PLAN_export_wallet_coldcard_edge.md` (plan r0)

Reviewer: Claude Opus 5.5 (R0 architect, adversarial). Date 2026-11-28.
Toolkit baseline: `origin/master` d72a18b1 (branch `claude/project-thread-0411jd`). The binary used for the experiments
below is `target/debug/mnemonic` 0.107.1, built from this checkout.

Coldcard sources were fetched with `curl -sfL raw.githubusercontent.com/Coldcard/firmware/<branch>/…` and with a shallow
`git fetch` of `edge`, `new_edge`, `edge634_final_touch` and `master`. Branch heads come from `git ls-remote`. The copies
are in `/tmp/claude-0/-home-user/60b3d9b3-c5ac-5b56-8550-14584d658108/scratchpad/ccfw-r0/`: `ed/` holds the `edge`
tree, `ne/` the `new_edge` tree, and `master/` plus the bare `git/` repo hold the rest. `ckcc_constants_pinned.py` is
ckcc-protocol at `650c54b5`, the submodule commit `new_edge` pins. All of it was read as data.

**Verdict: RED. 2 Critical, 8 Important, 8 Minor.** The plan cannot go to implementation as written. Its external facts
come from a branch that stopped two and a half years ago. On current Edge firmware, the plan's default case (the NUMS
internal key) produces a file that the device refuses.

---

## Answers to the plan's open question (lens 1)

The answers come from **`new_edge`** (HEAD `825ab7d`, 2026-11-17, "New release: 2026-11-17T1904-v6.6.2X"), which is
where Edge releases are built now. They do not come from `edge` (see C1). Paths are relative to `ne/`.

| Question | Answer on current Edge (6.6.2X) | Evidence |
|---|---|---|
| `<0;1>/*` multipath | **Yes.** A `/0/*` key is silently normalized to `<0;1>/*`. `/1/*` is refused ("need multipath"). Non-ranged keys are refused ("All keys must be ranged"). Hardened steps are refused. | `shared/desc_utils.py:174-218` (normalization 208-210); `:168` |
| `#checksum` suffix | **Yes.** It is optional, and it is verified when present ("Wrong checksum"). | `shared/descriptor.py:415-425`, `:428-441` |
| BIP-341 H as a bare x-only internal key | **NO, since 6.3.5X.** "Static internal keys disallowed - all keys need to be ranged extended keys". The parser asserts `key_str[1:4]=="pub"` ("only extended pubkeys allowed"). The docs mark the static-H form "deprecated in version 6.3.5X & 6.3.5QX". The recommended NUMS form is now an origin-less **xpub** whose pubkey is H, with a random chain code. | `releases/History-Edge.md:162-165`; `shared/desc_utils.py:290-291`; `docs/taproot.md:28-53`. (The old `edge` branch did accept it: `ed/shared/desc_utils.py:312-345`.) |
| `{"name","desc"}` JSON | **Yes.** `desc` is required and must not be empty. `name` is taken over the filename and over the checksum. A BIP-388 `{name,desc_template,keys_info}` object is also accepted. | `shared/auth.py:1585-1603` |
| `multi_a` and `sortedmulti_a` | **Yes, both.** | `shared/miniscript.py:816-850` |
| tpub on testnet | **Yes, but only when the device is in testnet mode.** The key's chain must equal the current key chain ("wrong chain"). Regtest maps to the testnet keys. A tpub on a mainnet device is refused. | `shared/desc_utils.py:297`; `shared/chains.py:367-372`, `:501-507` |
| Key count | The docs say 32 per leaf and 32 in the whole tree. The **code** allows 34: `MAX_TR_SIGNERS = const(34)`, applied to multi_a N and to `len(self.keys)`, which **includes the internal key**. Segwit v0 allows 20. | `docs/taproot.md:60-62`; `ckcc_constants_pinned.py:65`; `shared/miniscript.py:803,820`; `shared/descriptor.py:136-138,194`; `shared/utils.py:18-20` |
| Depth and leaf limits | The docs give depth 4 and 8 leaves. No depth or leaf counter exists in `Descriptor`/`Tapscript` parsing on `new_edge`. Both are irrelevant to a single leaf. | `docs/taproot.md:60`; `shared/descriptor.py:79-98` (no counter) |
| Other import refusals the plan does not list | The origin derivation length must equal the xpub depth. A depth-1 key's xfp must equal its parent fp. No duplicate keys within a leaf. A pubkey may not appear with a conflicting chain code or a conflicting origin. The device's own xfp must be present ("My key … missing"). Keys that claim the device's xfp must derive correctly. Wallet name: 1-30 printable ASCII characters. A plain file takes its name from the filename stem. The file picker wants `.txt`/`.json` and at least 100 bytes. | `shared/desc_utils.py:314-342`; `shared/descriptor.py:146,170-185,196`; `shared/wallet.py:29,521-523,1016,1030` |
| Is `edge` still the Edge release branch? | **No.** `edge` HEAD is `158c8a7` (2024-03-18) and its docs end at 6.2.2X. `edge634_final_touch` stops at 6.3.4X (2024-12-18). `new_edge` carries 6.6.2X (2026-11-17). | `git ls-remote` + `git log` (see the scratch `git/` dir) |
| Mainline can import taproot multisig? | **No.** master has no `miniscript.py` and no `tr()` parser. `descriptor.py` only sniffs the `tr(` prefix. | `master/shared_descriptor.py:258-259`; `git ls-tree master shared/` |

---

## Critical

### C1. The plan's "authoritative source" is a stale branch

Plan §2 (lines 19-22, F1-F4) anchors every external fact to `Coldcard/firmware` `edge`. That branch's last commit is
`158c8a7` on 2024-03-18, and its `EdgeChangeLog.md` ends at 6.2.2X. Current Edge (6.3.x through 6.6.2X, 2026-11-17)
ships from **`new_edge`**. F4 noticed the gap ("the `edge` branch's docs end at 6.2.2X … anything later is unknown")
but did not look for the branch that fills it. Since 6.2.2X, at least five import-relevant contract changes landed:

- static internal keys disallowed;
- origin-less keys allowed;
- legacy multisig import removed and BIP-388 policies added (6.4.0X);
- duplicate and conflicting-key refusals;
- name sanitization.

See `ne/releases/History-Edge.md:111-205` and `ne/releases/EdgeChangeLog.md:14-60`.

**Fix:** Rebase §2 on `new_edge` at a recorded commit (`825ab7d`) and cite file:line. Record the branch name as a fact
to re-check before the build, because a GREEN expires when its inputs move. Keep the `edge` citations only as history.

### C2. The default NUMS case exits 0 with a descriptor current Edge refuses

`build_tr_multi_a_descriptor` renders NUMS as the bare 64-hex H point (`wallet_export/pipeline.rs:141-143`,
`NUMS_XONLY_HEX` at `wallet_export/mod.rs:108-109`). Confirmed by experiment:
`export-wallet --template tr-sortedmulti-a --taproot-internal-key nums …` emits `tr(50929b74…,sortedmulti_a(…))#essg384w`
and exits 0.

Current Edge refuses that internal key: `ne/shared/desc_utils.py:290-291` ("only extended pubkeys allowed") and
`History-Edge.md:165`. Plan §3 calls this case "F2-allowed". Rows 1, 3 and 7 are built on it, and row 7 explicitly
admits `--descriptor 'tr(NUMS,…)'`.

This is funds-relevant. A user who funds the toolkit/Sparrow NUMS wallet and then finds the Coldcard cannot register it
cannot sign with that Coldcard. Miniscript signing requires a registered wallet, so if K needs the Coldcard, the funds
are stuck until the key is moved to other software.

The translation is not a one-line fix. Edge's recommended NUMS form, `tr(<xpub with pubkey H, random chaincode>/<0;1>/*, …)`,
is a **different wallet**: its internal key is a derived child of H, not H itself, so every address differs. It can never
equal `--format descriptor` output, and it has no md1/bundle representation today.

**Fix:**
1. Under `coldcard-edge`, refuse `--taproot-internal-key nums`, and on the `--descriptor` path refuse any internal key
   that is not a ranged xpub with origin (non-zero exit, stdout empty). Point the user to `--taproot-internal-key @N`.
   If the plan wants to be precise, it may add that Edge ≤ 6.3.4X accepted the static H.
2. Rewrite rows 1, 3 and 7 to use `@N`. Add rows `coldcard_edge_refuses_nums_internal_key` and
   `coldcard_edge_refuses_bare_hex_internal_key_descriptor`.
3. If an Edge-NUMS (H-xpub) option is wanted, it needs its own spec: a new wallet identity, a chain-code source, and
   md1/bundle/restore implications. Keep it out of this PR and file a FOLLOWUP.

---

## Important

### I1. Admission through `template_from_descriptor` cannot work, and `--from-import-json` refuses taproot

Plan §3 line 56-57 admits `--descriptor` / `--from-import-json` "whose resolved template (`template_from_descriptor`) is
one of those two". `template_from_descriptor` returns `Err(BadInput)` for every `Tr(_)` (`wallet_export/mod.rs:289-291`),
so it can never resolve to `TrMultiA`/`TrSortedMultiA`. Separately, `run_from_import_json` refuses every
`P2tr`/`P2trMulti` envelope before any emitter runs (`cmd/export_wallet.rs:961-973`, Fix-α, FOLLOWUP
`wallet-import-taproot-internal-key`). The plan does not mention lifting that refusal.

There is a second trap. The only taproot classifier is `script_type_from_descriptor`, which makes the `P2trMulti` call
with a **substring** test for `multi_a(` (`wallet_export/mod.rs:234-246`). `tr(K,{sortedmulti_a(…),pk(…)})` and
`tr(K,and_v(v:multi_a(…),older(…)))` would both pass as "taproot multisig".

**Fix:**
- Write a structural admission for the descriptor path: `Tr` with exactly one leaf, whose miniscript node is
  `Terminal::MultiA` or `Terminal::SortedMultiA` at depth 0. Record sorted vs unsorted from the node, not from a string.
- Either drop `--from-import-json` from scope, keeping today's refusal and adding a row that it still refuses, or state
  that Fix-α is lifted for `coldcard-edge` alone and specify where the internal key comes from.
- Add negative rows for the multi-leaf and `and_v` shapes.

### I2. The "missing fingerprint" check cannot be done on `ResolvedSlot`, and today the case exits 0

`ResolvedSlot.fingerprint` is not optional. An omitted `.fingerprint=` becomes `Fingerprint::default()` = `00000000`
(`cmd/bundle.rs:641-646`). Experiment: two `@N.xpub=` slots with no fingerprint emit `[00000000/87'/0'/0']xpub…` and
exit 0.

Edge imports `00000000` origins for foreign keys, because only a depth-1 or depth-0 key gets an fp check
(`ne/shared/desc_utils.py:318-330`). PSBT matching then uses those fps (`ne/shared/descriptor.py:241-278`), so the
registered wallet does not match a coordinator's PSBT.

**Fix:** Check the raw `args.slot` (`SlotInput`) for a `Fingerprint` subkey on each index, or refuse `00000000`
explicitly, naming the slot. On the `--descriptor` path, require every key to be a `DescriptorPublicKey::MultiXPub` with
`origin.is_some()`, a non-zero fp and a `/<0;1>/*` suffix (see M6). Origin info alone does not cover single keys, which
Edge refuses (`desc_utils.py:291`).

### I3. No check that the path is consistent with the xpub depth

Edge asserts `len(origin.derivation) == xpub.depth` and checks the parent fp at depth 1 (`ne/shared/desc_utils.py:318-330`).
The toolkit backfills a missing `.path` with the template or family default (`cmd/bundle.rs:647-660`; the bip87 default
is `87'/0'/0'`, three components) and never compares it with the xpub depth. Nothing on the export path does this.

Experiment: the fixture xpubs X1 and X2 (depth 4, child `80000002`, i.e. BIP-48 `…/2'`) export as `[…/87'/0'/0']xpub…`
with exit 0. Edge refuses the file ("deriv len != xpub depth").

**Fix:** Add a check before emitting: path length == `xpub.depth`, last path element == `xpub.child_number`, and at
depth 1 the fingerprint == `xpub.parent_fingerprint`. Add a row `coldcard_edge_refuses_path_depth_mismatch`.

### I4. Duplicate keys are not refused today

Plan §3 line 67-68 says "the toolkit pipeline may already refuse — reuse". That is only partly true:

- Identical `[fp/path]xpub` slots are refused by the strict parse ("repeated pubkeys", `pipeline.rs:39-41`).
- The same xpub under two fingerprints exits 0. Experiment: `sortedmulti_a(1,[b8688df1/…]X1/…,[28645006/…]X1/…)`.
- `--taproot-internal-key @0` with `@0`'s xpub repeated in `@1` exits 0. Experiment:
  `tr([b8688df1…]X1/<0;1>/*,sortedmulti_a(1,[b8688df1…]X1/<0;1>/*,…))`.
- The `--descriptor` path is lenient and runs no repeated-key check at all (`cmd/export_wallet.rs:617-623`,
  `parse_descriptor.rs:432-438`).

Edge refuses within-leaf duplicates and conflicting origins or chain codes (`ne/shared/descriptor.py:146,170-185`), and
`docs/miniscript.md:22` says "no duplicate keys".

**Fix:** Run `check_bip388_distinct_keys` (`wallet_export/bip388.rs:95`), or the same pubkey-distinctness logic, over
**every** key including the internal key, on both paths. Add rows for both demonstrated cases.

### I5. Row 10's address cross-check proves nothing

Row 10 compares the first address derived from row 1's descriptor with `--format bitcoin-core-addresses` for the same
inputs. Both outputs come from the same `canonical` string (`cmd/export_wallet.rs:806` → `EmitInputs`) through the same
rust-miniscript derivation, and `coldcard-edge` is specified as byte-identical to `--format descriptor`. The check
therefore cannot fail. It says nothing about how Coldcard interprets the descriptor, which is the risk.

**Fix:** Pin row 10 to an **independent** oracle. Options:
- an address computed in the test from `bitcoin` primitives only (tapleaf hash of the hand-built
  `<k1> CHECKSIG <k2> CHECKSIGADD … <K> NUMEQUAL` with the keys sorted, `TapTweak` with the internal key);
- a vector taken from Coldcard's own `testing/` suite or simulator for a `tr(@N-xpub, sortedmulti_a)` wallet;
- the existing ignored `bitcoind_differential` harness.

In each case, assert the emitted file's first `/0/0` address against that oracle.

### I6. `restore --format` gains `coldcard-edge` and the plan does not cover it

`restore --format` is typed `Option<CliExportFormat>` (`cmd/restore.rs:250`) and dispatches through the same
`emit_payload` (`cmd/restore.rs:3205-3230`, `3494-3560`). That path includes the general-tr arm with `template: None`
(possibly multi-leaf, classified `P2trMulti` by substring) and NUMS md1 cards. Adding the variant exposes
`restore … --format coldcard-edge` with none of the plan's checks specified for it.

The GUI mirror also shares the list: `EXPORT_FORMATS` is used for both export-wallet and restore
(`mnemonic-gui/src/schema/mnemonic.rs:185,608,1612`).

**Fix:** Either specify `coldcard-edge` under restore (same checks; rows for a template md1 with `@N`, a NUMS md1
refusal and a general-arm refusal) or refuse it there explicitly. Add to §4:
- manual `41-mnemonic.md:1396` and `:1598-1610` (restore `--format` list);
- manual-gui `4d-restore.md:112-152` (list plus anchor section);
- `docs/manual-gui/tests/expected_gui_schema_inventory.json` (export-wallet and restore `--format` lists);
- the GUI mirror PR covering both subcommands.

### I7. Exit-code contradiction

Plan §3 says "exit 2, `BadInput`". `ToolkitError::BadInput` maps to **exit 1** (`error.rs:643`). The coldcard taproot
refusal exits 1 today (verified), and the manual documents exit 1 (`41-mnemonic.md:1199`). The exit-2 refusals in this
path are `DescriptorParse` or typed variants. Rows 4 and 5 would therefore assert something the code cannot produce.

**Fix:** Pick one. Either add a typed `ToolkitError` variant with exit 2 (inserted alphabetically, per CLAUDE.md), or say
exit 1. Note the interaction with open PR #228.

### I8. Wallet-name constraints, and what triggers the JSON form

Edge requires a name of 1-30 printable ASCII characters (`ne/shared/wallet.py:29,521-523`; `utils.py:234`). For a plain
`.txt` import, the name is the **filename stem** (`wallet.py:1030`). A `--wallet-name` longer than 30 characters, or a
non-ASCII one, therefore exits 0 with a file Edge refuses. The same happens with a plain file whose `--output` stem is
longer than 30 characters.

The plan also keys the JSON form off "`--wallet-name` is given". The code already distinguishes three name sources:
explicit, lifted from a BIP-388 policy, and default (`wallet_name_is_non_default`, `cmd/export_wallet.rs:752-766,819`).
The plan does not say which of these selects JSON.

**Fix:**
- Refuse a `--wallet-name` that is not 1-30 printable ASCII under `coldcard-edge`.
- State that only an explicit `--wallet-name` selects JSON, or that every non-default source does.
- For a plain file written with `--output`, warn when the stem is longer than 30 characters or has no `.txt`/`.json`
  suffix (`wallet.py:1016`).

---

## Minor

- **M1. The 32-key limit needs a defined count.** The code allows 34 and counts the internal key
  (`descriptor.py:194`; `MAX_TR_SIGNERS=34`). Keeping the documented 32 is fine, but say what is counted: leaf keys plus
  the internal key. Row 6 is reachable, since slot indexes are `u8` and the multi_a cap is far higher
  (`slot_input.rs:154-156`). Drop the "(if the templates allow >32 at all…)" hedge.
- **M2. The `--json` bullet is vacuous.** Plan §3 line 82-83 refers to an envelope mode that `ExportWalletArgs` does not
  have (`cmd/export_wallet.rs:261-440`). Delete it.
- **M3. Row 11 names the wrong fixtures.** `cli_help_fixtures.rs` has no export-wallet snapshot. The real touch points
  are:
  - `tests/cli_gui_schema.rs:289-312` (choice list, `choices.len() == 12` → 13);
  - `tests/export_wallet/coldcard_refusal_tr_multi_a.stderr` (byte-exact; the plan's new pointer sentence changes it),
    plus any `coldcard-multisig` or restore stderr fixtures that carry the same text;
  - `docs/manual-gui/tests/expected_gui_schema_inventory.json`.
- **M4. The warning text needs current wording and more content.**
  - The current `EdgeChangeLog.md:3-10` no longer says "DO NOT use for large Bitcoin amounts". That sentence survives
    only in `History-Edge.md:7`. Quote the current text, or cite the history file.
  - The device's own key must be one of the cosigners (`descriptor.py:196`).
  - A testnet export needs the device in Testnet mode (`desc_utils.py:297`).
  - With `@N` internal, cosigner N can spend alone by key path, and Coldcard prefers the key path
    (`docs/taproot.md:64`).
  - Name the toolkit command that prints the first receive address, because `--format descriptor` prints none.
- **M5. BIP-388 is an alternative route.** Edge ≥ 6.4.0X imports BIP-388 wallet policies (`auth.py:1587-1590`;
  `History-Edge.md:126-127`), and `--format bip388` already emits one for `@N` wallets. Mention it in the manual note,
  or justify the descriptor form instead.
- **M6. Descriptor-path suffixes.** Edge rewrites `/0/*` to `<0;1>/*` (`desc_utils.py:208-210`), so the registered
  wallet gains a change chain that the input descriptor lacked. `xpub/5/<0;1>/*` parses differently again. Require
  exactly `/<0;1>/*` on every key, reusing `strip_multipath_suffix`'s rule (`pipeline.rs:266-276`), so that the output
  is the descriptor Edge registers.
- **M7. F1's limitation text is outdated.** "only keys with key origin info" no longer holds: current
  `docs/miniscript.md:24` also allows origin-less ("blinded") keys. The toolkit's own origin requirement is still a fine
  policy, but cite it as toolkit policy, not as an Edge rule.
- **M8. Lockstep list gaps.**
  - Add manual `30-workflows/37-wallet-export.md:444-451` (the coldcard taproot refusal paragraph) and
    `30-workflows/33-taproot-multi.md`, which teaches `nums` as the default and needs an Edge caveat given C2.
  - Add manual-gui `45-export-wallet.md:292-340` (list plus an anchor section, for anchor parity).
  - Add the `jade-tr-multi-a-pending-firmware` neighbour in FOLLOWUPS if its text cross-references Coldcard.
  - Correct the FOLLOWUPS status note to cite `new_edge`, not `edge`.
  - The CHANGELOG entry is listed. Good.
  - I could not verify the open-PR conflict claims (#228, #229, #231) from this session.

---

## Items checked and found true

- Mainline Coldcard cannot import taproot multisig (C1 table).
- F4's "6.6.0 (Edge Mk/Q)" reference: master `releases/ChangeLog.md:10`.
- The F2 limits text matches both branches' `docs/taproot.md`.
- The F3 6.0.0X and 6.2.2X quotes: `ed/releases/EdgeChangeLog.md:10-16,44-46`.
- `--format descriptor` emits `<desc>#<csum>` with `<0;1>/*` (`wallet_export/descriptor.rs:16-21`).
- Edge accepts the checksum.
- `@N` internal-key output (`[fp/path]xpub/<0;1>/*` with the cosigner removed from the leaf, `pipeline.rs:145-166`) is
  accepted by current Edge as long as I2, I3 and I4 hold.
- Single leaf by construction holds for the template path (`pipeline.rs:140-166`), but not for the descriptor or restore
  general arms (I1, I6).

## Required before re-review

Rebase §2 on `new_edge`. Then:
- make `nums` and any bare-key internal key a refusal (C2);
- replace the admission mechanism with a structural single-leaf check and settle `--from-import-json` (I1);
- add the fingerprint, depth and distinctness checks with rows (I2, I3, I4);
- give row 10 an independent oracle (I5);
- scope restore (I6);
- fix the exit code (I7) and the name rules (I8).

Run the plan's own gates once against the plan text: draft rows 1 and 7 against an actual build, in the spirit of "a
gate that has never executed is a hypothesis".
