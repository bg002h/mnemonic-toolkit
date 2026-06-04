# export-wallet --format descriptor — Phase 2 R0 Review

**Verdict (round 0): RED (0C / 1I)** → **GREEN after I1 fold + runtime ledger (0C / 0I).**

Phase 2 = docs (manual round-trip recipe + `--format` value list) + the v0.42.0 version bump. The opus architect reviewer's harness had **no shell**, so it completed the full static cross-reference and explicitly deferred the runtime ledger (recipe-command execution, `make audit` EXIT, `readme_version_current`, FAILED-count, clippy) to a reviewer with shell access. I (controller) completed that runtime ledger independently — results appended below. The single Important (I1) was a stale prose `Status:` line; folded.

---

## Important (1) — FOLDED

**I1 — Stale `Status: **v0.40.x**` prose line in both READMEs shipped on a v0.42.0 release commit.**
- `README.md:14` and `crates/mnemonic-toolkit/README.md:10` read `Status: **v0.40.x** — twenty-two mnemonic subcommands …`. NOT gated by `readme_version_current` (that test checks only the `<!-- toolkit-version: -->` marker, correctly at `0.42.0` in both). On a release commit whose purpose is the version bump, a human-visible `v0.40.x` is exactly the silent-decay class the readme guard exists to kill — just in the ungated sibling string.
- **Fold:** bumped both `Status: **v0.40.x**` → `**v0.42.x**`. Subcommand count unchanged (this cycle added a `--format` VALUE to an existing subcommand, not a subcommand), so "twenty-two" is left as-is. `readme_version_current` re-run after the edit: PASS (it checks the marker, not the Status line).

## Minor (2) — accepted, not folded

- **M1** The new round-trip recipe is prose-only; `make audit`'s `verify-examples` step replays only committed `docs/manual/transcripts/**/*.cmd`, so it does not exercise the recipe. Accepted: manual-prose-execution is discipline, not a per-section hard requirement; the controller ran every recipe command end-to-end (ledger below), which is the substantive guard.
- **M2** Recipe OUT examples elide the xpub and use `--blob wallet.json` (illustrative) where the real artifact is the fixture. Consistent with the rest of the chapter; left as-is.

## Verification ledger — runtime (controller, shell access) — ALL GREEN

Built `target/debug/mnemonic` (v0.42.0) + the 3 sibling CLIs. Test seed = public abandon×11+about vector.

- **single-sig recipe** (`37-wallet-export.md:261-283`): `mfp=73c5da0a` ✓; output `wpkh([73c5da0a/84'/0'/0']xpub6CatWdiZiodmU…/<0;1>/*)#hpg6d6w2` — checksum `#hpg6d6w2` matches prose `:282` EXACTLY.
- **multisig round-trip** (`:290-300`): via `crates/mnemonic-toolkit/tests/fixtures/wallet_import/sparrow-multisig-2of3-p2wsh-sortedmulti.json` → `wsh(sortedmulti(2,[b8688df1/48'/0'/0'/2']xpub6FQya…,[28645006/…]xpub6DnEB…,[5436d724/…]xpub6Buxw…))#he0ej3xr` — checksum `#he0ej3xr` AND cosigner order `b8688df1,28645006,5436d724` both match prose `:299` EXACTLY. (This was the reviewer's highest-risk unverified claim — the `sortedmulti` Display canonicalization preserves the printed order; CONFIRMED.)
- **taproot passthrough** (`:311-315`): `tr([73c5da0a/86'/0'/0']xpub6CatW…/<0;1>/*)#5tp3cj93` — emits (exit 0). Prose claims no specific checksum, only that the passthrough door works; CONFIRMED.
- **IN bundle --descriptor** (`:236-241`): emits `md1` template card + 2× `mk1` cosigner cards, no `ms1` (watch-only), exit 0. CONFIRMED.
- **`make -C docs/manual audit` (4 BIN + FIXTURES_DIR set):** literal **EXIT=0** — verify-examples 20 transcripts pass, markdownlint 0 errors, cspell 0 issues, lychee 172 OK / 0 errors, flag-coverage OK, anchor-check baseline. CONFIRMED.
- **`readme_version_current`:** PASS (post Status-line edit). CONFIRMED.
- **`cargo test -p mnemonic-toolkit --no-fail-fast | grep -cE '^test .* FAILED'`:** `0`. CONFIRMED.
- **`cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings`:** exit 0. CONFIRMED.

## Verification ledger — static (architect)

- Version bump complete at 0.42.0: `Cargo.toml:3`, `Cargo.lock` mnemonic-toolkit entry, both README markers (`README.md:13`, crate `README.md:9`), `scripts/install.sh:32`, `CHANGELOG.md` v0.42.0 entry (dated, SemVer-MINOR, describes `--format descriptor`).
- `41-mnemonic.md:700`: `descriptor` added to export-wallet `--format` list, description accurate.
- Taproot-via-`--from-import-json` refusal TRUE (`export_wallet.rs:677-687`); direct `--descriptor 'tr(...)'` passthrough needs no `--taproot-internal-key` for key-path-only. `descriptor` vs `green` distinction TRUE (`green.rs:36-40` refuses multisig; `descriptor.rs` has empty collect_missing + no refusal). md1-keyless framing accurate. `[00000000/…]` default-origin TRUE (`bundle.rs:562` `Fingerprint::default()`).
- Lockstep: NO `mnemonic-gui` file touched (paired GUI v0.23.0 deferred to a separate post-tag mini-cycle). Manual `--format` value-list update is the correct mirror discipline (flag-coverage lint is flag-NAME-only).

## Verdict rationale

The only finding was I1 (a 2-word prose-version decay, not gated, no code/command impact); folded. Every runtime claim the reviewer could not execute is now CONFIRMED green by the controller. The re-dispatch obligation after the I1 fold is satisfied by the upcoming **end-of-cycle R0 over `master..HEAD`**, which re-reviews the complete diff including this fold (a stronger check than re-running Phase 2 R0 on a 2-word prose edit). Phase 2 GREEN.
