# v0.8 SPEC review — r2

Date: 2026-05-11
Reviewer: opus-architect (r2) via general-purpose agent

## R1 verification

C-1: resolved. §5.2 line 118 now cites `<https://coldcard.com/docs/multisig>` with the parenthetical "(Coldcard's published spec; the firmware repo does not host this doc under `docs/`)". The 404 `multisig-wallets.md` URL is gone from §5.2; no other section references the dead URL.

C-2: resolved. §3 line 42 now says `src/wallet_export.rs:17-18`. Confirmed against source: `REFUSAL_SECRET_INPUT` occupies lines 17-18, and `format_stub_message` starts at line 21.

I-1: resolved. §7 JSON literal at line 161 no longer carries `numSignaturesRequired`; only `name` and `miniscript` remain inside `defaultPolicy`. The bullet at line 181 explicitly states `numSignaturesRequired` "is a derived getter, NOT a JSON field, and must not appear in the emitted shape". Threshold-via-arg-count narrative is correct.

I-2: resolved. §7 JSON literal line 161 uses `"script": "wpkh(@0/**)"`. Bullet at line 181 lists `wpkh(@0/**)` / `wsh(sortedmulti(K, @0/**, ...))` / `wsh(multi(K, @0/**, ...))` / `tr(@0/**)` consistently. No remaining occurrences of `wpkh(bip39)` anywhere in the SPEC.

I-3: resolved. §8 line 190 now cites `wallet_importer.py` as the canonical import-shape authority, with the REST GET schema URL retained as a "documents a different shape and is not authoritative for file-import" cross-reference.

I-4: resolved. §2 line 38 and §12 line 379 now both cite `src/cmd/export_wallet.rs:148-153`. Citations are mutually consistent and consistent with C-2's source-walk evidence (Sparrow arm 148-150, Specter arm 151-153).

I-5: resolved. §2 line 38 no longer asserts unconditional deletion. New wording: "are removed incrementally per §12 (Phase 2 deletes the Sparrow stub; Phase 3 deletes the Specter stub)". The deletion narrative is now defer-to-§12. §12 line 379 retains the per-phase incremental model with the explicit "Phase 1 does NOT delete them" guard.

I-6: resolved. §4 enumeration is now 7 variants (1. `MasterFingerprint`, 2. `DerivationPath`, 3. `Xpub`, 4. `ScriptType`, 5. `Threshold`, 6. `WalletName`, 7. `IncompatibleFormatForTemplate`). `Account` and `Network` have been removed and an explanatory paragraph at line 57 documents why they are NOT variants (both have clap defaults).

I-7: resolved. §4 refusal-shape line 67 ends with `Re-invoke with all missing fields supplied.` with no trailing ` (exit 2)`. The exit-code-2 callout earlier in §4 (line 46, "Exit code 2.") remains as the bullet-level documentation. Grep for `(exit 2)` returns only the review-log entry.

L-1: resolved. §10 line 259 now reads: `(Zendesk-hosted; programmatic fetchers may receive 403, verify in a browser)`. Both the in-prose URL citation and the help-comment URL retain the same hyperlink (the help-comment is the byte-exact emitted line, so changing it would break the spec); the prose disclaimer is the right place for the 403 note.

