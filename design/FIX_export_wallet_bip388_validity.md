# FIX: `--format bip388` emits policies BIP-388 calls invalid (I)

Model: Claude Opus 5.5 (claude-opus-5-5), 2026-11-24. Small-fix track (RULES.md "Authors"). Branch
`fix/sec-export-wallet-bip388-validity`, base `origin/master` `00947cb7` (tk 0.107.1). This document's commit
changes no production code and adds no test file; the builder writes the rows in section 6.

Source: security sweep unit `export-wallet-bip388-validity`, routing row
`coordinator/reports/security-sweep/grade/routing.md:40`, graded **I** from R06 tsv151, tsv180 (template half) and
`export-wallet-bip388-duplicate-keys-info` (`coordinator/agent-reports/grade-security-sweep-toolkit-gui.md:78`).
What changes (routing cell, verbatim): "`--format bip388` refuses a policy BIP-388 calls invalid: a literal non-KP
key (the NUMS internal key on the template arm) and non-distinct `keys_info`; the NUMS shape is product point 25".

Rulings this document builds on:

- **P25 (Brian 2026-11-24 "architect decides"; `coordinator/agent-reports/architect-p25-nums-template-1124.md`,
  option (a)):** the template arm refuses a NUMS internal key, exit 1, with the text already ruled for the
  descriptor arm (toolkit FOLLOWUPS `bip388-literal-nums-key-refusal-text`, L6218), from one shared helper.
- **Part (2) is decided here from BIP-388's own text** (raw `bip-0388.mediawiki` saved at
  `/scratch/cache/p25-bip/bip-0388.mediawiki`, "Additional rules", L193):

  > The public keys obtained by deserializing elements of the key information vector must be pairwise distinct

**Three STOPs are open (section 11).** The document is written for the recommended answers. Section 11 lists the
rows, texts and tests that flip under the other answers.

**How this document was checked.** The code, texts and tests were applied in this worktree as a scratch change and
reverted. They are saved as reference patches:

- source only: `/scratch/cache/bip388-validity-fix-reference.patch` (sha256
  `39e9130e13722ed3d10b49feb7a9c2e3a4439e70dbc619823ab9f5580eb0624c`, 5 files);
- source and tests: `/scratch/cache/bip388-validity-fix-reference-full.patch` (sha256
  `f6eda528c23ae680180ce559707e2783ef6fb1a30b9296ea346f214c6415dc5a`, 11 files; `git apply --check` passes at
  `00947cb7`); the new test file alone: `/scratch/cache/bip388-validity-cli_bip388_validity.rs` (sha256
  `0fd1cf11470c90127cadd509a61cf25e18165a17397ff314be5f6746b9531575`).

Every "today" cell in section 6 is the real output of a debug `mnemonic` built from `00947cb7`; every "expected"
cell passed with the full patch. The patches carry no doc-comment, manual, spec or CHANGELOG change (sections 4
and 5 give those as exact text).

## 1. Summary

