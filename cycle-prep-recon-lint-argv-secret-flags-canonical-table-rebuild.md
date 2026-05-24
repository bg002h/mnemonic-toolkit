# cycle-prep recon — 2026-05-24 — lint-argv-secret-flags-canonical-table-rebuild-from-clap

**Origin/master SHA at recon time:** `5fbed42`
**Local branch:** `master`
**Sync state:** `up-to-date (0 ahead / 0 behind)`
**Untracked:** `.claude/`

Slug verified: `lint-argv-secret-flags-canonical-table-rebuild-from-clap`. **Citations all ACCURATE, BUT the FOLLOWUP structurally UNDERSTATES the gap (3 named flags → actually ~16 missing flag-rows) AND its "rebuild from clap" framing is underspecified — clap/gui-schema's flag-level `secret` bit and the table's argv-ROUTE model are different axes.**

---

## Per-slug verification
### `lint-argv-secret-flags-canonical-table-rebuild-from-clap`
- **WHAT:** the hand-curated `CANONICAL_FLAG_ROWS` argv-leakage audit froze at v0.13.0 and silently omits post-v0.13.0 secret-bearing argv flags; rebuild it (prefer a clap-derived closure) so it's a leading gate.
- **Citations:**
  - `tests/lint_argv_secret_flags.rs` `CANONICAL_FLAG_ROWS` — **ACCURATE** (`:47`, `const CANONICAL_FLAG_ROWS: &[FlagRow] = &[`).
  - `assert_eq!(CANONICAL_FLAG_ROWS.len(), 28)` — **ACCURATE** (`:212-219`, in test `canonical_list_has_twenty_eight_rows` `:204`).
  - doc "New secret-bearing flag-rows must be added to `CANONICAL_FLAG_ROWS`" — **ACCURATE** (`:11-13`).
  - "froze at v0.13.0" — **ACCURATE.** Rows end at slip39 (v0.13.0); count comment (`:205-211`) stops at "v0.13.0 slip39 +5 = 28". No nostr/silent-payment/electrum-decrypt/import-wallet/xpub-search/inspect/repair/seedqr rows.
  - "NO closure deriving rows from clap" — **ACCURATE + BY DESIGN.** `:20-24`: "the canonical list lives inline rather than being derived from clap so the lint catches accidental flag removals AND ensures the SPEC table remains the single source of truth." The inline-not-clap choice was DELIBERATE — a rebuild reverses that original design rationale (must preserve removal-detection).
  - "fails no test" — **ACCURATE.** Only checks `len()==28` + per-row source-grep evidence; no enumeration of clap to find unlisted secret flags.
  - "enforced projections (`flag_is_secret` + `secret_in_argv_warning`) correct + complete" — **ACCURATE** (the real protection; the table is a secondary audit).