L-2: resolved. Grep for `(R1-[A-Z][0-9]+` returns zero hits in normative sections. (The string `R1-` appears only inside the trailing `Iterative-review log` section, which is the audit-trail location v0.7's house-style designates.)

L-3: resolved. Both fenced refusal blocks are now preceded by no-leading-whitespace wording:
- §5.1 line 106: "the emitted string has NO leading whitespace — the markdown-fenced-block indent under this bullet is presentation only"
- §6 line 141: "(no leading whitespace on the emitted line)"

N-1: resolved. §12 trait block at lines 314-316 has inline comments distinguishing `collect_missing` (per-format predicate) from `build_missing_fields_refusal` (cross-format formatter that turns the collected list into the byte-exact refusal text per §4).

Cross-fold from IMPLEMENTATION_PLAN R1 I-5: resolved. §13 fixture-table rows 397-398 now read "pinned to Phase 4 step 0 spike-observed byte shape" for both `electrum_single.json` and `electrum_multi_2of4.json`. The "Coldcard's stale samples" phrasing is gone from the table. §9 line 212 already used the spike-observed framing pre-fold; the table now matches.

Coherence check: the Iterative-review log entry enumerates 13 resolutions. Each log entry's prose matches the actual diff applied in the body of the SPEC. See N-2 below for one mis-labeled cross-cut tag.

## New findings

### N-2 — Review-log cross-cut label "C-2 / I-1" mis-identifies the I-1 entry

**Location:** `design/SPEC_export_wallet_v0_8.md:436`

**Evidence:** Line 436 reads `**C-2 / I-1 (cross-cut).** §3 line-ref `src/wallet_export.rs:17-25` was off-by-7 …`. R1's C-2 is the `wallet_export.rs:17-25` → `17-18` line-ref fix; R1's I-1 is the Sparrow `numSignaturesRequired` removal — two unrelated findings. The cross-cut label conflates them. The actual I-1 resolution is correctly recorded as its own bullet on line 437. The intended cross-cut almost certainly meant "this line-ref correction is mirrored in IMPLEMENTATION_PLAN" — i.e., the cross-cut is between the SPEC's C-2 and the IMPLEMENTATION_PLAN's matching finding, not between SPEC C-2 and SPEC I-1.

**Fix:** Change `**C-2 / I-1 (cross-cut).**` to `**C-2 (cross-cut with IMPLEMENTATION_PLAN R1 I-1).**`. The I-1 entry stands as-is.

### L-4 — §4 per-slot ordering paragraph: "interleaved" mis-describes the example, and the example doesn't show the "global-first" rule

**Location:** `design/SPEC_export_wallet_v0_8.md:75`

**Evidence:** The paragraph asserts two ordering rules — (a) "ALL global-discriminant entries first in enum-discriminant order; THEN per-slot entries in (enum-discriminant, slot-index) tuple order"; (b) the example "`MasterFingerprint for slot @0`, `MasterFingerprint for slot @1`, `DerivationPath for slot @0`, `DerivationPath for slot @1`, etc. — interleaved by global discriminant, not by slot".

Two coherence problems:
1. The example shows ONLY per-slot entries (no `Threshold` / `WalletName` global entries) but the rule says globals come first; the reader can't see the global-first behavior in the example.
2. The word "interleaved" contradicts the body. The example sequence is grouped-by-discriminant (`MF, MF, DP, DP` — all `MasterFingerprint` first, then all `DerivationPath`), which is exactly what `(enum-discriminant, slot-index)` tuple order produces. "Interleaved" suggests `MF@0, DP@0, MF@1, DP@1` — which would be `(slot-index, enum-discriminant)` order — the opposite of what the body specifies and what Phase 1 fixture-byte-pinning will lock.

A Phase 1 implementer reading "interleaved" may write the test wrong; a reviewer reading the example without parsing the body carefully may approve a wrong-ordering implementation.

This is pre-existing (not introduced by R1 fold) but the SPEC is being reviewed-to-convergence, so flagging.

**Fix:** Replace "interleaved by global discriminant, not by slot" with "grouped by enum discriminant, then ordered by slot index within each discriminant — not interleaved by slot". Also extend the example to show the global-first rule: prepend `Threshold` (global, discriminant 5) before the per-slot entries to make the rule observable in the example.

## Summary

Total NEW: 0C / 0I / 1L / 1N

Convergence: YES — 0C/0I. SPEC ready for STOP-and-check-user gate.

Notes:
- L-4 is a pre-existing wording issue that R1 didn't flag; the implementing agent can still ship Phase 1 correctly by reading the body of the paragraph rather than the example label, so it doesn't block convergence.
- N-2 is a one-line review-log relabel.
- All R1-flagged source-file:line refs (`wallet_export.rs:17-18`, `cmd/export_wallet.rs:148-153`) were independently re-verified against the live source files during this pass.
- Vendor URLs C-1 (Coldcard `coldcard.com/docs/multisig`) and I-3 (Specter `wallet_importer.py`) were content-checked in R1; R2 only verified the SPEC text was updated to the corrected URLs.
