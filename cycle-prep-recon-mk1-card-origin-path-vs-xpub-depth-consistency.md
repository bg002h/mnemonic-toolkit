# cycle-prep recon — 2026-05-30 — mk1-card-origin-path-vs-xpub-depth-consistency

**Origin/master SHA at recon time:** `a255060`
**Local branch:** `toolkit-mk-codec-0.4.0-repin` (2 ahead of origin/master — SPEC + superseded plan-doc commits; re-pin `mk-codec 0.3.1→0.4.0` applied uncommitted in working tree)
**Sync state:** up-to-date (origin/master clean on `mk-codec 0.3.1`)
**Untracked:** `.claude/`, prior recon docs, `design/agent-reports/toolkit-mk1-origin-path-spec-R0-review.md`, `feature-coverage-survey-2026-05-30.md`

Subject: **NOT a filed FOLLOWUP** — live discovery from the mk-codec 0.4.0 re-pin. Proposed slug `mk1-card-origin-path-vs-xpub-depth-consistency`. This recon verifies the root-cause citations + pins the exact failing-test census + resolves the depth-3-vs-depth-4 contradiction so the (R0-RED) SPEC can be re-grounded.

---

## WHAT

Re-pinning `mnemonic-toolkit` `mk-codec 0.3.1 → 0.4.0` surfaces **74 failing tests / 157 `XpubOriginPathMismatch` instances**. mk-codec 0.4.0 added an encode-guard rejecting any mk1 `KeyCard` whose `xpub.depth`/`child_number` disagree with `origin_path` (compact-73 drops + reconstructs depth/child from the path; the guard is correct by design — exact inverse of `reconstruct_xpub`). The toolkit (on guard-less 0.3.1) builds every card with `origin_path = the descriptor origin` while the card carries an xpub at a *different* depth. Proposed fix: a centralized `mk1_origin_path(xpub, descriptor_path)` helper deriving the mk1 path from the xpub's own depth/child; md1's `path_decl` keeps the full origin independently.

---

## Per-citation verification (vs branch source @ re-pin; `synthesize.rs`/`verify_bundle.rs` unchanged from origin/master `a255060`)

### 1. The 8 `mk_codec::KeyCard::new` sites — **ACCURATE (count/lines); M1 framing CORRECTED**
`synthesize.rs` `KeyCard::new` at **:137, :168, :221, :238, :386, :556, :773, :789** — confirmed; each passes a descriptor `path`/`c.path`/`s.path` + a slot `xpub`/`c.xpub`/`s.xpub`.
- **LIVE production emit paths:** `synthesize_descriptor` (:200 → cards :221/:238; called `bundle.rs:1414,1649`, `import_wallet.rs:1383`, `verify_bundle.rs:867`) and `synthesize_unified` (:676 → cards :773/:789; called `bundle.rs:377`, `verify_bundle.rs:374/463/566`).
- **Test-only helpers:** `synthesize_full` (:115, `#[allow(dead_code)]`), `synthesize_watch_only` (:153, `#[allow(dead_code)]`), `synthesize_multisig_full` (:299), `synthesize_multisig_watch_only` (:434) — all called ONLY from `#[cfg(test)]` mods (`verify_bundle.rs:2430/2583/2677/2688`, `synthesize.rs:1242+`). **Correction to R0 M1:** only `synthesize_full`+`_watch_only` carry the `dead_code` attr; `synthesize_multisig_*` do not, but are still test-only by call-graph. Applying the helper at all 8 sites is still correct (no-op for consistent cards); the load-bearing live sites are :221/:238/:773/:789.

