# Architect review (brainstorm-stage, pre-SPEC) — descriptor-form symmetry (A1) + stderr output-type advisory (B)

**Date:** 2026-05-31 · **Reviewer:** opus architect (feature-dev:code-architect) · **Repo SHA at review:** `ea8ba88`
**Scope:** (A) the A1 "bare-concrete ↔ @N descriptor symmetry" design; (B) evaluation of the proposed stderr output-type advisory.

> Persisted verbatim per CLAUDE.md ("Per-phase architect-review agent outputs persist verbatim to design/agent-reports/ BEFORE the fold-and-commit step"). This is the brainstorm-stage design review that precedes the formal SPEC R0.

---

## Verdict (A)
Architecturally sound at the seam, but the chokepoint enum as drawn is leaky, and the export-wallet `@N`-via-`--slot` arm is scope-creep that should be dropped in favor of a clear error. **2 Critical, 3 Important, 3 Minor.**

### Verification summary (all load-bearing claims re-checked)
- Asymmetry confirmed: `bundle` (`cmd/bundle.rs:1068`) + `verify-bundle` (`cmd/verify_bundle.rs:614`) → `lex_placeholders` → @N-only; `export-wallet --descriptor` (`cmd/export_wallet.rs:332`) → `MsDescriptor::from_str` → concrete-only.
- Converter `concrete_keys_to_placeholders` (`wallet_import/pipeline.rs:52`) returns `(String, Vec<ParsedKey>, Vec<ParsedFingerprint>)`, preserves declaration order, SLIP-132-aware.
- bundle already does concrete→@N internally on `--import-json` (`cmd/bundle.rs:1644-1656`).
- export-wallet `--template` sources keys from `args.slot` via `resolve_slots` + `validate_watch_only_resolved` (`cmd/export_wallet.rs:347-357`; `wallet_export/mod.rs:131`).
- Seedless watch-only bundle path exists: `BundleMode::SingleSigWatchOnly`/`MultisigWatchOnly` (`bundle.rs:1666/1669`); ms1 omitted (`:835`); D9 suppressed for watch-only (`:910`). Emits md1+mk1, no ms1.

## Critical

**C1 — `classify_descriptor_form` enum is leaky; the `Concrete` arm cannot flow through the same path the `@N` arm uses, because bundle/verify-bundle source `@N` keys from `--slot`, not the descriptor.**
`bundle_run_unified_descriptor` (`bundle.rs:1068-1077`) requires `--slot` to cover every `@N`; `verify_bundle.rs:622-624` binds keys from `args.slot`. The `@N` arm's key vector is populated from `--slot`, never the descriptor. The `Concrete` arm carries keys extracted from the descriptor. Two distinct downstream binding pipelines → a single uniform enum return does not compose.
**Fix:** seam = dispatch fork, not uniform enum. `AtN` → existing slot-sourced pipeline unchanged; `Concrete` → new "inline-keys → ResolvedSlots" adapter (lift the shape at `bundle.rs:1644-1659` into a shared `descriptor_concrete_to_resolved_slots` helper, call from all surfaces). They converge AT `parse_descriptor(input,&keys,&fps)`, nowhere earlier.

**C2 — "reuse the converter verbatim" is unsafe: `key_regex` rejects `h`-form hardened paths → real bare-concrete descriptors fail with a misleading "no keys found."**
`key_regex` (`pipeline.rs:38`) = `\[([0-9a-fA-F]{8})((?:/\d+'?)+)\]([xtyzuvYZUV]pub…)` — path group `(?:/\d+'?)+` accepts only apostrophe. `lex_placeholders` (`parse_descriptor.rs:70`) accepts `(?:'|h)?`. Core/Sparrow emit `h`-form (`[fp/84h/0h/0h]xpub…`). A `h`-form descriptor → zero matches → `"import-wallet: bsms: parse error: no [fp/path]xpub keys found in descriptor"` (`pipeline.rs:117`) despite keys present. No upstream `h`→`'` normalization exists.
**Fix:** widen `key_regex` path group to `(?:/\d+(?:'|h|H)?)+` (single source of truth; benefits the 8 import callers). Also the hardcoded `import-wallet: bsms:` error prefix (`pipeline.rs:74,107,116`) leaks the wrong command name on a `bundle --descriptor` paste — new entry points must remap the prefix (precedent: electrum/sparrow rewrite `bsms:`→their name).

## Important

**I1 — Drop the export-wallet `@N`-via-`--slot` arm; make export-wallet `@N` a clear refusal instead.**
`--descriptor` is pure miniscript passthrough (`export_wallet.rs:332`); `--template` already owns the `--slot`-sourced path (`:347`). Bolting `@N`-via-`--slot` onto `--descriptor` makes `--descriptor` sometimes-consume-`--slot` (mode-dependent flag coupling the codebase elsewhere treats as a hard mutex, `:310`), and overlaps semantically with `--from-import-json` (`:185`, `conflicts_with_all=["template","descriptor"]`). "All three accept both forms" is better served by bundle+verify-bundle gaining both, and export-wallet giving a clear actionable error on `@N` → `"use --template <T> --slot @N.xpub=… or --from-import-json"`.

