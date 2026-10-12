# CI hardening plan — review r3 (fold e3525e1c)

**Verdict: GREEN — 0 Critical / 0 Important / 1 Minor / 4 Nit.**

Scope: `git diff 988a4a44..e3525e1c` (gate, plan, one FOLLOWUPS entry). The fold
message's "37 base workflows" is a known slip (the count is 40) and is not a finding.

Machine checks run by this reviewer:
- `GATE_SCRATCH=/scratch/code/shibboleth/_work/r3-review python3 design/ci-hardening/ci-hardening-gate.py self-test /scratch/code/shibboleth/mnemonic-toolkit 4d703bb4`
  → rc 0, `SELF-TEST: PASS`. GREEN on the plan; all 14 mutations RED. Each of the three new
  ones is RED with `duplicate key 'run'` (manual.yml line 117; rust.yml line 120) or
  `duplicate key 'jobs'` (examples.yml line 162).
- The same three mutations with `_Unique` swapped back to plain `yaml.SafeLoader` (the
  loader before the fold): **all three GREEN**. So they are real, they are not no-ops, and
  they go RED only because of the new loader.
- Loader probes (`load()` called directly): it rejects duplicates in nested block mappings,
  in flow mappings (`{runs-on: x, runs-on: y}`), in flow mappings inside a sequence
  (`[{run: a, run: b}]`), in deep `with:` mappings, a key spelled once through an alias
  (`*k :` next to `run:`), and `on:` next to `On:` (both resolve to True). Alias reuse of
  one anchored mapping loads normally (the object is cached, so the constructor does not
  run twice). Own mutation: a duplicated `path:` in the K3 `with:` of manual-gui.yml
  → RED.

## Closure table

| r2 | Status | Evidence |
| --- | --- | --- |
| I-1 (duplicate keys pass; claimed M6 coverage) | **Closed** (one narrow residual, M-1 below) | `load()` now uses `_Unique`, which is registered for the default mapping tag, so it runs for every mapping, nested and flow. The three M6 variants are in `MUTATIONS` and are RED for that reason only (GREEN under the old loader, above). The plan text "14 … (review r1's M1–M8, with three variants of M6, and four more)" is arithmetically right: 7 + 3 + 4 = 14, against r2's list of the 11 old mutations. The new NOT COVERED entries match what the gate actually does (M-B). |
| I-2 (repo order undecided) | **Closed** | Plan 270–272: "edit, gate and commit the six other repos first, in any order. Do mnemonic-engrave last, and run `release-gen` immediately before its commit". This agrees with plan 297 ("once the engrave and the five generated repos are edited") and with plan 352 ("in the engrave commit, also the release-gen lines"). An implementer following the work order never reaches release-gen with an unedited generated repo, so the stop rule at plan 371–376 is no longer hit for no reason. |

Restructured error handling in `check_workflows` (question 2b): no file can be skipped
silently. A `GateError` from either load or from `expected()` is appended to `errs`
before the `continue`, so the run is RED whatever the counts say. When the skipped file
contributes counts, the counts check also goes RED (seen in all three new mutations). Any
other exception (a YAML syntax error, a `ConstructorError`) is not caught. It ends in a
traceback and exit 1, never GREEN.

## New findings

### M-1 — Minor — keys that GitHub reads as one string but PyYAML resolves to two values are not rejected as duplicates

**Location:** gate `_unique_mapping` (lines 102–109), which compares
`repr(construct_object(k))`, and `load()` (117–118, `y["on"] = y.pop(True)`). The claim
is plan 293 ("a repeated mapping key is rejected") and docstring line 19.

**Failure scenario (reproduced):** add the following above `jobs:` in rust.yml on the
plan-applied tree:
```yaml
"on":
  schedule: [{cron: '0 0 * * *'}]
```
PyYAML resolves the bare `on` to `True` and the quoted `"on"` to `'on'`, so the loader
sees no duplicate. `load()` then overwrites `y['on']` with `y.pop(True)`, which discards
the quoted value. The result is **gate GREEN**. GitHub reads both keys as the string
`on`. actionlint reports `key "on" is duplicated in "workflow" section` (rc 1), and GitHub
rejects the file. So the full procedure (gate plus actionlint plus CI) still catches it.
It cannot silently change a trigger, but the gate's new claim is broader than its
behaviour. The same gap applies to `1:` next to `'1':`, or `true:` next to `'true':`, in
any mapping (they load as two distinct keys, `{1: 'a', '1': 'b'}`).

**Why Minor and not Important:** it needs a deliberate, odd edit. It falls inside the
plan's NOT COVERED class "YAML 1.1 spellings PyYAML reads differently from GitHub". The
plan's mandatory actionlint step flags it. It is weaker than r2's M-A (`6_0`, which also
passes actionlint), and the owner ranked and deferred M-A as Minor.

**Fix:** in `_unique_mapping`, also track the raw scalar text: for a `ScalarNode` key,
add `k.value` to a second `seen` set and reject a repeat of either the text or the
constructed value. In `load()`, raise `GateError` if both `True` and `"on"` are keys
before the pop. Alternatively, M-A's added-lines whitelist closes this too.

### N-1 — Nit — a merge key `<<` now crashes the loader instead of loading

`_unique_mapping` calls `construct_object` on each key before `construct_mapping` runs
`flatten_mapping`. A `<<` key node (tag `tag:yaml.org,2002:merge`) has no constructor, so
`yaml.constructor.ConstructorError: could not determine a constructor for the tag
'tag:yaml.org,2002:merge'` is raised. This happened on all three merge probes. Plain
`safe_load` accepted them before the fold. None of the 40 bases uses `<<`, and the result
is a traceback with exit 1, never GREEN. Fix, if wanted: skip key nodes whose tag is the
merge tag in the duplicate check.

### N-2 — Nit — doubled file prefix in GateError messages

`check_workflows` now prints `f"{f}: {ex}"`, but the messages from `expected()` and
`insert_before()` already start with the file name. The result reads, for example,
`rust.yml: rust.yml: job x already has timeout-minutes in base (plan: STOP)`. This is
cosmetic.

### N-3 — Nit — misleading message for sequence keys

With `deep=False`, a sequence key has not been filled yet when `repr` runs, so two
different sequence keys both read as `[]` and are reported as `duplicate key []`. RED is
the right outcome anyway: SafeLoader rejects unhashable keys. Only the message is wrong.

### N-4 — Nit — docstring and plan now disagree on "positions"

The fold changed plan 291 to "K in their exact step order" (for M-B). The gate docstring
(line 16) still says "the exact K steps at their exact positions". Change it to match.

## Fold accounting

All 5 findings (1 Minor, 4 Nit) came from fold e3525e1c. M-1 arises from the new
loader's value-based comparison combined with the pre-existing `on` pop, and the fold's
new "repeated mapping key is rejected" claim. N-1 to N-4 are new code or text in that
fold. None is Critical or Important.
