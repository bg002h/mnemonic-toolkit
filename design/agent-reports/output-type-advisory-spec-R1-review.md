# SPEC R1 review (re-dispatch after R0 fold) — output-type-stderr-advisory Phase 1

**Date:** 2026-05-31 · **Reviewer:** opus architect · **SHA:** `18cfdce` · **Verdict: RED (0C/2I/4m).**

## R0 folds confirmed landed
- C1: `repair.rs:1333` calls `secret_on_stdout_warning(outcome.kind,_)` + writes repaired card to stdout (`:1316-1318`) ALWAYS (no conditional-stdout caller) → re-route safe for all 5+ callers; reached via `try_repair_and_short_circuit:1300` from verify_bundle (6 sites), seed_intake:182, inspect:135, convert:994. CardKind→{Ms1:P,Mk1:W,Md1:T} coherent.
- I1: electrum_decrypt:149 genuine 3rd `_unconditional` caller (`--json-out` branch :142-145 suppresses). ✓
- I2: 4 doc trees each gated (3× verify-examples.sh + docs/quickstart/Makefile); 12 toolkit test files match; ms literal `cmd/repair.rs:108`. ✓
- I3: coherent with NodeType enum (`convert.rs:31-52`): secret→P, xpub/mk1/address→W, path/fingerprint→inert. ✓

## Important
**I-A — `worst_class_on_stdout` return type unspecified; §4.3 says multi-artifact "→ one line" unconditionally, but the I3 fold made convert path/fingerprint-only ALL-INERT (no line).** OutputClass{P,W,T} has no inert variant. *Fix:* `worst_class_on_stdout(...) -> Option<OutputClass>` (None=all-inert); caller `if let Some(c)=… { emit(c) }`; correct §4.3 "→ one line, OR none when all artifacts inert (convert path/fingerprint-only)."
**I-B — §3 (line 88) + §4.1 (line 103) wrongly call `inspect`/`convert` "inert on NORMAL branch."** `inspect.rs:155-156` emits P for ms1 on normal (decode-success) branch; `convert.rs:1099` emits P/W on normal. The short-circuit (`inspect:135`, `convert:994`) is reached ONLY on input-decode-FAILURE (`is_codec_decode_err`), propagated via `?` → mutually exclusive with normal emit. Only verify-bundle + xpub-search are normal-branch-inert. *Fix:* scope "inert normal branch" to verify-bundle + xpub-search; for inspect/convert state "normal branch emits its §3-row class; auto-repair exit-5 (input-decode-failure) branch emits the repaired card's class."

## Minor
m1 — ms-cli literal `cmd/repair.rs:106-109` (SPEC :107-108). m2 — convert's normal gate `:1099` uses `is_argv_secret_bearing()` (wider, incl. minikey — output-unreachable today) NOT `SECRET_NODE_TYPES`; plan-doc should use `is_argv_secret_bearing` (matches live) or note equivalence. m3 — `ms derive` already emits a language-defaulted stderr note (`derive.rs:246-248`) coexisting with the new W-line; P4 derive cell asserts BOTH. m4 — §4.2 lumps `ms-cli/src/main.rs:120` (doc-comment) into the test sweep (cosmetic).

## SemVer/phases: correct (PATCH both, no GUI/list/lint lockstep, toolkit tag v0.38.2 + ms crates.io v0.5.1, no pin bump; P0 helpers before wiring, no forward-symbol refs).

## Controller folds: I-A → Option<OutputClass> return + §4.3 correction. I-B → scope inert-normal-branch to verify-bundle+xpub-search; inspect/convert emit normal class + repaired class on decode-failure branch. m2 → is_argv_secret_bearing; m3 → derive asserts both.