**I2 — Ordering/sortedmulti/distinctness is SAFE; add one convergence cell.**
`concrete_keys_to_placeholders` binds @N in first-occurrence order (`pipeline.rs:60,99`); miniscript Display sort is orthogonal. `check_key_vector_distinctness` (`parse_descriptor.rs:1208`) keys on typed `(xpub, DerivationPath)`, not slot index → sortedmulti safe. **Fix:** convergence test: out-of-lexicographic-order sortedmulti, concrete `--descriptor` vs `@N --descriptor + --slot`, byte-identical md1/mk1 (home: `tests/cli_wallet_cross_format_convergence.rs`).

**I3 — Detection: `@\d` discriminant sound; but origin-less-key error must route through md-codec policy, and mixed-form needs an explicit guard.**
`@\d` cannot misfire (base58 has no `@`; origins are hex+digits). But (a) origin-less single-sig (`wpkh(xpub/0/*)`) — the "clear error" must come from md-codec's origin-required policy, not the regex's silent non-match. (b) Mixed `@N`+inline-xpub: as drawn, `@\d`-first routes mixed → `AtN` arm → `lex_placeholders` parses the `@N` and ignores the inline xpub → wrong-but-not-errored. **Fix:** explicit "matches both `@\d` AND `key_regex`" guard → error before dispatch.

**I4 — SemVer PATCH + no-GUI-lockstep is correct, but manual mirror is non-trivial and new error strings must be pinned.**
No new flag/value → `schema_mirror` untouched; `--json` unchanged → PATCH (v0.38.1), GUI-lockstep-free. BUT the three `--descriptor` prose blocks (`docs/manual/src/40-cli-reference/`) document one accepted form each and must update; chapter-45 worked examples are execution-gated (`verify-examples.sh`). Pin every new error string (I1 refusal, I3 mixed-form).

## Minor
- **M1** — `cli-subcommands.list` / flag-coverage lint untouched (no flag/subcommand added); state explicitly in the spec.
- **M2** — proposed enum type wrong: converter returns `Vec<ParsedFingerprint>` (`{i:u8, fp:[u8;4]}`, `pipeline.rs:54`), not `bitcoin::bip32::Fingerprint`.
- **M3** — taproot: concrete `tr(NUMS, leaf-xpubs)` (Coldcard taproot multisig) — NUMS has no `[fp/path]` so won't match `key_regex` (fine); add one convergence cell. No new taproot scope.

---

## Recommendation (B) — stderr output-type advisory

**Call: SEPARATE, smaller cycle. Do NOT fold into A1. Re-scope from "classify the artifact" to "extend D9 into a watch-only-positive companion line." Do NOT build the full 2-axis taxonomy.**

- **Why separate:** done right it touches ~12 output-producing commands; a half-applied advisory is worse than none (users learn "no line = uncovered," not "= safe"). All-surfaces coverage is a correctness property → cannot ride A1's 3-command scope.
- **D9 relationship (verified):** D9 is per-command manually-placed, fires only when secret on stdout. Unconditional sites: `derive_child.rs:306`, `silent_payment.rs:286`, `nostr.rs:251`, `electrum_decrypt.rs:149`, `slip39.rs:684`. Conditional: `convert.rs:1099` (`is_argv_secret_bearing`), `bundle.rs:910` (`any_secret_bearing`), repair/inspect (`secret_advisory.rs:48`). Canonical text at `secret_advisory.rs:59-63` but four sites inline the literal. **Subsume-by-complement:** keep D9's exact pinned text for the secret case (changing it churns the transcript corpus); add positive lines for the silent cases; one shared helper; consolidate the four inlined literals.
- **Taxonomy (minimal, mutually exclusive — 3 classes):** `private key material (can spend)` / `watch-only` / `template`. Drop secondary axes (single/multi, mainnet/testnet) — decoration; the user's boundary is spend-capability. Prefer "private key material (can spend)" over "signing wallet" (toolkit emits spendable key material that is not a signing op).
- **No-signing boundary (key finding):** the toolkit DOES emit secret-bearing stdout today: `convert --to xprv/wif/bip38/ms1/entropy/seedqr` (`secret_taxonomy.rs:76`), `derive-child` (BIP-85), `silent-payment`, `nostr` (nsec), `electrum-decrypt`, `slip39`/`seed-xor`/`final-word`, seed-bearing `bundle`/`import-wallet`. "No signing" = no message/PSBT signing, not no private-key emission → class 1 already occurs.
- **Feasibility:** every wallet-artifact surface can determine its class statically at the emit site (per-command, no single funnel; classification centralizes in a helper). Inert-output commands (`decode-address`, `verify-message`, `inspect`, `compare-cost`) emit NO advisory — documented as inert so absence ≠ coverage gap.
- **SemVer:** stderr-only → PATCH (precedent `silent-default-with-stderr-notice`); transcript corpus re-capture is the bulk of the work (another reason to keep out of A1).

---

## Fold decisions (controller, 2026-05-31)
User took both architect calls: **I1 accepted** (export-wallet stays concrete + clear `@N` redirect error); **(B) = separate next cycle** (A1 now, advisory next). C1/C2/I2/I3/I4/M1/M2/M3 all folded into the SPEC. The advisory is filed for the following cycle.
