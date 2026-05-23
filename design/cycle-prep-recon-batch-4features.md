# cycle-prep recon — 2026-05-22 — minikey-pair / matrix / signet / argv-overwrite

**Origin/master SHA at recon time:** `f4d553e`
**Local branch:** `master`
**Sync state:** up-to-date (HEAD == origin/master)
**Untracked:** `.claude/`

Slug(s) verified: `convert-minikey-stdout-redaction`, `secret-taxonomy-argv-superset-promotion`, `wallet-import-format-mismatch-matrix-completion-discovered-gaps`, `wallet-import-signet-regtest-disambiguation`, `argv-overwrite-after-parse`. Four independent subsystems → four series cycles. Citations mostly ACCURATE-in-substance with line drift; the matrix audit confirms exactly 10 missing arms.

---

## Per-slug verification

### `convert-minikey-stdout-redaction` — ACCURATE (intent), DRIFTED cites
- **WHAT:** MiniKey (Casascius private-key encoding) is excluded from the convert.rs stdout-redaction + secret-on-stdout pathways (both gated by the *narrow* `is_secret_bearing`). Fix: switch the two call sites to the *wider* `is_argv_secret_bearing` (which already includes MiniKey).
- **Citations:**
  - `convert.rs:85-96` `is_secret_bearing` — **DRIFTED:** now `fn is_secret_bearing` at **`convert.rs:94`**.
  - `convert.rs:98-110` `is_argv_secret_bearing` — **DRIFTED:** now **`convert.rs:117-119`**, body `self.is_secret_bearing() || matches!(self, Self::MiniKey)`. Confirms MiniKey ∈ wide set, ∉ narrow set.
  - `convert.rs:769` (from_value redaction) — **DRIFTED → `convert.rs:1042`** (`let from_value = if primary.node.is_secret_bearing()`).
  - `convert.rs:796` (secret-on-stdout warning) — **DRIFTED → `convert.rs:1069`** (`if outputs.iter().any(|(n, _)| n.is_secret_bearing())`, the §7 warning at 1068-1069).
  - Doc comment `convert.rs:108-116` (the `is_argv_secret_bearing` doc-comment; line 117 is the `fn` signature) explicitly states the narrow predicate is preserved *because* it gates this machinery "whose MiniKey behavior is intentionally [deferred to this FOLLOWUP]." — **ACCURATE.** (Note the comment's own internal `convert.rs:769, 796` cite is the code's OWN stale snapshot — the live call sites are 1042/1069.)
- **Constraint:** **MUST switch the call sites to `is_argv_secret_bearing`, NOT widen `is_secret_bearing`.** A parity test at `convert.rs:1696` locks `is_secret_bearing` ↔ `SECRET_NODE_TYPES` (the GUI-mirrored taxonomy); widening it would force MiniKey into `SECRET_NODE_TYPES` → GUI supply-chain-snapshot lockstep + masking changes. The call-site switch is non-lockstep.
- **Action:** switch `convert.rs:1042` + `:1069` to `is_argv_secret_bearing()`; update `tests/cli_convert_minikey.rs` fixtures (currently expect NO advisory for minikey). Cite SHA `f4d553e`. **SemVer PATCH** (behavior change: MiniKey now redacted + warns; no new flag). Toolkit-only.

### `secret-taxonomy-argv-superset-promotion` — ACCURATE, DRIFTED cite
- **WHAT:** add `pub const SECRET_NODE_TYPES_ARGV: &[&str]` to `secret_taxonomy.rs` (the public mirror of the wider `is_argv_secret_bearing` set = narrow set + minikey) + a parity test, so downstream consumers (GUI run-confirm redaction) don't scrape private symbols.
- **Citations:**
  - `secret_taxonomy.rs` — **ACCURATE:** has `SECRET_NODE_TYPES` (`:76`) + `SECRET_SLOT_SUBKEYS` (`:91`); **NO `_ARGV` yet**. Comment `:69-72` documents the MiniKey exclusion + references `convert-minikey-stdout-redaction`.
  - `convert.rs::is_argv_secret_bearing` line 107 — **DRIFTED → `:117`.**
  - Existing narrow-set parity test at `convert.rs:1696` (`SECRET_NODE_TYPES` ↔ `is_secret_bearing`) — the new test mirrors this shape for `SECRET_NODE_TYPES_ARGV` ↔ `is_argv_secret_bearing`.
- **Action:** add the const + parity test. Cite SHA `f4d553e`. **SemVer PATCH** (additive public const; existing GUI consumption of `SECRET_NODE_TYPES` unchanged → no forced GUI lockstep). **Couple with the minikey slug** — one cycle (the const is the principled home for the wide set the call-site switch relies on).

### `wallet-import-format-mismatch-matrix-completion-discovered-gaps` — ACCURATE (audit confirms 10 missing)
- **WHAT:** extend the 4 narrow dispatch arms so every `--format X` refuses every mismatching sniff outcome (full 8×7 = 56-cell off-diagonal).
- **Citations / audit (live, `import_wallet.rs` dispatch `match args.format` at `:473`):** off-diagonal refusal coverage —
  - `bitcoin-core` 7/7 ✓ · `bsms` 7/7 ✓ · `coldcard-multisig` 7/7 ✓ · `jade` 7/7 ✓
  - **`coldcard` 5/7 — MISSING: electrum, jade**
  - **`electrum` 6/7 — MISSING: jade**
  - **`sparrow` 3/7 — MISSING: coldcard, electrum, jade, specter**
  - **`specter` 4/7 — MISSING: coldcard, electrum, jade**
  - **Total = 2+1+4+3 = 10 missing arms — EXACTLY the slug's claim. ACCURATE.**
