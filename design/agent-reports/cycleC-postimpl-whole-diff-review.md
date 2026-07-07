# POST-IMPL WHOLE-DIFF REVIEW — Cycle C — round 1

**Verdict: GREEN (0 Critical / 0 Important)**
**Reviewer:** FRESH independent opus execution reviewer (cold read vs the R0-GREEN SPEC/plan).
**Scope:** `git diff e226c3a8..HEAD` — 2 commits (P0 `6cb8b297` expander+wiring+tests+message/misattribution, P1 `7686b6f4` manual prose). Read-only. Persisted verbatim per CLAUDE.md.

The implementation faithfully, correctly, and safely realizes the R0-GREEN design. Ship-ready for the release ritual.

## 1. Build/test gates — PASS
- `cargo test -p mnemonic-toolkit` → 0 failed (full suite). New/repurposed: `cli_bip388_double_star_shorthand` 12/12, `cli_gui_schema_classify_descriptor` 9/9, `cli_import_wallet_descriptor` 10/10.
- `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` → clean.

## 2. Funds/correctness — every attack vector SAFE
`expand_literal_double_star` (parse_descriptor.rs:414) terminator-anchored + byte-safe:
- **Byte safety** — `find("/**")` → ASCII `/` offset; `pos+3` past 3 ASCII bytes → char boundary; slices safe; `.chars().next()` handles multibyte. No panic.
- **Non-final `/**` (origin/key)** — `[deadbeef/**]` followed by `]` (not in `) , } # ws EOS`) → untouched; `/**` inside base58/xonly impossible. SAFE.
- **`/***`,`/**'`** — next char not a terminator → copied through; scan resumes. SAFE (unit + e2e).
- **Partial multisig** — `sortedmulti(2,A/**,B/**)`: first before `,`, second before `)`; both expand. No silent partial → no wrong change chain; multi-key equivalence oracle backstops.
- **`--miniscript` abstract-label path** — expander only in `translate_descriptor`'s concrete `--descriptor` path; abstract path untouched. SAFE.

## 3. Call-site completeness — all 8 IN sites wired; no 9th path
- Chokepoint `parse_descriptor:946` covers all 14 production `parse_descriptor()` callers.
- Direct `lex_placeholders` callers (`bundle:1397`, `verify_bundle:1386`) covered by per-command expanders at `bundle:1395`/`verify_bundle:1354` (before Concrete/AtN split).
- Direct raw-user `from_str`: `descriptor_intake:301`, `roundtrip:247` (canonicalize_bsms + canonicalize_bitcoin_core), `export_wallet:522`, `bsms:307`, `cost/strip:29`.
- **No unwired user-`/**` path:** remaining `from_str` (`green.rs:52`, `export_wallet:640/803`, `nostr:142`, `wallet_export/bsms:105`, `verify_bundle:951` recompose, `descriptor_builder/gate:171`, `wallet_export/pipeline`, `bitcoin_core` import) all consume toolkit-generated/canonical descriptors (never `/**`). Plain `descriptor` format has no canonicalize arm (`_ => None` import_wallet:1468); sparrow/specter use pre-existing `expand_bip388_policy`. canonicalize_* family fully accounted for.

## 4. Equivalence oracles — non-tautological
§7.3 compares `/**` bundle JSON vs an INDEPENDENTLY-invoked `/<0;1>/*` bundle (separate run, literal explicit text) across wpkh/tr/sortedmulti; multi-key cell proves both `/**` expand. §7.4 AtN present. compare-cost cell = equivalence form (`/**` stderr+exit == explicit; contains "multipath key cannot be a DerivedDescriptorKey"; NOT "invalid child number format" — proves expansion fired). Raw-echo `descriptor`-field nulling legitimate (derives from ORIGINAL args.descriptor; md1/mk1/ms1 cards compared byte-for-byte).

## 5. Idempotence — no double-expansion
Non-`/**` → `Cow::Borrowed` (no-op asserted); JSON `@N/**` → `expand_bip388_policy` emits `/<0;1>/*` → expander no-op; `.into_owned()` re-enters parse_descriptor which re-expands the already-`/<0;1>/*` as no-op. §7.8 pins JSON path.

## 6. Docs match code
41-mnemonic.md + 45-foreign-formats.md accurately state `/**` ACCEPTED (→`/<0;1>/*`) on the exact literal-descriptor surface set, `/0/**` STILL rejects, `/***`/`/**'` untouched. compare-cost correctly OMITTED from the "accepted" list (no false multipath-accept claim). No residual false "/** rejected".

## 7. Misattribution — complete
No residual `/**`-attributed-to-BIP-389 in src/tests/docs; all `/**` → BIP-388. Genuine `/<a;b>/*`-is-BIP-389 refs left (41-mnemonic:141, 45-foreign-formats:308/1045, 39-cross-format:76 — multipath form IS BIP-389). New BIP-388 bibliography entry.

## 8. Collateral/determinism
No new `ToolkitError` variant; error.rs + mlock.rs untouched. Non-`/**` → `Borrowed` (byte-identical `.into_owned()` clone) → zero behavior change. No other subsystem touched.

## Non-blocking observation (not a finding)
The expander terminator set includes `#`, which the `lex_placeholders` residue floor (parse_descriptor.rs:208) does not. Benign: `/**#…` requires a bare unwrapped key-expression with a checksum (never a valid top-level descriptor — wrapping puts `)` before `#`), so never occurs on real input; expand-then-reject is funds-safe. Documented degenerate in SPEC §5. No action.

**GREEN — the release ritual (v0.78.0, MINOR; codecs NO-BUMP; no GUI/schema_mirror) may proceed.**
