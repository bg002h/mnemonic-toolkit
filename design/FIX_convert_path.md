# FIX: `mnemonic convert --path` is ignored on seed and xprv edges (C), and `--help` lists path forms that are refused (M)

Small-fix track (RULES.md "Process"). Branch `triage/convert-path`, base `origin/master` `6ccfdebc`. Before this document, the branch carries only two agent reports: `7f5a535e` (the triage) and `a527b9b2` (the grade and ruling). This document's commit adds the RED test file `crates/mnemonic-toolkit/tests/cli_convert_path.rs`. It does not change production code.

Sources, all in `design/agent-reports/`:

- `triage-convert-path.md` (triage: CONFIRMED)
- `grade-convert-path.md`, which holds the grades and the binding ruling R1-R5 (lines 64-100). This document applies that ruling. Section 10 lists the three points the ruling leaves open, each as a STOP.

Grades: **C** for the phrase, seedqr and entropy edges; **I** for the xprv source edge, which RED row `r4_xprv_to_xpub_path_refused` confirms; **M** for the named and hex forms in `--help`, folded into scope (R5).

**How this document was checked.**

- The code and manual edits were applied in this worktree as a scratch change, then reverted. They are saved as a reference patch, `/scratch/cache/convert-path/fix-reference.patch` (sha256 `835f13266468ddae1bcd9f8cc8fc8491fddd6139818d22028a3d42e46415cc43`, 2 files, 131 lines). `git apply --check` passes at `a527b9b2`. Section 3 is that patch's convert.rs part, written out. Section 5.1 is its manual part.
- RED, on unmodified production code (`git diff --stat` empty; the binary was rebuilt from that tree): `cargo nextest run --locked -p mnemonic-toolkit --no-fail-fast -E 'binary(cli_convert_path)'` gave 27 run, 2 passed, 25 failed. The 2 passes are the guard rows `r3_guard_explicit_account_zero_with_path_accepted` and `r4_guard_xpub_to_address_path_accepted`. Capture: `/scratch/cache/convert-path/red-final.txt` (sha256 `cdc96503815b333f67e930f2e5cb658eac1f9952b38b9686db4e5bc4fbcc2611`). Each row's failure reason is in `red-reasons.txt` (sha256 `aee2059b…f3e7`) and in the "today" column of section 6.
- GREEN, with the reference patch applied: the same filter gave 27/27 passed (`green3.txt`, sha256 `f7b04652…e170`).
  - Whole workspace on the patched tree: `cargo nextest run --locked --workspace --no-fail-fast` gave 4171 run, 4171 passed, 38 skipped (`full-fix.txt`, sha256 `aee972e4…df5`).
  - `cargo clippy --locked --all-targets --workspace -- -D warnings`: exit 0.
  - `cargo fmt --all --check`: exit 0.
- All runs used `PATH=$HOME/.cargo/bin:$PATH` (cargo 1.85.0, the `rust-toolchain.toml` pin), `--locked`, and no `--release`.
- Not run locally. These are the builder's gate items (section 8):
  - The `docs/manual` targets `lint`, `verify-examples` and `anchor-check`. In this environment they fail for environmental reasons: the sibling `md`/`ms`/`mk` binaries are not at their pins, and `tk-worktrees/mnemonic-key` is missing.
  - `.examples-build/gen.sh`.
  - The technical-manual, quickstart, manual-gui and fuzz-smoke workflows.
  - `mutation-gate.sh`.
- No example, transcript, `.examples-build` page or GUI schema runs `convert --path`. `grep -rn -- '--path' .examples-build docs/manual/src` finds only the two manual rows of section 5.1 inside convert's section. The named/hex text is in convert.rs and in `docs/manual-gui/src/50-md/53-encode.md`, which documents `md encode`, not convert, and does not change.

## 1. Summary

