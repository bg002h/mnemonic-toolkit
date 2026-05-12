# v0.8 SPEC review — r1

Date: 2026-05-11
Reviewer: opus-architect (r1) via general-purpose agent

## Summary

- Vendor URLs: 1C / 2I / 1L / 0N
- Source file:line refs: 1C / 1I / 0L / 0N
- Internal consistency: 0C / 2I / 0L / 1N
- Authoritative-data freshness: 0C / 0I / 0L / 0N
- House-style mirror: 0C / 0I / 1L / 0N
- Refusal-message shapes (§4 "(exit 2)" + byte-pinning): 0C / 1I / 1L / 0N
- §12 dispatch coverage: 0C / 0I / 0L / 0N

Total: 2C / 6I / 2L / 1N

---

## Findings — Vendor URLs

### C-1 — Coldcard `multisig-wallets.md` URL is a 404; no such file in the repo

**Location:** `design/SPEC_export_wallet_v0_8.md:118`

**Evidence:** §5.2 cites `https://github.com/Coldcard/firmware/blob/master/docs/multisig-wallets.md` as the format reference for the entire Coldcard multisig text emitter. `WebFetch` returns 404; `gh api /repos/Coldcard/firmware/contents/docs` returns only `generic-wallet-export.md`, `paperwallet.pdf`, `sample-electrum-wallets/`. There is no `multisig-wallets.md` in the `Coldcard/firmware` repo at any path. The canonical Coldcard multisig file format is documented at `https://coldcard.com/docs/multisig` (verified resolving to a 302 redirect → 200, content describes the exact Name/Policy/Format/Derivation/`<XFP>: xpub` line format that §5.2 specifies, with the 20-char Name truncation and SLIP-132-accepting-but-stored-as-BIP-32 invariant).

The Phase 1 RED tests in §13 (`coldcard_multisig_2of3_wsh.txt`, `jade_multisig_2of3_wsh.txt`) will be byte-pinning to a fixture whose authoritative source citation is broken. A reviewer following the SPEC's citation chain hits a dead link and cannot verify the byte-format claims.

**Fix:** In `design/SPEC_export_wallet_v0_8.md:118`, replace
```
Format reference: <https://github.com/Coldcard/firmware/blob/master/docs/multisig-wallets.md>; identical bytes accepted by Jade — see §6.
```
with
```
Format reference: <https://coldcard.com/docs/multisig> (Coldcard's published spec; the firmware repo does not host this doc in `docs/`); identical bytes accepted by Jade — see §6.
```

### I-1 — Sparrow `defaultPolicy.numSignaturesRequired` is not a serialized field

**Location:** `design/SPEC_export_wallet_v0_8.md:159-163`

**Evidence:** §7 example JSON places `"numSignaturesRequired": 1` inside `defaultPolicy`. I fetched `https://raw.githubusercontent.com/sparrowwallet/drongo/master/src/main/java/com/sparrowwallet/drongo/policy/Policy.java` and `Miniscript.java`: `Policy` declares exactly two instance fields — `private String name` and `private Miniscript miniscript`. `getNumSignaturesRequired()` is a derived getter that delegates to `miniscript.getNumSignaturesRequired()`; it is NOT an instance field that participates in serialization. Emitting `numSignaturesRequired` as a sibling of `name` and `miniscript` produces a JSON shape Sparrow's loader will silently ignore (Jackson is lenient) but that fails byte-equality with any reference fixture exported by Sparrow itself.

The Phase 2 RED fixtures (`sparrow_single_wpkh.json`, `sparrow_multi_2of3_wsh_sortedmulti.json`) will be locked to an emitter shape divergent from Sparrow's actual round-trip output — a v0.9 reviewer comparing toolkit output to a freshly-exported Sparrow file would see the spurious key.

