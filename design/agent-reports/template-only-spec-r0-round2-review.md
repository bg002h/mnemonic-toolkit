# SPEC R0 round 2 — bundle-md1-template-only-option (opus architect, verbatim)

> Reviewer: opus architect (read+bash; registry md-codec 0.36.0 == descriptor-mnemonic `c85cd49` diff-clean; mnemonic-key `913febc`). **Verdict: GREEN — 0 Critical, 0 Important.** Residual Minors folded. UNCOMMITTED (freeze). SPEC R0 converged.

---

## R0 round 2 — re-review of folds

### C1 — canonical gate now refuses multisig — CLOSED
`canonical_origin` (`canonical_origin.rs:45-79`) returns `Some` for `Pkh/Wpkh/Tr{tree:None}` single-key (`:48-54`) AND `wsh(multi/sortedmulti)→m/48'/0'/0'/2'` (`:58-61`), `sh(wsh(...))→.../1'` (`:65-70`). So `is_some()` alone admits canonical multisig — the `n==1` conjunct is load-bearing, as §4.2 now states. `Descriptor.n` (`encode.rs:18-19`) = distinct `@N` key count; a real k-of-m has n≥2, excluded by `n==1`. With `n==1` the only Some-shapes are pkh/wpkh/tr-keypath. `TemplateFormUnsupportedShape` sorts between `SlotInputViolation` (`error.rs:306`) and `UnknownHrp` (`:313`). One non-blocking edge: degenerate `wsh(multi(1,@0))` is `n==1`+Some → slips the gate, but it carries exactly one key (seed derives it) — NOT the C1 inversion. Minor (guard/test note at plan-doc).

### C2 — restore routing now reachable — CLOSED
Dispatch verbatim: `restore.rs:177-179` `if !args.md1.is_empty() { return run_multisig(...) }`; `run_multisig` reassembles `:1226-1229` then refuses keyless `:1232-1238`. The §4.5 carve-out (`!is_wallet_policy() && n==1 && canonical_origin().is_some()` → new completion; else `run_multisig`, whose `:1232` catches keyless multisig since its `n≥2` falsifies the predicate) is sound + reachable. Predicate computable at `:177` because `reassemble` is a pure fn of `args.md1` (in scope); the fold requires hoisting one `reassemble` call to the dispatch — trivial impl detail.

### I1 — self-check now branches for template form — CLOSED
Function is `self_check_bundle` (`bundle.rs:2139`), NOT `verify_self_consistency` — naming discrepancy in §4.3/round-1 (fix at plan-doc; line numbers accurate). Complete gate set a keyless template trips, all named: `:2151-2156` `!is_wallet_policy()→BundleMismatch`; `:2157-2162` `compute_wallet_policy_id`→`expected_stub`; `:2171-2177` `pubkeys.is_none()→Err`; `:2186/2220` `check_mk1_xpub_binding`. Read `:2178-2296`: surviving checks = stub-linkage `:2187/2236` (now template-id) + mk1 origin_fp↔privacy `:2193-2205` + ms1 parity tail `:2253-2294` (all still apply / ms1-independent). No other gate missed.

### Minors — CONFIRMED
M1/M2 production sites `synthesize_descriptor:258` (stub `:272`, n==1 csi `:290`, assert `:346`); `:180/216/192/228` dead. M4: `derive_stub_from_md1` (`mk-cli mod.rs:63-69`) needs the `!is_wallet_policy()` branch (compute_wallet_policy_id doesn't self-refuse on a keyless canonical template); both stale docs (`mod.rs:55-62`, `key_card.rs:25-30`) named. M5: `scripts/install.sh`; GUI `BUNDLE_FLAGS` `mnemonic.rs:190`/`:3778`.

### Drift check — no new C/I
C1 emit-gate (`n==1 && canonical_origin().is_some()`) and C2 restore-ingest predicate (`!is_wallet_policy() && n==1 && canonical_origin().is_some()`) consistent (restore adds `!is_wallet_policy()` because at ingest the md1 IS keyless). I4/D7 + account-normalization round-1-verified, undisturbed. §5 test gaps (Minor): add (a) keyless-multisig-refused at BOTH emit AND restore-ingest fall-through; (b) self-check passes for a template bundle.

### Verdict: GREEN
All three findings closed; four Minors folded; no new C/I; predicates internally consistent — plan-doc-ready for phase-1 single-sig. Residual Minors to fold: `self_check_bundle` name, §5 tests (keyless-multisig-refused-at-restore, template-bundle-self-check-passes), optional 1-of-1-multi edge note.