BIP-388 allows only `@N` key placeholders in a `description_template` (L149-162; L310 lists "Expression with a
non-KP key present" as invalid) and requires the `keys_info` public keys to be pairwise distinct (L193). Today
the toolkit breaks both at exit 0:

1. `export-wallet --template tr-multi-a|tr-sortedmulti-a --taproot-internal-key nums --format bip388`, and
   `restore --md1 <NUMS tr-multisig card> --format bip388`, emit
   `"description_template": "tr(50929b74…803ac0,sortedmulti_a(…))"`: a literal key. The `--descriptor` arm
   already refuses the same shape, with a text about the `/<0;1>/*` suffix.
2. Every `bip388` emitter writes one `keys_info` entry per key occurrence and never checks distinctness:
   `export-wallet --descriptor` (also a BIP-388 policy JSON given to `--descriptor`), `export-wallet --template`,
   `restore --md1` (a policy that repeats `@0`) and `build-descriptor --allow repeated-keys --format bip388` all
   emit a `keys_info` that lists one key twice.

After the fix each case refuses (exit 1, stdout empty, no file written) and names `--format descriptor` /
`--format bitcoin-core`, which still emit the wallet. `build-descriptor --json` keeps exit 0 and writes
`"bip388": null` (S1).

No new flag, `ToolkitError` variant or public API. One printed text is new (DUP), one is the ruled text (LIT), one
existing text gains two words (section 2.4). Small-fix track is enough.

## 2. Holder-visible behaviour (after the fix)

### 2.1 Texts (byte-exact stderr lines; `error: ` is added by the error printer)

**LIT** (ruled, P25; `<hex>` is the internal key as the canonical descriptor prints it, for NUMS
`50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0`):

`error: --format bip388 cannot express the literal key <hex> (a taproot internal key with no origin, e.g. the BIP-341 NUMS key): a BIP-388 wallet policy allows only @N key placeholders; use --format descriptor or --format bitcoin-core for this wallet`

**DUP** (new; export-wallet and restore):

`error: --format bip388 cannot express a wallet that uses one public key more than once (keys_info @<i> and @<j>: <xpub>): a BIP-388 wallet policy requires pairwise distinct keys; use --format descriptor or --format bitcoin-core for this wallet`

**DUP-BUILD** (new; build-descriptor, which has no `bitcoin-core` format):

`error: --format bip388 cannot express a wallet that uses one public key more than once (keys_info @<i> and @<j>: <xpub>): a BIP-388 wallet policy requires pairwise distinct keys; use --format descriptor for this wallet`

- `<i>`, `<j>`: indices into the `keys_info` the policy would have had (template arm: the slot `@N`; descriptor
  arm: key occurrence order in the descriptor). The pair reported is the first with `i < j`, smallest `i`, then
  smallest `j` (K1,K2,K1,K2 reports `@0 and @2`).
- `<xpub>`: entry `@<i>`'s extended key in base58, without its origin.
- Exit code 1 (`ToolkitError::BadInput`) for all three, as for every other `--format bip388` refusal. No new
  error variant. The bundle-side `Bip388Distinctness` (exit 2) is unchanged and not used here.

### 2.2 The rules

- **Literal key.** Template arm: `--taproot-internal-key nums` with `tr-multi-a` / `tr-sortedmulti-a` refuses
  with LIT. Descriptor arm: a `tr(...)` whose internal key is a single key (`DescriptorPublicKey::Single`) with
  **no origin** refuses with LIT. Any other literal key keeps today's refusal and text (a literal key in a `wsh`
  or tap leaf, an internal single key that carries an origin): `error: --format bip388 requires every descriptor
  key to end in `/<0;1>/*` for the receive/change pair; got key "<key>"`.
- **Distinct keys.** Two `keys_info` entries whose deserialized public keys are equal refuse with DUP (DUP-BUILD
  on build-descriptor). The comparison is the 33-byte public key of each extended key (S3), so it does not look at
  origins or fingerprints: one xpub under two origins refuses, and so do two extended keys that share a public key
  but differ in chain code.
- **Order on the descriptor arm** (`descriptor_to_bip388_wallet_policy`): parse; LIT (a NUMS key gets LIT even
  when the descriptor is not multipath); then today's multipath refusal; then today's per-key suffix refusal; then
  DUP. So one key under disjoint multipaths (`<0;1>` and `<2;3>`) keeps today's suffix refusal.
- **Order on the template arm** (`format_bip388_wallet_policy`): everything upstream is unchanged; LIT, then DUP. A
  taproot template with two equal slots never reaches either: the strict descriptor parse refuses it first, exit
  2, `error: export-wallet descriptor parse: Miniscript contains repeated pubkeys or pubkeyhashes` (unchanged).
- **No output on refusal.** stdout is empty, the `--output <file>` file is not created, and the
  `note: stdout is watch-only — public keys only, cannot spend` line is not printed. On `restore` the refusal is
  the only stderr line (no `multisig restore` preamble). On `build-descriptor` the `--allow` banner
  (`WARNING: sanity rules OVERRIDDEN by --allow and FIRED: repeated-keys. …`) prints first, then DUP-BUILD.

### 2.3 Surfaces (every `bip388` emitter; grep of `format_bip388_wallet_policy`, `descriptor_to_bip388_wallet_policy`, `Bip388Emitter` at `00947cb7`)

| surface | emitter | LIT | DUP |
|---|---|---|---|
| `export-wallet --template … --format bip388` | `format_bip388_wallet_policy` | yes (NUMS) | yes |
| `export-wallet --descriptor <descriptor or BIP-388 policy JSON> --format bip388` | `descriptor_to_bip388_wallet_policy` | yes (origin-less `tr` internal key) | yes |
| `restore --md1 … --format bip388`, template card | `format_bip388_wallet_policy` via `Bip388Emitter` | yes (NUMS tr-multisig card) | no row (bundle refuses equal cosigners, so no such card was built); the check is in the shared emitter |
| `restore --md1 … --format bip388`, general wsh card | `descriptor_to_bip388_wallet_policy` via `Bip388Emitter` | unreachable (general taproot cards refuse first, restore.rs:3539-3548) | yes |
| `build-descriptor --format bip388` | `descriptor_to_bip388_wallet_policy` | unreachable (builds `wsh` only) | yes, DUP-BUILD (needs `--allow repeated-keys`) |
| `build-descriptor --json` | same | — | `"bip388": null`, exit 0 (S1) |

`build-descriptor --format descriptor` and the human view no longer compute the BIP-388 policy at all, so they are
unchanged for every input. `import-wallet` and `xpub-search` read BIP-388 policies and emit none.

### 2.4 One existing text changes (restore, general taproot card, `--format bip388`)

restore.rs:3547's parenthetical `(A distinct-trunk tr-multisig md1 DOES export bip388 via its template path.)`
becomes `(A non-NUMS distinct-trunk tr-multisig md1 DOES export bip388 via its template path.)`. After this fix
a NUMS tr-multisig card refuses bip388, so the old sentence would be false. The rest of that line is unchanged.

### 2.5 Unchanged

Every `--format bip388` run whose keys are distinct and whose `tr` internal key is `@N` (all formats, all
surfaces); every other `--format`; `bundle`'s own distinctness rule (SPEC v0.5 §4.11.b, see section 12 line 3).

## 3. Code delta (the reference patch)

1. `wallet_export/bip388.rs`: `pub(crate) fn bip388_literal_key_refusal(hex: &str) -> ToolkitError` returning
   `BadInput` with LIT (architect's helper, verbatim); `BIP388_EXPORT_ALTERNATIVES = "--format descriptor or
   --format bitcoin-core"` and `BIP388_BUILD_ALTERNATIVES = "--format descriptor"`; `pub(crate) fn
   check_bip388_distinct_keys(xpubs: &[Xpub], alternatives: &str) -> Result<(), ToolkitError>`: the pairwise
   scan of section 2.1, comparing `xpub.public_key`, `BadInput` with DUP / DUP-BUILD.
2. `format_bip388_wallet_policy`: the `TaprootInternalKey::Nums` arm returns `bip388_literal_key_refusal(NUMS_XONLY_HEX)`;
   after the template is built, `check_bip388_distinct_keys(<slot xpubs>, BIP388_EXPORT_ALTERNATIVES)`.
   `Bip388Emitter::emit` passes `BIP388_EXPORT_ALTERNATIVES` to the descriptor converter.
3. `wallet_export/pipeline.rs` `descriptor_to_bip388_wallet_policy` gains `alternatives: &str`; after the parse,
   a `Descriptor::Tr` whose `internal_key()` is `Single` with `origin == None` returns
   `bip388_literal_key_refusal(<internal key>.to_string())`; the key loop collects each `MultiXPub` `xkey` and
   calls `check_bip388_distinct_keys` after the loop.
4. `cmd/build_descriptor.rs` `emit`: the BIP-388 policy is computed lazily: `--json` writes
   `bip388().unwrap_or(Value::Null)`; `--format bip388` propagates the error; `--format descriptor` and the human
   view do not compute it. `wallet_export/mod.rs` re-exports `BIP388_BUILD_ALTERNATIVES`.
5. `cmd/restore.rs:3547`: section 2.4.

Doc comments (not in the patch; the builder writes them):

- `bip388.rs` L61-64, `/// … Taproot multisig with cosigner-internal key uses `@N/**` / for the internal key; with
  NUMS, the literal hex is embedded directly.` becomes `/// … Taproot multisig with cosigner-internal key uses
  `@N/**` / for the internal key; with NUMS it refuses (`bip388_literal_key_refusal`: BIP-388 allows only `@N` key
  / placeholders). Keys whose public keys are not pairwise distinct refuse (`check_bip388_distinct_keys`).`
- `pipeline.rs` L170-175 (the `descriptor_to_bip388_wallet_policy` paragraph) gains, after `… other derivation
  suffixes are refused.`: `A `tr` whose internal key is an origin-less literal key refuses first (the ruled
  literal-key text), and keys whose public keys are not pairwise distinct refuse last (BIP-388 "Additional
  rules"); `alternatives` is the format list those texts name.`
- The helper and the check carry a one-line doc each that cites BIP-388 L310 and L193.

## 4. Spec delta

4.1. `design/SPEC_descriptor_builder_allow.md` L91, the bullet

> - **bip388 shape note (R0-r1 M5):** allowed repeated-keys emit `--format bip388` → exit 0; the duplicate key appears as TWO `keys_info` entries (no dedup — probed); pin the shape in a cell and document in the manual (hardware-signer registration behavior on duplicate `keys_info` is out of toolkit scope).

becomes

> - **bip388 shape note (R0-r1 M5; superseded 2026-11-24 by `FIX_export_wallet_bip388_validity.md`):** an allowed repeated-keys policy has no BIP-388 form (BIP-388 "Additional rules": the public keys of `keys_info` must be pairwise distinct). `--format bip388` refuses it (exit 1, the DUP-BUILD text of that FIX); `--json` carries `"bip388": null`; `--format descriptor` and the human view are unchanged. Was: exit 0 with the duplicate key as two `keys_info` entries.

4.2. Same file L77, the phrase `and a note that an allowed repeated-keys policy emits duplicate `keys_info`
entries in bip388 (signer registration behavior is signer-defined — R0-r1 M5)` becomes `and a note that an
allowed repeated-keys policy has no BIP-388 form (`--format bip388` refuses; `--json` `bip388` is null —
FIX_export_wallet_bip388_validity.md, superseding R0-r1 M5)`.

Justification (one line, for the erratum): the sweep grade (I) and routing row 40 decide that `--format bip388`
refuses a BIP-388-invalid policy; `--allow repeated-keys` waives a miniscript sanity rule, not BIP-388's format
rule (S2).

4.3. No other spec sentence changes. The P25 ruling contradicts no spec text: grep of `design/SPEC*.md` at
`00947cb7` finds no sentence saying the template arm emits the NUMS hex in a BIP-388 policy (only the manuals,
section 5).

## 5. Manual, GUI-manual, technical-manual and CHANGELOG delta (lockstep, RULES "Builds and tests")

No flag is added, removed or renamed and no `--help` text changes: no `gui-schema` change, no
`mnemonic-gui/src/schema/mnemonic.rs` change; `docs/manual/tests/lint.sh` flag coverage is unaffected.

5.1. `docs/manual/src/40-cli-reference/41-mnemonic.md:1199` (`export-wallet` `--format` row): before the row's
closing ` |`, after `… so use `bitcoin-core` or `sparrow` for taproot`, insert:

> . **`bip388`** refuses (exit 1) a policy BIP-388 calls invalid: a NUMS taproot internal key (`--taproot-internal-key nums`, or a `--descriptor` `tr(<NUMS hex>,…)`), which is a literal key and not an `@N` placeholder, and a wallet that uses one public key more than once (BIP-388 requires pairwise distinct `keys_info` keys, whatever their origins); use `descriptor` or `bitcoin-core` for these wallets

5.2. Same file L1620-1622, the sentence

> A **distinct-trunk taproot multisig** card (NUMS *or* non-NUMS), by contrast, takes the template path and *does* emit `bip388` faithfully — `tr(@idx/**, multi_a(k,…))`.

becomes (line-wrapped like its neighbours)

> A non-NUMS **distinct-trunk taproot multisig** card, by contrast, takes the template path and *does* emit `bip388` faithfully — `tr(@idx/**, multi_a(k,…))`. A **NUMS** taproot multisig card refuses `bip388` (exit 1): its internal key is a literal key, and BIP-388 allows only `@N` key placeholders; it emits `descriptor` / `bitcoin-core`. A card whose policy uses one public key more than once (e.g. a `bundle --descriptor` that repeats `@0`) refuses `bip388` too (exit 1; BIP-388 requires pairwise distinct keys).

L1639-1640 (`while a non-NUMS distinct-trunk **multisig** also emits `bip388``) stays: it is still true.

5.3. Same file L4920 (`build-descriptor` `--json` row): `(the `cost` field is the embedded `compare-cost --json`
object).` becomes `(the `cost` field is the embedded `compare-cost --json` object; `bip388` is `null` when BIP-388
cannot express the policy, e.g. an allowed `repeated-keys` policy).`

5.4. Same file L4985-4987, the bullet

> - An allowed `repeated-keys` policy emits duplicate `keys_info` entries in its BIP-388 output (no dedup); hardware-signer registration behavior on duplicate keys is signer-defined.

becomes

> - An allowed `repeated-keys` policy has no BIP-388 form (BIP-388 requires pairwise distinct `keys_info` keys): `--format bip388` refuses it (exit 1) with `error: --format bip388 cannot express a wallet that uses one public key more than once (keys_info @<i> and @<j>: <xpub>): a BIP-388 wallet policy requires pairwise distinct keys; use --format descriptor for this wallet`, and `--json` emits `"bip388": null`. `--format descriptor` and the human view are unchanged.

5.5. `docs/manual-gui/src/40-mnemonic/45-export-wallet.md`:

- L322-326 (`### bip388`): after `… upstream BIP-388 standard format directly.` add `It refuses (exit 1) a policy
  BIP-388 calls invalid: a NUMS taproot internal key, or one public key used twice; see Refusals below.`
- Refusals table: after the last row (L626, `--format specter` without `--wallet-name`), two rows:

> | `--format bip388` with `--taproot-internal-key nums`, or a `--descriptor` whose taproot internal key is a literal key with no origin (e.g. the NUMS key) | `--format bip388 cannot express the literal key <hex> (a taproot internal key with no origin, e.g. the BIP-341 NUMS key): a BIP-388 wallet policy allows only @N key placeholders; use --format descriptor or --format bitcoin-core for this wallet` |
> | `--format bip388` and two keys with one public key (a repeated key, or one xpub under two origins) | `--format bip388 cannot express a wallet that uses one public key more than once (keys_info @<i> and @<j>: <xpub>): a BIP-388 wallet policy requires pairwise distinct keys; use --format descriptor or --format bitcoin-core for this wallet` |

5.6. `docs/manual-gui/src/40-mnemonic/4d-restore.md` L137-139: after `… such a card emits `descriptor` /
`bitcoin-core` only.` add `A NUMS taproot multisig card, and a card whose policy uses one public key more than
once, refuse it too (exit 1): BIP-388 allows only `@N` key placeholders and requires pairwise distinct keys.`

5.7. `docs/manual-gui/src/40-mnemonic/4e-build-descriptor.md`:

- L217-220, `The BIP-388 output emits duplicate `keys_info` entries (no dedup); hardware-signer behavior on
  duplicate keys is signer-defined.` becomes `Such a policy has no BIP-388 form (BIP-388 requires pairwise
  distinct keys): `--format bip388` refuses it (exit 1) and `--json` carries `"bip388": null`.`
- L286-288 (`--json`): after `` `compare-cost --json` object).`` add `` `bip388` is `null` when BIP-388 cannot
  express the policy (an allowed `repeated-keys` policy).``

5.8. `docs/technical-manual/src/50-rust-api/54-mnemonic-toolkit-api.md:494`: `For `tr-multi-a` /
`tr-sortedmulti-a` with `--taproot-internal-key nums`, the literal `NUMS_XONLY_HEX` (`50929b74...803ac0`) is
embedded as the internal key; with` becomes `For `tr-multi-a` / `tr-sortedmulti-a` with `--taproot-internal-key
nums`, `format_bip388_wallet_policy` refuses via `bip388_literal_key_refusal` (`BadInput`, exit 1: BIP-388 allows
only `@N` key placeholders), as `descriptor_to_bip388_wallet_policy` does for a `tr` whose internal key is an
origin-less literal key; with`. After `… longest-first to avoid prefix collisions —
`wallet_export/pipeline.rs::descriptor_to_bip388_wallet_policy`.` add ` Both paths refuse (`BadInput`, exit 1)
keys whose public keys are not pairwise distinct (`check_bip388_distinct_keys`, BIP-388 "Additional rules").`

5.9. `CHANGELOG.md`, under `## [Unreleased]`, a new subsection, exact text:

> ### `--format bip388` refuses a policy BIP-388 calls invalid
>
> - **`export-wallet`, `restore`**: `--format bip388` now refuses (exit 1) a NUMS taproot internal key, which is a
>   literal key and not an `@N` placeholder. Before, `--template tr-*-multi-a --taproot-internal-key nums` and a
>   NUMS `restore --md1` card emitted `tr(50929b74…,…)` at exit 0, and `--descriptor` refused with a text about the
>   `/<0;1>/*` suffix. It also refuses a wallet that uses one public key more than once (before: duplicate
>   `keys_info` entries at exit 0). Use `--format descriptor` or `--format bitcoin-core` for these wallets.
> - **`build-descriptor`**: with `--allow repeated-keys`, `--format bip388` refuses (exit 1) and `--json` emits
>   `"bip388": null`; `--format descriptor` and the human view are unchanged.

5.10. Other doc hits checked and unchanged: `41-mnemonic.md:4255` (xpub-search input shapes), `42-md.md`
repeated-key notes (md templates, not BIP-388 policies), the technical manual's `42-anti-collision-invariants.md`
(bundle distinctness), and `42-md.md:412/454` (`tr(50929b74…,multi_a(2,@0,…))` md templates, per the architect).

## 6. RED rows

New file `crates/mnemonic-toolkit/tests/cli_bip388_validity.rs` (the reference copy; each `tests/*.rs` is its own
binary, nothing to register). It holds the architect's four `p25_*` rows verbatim except that
`p25_restore_nums_tr_multisig_bip388_refuses` asserts stderr **equal** to LIT (measured: restore prints nothing
else before a refusal), plus the `bip388v_*` rows. One row goes in `tests/cli_restore_taproot.rs` (it uses that
file's `bundle_md1` / `restore_args` helpers). Refusal rows assert exit code, **stdout empty** and stderr equal to
the text plus `\n`.

Constants (signet; public keys only):

| name | value |
|---|---|
| `C0`, `C1`, `C2` | BIP-39 test phrases `abandon ×11 about`, `legal winner … yellow`, `letter advice … above` |
| `XA` | `tpubDFH9dgzveyD8zTbPUFuLrGmCydNvxehyNdUXKJAQN8x4aZ4j6UZqGfnqFrD4NqyaTVGKbvEW54tsvPTK2UoSbCC1PJY8iCNiwTL3RWZEheQ` (`convert --from phrase=C0 --to xpub --network signet --path m/48'/1'/0'/2'`) |
| `XB` | `tpubDEfobrrtptRTbKf4gysDhoabneABDTAcdj3Vbn4XwPsLE2pmqpizSPRG6zHsbAMuiSgWmWPsYCLHTKTPpyrGJ5rAoTpKoQNZcxodiPf2tSJ` (same, phrase C1) |
| `XA_OTHER_CHAINCODE` | `tpubDFH9dgzveyD8zT1tvsUvpENmA4ap3U5kyXv4qKQGYqRbXvoGcJfGqSExEeptW6f7HbrfyKS1rrSTGmXAyCjZYf5rceNxnQ7pNBo8adwpjEd` (`XA` with chain-code byte 0 XOR 0x01, re-encoded: same public key) |
| `XA_BUNDLE` | `tpubDDuXvjq5jan2D61HJDa31HeizPJdArvkNnax3csWXAmrggizBTFT5opWe81xufQv33vJUbzYaCEtPA3qJQY9hvQrMFckTU7zKvB3wHAnLHe` (what `bundle --descriptor` derives for `@0.phrase=C0`) |
| `KA`, `KB`, `KA'` | `[73c5da0a/48'/1'/0'/2']XA`, `[b8688df1/48'/1'/0'/2']XB`, `[b8688df1/48'/1'/0'/2']XA` |
| `OR_D(a,b)` | `wsh(or_d(pk(a/<0;1>/*),and_v(v:pk(b/<0;1>/*),older(100))))` |
| `EXP(f,d)` | `export-wallet --format f --descriptor d --network signet --output -` |
| `TPL(f,x1,fp1,p1)` | `export-wallet --format f --template wsh-sortedmulti --threshold 1 --network signet --output - --slot @0.xpub=XA --slot @0.fingerprint=73c5da0a --slot @0.path=m/48'/1'/0'/2' --slot @1.xpub=x1 --slot @1.fingerprint=fp1 --slot @1.path=p1` |
| `BUILD(…)` | `build-descriptor --archetype simple-timelocked-inheritance --key KA --recovery-key KA --older 100 --network signet --allow repeated-keys …` |
| `BANNER` | `WARNING: sanity rules OVERRIDDEN by --allow and FIRED: repeated-keys. This descriptor failed miniscript's funds-safety analysis; you have accepted that risk after review.` |

6.1. RED (fail on `00947cb7`, pass with the patch).

| row | command | expected | today (`00947cb7`) | wrong implementation it catches |
|---|---|---|---|---|
| `p25_export_wallet_template_nums_bip388_refuses` | architect's `export_args("bip388","nums","-")` (tr-sortedmulti-a, 3 signet slots) | 1; ""; LIT | 0; `"tr(50929b74…,sortedmulti_a(2,@0/**,@1/**,@2/**))"` | no template-arm check |
| `p25_export_wallet_template_nums_bip388_writes_no_file` | same, `--output <tmp>/policy.json` | 1; LIT; no file | 0, file written | refusal after the file is opened |
| `p25_restore_nums_tr_multisig_bip388_refuses` | `bundle --template tr-sortedmulti-a` (C0,C1,C2), then `restore --md1 … --format bip388` | 1; ""; stderr == LIT | 0; the literal template; stderr `2-of-3 multisig restore …` | restore bypassing the shared emitter |
| `bughunt_tk_21_bip388_export_from_descriptor_taproot_nums_multisig` (existing, `#[ignore]` removed) | mainnet fixture xpubs, `--descriptor tr(NUMS,sortedmulti_a(…))` | 1; ""; LIT | 1; `…requires every descriptor key to end in `/<0;1>/*` …; got key "50929b74…"` | descriptor arm left on the suffix text |
| `bip388v_export_descriptor_nums_refused` | `EXP(bip388, tr(NUMS,sortedmulti_a(1,KA/<0;1>/*,KB/<0;1>/*)))` | 1; ""; LIT | 1; the suffix text naming the NUMS key | same, signet |
| `bip388v_export_descriptor_nums_literal_before_multipath` | `EXP(bip388, tr(NUMS,sortedmulti_a(1,KA/0/*,KB/0/*)))` | 1; ""; LIT | 1; `error: --format bip388 requires the --descriptor to use multipath form `/<0;1>/*` (BIP-388 receive/change shape)` | LIT placed after the multipath check |
| `bip388v_export_descriptor_repeated_key_refused` | `EXP(bip388, OR_D(KA,KA))` | 1; ""; DUP(0,1,XA) | 0; `keys_info` = `[KA, KA]` | no descriptor-arm check |
| `bip388v_export_descriptor_same_xpub_other_origin_refused` | `EXP(bip388, OR_D(KA,KA'))` | 1; ""; DUP(0,1,XA) | 0; `keys_info` = `[KA, KA']` | comparing `keys_info` strings or `(xpub, path)` (the bundle rule) |
| `bip388v_export_descriptor_same_pubkey_other_chaincode_refused` (S3) | `EXP(bip388, OR_D(KA,[b8688df1/48'/1'/0'/2']XA_OTHER_CHAINCODE))` | 1; ""; DUP(0,1,XA) | 0; two entries | extended-key equality (mutant run: only this row fails) |
| `bip388v_export_descriptor_sortedmulti_repeated_key_refused` | `EXP(bip388, wsh(sortedmulti(1,KA/<0;1>/*,KA/<0;1>/*)))` | 1; ""; DUP(0,1,XA) | 0; `"wsh(sortedmulti(1,@0/**,@1/**))"`, `[KA, KA]` | a check only in miniscript (non-`Miniscript`) nodes |
| `bip388v_export_descriptor_policy_json_repeated_key_refused` | `EXP(bip388, {"name":"dup","description_template":"wsh(or_d(pk(@0/**),and_v(v:pk(@1/**),older(100))))","keys_info":[KA,KA]})` | 1; ""; DUP(0,1,XA) | 0; `"name": "dup"`, `[KA, KA]` | a check before the policy JSON expansion only |
| `bip388v_export_template_repeated_slot_refused` | `TPL(bip388,XA,73c5da0a,m/48'/1'/0'/2')`; leg: `--output <tmp>/policy.json` | 1; ""; DUP(0,1,XA); leg: 1, DUP, no file | 0; `"wsh(sortedmulti(1,@0/**,@1/**))"`, `[KA, KA]` (leg not reached) | no template-arm check; check after the file write |
| `bip388v_export_template_same_xpub_other_path_refused` | `TPL(bip388,XA,b8688df1,m/48'/1'/1'/2')` | 1; ""; DUP(0,1,XA) | 0; `[KA, [b8688df1/48'/1'/1'/2']XA]` | the `(xpub, path)` rule |
| `bip388v_restore_repeated_key_bip388_refused` | `bundle --descriptor wsh(or_d(pk(@0/<0;1>/*),and_v(v:pk(@0/<0;1>/*),older(100)))) --slot @0.phrase=C0 --network signet --json --no-engraving-card`, then `restore --network signet --format bip388 --md1 …` | 1; ""; DUP(0,1,XA_BUNDLE) | 0; `[[73c5da0a/48'/1'/0'/2']XA_BUNDLE ×2]`; stderr `miniscript policy restore (1 cosigner)` … | restore's general arm bypassing the check |
| `bip388v_build_descriptor_repeated_key_bip388_refused` (S2) | `BUILD(--format bip388)` | 1; ""; stderr == BANNER + `\n` + DUP-BUILD(0,1,XA) | 0; `[KA, KA]`; stderr BANNER only | export text on build-descriptor; no build-descriptor check |
| `bip388v_build_descriptor_repeated_key_json_bip388_null` (S1) | `BUILD(--json)` | 0; `bip388` is null; `descriptor` starts `wsh(or_d(pk(`; stderr == BANNER | 0; `bip388` = the duplicate object | the whole `--json` run refusing; the object kept |
| `bip388v_restore_general_tr_text_names_non_nums` (in `cli_restore_taproot.rs`) | `bundle_md1("tr(NUMS,{pk(K0),pk(K1)})")`, `restore … --format bip388` | 1; ""; stderr == section 2.4's full line | 1; the line with `(A distinct-trunk tr-multisig md1 DOES …)` | the stale parenthetical |

6.2. Guards (pass on `00947cb7` and must keep passing).

| row | command | expected (= today) | wrong implementation it catches |
|---|---|---|---|
| `p25_controls_cosigner_internal_key_and_named_alternatives` | `--taproot-internal-key @0` bip388; nums with `descriptor` / `bitcoin-core` | 0; `"tr(@0/**,sortedmulti_a(2,@1/**,@2/**))"`; both alternatives 0 with `tr(<NUMS>,sortedmulti_a(2,` | refusing every taproot template; LIT's advice not working |
| `bip388v_guard_distinct_keys_emit` | `EXP(bip388, OR_D(KA,KB))`; `TPL(bip388,XB,b8688df1,m/48'/1'/0'/2')` | 0; 2 `keys_info`; `"wsh(sortedmulti(1,@0/**,@1/**))"` | a check that refuses distinct keys (e.g. comparing fingerprints or paths) |
| `bip388v_guard_named_alternatives_emit_repeated_key` | `EXP(f, OR_D(KA,KA))` and `TPL(f,XA,73c5da0a,…)` for `f` in `descriptor`, `bitcoin-core` | 0 each; stdout contains `XA` | the check placed in a shared pipeline step all formats run |
| `bip388v_guard_build_descriptor_other_outputs_emit` | `BUILD(--format descriptor)`; `BUILD()` | 0; stdout starts `wsh(or_d(pk(`; human view starts `descriptor:\n` | eager computation of the policy |
| `bip388v_guard_disjoint_multipath_keeps_suffix_text` | `EXP(bip388, wsh(or_d(pk(KA/<0;1>/*),and_v(v:pk(KA/<2;3>/*),older(100)))))` | 1; ""; `error: --format bip388 requires every descriptor key to end in `/<0;1>/*` for the receive/change pair; got key "KA/<2;3>/*"` | DUP placed before the suffix rule |
| `bip388v_guard_literal_leaf_key_keeps_suffix_text` | `EXP(bip388, wsh(or_d(pk(KA/<0;1>/*),and_v(v:pk(02<NUMS>),older(100)))))` | 1; ""; suffix text naming `02<NUMS>` | LIT for every literal key |
| `bip388v_guard_template_tr_repeated_slot_refused_upstream` | tr-sortedmulti-a, nums, threshold 1, two slots both `XA`/`73c5da0a`/`m/48'/1'/0'/2'`, bip388 | 2; ""; `error: export-wallet descriptor parse: Miniscript contains repeated pubkeys or pubkeyhashes` | a new check moved ahead of the strict parse |

Measured: RED run on `00947cb7` with the test changes of section 8 (`/scratch/cache/bip388v-red-final.log`):
`27 tests run: 8 passed, 19 failed` (the 17 rows of 6.1, `allow_repeated_keys_bip388_refused` and
`tr_multi_a_bip388_wallet_policy_with_nums` fail; the 7 guards and the rewritten
`export_wallet_tr_sortedmulti_bip48_emits_advisory` pass). Each failure is the "today" cell above (first failure
for the template row: `left: (0, "{\n  \"description_template\": \"wsh(sortedmulti(1,@0/**,@1/**))\"…`).

## 7. Coverage (BIP text and ruled text → rows)

| sentence | row |
|---|---|
| BIP-388 L149-162, L310: only `KP` key placeholders; a non-KP key is invalid; ruled LIT text, exit 1 | the four `p25_*` rows, `bughunt_tk_21_…`, `bip388v_export_descriptor_nums_refused` |
| LIT checked before the multipath rule (section 2.2) | `bip388v_export_descriptor_nums_literal_before_multipath` |
| LIT names only an origin-less `tr` internal key | `bip388v_guard_literal_leaf_key_keeps_suffix_text`; the origin-bearing internal key: no row; probed with the patched binary (`tr([73c5da0a/86'/1'/0']<NUMS>,sortedmulti_a(1,KA/<0;1>/*,KB/<0;1>/*))`: exit 1, the suffix text naming `[73c5da0a/86'/1'/0']<NUMS>`) |
| BIP-388 L193: `keys_info` public keys pairwise distinct; DUP, exit 1, stdout empty, no file | the seven `bip388v_export_*repeated*` / `*same*` rows, `bip388v_restore_repeated_key_bip388_refused` |
| compared on the public key, not origin or chain code | `…same_xpub_other_origin…`, `…same_xpub_other_path…`, `…same_pubkey_other_chaincode…` |
| first pair, `@i`/`@j`, `<xpub>` of `@i` | every DUP row (0,1); `allow_repeated_keys_bip388_refused` (0,2) |
| DUP after the suffix rule | `bip388v_guard_disjoint_multipath_keeps_suffix_text` |
| build-descriptor: DUP-BUILD after the banner; `--json` null; other outputs unchanged | `bip388v_build_descriptor_*`, `bip388v_guard_build_descriptor_other_outputs_emit` |
| the named alternatives still emit | `p25_controls_…`, `bip388v_guard_named_alternatives_emit_repeated_key` |
| section 2.4 text | `bip388v_restore_general_tr_text_names_non_nums` |
| manual / CHANGELOG sentences | the docs CI jobs (section 9) |

## 8. Tests that change

Found by running `cargo nextest run --locked --workspace --no-fail-fast` once with the **source-only** patch
applied (log `/scratch/cache/bip388v-ref-nextest2.log`): `4419 tests run: 4416 passed, 3 failed, 38 skipped`; the
failures were exactly these three:

1. `tests/cli_export_wallet.rs::tr_multi_a_bip388_wallet_policy_with_nums`: rewritten as the architect's diff
   (`/scratch/cache/p25-keep/fix.diff`): doc comment `/// Sweep P25 — taproot multisig + BIP-388 with a NUMS
   internal key refuses: / BIP-388 allows only `@N` key placeholders, never a literal key.`; asserts `.code(1)`,
   `.stdout("")`, stderr contains `error: --format bip388 cannot express the literal key {NUMS_HEX} (a taproot
   internal key with no origin`.
2. `tests/cli_tr_bip48_advisory.rs::export_wallet_tr_sortedmulti_bip48_emits_advisory`: its `--format bip388`
   becomes `--format descriptor` (the row tests the BIP-48 advisory, not bip388). Passes before and after.
3. `tests/cli_build_descriptor.rs::allow_repeated_keys_bip388_duplicate_keys_info` (the R0-r1 M5 pin): renamed
   `allow_repeated_keys_bip388_refused`, same `--allow repeated-keys --format bip388` with `repeated_keys_spec()`
   (K1,K2,K1,K2); asserts `.code(1)`, `.stdout("")`, stderr ends with DUP-BUILD(0, 2, `<K1's xpub after "]">`).
   Its doc comment says it replaces the M5 pin (S2).

Plus one ignored row un-ignored: `tests/bughunt_s3.rs::bughunt_tk_21_bip388_export_from_descriptor_taproot_nums_multisig`
(drop its `#[ignore = "deferred-minor: bip388-literal-nums-key-refusal-text"]`; body unchanged; passes with the
patch). No golden file changes.

## 9. Gate (the builder runs all of it; capture each run once to a file and grep it)

`PATH=$HOME/.cargo/bin:$PATH`; `cargo --version` must be 1.85.0; private `CARGO_TARGET_DIR` under `/scratch/cache`;
never `--release`.

1. RED commit: the new test file, the `cli_restore_taproot.rs` row, the section 8 edits and the un-ignore;
   production code untouched. `cargo nextest run --locked -p mnemonic-toolkit --no-fail-fast -E
   'binary(cli_bip388_validity) | test(bughunt_tk_21_) | test(=allow_repeated_keys_bip388_refused) |
   test(=tr_multi_a_bip388_wallet_policy_with_nums) | test(=export_wallet_tr_sortedmulti_bip48_emits_advisory) |
   test(=bip388v_restore_general_tr_text_names_non_nums)'`: exactly **27 run, 19 failed, 8 passed**. Paste the
   counts.
2. GREEN commit: sections 3, 4, 5. `cargo fmt --all -- --check`; `cargo clippy --locked --workspace --all-targets
   -- -D warnings`; `cargo nextest run --locked --workspace --no-fail-fast` 0 failed (reference:
   `4443 tests run: 4443 passed, 37 skipped`, `/scratch/cache/bip388v-green-full3.log`); `cargo test --locked
   --doc --workspace`; the step-1 filter: exactly **27 run, 27 passed** (a short count is a FAIL).
3. Docs CI jobs on the head (locally where possible, else green CI on the PR): `manual.yml` (incl. the toolkit
   `manual` job and the doc-flag-lint mutation test), `manual-gui.yml`, `technical-manual.yml`, `quickstart.yml`,
   `examples.yml`; the rest of `rust.yml` and `fuzz-smoke.yml` as green CI.
4. `coordinator/scripts/mutation-gate.sh <worktree> 00947cb7` after step 2; its missed list goes in the review
   brief. Risk class A (CLI text, manual).

Not covered by this gate: the mnemonic-gui build-descriptor form (it passes `--json`; grep of
`mnemonic-gui/src` finds `bip388` only in the `schema/mnemonic.rs` format lists, so a `null` `bip388` is not read);
the origin-bearing `tr` internal key boundary (probed, no row); hardware-signer behaviour.

## 10. WORDING_REVIEW proposals (mnemonic-toolkit has no `design/WORDING_REVIEW.md`; logged here)

| id | text | prints | status |
|---|---|---|---|
| SEC-B388-W1 | LIT | `export-wallet` / `restore` stderr, exit 1 | ruled (FOLLOWUPS L6218, P25) |
| SEC-B388-W2 | DUP | `export-wallet` / `restore` stderr, exit 1 | proposed; mirrors LIT's shape and advice |
| SEC-B388-W3 | DUP-BUILD | `build-descriptor` stderr, exit 1 | proposed; DUP with the one format build-descriptor has |
| SEC-B388-W4 | section 2.4's `(A non-NUMS distinct-trunk tr-multisig md1 DOES export bip388 via its template path.)` | `restore` stderr, exit 1 | proposed; two words inserted |
| SEC-B388-W5 | the section 5 manual and CHANGELOG sentences | manuals | proposed |

## 11. STOPs (product points the rulings do not decide; recommendation given, document written for it)

**S1. What `build-descriptor --json` writes for `bip388` when the policy has no BIP-388 form.** Today `emit`
computes the policy for every output and would propagate the refusal into `--format descriptor`, the human view and
`--json`. Options:

- (a) `"bip388": null`, exit 0, everything else unchanged (this document). Precedent: the same envelope already
  writes `"cost": null` on an allowed-insane emit (allow SPEC §3, R0-r1 C1). The GUI does not read the key.
- (b) Drop the `bip388` key from the envelope. Row `…json_bip388_null` asserts the key is absent; manual 5.3 / 5.7
  say "omitted".
- (c) Refuse the whole `--json` run (exit 1, DUP-BUILD). Breaks the GUI build of every allowed repeated-keys
  policy; row `…json_bip388_null` becomes a refusal row.

Recommendation: (a).

**S2. Does `build-descriptor --allow repeated-keys --format bip388` refuse?** This reverses SPEC
`SPEC_descriptor_builder_allow.md` R0-r1 M5 ("pin the shape"; signer behaviour "out of toolkit scope"). Options:

- (a) Refuse with DUP-BUILD (this document; section 4 is the erratum). For: the grade names M5 as the prior
  accepted note and still grades I/FIX; `--allow` waives a miniscript sanity rule, not BIP-388's format rule; a
  BIP-388 signer rejects or misreads the policy; the shared converter gives one rule on every surface.
- (b) Keep emitting on build-descriptor only (the converter takes a "check distinctness" flag). Rows
  `bip388v_build_descriptor_*` become guards with today's output; section 8 item 3, section 4 and manual 5.3, 5.4,
  5.7 are dropped; DUP-BUILD is not needed.

Recommendation: (a). S1 applies only under (a).

**S3. Which key equality is "pairwise distinct"?** BIP-388 L193 says "the public keys obtained by deserializing
elements of the key information vector". Options:

- (a) The 33-byte public key of each extended key (this document): stricter; refuses one public key under two
  chain codes.
- (b) The whole extended key: row `bip388v_export_descriptor_same_pubkey_other_chaincode_refused` becomes a guard
  (exit 0, two `keys_info` entries); every other row is unchanged.

Recommendation: (a). The BIP says "public keys", not "extended public keys" (L187 uses "serialized extended public
key" for the element itself); its footnote gives the reason as pubkey reuse in miniscript; and refusing fails closed, as
the named alternatives still emit the wallet. Only a crafted key reaches the difference.

Not STOPs (decided by the BIP text or precedent): one xpub under two origins refuses (both readings of L193); exit
1 (every `--format bip388` refusal is `BadInput`); refusal order (section 2.2: each order keeps every existing text
on the inputs it reaches today).

## 12. FOLLOWUPS (not appended in this branch; listed in the author report for a direct push)

1. Toolkit FOLLOWUPS `bip388-literal-nums-key-refusal-text` (L6215-6222; L6218 the ruled fix, L6219 "Open for the
   architect"): converted to the new format and closed by the GREEN commit of this unit (the template arm is
   ruled, both arms are built, the bughunt row is un-ignored).
2. New: `bip388-keyinfo-extra-derivation-step`. `export-wallet --descriptor 'wsh(or_d(pk(KA/5/<0;1>/*),…))'
   --format bip388` emits `keys_info` entry `…XA/5` at exit 0 (a `KEY_INFO` has no trailing path, BIP-388
   L180-187). Measured at `00947cb7`.
3. New: `bundle-distinctness-origin-blind`. `bundle`'s rule (SPEC v0.5 §4.11.b, "Two slots with identical xpubs
   but different non-empty paths are NOT colliding (BIP-388 letter; …)") accepts a key vector BIP-388 L193 calls
   invalid, and `bundle --descriptor` accepts a policy that repeats `@0` with one multipath (a non-disjoint
   BIP-388 template); both now refuse at `--format bip388`, not at `bundle`.
