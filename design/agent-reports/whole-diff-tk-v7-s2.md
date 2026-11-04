**⚠ 1 Important: the template-mode branches added by FIX-V7 (`bundle.rs:953-954` JSON, `bundle.rs:1383-1384` card) are not pinned by any test, so a fault there passes the whole suite and all 30 FIX rows.**

# Whole-diff review, toolkit FIX-V7 build, slice 2 (Sonnet 5.5, high)

Target `/scratch/code/shibboleth/tk-build-v7`, branch `build/v7-bundle-recovery`, head 9aef788e, diff `12e013bc..9aef788e`.
Scope: grade the 5 mutation survivors; tests-can-fail for the 30 FIX rows; run the FIX doc's RED command. s1 (advice/design) not duplicated.
Re-review stats: this is the first whole-diff review of the build; no previous-round fix commit, so 0 findings come from one.
All runs from a `git archive` copy of 9aef788e under `/scratch/cache/whole-diff-tk-v7/s2/` (own `CARGO_TARGET_DIR`, cargo 1.85.0, nextest). The target tree was not touched except this report. cargo-mutants was not run; mutants were applied by hand as scratch edits.

## 1. FIX doc RED command, run as printed
`cargo nextest run --locked --no-fail-fast -p mnemonic-toolkit -E "$(cat design/agent-reports/fix-doc-tk-v7-evidence/fix-rows-filter.txt)"`
- Head: `Summary [0.050s] 30 tests run: 30 passed, 4111 skipped`.
- Base production (12e013bc production files, head tests; row 30's `format.rs` test kept with the base recovery block and `threshold: u8`; output had `Compiling mnemonic-toolkit`): `Summary [0.023s] 30 tests run: 6 passed, 24 failed, 4111 skipped`. The 6 passes are rows 12, 25-29, as the FIX table says.
- Every one of the 24 base failures panics at the assertion line and for the reason the FIX table gives (t:96 false `# Threshold: K of N` for rows 1-9, 11; t:167 row 10; t:218 rows 13-16 and 23; t:424 row 17; t:433 row 18; t:453 row 19; t:467 row 20; t:494 row 21; t:519 row 22; t:240 row 24; row 30 at `format.rs` general-line assert, line 512 in my reconstructed tree vs 510 in the doc, a layout artefact of my overlay, not a finding). Row 20/21/22 error text `invalid type: null, expected u8` matches.

## 2. The 5 survivors: all equivalent
Each was applied alone as a scratch edit; the 30 FIX rows still pass (30/30) and probes of the CLI output (3-cosigner template card and JSON, single-sig `bip84` card) are byte-identical to head. Output files: `/scratch/cache/whole-diff-tk-v7/s2/mut-*.txt`, `ss-*.txt`.

| # | survivor | verdict | reason |
|---|---|---|---|
| S1 | `bundle.rs:1272:34` `c.len() == 1` -> `true` in `only_child` | equivalent | `only_child` is called only on `Wsh`, `Sh`, `Check`, `Swap`, `Alt` nodes. md-codec `tree.rs:310-322` decodes each of those tags with exactly one child, and `reassemble` is the only source of the tree. The guard is defensive against a hand-built malformed `Node`, which no caller can pass (a hand-built `Wsh` with 0 children would panic on `c[0]` under the mutant; with 2 it would read child 0). |
| S2 | `bundle.rs:1323:14` `leaf.tag != Tag::TapTree` -> `true` | equivalent | with a `TapTree` leaf the guard-less arm calls `kofn(leaf)`, and `kofn` matches only `Multi`/`SortedMulti`/`MultiA`/`SortedMultiA`/`Thresh`, so a `TapTree` returns `None` either way. The guard is strictly redundant. |
| S3-S5 | `bundle.rs:1384:32` `>` -> `>=`, `==`, `<` in `args.threshold.or(if n > 1 { Some(n) } else { None })` | equivalent (dead fallback) | In template mode `args.threshold` is `Some` whenever n > 1: `pre_check_threshold` (`bundle_unified.rs:81-87`) refuses a multisig template with n >= 2 and no `--threshold` ("--threshold required for multisig template"), and `bundle.rs:326` refuses `--threshold` on a single-sig template. So `.or(..)` is reached only for n == 1, where the original gives `None`; the `>=` and `<` mutants give `Some(1)`/`Some(0)` but `format.rs:283` prints a Threshold line only when `input.n > 1` and the FIX recovery block also needs `n > 1`. Checked on the single-sig card (`bip84`, phrase on stdin): identical under all three. |

No survivor removes FIX-V7 behaviour. No RED row is owed for them.

## 3. Tests-can-fail for the 30 rows
- 24 rows: fail on base for the named reason (section 1).
- 6 guard rows (12, 25-29), each planted on head and seen to fail (output `plant-*.txt`):
  - `wsh` arm returns `None`: rows 12, 25, 28, 29 and 24 fail (`missing Threshold line`, `left: Null`).
  - `thresh` guard forced false: rows 25, 28 fail.
  - `sh(multi)` arm `None`: row 26 fails (`want exactly one line '# Threshold: 2 of 3'`).
  - `sh(wsh(..))` arm `None`: rows 25, 26 fail.
  - `tr(NUMS,..)` arm `None`: rows 12, 25, 28 fail.
  - `format.rs` `if input.n > 1` guard removed from the recovery block: row 27 fails (`n == 1 prints neither line`).
  - JSON writer forced `null`: rows 28, 24 fail.
  - `import_wallet.rs` recompute forced `None`: row 29 fails (`bundles=1 ... threshold=none`).
- Gap: nothing fails when the FIX-V7 template-mode branches are broken (I-1 below).

## Findings

### I-1 (Important) template-mode branches of FIX-V7 are untested; a fault there prints a wrong card
Files: `crates/mnemonic-toolkit/src/cmd/bundle.rs:953-954` (JSON `multisig.threshold`) and `:1383-1384` (card `# Threshold:` / `# Recovery:`). FIX 2.1 / 2.3 state that template mode keeps `--threshold`. These `if args.template.is_some()` arms are new in this diff, and every one of the 30 FIX rows (and every older test) drives descriptor mode (`--descriptor`, `--import-json`, `import-wallet`), so they never run. cargo-mutants cannot mutate these `if` conditions or the `Some(..)` arm values, so the gate shows no survivor, but the faults below are real:
- JSON arm `Some(args.threshold.unwrap_or(n as u8))` replaced by `None`: `mnemonic bundle --template wsh-sortedmulti --threshold 2 --json ...` prints `"multisig":{...,"threshold":null,...}`. Full package suite (`cargo nextest run --locked -p mnemonic-toolkit`): `4104 tests run: 4104 passed, 37 skipped`; FIX rows `30 passed`.
- Card arm `args.threshold.or(..)` replaced by `None::<u8>`: a 2-of-3 template card loses its `# Threshold:` line and, because n > 1 with `threshold == None` now selects the new general arm, prints the "general policy, not k-of-n ... keep all 3 signing keys" Recovery line on a plain 2-of-3 card. Full package suite: `4104 passed, 37 skipped`. (The pre-FIX code had the same untested `.or`, but before FIX-V7 a `None` simply dropped the line; now it prints a false "not k-of-n" claim, so the new code raises the cost of the gap.)
Fix: add these two rows at the end of `crates/mnemonic-toolkit/tests/cli_bundle_recovery_threshold_text.rs`, add them to FIX section 4 as rows 31 and 32 (both PASS on base, guards like rows 25-29; the matched count and the "30" figures in the FIX doc, build report and `fix-rows-filter.txt` become 32), and keep `row-names.txt` in step.

```rust
// ---- Template mode keeps `--threshold` (FIX-V7 ruling 1 / 2.1) -------------

const TPL_XPUBS: [(&str, &str); 3] = [
    ("xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX", "b8688df1"),
    ("xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx", "5436d724"),
    ("xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6", "28645006"),
];

/// `bundle --template wsh-sortedmulti --threshold 2` over three xpub slots.
fn template_bundle(extra: &[&str]) -> std::process::Output {
    let mut args: Vec<String> = ["bundle", "--network", "mainnet", "--template", "wsh-sortedmulti", "--threshold", "2"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    for (i, (xpub, fp)) in TPL_XPUBS.iter().enumerate() {
        for (field, val) in [("xpub", *xpub), ("fingerprint", *fp), ("path", "48'/0'/0'/2'")] {
            args.push("--slot".into());
            args.push(format!("@{i}.{field}={val}"));
        }
    }
    args.extend(extra.iter().map(|s| s.to_string()));
    let out = Command::cargo_bin("mnemonic").unwrap().args(&args).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    out
}

#[test]
fn v7_template_mode_card_keeps_flag_threshold_and_recovery_lines() {
    let e = String::from_utf8(template_bundle(&[]).stderr).unwrap();
    assert!(e.lines().any(|l| l == "# Threshold: 2 of 3"), "{e}");
    assert!(
        e.lines().any(|l| l == "# Recovery: any 2 of 3 signing keys + md1 (template card)."),
        "{e}"
    );
    assert!(!e.contains("general policy"), "{e}");
}

#[test]
fn v7_template_mode_json_threshold_is_the_flag() {
    let out = template_bundle(&["--json"]);
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["multisig"]["threshold"], Value::from(2), "{v}");
}
```

RED/GREEN as run (command: `cargo nextest run --locked --no-fail-fast -p mnemonic-toolkit -E 'test(/(^|::)(v7_template_mode_card_keeps_flag_threshold_and_recovery_lines|v7_template_mode_json_threshold_is_the_flag)$/)'`):
- Head: `PASS` x2, `Summary 2 tests run: 2 passed, 4141 skipped`.
- Mutant J (JSON arm `None`): `FAIL v7_template_mode_json_threshold_is_the_flag` (panic at the `assert_eq!`, test file line 723), `1 passed, 1 failed`.
- Mutant K (card arm `None::<u8>`): `FAIL v7_template_mode_card_keeps_flag_threshold_and_recovery_lines` (line 711), `1 passed, 1 failed`.
- Mutant L (card `if args.template.is_some()` -> `if false`): both pass; equivalent, because a plain template's descriptor-derived K equals `--threshold`.
Both new rows are guards (green on base), like rows 25-29; row 12 in the FIX table's "6 PASS" count becomes 8.

### Minors
None.

### Nits
- N-1 `bundle.rs:1384`: the `.or(if n > 1 { Some(n) } else { None })` fallback is dead (S3-S5), as is the `unwrap_or(n as u8)` default at `bundle.rs:954` (`--threshold` is mandatory for a multisig template, so the "default N" in FIX 2.1 and the comments at `bundle.rs:951`/`:1381` never applies). Fix: write `args.threshold` at 1384, `Some(args.threshold.unwrap_or(n as u8))` may stay as a defensive default; clears S3-S5 from every future mutation run. Run: with that edit plus N-2 below, `cargo nextest run --locked -p mnemonic-toolkit` gave `4104 tests run: 4104 passed, 37 skipped`, no warnings.
- N-2 `bundle.rs:1323`: the `if leaf.tag != Tag::TapTree` guard is redundant with `kofn` (S2). Fix: write `) => kofn(leaf),`; same run as N-1 (green). Keeping it as documentation is also fine, but then it stays a permanent mutation survivor.
- N-3 `bundle.rs:1272`: the `c.len() == 1` guard (S1) is defensive for malformed hand-built trees. Keep; to silence the gate add a `#[cfg(test)]` unit test that calls `plain_multisig_threshold` on a hand-built `Wsh` with two children and expects `None`. Not run; not required.

C / I / M / N: 0 / 1 / 0 / 3