**Fix:** In `design/SPEC_export_wallet_v0_8.md:159-163`, remove the `numSignaturesRequired` line and update the bullet text below the JSON block. Replace:
```
  "defaultPolicy": {
    "name": "Default",
    "miniscript": { "script": "wpkh(bip39)" },
    "numSignaturesRequired": 1
  },
```
with:
```
  "defaultPolicy": {
    "name": "Default",
    "miniscript": { "script": "wpkh(@0/**)" }
  },
```
and delete the `- defaultPolicy.numSignaturesRequired: threshold (1 for singlesig).` bullet at line 183. The threshold is conveyed implicitly via the `multi(K,...)` or `sortedmulti(K,...)` argument count inside `miniscript.script`.

### I-2 — Sparrow miniscript token `wpkh(bip39)` is fabricated; not a real Sparrow descriptor

**Location:** `design/SPEC_export_wallet_v0_8.md:161, 182`

**Evidence:** §7's JSON literal uses `"script": "wpkh(bip39)"` and §7's bullet at line 182 lists `wpkh(bip39)` as the singlesig miniscript expression. `gh search` for `repo:sparrowwallet/drongo "wpkh(bip39)"` returns 0 matches. Sparrow's `Miniscript` class stores a descriptor string with concrete key placeholders or `@N/**` substitutions, not a literal `bip39` token. The token `bip39` has no defined meaning in BIP-380 descriptor grammar nor in Sparrow's policy templating.

**Fix:** In `design/SPEC_export_wallet_v0_8.md:161`, change `"script": "wpkh(bip39)"` to `"script": "wpkh(@0/**)"`. In `design/SPEC_export_wallet_v0_8.md:182`, change the bullet to use `@0/**` placeholder syntax consistently: `wpkh(@0/**) (singlesig wpkh) / wsh(sortedmulti(K, @0/**, @1/**, ...)) (multisig wsh-sortedmulti) / wsh(multi(K, @0/**, ...)) (multisig wsh-multi) / tr(@0/**) (singlesig p2tr)`.

### I-3 — Specter REST URL documents the GET response, not the import shape

**Location:** `design/SPEC_export_wallet_v0_8.md:192`

**Evidence:** §8 cites `https://docs.specter.solutions/desktop/api/ep_wallets_wallet/` as the REST schema reference for the import JSON. The URL resolves (HTTP 200), but its content documents the GET endpoint response (fields: `name`, `alias`, `description`, `address_type`, `recv_descriptor`, `change_descriptor`, `keys`, `devices`, `sigs_required`, etc.) — none of which are `label` or `blockheight` as scalars. The actual Specter wallet-import shape (verified via `src/cryptoadvance/specter/util/wallet_importer.py` on master: searched via `gh api '/search/code?q=repo:cryptoadvance/specter-desktop+"blockheight"+"descriptor"+"devices"'` → 12 hits including `wallet_importer.py`) accepts `descriptor` OR `recv_descriptor`, `label` (alias of `name`), `devices`, `blockheight`, `labels`. The cited URL is not authoritative for the schema §8 specifies.

**Fix:** In `design/SPEC_export_wallet_v0_8.md:192`, replace the REST schema URL with the actual import code path:
```
Format reference: <https://github.com/cryptoadvance/specter-desktop/blob/master/src/cryptoadvance/specter/util/wallet_importer.py> (canonical import-shape authority — the REST GET schema at <https://docs.specter.solutions/desktop/api/ep_wallets_wallet/> is a different shape).
```

### L-1 — Blockstream Green Help-Center URL returns 403 to non-browser clients

**Location:** `design/SPEC_export_wallet_v0_8.md:261, 267`

**Evidence:** `WebFetch` and `curl` against `https://help.blockstream.com/hc/en-us/articles/19340800530713-Set-up-watch-only-wallet` returns HTTP 403 Forbidden. The URL likely works in a browser (Zendesk-style help-desk with anti-scraping); a CI or reviewer using a programmatic fetcher cannot verify the cited content. The same URL is embedded into the §10 emitted comment line.

**Fix:** Keep the URL (it is the right human-facing surface), but in §10 add an inline note that the URL is human-only-resolvable, and document an alternate authoritative reference if any. Append to `design/SPEC_export_wallet_v0_8.md:261`:
```
(Note: this URL is the canonical user-facing help article; programmatic fetchers may receive 403 from Zendesk anti-scraping. Verify in a browser.)
```

