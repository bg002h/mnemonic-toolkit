# #25 Phase 2 (toolkit) execution review — per-key use-site override (opus architect, verbatim)

> Mandatory per-phase R0 over the IMPLEMENTED diff (`mnemonic-toolkit` `feature/per-key-use-site-override`, commits `fbfc76f`+`8695219` off `master`==`cfe411e`). Reviewer ran suite+clippy + a LIVE re-bundle experiment to scrutinize deviation #3. **Verdict: GREEN — 0 Critical, 0 Important.** 2 non-blocking Minors. Cleared for P2.6 (version + manual + ship 0.58.2).

---

**Verdict: GREEN — 0 Critical, 0 Important.**

## Verified correct

**Regression sweep (focus 7).**
- `cargo test -p mnemonic-toolkit` → **3111 passed / 0 failed / 13 ignored** (matches expected). `cargo clippy -p mnemonic-toolkit --tests` → clean.
- `mlock.rs` UNTOUCHED: `git diff master..HEAD -- …/mlock.rs` empty (g6 holds). `cargo fmt --check` diff ONLY in mlock.rs (the exempt file) — implementer correctly left it unformatted, touched nothing else.
- No `.unwrap()`/`.expect()`/`panic!`/index-access ADDED in restore.rs hunks. The two `unreachable!()` (:315,:1822) are pre-existing seed-node guards off the override path.

**Focus 1 — `ReconstructTranslator` reduction (restore.rs:1007-1072).** (a) Network correction applies to BOTH `MultiXPub` (:1055-1058) and `XPub` (:1060-1063) via `self.network.network_kind()`; live testnet test passes (`tpub`+`tb1`). (b) Strict-NUMS refusal intact (:1038-1045): H-point x-only pass-through, any other `Single`→`Err`. (c) Divergent suffix no longer clobbered: `multipath` field dropped; translator passes the group through unchanged; suffix comes from `to_miniscript_descriptor_multipath` (md-codec to_miniscript.rs:178-213). Live restore emits `@0…/<0;1>/*` AND `@1…/<2;3>/*`. (d) "cannot wrap" hint preserved (:1106-1111).

**Focus 2 — Deviation #3 (suspected hidden fidelity gap). NOT a gap — validated by live experiment.** Reconstructed divergent descriptor (live): `wsh(multi(2,[73c5da0a]xpub…/<0;1>/*,[b8688df1]xpub…/<2;3>/*))` — fingerprint-only origins as claimed. The missing origin PATH is a REAL pre-existing `bundle` limitation, NOT a regression: BOTH divergent AND all-baseline concrete reconstructions fail to re-bundle with the IDENTICAL error (`concrete descriptors must carry a key origin…`); bundle/wallet_export/import paths UNTOUCHED by this diff. Origin omission originates in md-codec's override-TLV encode (carries no per-key origin), so `assert_md1_fixed_point` is genuinely INAPPLICABLE for override cards. The funds-relevant SUFFIX is faithfully reconstructed (`<2;3>` visible) and the swapped oracle (address-equivalence + suffix-string + independent golden) is STRICTLY STRONGER than fixed-point for the funds-safety property. No real coverage lost.

**Focus 3 — P2.1 C1 routing (restore.rs:1278-1295).** `else if d.tlv.use_site_path_overrides.is_some() → (None,None)` (:1285-1292) reached ONLY by non-taproot override cards (taproot+override pre-refused at :1251 before classify). No over-route (non-override → `plain_template_from_tree` :1294 unchanged); no under-route (bare `sh(multi)` M1 test 11d, `sh(wsh(multi))` M2 test 11c both reach the faithful arm, passing).

**Focus 4 — P2.3 guard + P2.4 advisory parity (single source).** `taproot_override_card(d)` (restore.rs:1082, `pub(crate)`) used by guard (:1251) AND advisory (unrestorable_advisory.rs:104). `has_hardened_use_site` (md-codec to_miniscript.rs:89, scans baseline AND every override) shared by guard (:1244) + advisory (:90). Guard-refuses ⟺ advisory-fires (parity tests pass). Flipped pin (`per_key_use_site_override_refused`→faithful) correct. Baseline `/*h` (test 12) + override-hardened-`/*h` (test 11f, 2b) still refuse.

**Focus 5 — independent golden (gate, SPEC I1).** `assert_divergent_address_independent_golden` (cli_restore_multisig_general.rs:581) derives via rust-miniscript `into_single_descriptors` on the RECONSTRUCTED string (no re-entry into md-codec reconstruction) + a hand-baked const; anti-vacuity `divergent_golden_differs_from_baseline_and_anchors` (:614, NOT ignored, default CI) asserts `divergent != baseline`. `divergent_differential_golden` (bitcoind_differential.rs:468, NOT ignored) asserts `divergent == pinned golden` AND `!= all-baseline` — ran standalone, passes.

**Focus 6 — Step 0 error mapping.** `BaselineUseSiteOverride`/`RedundantUseSiteOverride` → exit 2 (error.rs:464-465, decode-reject group terminating `=> 2` at :497) — correct class. Friendly messages friendly.rs:262-269 (explicit arms, no swallowing wildcard in the md1 region). No `todo!()`/`unreachable!()` on the override path.

**Focus 8 — plan fidelity.** All P2.1-P2.5 present + faithful. Deviation #2 (override-hardened-ALT → override-hardened-WILDCARD `<2;3>/*h`): acceptable, both route through `has_hardened_use_site`→loud refuse. md-codec 0.37.0 API consumed correctly; Cargo.lock pins exactly one md-codec 0.37.0 (registry, no path-patch leak) → CI builds against the published crate.

## CRITICAL
None.

## IMPORTANT
None.

## MINOR
1. **P2.6 not done (in-scope-deferred).** Version still `0.58.1`; READMEs/install.sh/CHANGELOG/manual not updated. Expected (ship deferred). Version-site checklist for P2.6: Cargo.toml + BOTH READMEs + install.sh (self-pin + the md-cli sibling pin M4) + fuzz/Cargo.lock + main Cargo.lock + CHANGELOG + manual `### Unrestorable descriptor shapes` prose → `make -C docs/manual audit`. The load-bearing dep pin (`md-codec="0.37"`) is already in place.
2. **`taproot_override_card` lives in `cmd/restore.rs`, reached cross-module by `unrestorable_advisory.rs` via `crate::cmd::restore::taproot_override_card`.** Slightly unusual coupling direction (advisory→cmd) but the intended single-source design (M3), `pub(crate)`. No action; noted for future readers.

## To turn GREEN
Already GREEN. No changes required. Carry the MINOR-1 version-site checklist into P2.6 ship.
