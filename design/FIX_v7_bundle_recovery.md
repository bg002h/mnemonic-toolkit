# FIX-V7: bundle card recovery summary for general policies

Status: draft (author). Base: `ac3ff70a` + RED commit `283132e7`, branch `fix/v7-bundle-recovery`.

## 1. Inputs and scope

- Grade and rulings (bind): coordinator `agent-reports/grade-2026-11-03-tk-v5-v7.md`, rulings 1, 2, 3 and 7. V7-1..V7-11 are Critical.
- Small-fix track: no new flag, type or command. One repo (mnemonic-toolkit) plus its manual lockstep.
- Code files (ruling 7): `crates/mnemonic-toolkit/src/cmd/bundle.rs`, `src/format.rs` (card), `src/format.rs` `MultisigInfo` (JSON writer), `src/wallet_import/json_envelope.rs` `MultisigInfoView` (JSON reader), `src/cmd/export_wallet.rs` (reader consumer), `src/cmd/import_wallet.rs` (emitter).
- The builder works from this document and the committed appendix `design/FIX_v7_appendix/`. Nothing in the appendix is production code.
- Not in scope: T3 (lifting the `sortedmulti_a` whole-leaf gate) waits for this FIX to merge.

## 2. Spec delta (exact texts)

### 2.1 Plain predicate (ruling 1, ruling 3)

A descriptor is **plain** when exactly one of these holds. Otherwise it is **general**.

1. The root is `wsh`, `sh`, or `sh(wsh(..))`, and its only child is a k-of-n node.
2. The root is `tr` with internal key `NUMS` (md `InternalKey::NumsPoint`), and its script tree is a single leaf (not a branch) that is a k-of-n node.

A **k-of-n node** is one of the following:

- `multi`, `sortedmulti`, `multi_a` or `sortedmulti_a`, with its k;
- a **pure-key `thresh`**, with its k.

A pure-key `thresh` is one where every argument is `pk(K)` or `pkh(K)` under zero or more `s:`/`a:` wrappers and nothing else. Any other wrapper (`n:`, `l:`, `v:`, `d:`, `j:`, `u:`, `t:`) or argument (`multi`, `older`, `after`, a hash, a nested combinator) makes the `thresh` general.

Further rules:

- A `tr` internal key that is a real key (md `InternalKey::Slot`), or the Liana unspendable key (`InternalKey::LianaUnspendable`), makes the descriptor general, whatever the leaves are.
- A taptree with two or more leaves is general.
- k is the k of the k-of-n node. n is the bundle's signing-key count as today (the cosigner count), unchanged.
- **Template mode** (`mnemonic bundle` without `--descriptor`/`--descriptor-file`/`--import-json`) is always plain, with k = `--threshold` if given, else n. This is unchanged.

### 2.2 Engraving card (stderr), `mnemonic bundle`

| case | `# Threshold:` line | `# Recovery:` line |
|---|---|---|
| n == 1 (any policy) | absent (unchanged) | absent (unchanged) |
| n > 1, plain | `# Threshold: {k} of {n}` (unchanged) | `# Recovery: any {k} of {n} signing keys + md1 (template card).` (unchanged) |
| n > 1, general | **absent** | **`# Recovery: general policy, not k-of-n (the md1 policy decides which key sets can spend); keep all {n} signing keys, any hash preimages it names, and md1 (template card).`** |

The general Recovery line is one line: there is no wrap and the line ends with `\n`. It sits where the plain Recovery line sits today. Every other card line is unchanged. Exit code unchanged (0).

### 2.3 Bundle JSON (`mnemonic bundle --json`)

- `multisig.threshold`: JSON integer k for plain, JSON `null` for general. Its type becomes `Option<u8>`, serialized as `null` rather than omitted. Every other field is unchanged. `multisig` stays absent when n == 1, as today.
- Reader: `MultisigInfoView.threshold` becomes `Option<u8>`. An envelope with `"threshold": null` is accepted by every `--import-json` / `--from-import-json` consumer:
  - `mnemonic bundle --import-json`;
  - `mnemonic export-wallet --from-import-json`.
- `mnemonic bundle --import-json` ignores the envelope's `threshold`, as today. It recomputes k from the descriptor by 2.1, so a stale integer for a general descriptor still yields the general card.
- `export-wallet --from-import-json` passes no threshold (`None`) from a null envelope.
  - `--format descriptor` / `bitcoin-core` / `bip388` output is byte-identical to the same envelope carrying an integer.
  - `--format sparrow` and the other template formats keep their existing general-policy refusal, at exit 1, with the unchanged text:
    `error: --from-import-json: --format sparrow cannot represent a general wallet policy (timelocks/hashlocks/non-multisig miniscript); it is a plain k-of-n multisig format. Use --format descriptor / bitcoin-core / bip388 for faithful descriptor passthrough.`

### 2.4 `mnemonic import-wallet`

- After the input is parsed and the entry selected, every entry's threshold is recomputed from its descriptor by 2.1. It is k when plain, none when general, and it replaces whatever the source format carried. This applies, for example, to BSMS, which declares a k for a decaying policy.
- `--json`: `[i].bundle.multisig.threshold` is k or `null` by the same rule (via 2.3's type).
- Text summary: the existing `none` arm prints `bundles[{i}].threshold=none` and `threshold=none` for general entries; plain entries print the integer as today.
- n == 1 entries print as today. `wsh(multi(1,K0))` is plain, k = 1. A single-key entry prints `none`.

## 3. Production change (what, not how)

- `bundle.rs`: add the 2.1 predicate as one function over the md `tree::Node`. It returns `Option<u8>`: k when plain, `None` when general.
  - Descriptor mode and the `--import-json` path take k from it.
  - Template mode keeps `--threshold`, or n when the flag is absent.
  - A hint implementation sits outside the repo, at `/scratch/cache/tk-fix-v7-logs/fix-v7-production.hint.patch`; it is not normative.
- `format.rs`:
  - `MultisigInfo.threshold: Option<u8>`.
  - The unified card prints the 2.2 table.
- `json_envelope.rs`:
  - `MultisigInfoView.threshold: Option<u8>`.
  - Its two existing unit tests that build a view change `2` to `Some(2)`.
- `export_wallet.rs`: read the threshold as `Option` (`and_then`).
- `import_wallet.rs`: the 2.4 recompute. The emitter uses the recomputed value.
- STOP-2 (section 8) decides how `bundle --import-json` treats `--threshold`. Until it is ruled, the builder does not start (section 8).

## 4. RED rows
## 5. Tests that change
## 6. Lockstep files and transcripts
## 7. Gate
## 8. STOPs
## 9. Coverage map (ruling -> rows)
