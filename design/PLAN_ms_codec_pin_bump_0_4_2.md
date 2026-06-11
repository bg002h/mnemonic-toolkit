# PLAN — toolkit v0.53.5: bump ms-codec pin 0.4.0 → 0.4.2 (uppercase ms1 now decodes; inherit combine fixes)

**Cycle:** toolkit v0.53.5 (PATCH — advisor at tag) · **Source SHA:** `6101fe0` (= v0.53.4) · **Resolves:** `hrp-classifier-rejects-valid-uppercase-cards`'s ms1 leg (audit M11, the part v0.53.3 couldn't close toolkit-side) + `toolkit-ms-codec-pin-bump-0-4-1-combine-fix`.
**Precondition met:** ms-codec 0.4.1 + 0.4.2 published to crates.io (2026-06-10); toolkit resolves 0.4.0 → 0.4.2.

## What the bump brings (empirically measured, not predicted)

Applied `cargo update -p ms-codec` (lock 0.4.0 → 0.4.2) + ran the full suite. **Exactly 3 cells flip RED** (`cli_hrp_case_insensitive.rs`), all the staged uppercase-ms1 attribution cells that asserted the now-obsolete `WrongHrp` error — uppercase ms1 now DECODES (the uppercase twin of the lowercase card decodes identically). Nothing else in the workspace broke. The mk1/md1 uppercase cells + the mixed-case-rejects cells are unaffected.

## The 3 cell inversions (assertions + doc-comments)

The cards used are the uppercase twin `MS10ENTRSQQ…34V7F` of the lowercase TREZOR-12-zero ms1:

1. **`inspect_positional_uppercase_ms1_advisory_fires_ms_codec_attributed_no_echo`** — now exit 0 with `kind: ms1` / `tag: entr` (valid inspect report). Invert to: exit 0, stdout contains `kind: ms1` + `tag: entr`, the positional secret-argv advisory STILL fires on stderr, and the raw card is NOT echoed as an error (the report shows decoded fields, not the input dump). Rename off `..._ms_codec_attributed_no_echo` to reflect decode-success.
2. **`repair_ms1_flag_uppercase_reaches_ms_codec_repair_marker`** — now exit 0, stdout = the card passed through (no correction needed for a valid card), stderr = the `--ms1` secret-argv advisory. Invert to: exit 0, stdout contains the card, advisory fires. (Characterization note: `repair` echoes a clean card unchanged in its input case — corrected cards re-emit lowercase, clean cards pass through; pre-existing codec behavior, not this cycle's concern.)
3. **`silent_payment_uppercase_ms1_ms_codec_attributed`** — now exit 0, derives a silent-payment address. Invert to: exit 0, and ASSERT THE `sp1q…` ADDRESS EQUALS THE LOWERCASE TWIN'S (the real correctness pin — uppercase and lowercase cards must derive the same wallet; capture the lowercase output in the same cell or pin the literal `sp1q…` from the run).

Rewrite each cell's doc-comment (they say "uppercase ms1 cannot decode until the ms-codec envelope companion ships" — it shipped, ms-codec 0.4.2). **Repair cell removes a DISTINCT string (R0-r1 M2):** `repair: chunk 0 HRP mismatch — expected 'ms', found 'MS'` (`RepairError::HrpMismatch`), not the WrongHrp string — don't grep only for WrongHrp.

**4th cell — FALSE-BUT-GREEN, also invert (R0-r1 I1):** `verify_bundle_positional_uppercase_ms1_no_echo_ms_codec_attributed` (cli_hrp_case_insensitive.rs:327-353) STAYS green post-bump but for the wrong reason — uppercase ms1 now `ms1_decode: ok`; the exit-4 failure comes entirely from absent mk1/md1, not any ms1 error. Its name/doc(:324-326)/assertion-comment(:347) assert a now-false premise. Rewrite name + doc, or convert to a positive `ms1_decode: ok` assertion for the uppercase positional.

**Module-level doc-comment (R0-r1 I2):** the file header `cli_hrp_case_insensitive.rs:1-27` (esp. :5-6 "ms-codec 0.4.0's envelope layer rejects uppercase" + :12-13 "uppercase MS1 positional ... ms-codec-attributed error (WrongHrp 'MS')") is now FALSE — add it to the rewrite scope.

## ms-shares combine — REQUIRED security characterization (R0-r1 C1) + optional length

**REQUIRED (the 0.4.2 security guard the toolkit ships):** the toolkit's `ms-shares combine` delegates to `ms_codec::combine_shares` (`cmd/ms_shares.rs:385`), so PRE-bump `mnemonic ms-shares combine --share <UPPER secret-at-S> --to entropy` would have LEAKED the secret (the raw `b's'` guard missed `b'S'` → interpolation short-circuit returned it). Add a RED-FIRST cell pinning the consumer-side refusal: exit 2 + the `SecretShareSuppliedToCombine` prose + NO secret bytes on stdout. Fixture: the existing `VALID_MS1` uppercased IS a secret-at-S card (single-string ms1 = threshold-0/index-s). This is the toolkit-side proof of a shipped security fix — NOT optional.
**Optional (the 0.4.1 length fix):** a non-standard-length clean-reject cell — coverage already lives ms-codec-side; add only if a valid-checksum non-standard-length fixture is cheap, else skip (no fixture scope-creep).

## Ritual

- Pin: `crates/mnemonic-toolkit/Cargo.toml:20` `ms-codec = "0.4.0"` → `"0.4.2"` (documents intent; lock already bumped). The exact-pin comment at :23 (re the ms-cli transitive pin) stays accurate.
- Version ×4 + `scripts/install.sh:32`: Cargo.toml:3, CHANGELOG, README.md:13 + crates/mnemonic-toolkit/README.md markers, install.sh self-pin → v0.53.5; Cargo.lock follows.
- **Manual:** the v0.53.3 case-tolerance note (`docs/manual/src/40-cli-reference/41-mnemonic.md`, inspect section) says "`mk1`/`md1` decode end-to-end" — UPDATE to include `ms1` (now that the codec accepts it). Full manual lint (all 4 BINs).
- FOLLOWUPS: resolve `toolkit-ms-codec-pin-bump-0-4-1-combine-fix`; update `hrp-classifier-rejects-valid-uppercase-cards` (ms1 leg now CLOSED — uppercase ms1 decodes end-to-end); the mirrored ms-codec companion notes "consumed by toolkit v0.53.5".
- **SemVer:** advisor at tag. Leaning PATCH (uppercase ms1 was always spec-valid; this completes a conformance fix + inherits the combine-panic fix + the combine secret-guard fix — same family as v0.53.3's PATCH), but PATCH is STRONGLY favored (R0-r1 verified): v0.53.3 shipped the mk1/md1 legs as PATCH and explicitly deferred the ms1 leg to "the toolkit pin bump"; ms-codec classified 0.4.1+0.4.2 PATCH; uppercase ms1 was always spec-valid + merely misattributed (weaker than v0.49.0’s genuinely-new format). Advisor confirms at tag.
- Watch (R0-r1 M1): **rust.yml** (the inversions) + **manual.yml** (the manual edit, master-push `docs/manual/**`) + **technical-manual.yml** on the MASTER PUSH; then **changelog-check + install-pin-check** (the 2 toolkit-tag workflows) on the tag.

## Non-goals

repair clean-card case-canonicalization (separate question); any ms-codec change (shipped); `docs/manual/.../43-ms.md:321`’s `ms-cli v0.4.0` ref (sibling CLI version, NOT ms-codec — leave it; R0-r1 M3); the deferred audit [obs] items.
