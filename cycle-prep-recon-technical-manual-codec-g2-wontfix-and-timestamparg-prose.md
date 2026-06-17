# cycle-prep recon — 2026-06-07 — technical-manual-codec-g2-not-enforceable-in-single-repo-ci + technical-manual-glossary-timestamparg-default-prose-stale

**Origin/master SHA at recon time:** `d7ca67a`
**Local branch:** `master`
**Sync state:** up-to-date (0 ahead / 0 behind)
**Untracked:** only recon scaffolding (`cycle-prep-recon-*`, `cycle-b-*`, `feature-coverage-survey-*`, `.claude/`, `CONTINUITY.md`).

Slug(s) verified: both filed THIS session at `d7ca67a`. **Slug 1 = ACCURATE, accepted-wontfix (no action). Slug 2 = real staleness CONFIRMED, but the FOLLOWUP's own stated fix is STRUCTURALLY-WRONG (it mis-identifies the type) — the actual fix is narrower.**

---

## Per-slug verification

### `technical-manual-codec-g2-not-enforceable-in-single-repo-ci`
- **WHAT:** accepted-wontfix tracker — bare-CI `make lint` (toolkit-only checkout) cannot enforce codec-symbol G2 (needs absent sibling source). Full codec-G2 = local `make lint` with siblings present.
- **Citations:**
  - `.github/workflows/technical-manual.yml` runs single-repo (toolkit-only checkout) — **ACCURATE**: `actions/checkout@v4` with no `path:` (`:48`); the only crates path in the trigger is `crates/mnemonic-toolkit/**` (`:26,:35`), no sibling-repo checkout.
  - `symbol-ref-check.py` G2 skips codec refs when siblings absent — **ACCURATE**: `skip:absent-sibling` arm (`:173`, `not auth and not qualified and ABSENT`) + qualified `skip:<repo>` arm (`:123`) + authoritative `skip:<auth>` (`:135`).
  - "~59 non-auth + codec-authoritative chapters" skip — **ACCURATE** (matches the empirical probe last cycle: 59 non-auth codec refs; 427 total skipped incl. codec-authoritative chapters 21/22/23/51/52/53).
  - Wontfix rationale (multi-repo rejected; basename manifest = file-existence not symbol-existence; crates.io download couples to *published* codec versions lagging dev symbols) — **ACCURATE** design rationale (carried from the architect blueprint).
- **Action for brainstorm spec:** **NONE — this is a deliberate, documented accepted-wontfix tracker.** There is nothing to implement. It stays `open` as a discoverability marker (lifted out of the resolved parent entry on purpose). Revisit ONLY if codec-citation drift actually recurs in shipped docs. No cycle. Cite SHA `d7ca67a`.

### `technical-manual-glossary-timestamparg-default-prose-stale`
- **WHAT:** the `61-glossary.md` `TimestampArg` entry describes the pre-v0.47.3 default `now`; source default is now `0` (genesis rescan).
- **Citations:**
  - `61-glossary.md:383-385` `TimestampArg` entry says "default `now`" — **the staleness is ACCURATE/CONFIRMED.** Source: `#[arg(long, default_value = "0", value_parser = parse_timestamp)]` (`cmd/export_wallet.rs:212`); doc-comment `cmd/export_wallet.rs:210-211` "`0` (default; rescan from genesis…), `now`, or unix seconds". So the glossary's "default `now`" is wrong → should be "default `0` (genesis rescan)".
  - **FOLLOWUP body sub-claim "the enum is `TimestampArgValue` (a `wallet_export/mod.rs` type), not `TimestampArg`" — STRUCTURALLY-WRONG (self-mis-cite, caught by cycle-prep):**
    - `TimestampArg` IS still the `pub(crate) enum` with `Now`/`Unix(i64)` — at `wallet_export/mod.rs:144` (NOT removed). The glossary entry documents this enum CORRECTLY (`Now` → JSON `"now"`, `Unix(i64)` → JSON integer; both still true via `TimestampArg::to_json`).
    - `TimestampArgValue` is a separate **newtype struct** `pub struct TimestampArgValue(pub TimestampArg)` at `cmd/export_wallet.rs:297` (NOT `wallet_export/mod.rs`) — it is the clap **field** type (`ExportWalletArgs::timestamp: TimestampArgValue`, `:213`) that wraps the enum for parsing.
    - **So the glossary's `crates/mnemonic-toolkit/src/wallet_export/mod.rs::TimestampArg` citation is ACCURATE and must NOT be renamed to `TimestampArgValue`** (that would point at the wrong file AND mislabel a struct as the enum, and could break G2 — `TimestampArgValue` does not exist in `wallet_export/mod.rs`).
- **Action for brainstorm spec:** **NARROW fix — change only "default `now`" → "default `0` (rescan from genesis; `now` and unix seconds also accepted)" at `61-glossary.md:385`.** Do NOT touch the `TimestampArg` enum citation (accurate). Optionally (additive, not a correction) note the clap field is the `TimestampArgValue(TimestampArg)` newtype wrapper — but the entry is *about the enum*, so minimal fix is the default phrase only. Pure prose claim; **NOT gated by symbol-ref-check** (it pins locations, not claims) → re-verify by hand. Cite SHA `d7ca67a`.

---

## Cross-cutting observations
1. **Self-mis-cite in a freshly-filed FOLLOWUP, AGAIN (4th this session).** I filed `…-timestamparg-…` last cycle (d7ca67a) and its "fix" body wrongly said to swap `TimestampArg`→`TimestampArgValue`. Source shows `TimestampArg` is the correct enum to cite. This is the recurring class ([[feedback_*]] / MEMORY: "never trust a FOLLOWUP's stated correct-value, verify source"; "3rd same-day mis-cite"). The cycle-prep gate is exactly what catches it — had the fix been applied blind, it would have introduced a wrong citation (and possibly a G2 break, since `TimestampArgValue` is not in `wallet_export/mod.rs`).
2. **Slug 1 is a no-op tracker.** Cycle-prepping a wontfix confirms its citations are sound; the correct outcome is "do nothing, keep it open." No SPEC, no R0 needed unless we ever decide to actually attempt codec-G2-in-CI.
3. **Slug 2 is pure prose** → ungated by the symbol-ref gate (which pins locations, not claims) → this is the api-harvest *claim*-class, the very reason it was scoped OUT of the location-pinning cycles. Hand-verification is the only check.
4. No locksteps for either: no clap-flag / CLI / codec surface; no GUI `schema_mirror`, no manual-mirror, no sibling-codec companion.

---

## Recommended brainstorm-session scope
- **Slug 1:** **no cycle.** Accepted-wontfix tracker, citations verified accurate; leave `open` as a discoverability marker. (If you'd rather not carry an open wontfix, the alternative is to mark it `resolved (wontfix)` — a bookkeeping choice, not work.)
- **Slug 2:** a **trivial 1-line prose correction** in `61-glossary.md:385` (default `now` → `0`/genesis-rescan), with the cycle-prep correction that the `TimestampArg` enum citation is ACCURATE and stays. Docs-only, **no version bump / no tag**. SemVer N/A. Sizing: 1 line. Mandatory R0 still applies (project standard) but a 1-line prose fix is a single-round GREEN in practice — or **bundle it into the Node.js workflow-bump cycle** (both are docs/CI-adjacent no-bump housekeeping) so one R0 + one commit covers both. Ordering: independent; do whenever.