- **SCOPE CORRECTION (structural):** the FOLLOWUP names 3 missing flags. Live `gui-schema` shows **25 value-bearing secret flags** (kind=text); diffing against the 28 rows, **~16 flag-NAME rows are missing**, not 3: `electrum-decrypt --decrypt-password`, `import-wallet --decrypt-password`, `import-wallet --ms1`, `inspect --ms1`, `nostr --secret`, `repair --ms1`, `seedqr-decode --digits`, `silent-payment --secret`, `silent-payment --passphrase`, `verify-bundle --ms1`, `xpub-search-{path,account,passphrase}-of-xpub --ms1` (×3), `xpub-search-{path,account,passphrase}-of-xpub --passphrase` (×3).
- **MODEL-MISMATCH (the load-bearing finding):** the table and gui-schema enumerate DIFFERENT axes:
  - `CANONICAL_FLAG_ROWS` is a per-argv-ROUTE enumeration — it decomposes `--from <node>=` into 8 convert rows + 2 derive-child rows + …, and `--slot @N.<subkey>=` into 4 bundle rows, because the secret lives in the VALUE/node, not the flag name. gui-schema does NOT mark `--from`/`--slot` as `secret` (their flag NAMES aren't secret), so a naive gui-schema mirror would **DROP** the entire `--from`/`--slot` node-route coverage (≈15 of the 28 rows).
  - gui-schema's per-flag `secret` bit captures intrinsically-secret-NAMED flags (`--ms1`/`--secret`/`--decrypt-password`/`--digits`/`--share`/`--passphrase`), which is the set the table is MISSING.
  - So a correct closure is **flag_is_secret(flag-names) ∪ {`--from <node>=` : node ∈ secret_taxonomy::SECRET_NODE_TYPES} ∪ {`--slot @N.<subkey>=` : subkey ∈ SECRET_SLOT_SUBKEYS}** — NOT a plain gui-schema mirror. The `secret_taxonomy` public constants (the same ones the GUI consumes) are the missing input the FOLLOWUP didn't mention.
- **In-file doc drift (DRIFTED):** header `:5` ("20 flag-rows enumerated") + struct doc `:44` ("Canonical list of 20 … flag-rows") say **20**, but the list + assert are **28** — stale since the v0.11/0.12/0.13 additions bumped the count but not the prose. Fix in the same cycle.
- **Action for brainstorm spec:** correct the FOLLOWUP's "3 flags / rebuild from clap" framing to "~16 missing rows + a two-axis closure (flag_is_secret ∪ SECRET_NODE_TYPES ∪ SECRET_SLOT_SUBKEYS)". Decide: closure source = in-process `CommandFactory` walk (like `gui-schema`) for flag-level secret + `secret_taxonomy` constants for node/subkey routes. Reconcile the evidence-anchor model (the current per-row source-grep `evidence` may not survive a closure — a closure needs to assert each enumerated secret route has a paired stdin/`=-`, which is per-subcommand, not per-hand-written-row). Cite source SHA `5fbed42`.

---

## Cross-cutting observations
1. **Structural scope understatement (5×).** The FOLLOWUP's "nostr + silent-payment (3 flags)" is the tip — the real gap is ~16 flag-NAME rows across 8 subcommands (electrum-decrypt, import-wallet, inspect, repair, seedqr-decode, verify-bundle, xpub-search ×3 modes, nostr, silent-payment). Mirrors the v0.28.7 "Slug-3 scope 5× larger" pattern; a piecemeal "backfill the 3 named" would still leave 13 unlisted.
2. **Two-axis model mismatch (not a simple mirror).** clap/gui-schema flag-level `secret` ≠ the table's per-argv-route model; a clean closure needs `secret_taxonomy::{SECRET_NODE_TYPES, SECRET_SLOT_SUBKEYS}` for the `--from`/`--slot` routes the flag-bit can't see. This is the real design work — verify the public API of `secret_taxonomy` exists + is enumerable at test time.
3. **Original design intent.** The inline list was DELIBERATELY not-clap-derived (`:20-24`) to catch flag REMOVALS + keep the SPEC table authoritative. A closure must preserve removal-detection (e.g. assert the closure set ⊇ a frozen floor, or keep a SPEC cross-check) — don't silently lose that property.
4. **In-file "20" doc drift** (`:5`, `:44`) — pre-existing, fix opportunistically.
5. **No DRIFTED-by-N line-number drift** on the cited symbols (CANONICAL_FLAG_ROWS @:47, assert @:212 — both exact). The drift is semantic (scope + model), not positional.

---

## Recommended brainstorm-session scope
- **One PATCH cycle (test/lint-only) → v0.36.2.** No CLI surface change ⇒ **NO GUI schema_mirror lockstep, NO manual lockstep, NO sibling-codec companions.** Pure toolkit test hardening.
- **Sizing:** medium. The closure design is the work, not LOC. Likely a new test that (a) walks `CommandFactory`/`gui-schema` for flag-level `flag_is_secret` flags, (b) expands `--from`/`--slot` via `secret_taxonomy::{SECRET_NODE_TYPES, SECRET_SLOT_SUBKEYS}`, (c) asserts each enumerated secret-argv route has a paired stdin/`=-` evidence anchor, and (d) flags any secret route lacking one. Plus retire/repurpose the hand-curated 28-row table (keep as a "floor" for removal-detection, or fully replace). ~150–250 LOC of test + helper; touches `tests/lint_argv_secret_flags.rs` only (+ maybe a small `secret_taxonomy` re-export if the constants aren't already `pub`).
- **R0 must resolve:** (1) closure source + exact two-axis enumeration; (2) evidence-anchor model under the closure (per-subcommand stdin-route proof vs per-row grep); (3) preserve the original removal-detection property; (4) what to do with the 16 currently-missing routes that may lack a wired stdin alternative — **does each missing flag actually HAVE a `*-stdin`/`=-` route?** (recon shows the `-stdin` toggles exist for decrypt-password/secret/passphrase, and `--ms1` flows through the positional/`-` stdin path — but this MUST be verified per-route, because if any secret-argv flag lacks a stdin alternative, that is a REAL secret-hygiene gap, not just an audit-table gap, and would escalate the cycle's severity).
- **Severity watch:** if R0/impl finds a secret-argv flag with NO stdin/`=-` alternative (e.g. an `--ms1`-only subcommand), that promotes from "audit hygiene" to a real argv-leak fix (still PATCH, but adds a `*-stdin` flag → then it WOULD need GUI/manual lockstep). Recon could not fully rule this out for all 16 — the brainstorm's first task is the per-route stdin-coverage audit.
