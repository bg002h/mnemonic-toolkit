# Phase 3 — feature-dev:code-reviewer review, round 1

**Date:** 2026-05-07
**Branch:** `manual/v0_1` (commits `1e402f3` + `2aecc85`)
**Verdict:** Not converged. 2 critical / 2 important / 2 nits.

## Critical

### C-1 — `24-recover.md` Step 2: `md decode --mk1` flag does not exist

**File:** `src/20-quickstart/24-recover.md` lines 56–63.
**Confidence:** 100.

Command shown:
```sh
md decode \
  --mk1 mk1qprsqhpqqsq3cqtsleeutks2... \
  --mk1 mk1qprsqhpp0f30mtxzd65mvwcur9...
```

`md decode` accepts only positional args + `--json`. There is no `--mk1` flag. `md-cli` has zero `mk-codec` dependency — it cannot decode mk1 at all. The accompanying claim "(The `md` CLI handles mk1 decoding too)" is wrong.

**Fix:** Use `mnemonic convert` (which does have mk-codec coverage):
```sh
mnemonic convert \
  --from mk1="mk1qprsqhpqqsq3... mk1qprsqhpp0f30..." \
  --to xpub --to fingerprint --to path
```

Output (verified against pinned binary):
```
xpub: xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V
fingerprint: 73c5da0a
path: 84'/0'/0'
```

Multi-`--to` works in v0.8.0. Remove the "(The `md` CLI handles mk1 decoding too…)" parenthetical.

### C-2 — `24-recover.md` Step 3: `md decode --md1` flag does not exist

**File:** `src/20-quickstart/24-recover.md` lines 71–76.
**Confidence:** 100.

Same issue: `md decode` takes positional strings, not `--md1`. Verified against pinned binary — the positional form works:
```sh
md decode \
  md1zsxdspqqqpm6jzzqqvqz6qu79mg9p2sgfff6p2eph8wftp5uf6gqnlgzqqqnymv0 \
  md1zsxdspq259s3jnsrcrhnlagpftrf9apnc3m9fy8uqfc85cha4nqnh5k67ey2hzyc \
  md1zsxdspqjd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nvqhuuyvzgaejah6
```

Output: `wpkh(@0/<0;1>/*)`.

## Important

### I-1 — `.cmd` files absent; `verify-examples.sh` is a vacuous pass

**Files:** `transcripts/22-first-bundle.{out}`, `23-verify.{out}`, `24-recover.{out}` — `.cmd` siblings missing.
**Confidence:** 100.

`verify-examples.sh` iterates `*.cmd` files. With zero `.cmd` files, `count` stays 0 and the script exits "vacuous pass." Drift is undetectable.

**Fix:** Add three `.cmd` files paired with the existing `.out` files (using `$MNEMONIC_BIN`/`$MD_BIN`/`$MS_BIN` substitution).

### I-2 — `23-verify.md` reuses canonical seed without backreference to DANGER box

**File:** `src/20-quickstart/23-verify.md` line 18.
**Confidence:** 90.

Per AUTHORING.md the DANGER box is shown once per chapter; subsequent re-uses elsewhere should cross-reference the original. `23-verify.md` repeats the seed in plain text with no link back.

**Fix:** Add a one-liner before the command:
```
(Same canonical test seed as [Chapter 22](#your-first-bundle); see the DANGER box there.)
```

## Minor / nits

### N-1 — `22-first-bundle.md` engraving-card prose ambiguous

**File:** `src/20-quickstart/22-first-bundle.md` lines 106–107.

"The default emits text only" can be read as "no card emitted by default." The card IS emitted to stderr by default; "text only" means plain-text format vs. graphical. Suggested rephrase clarifies "emits to stderr by default" + "pass `--no-engraving-card` to suppress."

### N-2 — `24-recover.md` transcript only covers Step 1

**File:** `transcripts/24-recover.out` covers `mnemonic convert --from ms1 --to phrase` only. Steps 2 and 3 (mk1 decode, md1 decode) are not regressionchecked.

After C-1 / C-2 are fixed, add `24-recover-mk1.{cmd,out}` and `24-recover-md1.{cmd,out}` to lock the corrected commands.

## Verification of correct elements

| Check | Status |
|---|---|
| `22-first-bundle.out` and `23-verify.out` byte-match against pinned binaries | OK |
| BIP-84 mainnet derivation `m/84'/0'/0'` | OK |
| Flag accuracy for `21`, `22`, `23` (--network, --template, --slot, --ms1, --mk1, --md1, --no-engraving-card, --json) | OK |
| Forward-pointer `#recovery-paths-by-damaged-card-scenario` | resolves once Phase 4 lands |

## Convergence assessment

2 critical / 2 important / 2 nits. Not converged. After C-1/C-2 fixes + I-1 .cmd files + I-2 backreference, round-2 should land at 0C/0I.