1. **C. On a phrase, seedqr or entropy source, `--to xpub` / `--to xprv` ignore `--path`.** They derive at the `--template` path and exit 0, so the holder gets a key at a path they did not ask for, with no sign of it. Without `--template` they refuse with `--template is required…`, even though a path was given. Fix (R1): with `--path`, xpub and xprv derive at that path from the master key. `--template` becomes optional, and `--path` wins over it.
2. **I. On an xprv source, `--to xpub --path m/0` prints the parent xpub and exits 0.** The Xprv arm never reads `--path`. Fix (R4): `--path` that no requested target reads is refused (exit 2). This covers the xprv, xpub-to-non-address, mk1 and ms1 edges, and non-derivation targets from a seed.
3. **R2.** `--to fingerprint` stays the master fingerprint whether or not `--path` is given. The no-template refusal becomes `--template or --path is required for derivation targets (xpub/xprv/fingerprint)` (exit 1).
4. **R3.** A non-zero `--account` together with `--path` is refused (exit 2). An explicit `--account 0` is accepted: clap's default leaves it indistinguishable from an omitted `--account`.
5. **M / R5.** `--path` takes only the literal `m/...` form, bare `m` included. `--help` stops listing the named (`bip84`) and hex (`0x54`) forms. These were never accepted by convert: every read uses `DerivationPath::from_str`, which refuses them with `error: --path parse: invalid child number format` (exit 1). That behaviour is unchanged. There is no coin-type check.

## 2. Holder-visible behaviour (after the fix)

All refusals print `error: <text>` on stderr and nothing on stdout. Exit 2 is `ConvertRefusal`. Exit 1 is `BadInput`.

| case | exit | stderr / stdout |
|---|---|---|
| seed source (phrase / seedqr / entropy), `--to xpub` and/or `xprv`, `--path P` (with or without `--template`) | 0 | the key at P from the master key. Every target in the call (xpub, xprv, wif, bip38, address) uses the same P |
| the same, `--to fingerprint` among the targets | 0 | `fingerprint: <master fingerprint>` (unchanged by P) |
| seed source, a derivation target, neither `--template` nor `--path` | 1 | `error: --template or --path is required for derivation targets (xpub/xprv/fingerprint)` |
| `--path` given, and no requested `--to` target reads it from this source (table below) | 2 | `error: --path is not used by any requested --to target from this --from source; drop --path.` |
| `--path` given, `--account` non-zero | 2 | `error: --account has no effect when --path is given; put the account in --path (e.g. m/84'/1'/3') or drop --path.` |
| `--path` in a non-literal form (`bip84`, `0x54`, anything `DerivationPath::from_str` refuses) on an edge that reads it | 1 | `error: --path parse: <bitcoin error>` (for example `invalid child number format`); unchanged |
| `--from wif … --path` | 2 | the existing wif refusal (§4). It is checked before R4 and R3 and is unchanged |

The (source, target) edges that read `--path` (R4) are the following. Every other source, including xprv, mk1, ms1, wif, electrum-phrase, bip38 and minikey, reads it on no target.

| `--from` | targets that read `--path` |
|---|---|
| `phrase`, `seedqr`, `entropy` | `xpub`, `xprv`, `wif`, `bip38`, `address` |
| `xpub` | `address` |

The refusal fires when none of the requested targets is in the source's row. So `--to xpub,fingerprint --path P` from a phrase is accepted (xpub reads P), while `--to fingerprint --path P` alone is refused.

## 3. Code delta (exact), `crates/mnemonic-toolkit/src/cmd/convert.rs`

Line numbers are at `a527b9b2`.

3.1 The import at line 5. Old:

```
use crate::derive_slot::{derive_bip32_at_path, derive_bip32_from_entropy};
```

New:

```
use crate::derive_slot::{
    derive_bip32_at_path, derive_bip32_from_entropy, derive_bip32_from_entropy_at_path,
};
```

3.2 The `--path` doc comment (lines 234-240). Keep `#[arg(long, verbatim_doc_comment)]`. Old:

```
    /// BIP-32 derivation path. Accepts:
    ///   named:    `bip44` / `bip49` / `bip84` / `bip86`
    ///   hex:      `0xNN` (raw purpose byte)
    ///   literal:  `m/...` (full BIP-32 path string)
    ///
    /// Required for `--to wif` (no template-implied path) and for
    /// `(xpub, address)` derivation when no `--template` is supplied.
```

New:

```
    /// BIP-32 derivation path from the master key, literal form `m/...`. Overrides the
    /// --template/--account path for --to xpub/xprv; required for --to wif, --to address
    /// (from a seed) and composite --to bip38. Refused where no requested target uses it.
```

3.3 The `--account` doc comment (lines 285-286). Old:

```
    /// BIP-32 account index (default 0). Used to compute the
    /// template-derived path when `--path` is omitted.
```

New:

```
    /// BIP-32 account index (default 0). Used to compute the
    /// template-derived path when `--path` is omitted. Refused (non-zero)
    /// together with --path.
```

