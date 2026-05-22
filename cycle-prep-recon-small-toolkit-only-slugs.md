# cycle-prep recon — 2026-05-22 — small/toolkit-only slug bucket (7 slugs)

**Origin/master SHA at recon time:** `1d6436d`
**Local branch:** `master`
**Sync state:** `up-to-date (0 ahead / 0 behind)`
**Untracked:** `.claude/` only

Slug(s) verified: `hex-dep-unused`, `error-rs-exit-code-arm-fragmentation-post-sort`, `convert-minikey-stdout-redaction`, `wallet-import-signet-regtest-disambiguation`, `dead-inner-guard-bundle-watch-only`, `watch-only-stderr-warning-suborder`, `xpub-search-descriptor-md1-detection-bech32-validate`. **Expectation: heavy drift + several STALE** — all 7 predate the v0.27→v0.34 cycles; the recon found 3 stale (premise false / symbol deleted), 1 mischaracterized, 1 no-op decision, and only 2 genuinely actionable.

---

## Per-slug verification

### `hex-dep-unused`
- **WHAT (from FOLLOWUPS.md):** `hex = "0.4"` declared but unused in non-test source; remove (pending `feedback_dont_drop_reserved_deps` confirmation).
- **Citations:**
  - `crates/mnemonic-toolkit/Cargo.toml:27` `hex = "0.4"` — **DRIFTED-by-13** → now at `Cargo.toml:40`.
  - Claim "No `use hex` statement in any source module / unused in non-test source" — **STRUCTURALLY-WRONG (premise now false).** `hex::` is used in NON-test source at `bundle.rs:565,1286`, `convert.rs:1132,1164,1415,1520`, `import_wallet.rs:2146,2183,2338`, and `nostr.rs:50` (v0.34.0). The dep is heavily live; removing it breaks the build.
- **Action for brainstorm spec:** **CLOSE as resolved-by-obsolescence** — the dep is legitimately used (activated across v0.26 import-wallet / convert / v0.34.0 nostr). No code change; mark `Status: resolved` with a note. Cite SHA `1d6436d`.

### `error-rs-exit-code-arm-fragmentation-post-sort`
- **WHAT:** record/decide whether the post-alphabetical-sort single-variant `exit_code` arms should be re-grouped (recommended (a): keep as-is).
- **Citations:**
  - `error.rs` `exit_code` match "post-Cycle-4: L428-473" — **DRIFTED** → `pub fn exit_code` now at `error.rs:438`.
  - "44 single-variant arms" — **DRIFTED-by-1** → 45 now (v0.34.0 added `NostrKeyParse => 1`).
- **Action for brainstorm spec:** No-op decision slug. The entry's own recommendation is option (a) "keep single-variant arms forever" (alphabetical lock > grouping). **CLOSE with decision (a)** (no code change) OR leave open as a documented stance. Cite SHA `1d6436d`.

### `convert-minikey-stdout-redaction`  ← ACTIONABLE
- **WHAT:** widen `NodeType::is_secret_bearing` (or switch two call sites to `is_argv_secret_bearing`) so Casascius MiniKey is redacted in the `--from` echo + fires the secret-on-stdout warning.
- **Citations:**
  - `convert.rs::is_secret_bearing (~85-96)` — **ACCURATE-ish** → `convert.rs:94`.
  - `is_argv_secret_bearing (~98-110)` — **DRIFTED** → `convert.rs:117-118` (`is_secret_bearing() || matches!(self, Self::MiniKey)`, confirming MiniKey is excluded from the narrower predicate).
  - `from_value` redaction site `convert.rs:769` — **DRIFTED** → `convert.rs:1042` (`let from_value = if primary.node.is_secret_bearing()`).
  - secret-on-stdout site `convert.rs:796` — **DRIFTED** → `convert.rs:1069` (`if outputs.iter().any(|(n,_)| n.is_secret_bearing())`).
  - `tests/cli_convert_minikey.rs` (currently no advisory expected) — verify at impl time.
