# PLAN R0 review — Cycle H F3 network fail-open — round 1

**Reviewer:** Fable (plan R0, read-only), per user directive. Plan @ `713484c3` vs live source.
**Dispatched:** 2026-07-09 (Cycle H, plan-R0 round 1). Persisted verbatim per CLAUDE.md.

**VERDICT: GREEN — 0 Critical / 0 Important / 3 Minor.** Implementation may begin. All verification performed live at `713484c3`.

## 1. Phase split — single P0 is correct
Verified all five edge sites live: E1 `convert.rs:1524-1526`, E2 `address_of_xpub.rs:215-217`, E3 `silent_payment.rs:125-135`, E4 `export_wallet.rs` (`resolved_slots_ref` ~:625-631, `EmitInputs` :674, `master_xpub_at_0` `Option<Xpub>` :672), E5 `bsms.rs` FourLine (`parsed` ~:104-110 → `derive_first_address` :113). Five distinct files, one reused error variant (`NetworkMismatch`, exit 2 `error.rs:621`), zero shared-type change. `tests/cli_network_fail_open.rs` does not yet exist. Swept existing tests: `cli_wallet_cross_format_convergence.rs:625` + `cli_older_advisory.rs:171` use tpub via `bundle --network testnet` (agreeing); `cli_xpub_search_address_of_xpub.rs:542-567` pins tpub + `--network signet` = Test agreement (survives; the signet-ACCEPT cell re-pins). No suite collision.

## 2. Release ritual — version-site list COMPLETE
Confirmed at `713484c3`: `Cargo.toml:3`, `Cargo.lock:731`, `fuzz/Cargo.lock:579`, `README.md:13` + `crates/mnemonic-toolkit/README.md:9`, `scripts/install.sh:32` (install-pin-check greps exactly that pin), `.examples-build/gen.sh` = **exactly 6** occurrences (3/44/109/126/711/724), `Examples.md` = 8 (regen). Repo-wide `0.82.0` grep finds nothing else outside design docs. `changelog-check` fires on the tag, needs `[0.83.0]`. No sibling-pin / re-vendor / publish.

## 3. examples-gate — trigger confirmed
`examples.yml` triggers on `crates/**`/`Cargo.*`/`scripts/install.sh`/`.examples-build/**`; gen.sh:44 FATALs unless `--version` == `mnemonic 0.82.0`. gen.sh's three `--format bsms` runs use mainnet + `--network mainnet` (E5 no-op); 0 tpub/tprv hits in the corpus → pure version diff.

## 4. Manual lockstep — right file, no gate breakage
`41-mnemonic.md`: convert :873, export-wallet :924, silent-payment :2808, xpub-search :3548. No clap change → no flag-table edit → `lint.sh` unaffected. Zero testnet extended-key strings under `docs/manual/` or `.examples-build/`; nothing regens for a content diff.

## 5. FOLLOWUPs — adequate
No existing F3/fail-open slug in `design/FOLLOWUPS.md` → nothing to flip; the 2 new slugs + eval-F3-RESOLVED note are complete.

## 6. Guard-rails — sufficient
G-A matches the `network.rs:88-93` precondition with the right non-regression pins; G-D `xkey_network` confirmed `key.rs:1043`. G-B/G-C/G-E sound.

## Minor findings (folded)
- **M1** — CI-verify list under-enumerates: the tag also fires `man-pages` (→ `reproducible-musl-build` asset leg, the v0.74.0-recut culprit), `vendor-freshness`, `fuzz-smoke`, `gui-pin-drift-check` (warn-only), `manual` (the 41-mnemonic.md edit). None can plausibly red, but "verify CI" should watch the full set.
- **M2** — regen ordering: `Examples.md:102/115/116` embed install.sh's self-pin via a live dry-run transcript → gen.sh regen must run AFTER the install.sh + gen.sh bumps in the same tree ("regen last").
- **M3** — GUI companion: CLAUDE.md cross-repo convention wants companion FOLLOWUP entries in BOTH repos with cross-citing `Companion:` lines; add the toolkit-side mirror entry for the deferred GUI dropdown-default fix.

**Gate: plan-R0 GREEN — implementation may begin.**