### 2. verify-bundle cross-checks — **ACCURATE (C1 confirmed)**
`emit_watch_only_xpub_path_cross_check` (**:2024**) decodes the SUPPLIED mk1 (:2104/:2081) and at :2117-2126 fires `"mk1 xpub depth ({}) does not match md1 origin-path length ({})"` when `card.xpub.depth != md_path.components.len()`, + child check :2129-2146 — comparing against **md1's** origin path, INDEPENDENT of any synthesized expectation. `emit_full_path_parent_fingerprint_check` (**:2239**) exists. **After the fix, mk1 path = xpub-depth (e.g. 3) while md1 = full origin (e.g. 4) → these stderr cross-checks WILL false-positive on correct bundles.** C1 is real and the SPEC's "adjust if it false-positives" under-specifies it. (Note: the schema'd `result` verdict via `mk1_xpub_match`/`mk1_path_match` is symmetric — both sides re-synthesize through the helper — so only the stderr cross-checks regress.)

### 3. Tampered cross-check fixtures — **ACCURATE (C2 confirmed; precisely located)**
The two fixtures ARE the entire `3→1` + `3→2` census buckets: `cross_check_mk1_parent_fingerprint_mismatch_warns` (3→1) and `cross_check_mk1_depth_lt_md1_path_warns` (3→2) in `tests/cli_verify_bundle_watch_only.rs`. They build a deliberately-inconsistent card via `mk_codec::encode(&tampered).expect(...)` which on 0.4.0 panics at the guard (no public guard-bypass encoder). Must rebuild via the two-internally-consistent-cards-that-disagree pattern (precedent: `cross_check_mk1_child_number_ne_md1_last_warns`).

### 4. `bind_watch_only_singlesig:1062` empty-path default — **ACCURATE (I2 confirmed)**
`let path = path_from_decl(resolved, 0).unwrap_or(DerivationPath::master());` — `master()` = empty. Drives the `3→0` bucket (7 tests, all watch-only single-sig / zpub).

### 5. Foreign-format account-xpub export — **ACCURATE + EMPIRICALLY CONFIRMED**
Multisig-import cosigner xpubs are genuinely **depth-3 account keys** stored verbatim. Decoded `cli_bundle_import_json.rs`: `xpub6Cbhr/xpub6DBji/xpub6DB7H` all **depth=3, child=0'** (`m/48'/0'/0'`), paired with depth-4 `Derivation`. Card depth == source depth → **no truncation**. The architect model holds for the dominant class.

### 6. CENSUS (RE-GROUNDED) + the depth contradiction — **RESOLVED**
Full `cargo test -p mnemonic-toolkit --no-fail-fast` (branch, re-pinned): **74 failed**. Per-class (by `XpubOriginPathMismatch` signature `xpub_depth→path_depth`):

| bucket | #tests | nature |
|---|---|---|
| **3→4** | 33 | depth-3 account xpub + depth-4 BIP-48 path. **CLEAN** — helper truncates mk1 path to depth-3. BSMS, import-json multisig (all 6 formats), source-name-lift, p11 export-wallet, envelope. |
| **3→0** | 7 | depth-3 xpub + empty path (watch-only/zpub, `bind_watch_only_singlesig:1062`). I2. |
| **3→1, 3→2** | 2 | the C2 tampered cross-check fixtures (test-construction). |
| **4→3** | 20 | **HETEROGENEOUS — needs per-test audit (I1).** |
| **4→4** | 1 | `c3_multisig_sh_wsh_sortedmulti_converges_across_formats` (terminal-child disagree). |
| OTHER | 11 | collateral — failing test whose first panic block lacked a parsed signature (mostly p11 export-wallet + blob-network + name-override; likely downstream of the mismatch). Triage during impl. |

**Depth contradiction RESOLVED:** card.xpub.depth == the SOURCE xpub depth in every case decoded; there is NO silent truncation. The earlier "xpub6FQya decodes depth-4 but card is depth-3" was test-conflation — `xpub6FQya` (depth-4 `child 2'`) belongs to a *singlesig/4→3* coldcard test, NOT the 2-of-3 multisig test (which feeds depth-3 account xpubs). Different tests feed different-depth xpubs.

**I1 AUDIT — the 4→3 bucket is genuinely mixed; do NOT blanket-"extend the path":**
- **Genuine depth-4 leaf sources** (helper-extends-correctly): coldcard singlesig (`cli_import_wallet_coldcard.rs` `xpub6FQya` = depth-4 `m/48'/0'/0'/2'` leaf), several `coldcard_json_envelope_*`. Here the xpub IS the leaf; mk1 path should be depth-4.
- **ANOMALY needing investigation:** `tr_multi_a_nums_2of3_*` (sparrow taproot) — the only xpub decoded in `cli_import_wallet_sparrow_taproot.rs` is **depth-3 child-0'**, yet the card reports **xpub_depth:4 child:2'**. Either (a) the bundle is SEED-derived (full-mode) so the depth-4 card xpub is legitimately derived (helper-extends-correct), or (b) a different transformation. MUST read `tr_multi_a_nums_2of3_imports_successfully` before trusting "extend the path" for this sub-bucket.
- **verify_bundle helpers** (`helper_multisig_full_*`, `helper_multisig_missing_ms1_*`) — depth-4 xpub fed to a depth-3 `Bip84` `synthesize_full` → pure fixture inconsistency (the test reads only `ms1[0]`); fix the fixture (derive at the template path) OR let the helper normalize.

### 7. mk-codec 0.4.0 guard/reconstruct + on-wire path preservation — **ACCURATE**
Guard rejects `xpub.depth != count || xpub.child != last.unwrap_or(Normal{0})`. Intermediate path components are non-load-bearing for the reconstructed **Xpub** (only count→depth, last→child) BUT are written to the wire (`encode_path`) and restored verbatim by `decode_path` → they surface in `inspect --mk1` (`cmd/inspect.rs:222` `"origin_path: m/{}"`). So a fabricated `3→0` pad path (`[H0,H0,child]`) IS user-visible — the SPEC must stop calling it "purely informational" unqualified (I2).

---

## Cross-cutting observations

1. **The SPEC's "one clean helper" is necessary but NOT sufficient.** Confirmed-required beyond the helper: (a) **C1** redesign the two verify-bundle stderr cross-checks (compare xpub.depth/child against md1 origin *truncated/extended to the xpub's length*, and the parent-fp check against the xpub's actual depth); (b) **C2** rebuild 2 tampered fixtures via the consistent-cards-disagree pattern; (c) **I2** decide the `3→0` user-visible path (helper-pad-and-pin-a-test vs `bind_watch_only_singlesig` default-to-template-path — the latter ripples into md1 path_decl); (d) **I1** per-test audit of the 20-test `4→3` bucket (mixed: genuine depth-4 leaves + the `tr_multi` anomaly + 2 fixture bugs).
2. **The clean vs needs-investigation split:** 3→4 (33) + 3→0 (7) = **40 tests are the clean helper+default story**. 4→3 (20) + 4→4 (1) + OTHER (11) = **32 tests need per-test triage** before their snapshots can be blessed (R0 I1: "semantic round-trip" does NOT prove the xpub is the *correct* one).
3. **Snapshot-regen surface is large** — ~40+ pinned mk1-chunk/byte-exact/transcript/convergence assertions change. Each must be confirmed to *semantically* round-trip (decode → correct xpub), not blind-accepted.
4. **No new error variant; mirrors fallback-safe.** The re-pin compiles; `friendly.rs:124`/`error.rs:393` `_ =>` fallbacks already catch the variant (explicit arms are hygiene).
5. **md1 path semantics are OUT OF SCOPE** (R4). This cycle changes only the mk1 card path; md1 `path_decl` keeps the full descriptor origin. If a `4→3` audit reveals md1 *should* carry a depth-4 origin (under-built), that is a separate FOLLOWUP.

---

## Recommended brainstorm-session scope

- **Slug to file:** `mk1-card-origin-path-vs-xpub-depth-consistency` (toolkit `design/FOLLOWUPS.md`), companion to `mnemonic-key mk1-no-path-depth0-support`; subsumes `mk1-wif-bundle-depth0-invalid-card` + `mk1-depth-child-compensating-check-watch`.
- **SemVer:** toolkit **PATCH** 0.37.9 → 0.37.10. mk1 chunk *bytes* change for ~40+ flows (correctness fix to previously-wrong cards) but **no clap flag/subcommand/JSON-wire/output-shape change → no GUI schema-mirror, no manual lockstep.** Confirm no `--json` wire-shape field exposes the mk1 origin_path string (spot-check `bundle --json`/envelope shape).
- **Size:** larger than the original "150-250 LOC helper" estimate. Helper (~30 LOC) + 4 live call-site edits + the C1 cross-check redesign (~40-60 LOC, the riskiest) + C2 fixture rebuild + I2 default decision + the I1 4→3 audit + ~40 snapshot regens. Realistically a **5-7 task, multi-phase cycle** dominated by the cross-check redesign + per-test audit/regen, not the helper.
- **Phasing (supersede the existing SPEC after folding R0 + this recon):**
  1. Helper + 4 live call sites + unit tests (the 40 clean 3→4/3→0 tests — but 3→0 needs the I2 decision first).
  2. **C1** verify-bundle cross-check redesign + a pinned no-false-positive regression for a real 3→4 bundle.
  3. **I1** audit the 4→3/4→4 buckets per-test (esp. `tr_multi_a` — confirm seed-derived-depth-4 vs transformation); fix fixtures vs rely on helper accordingly.
  4. **C2** rebuild the 2 tampered fixtures.
  5. Snapshot/transcript/convergence regen with semantic-round-trip verification.
  6. Error-mirror arms + WIF round-trip regression + FOLLOWUPs + version + full-suite gate + end-of-cycle R0.
- **Mandatory R0** on the re-folded SPEC AND the plan-doc before any code. The current SPEC is R0-RED (2C/3I/3M); fold C1/C2/I1/I2/I3 + this recon's corrections, then re-dispatch.
- **Inter-cycle note:** mk-codec 0.4.0 + mk-cli 0.5.0 are already published & tagged; this toolkit cycle is the downstream consumer adoption. Master is clean on 0.3.1 — no regression while this cycle runs.
