# SPEC — manual-gui `export-wallet --timestamp` default-`now` prose fix

**FOLLOWUP:** `manual-gui-export-wallet-timestamp-default-now-stale`.
**Source SHA (origin/master at write time):** `6eb175a`.
**Recon:** `cycle-prep-recon-session-followups-closure-and-manual-gui-timestamp.md` (caught a STRUCTURAL error in the FOLLOWUP: 3 stale sites, not 2; the cited-as-"wrong" `:422` is a real stale site).
**Cycle type:** docs-only prose fix in the toolkit repo's GUI user manual (`docs/manual-gui/`, a separate `manual-gui-v*` release cadence). **NO toolkit crate version bump, NO tag** — precedent: cli-help-cleanup (`a83dc75`) + anchor-dangler (`dd7c228`). Plain commit to `master`.
**Locksteps:** `make -C docs/manual-gui lint` GREEN + `.github/workflows/manual-gui.yml` CI (fires on `docs/manual-gui/**`). **No** GUI `schema_mirror`, **no** sibling-codec, **no** `docs/manual/` change.

---

## 1. Problem

toolkit v0.47.3 (`export-wallet-timestamp-default-zero`) flipped `export-wallet --timestamp`'s default `now → 0` (rescan from genesis). The GUI user manual still documents the old `now` default at **three** sites in `docs/manual-gui/src/40-mnemonic/45-export-wallet.md` (recon corrected the FOLLOWUP, which listed only two and falsely called `:422` a non-site).

## 2. Changes (the 3 stale sites)

### 2a. `:30` — flag-list summary
From: ``- [`--timestamp`](#mnemonic-export-wallet-timestamp) — Bitcoin Core `timestamp` field (default `now`)``
To: ``- [`--timestamp`](#mnemonic-export-wallet-timestamp) — Bitcoin Core `timestamp` field (default `0`; rescan from genesis)``

### 2b. `:342-346` — the `--timestamp` section prose (R0 M1: block runs `:342-346`, 5 lines — replace the paragraph, not line-targeted)
From: "Bitcoin Core `timestamp` field. Two valid forms: `now` (the default; emits the literal string `"now"` in the JSON, which Core interprets at import time as the current block timestamp) or a non-negative integer Unix-seconds value. Used only for `--format bitcoin-core`."
To (lead with the `0` default; keep `now`/unix as the other forms; preserve the `now` explanation): "Bitcoin Core `timestamp` field. The default is `0` (rescan from genesis, so an existing wallet's full transaction history is discovered). Other forms: `now` (emits the literal string `"now"`, which Core interprets at import time as the current block timestamp — watch going forward, skipping the historical rescan) or a non-negative integer Unix-seconds value. Used only for `--format bitcoin-core`."

### 2c. `:422` — the worked `importdescriptors` JSON example
From: `       "timestamp": "now",`
To: `       "timestamp": 0,`
(The toolkit emits `0` as a JSON **number** since v0.47.3 — no quotes. Only this line changes; the surrounding `desc`/`active`/`internal`/`range`/`next_index` fields are unchanged and out of scope.)

## 3. FOLLOWUP correction + resolution
- Correct `manual-gui-export-wallet-timestamp-default-now-stale`: remove the false "*the cited `:422` ref was wrong (no such timestamp example at that line)*" clause; record the THREE sites (`:30`, `:342-344`, `:422`); note the recon's repo-location-trap root cause (the GUI v0.28.0 recon grepped the mnemonic-gui repo, which has no `docs/`).
- Flip its **Status → resolved** on ship.

## 4. Verification (no RED phase — prose)
- `make -C docs/manual-gui lint` GREEN (markdownlint + cspell + lychee + the rest of the 7-phase lint).
- `grep -n 'default `now`\|timestamp.*: "now"\|`now` (the default' docs/manual-gui/src/40-mnemonic/45-export-wallet.md` returns nothing (no stale default-`now` claim left). A residual `now` token is fine ONLY where it describes the `now` FORM (2b keeps one such mention, correctly framed as a non-default alternative).
- `make -C docs/manual-gui html` builds clean (anchor `#mnemonic-export-wallet-timestamp` intact).

## 5. Phasing
- **Phase 1 (implement):** 2a + 2b + 2c. Run §4.
- **Phase 2 (review + ship):** per-phase opus review → 0C/0I → §3 FOLLOWUP correct+resolve → ff-merge to `master` → push → watch `manual-gui` CI. **No tag, no version bump.**

## 6. R0 must decide / confirm
1. **No toolkit version bump / no tag** for a `docs/manual-gui/` prose fix (vs a `manual-gui-v*` cadence bump). SPEC recommends no-bump plain commit (the cli-help-cleanup precedent; manual-gui PDF release is a separate `release-attach VERSION=…` action, not triggered by a content commit).
2. **Completeness:** are there OTHER stale `now`-default mentions in `docs/manual-gui/` beyond the 3? (Recon grep found only these 3; R0 re-greps.)
3. **`:422` → `0` (number) not `"0"` (string)** — confirm the toolkit emits a JSON number (it does: `TimestampArg::Unix(0) → json!(0)`).
4. **2b reword** keeps the `now`-form explanation accurate (it does — `now` still emits `"now"`); confirm no new falsehood.

## 7. Out of scope
- `docs/manual/` (the toolkit CLI manual — already fixed in v0.47.3).
- The `next_index`/other fields in the `:422` JSON example.
- Any `manual-gui-v*` release/version bump.