3.4 Insert immediately before `fn refusal_wif_with_path()` (line 509):

```
/// Ruling R4: which (source, target) edges read `--path`.
fn reads_path(from: NodeType, to: NodeType) -> bool {
    use NodeType::*;
    match from {
        Seedqr | Phrase | Entropy => matches!(to, Xpub | Xprv | Wif | Bip38 | Address),
        Xpub => to == Address,
        _ => false,
    }
}

fn refusal_path_unused() -> ToolkitError {
    ToolkitError::ConvertRefusal(
        "--path is not used by any requested --to target from this --from source; drop --path."
            .into(),
    )
}

fn refusal_account_with_path() -> ToolkitError {
    ToolkitError::ConvertRefusal(
        "--account has no effect when --path is given; put the account in --path (e.g. m/84'/1'/3') or drop --path."
            .into(),
    )
}
```

3.5 In `run()`, insert immediately after the block that ends `return Err(refusal_wif_with_path());` and its closing `}` (line 953), and before the `// 5.a) SPEC §11.a` comment:

```
    // 5.0a) Ruling R4: `--path` must be read by at least one requested target.
    if args.path.is_some() && !targets.iter().any(|&t| reads_path(primary.node, t)) {
        return Err(refusal_path_unused());
    }

    // 5.0b) Ruling R3: a non-zero `--account` is refused together with `--path`.
    if args.path.is_some() && args.account != 0 {
        return Err(refusal_account_with_path());
    }
```

The order of 5.0a and 5.0b, and their placement before 5.a, are ruled (section 10, STOP-1 and STOP-2: proposals stand; architect bughunt-fixdocs QT1, QT2). Build them as written. The R3 check also runs before the R5 parse error (STOP-3, QT3).

3.6 In `compute_outputs`, the seed arm (lines 1302-1311). Old:

```
            let derived = if needs_derive {
                let template = args.template.ok_or_else(|| {
                    ToolkitError::BadInput(
                        "--template is required for derivation targets (xpub/xprv/fingerprint)"
                            .into(),
                    )
                })?;
```

New:

```
            let derived = if needs_derive && args.path.is_some() {
                // Ruling R1: --path overrides the template path; R2: the
                // fingerprint stays the master fingerprint.
                let path_str = args.path.as_deref().unwrap_or_default();
                let path = bip32::DerivationPath::from_str(path_str)
                    .map_err(|e| ToolkitError::BadInput(format!("--path parse: {e}")))?;
                Some(derive_bip32_from_entropy_at_path(
                    &entropy,
                    pbkdf2_passphrase,
                    language.into(),
                    network,
                    &path,
                )?)
            } else if needs_derive {
                let template = args.template.ok_or_else(|| {
                    ToolkitError::BadInput(
                        "--template or --path is required for derivation targets (xpub/xprv/fingerprint)"
                            .into(),
                    )
                })?;
```

The rest of the `else if needs_derive` branch is unchanged. The fingerprint target keeps reading the master fingerprint. `derive_bip32_from_entropy_at_path` returns the master fingerprint, as `r2_path_fingerprint_is_master` checks. No other file under `crates/` changes.

## 4. Spec delta, `design/SPEC_convert_v0_6.md`

Each old text occurs exactly once (checked with `grep -c`).

**S1. Line 77** (phrase row, mechanism cell). Old fragment:

`then `derive_slot::derive_bip32_from_entropy` (yields `DerivedAccount` with `account_xpriv` + `account_xpub`); equivalent to calling `derive::derive_full``

New fragment:

`then `derive_slot::derive_bip32_from_entropy` (yields `DerivedAccount` with `account_xpriv` + `account_xpub`); equivalent to calling `derive::derive_full`. With `--path`, `derive_slot::derive_bip32_from_entropy_at_path` at that path from the master key instead; `--path` overrides `--template`/`--account` and `fingerprint` stays the master fingerprint`

**S2. Line 78** (entropy row). Old fragment: `feed entropy bytes directly to `derive_slot::derive_bip32_from_entropy` (no parse step)`. New fragment: `feed entropy bytes directly to `derive_slot::derive_bip32_from_entropy` (or `derive_bip32_from_entropy_at_path` when `--path` is given; no parse step)`

**S3. Line 111.** Old fragment: `are both produced by a single `derive_slot::derive_bip32_from_entropy` call`. New fragment: `are both produced by a single `derive_slot::derive_bip32_from_entropy` call (or `derive_bip32_from_entropy_at_path` when `--path` is given)`