## Findings — source file:line refs

### C-2 — `REFUSAL_SECRET_INPUT` is at `wallet_export.rs:17-18`, not `17-25`

**Location:** `design/SPEC_export_wallet_v0_8.md:42`

**Evidence:** §3 cites `src/wallet_export.rs:17-25` for `REFUSAL_SECRET_INPUT`. Reading `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/wallet_export.rs`:
- Line 17: `pub const REFUSAL_SECRET_INPUT: &str =`
- Line 18: refusal text literal + closing `;`
- Lines 19 (blank), 20-25: `format_stub_message` function (a different symbol).

`REFUSAL_SECRET_INPUT` occupies lines 17-18 only. The same incorrect citation appears in `IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md:12`. A reader trying to verify the byte-exact refusal text by jumping to "lines 17-25" gets the refusal string plus a separate helper function and misreads the scope of the constant.

**Fix:** In `design/SPEC_export_wallet_v0_8.md:42`, change `src/wallet_export.rs:17-25` to `src/wallet_export.rs:17-18`. Mirror the same edit in `IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md:12`.

### I-4 — Stub-arm line range cited inconsistently as `148-154` (§2) and `148-155` (§12)

**Location:** `design/SPEC_export_wallet_v0_8.md:38, 378`

**Evidence:** §2 line 38: `Stub arms for sparrow / specter at src/cmd/export_wallet.rs:148-154 are deleted.` §12 line 378: `The existing stub arms for Sparrow / Specter at src/cmd/export_wallet.rs:148-155`. Reading `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/export_wallet.rs`:
- Line 147: `match args.format {`
- Lines 148-150: `CliExportFormat::Sparrow => { return Err(...); }`
- Lines 151-153: `CliExportFormat::Specter => { return Err(...); }`
- Line 154: `_ => {}`
- Line 155: closing `}` of the match

The truly-stub arms (the ones that get deleted) are 148-153. The line 154 wildcard and line 155 close-brace REMAIN (the wildcard becomes unreachable and would be removed by clippy; the match closes the same way). Both the §2 and §12 citations are off; they also disagree with each other.

**Fix:** In `design/SPEC_export_wallet_v0_8.md:38`, change `src/cmd/export_wallet.rs:148-154 are deleted` to `src/cmd/export_wallet.rs:148-153 are deleted (the wildcard arm at line 154 and the match-close at line 155 remain)`. In `design/SPEC_export_wallet_v0_8.md:378`, change `at src/cmd/export_wallet.rs:148-155` to `at src/cmd/export_wallet.rs:148-153`. Make both citations agree.

## Findings — internal consistency

### I-5 — §2 says stub arms are "deleted"; §12 says they "remain until each format's phase replaces them"

**Location:** `design/SPEC_export_wallet_v0_8.md:38` vs `design/SPEC_export_wallet_v0_8.md:378`

**Evidence:** §2 line 38: `Stub arms for sparrow / specter at src/cmd/export_wallet.rs:148-154 are deleted.` This reads as a Phase 0 (or Phase 1) action. §12 line 378: `(R1-I3 hardening: stub arms remain until each format's phase replaces them — Phase 1 does NOT delete them). The deletion happens incrementally: Phase 2 deletes the Sparrow stub arm; Phase 3 deletes the Specter stub arm.` These are flatly contradictory: §2 asserts deletion as part of the grammar change; §12 asserts incremental deletion per-phase with R1-I3 hardening explicitly forbidding Phase 1 deletion.

A Phase 1 implementer who only reads §2 will delete both stub arms in Phase 1 and trigger a behavior regression — between Phase 1 and Phase 2/3 the Sparrow/Specter `--format` selectors would panic on `unreachable!()` or fall into the Coldcard/Jade dispatch instead of returning a clean stub refusal.

