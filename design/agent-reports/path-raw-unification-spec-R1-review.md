# R1 ARCHITECT REVIEW — `SPEC_path_raw_bracketed_bare_unification.md`

**Reviewed at:** `origin/master = dd7c228`. Opus feature-dev:code-reviewer, R1 of the mandatory pre-impl reviewer-loop. Persisted verbatim before fold. All findings independently grep/read-verified against live source. This round confirms the R0 folds and hunts for fold-introduced drift.

## R0 fold confirmation (all verified resolved)

- **C-1 (two missing producers + A3):** CONFIRMED RESOLVED. `bundle.rs:1382` and `verify_bundle.rs:830` are both genuine `CosignerKeyInfo { … path_raw, … }` literals. The A3 trace is accurate: at `bundle.rs:1322-1333` the `Some(p) => (parsed, p.value.clone())` arm stores raw user bytes in `path_raw`; re-wrapped at `bundle.rs:1442` and flows through `emit_unified` → C5 (`:755`) / C6 (`:767`) / C7 (`:1000-1003`). The `None` arm uses `anno_path.to_string()` (canonical) — A3 correctly distinguishes the arms; does NOT overstate. "No test pins it" holds.
- **C-2 (C4 rewrite keeps fallback):** CONFIRMED RESOLVED. `key_origin_str` (`pipeline.rs:33-44`) has the path-bearing fallback branch; both callers pass a real fallback (`pipeline.rs:75`, `bip388.rs:73`). Rewritten C4 + T5(e) preserve it. Non-fallback case byte-compatible for canonical/band-aid input (see I-A for the non-canonical caveat).
- **I-1:** CONFIRMED — T1 explicitly invokes `bundle --import-json`.
- **I-2:** CONFIRMED — `json_envelope.rs:350-357` builds bracketed; A1 now says bracketed→bare.
- **I-3:** CONFIRMED — `origin_path_from_bracket` `None => "m"` arm; reachability argument sound.
- **I-4:** CONFIRMED — `#[cfg(test)] mod tests` begins `synthesize.rs:778`; `:1141`/`:1262`/`:1375` all inside.
- **M-1..M-4:** all folded sensibly.
- **T9 Display value:** CONFIRMED — typed `DerivationPath` folds `h`→`'` (pinning test `:1931`), so `origin_path_bare()` on `48h/0h/0h/2h` → `m/48'/0'/0'/2'`.

## CRITICAL
None.

## IMPORTANT

### I-A. (FOLD-INTRODUCED) A3 documents only the `bundle --json` surface; the C1/C2/C3/C4 swaps ship the IDENTICAL raw→canonical wire-value change on the `export-wallet` direct surface, undeclared.

**Files:** `wallet_export/electrum.rs:163-164` (C1), `wallet_export/coldcard.rs:321` (C2), `wallet_export/sparrow.rs:130` (C3), `wallet_export/pipeline.rs:33-44` + `wallet_export/bip388.rs:73` (C4).

The fold added A3 for the **`bundle --json` origin_path** surface (C5/C6/C7). But the same root cause — `resolve_slots`' `Xpub` branch storing raw `p.value.clone()` at `bundle.rs:547` — feeds the **`export-wallet` consumers C1/C2/C3/C4** for direct export, and A3 does not mention them.

For `export-wallet … --slot @N.path=48h/0h/0h/2h`:
- `key_origin_str` (C4) only strips `m/`; does NOT fold `h`→`'`. Today emits `[fp/48h/0h/0h/2h]`; after → `[fp/48'/0'/0'/2']`.
- coldcard `normalize_path` (`coldcard.rs:308-317`) only prepends `m/`; today `m/48h/...`; after → `m/48'/...`.
- sparrow `normalize_derivation` multisig branch (`sparrow.rs:243-251`) preserves `path_raw` verbatim. Same shift.
- electrum (C1) emits `path_raw.clone()` verbatim (no `m/` for raw direct input); after → both prepends `m/` AND folds `h`→`'`.

Currently untested (every existing `export-wallet` test passes canonical `--slot @N.path=m/48'/0'/0'/2'`; suite stays green) — which is why it would ship silently. Same class as A3 on a user-facing surface (+ bip388 `keys_info`).

**Fix:** Broaden A3 (or add A4) to state descriptor-spelling canonicalization (`h`→`'`, `m/`-prefix, numeric→hardened) ALSO applies to `export-wallet`'s direct consumers C1/C2/C3/C4 when a user supplies non-canonical `--slot @N.path=`. Add one integration cell (parallel to T9). Note in §10 it rides the same CHANGELOG note. Functionally benign — key material unchanged, output canonical and accepted by miniscript/wallets — but a declared-vs-undeclared parity gap.

## MINOR

### M-A. §4 cosigner-push-literal line numbers point at the destructure/push line, not the `path_raw:` field-init line (off by 2-4).
Actual `path_raw,` field-init lines: `specter.rs:255`, `sparrow.rs:437`, `bsms.rs:259`, `electrum.rs:401`, `bitcoin_core.rs:298`, `coldcard.rs:340`, `json_envelope.rs:362` (SPEC cites the `let (…) =` / `out.push(…)` line). No functional impact (field-deletion forcing function); tighten for the implementer. `coldcard_multisig.rs:489` + `bundle.rs:505/581/630/674/678/1382/1442` + `verify_bundle.rs:830` cited exactly right.

### M-B. §4(b) bracketed-builder lines verified present/correct (no-action confirmation).
All six builder lines (`specter.rs:387`, `sparrow.rs:607`, `coldcard.rs:534`, `electrum.rs:946`, `bitcoin_core.rs:438`, `bsms.rs:389`) verified; `coldcard_multisig.rs:422` correctly excluded (local `ResolvedCosigner`, §7).

## Internal-consistency cross-check
No contradictions. §2.1 pillar-4 ↔ §3 `.to_lowercase()` ↔ M-1 consistent. §3 `bracketed_origin()` (returns `[fp]` for default) ↔ §5 C4 (only calls `bracketed_origin()` on the non-empty arm, separate `fallback_path` build for empty) mutually consistent. §8 T9 ↔ `origin_path_bare()` + `h`→`'` fold consistent. §9 ordering sound; Phase 2 smoke-check should add a non-canonical `@N.path=` export per I-A.

## VERDICT: RED (0C / 1I)
Counts: **Critical 0, Important 1, Minor 2.** Both R0 Criticals correctly folded and verified. The one new finding is fold-introduced (the parallel `export-wallet` raw→canonical change is undeclared/untested) — Important, not Critical (functionally benign, suite stays green). Fold I-A → re-dispatch per after-every-fold discipline.