**S4. Line 326.** Old fragment: `call `crate::derive_slot::derive_bip32_from_entropy` once;`. New fragment: `call `crate::derive_slot::derive_bip32_from_entropy` (or `derive_bip32_from_entropy_at_path` when `--path` is given) once;`

**S5. §4, insert after the wif bullet's closing code fence (line 222)**, before the bullet `- Any `--to <node>` not reachable…`:

````
- `--path` given and no requested `--to` target reads it from the `--from` source: REFUSED (exit 2). Only these edges read `--path`: `phrase`/`seedqr`/`entropy` → `xpub`, `xprv`, `wif`, `bip38`, `address`; `xpub` → `address`. Checked after the wif refusal above. Stderr (byte-exact):
  ```
  error: --path is not used by any requested --to target from this --from source; drop --path.
  ```
- `--path` given with a non-zero `--account`: REFUSED (exit 2). An explicit `--account 0` is accepted (indistinguishable from the default). Stderr (byte-exact):
  ```
  error: --account has no effect when --path is given; put the account in --path (e.g. m/84'/1'/3') or drop --path.
  ```
- `--path` accepts only the literal form `m/...` (bare `m` is the master key); any other string is refused at the edge that reads it with `error: --path parse: <e>` (exit 1), `<e>` being the `bitcoin::bip32` parse error (e.g. `invalid child number format`). No coin-type check.
- A seed source with a derivation target (`xpub`/`xprv`/`fingerprint`) and neither `--template` nor `--path`: REFUSED (exit 1). Stderr (byte-exact):
  ```
  error: --template or --path is required for derivation targets (xpub/xprv/fingerprint)
  ```
````

**S6. Line 304.** Old: `| `--template` + `--account` | edge derives at a template path (substitutes for explicit `--path`) |`. New: `| `--template` + `--account` | edge derives at a template path when `--path` is absent |`

**S7. Line 305.** Old: `| `--path` | edge derives at a custom BIP-32 path (mutually exclusive with `--template`) |`. New: `| `--path` | edge derives at a custom BIP-32 path from the master key, literal `m/...` only; overrides `--template`/`--account` for `xpub`/`xprv`; `fingerprint` stays the master fingerprint |`

**S8. Line 308.** Old fragment: `Side-inputs that are not meaningful for the chosen edge are IGNORED (not refused).` New fragment: `Side-inputs that are not meaningful for the chosen edge are IGNORED (not refused), except `--path` (refused when no requested target reads it) and a non-zero `--account` given with `--path` (§4).`

## 5. Manual and help delta

5.1 `docs/manual/src/40-cli-reference/41-mnemonic.md`, lines 1102-1103. Line 1158 repeats the `--account` row in another command's table, so match the two lines as a pair. Old:

```
| `--path <PATH>` | derivation path override |
| `--account <ACCOUNT>` | account index (default 0) |
```

New:

```
| `--path <PATH>` | BIP-32 path from the master key, literal `m/...` only. For `--to xpub`/`--to xprv` it overrides the `--template`/`--account` path (and makes `--template` optional); required for `--to wif`, `--to address` from a seed, and composite `--to bip38`. Does not change `--to fingerprint` (always the master fingerprint). Refused (exit 2) with a non-zero `--account`, or when no requested target uses it |
| `--account <ACCOUNT>` | account index (default 0); refused (exit 2) when non-zero together with `--path` |
```

5.2 `mnemonic convert --help` after the fix. These are lines of the reference build's output, with leading whitespace trimmed; both rows assert them:

```
--path <PATH>
BIP-32 derivation path from the master key, literal form `m/...`. Overrides the
--template/--account path for --to xpub/xprv; required for --to wif, --to address
(from a seed) and composite --to bip38. Refused where no requested target uses it.
```

```
BIP-32 account index (default 0). Used to compute the template-derived path when `--path` is omitted. Refused (non-zero) together with --path
```

(Clap joins the non-verbatim `--account` doc into one line and drops its final period.) The help contains no `named:` and no `hex:`.

5.3 No other manual changes. The GUI manual (`docs/manual-gui/src/40-mnemonic/44-convert.md:380-385` and `:472`) already says that `--path` overrides the template path, which stays accurate. It omits the two new refusals; that is a Minor (section 9, F2), not part of this build.

## 6. RED rows