- **Action for brainstorm spec:** Genuinely actionable, small, toolkit-only. Pick the fix (widen `is_secret_bearing` to include MiniKey — simplest — OR switch the two `:1042`/`:1069` sites to `is_argv_secret_bearing`). Note: changes user-facing output (adds advisory/redaction for MiniKey) → entrains `tests/cli_convert_minikey.rs` fixture updates. SemVer PATCH; **no flag change → no GUI schema-mirror / manual lockstep.** Cite SHA `1d6436d`. (NB: `convert.rs:1602` already carries a doc comment referencing this widening.)

### `wallet-import-signet-regtest-disambiguation`  ← NOT small (design/scope + lockstep)
- **WHAT:** coin-type-1 collapses signet/regtest→testnet; v0.27+ may add `--network signet|regtest` override on `import-wallet` OR an origin-path disambiguator. "User-direction needed."
- **Citations:**
  - `wallet_import/bsms.rs:14-15` doc comment — **DRIFTED** → the FOLLOWUP citation is at `bsms.rs:26` now (`…both are imported as testnet (FOLLOWUP wallet-import-signet-regtest-disambiguation)`).
  - `design/SPEC_wallet_import_v0_26_0.md §4.2 step 8` — not re-verified (SPEC doc); the normative testnet-collapse text.
- **Action for brainstorm spec:** **MISCHARACTERIZED as small/toolkit-only.** The proposed fix (a) adds a NEW `--network` flag to `import-wallet` → **MANDATORY GUI `schema_mirror` lockstep + manual `41-mnemonic.md` lockstep** (net-new flag NAME). Needs a user-direction call (add flag vs origin-path hint vs leave-documented). Defer to its own MINOR cycle with paired GUI; do NOT bundle into a toolkit-only PATCH. Cite SHA `1d6436d`.

### `dead-inner-guard-bundle-watch-only`
- **WHAT:** a redundant `--xpub`-needs-`--master-fingerprint` `BadInput` guard inside `bundle_watch_only`, unreachable behind the mode-violation pre-check.
- **Citations:**
  - `crates/mnemonic-toolkit/src/cmd/bundle.rs:200` (`bundle_watch_only`) + pre-check `bundle.rs:93` — **STRUCTURALLY-WRONG.** `grep 'fn bundle_watch_only'` returns nothing: the function was **DELETED in `bc59ee3`** ("v0.4.2 Phase M.3: delete legacy CLI dispatch helpers (~990 lines)"). The redundant guard went with it. The FOLLOWUP itself predicted "v0.2 will refactor mode dispatch and naturally clean this up."
- **Action for brainstorm spec:** **CLOSE as resolved-by-refactor (`bc59ee3`).** No code change. Cite SHA `1d6436d`.

### `watch-only-stderr-warning-suborder`
- **WHAT:** depth advisory emitted before the account-index hazard in the watch-only path; SPEC §5.2 doesn't pin the sub-order; fixtures don't cover it.
- **Citations:**
  - `crates/mnemonic-toolkit/src/cmd/bundle.rs::bundle_watch_only` — **STRUCTURALLY-WRONG.** Same deleted function (`bc59ee3`). The watch-only path is now folded into `bundle.rs::run` with mode detection via `bundle.any_secret_bearing()` (`bundle.rs:678`); the specific old two-advisory sub-order site no longer exists in that form.
- **Action for brainstorm spec:** Re-assess against the current `bundle.rs::run` watch-only path: determine whether the depth-advisory-vs-account-index-hazard sub-order concern still exists post-refactor. **Likely CLOSE as moot/resolved-by-refactor;** if a real ordering ambiguity persists in the new code, re-file with current citations. Cite SHA `1d6436d`.

