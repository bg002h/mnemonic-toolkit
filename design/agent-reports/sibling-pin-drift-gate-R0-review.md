# R0 review — `SPEC_sibling_pin_drift_gate.md` (verbatim)

**Reviewer:** opus architect (R0, pre-implementation)
**Spec SHA at write time:** `origin/master` `6ae7372` (asserted by spec §6)
**Cycle:** `manual-yml-sibling-pin-vs-install-sh-drift-gate` (FOLLOWUP at `design/FOLLOWUPS.md:102-109`)
**Scope under review:** CI-only PATCH adding `.github/workflows/sibling-pin-check.yml`; resolves the "no static gate exists between manual.yml/quickstart.yml sibling pins and install.sh canonical pins" FOLLOWUP.

## VERDICT: GREEN (0 Critical / 0 Important / 4 Minor)

The spec passes R0 with citations verified line-by-line against current `origin/master`, scope class-closure confirmed, authority model consistent with `design/RELEASE_CHECKLIST.md`, and SemVer-disposition consistent with the post-`manual-prose-execution-gate` precedent. Implementation may proceed once minors below are folded (or, per the explicit "minors are optional" rule, acknowledged as deferred).

## §2 citation verification

All 7 citations verified against current bytes in `origin/master`:

| Spec citation | Actual content | Verdict |
|---|---|---|
| `scripts/install.sh:35` md-cli | `echo "md-cli\|https://github.com/bg002h/descriptor-mnemonic\|descriptor-mnemonic-md-cli-v0.6.1\|yes\|cli-compiler"` | ✓ |
| `scripts/install.sh:38` ms-cli | `echo "ms-cli\|https://github.com/bg002h/mnemonic-secret\|ms-cli-v0.4.1\|yes\|"` | ✓ |
| `scripts/install.sh:41` mk-cli | `echo "mk-cli\|https://github.com/bg002h/mnemonic-key\|mk-cli-v0.4.2\|yes\|"` | ✓ |
| `manual.yml:77` mk-cli install | `run: cargo install --git https://github.com/bg002h/mnemonic-key --tag mk-cli-v0.4.2 mk-cli` | ✓ |
| `manual.yml:84` md-cli install | `run: cargo install --git https://github.com/bg002h/descriptor-mnemonic --tag descriptor-mnemonic-md-cli-v0.6.1 md-cli --features cli-compiler` | ✓ |
| `manual.yml:88` ms-cli install | `run: cargo install --git https://github.com/bg002h/mnemonic-secret --tag ms-cli-v0.4.1 ms-cli` | ✓ |
| `quickstart.yml:71` mk-cli install | `run: cargo install --git https://github.com/bg002h/mnemonic-key --tag mk-cli-v0.4.2 mk-cli` | ✓ |
| `quickstart.yml:75` MD_BIN=true MS_BIN=true | `run: make lint MNEMONIC_BIN=true MD_BIN=true MS_BIN=true MK_BIN=mk` | ✓ |
| `install-pin-check.yml:51-55` style ref | error block at lines 51-55 (3 `::error::` lines + `exit 1`) | ✓ |

§2.3 claim that `install-pin-check.yml` "fires on `mnemonic-toolkit-v*` tag push only" verified at `install-pin-check.yml:27-30`. The motivation for the new gate to fire on every push + PR (not just tag) is correctly argued.

## Negative-claim verification

Spec §2.2 claim "Quickstart installs only `mk-cli`" verified via `grep -nE 'md-cli|ms-cli|mk-cli' quickstart.yml` → only lines 70-71 reference siblings (mk-cli). ✓

## Class-closure verification

All 5 workflow files audited (`manual.yml`, `quickstart.yml`, `manual-gui.yml`, `rust.yml`, `install-pin-check.yml`); only the two named have sibling `cargo install --git --tag` lines:

```
.github/workflows/quickstart.yml:71  (mk-cli)
.github/workflows/manual.yml:77      (mk-cli)
.github/workflows/manual.yml:84      (md-cli)
.github/workflows/manual.yml:88      (ms-cli)
```

The class closure is correct. Adjacent siblings considered + correctly excluded: `manual-gui.yml:58` GUI-clone (different canonical source); `rust.yml:218` floating master ref (no tag pin); tool-dep pins out of scope per FOLLOWUP intent.

## Authority-model verification

`design/RELEASE_CHECKLIST.md:13-24` + `:50-57` explicitly designate install.sh as the canonical sink for all 5 component pins. Spec's framing consistent.

## SemVer-disposition verification

Spec §5 no-bump correctly motivated by recent `manual-prose-execution-gate` precedent (test/docs/CI-only, no bump).

## Test-plan disposition

§4.1 synthetic-drift sed-edit is non-tautological: requires the gate to actually parse + compare.

## Minor findings

### M1 (confidence 80) — tool-dep pins as explicit non-goal
§3.4 doesn't explicitly state GHA tool-dep pins (`actions/checkout@v4`, `lychee-v0.24.2`, `markdownlint-cli2@^0.13`, `dtolnay/rust-toolchain@1.85.0`) are out of scope. Fold inline.

### M2 (confidence 70) — "skip unknown sibling" algorithm under-specified
§3.1 says "Skips lines that don't reference a known sibling" but doesn't define "known". Two interpretations: (a) whitelist by exact package name from install.sh's `case` arms; (b) pattern match by `-cli` suffix. Implementation should pick (a) — exact-name match against install.sh's parsed table. Fold inline.

### M3 (confidence 65) — quickstart md/ms-mock asymmetry
§3.4 doesn't acknowledge the structural reason quickstart only installs mk-cli (`MD_BIN=true MS_BIN=true` mock). Fold inline.

### M4 (confidence 55) — CHANGELOG `[Unreleased]` disposition
§5 says "CHANGELOG entry under a `[Unreleased]` section OR rolled into next toolkit-bump PATCH". Resolve at implementation time via `grep Unreleased CHANGELOG.md`. Defer.

## Folds checklist

- [x] M1: tool-dep non-goal bullet
- [x] M2: "known sibling = exact-match against install.sh-parsed table" + warning message format
- [x] M3: quickstart-asymmetry non-goal bullet
- [ ] M4: deferred to implementation time

## Per-prompt question answers

1. Every §2 citation matches origin/master bytes — ✓
2. install-pin-check.yml cadence vs new-gate cadence motivation correct — ✓
3. "install.sh is canonical" framing matches reality — ✓ (RELEASE_CHECKLIST.md:13-24)
4. §3.4 scope cuts defensible — ✓ (M1+M3 suggest making the non-goals more explicit)
5. §3.1 forward-compat algorithm sound — Mostly (M2: specify exact-name match)
6. SemVer-PATCH no-bump consistent — ✓
7. §4.1 synthetic-drift test real — ✓

## Status

**RECOMMEND: PROCEED TO IMPLEMENTATION.** GREEN at R0 with 4 sub-threshold minors. M1+M2+M3 folded inline before implementation; M4 deferred.