File: `crates/mnemonic-toolkit/tests/cli_convert_path.rs`, committed with this document. The toolkit has no `tests/all.rs`: each `tests/*.rs` file is its own binary, so nothing else is registered. Every row runs `mnemonic --allow-argv-secret convert <args>`.

Constants:

| name | value |
|---|---|
| `P` | `phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about` (master fingerprint `73c5da0a`) |
| `SQ` | `seedqr=000000000000000000000000000000000000000000000003` |
| `EN` | `entropy=00000000000000000000000000000000` |
| `S` | `--network signet` |

The keys are the test file's constants: `XPUB_87`, `XPRV_87`, `WIF_87` at `m/87'/1'/0'`, `XPUB_48`, `XPRV_48`, `WIF_48` at `m/48'/1'/0'/2'`, `XPUB_84` and `XPRV_84` (BIP-84 account), `XPUB_M` (master). "R3 text", "R4 text" and "R5 text" are the stderr lines in section 2, with R5 being `error: --path parse: invalid child number format`. A row passes when the exit code matches and every expected line is a whole line of the named stream.

| row | args | expected | today (unmodified) |
|---|---|---|---|
| `r1_phrase_path_xpub_xprv_without_template` | `--from P S --to xpub,xprv --path m/87'/1'/0'` | 0; `xpub: XPUB_87`, `xprv: XPRV_87` | exit 1, `--template is required…` |
| `r1_seedqr_path_xpub_xprv_without_template` | `--from SQ S --to xpub,xprv --path m/87'/1'/0'` | same | same |
| `r1_entropy_path_xpub_xprv_without_template` | `--from EN S --to xpub,xprv --path m/87'/1'/0'` | same | same |
| `r1_path_wins_over_template` | `--from P S --template bip84 --to xpub,xprv --path m/87'/1'/0'` | 0; the 87 keys; stdout has neither `XPUB_84` nor `XPRV_84` | exit 0 with `XPUB_84` (path ignored) |
| `convert_template_xpub_path_not_silently_ignored` (the triage row) | `--from P --template wsh-sortedmulti S --to xpub`, then the same plus `--path m/99'/1'/5'` | the second run exits non-zero or prints different stdout | both print `XPUB_87` and exit 0 |
| `r1_path_shared_by_xpub_and_wif` | `--from P S --to xpub,wif --path m/87'/1'/0'` | 0; `xpub: XPUB_87`, `wif: WIF_87` | exit 1, `--template is required…` |
| `r1_phrase_bip48_path_xpub_xprv_match_wif` | `--from P S --to xpub,xprv,wif --path m/48'/1'/0'/2'` | 0; `XPUB_48`, `XPRV_48`, `WIF_48` lines | exit 1, `--template is required…` |
| `r2_path_fingerprint_is_master` | `--from P S --to xpub,fingerprint --path m/87'/1'/0'` | 0; `xpub: XPUB_87`, `fingerprint: 73c5da0a` | exit 1, `--template is required…` |
| `r2_neither_template_nor_path_names_both` | `--from P S --to xpub` | 1; `error: --template or --path is required for derivation targets (xpub/xprv/fingerprint)` | exit 1 with the old text |
| `r3_account_nonzero_with_path_refused_wif` | `--from P S --to wif --path m/84'/1'/0' --account 3` | 2; R3 text | exit 0, prints a wif |
| `r3_account_nonzero_with_path_refused_template_xpub` | `--from P S --template wsh-sortedmulti --account 3 --to xpub --path m/87'/1'/3'` | 2; R3 text | exit 0, prints the template xpub |
| `r3_guard_explicit_account_zero_with_path_accepted` (guard) | `--from P S --to wif --path m/84'/1'/0' --account 0` | 0; `wif: cRR3sz6fBG6cWeuJE8cRngfY4LGVzXvqxTSFqRPeak5k8beCGBX8` | passes (guard) |
| `r4_xprv_to_xpub_path_refused` (the I) | `--from xprv=XPRV_84 S --to xpub --path m/0` | 2; R4 text | exit 0, `xpub: XPUB_84` (path ignored) |
| `r4_xprv_to_fingerprint_path_refused` | `--from xprv=XPRV_84 S --to fingerprint --path m/0` | 2; R4 text | exit 0, `fingerprint: e99b8628` |
| `r4_phrase_to_fingerprint_path_refused` | `--from P S --template bip84 --to fingerprint --path m/84'/1'/0'` | 2; R4 text | exit 0, `fingerprint: 73c5da0a` |
| `r4_phrase_to_entropy_path_refused` | `--from P --to entropy --path m/0` | 2; R4 text | exit 0, prints entropy |
| `r4_seedqr_to_phrase_path_refused` | `--from SQ --to phrase --path m/0` | 2; R4 text | exit 0, prints phrase |
| `r4_entropy_to_phrase_path_refused` | `--from EN --to phrase --path m/0` | 2; R4 text | exit 0, prints phrase |
| `r4_xpub_to_fingerprint_path_refused` | `--from xpub=XPUB_84 S --to fingerprint --path m/0/0` | 2; R4 text | exit 0, `fingerprint: e99b8628` |
| `r4_mk1_to_xpub_path_refused` | `--from mk1=<MK1_84, two strings> --to xpub --path m/0` | 2; R4 text | exit 0, `xpub: XPUB_84` |
| `r4_ms1_to_phrase_path_refused` | `--from ms1=ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f --to phrase --path m/0` | 2; R4 text | exit 0, prints phrase |
| `r4_guard_xpub_to_address_path_accepted` (guard) | `--from xpub=XPUB_84 S --to address --script-type p2wpkh --path m/0/0` | 0; `address: tb1q6rz28mcfaxtmd6v789l9rrlrusdprr9pqcpvkl` | passes (guard) |
| `r5_named_path_form_refused` | `--from P S --template bip84 --to xpub --path bip84` | 1; R5 text | exit 0, `XPUB_84` (path ignored) |
| `r5_hex_path_form_refused` | `--from P S --template bip84 --to xpub --path 0x54` | 1; R5 text | exit 0, `XPUB_84` |
| `r5_bare_m_is_master` | `--from P S --to xpub --path m` | 0; `xpub: XPUB_M` | exit 1, `--template is required…` |
| `r4_before_r3_xprv_to_xpub_path_and_account` (QT1 guard) | `--from xprv=XPRV_84 S --to xpub --path m/0 --account 3` | 2; R4 text | exit 0, `xpub: XPUB_84` (path ignored) |
| `r4_before_xpub_prefix_network` (QT2 guard) | `--from P --to fingerprint --path m/0 --xpub-prefix zpub` (no `--network`) | 2; R4 text | exit 2, the 5.a text (`--xpub-prefix <variant> requires explicit --network …`) |
| `r3_before_r5_named_path_with_account` (QT3 guard) | `--from P S --to wif --path bip84 --account 3` | 2; R3 text | exit 1, R5 text |
| `r5_help_path_block_literal_only` | `convert --help` | the four lines of 5.2 in order; no `named:`, no `hex:` | old block present |
| `r3_help_account_refused_with_path` | `convert --help` | the `--account` line of 5.2 | old line |