### `xpub-search-descriptor-md1-detection-bech32-validate`  ← ACTIONABLE
- **WHAT:** tighten the md1 tie-break from `starts_with("md1")` to real bech32 validation.
- **Citations:**
  - `crates/mnemonic-toolkit/src/cmd/xpub_search/descriptor_intake.rs:167` — **ACCURATE.** Confirmed verbatim: `if !tokens.is_empty() && tokens.iter().all(|t| t.starts_with("md1")) {` at line `167`.
- **Action for brainstorm spec:** Genuinely actionable, small, toolkit-only. Tighten to `t.starts_with("md1") && bech32::decode(t).is_ok()` (or a shape-only check). The entry notes it is "defensible to leave as-is" (false-positives surface as typed `md_codec::chunk::reassemble` errors, not silent misroutes) — so this is hardening, not a bug fix. SemVer PATCH; **no flag change → no lockstep.** Add a unit cell for `md1xxx`-garbage → no-misroute. Cite SHA `1d6436d`.

---

## Cross-cutting observations
1. **3 of 7 are STALE** — strong "statuses decay" signal: `hex-dep-unused` (premise false — dep used in 4+ non-test modules incl. v0.34.0 nostr); `dead-inner-guard-bundle-watch-only` + `watch-only-stderr-warning-suborder` both cite `bundle_watch_only`, **deleted in `bc59ee3` (v0.4.2 Phase M.3)**. These should be closed in the registry, not implemented.
2. **`wallet-import-signet-regtest-disambiguation` is mischaracterized as small/toolkit-only** — its proposed fix adds a `--network` flag to `import-wallet` (NEW flag NAME → mandatory GUI schema-mirror + manual lockstep). It is a MINOR cycle with paired GUI, needing user-direction. Pulled out of the "small" bucket.
3. **`error-rs-exit-code-arm-fragmentation-post-sort` is a no-op readability decision** (recommended (a) = keep single-variant); close-with-decision or leave as a documented stance.
4. **Pervasive line-number drift** — every citation predates v0.27→v0.34; line numbers moved (Cargo.toml 27→40, error.rs 428→438, convert.rs 769→1042 / 796→1069, bsms.rs 14→26). Only `xpub_search/descriptor_intake.rs:167` is byte-on-line ACCURATE.
5. **Genuinely actionable small toolkit-only set is just 2:** `convert-minikey-stdout-redaction` + `xpub-search-descriptor-md1-detection-bech32-validate`. Both PATCH, no lockstep.
6. Sync clean; no DRIFTED-by-N affecting correctness of the two actionable slugs.

---

## Recommended brainstorm-session scope

**(A) Registry-hygiene close (docs-only, no code, do first):** mark `hex-dep-unused` resolved-by-obsolescence, `dead-inner-guard-bundle-watch-only` resolved-by-refactor (`bc59ee3`), `watch-only-stderr-warning-suborder` moot-pending-reassessment (or resolved-by-refactor), `error-rs-exit-code-arm-fragmentation-post-sort` closed-with-decision-(a). One `docs(followups):` commit. No SemVer impact.

**(B) Actionable PATCH cycle → v0.34.2 (toolkit-only, NO lockstep):** `convert-minikey-stdout-redaction` (~5-10 LOC: widen `is_secret_bearing` to include MiniKey OR switch the two `convert.rs:1042/1069` sites to `is_argv_secret_bearing`; + `tests/cli_convert_minikey.rs` advisory fixture) + `xpub-search-descriptor-md1-detection-bech32-validate` (~3-5 LOC: bech32-validate the md1 tie-break at `descriptor_intake.rs:167` + 1 unit cell). Both PATCH, no flag change → no GUI/manual lockstep. Independent; either order. **Mandatory opus R0 on the plan-doc before code (CLAUDE.md).**

**(C) Defer to its own MINOR cycle (needs user-direction + GUI/manual lockstep):** `wallet-import-signet-regtest-disambiguation` — NOT small. Decide flag-vs-hint-vs-document first.
