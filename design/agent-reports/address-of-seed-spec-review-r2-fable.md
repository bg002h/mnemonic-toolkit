# R0 review r2 (fable): SPEC `xpub-search address-of-seed`

**Reviewed:** `/mnt/project-files/address-search/SPEC_xpub_search_address_of_seed.md` (DRAFT r2, 633 lines).
**Against:** mnemonic-toolkit `claude/project-thread-utm1aa` at `40b35942`, binary `target/debug/mnemonic` 0.107.1.
**Read-only:** the spec and the repo were not edited. My scratch files are in `/tmp/claude-0/-home-user/efb9fedd-ab0a-5019-871b-00d31aeaaeb6/scratchpad/r2f/` (`t.sh` runs a `bundle` command with the abandon phrase on stdin, then `restore --md1`; `cb/` is a variant of the author's `costbench`).

**Verdict: RED. 0 Critical, 6 Important, 13 Minor, 5 Nit.**
The r1 Critical is fixed for the direction r1 demonstrated. Two of the Important findings are defects in r1 folds (C1 and I4); two are printed commands that `bundle` refuses (I3, I4); one is the quality of the exit-4 advice (I5); one is a keyed-descriptor refusal (I6).

## What was checked and holds

- **Vectors, recomputed with a second implementation.** I minted each wallet with the binary (`bundle --descriptor … --slot @0.phrase=- --slot @1.phrase=@env:…`) and read the addresses back with `restore --md1`. That path is rust-miniscript, independent of the author's `derive.py`.
  - M1 (index 0 and index 1, so M5's second target too), M2's twin `multi(Z,A)` (index 0 equals M1; index 1 is `bc1qmkt0wm4y…5ng527`), M3, R1, R1's `multi_a(A,Z)` twin (same address), R2, U1, U1 swapped (`bc1qu97h4hd0…`), N1, T1 (also signet; regtest gives `bcrt1q6rz28mcfaxtmd6v789l9rrlrusdprr9pz3cppk`) and P1 all agree with the spec.
  - M4 agrees with `derive.py` (change chain, not listed by `restore`).
  - `bundle` refuses the P1 command with the wrong fingerprint, as §5 says ("derives master fingerprint b4e3f5ed").
- **Printed commands that `bundle` accepts (exit 0).**
  - Single-sig `pkh`, `sh(wpkh)`, `wpkh`, `tr` at `m/45'`, at BIP-48/87 paths, at `m/0'`, at the Refugium path, with `<2;3>`, `<0;1;2>` and bare `/*`.
  - Two seeds with one on stdin and one on `@env:`, in either slot order; placeholders out of numeric order; per-slot multipaths (U1); `sh(wsh(sortedmulti))`; `sh(sortedmulti)` at `m/45'`; `tr(NUMS,…)` with the word `NUMS` or the hex key.
  - Cosigner as `@1.xpub` + `@1.fingerprint` + `@1.path`, with or without the origin also inline.
  - `@0.entropy=@env:`, `@0.ms1=@env:`; `--passphrase -` beside `@env:` seeds; `--passphrase @env:` beside a stdin seed. `--passphrase -` beside `@0.phrase=-` is refused, as the spec expects.
- **Cost figures.**
  - `costbench` re-run: 0.8 / 1.1 / 1.3 µs for wsh/sh and 46.0 / 45.7 µs for taproot (spec: 0.8 to 1.4 and 46).
  - `path-of-xpub` over 160,001 paths took 13.9 s, 87 µs per path (spec: 89 µs).
  - Larger shapes stay inside the planning figures: 4-of-7 `wsh(multi)` 1.2 µs, 8-of-15 `wsh(sortedmulti)` 6.8 µs, 4-of-7 `tr(NUMS,multi_a)` 44.9 µs.
  - The table arithmetic I sampled is right (38,640; 122,880; 1,036,840; the 22× and 4.5× ratios).
  - Not re-run: the `restore` evaluator row (165 / 240 µs).
- **Engine citations.** `collect_all`, `stop_at`, `early_exit`, `flatten`, `cap_decision` and `calibrate_per_candidate` are where §4.1 says, and the evaluator interface (`matches(&[usize], u64) -> bool`) can carry the new enumeration.
- **r1 folds that are sound:** I2, I5, I6, I7, and C1 for a `multi` template whose wallet is really `sortedmulti` (one extra script per match, independent of pinning).

---

## Important

### I1. The twin check for a sorted template is over "f! orders": it misses twins when slots are pinned and cannot run for large n (§5 "Sorted/unsorted twins"; C1 fold)

**Evidence.**
- §5 says a match from a sorted fragment is checked against "`multi` / `multi_a` in each of the f! orders … (f! × targets scripts, a negligible cost)". §4 defines f as "keys without a pinned slot".
- **Pinned slots.** The twin of a sorted match is `multi` with its *arguments* in the order that sorts the child keys at that index. That is a permutation of all n arguments, whatever the user pinned. With `--cosigner @1=<xpub>` or `[fp]@0`, f is 0 or 1 and only the template's own order is tried.
  - Row M1 with the cosigner placed (`@1=` zoo's xpub) would then exit 0: the twin `multi(Z,A)` is never built. The same wallet and target with the cosigner unplaced exits 4.
  - §4 even recommends `--cosigner @N=` and `[fp]@N` as the way to narrow a run.
- **Cost.** With nothing pinned, f = n. `bundle` allows n up to 16. I measured 6.8 µs per script for 8-of-15 `wsh(sortedmulti)`. Per matched target that is about 15 s at n = 10, about 40 min at n = 12 and about 100 days at n = 15. The cost table promises "sorted, any n: under 0.1 s".
- **Asymmetric slots (U1).** There "orders" is ambiguous: reassigning keys to slots changes the keys (`Z/0/i`, `A/2/i`), while the twin needs the same keys in another argument order.

**Fix.** Use the closed form; no enumeration is needed.
1. For a sorted match and a target matched at (alternative, index), exactly one `multi` argument order gives that script: the lexicographic order of the n child keys there.
2. Compute that order per matched target. A `multi` twin fits every target if and only if the order is the same for all of them. That is O(targets × n log n).
3. State that the twin permutes the fragment's arguments (each with its use-site path), not the key-to-slot map, and that pins do not reduce it.
4. Add a row: M1 with `--cosigner @1=<Z xpub>` still exits 4.

### I2. The sorted collapse is unsound for templates with more than one fragment or a key outside the sorted fragment (§4 pruning 4; I4 fold)

**Evidence.**
- The condition is "every key sits in `sortedmulti` or `sortedmulti_a`, every slot has the same use-site alternatives, and the slots are unpinned or pinned to the same path". It does not require one fragment.
- `bundle` on this branch accepts all of these (exit 0, run with four watch-only xpubs):
  - `tr(NUMS,{sortedmulti_a(1,@0,@1),sortedmulti_a(1,@2,@3)})`. Every key sits in a sorted fragment, so the rule collapses to one order. But which seeds share a leaf changes the leaf scripts: there are three distinct wallets and two are never searched.
  - `tr(@0,sortedmulti_a(2,@0,@1,@2))`. Every key sits in the sorted fragment, but `@0` is also the internal key. Three wallets (one per choice of internal key), one searched.
  - `wsh(or_d(sortedmulti(2,@0,@1),and_v(v:pk(@2),older(144))))`. Here `@2` is outside, so the rule already falls back to f!. Correct, but only by the wording.
- The result is a missed wallet in the stated space (no match, exit 4), not a false exit 0.

**Fix.**
1. Define the symmetry per fragment: slots are interchangeable only if they appear in the same single sorted fragment and nowhere else, with equal use-site paths and equal pins.
2. The order count is then n! divided by the product of the group factorials. Simpler: collapse only when the whole template is one sorted fragment under `wsh`, `sh`, `sh(wsh)` or `tr(NUMS,…)`, and search f! otherwise, leaving the rest to the post-scan merge.
3. Add test rows for the two-leaf template and the internal-key template.
4. Say which templates the search accepts (see M1 below).

### I3. A bare cosigner xpub usually makes the printed command exit 2, not 4 (§6 "A bare xpub with no known origin", §9 "exits 0 (4 for a bare cosigner xpub)")

**Evidence.** `bundle` accepts a slot with only `@N.xpub=` when the xpub's own header (depth and child number) matches the shape's default origin. Otherwise it refuses:

```
error: slot @1: no `@1.path=` was given, and the default origin m/48'/0'/0'/2' does not match
the xpub's own header (depth 3, child number 0'); the key does not live at the default path.
```

- Exit 4, with the warning the spec describes: zoo's `m/48'/0'/0'/2'` xpub in `wsh(sortedmulti)`.
- Exit 2: the same xpub in `tr(NUMS,sortedmulti_a)` and in `sh(sortedmulti)` (the default there is `m/87'/0'/0'`); a BIP-87 or BIP-84 account xpub in `wsh(sortedmulti)`; any xpub with account ≠ 0. Adding only `@1.fingerprint=` does not help.
- A path without a fingerprint is accepted (exit 0).

**Fix.**
1. At intake, read the header of each origin-less cosigner xpub. If it does not match the template's canonical origin, the wallet can still be found, but no command can be printed for that slot.
2. In that case print the match and say that `bundle` needs `--slot @N.path=`, which only the user knows. Use exit 4, and set `bundle_command` to null or carry a `<path>` placeholder.
3. Correct §6 and the §9 row; add a row for the refused case.

### I4. A match at a path step of 2²⁹ or more prints a command `bundle` refuses, and §3 recommends such paths (§3 "Common paths outside the set", §2 account flags)

**Evidence.**
- `bundle --descriptor 'wpkh([73c5da0a/84h/0h/2147483646h]@0/<0;1>/*)'` exits 2 with `error: md1 varint overflow: 2147483646`.
- Bisected: account 536,870,911 is accepted and 536,870,912 (2²⁹) is refused. A non-hardened step of that size is refused too.
- §3 names Samourai's `m/84'/0'/2147483644'` to `2147483646'` as `--add-path` suggestions. `--max-account` and `--number-of-accounts` also reach that range.

**Fix.**
1. Treat a step ≥ 2²⁹ like the BIP-45 case: report the match and its full path, say that no card can record it, print no `bundle` command, and exit 4.
2. Say so next to the Samourai hint in §3.
3. Add a test row, and a FOLLOWUP for the md1 limit if none exists.

### I5. The exit-4 advice "give a second address at another index" fails about half the time, and the tool could give the answer itself (§5 twins, open question 6)

**Evidence.**
- With a sorted template and one target, the result is always exit 4. That is the most common multisig case, since most wallets use `sortedmulti`.
- The spec's own vector agrees with its twin at 11 of the first 20 receive indices (0, 2, 4, 6, 7, 10, 12, 13, 14, 18, 19). A second address picked without guidance leaves the result ambiguous in about half the cases for two keys.
- At that point the tool holds both candidate wallets and every child key in the window. It can compute where they differ.

**Fix.**
1. In the ambiguous block, print the first few indices where the two wallets differ, with both addresses at each: "index 1: sortedmulti gives `bc1qp3z0…`, multi(Z,A) gives `bc1qmkt0…`; whichever your wallet shows decides it".
2. Put the same in JSON (for example `distinguishing: [{alternative, index, addresses: […]}]`).
3. State in the text which of the two is the template the user supplied.
4. Keep exit 4.

### I6. A keyed descriptor places seeds by fingerprint first, so a wrong or zero fingerprint refuses a seed that does hold a key (§2 "A keyed descriptor or keyed md1"; I3 fold)

**Evidence.**
- §2: each seed is placed "by matching its master fingerprint to a key's stated `[fp/path]`"; only an empty origin falls back to a path search; "a seed that matches no key is refused (exit 2)".
- The toolkit itself writes zero fingerprints. The bare-cosigner run above was restored as `[00000000/48'/0'/0'/2']xpub6DXuQ…`. Watch-only exports from other wallets carry placeholder fingerprints too.
- A user who passes such a descriptor with the seed for that key gets exit 2 and "matches no key", although deriving at the stated path would match the xpub exactly.

**Fix.**
1. For every (seed, key with a stated path) pair, derive at the path and compare the xpub. That is k × n derivations at 87 µs each.
2. Use the fingerprint only as a shortcut and as a reported mismatch: "key @1 states 00000000; this seed is 3f635a63".
3. Use the seed's real fingerprint in the printed command, as the passphrase rule in §5 already requires.
4. Add a row.

---

## Minor

**M1. The accepted template class is not stated (§2, §4.1 item 3).** `bundle` mints shapes for which the toolkit cannot derive addresses: `restore --md1` refuses `sortedmulti_a` in a multi-leaf tree, `sortedmulti` inside a combinator and a trunk key that is also a leaf key. rust-miniscript at the toolkit's pin parses all three (1.8 µs and 46 µs measured), so the evaluator can handle them. The spec should say whether the search accepts everything `parse_descriptor` accepts, and that for these shapes the "printed command, run" test in §9 needs an address oracle other than `restore`.

**M2. Test rows use bare `@0,@1` (§9 M1 to M6, R1, R2, N1).** `bundle` refuses a bare `@0`: "would be recorded as `@0/*` … Write `@0/<0;1>/*`". §2 lists `<a;b>/*` and bare `/*` but not a bare `@N`. Either refuse it as `bundle` does, with a row, or write the rows in full. `/**` is accepted by `bundle` and is also unlisted.

**M3. "Repeated `--from`, as `restore` does" is wrong (§2).** `restore`'s `--from` is `Option<String>` (`restore.rs:78`). Only `convert` has a repeatable one, with a different value type. The multi-seed intake is new code: say so, and specify `refuse_second_stdin` across `--from`, `--passphrase` and `--descriptor-from md1=-`. The argv guard does cover `--from` on every verb (`argv_guard.rs:296`).

**M4. The redaction rule can produce two stdin readers (§6).** "A value that was inline becomes `-` for the first such seed" ignores a seed or passphrase already given as `-`. The rule should be: `-` only if no other value in the printed command reads stdin. The name of the passphrase's variable is not given (`MNEMONIC_SEED_<N>` does not fit), and `<N>` should say whether it is the seed index or the slot.

**M5. `--json` says "nothing goes to stderr", but §4 prints an estimate line and progress (§4, §6).** Say which wins under `--json`.

**M6. The JSON shape for twins is not specified (§6).** The GUI consumes this shape and no gate checks it. Missing: whether a twin is a `wallets[]` entry and how it is marked; the `result` value when a twin matches more targets than any candidate; a result that is both partial and ambiguous; `template` and `account` for `m/45'`, `--add-path` and Refugium paths; and the `searched` block in keyed mode.

**M7. A keyless md1 carries origin paths (§2).** "A keyless md1 is treated as a template" does not say whether its recorded paths pin the search (as "whatever the descriptor already states is not searched" suggests) or are ignored. Pinning is fast but depends on a path that may be `bundle`'s default.

**M8. Exit code with two or more seeds and no descriptor (§2, §5).** If seed 0 holds every target and seed 1 holds none, is that exit 0? State it, and add a row.

**M9. Engine details the plan must not miss (§4.1).**
- With `collect_all`, one match returns `Unique`, not `Enumerated` (test `collect_all_with_one_match_is_unique_not_a_list`).
- `Enumerated` drops the address index and repeats an assignment once per matched (alternative, index).
- The capped match list is new engine code.
- `restore` has a hard ceiling of 10¹⁵ candidates (`REALIZED_S_MAX`), so "no ceiling … `restore`'s policy" is not quite right. The engine clamps the total to `u64` silently, and `--independent-paths` with nine seeds exceeds it. Refuse above the clamp.

**M10. Keyed mode and the twin check (§2, §5).** Exit 0 is defined as "no twin", but §2 says a keyed descriptor is "not searched". Say whether the twin check runs there. It costs one comparison under I1's fix.

**M11. Lockstep sites not listed (§8).**
- Toolkit: `docs/manual/tests/cli-subcommands.list`; `docs/manual-gui/transcripts/gui/mnemonic-xpub-search-*.gui`; `tests/cli_gui_schema.rs` (subcommand list, line 115); `tests/cli_network_fail_open.rs` (a per-verb network-mismatch row); `tests/cli_xpub_search_env_var_sentinel.rs`.
- mnemonic-gui: `tests/xpub_search_schema_mirror.rs` (verb list, line 47) and `tests/f5_nested_argv.rs` (line 44).

**M12. `--cosigner` with private material (§2, §7).** The refusal is stated for `--descriptor` only. Say that an xprv or WIF on `--cosigner` is refused without echo as well.

**M13. A mixed-type target list is refused whole (§5).** "A descriptor that cannot produce a target's address type" exits 2 even if other targets fit. Reporting the mismatched ones as unmatched (partial, exit 4) would lose nothing. Either choice is fine, but state it.

## Nit

- **N1.** §5 merges candidates "with identical scripts across the whole window". With `--gap-limit 1 --external-only` the window is one script. No real collision follows, but the merge is better defined by the symmetry (same sorted fragment, tap siblings) than by the window.
- **N2.** §4 "about 1.2 MB for three seeds" assumes 65 bytes per point; 19,320 compressed keys are 0.64 MB.
- **N3.** `address-of-xpub`'s help still calls `--gap-limit` a "BIP-44 gap-limit window". If §2's wording is the standard, the sibling's help should follow.
- **N4.** §9 says the new rows were derived with `bundle` then `restore --md1`. That works for the listed rows, but not for single-sig `tr` ("taproot tree has no script leaf"). The implementer's "printed command, run" test needs `restore --from` or Core for S3 and S7.
- **N5.** §6 sample: the `account:` line for a multisig match with independent paths is not shown. One multisig sample block would settle the layout.

---

## r1 disposition check

| r1 | fold | verdict |
| --- | --- | --- |
| C1 | twin check, exit 4 | Fixed for a `multi` template. For a sorted template the "f! orders" rule is wrong: I1. Advice quality: I5. |
| I1 | engine mode, enumeration, evaluator | Sound and implementable; details in M9. |
| I2 | taproot cost | Confirmed by re-run. |
| I3 | keyed descriptors by origin | Sound except for fingerprint-first placement: I6. |
| I4 | collapse conditions | Right for U1; unsound for multi-fragment templates: I2. |
| I5 | alternatives per slot | Sound; bare `@N` unstated: M2. |
| I6 | `m/45'` wording, BIP-45 excluded | Sound. |
| I7 | GUI lockstep | Sound; a few more sites: M11. |
| M3 | bare cosigner | The fold's claim (exit 4) holds only when the xpub header matches the default: I3. |
| M6 | hint paths | Samourai paths cannot be minted: I4. |
| others | | No new defect found. |
