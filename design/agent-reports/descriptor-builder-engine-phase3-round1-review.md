<!-- VERBATIM opus-architect Phase-3 per-phase review, round 1, descriptor-builder emit + CLI. Persisted per CLAUDE.md. @ pre-I1-fold; source b596d3f. Verdict: RED 0C/1I/1m — I1 watch-only no-leak test missing. -->

# Phase-3 Review — descriptor-builder engine — **RED** (1 Important)

The engine is funds-safe on the property that matters most — **emit never outputs an ungated descriptor** — and all four Phase-3 decisions are sound and faithfully implemented. But one funds-safety invariant the SPEC explicitly claims (**watch-only-out / no-secrets**) is neither enforced nor tested, and its correctness currently rests on the error-Display format of an unpinned upstream miniscript git-rev that this very feature bumps.

## CRITICAL
None.

## IMPORTANT

### I1 — `build-descriptor` is SPEC-declared watch-only-out, but secret input is neither refused nor tested; the no-leak property is load-bearing on an unpinned upstream git-rev. (confidence 85)
- SPEC §0: "Watch-only-out (the builder takes cosigner XPUBs — no secrets)." Yet `Pk(String)`/`Pkh(String)` are opaque, `validate_fields` does nothing for `Pk`/`Pkh`, and there is no secret-shape screen. A spec with `{"pk":"<xprv>"}`/`{"pk":"<WIF>"}` reaches step 2 unscreened.
- It IS refused (exit 2): `from_str::<DescriptorPublicKey>` cannot parse a secret → `TypeError`. So emit never sees ungated input — focus-area-1 holds.
- The gap is the LEAK surface + missing test. Both diagnostic sites format the raw upstream error into the user-facing message (`build_descriptor.rs:79-84` → `gate.rs:107-109` → `gate.rs:272` `format!("miniscript type/parse error: {top_err}")`) → stderr (human) + `--json` `diagnostics[].message`. At `95fdd1c` a structurally-valid `pk(<xprv>)`/`pk(<WIF>)` routes to the non-echoing key-error path, so it very likely does NOT leak today. But this feature exists because the rev gets bumped; a future rev whose Display echoes the token (the top-level `Error` already has payload-echoing arms `Unexpected(String)`/`NonTopLevel(String)`/`Trailing(String)`) would silently start emitting key material, and nothing in the suite would catch it.
- The rest of the codebase treats no-secret-leak as a TESTED invariant (`cli_restore.rs` `!contains("tprv")`, `ImportWalletXprvForbidden`); this subcommand is the unguarded exception.

**Fix:** add an integration test feeding a secret in a key node (`pk(<xprv>)`, `pk(<WIF>)`, ideally `multi(2,<xprv>,…)`) asserting (a) exit 2 and (b) neither stderr nor `--json` `diagnostics[].message` contains an `xprv`/`tprv`/WIF substring. If it passes → invariant pinned against future rev bumps → GREEN. If it reveals an echo → stop formatting raw `top_err` (`gate.rs:109`/`:272`) or refuse secret-shaped keys at the IR boundary (`ExportWalletSecretInput` precedent).

## MINOR
**M1 — `emit_human` writes the descriptor to stdout before the `?`-propagating cost preview** (`build_descriptor.rs:195` then `:207-216`): a cost-preview error → partial stdout + stderr error + exit 2. Cap-agreement (`DEFAULT_PREVIEW_CAP` == compare-cost default, pinned) makes this practically unreachable for gate-passing input. One-line note, not a fix.

## What passes
- **Emit-gating holds.** `run()` emits a descriptor/bip388 only on `gate::validate`'s `Ok` arm; the only pre-gate short-circuit is `--spec-schema` (static grammar, no descriptor).
- **Decision 1 (`--network` optional, default mainnet) sound** — output network-agnostic; avoids a GUI conditional-rule cross-repo arc; "mainnet descriptor from tpub" not weakened (this command emits no network claim).
- **Decision 2 (output matrix) coherent** — `--json` overrides `--format`; bare vs human; first address best-effort.
- **Decision 3 (cost projection + buffer re-parse) correct** — `into_single_descriptors()[0]` (I2); boundary-agreement pins gate-raw == enumerate-raw so the preview never trips `ConditionsTooMany` on gate-passing input.
- **Decision 4 (exit codes) consistent** — gate diags → Ok(2); spec parse/IO → `BuildDescriptorSpec` exit 2; no attacker-input panic.
- **Goldens trustworthy + non-vacuous** (checksums, multipath/`@N/**`, origins `h`→`'`); `negative_discrimination_mutated_threshold_breaks_golden` genuinely discriminates.
- **bip388 round-trip genuine** (built descriptor → `export-wallet --descriptor --format bip388` == build-descriptor bip388; both via `descriptor_to_bip388_wallet_policy`).
- **Dead-code removals clean** (`ValidatedPolicy.rendered`, `ir::children()` — gate uses its own `child_paths`).
- **gui-schema (this repo) current** (subcommand list 29→30; flags auto-derived from clap). GUI-repo `schema_mirror` + manual mirror remain the separate ship-lockstep.
- **error.rs correct** (`BuildDescriptorSpec` alphabetical, exit 2, kind/Display; no `details` arm needed).

**RED scope is exactly I1.** Once it folds to GREEN, Phase 3 is done; only the ship-lockstep (manual + GUI schema-mirror + version bump v0.50.0 + tag) remains.