Source-arm coverage:

- phrase: R1-R5.
- seedqr and entropy: R1 and R4.
- xprv and xpub: R4, with the xpub→address guard.
- mk1 and ms1: R4. ms1 → xpub/address/wif is already refused as "cryptographically unrecoverable" and does not change.
- wif: covered by the existing §4 refusal, unchanged.
- electrum-phrase, bip38 and minikey: the general R4 rule (`_ => false`). There is no row per source because the code path is the same match arm as xprv/mk1/ms1.

The three overlaps of section 10 are covered by the guard rows `r4_before_r3_xprv_to_xpub_path_and_account` (STOP-1), `r4_before_xpub_prefix_network` (STOP-2) and `r3_before_r5_named_path_with_account` (STOP-3). The row `r4_before_xpub_prefix_network` has no `--network` on purpose: the `S` shorthand is not used in it.

## 7. Tests that change

None. The full workspace on the reference patch passes 4171/4171, so no existing test or snapshot asserts the old texts. A grep for the old `--template is required for derivation` finds it only in convert.rs and agent reports. The triage row `convert_template_xpub_path_not_silently_ignored`, drafted in an uncommitted `tests/triage_convert_path.rs`, is folded into `cli_convert_path.rs` under the same name. That file is not committed and does not exist in this branch.

## 8. Gate (the builder runs all of it; capture each run once to a file and grep it)

Use `PATH=$HOME/.cargo/bin:$PATH`. `cargo --version` must be 1.85.0. Never pass `--release`.

