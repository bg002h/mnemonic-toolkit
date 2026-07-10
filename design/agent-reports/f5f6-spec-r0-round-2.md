# SPEC R0 review — F5+F6 GUI recovery wiring — round 2 (convergence)

**Reviewer:** Fable (SPEC R0 round 2, read-only). SPEC @ GUI `f5cb11f` / toolkit `3d985798`. Round-1: `f5f6-spec-r0-round-1.md`.
**Dispatched:** 2026-07-10 (F5+F6, SPEC R0 round 2). Persisted verbatim per CLAUDE.md.

## VERDICT: GREEN — 0 Critical / 0 Important / 1 Minor. Implementation may begin; apply the one-string M-4 precision fold at fold-in — it does not gate.

## Round-1 folds — each verified resolved
**I-3 (persistence migration) — RESOLVED (all 4 sub-checks):**
- (a) `normalize_loaded_form_values` (`persistence.rs:370-401`) invoked from `load()` (`:342`), per-lookup fail-open; Text/Path scoping is a `match (&flag.kind, value)` arm (`:394-398`) → a Dropdown branch is clean. NOTE: the branch must be independent of the `flag.default_value else return true` gate (`:389-391`) since both targets are `default_value:None` (`schema/mnemonic.rs:3252,:3264`).
- (b) Reset-to-inference F6-consistent both variants: DROP → re-materializes `Dropdown(opts.first())` = `Dropdown("")` under `_INFER`; SET-`Dropdown("")` → `""` ∈ `_INFER`, displays `(none)`, emits nothing (`invocation.rs:~415` guard). Load-time transform before any render; idempotent.
- (c) Triple gating right: pins the OLD literals `"p2pkh"`/`"mainnet"` (NOT "current opts[0]" which is now `""` — the wrong-constant trap correctly sidestepped); deliberate `p2tr`/`testnet` preserved; one form × two flags.
- (d) **Persisted key = FLATTENED, confirmed:** `form_state_per_subcommand` keys are `"<cli>:<sub.name>"` (`persistence.rs:59-60`), `sub.name` = `"xpub-search-address-of-xpub"` (`schema/mnemonic.rs:4645`), unchanged by F5 → the at-risk key is **`"mnemonic:xpub-search-address-of-xpub"`**. The mandated post-load unit test structurally forces the right key (a wrong key fail-opens the schema lookup → stale KEPT → assertion reds).

**I-1 (test list) — RESOLVED, exhaustive.** Repo-wide grep = EXACTLY the 8: `xpub_search_widgets.rs:78,154,224,306` + `widget_interaction.rs:287,335,381,432` (slip39-split/combine, seed-xor-split/combine). Other `argv[1]` asserts are FLAT subs (`default_form_state.rs:68`, `argv_assembler.rs:308`) — unaffected.

**I-2 (snapshots) — RESOLVED.** `tests/gui_form_snapshots.rs`; exactly one `mnemonic-xpub-search-address-of-xpub.png` among 61; siblings render `--network` off the UNCHANGED shared `NETWORKS`+`Some("mainnet")` → no churn; tutorial snapshots render zero xpub-search forms; F5 churns no PNG.

**M-1/M-2/M-3 — captured.** §3 scopes choices-comparison to non-null-JSON-choices (the `--separator` GUI-Dropdown-over-toolkit-text divergence `schema/mnemonic.rs:326`); §1 has `*p` + the `invocation.rs:116` doc-invariant; G6 names `verify-bundle --template` (fail-SAFE, keyless-template short-circuits `verify_bundle.rs:379-400`).

## New-gap sweep — clean
I-3↔F6 consts consistent; migration runs before materialization; `ui_harness_i1_roundtrip.rs` dropdown cells don't touch address-of-xpub; `restore_templates_append_census` pins APPEND on restore/bundle only (PREPEND can't trip it); no test asserts virgin address-of-xpub materializes p2pkh/mainnet; I4 BIP-84 vector valid. Fully GUI-only; no toolkit change/pin; schema_mirror green; MINOR v0.58.0; §5 full required-context set.

## MINOR
- **M-4 (precision) — state the literal persisted key.** §2 (`:46`) and G7 (`:73`) phrase the gate generically; name the exact on-disk key **`"mnemonic:xpub-search-address-of-xpub"`** (flattened `sub.name`, unchanged by F5), and require the migration unit test's fixture to use that exact map key through `load()`. Self-correcting via the mandated test (wrong key → reds), hence Minor.

**Cleared:** SPEC R0-GREEN at round 2 — dispatch the implementer on the GREEN plan with M-4's literal-key fold applied; no further SPEC round required.