- **Action:** add the 10 `SniffOutcome::Y => return Err(ImportWalletFormatMismatch{supplied,sniffed})` arms to the 4 incomplete `Some("X")` blocks + ~10 integration cells. Cite SHA `f4d553e`. **SemVer PATCH** (more defensive refusals; no surface change). Toolkit-only test-hygiene. Mechanical — each new arm mirrors the existing complete arms exactly.

### `wallet-import-signet-regtest-disambiguation` — GENUINELY OPEN, needs a design decision
- **WHAT:** coin-type-1 collapses signet/regtest→testnet; add a way to recover signet/regtest semantics.
- **Citations:**
  - `wallet_import/bsms.rs:24-26` doc comment — **ACCURATE** (refreshed last cycle).
  - `SPEC_wallet_import_v0_26_0.md §4.2 step 8` — **ACCURATE** (`:163`): testnet-collapse normative text. **DESIGN TENSION:** the SPEC says users "must supply `--network signet|regtest` post-import **via a downstream subcommand**", whereas the FOLLOWUP option (a) says "a `--network` override **on `import-wallet`** (post-parse network re-binding)." Brainstorm must resolve flag-on-import-wallet vs downstream-subcommand.
  - No `--network` flag on `import-wallet` today — **confirmed** (`import-wallet --help`).
- **Design subtlety:** coin-type-1 is the only ambiguous case → a `--network` override should only be honored when the parsed network resolved to `Testnet` (refuse/ignore on a mainnet wallet, where coin-type-0 is unambiguous). Affects watch-only address encoding (xpub→address HRP).
- **Action:** brainstorm the flag-vs-subcommand decision (needs **user direction**) + the testnet-only-override guard. Cite SHA `f4d553e`. **SemVer PATCH** if an additive `import-wallet --network` flag → **mandatory GUI `schema_mirror` + manual lockstep** (new flag NAME).

### `argv-overwrite-after-parse` — GENUINELY OPEN, biggest + cross-repo + FFI
- **WHAT:** zero-overwrite secret bytes in `/proc/$PID/cmdline` after `clap::parse()` (or `prctl(PR_SET_DUMPABLE,0)`), closing the residual leak the Phase-1 advisory only warns about.
- **Citations:**
  - new module `crates/mnemonic-toolkit/src/argv_overwrite.rs` — **ACCURATE: does not exist** (confirmed).
  - Phase-1 advisory lives at `secret_advisory.rs:37` ("secret material on argv (…) — pipe via … to avoid /proc/$PID/cmdline exposure"); no mutation today — **ACCURATE.**
  - "Touches every binary entry-point (`mnemonic`, `md`, `mk`, `ms`)" — **cross-repo**: only `mnemonic` lives here; `md`/`ms`/`mk` are sibling repos (companion FOLLOWUPs required).
- **Action:** brainstorm the FFI approach (zero-overwrite the in-place `argv[][i]` byte ranges vs `PR_SET_DUMPABLE`), platform cfg-gating (Linux-only mutation; no-op elsewhere), and safety (the overwrite must run after clap has copied the values into owned Strings). Cite SHA `f4d553e`. **SemVer PATCH** (internal hardening, no surface change) but **cross-repo coordination** for the 4 binaries. Highest design + safety risk → LAST.

---

## Cross-cutting observations
1. **No stale-shipped surprises this batch** (unlike the BSMS cluster) — all 5 are genuinely open. Line cites drifted (convert.rs ~700-line drift since v0.9.0) but substance holds.
2. **The minikey pair (slugs 1+2) is one coupled cycle** — slug 1's call-site switch relies on the wide predicate that slug 2 promotes to a public const; do them together.
3. **The matrix audit is the recon's headline confirmation:** exactly the 10 pairs the slug claims, no more/no less — so the cycle is fully scoped before brainstorm (no further audit needed).
4. **Two slugs need a design decision before/during brainstorm:** signet (flag-on-import-wallet vs downstream-subcommand — SPEC vs FOLLOWUP tension; needs user direction) and argv-overwrite (FFI approach + cross-repo coordination).
5. **Lockstep map:** minikey pair = toolkit-only no-lockstep; matrix = toolkit-only no-lockstep; signet = GUI `schema_mirror` + manual lockstep (new flag NAME) IF flag-on-import-wallet; argv-overwrite = sibling-repo companion FOLLOWUPs (md/ms/mk) but no clap-flag lockstep.

---

## Recommended brainstorm-session scope (four series cycles — NO parallel code-gen)

| # | Cycle | Slugs | Size | SemVer | Lockstep | Risk |
|---|---|---|---|---|---|---|
| 1 | minikey-leak hardening | `convert-minikey-stdout-redaction` + `secret-taxonomy-argv-superset-promotion` | small (~2 call-site switches + 1 pub const + parity test + fixture updates) | PATCH | none | low |
| 2 | format-mismatch matrix | `wallet-import-format-mismatch-matrix-completion-discovered-gaps` | small-medium (10 arms + ~10 cells, mechanical) | PATCH | none | low |
| 3 | signet `--network` | `wallet-import-signet-regtest-disambiguation` | medium (flag + testnet-only guard + re-binding) | PATCH | GUI + manual | medium (design decision + user direction) |
| 4 | argv-overwrite | `argv-overwrite-after-parse` | large (FFI module + main.rs + 4-binary cross-repo) | PATCH | sibling-repo companions | high (FFI safety + cross-repo) |

**Order rationale:** ascending risk/coupling. 1 & 2 are clean toolkit-only PATCHes; 3 needs a user design call (flag vs subcommand) + lockstep; 4 is FFI + cross-repo, do last with the most care. Each is its own brainstorm → plan → opus R0 (0C/0I) → ship, in series. Per the no-parallel-codegen rule, do NOT dispatch concurrent implementer agents across cycles.