1. `cargo fmt --all --check` (CI uses `cargo +1.95.0 fmt --all -- --check`; run that too if 1.95.0 is installed).
2. `cargo clippy --locked --all-targets --workspace -- -D warnings`
3. `cargo nextest run --locked --workspace --no-fail-fast`: 0 failed.
4. `cargo test --locked --doc --workspace`
5. `cargo nextest run --locked -p mnemonic-toolkit -E 'binary(cli_convert_path)'`: exactly **30 run, 30 passed**. Quote the count in the GREEN commit message. A short count is a FAIL.
6. The docs CI jobs on the head, run locally where the environment allows, else as green CI on the PR:
   - `manual.yml` (lint, verify-examples, anchor-check, pdf, html)
   - `quickstart.yml`
   - `technical-manual.yml`
   - `manual-gui.yml`
   - `examples.yml` (`bash .examples-build/gen.sh > .examples-build/Examples.md` gives no diff)
7. `fuzz-smoke.yml` and the rest of `rust.yml` as green CI on the PR.
8. `bash /scratch/code/shibboleth/refugium-coordinator/coordinator/scripts/mutation-gate.sh <worktree> 6ccfdebc` after steps 1-5. Its missed list goes into the review brief. Risk class: A (CLI text, manual) and B (key derivation).

`man-pages.yml` and `changelog-check.yml` run only on tags and are not part of this gate.

## 9. FOLLOWUPS (the builder appends both to `design/FOLLOWUPS.md` in the GREEN commit)

F1 (build impact, closed by the GREEN commit):

```
### `convert-path-fix` — `convert --path` honoured on seed xpub/xprv (R1/R2), refused where unused (R4) or with non-zero `--account` (R3), literal-only help (R5) (repo: **mnemonic-toolkit**; owning phase: **FIX_convert_path build**)
- Surfaced: triage `design/agent-reports/triage-convert-path.md`; grade `grade-convert-path.md` (C, I, M).
- Build impact (1): code delta section 3, spec delta S1-S8, manual rows 1102-1103, RED rows `cli_convert_path` (30).
- Status: `owed-to-build`, closed by this build's GREEN commit (write its sha here).
- Tier: `toolkit`.
```

F2 (Minor, deferred):

```
### `convert-path-gui-manual-refusals` — the GUI manual's convert page does not mention the new `--path` refusals (repo: **mnemonic-toolkit** `docs/manual-gui`; owning phase: **next manual-gui pass**)
- Surfaced: FIX_convert_path section 5.3. `docs/manual-gui/src/40-mnemonic/44-convert.md:380-385` (`--path`) and `:472` (`--account`) omit R3 (non-zero `--account` with `--path`, exit 2) and R4 (`--path` unused, exit 2).
- Status: `deferred-minor`.
- Tier: `toolkit`.
```

## 10. STOPs (all three ruled: proposal stands; architect `reports/architect-bughunt-fixdocs.md`, QT1-QT3)

Final order in `run()`: section 3 classify_edge, the wif+path refusal, 5.0a R4, 5.0b R3, 5.a `--xpub-prefix` needs `--network`, 5.b bip38; the R5 parse error comes later, in `compute_outputs`.

**STOP-1. R3 vs R4 order. Ruled (QT1): R4 first.** Example: `--from xprv=… --to xpub --path m/0 --account 3` breaks both rules and shows the R4 text (exit 2). Why: with R3 first the holder moves the account into `--path` and R4 then refuses `--path` anyway; with R4 first, dropping `--path` is the whole fix. Guard row: `r4_before_r3_xprv_to_xpub_path_and_account`.

**STOP-2. R3/R4 vs the `--xpub-prefix` refusal (5.a, exit 2, needs `--network`). Ruled (QT2): R4/R3 before 5.a.** Example (corrected: the earlier `vpub` is rejected by clap with exit 64 before `run()`, so it could not show the order): `--from P --to fingerprint --path m/0 --xpub-prefix zpub` with no `--network` shows the R4 text (exit 2). With `--account 3` and a literal path it shows R3; without the path error it shows 5.a. Guard row: `r4_before_xpub_prefix_network`.

**STOP-3. R3 vs the R5 parse error. Ruled (QT3): R3 first.** Example: `--path bip84 --account 3` on a seed `--to wif` shows the R3 text (exit 2); without `--account` it shows R5 (exit 1). R3 shows the literal form, so one step fixes both. Guard row: `r3_before_r5_named_path_with_account`.

Section 3.5 code is unchanged.