**Fix:** Reword `design/SPEC_export_wallet_v0_8.md:38` to defer the deletion narrative to §12. Replace:
```
The `--format` enum gains six values; existing `bitcoin-core` and `bip388` remain default-priority. Stub arms for `sparrow` / `specter` at `src/cmd/export_wallet.rs:148-154` are deleted.
```
with:
```
The `--format` enum gains six values; existing `bitcoin-core` and `bip388` remain default-priority. Stub arms for `sparrow` / `specter` at `src/cmd/export_wallet.rs:148-153` are removed incrementally per §12 (Phase 2 deletes the Sparrow stub; Phase 3 deletes the Specter stub).
```

### I-6 — §4 enumerates `Account` and `Network` as MissingField variants, but both have clap defaults so cannot be "missing"

**Location:** `design/SPEC_export_wallet_v0_8.md:48-58`

**Evidence:** §4 lists 9 `MissingField` discriminants. `Account` (#6) and `Network` (#7) are user-supplied via `--account` (per §5.1: "clap default 0") and `--network` (per §2: "default: ... mainnet" per existing v0.7 behavior). Since both have clap defaults, the resolved `EmitInputs` is always populated for these fields. They cannot be missing.

If these variants are reserved for future flag-removal scenarios, that's worth a SPEC note; otherwise they're dead variants that complicate the `collect_missing()` trait method (every emitter must explicitly NOT report them).

**Fix:** Either (a) remove `Account` and `Network` from the `MissingField` enum at `design/SPEC_export_wallet_v0_8.md:53-55` and renumber subsequent variants; or (b) add an explanatory paragraph after the enumeration noting that `Account` and `Network` are reserved for `--account` / `--network` flag-suppression scenarios and are unreachable in the current grammar. Recommend (a) — it shrinks the enum to 7 variants and removes a class of write-tests-for-unreachable-code burden.

### N-1 — §12 trait method `collect_missing` signature is described twice with inconsistent role description

**Location:** `design/SPEC_export_wallet_v0_8.md:316`

**Evidence:** §12 trait block declares `fn collect_missing(inputs: &EmitInputs) -> Vec<MissingField>;` (associated function, no `self`). §4 line 71 describes the same logic flow as a free function `build_missing_fields_refusal(format, &[MissingField]) -> String`. The trait method is the per-format collector; the free function is the per-format formatter — these are two different things but the SPEC reads as if they're both "missing-fields refusal builders" without distinguishing roles. Phase 0 implementer may conflate them.

**Fix:** Append a one-line role distinction to the trait block at `design/SPEC_export_wallet_v0_8.md:319`:
```
//   collect_missing: per-format predicate (which fields does THIS format require?)
//   build_missing_fields_refusal (in mod.rs): cross-format formatter (turns the
//   collected list into the byte-exact refusal text per §4).
```

## Findings — house-style mirror

### L-2 — SPEC carries `R1-C1`/`R1-I1`/...-hardening tags inline; v0.7 SPEC has none

**Location:** `design/SPEC_export_wallet_v0_8.md:75, 106, 214, 312, 341, 378, 410, 412`, etc.

**Evidence:** v0.7 SPEC (`design/SPEC_export_wallet_v0_7.md`) is plain prose — no "R1-X-hardening" tags in section text. v0.8 SPEC pervasively embeds parenthetical tags like `(R1-N3 hardening)`, `(R1-C1 hardening)`, `(R1-I1 hardening)` directly in normative paragraphs. These tags refer to the plan-level R1 review log (`Iterative-review log` section at line 428-432). For a v0.9 reader, the tags are noise: the resolutions are normative SPEC content regardless of which round produced them. v0.7's convention is to fold review resolutions silently and log them in the trailing review-log section.

Not a correctness bug, but a substantive house-style deviation that compounds across the doc (8+ inline tags).

**Fix:** Strip the `(R1-XN hardening)` parentheticals from §4 line 75, §5.1 line 106, §9 line 214, §12 lines 312, 341, 378, §13 lines 410, 412 (8 sites). Keep the `Iterative-review log` section at line 428-432 as the audit trail. Example: change `**Per-slot vs global field ordering (R1-N3 hardening):**` to `**Per-slot vs global field ordering:**`.

## Findings — refusal-message shape

### I-7 — `(exit 2)` embedded in stderr message text breaks v0.7 precedent and Unix convention

**Location:** `design/SPEC_export_wallet_v0_8.md:67`

**Evidence:** §4 refusal shape ends with the line:
```
Re-invoke with all missing fields supplied. (exit 2)
```
v0.7 §3 refusal (the precedent set by this very subcommand) does not embed exit code in message text:
```
error: mnemonic export-wallet is watch-only by definition; supply only xpub/fingerprint/path slots. To produce an artifact that includes secret material, use 'mnemonic bundle'.
```
The exit code is metadata communicated via the process exit status (set to 2 by `ToolkitError`), not stderr text. Unix convention is that exit codes are out-of-band; embedding them in stderr is a smell that downstream parsers will key on, then break when the text changes.

This embedded-(exit-2) shape would set a new precedent for the toolkit that the rest of the codebase does not follow.

**Fix:** Remove ` (exit 2)` from `design/SPEC_export_wallet_v0_8.md:67`. The refusal shape becomes:
```
error: mnemonic export-wallet --format <FORMAT> requires the following missing fields:
  - <field_name> for slot @<N> (<one-line explanation pointing at the supply mechanism>)
  - <field_name> for slot @<N> (...)
Re-invoke with all missing fields supplied.
```
The exit code 2 is conveyed by `ToolkitError::ExportWalletMissingFields` (already specified at line 72). Document the exit code in the bullet at line 46 (`Exit code 2.`) rather than embedding it in user-facing text.

### L-3 — §5.1 bip86 refusal and §6 Jade singlesig refusal are 2-space-indented in markdown; byte-exact pin needs clarification

**Location:** `design/SPEC_export_wallet_v0_8.md:108-110, 143-145`

**Evidence:** §5.1 bip86 refusal at lines 108-110:
```
  ```
  error: --format coldcard does not yet support BIP-86 (P2TR) — Coldcard's generic-wallet-export schema documents only bip44/bip49/bip84. Use --format bitcoin-core (descriptor) or --format sparrow for taproot watch-only setup.
  ```
```
The fenced block is indented under a bullet (2 leading spaces). The byte-exact emitted string presumably should NOT carry the 2-space markdown indent — but the SPEC does not say so. A Phase 1 implementer eyeballing the rendered markdown might or might not include the indent. §6 Jade singlesig (lines 143-145) is at column 0 (no bullet-indent ambiguity) but the same principle applies for any line-wrap in rendered markdown.

**Fix:** Add a one-line clarification at `design/SPEC_export_wallet_v0_8.md:107` (just before the §5.1 fenced block):
```
  The byte-exact emitted text below has no leading whitespace (the markdown-fenced-block indent under this bullet is presentation only):
```
And mirror the same note at line 142 for §6.

---

## Closing

Two C findings (Coldcard multisig URL 404; `REFUSAL_SECRET_INPUT` line ref off-by-7) and six I findings (Sparrow shape inaccuracies, Specter URL authority, internal §2-vs-§12 stub-arm contradiction, dead-variant MissingField fields, `(exit 2)` precedent break, line-ref inconsistency). The vendor-URL block in particular needs another pass — half of the cited authorities either don't resolve or don't document what the SPEC implies they document.

Authoritative-data freshness on `FINAL_SEED_VERSION = 71` verified directly against current Electrum master via WebFetch — the SPEC's `71` claim is correct, and the deferral to a Phase 4 spike for the actual emitted value is the right call. §12 dispatch covers all 8 §2 enum variants exactly.

Source files cited (verified):
- `crates/mnemonic-toolkit/src/wallet_export.rs:17-18` actual `REFUSAL_SECRET_INPUT`
- `crates/mnemonic-toolkit/src/cmd/export_wallet.rs:147-155` actual match-block layout
- `crates/mnemonic-toolkit/src/cmd/convert.rs:224` actual `ScriptType` declaration — SPEC claim correct
- `crates/mnemonic-toolkit/src/slip0132.rs:169` actual `BIP84_REF_ZPUB`
